//! The deny surface — the typed `Result` errors the chokepoint returns,
//! and the per-call decision enum the list-verb uses to decide between an
//! empty reply and a deny.
//!
//! Conventions:
//! - `assert_reach` returns `Result<(), AuthzError>` — `Deny` on miss.
//! - `reachable_children` returns `Vec<String>` — **empty** on miss,
//!   never `Err`. List verbs translate `[]` to an empty reply.
//! - `ReachDecision` is the rare third path (for verb bodies that want to
//!   branch on the deny reason without bubbling an Err). The matrix
//!   harness uses it for the `allow / deny / empty` declarative table.

use std::fmt;

/// Every deny path produces one of these. `reason` is the audit key a
/// later milestone will route to the platform audit reactor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthzError {
    /// The principal has no live edge to the child (or the child is in a
    /// different workspace — which the gate-1 wall already rejects, so
    /// this only fires when the cross-workspace edge slipped through).
    Denied { reason: &'static str },
    /// The principal's role is unrecognized. Belt-and-braces — the
    /// `WorkspaceAdmin` / `Member` / `SuperAdmin` check upstream should
    /// never produce this.
    UnknownRole,
}

impl fmt::Display for AuthzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthzError::Denied { reason } => write!(f, "denied: {reason}"),
            AuthzError::UnknownRole => write!(f, "unknown role"),
        }
    }
}

impl std::error::Error for AuthzError {}

/// The list-verb decision: allow (with the resolved set), deny
/// (the principal reaches nothing — caller returns an empty reply), or
/// admin-wildcard (the principal is admin and the verb body should
/// query without a filter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReachDecision {
    /// The principal reaches exactly the listed children (or rooms).
    Allow(Vec<String>),
    /// The principal reaches nothing — list-verbs return an empty reply.
    Empty,
    /// Admin wildcard — list-verbs query unfiltered (one audit point).
    Wildcard,
}
