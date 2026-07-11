//! Era-2 host-callback delegation — the chokepoint's platform-enforced reach path.
//!
//! From `node-v0.3.0` / `sdk-v0.3.0` a native (Tier-2) extension can call
//! granted host MCP verbs BACK into the core through the re-exported
//! [`SidecarClient`] (`lb-ext-native`'s host-callback client). The care
//! chokepoint uses that to delegate reach resolution to lb's entity-scoped
//! grants (`authz.check_scoped` / `authz.scope_filter`) instead of resolving
//! it from `guardianship` records itself (era 1, now the documented fallback).
//!
//! This module owns ONE responsibility: translate the chokepoint's typed
//! questions into the two host verbs' JSON wire shapes and back. It is verb-
//! agnostic to the core (rule 10) — `table`/`ids` are opaque data; the care
//! extension owns what they mean.
//!
//! ## Wire shapes (lb `authz.check_scoped` / `authz.scope_filter`)
//!
//! - `authz.check_scoped { cap, table, id }` → `{ "allowed": bool }`
//! - `authz.scope_filter { cap, table }` → `{ "filter": "all" }`
//!   or `{ "filter": { "ids": [...] } }`
//!
//! The principal is the CALLER's own (derived from the child's scoped
//! `LB_EXT_TOKEN`) — these verbs never accept a `user` argument, so the
//! callback can only ever learn its OWN reach. But the care chokepoint asks
//! reach questions ABOUT a guardian principal, not about the extension's own
//! token. That is why the scoped grants are keyed to the **guardian's**
//! subject (derived on `guardianship.link`) and the chokepoint reads them
//! back through [`ReachClient`] with the guardian's identity carried in the
//! grant, resolved host-side. See `care-authz-scope.md` §"Era 2".

use lb_ext_native::{CallError, SidecarClient};
use serde_json::{json, Value};

use super::caps::{REACH_CAP, REACH_TABLE};

/// The era-2 reach client — a thin typed facade over the host-callback
/// [`SidecarClient`]. Holds the client and answers the chokepoint's two
/// questions by calling `authz.check_scoped` / `authz.scope_filter`.
///
/// Cloneable and cheap (the underlying `SidecarClient` pools one HTTP
/// client); construct once at sidecar start and share on the [`Chokepoint`].
#[derive(Clone)]
pub struct ReachClient {
    client: SidecarClient,
}

/// The result of `authz.scope_filter` for the `child` table: either the
/// principal reaches every child (`All`) or exactly the listed ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReachFilter {
    /// The principal reaches every child in the workspace (an `All` grant).
    All,
    /// The principal reaches exactly these child ids (the union of its
    /// scoped grants).
    Ids(Vec<String>),
}

impl ReachClient {
    /// Wrap a ready [`SidecarClient`] (the normal path builds it from the
    /// injected env via `SidecarClient::from_env()` at sidecar start).
    pub fn new(client: SidecarClient) -> Self {
        Self { client }
    }

    /// `authz.check_scoped { cap: REACH_CAP, table: "child", id }` for
    /// `guardian_sub` — may that guardian reach `child_id`? The scoped grant
    /// derived on `guardianship.link` carries the guardian's reach under
    /// [`REACH_CAP`]; the host resolves the caller's grants and answers.
    ///
    /// A `403` (`CallError::Denied`) is NOT a reach answer — it means the
    /// EXTENSION's own token lacks `mcp:authz.check_scoped:call`, a
    /// misconfiguration; it surfaces as an `Err` so the chokepoint fails
    /// CLOSED (deny), never silently allows.
    pub async fn reaches(&self, child_id: &str) -> Result<bool, CallError> {
        let out = self
            .client
            .call_tool(
                "authz.check_scoped",
                json!({ "cap": REACH_CAP, "table": REACH_TABLE, "id": child_id }),
            )
            .await?;
        Ok(out
            .get("allowed")
            .and_then(Value::as_bool)
            .unwrap_or(false))
    }

    /// `authz.scope_filter { cap: REACH_CAP, table: "child" }` — which
    /// children does the caller reach? Translates the wire `{"filter":"all"}`
    /// / `{"filter":{"ids":[...]}}` into [`ReachFilter`].
    pub async fn reachable(&self) -> Result<ReachFilter, CallError> {
        let out = self
            .client
            .call_tool(
                "authz.scope_filter",
                json!({ "cap": REACH_CAP, "table": REACH_TABLE }),
            )
            .await?;
        match out.get("filter") {
            Some(Value::String(s)) if s == "all" => Ok(ReachFilter::All),
            Some(Value::Object(o)) => {
                let ids = o
                    .get("ids")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                Ok(ReachFilter::Ids(ids))
            }
            // An unexpected shape denies (empty) rather than panicking —
            // a protocol drift is visible as "reaches nothing", never a leak.
            _ => Ok(ReachFilter::Ids(Vec::new())),
        }
    }

    /// Borrow the underlying host-callback client so the grant-derivation
    /// path (`guardianship.link`/`unlink`) can call `grants.assign` /
    /// `grants.revoke` through the SAME client (one dependency, both the
    /// read and the derive directions). See [`super::grant`].
    pub fn client(&self) -> &SidecarClient {
        &self.client
    }
}
