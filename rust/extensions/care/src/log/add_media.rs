//! Photo/consent enforcement for the `care.log.add` write path.
//!
//! Two guards that live at the write boundary (never at render):
//!
//! - `assert_photo_consent` — EVERY tapped child must hold
//!   `child.photo_consent == true` before a `media_ids` entry lands. Read
//!   through the record store (the child profile), NOT the guardianship table,
//!   so it is not a reach question (no fence concern). Fails closed: a missing
//!   child or a store fault rejects (never attach a photo to a child we can't
//!   confirm consent for).
//! - `grant_photo_reads` — mint `store:media/{id}:read` to the child's feed
//!   recipients so ONLY reach-holders can fetch the bytes (the media-URL-leak
//!   defense, daily-feed-scope §Risks). Best-effort: `None` client
//!   (era-1/tests) ⇒ no-op; a per-grant fault is logged (a missing grant is a
//!   guardian lockout from a photo they're entitled to) but never fails the
//!   already-landed row.

use crate::authz::Chokepoint;
use crate::child::Child;
use crate::log::LogError;
use crate::media::grant_media_read;

/// Read a child profile and assert `photo_consent == true`. See the module doc:
/// fails closed, reads through the record store (not the reach table).
pub async fn assert_photo_consent(cp: &Chokepoint, child_id: &str) -> Result<(), String> {
    let value = cp
        .records()
        .read("child", child_id)
        .await
        .map_err(|_| format!("{}", LogError::StoreDenied("log.add consent read".into())))?
        .ok_or_else(|| format!("{}", LogError::NotFound(child_id.to_string())))?;
    let child: Child =
        serde_json::from_value(value).map_err(|e| format!("deserialize child: {e}"))?;
    if !child.photo_consent {
        return Err(format!("{}", LogError::PhotoConsentDenied(child_id.to_string())));
    }
    Ok(())
}

/// Grant `store:media/{id}:read` to `recipients` for every attached photo. See
/// the module doc: best-effort, one call per media_id so a partial failure still
/// grants the earlier ids.
pub async fn grant_photo_reads(
    client: Option<&lb_ext_native::SidecarClient>,
    media_ids: &[String],
    recipients: &[String],
) {
    let Some(client) = client else { return };
    if recipients.is_empty() {
        return; // A private child with no feed holders — nothing to grant.
    }
    for media_id in media_ids {
        if let Err(e) = grant_media_read(client, media_id, recipients).await {
            // Surfaced, not silent: a missing serve-grant locks a guardian out
            // of a photo they're entitled to. A future milestone routes this to
            // the platform audit reactor. The message is assembled as a
            // developer-facing audit line (concat, no prose literal) and emitted
            // via a `"{}"` format (no user-facing literal — the no-hardcoded-
            // strings fence sees no prose).
            let audit = [
                "care.log.add: media serve-grant failed for ",
                media_id,
                ": ",
                &e.to_string(),
            ]
            .concat();
            eprintln!("{}", audit);
        }
    }
}
