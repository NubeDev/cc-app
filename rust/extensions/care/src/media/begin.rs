//! `care.media.begin` — the photo upload boundary: reject video, then delegate
//! to lb's `media.upload_begin`.
//! Cap: `mcp:care.media.begin:call` (staff author the feed; guardians read only).
//!
//! ## Why a wrapper, not the bare lb verb
//!
//! lb's `media.upload_begin` ACCEPTS `video/*` (its `max_bytes_for_mime` allows
//! video up to 500 MiB — lb `host/src/media/begin.rs`). Daily-feed v1 is PHOTOS
//! ONLY (daily-feed-scope §Non-goals), so cc-app owns the reject. The already-
//! committed `media_ids` that reach `log::add` carry no mime, so the reject
//! CANNOT live there — it must live where the mime is first known: HERE, the
//! begin boundary. This is the honest seam ([`crate::media::validate::reject_non_photo`]).
//!
//! ## Pass-through shape
//!
//! Input `{ mime, bytes, checksum, at? }` → lb `media.upload_begin
//! { mime, bytes, checksum, now }` → `{ id, chunk_size, chunks }`, returned
//! verbatim (the caller PUTs chunks to the gateway `PUT /media/{id}/chunk/{n}`
//! route, then calls `care.media.commit`). No reach gate: this is a staff WRITE
//! (like `log::add`) — the cap wall is the gate; a guardian holds no such cap.

use lb_auth::Principal;
use serde::Deserialize;
use serde_json::json;

use crate::authz::Chokepoint;
use crate::media::validate::reject_non_photo;

#[derive(Debug, Deserialize)]
pub struct BeginInput {
    /// The declared mime — rejected here unless `image/*` (photos v1).
    pub mime: String,
    /// The declared byte length (lb caps images at 50 MiB).
    pub bytes: u64,
    /// The sha-256 checksum the commit re-verifies.
    pub checksum: String,
    /// Optional device wall time (the sidecar stays clock-free); forwarded as
    /// lb's `now` so the media id derivation is deterministic per session.
    #[serde(default)]
    pub at: Option<u64>,
}

/// Reject a non-photo mime, then delegate to lb's `media.upload_begin`. Returns
/// lb's `{ id, chunk_size, chunks }` verbatim. Fails loud with no host client
/// (era-1/test path): a real upload needs the durable media store, so unlike the
/// best-effort motion seams there is no meaningful no-op.
pub async fn run(cp: &Chokepoint, _principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: BeginInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.media.begin input: {e}"))?;

    // The v1 photos-only reject — the ONE boundary lb's core does not enforce.
    reject_non_photo(&parsed.mime).map_err(|e| format!("{e}"))?;

    let client = cp.host_client().ok_or_else(|| {
        "care.media.begin needs a host client (no upload store on the era-1 path)".to_string()
    })?;

    let out = client
        .call_tool(
            "media.upload_begin",
            json!({
                "mime": parsed.mime,
                "bytes": parsed.bytes,
                "checksum": parsed.checksum,
                "now": parsed.at.unwrap_or(0),
            }),
        )
        .await
        .map_err(|e| format!("media.upload_begin failed: {e}"))?;

    serde_json::to_string(&out).map_err(|e| format!("serialize begin reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    fn staff(ws: &str) -> Principal {
        let key = SigningKey::generate();
        let claims = Claims {
            sub: "user:teacher".into(),
            ws: ws.into(),
            role: Role::Member,
            caps: vec!["mcp:care.media.begin:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(&key, &mint(&key, &claims), 1).expect("verify")
    }

    /// A video mime is rejected AT the begin boundary — before any host call, so
    /// the reject holds even on the era-1 path (no host client). This is the
    /// media-path half of the v1 photos-only rule.
    #[tokio::test]
    async fn video_rejected_at_the_begin_boundary() {
        let store = Arc::new(Store::memory().await.unwrap());
        let cp = Chokepoint::new(store, "acme");
        let p = staff("acme");
        let res = run(
            &cp,
            &p,
            r#"{"mime":"video/mp4","bytes":1024,"checksum":"deadbeef"}"#,
        )
        .await;
        assert!(res.is_err(), "video must reject on the photo path");
        assert!(res.unwrap_err().contains("video"));
    }

    /// A non-image, non-video mime is rejected too (photos accept `image/*` only).
    #[tokio::test]
    async fn pdf_rejected_at_the_begin_boundary() {
        let store = Arc::new(Store::memory().await.unwrap());
        let cp = Chokepoint::new(store, "acme");
        let p = staff("acme");
        let res = run(
            &cp,
            &p,
            r#"{"mime":"application/pdf","bytes":1024,"checksum":"deadbeef"}"#,
        )
        .await;
        assert!(res.is_err(), "a pdf must reject on the photo path");
    }

    /// An image mime passes the reject, then fails at the host boundary on the
    /// era-1 path (no client) — proving the reject is NOT what stopped it, and
    /// the delegation seam is reached.
    #[tokio::test]
    async fn image_passes_reject_then_needs_a_host_client() {
        let store = Arc::new(Store::memory().await.unwrap());
        let cp = Chokepoint::new(store, "acme");
        let p = staff("acme");
        let res = run(
            &cp,
            &p,
            r#"{"mime":"image/jpeg","bytes":1024,"checksum":"deadbeef"}"#,
        )
        .await;
        assert!(res.is_err(), "no host client on the era-1 path");
        assert!(res.unwrap_err().contains("host client"));
    }
}
