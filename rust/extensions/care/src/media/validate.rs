//! The photo-only mime guard + the typed [`MediaError`] the media path returns.
//!
//! Daily-feed v1 is PHOTOS ONLY (daily-feed-scope §Non-goals: "video is a v2
//! concern"). lb's own `media.upload_begin` ACCEPTS `video/*` (up to 500 MiB —
//! see lb `media/begin.rs::max_bytes_for_mime`), so the video reject is NOT
//! enforced by the core; cc-app MUST reject a non-image mime at ITS OWN
//! boundary. That boundary is the `care.media.begin` wrapper verb
//! ([`crate::media::begin`]): the honest place to reject, because it is where
//! the mime is known (the already-committed `media_ids` that reach `log::add`
//! carry no mime). [`reject_non_photo`] is that single guard.

use std::fmt;

/// Typed errors the media path returns. Mapped to the opaque MCP `ToolError`
/// shape by the verb layer (the Display text is a developer-facing / audit key,
/// never rendered verbatim to a user — same posture as [`crate::log::LogError`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaError {
    /// A `video/*` mime was offered to the photo path — rejected (v1 is photos
    /// only; daily-feed-scope §Non-goals). Distinct from [`MediaError::UnsupportedMime`]
    /// so the reject reason is explicit in the audit trail.
    VideoRejected(String),
    /// A non-image, non-video mime (`application/pdf`, `text/plain`, an empty
    /// mime, …) was offered — the photo path accepts `image/*` only.
    UnsupportedMime(String),
    /// The underlying `media.upload_begin` / `media.upload_commit` host verb
    /// failed (denied, host unreachable, bad input) — carries the wire context.
    UploadFailed(String),
    /// Minting or revoking the per-media `store:media/{id}:read` serve grant
    /// failed. Best-effort at the `log::add` call site (like the bus emit), but
    /// surfaced here so a grant fault is visible, not silent.
    GrantFailed(String),
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaError::VideoRejected(m) => write!(f, "video is rejected on the photo path: {m:?}"),
            MediaError::UnsupportedMime(m) => write!(f, "unsupported media mime: {m:?}"),
            MediaError::UploadFailed(c) => write!(f, "media upload failed: {c}"),
            MediaError::GrantFailed(c) => write!(f, "media serve-grant failed: {c}"),
        }
    }
}

impl std::error::Error for MediaError {}

/// Accept only a photo (`image/*`); reject `video/*` and every other mime.
///
/// Pure — the ONE place the v1 photos-only rule lives. lb's core would accept a
/// video (its `max_bytes_for_mime` allows `video/*` up to 500 MiB), so this is
/// cc-app's own boundary reject (daily-feed-scope §Non-goals). The mime is
/// matched case-insensitively on its type prefix (`image/`), tolerating
/// `Image/JPEG` from a sloppy client.
pub fn reject_non_photo(mime: &str) -> Result<(), MediaError> {
    let lower = mime.trim().to_ascii_lowercase();
    if lower.starts_with("image/") {
        return Ok(());
    }
    if lower.starts_with("video/") {
        return Err(MediaError::VideoRejected(mime.to_string()));
    }
    Err(MediaError::UnsupportedMime(mime.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_mimes_are_accepted() {
        assert!(reject_non_photo("image/jpeg").is_ok());
        assert!(reject_non_photo("image/png").is_ok());
        assert!(reject_non_photo("image/heic").is_ok());
        // Case-insensitive on the type prefix (a sloppy client).
        assert!(reject_non_photo("Image/JPEG").is_ok());
    }

    #[test]
    fn video_mimes_are_rejected_as_video() {
        assert_eq!(
            reject_non_photo("video/mp4"),
            Err(MediaError::VideoRejected("video/mp4".to_string()))
        );
        assert_eq!(
            reject_non_photo("video/quicktime"),
            Err(MediaError::VideoRejected("video/quicktime".to_string()))
        );
    }

    #[test]
    fn other_mimes_are_unsupported() {
        assert_eq!(
            reject_non_photo("application/pdf"),
            Err(MediaError::UnsupportedMime("application/pdf".to_string()))
        );
        assert_eq!(
            reject_non_photo("text/plain"),
            Err(MediaError::UnsupportedMime("text/plain".to_string()))
        );
        // Empty mime is unsupported (never silently allowed).
        assert!(matches!(
            reject_non_photo(""),
            Err(MediaError::UnsupportedMime(_))
        ));
    }
}
