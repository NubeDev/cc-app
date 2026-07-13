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
    // Zero-pad into YYYY-MM-DD — positional specifiers only (no user-facing
    // literal; a pure date-key builder, rule-8 lint distinguishes it from chrome).
    let y = format!("{:04}", year);
    let m = format!("{:02}", month);
    let d = format!("{:02}", day);
    Some([y, m, d].join("-"))
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
#[path = "week_tests.rs"]
mod tests;
