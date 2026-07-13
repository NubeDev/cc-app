//! `care.channel.reconcile` — provision a care channel + heal its membership.
//! Cap: `mcp:care.channel.reconcile:call` (admin/staff — never a guardian).
//!
//! The idempotent healing sweep (`messaging-scope.md` §"Derived membership, one
//! reconciler": event-driven primary, sweep as repair). Given a channel target
//! (a child or a room), it:
//!
//! 1. Ensures the lb channel EXISTS — idempotent `channel.create` (create-on-
//!    reconcile; lb upserts the registry row, so re-running is safe).
//! 2. Derives the membership from domain records (`authz::channel_members` —
//!    behind the fence) and GRANTS each member their role's caps
//!    (`reconcile::reconcile_channel`).
//!
//! It does NOT revoke here — revokes ride the unlink/unassign handlers that know
//! the departed subject (care holds no "who currently has this cap" query). The
//! sweep REPAIRS a member who should have access but lost it (a dropped event).
//!
//! No host client (era-1 / unit tests) ⇒ the derivation still runs (unit-tested)
//! but the channel-create + grants are no-ops — the round-trip is a live-node
//! concern.

use lb_auth::Principal;
use serde_json::json;

use crate::authz::{channel_members, Chokepoint, ChannelTarget};
use crate::messaging::channel_id::{child_channel, room_channel};
use crate::messaging::reconcile::reconcile_channel;

#[derive(Debug, serde::Deserialize)]
pub struct ReconcileInput {
    /// The target kind — `"child"` or `"room"`.
    pub target: String,
    /// The domain id (a child id or a room id).
    pub id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ReconcileReply {
    pub channel_id: String,
    /// The count of members granted (the derived set size).
    pub members: usize,
}

pub async fn run(cp: &Chokepoint, _principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: ReconcileInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.channel.reconcile input: {e}"))?;
    if parsed.id.trim().is_empty() {
        return Err("care.channel.reconcile: id is required".to_string());
    }

    let (channel_id, target) = match parsed.target.as_str() {
        "child" => (child_channel(&parsed.id), ChannelTarget::Child(&parsed.id)),
        "room" => (room_channel(&parsed.id), ChannelTarget::Room(&parsed.id)),
        other => return Err(format!("care.channel.reconcile: unknown target {other:?}")),
    };

    // 1) Ensure the channel exists (idempotent upsert). Best-effort no-op
    //    without a host client (the derivation below still runs for tests).
    if let Some(client) = cp.host_client() {
        client
            .call_tool("channel.create", json!({ "cid": channel_id }))
            .await
            .map_err(|e| format!("channel.create failed: {e}"))?;
    }

    // 2) Derive membership + grant each member their role's caps.
    let members = channel_members(cp, &target).await;
    reconcile_channel(cp.host_client(), &channel_id, &members)
        .await
        .map_err(|e| format!("channel membership grant failed: {e}"))?;

    let reply = ReconcileReply { channel_id, members: members.len() };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authz::Chokepoint;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.channel.reconcile:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    // Reconcile on the era-1 (no host client) path: the derivation runs and the
    // grant/channel-create are no-ops, so it returns the derived member count
    // (0 here — no seeded edges) without erroring.
    #[tokio::test]
    async fn reconcile_child_channel_no_client_is_ok() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let out = run(&cp, &p, r#"{"target":"child","id":"child:leo"}"#)
            .await
            .expect("reconcile");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["channel_id"], "care-child-child:leo");
        assert_eq!(v["members"], 0);
    }

    #[tokio::test]
    async fn unknown_target_rejects() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        assert!(run(&cp, &p, r#"{"target":"teacher","id":"x"}"#).await.is_err());
    }
}
