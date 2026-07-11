//! The durable `guardianship` edge record + its typed error —
//! ORCHESTRATOR-OWNED schema (milestone 03 §Subagent notes).
//!
//! The edge is the AUTHZ SOURCE OF TRUTH (`enrollment-invites-scope.md`
//! §"Data: edges are the authz source of truth"): a guardian reaches a child
//! iff a LIVE edge exists (`live == true`). The chokepoint reads this table
//! (era 1) and, in era 2, a scoped grant DERIVED from the edge on link
//! (`care-authz-scope.md` §"Era 2"). The five per-edge flags below ride the
//! same record.
//!
//! The edge id is deterministic: `<guardian_sub>::<child_id>` (so the
//! chokepoint's `edge_id` helper and the matrix harness can address it
//! directly). One guardian↔child pair ⇒ one edge.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A `guardianship` edge (workspace-scoped, many-to-many guardian↔child).
///
/// The chokepoint reads a MINIMAL projection of this (`authz::Edge`:
/// `guardian_sub` + `child_id` + `live`); the durable shape here adds the
/// relationship + the five flags the product hangs behaviour off.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Guardianship {
    /// The guardian's subject (`user:...`) — matches `guardian.sub` once the
    /// invite is accepted, or the guardian record's pre-account placeholder.
    pub guardian_sub: String,
    /// The child this edge reaches.
    pub child_id: String,
    /// The relationship label (an ENUM key, never free English — the catalog
    /// stores the words; CLAUDE.md rule 8). E.g. `mother`, `father`,
    /// `grandparent`, `guardian`.
    pub relationship: Relationship,
    /// Liveness. `true` while linked; `unlink` sets it `false` (soft — the
    /// edge is retained for audit, reach is denied). The chokepoint denies on
    /// a missing OR `live == false` edge.
    pub live: bool,
    /// The five per-edge flags (`enrollment-invites-scope.md` §Goals).
    #[serde(flatten)]
    pub flags: EdgeFlags,
}

/// The five per-edge flags — each governs a downstream behaviour, stored on
/// the edge and consumed by later milestones (feed, billing, attendance).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EdgeFlags {
    /// May this guardian collect the child (attendance pickup gate).
    #[serde(default)]
    pub can_pickup: bool,
    /// Does this guardian receive the daily feed (a `false` edge gets NO
    /// feed — asserted in daily-feed tests).
    #[serde(default)]
    pub receives_daily_feed: bool,
    /// Does this guardian receive billing (consumed by billing-scope).
    #[serde(default)]
    pub receives_billing: bool,
    /// Is this guardian an emergency contact for the child.
    #[serde(default)]
    pub emergency_contact: bool,
    /// Free-text custody notes — DISPLAY data for staff only; NOT legal
    /// logic and NOT a reach input (`care-authz-scope.md` §Non-goals: reach
    /// is edges only). Never translated.
    #[serde(default)]
    pub custody_notes: Option<String>,
}

/// The relationship label — an enum key (the catalog owns the words per
/// locale; CLAUDE.md rule 8). A new relationship is a catalog + enum add.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Relationship {
    Mother,
    Father,
    Grandparent,
    /// A legal guardian who is not a parent.
    Guardian,
    /// Any other relationship (aunt, foster, etc.) — the catalog key is
    /// `other`; the specific label is not modeled v1.
    Other,
}

impl Relationship {
    /// Parse from the wire string (the catalog key). Rejects anything outside
    /// the enum — a typo fails fast rather than silently becoming `Other`.
    pub fn parse(s: &str) -> Result<Self, GuardianshipError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "mother" => Ok(Relationship::Mother),
            "father" => Ok(Relationship::Father),
            "grandparent" => Ok(Relationship::Grandparent),
            "guardian" => Ok(Relationship::Guardian),
            "other" => Ok(Relationship::Other),
            other => Err(GuardianshipError::InvalidRelationship(other.to_string())),
        }
    }
    /// The catalog key (`"mother"`, …) used to look up the localized word.
    pub fn as_key(&self) -> &'static str {
        match self {
            Relationship::Mother => "mother",
            Relationship::Father => "father",
            Relationship::Grandparent => "grandparent",
            Relationship::Guardian => "guardian",
            Relationship::Other => "other",
        }
    }
}

/// Typed errors the verb layer maps to the MCP `ToolError` shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardianshipError {
    /// The `relationship` value is outside the enum.
    InvalidRelationship(String),
    /// A referenced guardian or child id was empty.
    MissingField(&'static str),
    /// The edge already exists (`link` is first-write on the derived id).
    AlreadyExists(String),
    /// The edge was not found (`unlink`/`update` on a missing pair).
    NotFound(String),
    /// The scoped-grant derivation failed (era 2) but the edge was rolled
    /// back to a consistent state — the edge write MUST NOT stand without its
    /// grant (a live edge with no grant is a lockout; `care-authz-scope.md`
    /// §Risks). Carries the underlying cause.
    GrantDerivationFailed(String),
    /// The scoped-grant derivation AND its rollback both failed — the edge and
    /// its grant have DIVERGED (a live edge with no grant = lockout, or an
    /// archived edge with a surviving grant = the existential leak). The admin
    /// must reconcile (re-run unlink/link). Carries the diverged edge id +
    /// cause. This message is developer/admin-facing (opaque to guardians).
    GrantDerivationDiverged { edge: String, cause: String },
    /// The store denied the read or write.
    StoreDenied(String),
}

impl fmt::Display for GuardianshipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GuardianshipError::InvalidRelationship(s) => write!(f, "invalid relationship: {s:?}"),
            GuardianshipError::MissingField(s) => write!(f, "missing required field: {s}"),
            GuardianshipError::AlreadyExists(s) => write!(f, "guardianship already exists: {s}"),
            GuardianshipError::NotFound(s) => write!(f, "guardianship not found: {s}"),
            GuardianshipError::GrantDerivationFailed(s) => {
                write!(f, "scoped-grant derivation failed: {s}")
            }
            GuardianshipError::GrantDerivationDiverged { edge, cause } => write!(
                f,
                "scoped-grant derivation diverged for edge {edge} — reconcile via unlink/link: {cause}"
            ),
            GuardianshipError::StoreDenied(what) => write!(f, "store denied: {what}"),
        }
    }
}

impl std::error::Error for GuardianshipError {}

/// The deterministic edge id for a `(guardian_sub, child_id)` pair — the
/// SAME shape the chokepoint's `authz::edge_id` uses, so the derive path and
/// the read path address the identical row (a drift here would be a leak or a
/// lockout). One owner of the format: re-exported from the chokepoint.
pub fn edge_id(guardian_sub: &str, child_id: &str) -> String {
    crate::authz::edge_id(guardian_sub, child_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relationship_parses_the_enum_set() {
        assert_eq!(Relationship::parse("mother").unwrap(), Relationship::Mother);
        assert_eq!(Relationship::parse("GUARDIAN").unwrap(), Relationship::Guardian);
        assert!(Relationship::parse("cousin").is_err());
    }

    #[test]
    fn edge_id_is_deterministic_and_matches_the_chokepoint() {
        assert_eq!(edge_id("user:sam", "child:leo"), "user:sam::child:leo");
    }

    #[test]
    fn flags_default_to_all_false() {
        let f = EdgeFlags::default();
        assert!(!f.can_pickup && !f.receives_daily_feed && !f.receives_billing);
        assert!(!f.emergency_contact && f.custody_notes.is_none());
    }
}
