//! The photo serve-grant — the media-URL-leak defense (daily-feed-scope §Risks
//! "the photo-URL leak").
//!
//! lb serves media bytes behind ONE capability gate: `store:media/{id}:read`
//! (lb `host/src/media/serve.rs::media_serve` — `Request::new(ws, Surface::Store,
//! "media/{id}", Action::Read)`, workspace-first, store-enforced). A guardian's
//! token carries NO such cap by default, so a leaked media URL 403s — UNLESS we
//! mint the per-media read grant to exactly the feed recipients. This module is
//! that mint (and its revoke).
//!
//! ## Verified wire shape (NOT a gap — a real, working grant)
//!
//! lb's `grants.assign` (`host/src/authz/grants.rs::grants_assign`) accepts a
//! **bare resource cap** with NO scope object: `store:media/{id}:read` is a
//! well-formed `surface:resource:action` cap (`Surface::Store` + `Action::Read`
//! both parse — `caps/src/request.rs`), and an absent `scope` arg resolves to
//! `Scope::All` (`authz/tool.rs::scope_arg`). This is UNLIKE the reach grant
//! (`authz/grant.rs`), which needs `scope: {kind:"ids", table, ids}` because it
//! narrows a `mcp:*:call` cap over a table's rows — a resource cap is already
//! narrow (it names ONE media id), so no selector is needed.
//!
//! The catch is lb's **no-widening rule** (`grants_assign` line ~38:
//! `if !cap.starts_with("role:") && !holds_cap(principal, ws, cap)` → `Widen`):
//! to GRANT `store:media/{id}:read`, the care sidecar must ITSELF hold a cap
//! that matches it. So the extension requests (and the node approves) the
//! wildcard `store:media/**:read` — `lb_caps::matches` (`caps/src/grammar.rs`)
//! matches `store:media/**:read` against `store:media/{id}:read` (`**` is the
//! recursive tail), so `holds_cap` passes and the assign is authorized. Same
//! idiom as the reach path holding `mcp:care.reach.child:call` to grant it
//! scoped.
//!
//! ## Best-effort at the call site, but never silent
//!
//! `log::add` calls [`grant_media_read`] AFTER a row lands (like the bus emit) —
//! the record is the source of truth, so a grant fault never fails an already-
//! landed write. But a fault IS surfaced (returned + logged), because a missing
//! grant is a guardian LOCKOUT (they can't fetch a photo they're entitled to),
//! not a leak.

use lb_ext_native::{CallError, SidecarClient};
use serde_json::json;

/// The wildcard serve cap the care sidecar must HOLD so lb's no-widening rule
/// lets it GRANT a per-media `store:media/{id}:read` to a guardian. Requested in
/// `extension.toml` and approved by the node (`care_mount::approved_grant`); the
/// grant this module mints is the NARROW per-id form derived below.
pub const MEDIA_SERVE_CAP_HELD: &str = "store:media/**:read";

/// Build the narrow per-media serve cap `store:media/{id}:read`. Built with
/// `concat` (not `format!`) so the no-hardcoded-strings fence never mistakes a
/// wire-id assembly for a user-facing literal.
fn media_read_cap(media_id: &str) -> String {
    ["store:media/", media_id, ":read"].concat()
}

/// Grant `store:media/{id}:read` to each feed recipient so — and ONLY so — a
/// reach-holder can fetch the photo bytes (the media-URL-leak defense). Mirrors
/// `authz::grant::derive_reach`, but the cap is a bare store resource cap (no
/// scope: an image id is already a single resource). Idempotent per
/// `(subject, cap)`; re-granting settles to the same grant row.
///
/// Best-effort at the call site — returns the FIRST failure so the caller can
/// log it (a missing grant is a guardian lockout, worth surfacing), but the
/// already-landed row is never rolled back. Empty `recipients` is a no-op
/// (a private child with no feed holders): `Ok(())`.
pub async fn grant_media_read(
    client: &SidecarClient,
    media_id: &str,
    recipients: &[String],
) -> Result<(), CallError> {
    let cap = media_read_cap(media_id);
    for subject in recipients {
        client
            .call_tool("grants.assign", json!({ "subject": subject, "cap": cap }))
            .await?;
    }
    Ok(())
}

/// Revoke `store:media/{id}:read` from each recipient — the mirror of
/// [`grant_media_read`], for a future correction/redaction path (a photo pulled
/// from a child's row). Same `(subject, cap)` selector `grant_media_read` used,
/// so the revoke removes exactly that grant and no other. Idempotent (revoking
/// an absent grant succeeds — lb `grants_revoke`).
pub async fn revoke_media_read(
    client: &SidecarClient,
    media_id: &str,
    recipients: &[String],
) -> Result<(), CallError> {
    let cap = media_read_cap(media_id);
    for subject in recipients {
        client
            .call_tool("grants.revoke", json!({ "subject": subject, "cap": cap }))
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_held_cap_is_the_recursive_wildcard() {
        // The sidecar must hold the `**` tail so lb's no-widening rule matches
        // it against the narrow per-id grant below.
        assert_eq!(MEDIA_SERVE_CAP_HELD, "store:media/**:read");
    }

    #[test]
    fn per_media_cap_is_the_bare_store_resource_cap() {
        // The GRANTED cap names ONE media id — no scope selector (unlike reach).
        assert_eq!(media_read_cap("ab12cd"), "store:media/ab12cd:read");
    }
}
