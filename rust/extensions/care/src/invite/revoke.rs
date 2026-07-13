//! `care.invite.revoke` — admin kills an outstanding invite. Cap:
//! `mcp:care.invite.revoke:call`. Admin-only.
//!
//! Revoke sets the mirror row's `status = Revoked` + `revoked_at_ms` so the
//! admin list reflects it immediately, then calls lb's `invite.revoke` over
//! the host-callback (the SAME `SidecarClient` the era-2 chokepoint reads
//! from).
//!
//! Revoking an already-revoked invite is a no-op (idempotent — admin
//! retry on a flaky network doesn't blow up). Revoking an accepted
//! invite is a typed error: the bind already happened, the admin must
//! deal with the bound account through the guardianship path (unlink
//! the edge — `care.guardianship.unlink`), not through the invite
//! (which is a pre-auth artifact).

use lb_auth::Principal;
use lb_ext_native::CallError;

use crate::authz::Chokepoint;
use crate::center::Locale;
use crate::i18n::t;
use crate::invite::{InviteError, InviteStatus};

#[derive(Debug, serde::Deserialize)]
pub struct RevokeInput {
    pub invite_id: String,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct RevokeReply {
    pub invite_id: String,
    pub status: InviteStatus,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let _ = principal; // admin-gated at the wall

    let parsed: RevokeInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.invite.revoke input: {e}"))?;

    if parsed.invite_id.is_empty() {
        return Err(format!("{}", InviteError::MissingField("invite_id")));
    }

    let mut row = cp
        .records()
        .read("invite", &parsed.invite_id)
        .await
        .map_err(|e| format!("{}: {e}", InviteError::StoreDenied("invite read".into())))?
        .ok_or_else(|| format!("{}", InviteError::NotFound(parsed.invite_id.clone())))?;

    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // Idempotent: revoking an already-revoked invite returns Ok with the
    // current state. A flaky retry doesn't blow up.
    let current_status = row
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if current_status == "revoked" {
        let reply = RevokeReply {
            invite_id: parsed.invite_id.clone(),
            status: InviteStatus::Revoked,
            message: t(
                locale,
                "invite.already_revoked",
                &[("id", &parsed.invite_id)],
            ),
        };
        return serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"));
    }
    // Accepted invites are bound — admin unlinks the edge, doesn't revoke
    // the pre-auth artifact.
    if current_status == "accepted" {
        return Err(format!(
            "{}",
            InviteError::StoreDenied(
                "cannot revoke an accepted invite — unlink the guardianship edge instead".into()
            )
        ));
    }

    let now = now_ms();
    row["status"] = serde_json::Value::String("revoked".to_string());
    row["revoked_at_ms"] = serde_json::Value::Number(now.into());
    cp.records()
        .write("invite", &parsed.invite_id, &row)
        .await
        .map_err(|e| format!("{}: {e}", InviteError::StoreDenied("invite write".into())))?;

    // Era-2 wired: call lb's `invite.revoke` over the host-callback
    // (the SHARED SidecarClient the chokepoint reads from). The mirror
    // is already flipped to Revoked above — the admin list reflects the
    // intent immediately; the lb call is the durable kill. A typed
    // deny surfaces here (a swallowed revoke = the invitee could still
    // accept a token the admin believes is dead).
    //
    // `token_hash` is what the care mirror persists as `lb_invite_id`
    // (see create_guardian / create_staff) — lb's `invite.revoke` looks
    // up by hash. Absent ⇒ an admin revoked before the lb mint landed
    // (edge case for an era-1 boot); surface a typed `StoreDenied`.
    let lb_invite_id = row
        .get("lb_invite_id")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let lb_invite_id = match lb_invite_id {
        Some(id) if !id.is_empty() => id,
        _ => {
            return Err(format!(
                "{}: invite has no lb_invite_id (mint did not complete); the mirror row is Revoked locally but lb was never notified",
                InviteError::StoreDenied("invite.revoke missing lb_invite_id".into())
            ));
        }
    };
    let reach = match cp.reach() {
        Some(r) => r,
        None => {
            return Err(format!(
                "{}: care.invite.revoke requires the host-callback path (era 2); the mirror is locally Revoked but lb was not notified",
                InviteError::StoreDenied("invite.revoke no host-callback".into())
            ));
        }
    };
    reach
        .client()
        .call_tool(
            "invite.revoke",
            serde_json::json!({ "token_hash": lb_invite_id }),
        )
        .await
        .map_err(|e| match e {
            CallError::Denied => format!(
                "{}: lb denied invite.revoke (the extension lacks mcp:invite.create:call?)",
                InviteError::StoreDenied("invite.revoke denied".into())
            ),
            other => format!(
                "{}: invite.revoke callback failed: {other}",
                InviteError::StoreDenied("invite.revoke callback".into())
            ),
        })?;

    let reply = RevokeReply {
        invite_id: parsed.invite_id.clone(),
        status: InviteStatus::Revoked,
        message: t(
            locale,
            "invite.revoked",
            &[("email", &email_for_message(&row))],
        ),
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

/// Pull the email field off a mirror row for the localized `t(...)`
/// call. Avoids a runtime error when the row has no email (paranoid —
/// every mint writes one; the read in this verb only filters by id).
fn email_for_message(row: &serde_json::Value) -> String {
    row.get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invite::{Invite, InviteRole, Locale};
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create as store_create, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.invite.revoke:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    /// Async seed (NOT `futures::executor::block_on` inside a tokio runtime
    /// — that deadlocks; see `list.rs` tests for the rationale).
    async fn seed_invite(store: &lb_store::Store, ws: &str, id: &str, status: &str) {
        let invite = Invite {
            id: id.into(),
            guardian_id: Some("sam".into()),
            email: "sam@example.com".into(),
            role: InviteRole::GuardianMember,
            room_id: None,
            locale: Locale::En,
            status: match status {
                "pending" => InviteStatus::Pending,
                "accepted" => InviteStatus::Accepted,
                "revoked" => InviteStatus::Revoked,
                _ => InviteStatus::Pending,
            },
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

    #[tokio::test]
    async fn revoke_rejects_an_empty_invite_id() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(&cp, &p, r#"{"invite_id":""}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("missing required field"));
    }

    #[tokio::test]
    async fn revoke_rejects_a_missing_invite() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(&cp, &p, r#"{"invite_id":"inv-ghost"}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("invite not found"));
    }

    #[tokio::test]
    async fn revoke_updates_the_mirror_then_surfaces_missing_lb_invite_id() {
        // The test seed doesn't carry `lb_invite_id` (no live mint
        // landed), so the verb's typed deny fires FIRST — before the
        // host-callback check. The mirror WAS flipped to Revoked
        // (admin's intent is recorded), but the lb kill never
        // happened (the mint never happened either). Same defensive
        // posture: the typed error is the admin's diagnostic.
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");
        seed_invite(&cp.store, &cp.ws, "inv-sam", "pending").await;

        let res = run(&cp, &p, r#"{"invite_id":"inv-sam"}"#).await;
        assert!(res.is_err(), "missing lb_invite_id surfaces typed deny");
        let err = res.unwrap_err();
        assert!(err.contains("lb_invite_id"), "got: {err}");

        // The mirror row was flipped to Revoked BEFORE the typed deny
        // (the admin list reflects the intent immediately; the lb call
        // is the eventual-consistency follow-up).
        let row = lb_store::read(&cp.store, "acme", "invite", "inv-sam")
            .await
            .unwrap()
            .expect("present");
        assert_eq!(row["status"], "revoked");
        assert!(row["revoked_at_ms"].as_u64().is_some());
    }

    #[tokio::test]
    async fn revoke_of_an_accepted_invite_is_rejected() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");
        seed_invite(&cp.store, &cp.ws, "inv-sam", "accepted").await;

        let res = run(&cp, &p, r#"{"invite_id":"inv-sam"}"#).await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("accepted"));
        assert!(err.contains("unlink"));
    }
}
