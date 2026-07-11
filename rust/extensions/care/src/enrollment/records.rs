//! The durable `enrollment` record + its typed error — ORCHESTRATOR-OWNED
//! schema (milestone 03 §Subagent notes).
//!
//! An enrollment binds a child to a room with a schedule and a status
//! (`enrollment-invites-scope.md` §Goals): `waitlist | enrolled | withdrawn`.
//! A child on the waitlist has a FIFO position PER ROOM (the resolved v1 open
//! question: FIFO per room, not priority tiers — `03-enrollment.md`).
//!
//! The `waitlist_seq` is a monotonic per-room sequence stamped at
//! create-time; ordering a room's waitlist is `ORDER BY waitlist_seq ASC`.
//! Stable across withdrawals: withdrawing a mid-list child does NOT renumber
//! the rest (the scope's "waitlist ordering stable" test).

use serde::{Deserialize, Serialize};
use std::fmt;

/// An `enrollment` record (workspace-scoped) — child↔room + schedule + status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Enrollment {
    /// The enrolled child's id.
    pub child_id: String,
    /// The room the child is enrolled into (or waitlisted for).
    pub room_id: String,
    /// The enrollment status.
    pub status: EnrollmentStatus,
    /// The weekly schedule — which days the child attends. Empty is allowed
    /// (a waitlist entry may not have a schedule yet).
    #[serde(default)]
    pub schedule: Vec<Weekday>,
    /// The FIFO waitlist position within the room — a monotonic per-room
    /// sequence stamped at create. Only meaningful when `status == Waitlist`;
    /// `0` for a directly-enrolled child. Stable across withdrawals.
    #[serde(default)]
    pub waitlist_seq: u64,
    /// ISO-8601 `YYYY-MM-DD` start date (validated at the verb boundary).
    #[serde(default)]
    pub start_date: Option<String>,
}

/// The enrollment lifecycle status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EnrollmentStatus {
    /// On the room's waitlist (has a `waitlist_seq`).
    Waitlist,
    /// Actively enrolled (occupies a room place).
    Enrolled,
    /// Withdrawn (retained for audit; frees no waitlist seq — ordering stays
    /// stable).
    Withdrawn,
}

impl EnrollmentStatus {
    /// Parse from the wire string. Rejects anything outside the enum.
    pub fn parse(s: &str) -> Result<Self, EnrollmentError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "waitlist" => Ok(EnrollmentStatus::Waitlist),
            "enrolled" => Ok(EnrollmentStatus::Enrolled),
            "withdrawn" => Ok(EnrollmentStatus::Withdrawn),
            other => Err(EnrollmentError::InvalidStatus(other.to_string())),
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            EnrollmentStatus::Waitlist => "waitlist",
            EnrollmentStatus::Enrolled => "enrolled",
            EnrollmentStatus::Withdrawn => "withdrawn",
        }
    }
}

/// A day of the week (the schedule unit). Serialized as a lowercase key.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Weekday {
    /// Parse a weekday key (`"mon"`, …). Rejects anything else.
    pub fn parse(s: &str) -> Result<Self, EnrollmentError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "mon" => Ok(Weekday::Mon),
            "tue" => Ok(Weekday::Tue),
            "wed" => Ok(Weekday::Wed),
            "thu" => Ok(Weekday::Thu),
            "fri" => Ok(Weekday::Fri),
            "sat" => Ok(Weekday::Sat),
            "sun" => Ok(Weekday::Sun),
            other => Err(EnrollmentError::InvalidWeekday(other.to_string())),
        }
    }
}

/// Typed errors the verb layer maps to the MCP `ToolError` shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnrollmentError {
    /// The id was empty / whitespace / too long.
    InvalidId(String),
    /// The `status` value is outside the enum.
    InvalidStatus(String),
    /// A `schedule` entry was not a weekday key.
    InvalidWeekday(String),
    /// A required field (child_id / room_id) was empty.
    MissingField(&'static str),
    /// The enrollment already exists (`create` is first-write).
    AlreadyExists(String),
    /// The enrollment id was not found (`update` on a missing id).
    NotFound(String),
    /// The store denied the read or write.
    StoreDenied(String),
}

impl fmt::Display for EnrollmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnrollmentError::InvalidId(s) => write!(f, "invalid id: {s:?}"),
            EnrollmentError::InvalidStatus(s) => write!(f, "invalid status: {s:?}"),
            EnrollmentError::InvalidWeekday(s) => write!(f, "invalid weekday: {s:?}"),
            EnrollmentError::MissingField(s) => write!(f, "missing required field: {s}"),
            EnrollmentError::AlreadyExists(s) => write!(f, "enrollment already exists: {s}"),
            EnrollmentError::NotFound(s) => write!(f, "enrollment not found: {s}"),
            EnrollmentError::StoreDenied(what) => write!(f, "store denied: {what}"),
        }
    }
}

impl std::error::Error for EnrollmentError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_and_weekday_parse_their_sets() {
        assert_eq!(
            EnrollmentStatus::parse("waitlist").unwrap(),
            EnrollmentStatus::Waitlist
        );
        assert!(EnrollmentStatus::parse("pending").is_err());
        assert_eq!(Weekday::parse("mon").unwrap(), Weekday::Mon);
        assert!(Weekday::parse("funday").is_err());
    }
}
