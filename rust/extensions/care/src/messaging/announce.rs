//! `care.announce.post` — admin/staff post to a center's announcements channel.
//! Cap: `mcp:care.announce.post:call` (admin/staff hold it; guardians do NOT —
//! deny-tested). The READ-ONLY policy is enforced TWICE, no care hack:
//!
//! 1. Guardians hold no `mcp:care.announce.post:call` cap ⇒ the host wall denies
//!    them this verb.
//! 2. Even reaching lb directly, a guardian holds only `bus:chan/care-center-*:sub`
//!    (granted read-only by the announcements-provisioning path), never `:pub` —
//!    so lb's `channel.post` gate (`bus:chan/{cid}:pub`) 403s the post.
//!
//! The verb is a thin wrapper over lb's `channel.post`: the announcement BODY is
//! author-supplied content (never translated — chrome is localized, content is
//! not, per `messaging-scope.md`). Fails loud with no host client (a post needs
//! the durable channel).

use lb_auth::Principal;
use serde_json::json;

use crate::authz::Chokepoint;
use crate::messaging::channel_id::center_channel;

#[derive(Debug, serde::Deserialize)]
pub struct AnnounceInput {
    /// The center whose announcements channel to post to.
    pub center_id: String,
    /// The message body — author content, verbatim (not translated).
    pub body: String,
}

pub async fn run(cp: &Chokepoint, _principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: AnnounceInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.announce.post input: {e}"))?;
    if parsed.center_id.trim().is_empty() {
        return Err("care.announce.post: center_id is required".to_string());
    }
    if parsed.body.trim().is_empty() {
        return Err("care.announce.post: body is required".to_string());
    }

    let channel_id = center_channel(&parsed.center_id);
    let client = cp
        .host_client()
        .ok_or_else(|| "care.announce.post needs a host client (no channel on the era-1 path)".to_string())?;

    // lb's channel.post gate requires `bus:chan/{cid}:pub` — which admin/staff
    // hold (granted on center provisioning) and guardians do not. The care
    // install holds the `bus:chan/care-**:pub` wildcard so this call is authorized
    // when the caller's own grant matches; lb charges the post to the caller.
    let out = client
        .call_tool(
            "channel.post",
            json!({ "cid": channel_id, "body": parsed.body }),
        )
        .await
        .map_err(|e| format!("channel.post failed: {e}"))?;
    Ok(out.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    fn staff(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:teacher".into(),
            ws: ws.into(),
            role: Role::Member,
            caps: vec!["mcp:care.announce.post:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn empty_body_rejects_before_any_post() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = staff(&key, "acme");
        assert!(run(&cp, &p, r#"{"center_id":"center:hq","body":"  "}"#).await.is_err());
    }

    #[tokio::test]
    async fn no_host_client_fails_loud() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = staff(&key, "acme");
        // A well-formed post needs the durable channel — no silent no-op.
        assert!(run(&cp, &p, r#"{"center_id":"center:hq","body":"snow day"}"#)
            .await
            .is_err());
    }
}
