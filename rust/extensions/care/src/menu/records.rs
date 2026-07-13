//! The durable `menu` record + its typed error — ORCHESTRATOR-OWNED schema
//! (menus-scope §Goals; subagents never decide record shapes).
//!
//! A `menu` is one `(date, room, slot)` cell: the food PLAN for that slot,
//! with each item carrying allergen TAGS ([`Allergen`]) and a per-restriction
//! substitute table entered once per `(menu, restriction)` pair. Allergy TRUTH
//! never lives here — it lives on the child record only (`child.allergies`);
//! this record carries tags, and [`super::derive`] intersects the two at
//! read/serve time (menus-scope §"Derivation, not entry"). That one-way flow
//! is the safety design: one place to update an allergy, every planned menu
//! re-derives.
//!
//! The id is deterministic — `<date>::<room>::<slot>` — so `set` is an upsert
//! on the natural key (re-planning a slot overwrites it, copy-week is
//! idempotent) and a `get` addresses the exact cell.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::allergen::{Allergen, Slot};

/// A `menu` record (workspace-scoped) — the plan for one `(date, room, slot)`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Menu {
    /// ISO-8601 `YYYY-MM-DD` service date (validated at the verb boundary).
    pub date: String,
    /// The room this menu is served to (a `room` id).
    pub room_id: String,
    /// The meal slot.
    pub slot: Slot,
    /// The planned items, each with its allergen tags. An item with an EMPTY
    /// tag list is `untaggable` and flags conservatively in the derivation
    /// (never a silent safe — menus-scope §"Risks").
    #[serde(default)]
    pub items: Vec<MenuItem>,
    /// The substitute table: for a given restriction (an allergen key), what
    /// is served instead. Entered once per `(menu, restriction)` pair; a
    /// restriction with a matching item but NO entry here is an UNRESOLVED
    /// substitution — loud at plan time (menus-scope §"Safety surface").
    #[serde(default)]
    pub substitutions: Vec<Substitution>,
}

/// One planned menu item + its allergen tags.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuItem {
    /// The dish name (admin/staff-entered; never translated — recorded as-is).
    pub name: String,
    /// The allergen tags on this item (from the fixed set + `other:`). Empty
    /// ⇒ UNTAGGABLE → conservative flag for any child with any allergy.
    #[serde(default)]
    pub allergens: Vec<Allergen>,
}

/// A substitute served for one restriction (allergen). The `substitute` is
/// the replacement dish name; an absent entry for a matched allergen is the
/// unresolved case the plan view flags red.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Substitution {
    /// The restriction this substitute addresses — an allergen key.
    pub restriction: Allergen,
    /// The replacement dish name (never translated — recorded as-is).
    pub substitute: String,
}

impl Menu {
    /// The deterministic record id for a `(date, room, slot)` cell. `set`
    /// upserts on it; `get`/`week` address it directly; copy-week reuses it
    /// with the target date. One owner of this rule (a drift = a lost cell).
    pub fn id(date: &str, room_id: &str, slot: Slot) -> String {
        // Join (never a `format!` with a literal) so the id-builder is a pure
        // key construction, distinct from user-facing chrome (rule 8 lint).
        [date, room_id, slot.key()].join("::")
    }
}

/// Typed errors the verb layer maps to the MCP `ToolError` shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuError {
    /// The `date` was not ISO-8601 `YYYY-MM-DD` (a plan key — fail hard so a
    /// malformed date never fragments a room's week).
    InvalidDate(String),
    /// The `slot` value is outside the fixed set.
    InvalidSlot(String),
    /// A required field (room_id, item name) was empty.
    MissingField(&'static str),
    /// The menu id was not found (`get` on a missing cell).
    NotFound(String),
    /// The store denied the read or write.
    StoreDenied(String),
}

impl fmt::Display for MenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuError::InvalidDate(s) => write!(f, "invalid date (expect YYYY-MM-DD): {s:?}"),
            MenuError::InvalidSlot(s) => write!(f, "invalid slot: {s:?}"),
            MenuError::MissingField(s) => write!(f, "missing required field: {s}"),
            MenuError::NotFound(s) => write!(f, "menu not found: {s}"),
            MenuError::StoreDenied(what) => write!(f, "store denied: {what}"),
        }
    }
}

impl std::error::Error for MenuError {}

/// Validate an ISO-8601 `YYYY-MM-DD` date (shape + sane ranges — same posture
/// as `child::validate_dob`, kept local so `menu` owns its own key guard).
pub fn validate_date(s: &str) -> Result<(), MenuError> {
    let bytes = s.as_bytes();
    let shape_ok = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit());
    if !shape_ok {
        return Err(MenuError::InvalidDate(s.to_string()));
    }
    let month: u8 = s[5..7].parse().unwrap_or(0);
    let day: u8 = s[8..10].parse().unwrap_or(0);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(MenuError::InvalidDate(s.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_is_deterministic_on_the_natural_key() {
        assert_eq!(
            Menu::id("2026-07-14", "room:possums", Slot::Lunch),
            "2026-07-14::room:possums::lunch"
        );
    }

    #[test]
    fn date_validation_matches_the_dob_posture() {
        assert!(validate_date("2026-07-14").is_ok());
        assert!(validate_date("2026-7-14").is_err());
        assert!(validate_date("2026-13-01").is_err());
    }

    #[test]
    fn menu_round_trips_through_json() {
        let m = Menu {
            date: "2026-07-14".into(),
            room_id: "room:possums".into(),
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
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(v["items"][0]["allergens"][0], "peanut");
        let back: Menu = serde_json::from_value(v).unwrap();
        assert_eq!(back, m);
    }
}
