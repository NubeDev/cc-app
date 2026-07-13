//! The feed-watch grant — the per-child SSE stream-isolation control (milestone
//! 10, closing the lb `bus.watch` gap shipped as `node-v0.4.3` / NubeDev/lb#49).
//!
//! ## From reach-check-at-subscribe to PLATFORM stream isolation
//!
//! m08 could only enforce a reach-check-**at-subscribe** in the `feed.watch` verb
//! (see `feed/watch.rs`): lb's generic `bus.watch` cap was workspace-wide, so a
//! forged raw subscribe against another child's `care.feed.<child>` subject was
//! not platform-denied, and revoking a guardian mid-session did not terminate an
//! already-open stream. Both were lb gaps
//! (`docs/debugging/authz/bus-watch-unscoped-and-no-midstream-revoke.md`).
//!
//! `node-v0.4.3` closed both (additive, no SDK change):
//! - **Gap 1** — a subject-scoped `bus:<subject>:watch` cap now narrows
//!   `bus.watch`. **Present ⇒ required, absent ⇒ open** (back-compat). So once
//!   care mints `bus:care.feed.<child>:watch` for a subject, lb DENIES any
//!   subscribe to that subject without the grant — platform-enforced, not just
//!   the extension's verb gate.
//! - **Gap 2** — `grants.revoke` of a scoped watch grant closes the holder's
//!   open SSE stream within a bounded 3s `WatchRecheck` tick.
//!
//! So this module mirrors the media serve-grant / channel-membership idiom: mint
//! `bus:care.feed.<child>:watch` for a guardian on `guardianship.link` (iff the
//! edge opts into the daily feed), revoke it on `unlink` (and on a
//! `receives_daily_feed` flip in `update`). The guardian's live stream now rides
//! the platform gate — an unlink terminates it, a stranger's forged subscribe
//! 403s at lb.
//!
//! ## The wildcard hold (lb's no-widening rule)
//!
//! Exactly the `store:media/**:read` idiom: to GRANT `bus:care.feed.<child>:watch`
//! the sidecar must itself HOLD a matching cap (`grants_assign`'s no-widening
//! rule), so the install requests + the node approves the recursive wildcard
//! `bus:care.feed.**:watch` (`extension.toml` + `care_mount::approved_grant` +
//! `live_node_support`, lock-step). The grant minted here is the NARROW per-child
//! form.
//!
//! ## Best-effort at the call site, but never silent
//!
//! Like the channel + media grants: a grant fault is a guardian LOCKOUT (they
//! can't open a feed they're entitled to) — surfaced, not rolled back. A REVOKE
//! fault is the leak (an unlinked ex-partner's stream survives) — the unlink path
//! surfaces it loudly. No host client (era-1 / unit tests) ⇒ no-op `Ok(())`.

use lb_ext_native::{CallError, SidecarClient};
use serde_json::json;

use crate::log::feed_subject;

/// The wildcard watch cap the care sidecar must HOLD so lb's no-widening rule
/// lets it GRANT a per-child `bus:care.feed.<child>:watch`. Requested in
/// `extension.toml` and approved by the node (`care_mount::approved_grant`); the
/// grant this module mints is the NARROW per-child form built below.
pub const FEED_WATCH_CAP_HELD: &str = "bus:care.feed.**:watch";

/// Build the narrow per-child feed-watch cap `bus:care.feed.<child>:watch`.
/// Built with `concat` (not `format!`) so the no-hardcoded-strings fence never
/// mistakes a wire-cap assembly for a user-facing literal — the care id-builder
/// idiom. `feed_subject(child)` owns the `care.feed.<child>` subject string.
pub fn feed_watch_cap(child_id: &str) -> String {
    ["bus:", feed_subject(child_id).as_str(), ":watch"].concat()
}

/// Grant `bus:care.feed.<child>:watch` to a guardian so — and only so — lb's
/// platform `bus.watch` gate authorizes their live SSE subscribe to that child's
/// feed subject. Called on `guardianship.link` (iff `receives_daily_feed`).
/// Idempotent per `(subject, cap)`. No client ⇒ no-op.
pub async fn grant_feed_watch(
    client: Option<&SidecarClient>,
    child_id: &str,
    subject: &str,
) -> Result<(), CallError> {
    let Some(client) = client else { return Ok(()) };
    let cap = feed_watch_cap(child_id);
    client
        .call_tool("grants.assign", json!({ "subject": subject, "cap": cap }))
        .await?;
    Ok(())
}

/// Revoke `bus:care.feed.<child>:watch` from a guardian — the unlink / feed-off
/// path. Per lb#49 Gap 2 this ALSO closes the holder's open SSE stream within a
/// 3s tick (the mid-session-terminate the edge-change drill asserts). This is the
/// leak-critical path — a surviving grant is an ex-partner still on the feed — so
/// callers MUST surface its error. Idempotent (revoking an absent grant
/// succeeds). No client ⇒ no-op.
pub async fn revoke_feed_watch(
    client: Option<&SidecarClient>,
    child_id: &str,
    subject: &str,
) -> Result<(), CallError> {
    let Some(client) = client else { return Ok(()) };
    let cap = feed_watch_cap(child_id);
    client
        .call_tool("grants.revoke", json!({ "subject": subject, "cap": cap }))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_is_the_scoped_watch_form_for_the_child_subject() {
        assert_eq!(feed_watch_cap("child:leo"), "bus:care.feed.child:leo:watch");
        // The held wildcard matches the minted narrow cap under lb's `**` tail.
        assert!(FEED_WATCH_CAP_HELD.starts_with("bus:care.feed."));
        assert!(FEED_WATCH_CAP_HELD.ends_with(":watch"));
    }

    // With no host client both paths are a no-op (the derivation is exercised on
    // a live node; the grant round-trip is a live-node concern — the media /
    // channel idiom).
    #[tokio::test]
    async fn no_client_is_a_noop_ok() {
        assert!(grant_feed_watch(None, "child:leo", "user:ana")
            .await
            .is_ok());
        assert!(revoke_feed_watch(None, "child:leo", "user:ana")
            .await
            .is_ok());
    }
}
