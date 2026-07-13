//! `care.media.commit` — finalize a photo upload: delegate to lb's
//! `media.upload_commit` (verify chunks + checksum, flip to Ready, derive thumb).
//! Cap: `mcp:care.media.commit:call` (staff).
//!
//! The thin commit half of the [`crate::media::begin`] boundary. The caller
//! `care.media.begin`s (gets `{id, chunk_size, chunks}`), PUTs the chunks to the
//! gateway, then `care.media.commit`s with the id. lb owns the checksum verify +
//! variant derivation (`host/src/media/commit.rs`); this wrapper exists so the
//! whole photo path is ONE cc-app surface (begin/commit) rather than a mix of
//! cc-app + bare-lb verbs — the same reason `care.media.begin` wraps rather than
//! exposing lb's verb directly. The returned media id is what a later
//! `care.log.add` attaches (and grants `store:media/{id}:read` to feed recipients
//! for — [`crate::media::serve_grant`]).
//!
//! No mime reject here: the mime was already validated at `begin`; commit only
//! finalizes an id `begin` already accepted.

use lb_auth::Principal;
use serde::Deserialize;
use serde_json::json;

use crate::authz::Chokepoint;

#[derive(Debug, Deserialize)]
pub struct CommitInput {
    /// The media id `care.media.begin` returned.
    pub id: String,
    /// Optional device wall time, forwarded as lb's `now` (the `ready_ts`).
    #[serde(default)]
    pub at: Option<u64>,
}

/// Delegate to lb's `media.upload_commit`, returning its `{ ok, variants }`
/// verbatim. Fails loud with no host client (era-1/test path): commit needs the
/// durable media store.
pub async fn run(cp: &Chokepoint, _principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: CommitInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.media.commit input: {e}"))?;

    let client = cp
        .host_client()
        .ok_or_else(|| "care.media.commit needs a host client (no upload store on the era-1 path)".to_string())?;

    let out = client
        .call_tool(
            "media.upload_commit",
            json!({ "id": parsed.id, "now": parsed.at.unwrap_or(0) }),
        )
        .await
        .map_err(|e| format!("media.upload_commit failed: {e}"))?;

    serde_json::to_string(&out).map_err(|e| format!("serialize commit reply: {e}"))
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
            caps: vec!["mcp:care.media.commit:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(&key, &mint(&key, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn commit_needs_a_host_client_on_the_era1_path() {
        let store = Arc::new(Store::memory().await.unwrap());
        let cp = Chokepoint::new(store, "acme");
        let p = staff("acme");
        let res = run(&cp, &p, r#"{"id":"ab12cd"}"#).await;
        assert!(res.is_err(), "no host client on the era-1 path");
        assert!(res.unwrap_err().contains("host client"));
    }

    #[tokio::test]
    async fn malformed_input_rejects() {
        let store = Arc::new(Store::memory().await.unwrap());
        let cp = Chokepoint::new(store, "acme");
        let p = staff("acme");
        let res = run(&cp, &p, r#"{"no_id":true}"#).await;
        assert!(res.is_err(), "missing id rejects");
    }
}
