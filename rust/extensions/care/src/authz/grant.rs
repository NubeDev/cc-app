//! Era-2 grant derivation — the write half of the chokepoint's platform path.
//!
//! When `guardianship.link` adds an edge, the extension DERIVES a scoped
//! grant so the guardian reaches that child through lb's entity-scoped grants
//! (`authz.check_scoped` / `authz.scope_filter`). When `guardianship.unlink`
//! removes the edge, the extension REMOVES that grant in the same handler —
//! edge gone AND grant gone, or neither (`care-authz-scope.md` §"Risks":
//! grant surviving unlink = leak; edge without grant = lockout).
//!
//! Both directions go through the host-callback [`SidecarClient`] calling the
//! granted generic `grants.assign` / `grants.revoke` verbs — the core owns
//! *what a scoped grant means*, the extension owns *when* one exists (the
//! guardianship edge is the source of truth). `table`/`ids` are opaque data
//! to the core (rule 10).
//!
//! ## Wire shapes (lb `grants.assign` / `grants.revoke`)
//!
//! - `grants.assign { subject, cap, scope: { kind: "ids", table, ids } }` → `{ ok: true }`
//! - `grants.revoke { subject, cap, scope: { kind: "ids", table, ids } }` → `{ ok: true }`
//!
//! `grants.revoke` matches on `(subject, cap, scope)` — the scope selector is
//! part of the grant's durable id — so revoke passes the SAME single-child
//! selector `assign` used, removing exactly that guardian↔child grant and no
//! other. That is why link/unlink derive ONE grant per edge (not a rewrite of
//! the guardian's whole set): the operation is surgical and idempotent.

use lb_ext_native::{CallError, SidecarClient};
use serde_json::json;

use super::caps::{REACH_CAP, REACH_TABLE};

/// Derive the scoped reach grant for a newly-linked `guardian_sub → child_id`
/// edge: `grants.assign` the guardian [`REACH_CAP`] scoped to `{child, [child_id]}`.
///
/// Called transactionally with the edge write in `guardianship.link` (and
/// `guardianship.update` when it re-affirms an edge). Idempotent: re-assigning
/// the same `(subject, cap, scope)` settles to the same grant row (lb's grant
/// store is first-settle on the derived id).
///
/// A `403` (`CallError::Denied`) means the EXTENSION lacks
/// `mcp:grants.assign:call` — a misconfiguration the caller must surface, NOT
/// swallow (a swallowed assign = a live edge with no grant = a lockout).
pub async fn derive_reach(
    client: &SidecarClient,
    guardian_sub: &str,
    child_id: &str,
) -> Result<(), CallError> {
    client
        .call_tool(
            "grants.assign",
            json!({
                "subject": guardian_sub,
                "cap": REACH_CAP,
                "scope": { "kind": "ids", "table": REACH_TABLE, "ids": [child_id] },
            }),
        )
        .await
        .map(|_| ())
}

/// Remove the scoped reach grant for an unlinked `guardian_sub → child_id`
/// edge: `grants.revoke` the same `(subject, REACH_CAP, {child, [child_id]})`
/// selector `derive_reach` assigned.
///
/// Called transactionally with the edge archive in `guardianship.unlink` —
/// the grant is gone the instant the edge is (no cache, no grace: a surviving
/// grant is a cross-family leak, the existential bug, `care-authz-scope.md`
/// §"Risks"). A `403` here is the same misconfiguration signal as `derive_reach`.
pub async fn remove_reach(
    client: &SidecarClient,
    guardian_sub: &str,
    child_id: &str,
) -> Result<(), CallError> {
    client
        .call_tool(
            "grants.revoke",
            json!({
                "subject": guardian_sub,
                "cap": REACH_CAP,
                "scope": { "kind": "ids", "table": REACH_TABLE, "ids": [child_id] },
            }),
        )
        .await
        .map(|_| ())
}
