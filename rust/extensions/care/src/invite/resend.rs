//! `care.invite.resend` — admin re-sends the invite email. Cap:
//! `mcp:care.invite.resend:call`. Admin-only.
//!
//! Calls lb's `invite.resend` over the host-callback (the SAME
//! `SidecarClient` the era-2 chokepoint reads from). lb rotates the
//! token, refreshes the TTL, and enqueues a fresh email effect atomically
//! (so a resend never yields a born-expired link — see lb
//! `host::invites::resend`). The care mirror flips its `sent_at_ms`
//! and stores the new `lb_invite_id` (the rotated `token_hash`).
//!
//! The invite MUST be in `Pending` or `Sent` status — re-sending a
//! Revoked / Expired / Accepted / Parked invite is a typed error (the
//! email would either be dead, or land in a state the admin didn't mean
//! to put the invitee in).

use lb_auth::Principal;
use lb_ext_native::CallError;

use crate::authz::Chokepoint;
use crate::invite::{hash_invite_token, InviteError, InviteStatus};

#[derive(Debug, serde::Deserialize)]
pub struct ResendInput {
    pub invite_id: String,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ResendReply {
    pub invite_id: String,
    pub status: InviteStatus,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let _ = principal; // admin-gated at the wall

    let parsed: ResendInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.invite.resend input: {e}"))?;

    if parsed.invite_id.is_empty() {
        return Err(format!("{}", InviteError::MissingField("invite_id")));
    }

    let mut row = cp
        .records()
        .read("invite", &parsed.invite_id)
        .await
        .map_err(|e| format!("{}: {e}", InviteError::StoreDenied("invite read".into())))?
        .ok_or_else(|| format!("{}", InviteError::NotFound(parsed.invite_id.clone())))?;

    let current_status = row
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    match current_status {
        "pending" | "sent" => {}
        "accepted" => {
            return Err(format!(
                "{}",
                InviteError::StoreDenied(
                    "cannot resend an accepted invite — the bind already happened".into()
                )
            ));
        }
        "revoked" => {
            return Err(format!(
                "{}",
                InviteError::StoreDenied("cannot resend a revoked invite".into())
            ));
        }
        "expired" => {
            return Err(format!(
                "{}",
                InviteError::StoreDenied("cannot resend an expired invite".into())
            ));
        }
        "parked" => {
            return Err(format!(
                "{}",
                InviteError::StoreDenied(
                    "cannot resend a parked invite — resolve the mismatch first".into()
                )
            ));
        }
        other => {
            return Err(format!(
                "{}: {other:?}",
                InviteError::StoreDenied("unknown invite status".into())
            ));
        }
    }

    // Era-2 wired: SidecarClient::call_tool("invite.resend", …); lb
    // rotates the token + the TTL + enqueues a fresh email effect
    // atomically. We hash the new raw token locally (same SHA-256
    // idiom as create_guardian) and store the new `lb_invite_id` so
    // the next revoke hits the rotated row (not the dead one).
    let lb_invite_id = row
        .get("lb_invite_id")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let lb_invite_id = match lb_invite_id {
        Some(id) if !id.is_empty() => id,
        _ => {
            return Err(format!(
                "{}: invite has no lb_invite_id (mint did not complete); resend requires a minted invite",
                InviteError::StoreDenied("invite.resend missing lb_invite_id".into())
            ));
        }
    };
    let reach = match cp.reach() {
        Some(r) => r,
        None => {
            return Err(format!(
                "{}: care.invite.resend requires the host-callback path (era 2)",
                InviteError::StoreDenied("invite.resend no host-callback".into())
            ));
        }
    };
    let now = now_ms();
    let reply = reach
        .client()
        .call_tool(
            "invite.resend",
            serde_json::json!({ "token_hash": lb_invite_id, "now": now }),
        )
        .await
        .map_err(|e| match e {
            CallError::Denied => format!(
                "{}: lb denied invite.resend (the extension lacks mcp:invite.create:call?)",
                InviteError::StoreDenied("invite.resend denied".into())
            ),
            other => format!(
                "{}: invite.resend callback failed: {other}",
                InviteError::StoreDenied("invite.resend callback".into())
            ),
        })?;
    let new_raw_token = reply.get("token").and_then(|v| v.as_str()).ok_or_else(|| {
        format!(
            "{}: invite.resend reply missing token",
            InviteError::StoreDenied("invite.resend reply".into())
        )
    })?;
    let new_lb_invite_id = hash_invite_token(new_raw_token);

    // Mirror row: bump `sent_at_ms` + record the new lb-internal id
    // (the rotated row's hash — the old one is dead).
    row["lb_invite_id"] = serde_json::Value::String(new_lb_invite_id);
    row["sent_at_ms"] = serde_json::Value::Number(now.into());
    cp.records()
        .write("invite", &parsed.invite_id, &row)
        .await
        .map_err(|e| format!("{}: {e}", InviteError::StoreDenied("invite write".into())))?;

    let reply = ResendReply {
        invite_id: parsed.invite_id.clone(),
        status: InviteStatus::Sent,
        message: "invite resent".to_string(),
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

/// Unix-ms now (no clock dependency — the host stamps the real time on
/// the per-call principal's `iat`; the lb call uses our value as
/// `now` so the TTL rotation is reproducible).
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invite::{Invite, InviteStatus, Locale};
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create as store_create, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.invite.resend:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    /// Async seed (NOT `futures::executor::block_on` inside a tokio runtime
    /// — that deadlocks; see `list.rs` tests for the rationale).
    async fn seed_invite(store: &lb_store::Store, ws: &str, id: &str, status: InviteStatus) {
        let invite = Invite {
            id: id.into(),
            guardian_id: Some("sam".into()),
            email: "sam@example.com".into(),
            role: crate::invite::InviteRole::GuardianMember,
            room_id: None,
            locale: Locale::En,
            status,
            lb_invite_id: None,
            created_at_ms: 100,
            sent_at_ms: None,
            accepted_at_ms: None,
            revoked_at_ms: None,
            expired_at_ms: None,
            parked_reason: None,
        };
        let value = serde_json::to_value(&invite).unwrap();
        store_create(store, ws, "invite", id, &value)
            .await
            .expect("seed");
    }

    // The old `resend_of_a_pending_invite_returns_not_implemented` test
    // was replaced by `resend_of_a_pending_invite_without_lb_invite_id_surfaces_a_typed_error`
    // above: the verb no longer returns `NotImplemented` (the
    // SidecarClient call is wired), and the era-1 unit-test path
    // surfaces a typed deny instead.

    #[tokio::test]
    async fn resend_rejects_an_empty_invite_id() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(&cp, &p, r#"{"invite_id":""}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("missing required field"));
    }

    #[tokio::test]
    async fn resend_rejects_a_revoked_invite() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");
        seed_invite(&cp.store, &cp.ws, "inv-sam", InviteStatus::Revoked).await;

        let res = run(&cp, &p, r#"{"invite_id":"inv-sam"}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("revoked"));
    }

    #[tokio::test]
    async fn resend_rejects_an_accepted_invite() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");
        seed_invite(&cp.store, &cp.ws, "inv-sam", InviteStatus::Accepted).await;

        let res = run(&cp, &p, r#"{"invite_id":"inv-sam"}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("accepted"));
    }

    #[tokio::test]
    async fn resend_of_a_pending_invite_without_lb_invite_id_surfaces_a_typed_error() {
        // Era-1 chokepoint (no host-callback) AND no `lb_invite_id` on
        // the mirror — the verb surfaces the typed deny that fires
        // first (the lb_invite_id check precedes the host-callback
        // check). The unit test for the live callback mint lives in
        // `tests/matrix_invites_live.rs` (the era-2 path exercises the
        // SidecarClient + the lb invite row together).
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");
        seed_invite(&cp.store, &cp.ws, "inv-sam", InviteStatus::Pending).await;

        let res = run(&cp, &p, r#"{"invite_id":"inv-sam"}"#).await;
        assert!(
            res.is_err(),
            "resend without lb_invite_id surfaces typed deny"
        );
        let err = res.unwrap_err();
        assert!(err.contains("lb_invite_id"), "got: {err}");
    }
}
