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
//! - `authz.check_scoped { cap, table, id, subject }` → `{ "allowed": bool }`
//! - `authz.scope_filter { cap, table, subject }` → `{ "filter": "all" }`
//!   or `{ "filter": { "ids": [...] } }`
//!
//! ## Reach ABOUT the caller (native-caller-identity scope, node-v0.4.0)
//!
//! The sidecar's callback token is the EXTENSION's identity, not the
//! guardian's — so a bare `check_scoped` would only ever learn the
//! extension's own reach. `node-v0.4.0` closed this: the reach verbs gained
//! an optional `subject`, and a caller holding the delegation cap
//! `mcp:authz.delegate_reach:call` may name it — the host then resolves
//! THAT subject's scoped grants instead of the caller's. The care extension
//! requests exactly that delegation cap at install; the chokepoint passes
//! `subject = <the frame caller's sub>` (a guardian's `user:<x>`, projected
//! from the native call frame). So the reach question is genuinely ABOUT the
//! guardian who made the call — rule 7 enforced in-sidecar. See
//! `care-authz-scope.md` §"Era 2" + `native-caller-identity-scope.md`.

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

    /// `authz.check_scoped { cap: REACH_CAP, table: "child", id, subject }` —
    /// may `subject` (the guardian who made the call, projected from the frame
    /// caller) reach `child_id`? The scoped grant derived on
    /// `guardianship.link` carries THAT guardian's reach under [`REACH_CAP`];
    /// the host resolves `subject`'s grants (authorized by the extension's
    /// `mcp:authz.delegate_reach:call` install grant) and answers.
    ///
    /// A `403` (`CallError::Denied`) is NOT a reach answer — it means the
    /// EXTENSION's own token lacks `mcp:authz.check_scoped:call` or the
    /// delegation cap, a misconfiguration; it surfaces as an `Err` so the
    /// chokepoint fails CLOSED (deny), never silently allows.
    pub async fn reaches(&self, subject: &str, child_id: &str) -> Result<bool, CallError> {
        let out = self
            .client
            .call_tool(
                "authz.check_scoped",
                json!({ "cap": REACH_CAP, "table": REACH_TABLE, "id": child_id, "subject": subject }),
            )
            .await?;
        Ok(out.get("allowed").and_then(Value::as_bool).unwrap_or(false))
    }

    /// `authz.scope_filter { cap: REACH_CAP, table: "child", subject }` —
    /// which children does `subject` (the frame caller's guardian) reach?
    /// Translates the wire `{"filter":"all"}` / `{"filter":{"ids":[...]}}`
    /// into [`ReachFilter`]. `subject` is resolved host-side behind the
    /// extension's delegation cap (native-caller-identity scope).
    pub async fn reachable(&self, subject: &str) -> Result<ReachFilter, CallError> {
        let out = self
            .client
            .call_tool(
                "authz.scope_filter",
                json!({ "cap": REACH_CAP, "table": REACH_TABLE, "subject": subject }),
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
