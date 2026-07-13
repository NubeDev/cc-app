//! `care.menu.get` — read ONE `(date, room, slot)` menu cell.
//!
//! A menu is a ROOM plan, not a child record — so this read is ROOM-scoped,
//! not reach-scoped (menus-scope §"How it fits"). Admin reads any room; a
//! staff member reads only the rooms they are assigned to (the chokepoint's
//! `reachable_rooms`). A caller with no assignment to the requested room gets
//! a NotFound — NOT a distinct 403 — so the verb never leaks which rooms
//! exist (a room-existence oracle is a soft leak; fail closed as "not found").
//!
//! Guardians hold no `menu.get` cap at all: the host cap wall blocks them
//! before this verb runs (the guardian food view is `care.menu.week`, which
//! returns only their own child's derived rows). So this verb is admin/staff
//! only, and the room-scope check is the whole of its authorization.

use lb_auth::Principal;

use crate::authz::{reachable_rooms, Chokepoint};
use crate::menu::{Menu, MenuError, Slot};

#[derive(Debug, serde::Deserialize)]
pub struct GetInput {
    pub date: String,
    pub room_id: String,
    pub slot: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: GetInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.menu.get input: {e}"))?;

    // The slot is a fixed-set key — an unknown value is a hard input error
    // (never a silent "not found", which would hide a client bug).
    let slot = Slot::parse(&parsed.slot)
        .ok_or_else(|| format!("{}", MenuError::InvalidSlot(parsed.slot.clone())))?;

    // Room-scope gate. Admin => ["*"] wildcard reads any room. Staff => the
    // requested room must be in their assigned set; otherwise NotFound (we do
    // NOT distinguish "unassigned" from "missing" — no room-existence oracle).
    let rooms = reachable_rooms(cp, principal).await;
    let is_admin = rooms.iter().any(|r| r == "*");
    if !is_admin && !rooms.iter().any(|r| r == &parsed.room_id) {
        return Err(format!(
            "{}",
            MenuError::NotFound(Menu::id(&parsed.date, &parsed.room_id, slot))
        ));
    }

    let id = Menu::id(&parsed.date, &parsed.room_id, slot);
    let row = cp
        .records()
        .read("menu", &id)
        .await
        .map_err(|e| format!("{}", MenuError::StoreDenied(format!("{e}"))))?;
    let value = row.ok_or_else(|| format!("{}", MenuError::NotFound(id.clone())))?;
    let menu: Menu = serde_json::from_value(value).map_err(|e| format!("deserialize menu: {e}"))?;

    serde_json::to_string(&menu).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::{MenuItem, Substitution};
    use crate::menu::allergen::Allergen;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create as store_create, Store};
    use serde_json::json;
    use std::sync::Arc;

    const DATE: &str = "2026-07-14";
    const ROOM: &str = "possums";

    fn admin(signing: &SigningKey) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: "acme".into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.menu.get:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    fn staff(signing: &SigningKey, sub: &str) -> Principal {
        let claims = Claims {
            sub: sub.into(),
            ws: "acme".into(),
            role: Role::Member,
            caps: vec!["mcp:care.menu.get:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    /// Seed one menu cell via the real store write path.
    async fn seed_menu(store: &Arc<Store>) -> String {
        let menu = Menu {
            date: DATE.into(),
            room_id: ROOM.into(),
            slot: Slot::Lunch,
            items: vec![MenuItem {
                name: "Peanut satay".into(),
                allergens: vec![Allergen::Peanut],
            }],
            substitutions: vec![Substitution {
                restriction: Allergen::Peanut,
                substitute: "Sunflower satay".into(),
            }],
        };
        let id = Menu::id(DATE, ROOM, Slot::Lunch);
        store_create(store, "acme", "menu", &id, &serde_json::to_value(&menu).unwrap())
            .await
            .unwrap();
        id
    }

    async fn assign_staff(store: &Arc<Store>, sub: &str, room: &str) {
        store_create(
            store,
            "acme",
            "staff_assignment",
            &[sub, room].join("::"),
            &json!({"staff_sub": sub, "room_id": room}),
        )
        .await
        .unwrap();
    }

    fn get_input() -> String {
        json!({"date": DATE, "room_id": ROOM, "slot": "lunch"}).to_string()
    }

    #[tokio::test]
    async fn admin_gets_any_room_menu() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_menu(&store).await;

        let out = run(&cp, &admin(&key), &get_input()).await.expect("admin get");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["room_id"], ROOM);
        assert_eq!(v["items"][0]["name"], "Peanut satay");
        assert_eq!(v["substitutions"][0]["substitute"], "Sunflower satay");
    }

    #[tokio::test]
    async fn assigned_staff_gets_the_menu() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_menu(&store).await;
        assign_staff(&store, "user:sam", ROOM).await;

        let out = run(&cp, &staff(&key, "user:sam"), &get_input())
            .await
            .expect("assigned staff get");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["room_id"], ROOM);
    }

    #[tokio::test]
    async fn unassigned_staff_is_denied_as_not_found() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_menu(&store).await;
        // Nadia is assigned to a DIFFERENT room only.
        assign_staff(&store, "user:nadia", "koalas").await;

        let err = run(&cp, &staff(&key, "user:nadia"), &get_input())
            .await
            .expect_err("unassigned staff must be denied");
        // Denied as "not found" — no room-existence leak.
        assert!(err.contains("menu not found"), "got: {err}");
    }

    #[tokio::test]
    async fn missing_cell_is_not_found() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        // No seed — the cell does not exist. Admin passes the room gate but
        // the read misses.
        let err = run(&cp, &admin(&key), &get_input())
            .await
            .expect_err("missing cell must be NotFound");
        assert!(err.contains("menu not found"), "got: {err}");
    }
}
