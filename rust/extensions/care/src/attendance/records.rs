//! The durable `attendance_event` record + its typed errors + the pickup
//! deny-reason enum — ORCHESTRATOR-OWNED schema (attendance-scope §Goals;
//! subagents never decide record shapes).
//!
//! ## An append-only ledger, never edits
//!
//! Attendance is a LEDGER (attendance-scope §"Intent"): every check-in,
//! check-out, and staff presence is an APPEND. A wrong tap is fixed by a
//! COMPENSATING event (`correction_of` points at the event being corrected),
//! never by mutating a row. "Who is here now" is a DERIVED read over the
//! ledger (`attendance.now`), not a mutable `present: bool` (which loses
//! history, races, and can't be audited). Regulators read this table.
//!
//! ## Staff presence in the SAME table (resolved open question)
//!
//! attendance-scope §"Open questions" recommends staff presence shares this
//! table. A staff event sets `staff_sub` (and leaves `child_id`/`person`
//! empty); a child event sets `child_id`, `room_id`, and the drop-off/pickup
//! `person`. One ledger → `now` computes both `{children, staff}` for the
//! ratio in one scan.

use serde::{Deserialize, Serialize};
use std::fmt;

/// One `attendance_event` (workspace-scoped, append-only). The id is a unique
/// event id (caller-supplied or derived); the ledger never overwrites.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttendanceEvent {
    /// `check_in` | `check_out` — the ledger direction.
    pub kind: EventKind,
    /// The child this event is for (a `child` id). Empty for a STAFF-presence
    /// event (which sets `staff_sub` instead).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_id: Option<String>,
    /// The staff member this event is for (their auth `sub`). Empty for a
    /// CHILD event. Exactly one of `child_id` / `staff_sub` is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staff_sub: Option<String>,
    /// The room the event occurred in (a `room` id). Always set — the ratio
    /// read-out is per-room.
    pub room_id: String,
    /// The event time, ISO-8601 (`YYYY-MM-DDTHH:MM:SSZ`); caller-supplied so
    /// the sidecar stays clock-free (no `Date::now` in the child) and the
    /// kiosk/staff device stamps the real wall time.
    pub at: String,
    /// WHO performed the tap — a staff `sub` or a kiosk `key:` id (audit).
    pub performed_by: String,
    /// The drop-off / pick-up PERSON for a child event — a guardian id or an
    /// authorized-pickup entry name (attendance-scope §Goals). Empty for
    /// staff events and (optionally) for a plain check-in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub person: Option<String>,
    /// A compensating correction: the id of the event this one corrects.
    /// `Some` ⇒ this is a correction event (never an edit — the corrected row
    /// stays in the ledger for audit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction_of: Option<String>,
    /// An admin-capped, AUDITED pickup override: `true` ⇒ this check-out was
    /// allowed past a failed pickup gate by an admin (attendance-scope
    /// §"Pickup authorization"). The `deny_reason` records WHY the gate would
    /// have refused, so the override is fully audited.
    #[serde(default)]
    pub pickup_override: bool,
    /// The pickup-gate deny reason this override bypassed (only meaningful
    /// when `pickup_override`). Rendered per-locale for the audit trail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_reason: Option<PickupDeny>,
    /// Optional free-text note (never translated).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// The ledger direction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    CheckIn,
    CheckOut,
}

impl EventKind {
    pub fn parse(s: &str) -> Option<EventKind> {
        match s.trim().to_ascii_lowercase().as_str() {
            "check_in" | "checkin" => Some(EventKind::CheckIn),
            "check_out" | "checkout" => Some(EventKind::CheckOut),
            _ => None,
        }
    }
    pub fn key(&self) -> &'static str {
        match self {
            EventKind::CheckIn => "check_in",
            EventKind::CheckOut => "check_out",
        }
    }
}

/// WHY a pickup was refused at check-out — an ENUM, not free text, so it
/// renders per-locale (attendance-scope §exit-gate: "a Spanish-speaking
/// teacher must read why the gate refused"). Each arm maps to an
/// `attendance.deny.<key>` catalog key. This is a child-safety control — the
/// reason must be legible to the staff member holding the child.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PickupDeny {
    /// The collector is not a `can_pickup` guardian nor an authorized-pickup
    /// entry on the child's record.
    NotAuthorized,
    /// The collector matched no name on the child's record at all.
    UnknownPerson,
    /// The child has `custody_notes` that must be read before release (the
    /// note surfaces to staff; release needs the admin override).
    CustodyHold,
}

impl PickupDeny {
    /// The i18n catalog key for the human reason (`attendance.deny.<key>`).
    pub fn catalog_key(&self) -> &'static str {
        match self {
            PickupDeny::NotAuthorized => "attendance.deny.not_authorized",
            PickupDeny::UnknownPerson => "attendance.deny.unknown_person",
            PickupDeny::CustodyHold => "attendance.deny.custody_hold",
        }
    }
    pub fn key(&self) -> &'static str {
        match self {
            PickupDeny::NotAuthorized => "not_authorized",
            PickupDeny::UnknownPerson => "unknown_person",
            PickupDeny::CustodyHold => "custody_hold",
        }
    }
}

/// Typed errors the verb layer maps to the MCP `ToolError` shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttendanceError {
    /// The id was empty / whitespace / too long.
    InvalidId(String),
    /// The `kind` value is outside {check_in, check_out}.
    InvalidKind(String),
    /// The `at` timestamp was empty or malformed.
    InvalidTimestamp(String),
    /// A required field was empty (child_id/staff_sub, room_id, performed_by).
    MissingField(&'static str),
    /// The pickup gate refused (carries the reason enum for the localized
    /// staff message). Only an admin `pickup_override` gets past it.
    PickupDenied(PickupDeny),
    /// The event already exists (append is first-write on the event id).
    AlreadyExists(String),
    /// The referenced record (child / corrected event) was not found.
    NotFound(String),
    /// The store denied the read or write.
    StoreDenied(String),
}

impl fmt::Display for AttendanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttendanceError::InvalidId(s) => write!(f, "invalid id: {s:?}"),
            AttendanceError::InvalidKind(s) => write!(f, "invalid kind: {s:?}"),
            AttendanceError::InvalidTimestamp(s) => write!(f, "invalid timestamp: {s:?}"),
            AttendanceError::MissingField(s) => write!(f, "missing required field: {s}"),
            AttendanceError::PickupDenied(r) => {
                write!(f, "pickup denied: {}", r.key())
            }
            AttendanceError::AlreadyExists(s) => write!(f, "attendance event already exists: {s}"),
            AttendanceError::NotFound(s) => write!(f, "not found: {s}"),
            AttendanceError::StoreDenied(what) => write!(f, "store denied: {what}"),
        }
    }
}

impl std::error::Error for AttendanceError {}

/// Validate an ISO-8601 timestamp shape (`YYYY-MM-DDTHH:MM...`). Shape-only —
/// the device supplies real wall time; we reject the obviously-malformed so a
/// garbage `at` never fragments the `now`/`list` time ordering.
pub fn validate_timestamp(s: &str) -> Result<(), AttendanceError> {
    let b = s.as_bytes();
    let ok = b.len() >= 19
        && b[4] == b'-'
        && b[7] == b'-'
        && (b[10] == b'T' || b[10] == b' ')
        && b[13] == b':'
        && b[16] == b':'
        && b[..10]
            .iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
    if ok {
        Ok(())
    } else {
        Err(AttendanceError::InvalidTimestamp(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_kind_parses_its_set_only() {
        assert_eq!(EventKind::parse("check_in"), Some(EventKind::CheckIn));
        assert_eq!(EventKind::parse("CHECKOUT"), Some(EventKind::CheckOut));
        assert_eq!(EventKind::parse("teleport"), None);
    }

    #[test]
    fn deny_reasons_carry_localizable_keys() {
        assert_eq!(
            PickupDeny::NotAuthorized.catalog_key(),
            "attendance.deny.not_authorized"
        );
        assert_eq!(PickupDeny::CustodyHold.key(), "custody_hold");
    }

    #[test]
    fn timestamp_shape_is_enforced() {
        assert!(validate_timestamp("2026-07-14T08:02:00Z").is_ok());
        assert!(validate_timestamp("2026-07-14 08:02:00").is_ok());
        assert!(validate_timestamp("yesterday").is_err());
        assert!(validate_timestamp("2026-07-14").is_err());
    }

    #[test]
    fn a_child_event_round_trips() {
        let e = AttendanceEvent {
            kind: EventKind::CheckIn,
            child_id: Some("child:leo".into()),
            staff_sub: None,
            room_id: "room:possums".into(),
            at: "2026-07-14T08:02:00Z".into(),
            performed_by: "user:teacher".into(),
            person: Some("Sam".into()),
            correction_of: None,
            pickup_override: false,
            override_reason: None,
            note: None,
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["kind"], "check_in");
        assert_eq!(v["child_id"], "child:leo");
        assert!(
            v.get("staff_sub").is_none(),
            "staff_sub omitted for a child event"
        );
        let back: AttendanceEvent = serde_json::from_value(v).unwrap();
        assert_eq!(back, e);
    }
}
