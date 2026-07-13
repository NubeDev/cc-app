//! `care.feed.watch` — AUTHORIZE a guardian's live per-child SSE subscription.
//! Cap: `mcp:care.feed.watch:call`. Every subscribe routes through the authz
//! chokepoint (CLAUDE.md rule 7).
//!
//! ## What this verb DOES (and what lb does, not us)
//!
//! lb's `bus.watch` is an HTTP SSE stream (`GET /bus/stream?subject=…`), NOT a
//! synchronous `call_tool` — a native extension cannot proxy its bytes. So this
//! verb does the ONE thing the extension can enforce correctly: a
//! **reach-check-at-subscribe** (`assert_reach` on the child), then it hands the
//! guardian UI the authorized subject + the stream descriptor to open directly
//! against the gateway. A stranger's `feed.watch` is DENIED here (403), so the
//! UI never receives a subject to open (matrix row asserts it).
//!
//! ## Two documented lb gaps (NOT worked around here)
//!
//! See `docs/debugging/authz/bus-watch-unscoped-and-no-midstream-revoke.md`:
//! 1. The generic `bus.watch` cap is workspace-wide — lb has no per-subject/
//!    per-child bus scoping today, so full stream-level isolation against a
//!    forged raw subscribe is an lb gap. The DURABLE leak surfaces
//!    (`log.list` / `log.day` / media serve) ARE reach-checked; the live bus is a
//!    best-effort channel over already-authorized data.
//! 2. lb does not terminate an OPEN stream on `grants.revoke` — an unlinked
//!    guardian's live stream survives until they disconnect, though their NEXT
//!    subscribe and every durable read are denied immediately.
//!
//! Both are lb-owned fixes (WORKFLOW-LB.md — fix lb generically first). Until
//! they ship, this verb's reach gate is the enforcement point the extension owns.
//!
//! ## Reply — the stream descriptor
//!
//! `{ subject, stream_path, event, reachable }`: the walled subject the UI
//! subscribes to (`care.feed.<child>`), the gateway route to open, the SSE event
//! name lb emits (`message`), and `reachable: true` (a denied caller never gets
//! a reply — the error IS the deny). The UI opens `GET <stream_path>` with its
//! own session token; lb re-checks `mcp:bus.watch:call` + workspace at the route.

use lb_auth::Principal;

use crate::authz::{assert_reach, Chokepoint};
use crate::log::feed_subject;

#[derive(Debug, serde::Deserialize)]
pub struct WatchInput {
    /// The child whose feed the caller wants to watch. Reach-checked.
    pub child_id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct WatchReply {
    /// The walled bus subject the UI subscribes to (`care.feed.<child>`).
    pub subject: String,
    /// The gateway SSE route the UI opens (subject as a query param, since a
    /// subject contains characters that don't sit in a path segment). The UI
    /// appends its own `&token=<session>` — this verb never handles the token.
    pub stream_path: String,
    /// The SSE `event:` name lb frames each payload under (`message`).
    pub event: &'static str,
    /// Always `true` in a reply — a non-reaching caller is denied before this
    /// point (the error is the deny). Carried so the reply is self-describing.
    pub reachable: bool,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: WatchInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.feed.watch input: {e}"))?;

    // Reach check FIRST — a miss is a 403 (never a subject handed back). Admin
    // passes via the chokepoint's audited role check; a linked guardian passes;
    // a stranger is denied. This is the enforcement point the extension owns for
    // the live feed (the durable reads + media serve are separately reach-checked).
    assert_reach(cp, principal, &parsed.child_id)
        .await
        .map_err(|e| format!("{e}"))?;

    let subject = feed_subject(&parsed.child_id);
    // The gateway SSE route (lb `gateway/routes/bus.rs`): subject as a query
    // param. A join, not a formatted literal — pure route construction (rule 8).
    let stream_path = ["/bus/stream?subject=", subject.as_str()].concat();

    let reply = WatchReply {
        subject,
        stream_path,
        event: "message",
        reachable: true,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guardianship::link as guardianship_link;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.guardianship.link:call".into(),
                "mcp:care.feed.watch:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    fn member(signing: &SigningKey, sub: &str, ws: &str) -> Principal {
        let claims = Claims {
            sub: sub.into(),
            ws: ws.into(),
            role: Role::Member,
            caps: vec!["mcp:care.feed.watch:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    /// A LINKED guardian is authorized and gets the child's subject back.
    #[tokio::test]
    async fn linked_guardian_gets_the_subject() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let a = admin(&key, "acme");
        guardianship_link::run(
            &cp,
            &a,
            r#"{"guardian_sub":"user:ana","child_id":"child:leo","relationship":"mother","receives_daily_feed":true}"#,
        )
        .await
        .expect("link");

        let ana = member(&key, "user:ana", "acme");
        let out = run(&cp, &ana, r#"{"child_id":"child:leo"}"#)
            .await
            .expect("linked guardian may watch");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["subject"], "care.feed.child:leo");
        assert_eq!(v["event"], "message");
        assert!(v["stream_path"].as_str().unwrap().contains("care.feed.child:leo"));
    }

    /// RULE 7: a STRANGER guardian (no edge) is DENIED at subscribe — never
    /// handed a subject. A leak here would let a forged subscribe target another
    /// family's feed.
    #[tokio::test]
    async fn stranger_guardian_is_denied() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let stranger = member(&key, "user:mallory", "acme");

        let res = run(&cp, &stranger, r#"{"child_id":"child:leo"}"#).await;
        assert!(res.is_err(), "a stranger must be denied at subscribe (rule 7)");
    }

    /// Admin may watch any child (audited role pass in the chokepoint).
    #[tokio::test]
    async fn admin_may_watch() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let a = admin(&key, "acme");
        let out = run(&cp, &a, r#"{"child_id":"child:leo"}"#)
            .await
            .expect("admin may watch");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["subject"], "care.feed.child:leo");
    }
}
