//! The reconciler — the ONE place channel membership is granted/revoked, and
//! the milestone's leak vector (§Subagent notes: "the reconciler is one careful
//! agent"). It grants/revokes lb per-channel caps so membership always equals
//! the domain-record derivation (`authz::channel_members`).
//!
//! ## Grant the desired set, revoke the departed — same handler as the edge
//!
//! Every handler that changes a membership source (guardianship link/unlink/
//! update, staff room assignment/move) calls the reconciler in the SAME breath
//! as the edge write (the transaction discipline `care-authz-scope.md` demands
//! for grants — an ex-partner reading the child channel is a feed-leak-severity
//! bug). Two shapes:
//!
//! - [`grant_membership`] — grant a member their role's caps (`sub` always;
//!   `pub` iff Full). Called on link/enable/assign for the affected subject.
//! - [`revoke_membership`] — revoke BOTH caps from a subject. Called on
//!   unlink/disable/unassign. Idempotent (revoking an absent grant succeeds).
//!
//! ## Best-effort at the call site, but NEVER silent
//!
//! Like the media serve-grant + the feed emit: a fault is returned so the caller
//! logs it, but the durable edge write is the source of truth and is not rolled
//! back on a grant fault. A missing GRANT is a lockout (a member can't read a
//! channel they're entitled to); a missing REVOKE is the leak — so the revoke
//! path's failure is the one that matters, and callers MUST surface it.
//!
//! ## No host client (era-1 / unit tests) ⇒ no-op
//!
//! The reconciler needs the `SidecarClient` to reach `grants.*`. Without it (the
//! store-only test path) it is a no-op `Ok(())` — the derivation is still unit-
//! tested directly; the grant round-trip is a live-node concern.

use lb_ext_native::{CallError, SidecarClient};
use serde_json::json;

use crate::authz::ChannelMember;
use crate::messaging::channel_id::{pub_cap, sub_cap, ChannelRole};

/// Grant `member` their role's caps on `channel_id`. Full ⇒ `sub` + `pub`;
/// ReadOnly ⇒ `sub` only (the announcements policy). Idempotent — re-granting
/// settles to the same grant row. No client ⇒ no-op.
pub async fn grant_membership(
    client: Option<&SidecarClient>,
    channel_id: &str,
    subject: &str,
    role: ChannelRole,
) -> Result<(), CallError> {
    let Some(client) = client else { return Ok(()) };
    for cap in role.caps(channel_id) {
        client
            .call_tool("grants.assign", json!({ "subject": subject, "cap": cap }))
            .await?;
    }
    Ok(())
}

/// Revoke BOTH channel caps from `subject` on `channel_id` — the unlink path.
/// Revokes `pub` AND `sub` regardless of the prior role (a downgrade from Full
/// to gone must drop both). Idempotent: revoking an absent grant succeeds
/// (lb `grants_revoke`). No client ⇒ no-op. THIS is the leak-critical path — a
/// caller must surface its error.
pub async fn revoke_membership(
    client: Option<&SidecarClient>,
    channel_id: &str,
    subject: &str,
) -> Result<(), CallError> {
    let Some(client) = client else { return Ok(()) };
    for cap in [sub_cap(channel_id), pub_cap(channel_id)] {
        client
            .call_tool("grants.revoke", json!({ "subject": subject, "cap": cap }))
            .await?;
    }
    Ok(())
}

/// Reconcile a whole channel to its derived membership (the idempotent HEALING
/// sweep — repair, not the primary path). Grants every derived member their
/// role's caps. It does NOT revoke here (care holds no "who currently has this
/// cap" query — revokes ride the event handlers that know the departed
/// subject); the sweep's job is to REPAIR a member who should have access but
/// lost it (a dropped event, a half-applied link). No client ⇒ no-op.
pub async fn reconcile_channel(
    client: Option<&SidecarClient>,
    channel_id: &str,
    members: &[ChannelMember],
) -> Result<(), CallError> {
    for m in members {
        let role = if m.full { ChannelRole::Full } else { ChannelRole::ReadOnly };
        grant_membership(client, channel_id, &m.subject, role).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // With no host client the reconciler is a no-op (the derivation is tested
    // directly in `authz::scope`; the grant round-trip is a live-node concern).
    #[tokio::test]
    async fn no_client_is_a_noop_ok() {
        assert!(grant_membership(None, "care-child-leo", "user:ana", ChannelRole::Full)
            .await
            .is_ok());
        assert!(revoke_membership(None, "care-child-leo", "user:ana").await.is_ok());
        let members = vec![ChannelMember { subject: "user:ana".into(), full: true }];
        assert!(reconcile_channel(None, "care-child-leo", &members).await.is_ok());
    }
}
