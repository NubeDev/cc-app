//! The durable `child` record + its typed error — ORCHESTRATOR-OWNED schema
//! (milestone 03 §Subagent notes: subagents never decide record shapes).
//!
//! A child profile carries the SAFETY data the whole product hangs off
//! (`enrollment-invites-scope.md` §"Child profile carries the safety data"):
//! DOB, allergies/dietary (the input to `menus-scope.md` substitutions),
//! medical notes, immunization records, emergency contacts, and the
//! authorized-pickup list (persons, not necessarily guardians — resolved v1
//! as plain child-record entries, `03-enrollment.md` open-question close).
//!
//! `archived` is the soft-delete flag (CLAUDE.md / scope §"archive, never
//! delete"): an archived child is invisible to guardians and recoverable by
//! admin. The list verb filters archived rows from non-admin reads; get
//! returns the row to admin only (the verb layer enforces this).

use serde::{Deserialize, Serialize};
use std::fmt;

/// The `child` record (workspace-scoped). Admin-managed; guardians/staff see
/// it via the authz chokepoint (guardians only for children they hold a live
/// edge to — CLAUDE.md rule 7).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Child {
    /// Given/display name (admin-entered; never translated — recorded as-is).
    pub name: String,
    /// Date of birth, ISO-8601 `YYYY-MM-DD` (validated at the verb boundary).
    pub dob: String,
    /// The room the child is enrolled into (a `room` id). Optional — a child
    /// on the waitlist has no room yet (see `enrollment`).
    #[serde(default)]
    pub room_id: Option<String>,
    /// Allergies/dietary restrictions — the safety input to menu
    /// substitutions. A medical field: validated hard, never best-effort
    /// (`enrollment-invites-scope.md` §"Import is where garbage enters").
    #[serde(default)]
    pub allergies: Vec<String>,
    /// Free-text medical notes (staff-visible; never translated).
    #[serde(default)]
    pub medical_notes: Option<String>,
    /// Immunization record entries (opaque strings v1 — a structured shape is
    /// a phase-2 slice per the scope's immunization non-goal).
    #[serde(default)]
    pub immunizations: Vec<String>,
    /// Emergency contacts (name + phone, free-text; never translated).
    #[serde(default)]
    pub emergency_contacts: Vec<EmergencyContact>,
    /// Authorized-pickup persons — plain child-record entries v1 (the
    /// resolved open question). A pickup person is NOT necessarily a guardian
    /// and gets NO feed reach (grandma can collect but sees nothing).
    #[serde(default)]
    pub authorized_pickups: Vec<PickupPerson>,
    /// Photo-consent flag (daily-feed policy input — a `false` child never
    /// appears in a shared photo). Stored here; enforced in daily-feed.
    #[serde(default)]
    pub photo_consent: bool,
    /// Soft-delete flag. `true` ⇒ invisible to guardians, recoverable by admin.
    #[serde(default)]
    pub archived: bool,
}

/// An emergency contact — free-text name + phone. Not a first-class contact
/// record v1 (the scope's open question resolved to child-record entries).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmergencyContact {
    pub name: String,
    pub phone: String,
    /// Relationship label (free-text: "aunt", "neighbor") — never translated.
    #[serde(default)]
    pub relationship: Option<String>,
}

/// An authorized-pickup person — name + optional phone. A reach-less contact
/// (`care-authz-scope.md` open question): appears on the pickup list, has no
/// guardianship edge, sees no feed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PickupPerson {
    pub name: String,
    #[serde(default)]
    pub phone: Option<String>,
}

/// Typed errors the verb layer maps to the MCP `ToolError` shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildError {
    /// The id was empty / whitespace / too long (durability guard — ids are
    /// URLs in the served UI).
    InvalidId(String),
    /// The `dob` was not ISO-8601 `YYYY-MM-DD` (a safety field — fail hard).
    InvalidDob(String),
    /// A required field was empty (e.g. `name`).
    MissingField(&'static str),
    /// The record already exists (`create` is first-write, not upsert).
    AlreadyExists(String),
    /// The child id was not found (get/update/archive on a missing id).
    NotFound(String),
    /// The store denied the read or write.
    StoreDenied(String),
}

impl fmt::Display for ChildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChildError::InvalidId(s) => write!(f, "invalid id: {s:?}"),
            ChildError::InvalidDob(s) => write!(f, "invalid dob (expect YYYY-MM-DD): {s:?}"),
            ChildError::MissingField(s) => write!(f, "missing required field: {s}"),
            ChildError::AlreadyExists(s) => write!(f, "child already exists: {s}"),
            ChildError::NotFound(s) => write!(f, "child not found: {s}"),
            ChildError::StoreDenied(what) => write!(f, "store denied: {what}"),
        }
    }
}

impl std::error::Error for ChildError {}

/// Validate an ISO-8601 `YYYY-MM-DD` date (a safety field — the verb layer
/// rejects a malformed DOB loudly, never stores garbage). Shape-only (not a
/// calendar check): `NNNN-NN-NN`, all digits, sane month/day ranges.
pub fn validate_dob(s: &str) -> Result<(), ChildError> {
    let bytes = s.as_bytes();
    let shape_ok = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit());
    if !shape_ok {
        return Err(ChildError::InvalidDob(s.to_string()));
    }
    let month: u8 = s[5..7].parse().unwrap_or(0);
    let day: u8 = s[8..10].parse().unwrap_or(0);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(ChildError::InvalidDob(s.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dob_accepts_valid_iso_dates() {
        assert!(validate_dob("2021-03-15").is_ok());
        assert!(validate_dob("2019-12-01").is_ok());
    }

    #[test]
    fn dob_rejects_malformed_and_out_of_range() {
        assert!(validate_dob("2021-3-15").is_err()); // not zero-padded
        assert!(validate_dob("15-03-2021").is_err()); // wrong order
        assert!(validate_dob("2021-13-01").is_err()); // month 13
        assert!(validate_dob("2021-03-32").is_err()); // day 32
        assert!(validate_dob("").is_err());
        assert!(validate_dob("not-a-date").is_err());
    }
}
