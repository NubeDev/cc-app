//! `care.menu.copy_week` — admin/staff copy a room's WHOLE WEEK of menu cells
//! to a target week (menus-scope §Goals "copy-last-week").
//!
//! Planning a fresh week from scratch is the tedious path; most weeks repeat.
//! This verb takes a source Monday and a target Monday and, for each of the 7
//! days × each [`Slot`], copies the source cell (if it exists) to the same
//! `(day-offset, room, slot)` position in the target week — SAME items and
//! substitutions, only the `date` field advanced to the target day.
//!
//! ## Idempotent by construction
//!
//! Each copy is an UPSERT (`cp.records().write` on the deterministic
//! `Menu::id`), exactly like `set`. Running `copy_week` twice therefore yields
//! the identical target cells — no conflict, no duplication (menus-scope
//! §Testing "copy-week idempotent"). A missing source cell is simply skipped;
//! an empty source week copies zero cells.
//!
//! ## Dates come from INPUT, never the clock
//!
//! Both weeks are given by the caller (each a Monday, ISO `YYYY-MM-DD`). The
//! day-offset walk uses a pure ISO-date add (`add_days`, 0..7) — NOT
//! `Date::now` — so the verb is deterministic and testable, and correct across
//! a month or year boundary (day-of-month + carry through real month lengths,
//! leap years included).
//!
//! ## Who may write
//!
//! Admin OR staff — the same posture as `set`: a guardian never holds
//! `mcp:care.menu.copy_week:call`, so the host capability wall blocks them
//! before this body runs. A `Member` reaching here holds the cap and is staff
//! by definition; only an admin writer takes the audit hop through the
//! chokepoint.

use lb_auth::Principal;

use crate::authz::{assert_reach, Chokepoint, RecordError};
use crate::center::Locale;
use crate::i18n::t;
use crate::menu::{validate_date, Menu, MenuError, Slot};

#[derive(Debug, serde::Deserialize)]
pub struct CopyWeekInput {
    pub room_id: String,
    /// Source week Monday, ISO `YYYY-MM-DD`.
    pub from_week_start: String,
    /// Target week Monday, ISO `YYYY-MM-DD`.
    pub to_week_start: String,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CopyWeekReply {
    /// How many source cells were found and written to the target week.
    pub copied: usize,
    pub message: String,
}

/// Days in a given month, leap-year aware (`month` is 1..=12).
fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            if leap {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Add `offset` days (0..7 in practice) to an ISO `YYYY-MM-DD` date, carrying
/// through real month lengths and year rollover. The date is already
/// shape-validated by the caller, so parsing is infallible here; we still guard
/// defensively rather than panic. Pure — no clock.
fn add_days(date: &str, offset: u32) -> String {
    let mut year: i64 = date[0..4].parse().unwrap_or(0);
    let mut month: u32 = date[5..7].parse().unwrap_or(1);
    let mut day: u32 = date[8..10].parse().unwrap_or(1);

    day += offset;
    loop {
        let dim = days_in_month(year, month);
        if day <= dim {
            break;
        }
        day -= dim;
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
    }
    // Zero-pad each component and join with '-' — a pure date-key builder, no
    // user-facing literal (rule 8 lint distinguishes chrome from key-building).
    let y = format!("{:04}", year);
    let m = format!("{:02}", month);
    let d = format!("{:02}", day);
    [y, m, d].join("-")
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: CopyWeekInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.menu.copy_week input: {e}"))?;

    // Hard-validate BOTH week keys — a malformed Monday must never seed a
    // fragmented target week (menus-scope §"Safety surface").
    validate_date(&parsed.from_week_start).map_err(|e| format!("{e}"))?;
    validate_date(&parsed.to_week_start).map_err(|e| format!("{e}"))?;

    if parsed.room_id.trim().is_empty() {
        return Err(format!("{}", MenuError::MissingField("room_id")));
    }

    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // For each of the 7 days × each slot: read the source cell, and if present,
    // write a copy at the target position with the date advanced. UPSERT keeps
    // the whole verb idempotent.
    let mut copied = 0usize;
    for offset in 0..7u32 {
        let from_date = add_days(&parsed.from_week_start, offset);
        let to_date = add_days(&parsed.to_week_start, offset);

        for &slot in Slot::ALL {
            let src_id = Menu::id(&from_date, &parsed.room_id, slot);
            let existing = cp
                .records()
                .read("menu", &src_id)
                .await
                .map_err(|e| match e {
                    RecordError::Conflict => {
                        format!("{}", MenuError::StoreDenied("copy_week read".into()))
                    }
                    RecordError::Store(s) => {
                        format!("{}: {s}", MenuError::StoreDenied("copy_week read".into()))
                    }
                })?;

            let Some(value) = existing else { continue };
            let mut menu: Menu = serde_json::from_value(value)
                .map_err(|e| format!("{}: {e}", MenuError::StoreDenied("copy_week decode".into())))?;

            // Same items + substitutions; only the service date advances.
            menu.date = to_date.clone();
            let dst_id = Menu::id(&to_date, &parsed.room_id, slot);
            let out = serde_json::to_value(&menu).map_err(|e| format!("serialize menu: {e}"))?;

            cp.records()
                .write("menu", &dst_id, &out)
                .await
                .map_err(|e| match e {
                    RecordError::Conflict => {
                        format!("{}", MenuError::StoreDenied("copy_week write".into()))
                    }
                    RecordError::Store(s) => {
                        format!("{}: {s}", MenuError::StoreDenied("copy_week write".into()))
                    }
                })?;
            copied += 1;
        }
    }

    // Admin audit through the chokepoint (one audit point) when the writer is an
    // admin; a staff Member holds the cap and needs no reach hop.
    if principal.role() == lb_auth::Role::WorkspaceAdmin
        || principal.role() == lb_auth::Role::SuperAdmin
    {
        let _ = assert_reach(cp, principal, &parsed.room_id).await;
    }

    let reply = CopyWeekReply {
        copied,
        message: t(
            locale,
            "menu.week_copied",
            &[("date", &parsed.to_week_start)],
        ),
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::{Allergen, MenuItem, Substitution};
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create as store_create, read, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.menu.copy_week:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    async fn seed(store: &Arc<Store>, date: &str, room: &str, slot: Slot, item: &str) {
        let menu = Menu {
            date: date.into(),
            room_id: room.into(),
            slot,
            items: vec![MenuItem {
                name: item.into(),
                allergens: vec![Allergen::Peanut],
            }],
            substitutions: vec![Substitution {
                restriction: Allergen::Peanut,
                substitute: "Sunflower".into(),
            }],
        };
        let id = Menu::id(date, room, slot);
        store_create(store, "acme", "menu", &id, &serde_json::to_value(&menu).unwrap())
            .await
            .unwrap();
    }

    #[test]
    fn add_days_crosses_a_month_boundary() {
        // A week from 2026-06-29 (Mon) → 06-29..07-05.
        let got: Vec<String> = (0..7).map(|o| add_days("2026-06-29", o)).collect();
        assert_eq!(
            got,
            vec![
                "2026-06-29", "2026-06-30", "2026-07-01", "2026-07-02", "2026-07-03", "2026-07-04",
                "2026-07-05"
            ]
        );
    }

    #[test]
    fn add_days_crosses_a_year_and_leap_february() {
        assert_eq!(add_days("2024-12-30", 3), "2025-01-02"); // year rollover
        assert_eq!(add_days("2024-02-28", 1), "2024-02-29"); // 2024 is a leap year
        assert_eq!(add_days("2025-02-28", 1), "2025-03-01"); // 2025 is not
    }

    #[tokio::test]
    async fn copy_week_copies_source_cells_to_the_target_week() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        // Two cells in the source week: Mon lunch (offset 0), Wed breakfast (offset 2).
        seed(&store, "2026-07-06", "room:possums", Slot::Lunch, "Satay").await;
        seed(&store, "2026-07-08", "room:possums", Slot::Breakfast, "Oats").await;

        let out = run(
            &cp,
            &p,
            r#"{"room_id":"room:possums","from_week_start":"2026-07-06","to_week_start":"2026-07-13"}"#,
        )
        .await
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["copied"], 2);

        // Target Mon lunch (2026-07-13) exists with same items + advanced date.
        let row = read(&store, "acme", "menu", "2026-07-13::room:possums::lunch")
            .await
            .unwrap()
            .unwrap();
        let back: Menu = serde_json::from_value(row).unwrap();
        assert_eq!(back.date, "2026-07-13");
        assert_eq!(back.items[0].name, "Satay");
        assert_eq!(back.items[0].allergens, vec![Allergen::Peanut]);
        assert_eq!(back.substitutions[0].substitute, "Sunflower");

        // Target Wed breakfast (offset 2 → 2026-07-15).
        let row2 = read(&store, "acme", "menu", "2026-07-15::room:possums::breakfast")
            .await
            .unwrap()
            .unwrap();
        let back2: Menu = serde_json::from_value(row2).unwrap();
        assert_eq!(back2.date, "2026-07-15");
        assert_eq!(back2.items[0].name, "Oats");
    }

    #[tokio::test]
    async fn copy_week_is_idempotent() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        seed(&store, "2026-07-06", "room:possums", Slot::Lunch, "Satay").await;
        let input = r#"{"room_id":"room:possums","from_week_start":"2026-07-06","to_week_start":"2026-07-13"}"#;

        let first: serde_json::Value =
            serde_json::from_str(&run(&cp, &p, input).await.unwrap()).unwrap();
        let second: serde_json::Value =
            serde_json::from_str(&run(&cp, &p, input).await.unwrap()).unwrap();
        assert_eq!(first["copied"], 1);
        assert_eq!(second["copied"], 1); // no conflict, no duplication

        let row = read(&store, "acme", "menu", "2026-07-13::room:possums::lunch")
            .await
            .unwrap()
            .unwrap();
        let back: Menu = serde_json::from_value(row).unwrap();
        assert_eq!(back.items[0].name, "Satay");
    }

    #[tokio::test]
    async fn copy_week_on_an_empty_source_copies_zero() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        let out = run(
            &cp,
            &p,
            r#"{"room_id":"room:possums","from_week_start":"2026-07-06","to_week_start":"2026-07-13"}"#,
        )
        .await
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["copied"], 0);
    }
}
