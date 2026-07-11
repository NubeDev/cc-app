//! The durable `guardian` record + its typed error — ORCHESTRATOR-OWNED
//! schema (milestone 03 §Subagent notes).
//!
//! Records-before-accounts is the deliberate shape (`enrollment-invites-
//! scope.md` §"Intent"): a guardian record exists BEFORE the person has an
//! account. An invite (milestone 05) later binds a `sub` to this record on
//! `invite.accepted`. Until then `sub` is `None` — the record is real, the
//! account is pending.
//!
//! `locale` is on the record PRE-account because invites (milestone 05) need
//! it to render the invite email in the guardian's language before they ever
//! sign in (CLAUDE.md rule 8 — en/es from day one).

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::center::Locale;

/// The `guardian` record (workspace-scoped). Admin-created; becomes reachable
/// (its `sub` bound) when the guardian accepts their invite.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Guardian {
    /// Display name (admin-entered; never translated).
    pub name: String,
    /// Contact email — the invite address; MUST match at invite-accept
    /// (`enrollment-invites-scope.md` §"Risks: wrong-person binding").
    pub email: String,
    /// Free-text phone (never translated).
    #[serde(default)]
    pub phone: Option<String>,
    /// The bound workspace-member subject once the invite is accepted
    /// (`user:...`). `None` pre-account — the record exists, the account is
    /// pending (records-before-accounts).
    #[serde(default)]
    pub sub: Option<String>,
    /// The guardian's preferred locale — invites render in it BEFORE the
    /// account exists (CLAUDE.md rule 8). Defaults to the workspace default
    /// at create; overridable.
    pub locale: Locale,
}

/// Typed errors the verb layer maps to the MCP `ToolError` shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardianError {
    /// The id was empty / whitespace / too long (durability guard).
    InvalidId(String),
    /// The email was empty or shape-invalid (an invite target — validate it).
    InvalidEmail(String),
    /// A required field was empty (e.g. `name`).
    MissingField(&'static str),
    /// The record already exists (`create` is first-write, not upsert).
    AlreadyExists(String),
    /// The guardian id was not found.
    NotFound(String),
    /// The store denied the read or write.
    StoreDenied(String),
}

impl fmt::Display for GuardianError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GuardianError::InvalidId(s) => write!(f, "invalid id: {s:?}"),
            GuardianError::InvalidEmail(s) => write!(f, "invalid email: {s:?}"),
            GuardianError::MissingField(s) => write!(f, "missing required field: {s}"),
            GuardianError::AlreadyExists(s) => write!(f, "guardian already exists: {s}"),
            GuardianError::NotFound(s) => write!(f, "guardian not found: {s}"),
            GuardianError::StoreDenied(what) => write!(f, "store denied: {what}"),
        }
    }
}

impl std::error::Error for GuardianError {}

/// Shape-validate an email (an invite target — a typo parks the invite, so
/// reject the obvious garbage at create). Loose on purpose: exactly one `@`,
/// non-empty local + domain, a `.` in the domain. Full RFC validation is not
/// the job — the invite round-trip is the real proof.
pub fn validate_email(s: &str) -> Result<(), GuardianError> {
    let parts: Vec<&str> = s.split('@').collect();
    let ok = parts.len() == 2
        && !parts[0].is_empty()
        && !parts[1].is_empty()
        && parts[1].contains('.')
        && !parts[1].starts_with('.')
        && !parts[1].ends_with('.');
    if ok {
        Ok(())
    } else {
        Err(GuardianError::InvalidEmail(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_accepts_plausible_addresses() {
        assert!(validate_email("sam@example.com").is_ok());
        assert!(validate_email("ana.perez@care.co.uk").is_ok());
    }

    #[test]
    fn email_rejects_garbage() {
        assert!(validate_email("sam").is_err());
        assert!(validate_email("sam@").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("sam@example").is_err());
        assert!(validate_email("").is_err());
    }
}
