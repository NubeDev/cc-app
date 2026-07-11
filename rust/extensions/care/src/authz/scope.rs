//! The era-1 / era-2 scope resolution. The CHOKEPOINT's call sites call
//! into this module; the module decides which era implements the look-up.
//!
//! ## Era 1 (now)
//!
//! `resolve_era1_guardian` reads the `guardianship` edge record directly
//! from the store (workspace-scoped) and asserts the edge is live.
//! `resolve_era1_guardian_set` lists all live edges for the principal.
//! `resolve_era1_staff_rooms` lists all live `staff_assignment` records.
//!
//! All three resolve per call, per-request cache only. No cross-request
//! cache (staleness = leak, see `mod.rs` doc).
//!
//! ## Era 2 (LIVE — `super::host_callback`)
//!
//! Era 2 is wired and live as of milestone 03 (node-v0.3.0 shipped the
//! native host-callback client). The chokepoint delegates to lb's
//! entity-scoped grants (`authz.check_scoped` / `authz.scope_filter`)
//! through [`super::host_callback::ReachClient`] whenever a
//! [`super::Chokepoint`] carries one (`with_host_callback`). The functions
//! in THIS module are the era-1 FALLBACK — used when no host-callback client
//! is present (store-only unit tests, or when lb's verbs aren't reachable).
//! The call sites in `mod.rs` are identical across both eras (the whole
//! point of the chokepoint), so which era runs is a construction choice, not
//! a call-site change.

use lb_auth::Principal;
use lb_store::{list, read};

use super::AuthzError;
use super::Chokepoint;

/// Resolve a single guardian→child reach decision (era 1).
///
/// Reads the `guardianship:<edge_id>` record from the store (workspace-
/// scoped) and asserts the edge is live. The edge id is derived from
/// `(guardian_sub, child_id)` so the read is O(1) and the matrix harness
/// can seed it directly.
///
/// TODO(era-2): replace this body with a call to lb's
/// `authz.check_scoped` once the native child has a host-callback
/// client. The call site in [`super::assert_reach`] does NOT change.
pub async fn resolve_era1_guardian(
    cp: &Chokepoint,
    principal: &Principal,
    child_id: &str,
) -> Result<(), AuthzError> {
    let edge_id = edge_id(principal.sub(), child_id);
    let row = match read(&cp.store, &cp.ws, "guardianship", &edge_id).await {
        Ok(Some(row)) => row,
        // No edge ⇒ deny. (Empty for get/update/watch paths — a list
        // would have translated this to an empty reply in
        // `reachable_children`.)
        Ok(None) => {
            return Err(AuthzError::Denied {
                reason: "no live guardianship edge",
            });
        }
        Err(_) => {
            // Store errors deny — the chokepoint never bubbles them. A
            // future milestone routes the read failure to the platform
            // audit reactor.
            return Err(AuthzError::Denied {
                reason: "guardianship read failed",
            });
        }
    };

    let live = row.get("live").and_then(|v| v.as_bool()).unwrap_or(false);
    if !live {
        return Err(AuthzError::Denied {
            reason: "guardianship edge is archived",
        });
    }
    Ok(())
}

/// Resolve the set of children a guardian reaches (era 1).
///
/// Lists all live `guardianship` rows where `data.guardian_sub = sub`.
/// Empty when none (list-verbs return an empty reply).
///
/// TODO(era-2): replace this body with a call to lb's `authz.scope_filter`
/// — the verb body interprets an empty reply identically.
pub async fn resolve_era1_guardian_set(cp: &Chokepoint, principal: &Principal) -> Vec<String> {
    let rows = match list(
        &cp.store,
        &cp.ws,
        "guardianship",
        "guardian_sub",
        principal.sub(),
    )
    .await
    {
        Ok(rs) => rs,
        Err(_) => return Vec::new(),
    };
    rows.into_iter()
        .filter_map(|row| {
            let live = row.get("live").and_then(|v| v.as_bool()).unwrap_or(false);
            if !live {
                return None;
            }
            row.get("child_id")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect()
}

/// Resolve the rooms a staff member reaches (era 1).
///
/// Lists all `staff_assignment` rows where `data.staff_sub = sub`. Empty
/// when none. Same era-2 swap as the guardian set.
pub async fn resolve_era1_staff_rooms(cp: &Chokepoint, principal: &Principal) -> Vec<String> {
    let rows = match list(
        &cp.store,
        &cp.ws,
        "staff_assignment",
        "staff_sub",
        principal.sub(),
    )
    .await
    {
        Ok(rs) => rs,
        Err(_) => return Vec::new(),
    };
    rows.into_iter()
        .filter_map(|row| {
            row.get("room_id")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect()
}

/// The deterministic edge id (so the matrix harness can seed it directly).
pub fn edge_id(guardian_sub: &str, child_id: &str) -> String {
    // Same shape as the durable `guardianship` edge id (the link verb in
    // milestone 03 derives it this way too). Sorting not needed — the
    // edge has a natural direction (guardian → child).
    format!("{}::{}", guardian_sub, child_id)
}

/// The `Scope` of a single verb — the cap + the chokepoint call shape.
/// Used by the matrix harness to enumerate verbs and assert the allow /
/// deny / empty table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scope {
    /// The `mcp:care.<verb>:call` cap the verb requires (the host's
    /// wall check).
    pub cap: &'static str,
    /// The chokepoint call the verb body makes.
    pub kind: ScopeKind,
}

/// The two scope shapes the chokepoint enforces. The matrix harness
/// runs each verb against the right shape — `Single` for get/update/
/// watch, `Set` for list (empty-on-miss), `Rooms` for staff-scoped
/// lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// `assert_reach(principal, child_id)` — get/update/watch.
    Single,
    /// `reachable_children(principal)` — list of child ids (empty on
    /// miss).
    Set,
    /// `reachable_rooms(principal)` — list of room ids (empty on miss).
    Rooms,
}
