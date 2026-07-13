//! `care.menu.set` — admin/staff upsert of ONE `(date, room, slot)` menu cell.
//!
//! A `set` writes the food PLAN for one slot: the items (each with allergen
//! TAGS) and the per-restriction substitute table. It is an UPSERT on the
//! natural key (`Menu::id` = `<date>::<room>::<slot>`) — re-planning a slot
//! overwrites the cell, so `set` is safely re-plannable and a copy-week reuse
//! is idempotent (menus-scope §"Derivation, not entry"). Contrast `room.create`
//! (first-write, conflict on dup): a menu cell has no create/update split.
//!
//! ## Who may write
//!
//! Admin OR staff. A guardian NEVER holds `mcp:care.menu.set:call`, so the host
//! capability wall blocks them before this verb runs — a `Member` who reaches
//! this body holds the cap and is staff by definition (menus-scope §"Who
//! enters"). We therefore need no extra role gate beyond the admin-audit hop:
//! we only branch admin-vs-staff to route the audit through the chokepoint when
//! the writer is an admin.
//!
//! ## Safety posture (menus-scope §"Risks")
//!
//! Allergen tags NEVER reject — `Allergen::parse` folds any unknown label to
//! `Other(..)` so a food-safety tag is always recorded (garbage flags
//! conservatively, never drops). Only the plan KEYS are hard-validated: a bad
//! `date` or an unknown `slot` fails loud so a malformed key never fragments a
//! room's week.

use lb_auth::Principal;

use crate::authz::{assert_reach, Chokepoint, RecordError};
use crate::center::Locale;
use crate::i18n::t;
use crate::menu::{validate_date, Allergen, Menu, MenuError, MenuItem, Slot, Substitution};

/// One planned item on the wire — allergens are free-text tags parsed into the
/// fixed enum (never rejected).
#[derive(Debug, serde::Deserialize)]
pub struct ItemInput {
    pub name: String,
    #[serde(default)]
    pub allergens: Vec<String>,
}

/// One substitute-table entry on the wire — `restriction` is a free-text
/// allergen tag (parsed, never rejected).
#[derive(Debug, serde::Deserialize)]
pub struct SubstitutionInput {
    pub restriction: String,
    pub substitute: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SetInput {
    pub date: String,
    pub room_id: String,
    pub slot: String,
    #[serde(default)]
    pub items: Vec<ItemInput>,
    #[serde(default)]
    pub substitutions: Vec<SubstitutionInput>,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct SetReply {
    pub id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: SetInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.menu.set input: {e}"))?;

    // Hard-validate the plan KEYS (date, slot, room) — a malformed key must
    // never fragment a room's week (menus-scope §"Safety surface").
    validate_date(&parsed.date).map_err(|e| format!("{e}"))?;

    let slot = Slot::parse(&parsed.slot)
        .ok_or_else(|| format!("{}", MenuError::InvalidSlot(parsed.slot.clone())))?;

    if parsed.room_id.trim().is_empty() {
        return Err(format!("{}", MenuError::MissingField("room_id")));
    }

    // Items: name is required; allergen tags fold to the enum (never reject).
    let mut items = Vec::with_capacity(parsed.items.len());
    for item in &parsed.items {
        if item.name.trim().is_empty() {
            return Err(format!("{}", MenuError::MissingField("item name")));
        }
        items.push(MenuItem {
            name: item.name.clone(),
            allergens: item.allergens.iter().map(|a| Allergen::parse(a)).collect(),
        });
    }

    let substitutions = parsed
        .substitutions
        .iter()
        .map(|s| Substitution {
            restriction: Allergen::parse(&s.restriction),
            substitute: s.substitute.clone(),
        })
        .collect();

    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    let menu = Menu {
        date: parsed.date.clone(),
        room_id: parsed.room_id.clone(),
        slot,
        items,
        substitutions,
    };
    let id = Menu::id(&menu.date, &menu.room_id, slot);
    let value = serde_json::to_value(&menu).map_err(|e| format!("serialize menu: {e}"))?;

    // UPSERT — `set` is re-plannable (copy-week idempotent), so `write`, not
    // `create`: a second set on the same cell overwrites, never conflicts.
    cp.records()
        .write("menu", &id, &value)
        .await
        .map_err(|e| match e {
            RecordError::Conflict => format!("{}", MenuError::StoreDenied("set (conflict)".into())),
            RecordError::Store(s) => format!("{}: {s}", MenuError::StoreDenied("set".into())),
        })?;

    // Admin audit through the chokepoint (one audit point) when the writer is
    // an admin; a staff Member holds the cap and needs no reach hop.
    if principal.role() == lb_auth::Role::WorkspaceAdmin
        || principal.role() == lb_auth::Role::SuperAdmin
    {
        let _ = assert_reach(cp, principal, &menu.room_id).await;
    }

    let reply = SetReply {
        message: t(locale, "menu.saved", &[("slot", slot.key()), ("date", &menu.date)]),
        id,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{read, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.menu.set:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn set_writes_a_menu_cell_that_round_trips() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        let out = run(
            &cp,
            &p,
            r#"{
                "date":"2026-07-14",
                "room_id":"room:possums",
                "slot":"lunch",
                "items":[{"name":"Peanut satay","allergens":["peanut","dairy"]}],
                "substitutions":[{"restriction":"peanut","substitute":"Sunflower satay"}]
            }"#,
        )
        .await
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["id"], "2026-07-14::room:possums::lunch");

        let row = read(&store, "acme", "menu", "2026-07-14::room:possums::lunch")
            .await
            .unwrap()
            .unwrap();
        let back: Menu = serde_json::from_value(row).unwrap();
        assert_eq!(back.items.len(), 1);
        assert_eq!(back.items[0].name, "Peanut satay");
        assert_eq!(
            back.items[0].allergens,
            vec![Allergen::Peanut, Allergen::Milk]
        );
        assert_eq!(back.substitutions[0].restriction, Allergen::Peanut);
    }

    #[tokio::test]
    async fn set_is_an_upsert_second_write_overwrites_no_conflict() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        let first = r#"{"date":"2026-07-14","room_id":"room:possums","slot":"lunch",
            "items":[{"name":"Rice","allergens":[]}],"substitutions":[]}"#;
        run(&cp, &p, first).await.expect("first set");

        let second = r#"{"date":"2026-07-14","room_id":"room:possums","slot":"lunch",
            "items":[{"name":"Pasta","allergens":["wheat"]}],"substitutions":[]}"#;
        run(&cp, &p, second).await.expect("second set overwrites");

        let row = read(&store, "acme", "menu", "2026-07-14::room:possums::lunch")
            .await
            .unwrap()
            .unwrap();
        let back: Menu = serde_json::from_value(row).unwrap();
        assert_eq!(back.items.len(), 1);
        assert_eq!(back.items[0].name, "Pasta");
        assert_eq!(back.items[0].allergens, vec![Allergen::Wheat]);
    }

    #[tokio::test]
    async fn set_rejects_a_bad_date() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let res = run(
            &cp,
            &p,
            r#"{"date":"2026-13-40","room_id":"room:possums","slot":"lunch","items":[],"substitutions":[]}"#,
        )
        .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("invalid date"));
    }

    #[tokio::test]
    async fn set_rejects_an_invalid_slot() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let res = run(
            &cp,
            &p,
            r#"{"date":"2026-07-14","room_id":"room:possums","slot":"brunch","items":[],"substitutions":[]}"#,
        )
        .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("invalid slot"));
    }
}
