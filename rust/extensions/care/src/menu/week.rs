//! `care.menu.week` — the GUARDIAN food view: a room's 7-day menu grid PLUS
//! ONLY the asking child's derived substitution rows per `(date, slot)`.
//!
//! This is the medical-leak surface (CLAUDE.md rule 7 — SACRED). A guardian
//! asks for THEIR child; the verb returns that child's ROOM's week of menus,
//! and against each present cell it derives ONLY that one child's substitution
//! rows (`derive::derive_for_child`). It NEVER returns another child's data, and
//! it never surfaces the raw allergen tags of items — only item NAMES + THIS
//! child's derived rows (menus-scope §"How it fits": the guardian read returns
//! only the asking child's rows, never the room's).
//!
//! Authorization is a single gate: `assert_reach` FIRST. A guardian with no
//! live edge to the requested child is DENIED with a 403 (an error on the wire),
//! never a phantom empty week — a leak attempt fails closed. Without this gate
//! Ana could read Mia's room's plan by asking for Mia's `child_id`.
//!
//! A child with no room yet (`room_id == None`) is not an error — the guardian
//! simply gets an empty week (the child is enrolled but not yet placed).

use lb_auth::Principal;
use serde::Serialize;

use crate::authz::{assert_reach, Chokepoint};
use crate::child::Child;
use crate::menu::{allergy_keys, derive_for_child, DerivedRow, Menu, MenuError, Slot};

#[derive(Debug, serde::Deserialize)]
pub struct WeekInput {
    /// The child whose room + derived rows are requested. Reach-gated.
    pub child_id: String,
    /// The Monday of the requested week, ISO-8601 `YYYY-MM-DD`.
    pub week_start: String,
}

/// One item as the guardian sees it — the NAME only. The item's allergen tags
/// are deliberately omitted: surfacing them would leak the room's/other
/// children's restriction relevance (menus-scope §"the medical-leak class").
#[derive(Debug, Serialize)]
struct ItemView {
    name: String,
}

/// One `(date, slot)` cell as the guardian sees it: the planned item names +
/// ONLY this child's derived substitution rows.
#[derive(Debug, Serialize)]
struct SlotView {
    slot: &'static str,
    items: Vec<ItemView>,
    substitutions: Vec<DerivedRow>,
}

/// One day of the week: its date + the four slots (present cells only).
#[derive(Debug, Serialize)]
struct DayView {
    date: String,
    slots: Vec<SlotView>,
}

/// The reply: the child, its room, and the 7-day grid. No other child's data,
/// no raw allergen tags — item names + this child's derived rows only.
#[derive(Debug, Serialize)]
struct WeekReply {
    child_id: String,
    room_id: Option<String>,
    days: Vec<DayView>,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: WeekInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.menu.week input: {e}"))?;

    // RULE 7 — the leak gate. FIRST, before any read: a guardian who does not
    // reach this child is DENIED (403). Without this, asking for another
    // family's child_id would return that child's room's plan.
    assert_reach(cp, principal, &parsed.child_id)
        .await
        .map_err(|e| format!("{e}"))?;

    // The week_start must be a valid Monday-anchored ISO date — a malformed key
    // would fragment the grid; fail hard (same posture as `menu::validate_date`).
    crate::menu::validate_date(&parsed.week_start).map_err(|e| format!("{e}"))?;

    // Read the child to learn its room + allergies. The child truth is the ONLY
    // source of the allergy set the derivation intersects on (menus-scope
    // §"Derivation, not entry").
    let value = cp
        .records()
        .read("child", &parsed.child_id)
        .await
        .map_err(|e| format!("{}", MenuError::StoreDenied(format!("{e}"))))?
        .ok_or_else(|| "child not found".to_string())?;
    let child: Child =
        serde_json::from_value(value).map_err(|e| format!("deserialize child: {e}"))?;

    // No room yet ⇒ an empty week (enrolled but not placed — not an error).
    let room_id = match child.room_id.clone() {
        Some(r) => r,
        None => {
            let reply = WeekReply {
                child_id: parsed.child_id,
                room_id: None,
                days: Vec::new(),
            };
            return serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"));
        }
    };

    let keys = allergy_keys(&child.allergies);

    // Build the 7-day grid: week_start + 0..7 days. Dates come from INPUT — no
    // `Date::now`; a small pure ISO-date-add walks the calendar.
    let mut days = Vec::with_capacity(7);
    for offset in 0..7 {
        let date = add_days(&parsed.week_start, offset)
            .ok_or_else(|| format!("{}", MenuError::InvalidDate(parsed.week_start.clone())))?;

        let mut slots = Vec::new();
        for &slot in Slot::ALL {
            let id = Menu::id(&date, &room_id, slot);
            let row = cp
                .records()
                .read("menu", &id)
                .await
                .map_err(|e| format!("{}", MenuError::StoreDenied(format!("{e}"))))?;
            let Some(v) = row else { continue };
            let menu: Menu =
                serde_json::from_value(v).map_err(|e| format!("deserialize menu: {e}"))?;

            // Item NAMES only — never the raw allergen tags (leak guard).
            let items = menu
                .items
                .iter()
                .map(|it| ItemView {
                    name: it.name.clone(),
                })
                .collect();
            // ONLY this child's derived rows (rule 7). derive_for_child intersects
            // the menu's tags with THIS child's allergy keys and nothing else.
            let substitutions = derive_for_child(&menu, &keys);

            slots.push(SlotView {
                slot: slot.key(),
                items,
                substitutions,
            });
        }

        days.push(DayView { date, slots });
    }

    let reply = WeekReply {
        child_id: parsed.child_id,
        room_id: Some(room_id),
        days,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

/// Add `n` days to an ISO-8601 `YYYY-MM-DD` date, returning the new ISO date.
/// A small PURE calendar walk — dates come from input, never the clock, so this
/// is deterministic and testable. Returns `None` on a malformed input date.
///
/// Handles month lengths + leap years (Gregorian) so week_start on the 28th of
/// February crosses the month/year boundary correctly.
fn add_days(date: &str, n: u32) -> Option<String> {
    let b = date.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let mut year: i64 = date.get(0..4)?.parse().ok()?;
    let mut month: u32 = date.get(5..7)?.parse().ok()?;
    let mut day: u32 = date.get(8..10)?.parse().ok()?;
    if !(1..=12).contains(&month) || day < 1 {
        return None;
    }

    for _ in 0..n {
        day += 1;
        if day > days_in_month(year, month) {
            day = 1;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }
    Some(iso_date(year, month, day))
}

/// Zero-pad `(year, month, day)` into `YYYY-MM-DD` — a pure date-key builder
/// (no user-facing literal; each `format!` uses positional specifiers only).
fn iso_date(year: i64, month: u32, day: u32) -> String {
    let y = format!("{:04}", year);
    let m = format!("{:02}", month);
    let d = format!("{:02}", day);
    [y, m, d].join("-")
}

/// The number of days in a (year, month), Gregorian leap-year aware.
fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::child::create as child_create;
    use crate::menu::allergen::Allergen;
    use crate::menu::{MenuItem, Substitution};
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create as store_create, Store};
    use serde_json::json;
    use std::sync::Arc;

    const WS: &str = "acme";
    const MON: &str = "2026-07-13"; // a Monday
    const ROOM: &str = "room:possums";

    fn admin(signing: &SigningKey) -> Principal {
        principal(signing, "user:admin", Role::WorkspaceAdmin)
    }
    fn guardian(signing: &SigningKey, sub: &str) -> Principal {
        principal(signing, sub, Role::Member)
    }
    fn principal(signing: &SigningKey, sub: &str, role: Role) -> Principal {
        let claims = Claims {
            sub: sub.into(),
            ws: WS.into(),
            role,
            caps: vec![
                "mcp:care.child.create:call".into(),
                "mcp:care.menu.week:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    /// Seed a LIVE guardianship edge via the real store write path (the authz
    /// source of truth the chokepoint reads).
    async fn seed_edge(store: &Arc<Store>, guardian_sub: &str, child_id: &str) {
        let id = [guardian_sub, child_id].join("::");
        store_create(
            store,
            WS,
            "guardianship",
            &id,
            &json!({
                "guardian_sub": guardian_sub,
                "child_id": child_id,
                "relationship": "mother",
                "live": true,
            }),
        )
        .await
        .unwrap();
    }

    /// Seed one menu cell for the possums room on Monday lunch.
    async fn seed_menu(store: &Arc<Store>, subs: Vec<Substitution>) {
        let menu = Menu {
            date: MON.into(),
            room_id: ROOM.into(),
            slot: Slot::Lunch,
            items: vec![MenuItem {
                name: "Peanut satay".into(),
                allergens: vec![Allergen::Peanut],
            }],
            substitutions: subs,
        };
        let id = Menu::id(MON, ROOM, Slot::Lunch);
        store_create(store, WS, "menu", &id, &serde_json::to_value(&menu).unwrap())
            .await
            .unwrap();
    }

    fn week_input(child_id: &str) -> String {
        json!({ "child_id": child_id, "week_start": MON }).to_string()
    }

    /// Seed the two-family fixture: Sam→Leo,Mia; Ana→Leo. Leo is peanut-allergic
    /// and roomed in possums. Returns the store + signing key.
    async fn fixture() -> (Arc<Store>, SigningKey, Chokepoint) {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), WS);
        let a = admin(&key);

        child_create::run(
            &cp,
            &a,
            &json!({
                "id": "leo",
                "name": "Leo",
                "dob": "2021-03-15",
                "allergies": ["peanut"],
                "room_id": ROOM,
            })
            .to_string(),
        )
        .await
        .unwrap();
        child_create::run(
            &cp,
            &a,
            r#"{"id":"mia","name":"Mia","dob":"2020-06-01"}"#,
        )
        .await
        .unwrap();

        // Edges: Sam reaches Leo + Mia; Ana reaches only Leo.
        seed_edge(&store, "user:sam", "leo").await;
        seed_edge(&store, "user:sam", "mia").await;
        seed_edge(&store, "user:ana", "leo").await;

        (store, key, cp)
    }

    #[tokio::test]
    async fn ana_reaches_leo_sees_his_peanut_substitution_row() {
        let (store, key, cp) = fixture().await;
        seed_menu(&store, vec![]).await; // no substitute entered → unresolved

        let ana = guardian(&key, "user:ana");
        let out = run(&cp, &ana, &week_input("leo")).await.expect("week");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();

        assert_eq!(v["child_id"], "leo");
        assert_eq!(v["room_id"], ROOM);
        assert_eq!(v["days"].as_array().unwrap().len(), 7);

        // Day 0 (Monday) has the seeded lunch cell.
        let day0 = &v["days"][0];
        assert_eq!(day0["date"], MON);
        let slot = &day0["slots"][0];
        assert_eq!(slot["slot"], "lunch");
        assert_eq!(slot["items"][0]["name"], "Peanut satay");
        // The item's allergen tags are NOT surfaced (leak guard).
        assert!(slot["items"][0].get("allergens").is_none());
        // Leo's peanut substitution row IS present, unresolved.
        assert_eq!(slot["substitutions"][0]["reason"], "peanut");
        assert_eq!(slot["substitutions"][0]["resolved"], false);
    }

    #[tokio::test]
    async fn rule7_ana_cannot_read_mia_the_cross_family_deny() {
        // MANDATORY cross-family row: Ana has NO edge to Mia → 403, no data.
        let (store, key, cp) = fixture().await;
        seed_menu(&store, vec![]).await;

        let ana = guardian(&key, "user:ana");
        let err = run(&cp, &ana, &week_input("mia"))
            .await
            .expect_err("Ana must be DENIED Mia — the leak gate");
        // Denied, and no room/menu data leaked.
        assert!(!err.contains(ROOM), "must not leak the room: {err}");
        assert!(!err.contains("Peanut"), "must not leak menu items: {err}");
    }

    #[tokio::test]
    async fn a_resolved_substitution_shows_resolved_true() {
        let (store, key, cp) = fixture().await;
        seed_menu(
            &store,
            vec![Substitution {
                restriction: Allergen::Peanut,
                substitute: "Sunflower satay".into(),
            }],
        )
        .await;

        let ana = guardian(&key, "user:ana");
        let out = run(&cp, &ana, &week_input("leo")).await.expect("week");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let sub = &v["days"][0]["slots"][0]["substitutions"][0];
        assert_eq!(sub["resolved"], true);
        assert_eq!(sub["substitute"], "Sunflower satay");
    }

    #[tokio::test]
    async fn child_with_no_room_gets_an_empty_week() {
        // Mia has no room_id → Sam (who reaches her) gets an empty week, not err.
        let (_store, key, cp) = fixture().await;
        let sam = guardian(&key, "user:sam");
        let out = run(&cp, &sam, &week_input("mia")).await.expect("empty week");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["room_id"], serde_json::Value::Null);
        assert_eq!(v["days"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn add_days_walks_the_calendar_across_boundaries() {
        assert_eq!(add_days("2026-07-13", 0).unwrap(), "2026-07-13");
        assert_eq!(add_days("2026-07-13", 6).unwrap(), "2026-07-19");
        // month boundary
        assert_eq!(add_days("2026-07-30", 3).unwrap(), "2026-08-02");
        // leap year: 2024-02-28 + 1 = 2024-02-29
        assert_eq!(add_days("2024-02-28", 1).unwrap(), "2024-02-29");
        // non-leap: 2026-02-28 + 1 = 2026-03-01
        assert_eq!(add_days("2026-02-28", 1).unwrap(), "2026-03-01");
        // year boundary
        assert_eq!(add_days("2026-12-31", 1).unwrap(), "2027-01-01");
    }
}
