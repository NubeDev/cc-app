//! authz — the guardian-isolation chokepoint.
//!
//! **Every** read and write verb in the care extension passes through this
//! module. CLAUDE.md rule 7 is sacred: a guardian may only ever see records
//! for children they hold a live `guardianship` edge to. The chokepoint
//! exists so there is ONE place where "who sees what" lives — a leak across
//! families is the existential bug this product can have, and N
//! per-verb-inline-filter implementations is N chances to leak.
//!
//! ## Two-call surface (milestone 02 deliverable)
//!
//! - [`assert_reach`] — for `get` / `update` / `watch` paths. Returns
//!   `Ok(())` when the principal reaches the child, `Err(Deny)` otherwise.
//! - [`reachable_children`] — for `list` paths. Returns the set of child
//!   ids the principal reaches. **Empty** (not error) when the principal
//!   reaches none — list-verbs deny by returning zero rows, never an error
//!   (CLAUDE.md rule 7 / scope §Testing).
//! - [`reachable_rooms`] — staff analog of the above, for room-scoped lists.
//!
//! Admin passes via an **audited role check** inside this module (one audit
//! point), never a bypass at the call site. The harness asserts this in
//! `tests/matrix.rs`.
//!
//! ## Era 1 / era 2 (two implementations of the SAME two calls)
//!
//! - **Era 1 (now):** resolve from `guardianship` / staff-assignment
//!   records in the lb store per call. Per-request cache only — no
//!   cross-request cache (staleness = leak). This is what the matrix
//!   harness exercises.
//! - **Era 2 (lb entity-scoped grants, already shipped in the patched lb
//!   source as `mcp:authz.check_scoped:call` / `mcp:authz.scope_filter:call`):
//!   the chokepoint delegates to the wall. The call sites do NOT change —
//!   that's the point of the chokepoint (the era-2 swap is a one-file fix,
//!   see [`scope::resolve_era2_todo`]). Until a native child has a
//!   host-callback client, the era-2 path is a stub; the matrix harness
//!   tests the era-1 path.
//!
//! ## Grep fence (milestone 02 exit gate)
//!
//! `scripts/check-authz-fence.sh` fails CI if any `*.rs` file outside this
//! module contains the word `guardianship` (the table name) — that's the
//! only way to keep N-verbs-from-leaking honest.
//!
//! ## See also
//!
//! - [`../../../../../docs/scope/care/care-authz-scope.md`](../../../../../docs/scope/care/care-authz-scope.md)
//! - [`../../../../../docs/build/02-care-skeleton-authz.md`](../../../../../docs/build/02-care-skeleton-authz.md)

pub mod caps;
pub mod deny;
pub mod grant;
pub mod host_callback;
pub mod principal;
mod records;
mod scope;

use lb_auth::Principal;

pub use deny::{AuthzError, ReachDecision};
pub use host_callback::{ReachClient, ReachFilter};
pub use records::{Edge, Guardian, StaffAssignment};
pub use scope::{edge_id, Scope};

use std::sync::Arc;

/// The inputs the chokepoint needs to answer "can this principal reach
/// this child?". Bundled so call sites don't grow as era 2 swaps in.
///
/// ## Two eras behind ONE surface
///
/// - **Era 2 (live when `reach` is `Some`):** the platform-enforced path.
///   The chokepoint delegates to lb's entity-scoped grants via the
///   host-callback ([`ReachClient`] → `authz.check_scoped` /
///   `authz.scope_filter`). Built with [`Chokepoint::with_host_callback`]
///   at sidecar start (the production path).
/// - **Era 1 (fallback when `reach` is `None`):** resolve from
///   `guardianship` / `staff_assignment` records in the store directly.
///   Built with [`Chokepoint::new`]. This is the documented fallback for
///   when lb's verbs aren't reachable, and the path the store-only unit
///   tests exercise.
///
/// The call sites (`assert_reach` / `reachable_children` / `reachable_rooms`)
/// are IDENTICAL across both eras — that is the entire point of the
/// chokepoint. Which era runs is decided here, once, by whether a
/// [`ReachClient`] is present.
#[derive(Clone)]
pub struct Chokepoint {
    /// The lb store the chokepoint resolves `guardianship` / staff-assignment
    /// records from (era 1, the fallback). The handle is `Arc`-shared so the
    /// chokepoint is cheap to clone per request.
    pub store: Arc<lb_store::Store>,
    /// The workspace the resolution is scoped to (every record lookup is
    /// workspace-scoped; this matches the hard wall).
    pub ws: String,
    /// The era-2 host-callback reach client. `Some` ⇒ the platform-enforced
    /// path is live (delegate to `authz.check_scoped` / `authz.scope_filter`);
    /// `None` ⇒ fall back to the era-1 store resolution. Not part of the
    /// `PartialEq`/`Debug` surface — it holds an HTTP client.
    reach: Option<ReachClient>,
}

impl Chokepoint {
    /// Build an ERA-1 chokepoint for `store` + `ws` (the store-only fallback
    /// path). The store handle is shared across requests; the chokepoint does
    /// NOT cache resolved edges (per-request only — staleness = leak, see
    /// module doc). Use [`Chokepoint::with_host_callback`] for the live
    /// era-2 platform path.
    pub fn new(store: Arc<lb_store::Store>, ws: impl Into<String>) -> Self {
        Self {
            store,
            ws: ws.into(),
            reach: None,
        }
    }

    /// Build an ERA-2 chokepoint (the live production path): reach resolves
    /// through the platform via the host-callback `reach` client. The `store`
    /// is still carried so the era-1 fallback remains available if a call
    /// needs it, but the reach questions delegate to lb's scoped grants.
    pub fn with_host_callback(
        store: Arc<lb_store::Store>,
        ws: impl Into<String>,
        reach: ReachClient,
    ) -> Self {
        Self {
            store,
            ws: ws.into(),
            reach: Some(reach),
        }
    }

    /// The era-2 reach client if this chokepoint runs the platform path,
    /// else `None` (era-1 fallback). Used by the grant-derivation path in
    /// `guardianship.link`/`unlink` to reach `grants.assign`/`revoke` through
    /// the SAME host-callback client.
    pub fn reach(&self) -> Option<&ReachClient> {
        self.reach.as_ref()
    }
}

/// MANDATORY CALL (milestone 02 deliverable): every `get`/`update`/`watch`
/// verb in the care extension routes through here. Returns `Ok(())` when
/// the principal reaches the child, `Err(AuthzError::Denied)` otherwise.
/// **Empty list filters are NEVER an error here** — this is for
/// single-target verbs (the list analog is [`reachable_children`], which
/// returns an empty vec, not an Err).
///
/// Admin reaches everything — but only via the audited role check inside
/// this function, never via a call-site bypass. The harness asserts both
/// the allow and the deny + the audit-log path in `tests/matrix.rs`.
pub async fn assert_reach(
    cp: &Chokepoint,
    principal: &Principal,
    child_id: &str,
) -> Result<(), AuthzError> {
    // Admin: audited role check inside the chokepoint. One audit point —
    // never a call-site bypass. The audit is currently an `eprintln!`; a
    // future milestone will route it to the platform audit reactor.
    if principal.role() == lb_auth::Role::WorkspaceAdmin
        || principal.role() == lb_auth::Role::SuperAdmin
    {
        eprintln!(
            "authz: admin pass — sub={} ws={} role={:?} child={}",
            principal.sub(),
            principal.ws(),
            principal.role(),
            child_id,
        );
        return Ok(());
    }

    // Era 2 (live path): delegate to lb's entity-scoped grants via the
    // host-callback. The `reach` client authenticates AS this caller, so
    // `authz.check_scoped` resolves THIS principal's scoped grants — the
    // guardian reaches exactly the children whose reach grant was derived
    // on `guardianship.link` (`care-authz-scope.md` §"Era 2"). A callback
    // error (misconfigured extension grant, host unreachable) fails CLOSED.
    if let Some(reach) = cp.reach() {
        return match reach.reaches(child_id).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(AuthzError::Denied {
                reason: "no scoped reach grant (platform)",
            }),
            Err(_) => Err(AuthzError::Denied {
                reason: "reach callback failed",
            }),
        };
    }

    // Era 1 (fallback): resolve the live `guardianship` edge from the store
    // directly. Used when no host-callback client is present (the store-only
    // path / the documented fallback when lb's verbs aren't reachable).
    scope::resolve_era1_guardian(cp, principal, child_id).await
}

/// MANDATORY CALL (milestone 02 deliverable): every `list` verb in the care
/// extension routes through here. Returns the set of child ids the
/// principal reaches; **empty** when none. Never an error.
///
/// Staff get their assigned-room children (via [`reachable_rooms`] +
/// room→child expansion, called by the verb body — this function is the
/// guardian slice). Guardians get the children they hold a live edge to.
/// Admins get the wildcard `["*"]` sentinel — the verb body interprets that
/// as "no filter, return everything" (one audit point).
pub async fn reachable_children(cp: &Chokepoint, principal: &Principal) -> Vec<String> {
    if principal.role() == lb_auth::Role::WorkspaceAdmin
        || principal.role() == lb_auth::Role::SuperAdmin
    {
        eprintln!(
            "authz: admin list — sub={} ws={}",
            principal.sub(),
            principal.ws(),
        );
        return vec!["*".to_string()];
    }

    // Era 2 (live path): `authz.scope_filter` returns the caller's reachable
    // child set (the union of their scoped reach grants). `All` degrades to
    // the admin wildcard's meaning for a guardian only if the platform
    // granted `All` — which the derive path never does for a guardian, so in
    // practice this is the `Ids` set. A callback error resolves to empty
    // (list-verbs deny by returning zero rows — never an error, never a leak).
    if let Some(reach) = cp.reach() {
        return match reach.reachable().await {
            Ok(ReachFilter::Ids(ids)) => ids,
            Ok(ReachFilter::All) => vec!["*".to_string()],
            Err(_) => Vec::new(),
        };
    }

    scope::resolve_era1_guardian_set(cp, principal).await
}

/// MANDATORY CALL (milestone 02 deliverable): staff analog of
/// [`reachable_children`]. Returns the rooms the staff member is assigned
/// to. Empty when unassigned. Admins get `["*"]`.
///
/// Staff room scoping stays ERA-1 (store-resolved) in milestone 03: the
/// guardianship grant-derivation (Step C) covers guardian→child reach; the
/// staff→room scoped-grant derivation is a later slice (`staff_assignment`
/// edges, not guardianship). The call site is unchanged, so swapping it to
/// era-2 later is the same one-place fix as the guardian path.
pub async fn reachable_rooms(cp: &Chokepoint, principal: &Principal) -> Vec<String> {
    if principal.role() == lb_auth::Role::WorkspaceAdmin
        || principal.role() == lb_auth::Role::SuperAdmin
    {
        return vec!["*".to_string()];
    }
    scope::resolve_era1_staff_rooms(cp, principal).await
}
