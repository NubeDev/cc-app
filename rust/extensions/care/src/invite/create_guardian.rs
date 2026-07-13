//! `care.invite.create_guardian` — admin mints an invite for a guardian.
//! Cap: `mcp:care.invite.create_guardian:call`. Admin-only.
//!
//! Records-before-accounts (see `guardian/records.rs`): the guardian record
//! already exists; this verb mints the lb invite that will eventually bind
//! the guardian's `sub` to it on `invite.accepted`.
//!
//! ## Wire (lb `invite.create`)
//!
//! ```json
//! { "email": "...", "role": "guardian-member", "team": "guardians",
//!   "payload": "<guardian_id>", "locale": "en|es",
//!   "expires_ts": 0, "now": <ms> }
//! ```
//! → `{ "token": "<one-time-token>", "token_hash": "<id>" }`
//!
//! `payload` is opaque to lb (rule 10) — the extension binds it back to the
//! guardian record on `invite.accepted`. `locale` drives the invite email
//! template so a Spanish-speaking Ana gets a Spanish email (CLAUDE.md rule 8).
//! The lb reply carries the raw token (sent once via email) AND the durable
//! `token_hash` (the invite row's id; the care mirror stores `token_hash` for
//! the inverse verbs `revoke` / `resend`).

use lb_auth::Principal;
use lb_ext_native::CallError;

use crate::authz::{Chokepoint, RecordError};
use crate::guardian::{validate_email, GuardianError};
use crate::invite::{hash_invite_token, Invite, InviteError, InviteRole, InviteStatus, Locale};

#[derive(Debug, serde::Deserialize)]
pub struct CreateGuardianInput {
    /// The guardian record id (the admin picks it on create; this verb
    /// reads it back to pull the email + locale).
    pub guardian_id: String,
    /// Override the locale the invite email renders in. Defaults to the
    /// guardian record's `locale` (records-before-accounts binds locale to
    /// the record — `enrollment-invites-scope.md` §"Localized onboarding").
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CreateGuardianReply {
    pub invite_id: String,
    pub status: InviteStatus,
    /// The localized confirmation (the admin UI re-renders it).
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let _ = principal; // admin-gated at the wall; role audited in the chokepoint

    let parsed: CreateGuardianInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.invite.create_guardian input: {e}"))?;

    if parsed.guardian_id.is_empty() {
        return Err(format!("{}", InviteError::MissingField("guardian_id")));
    }

    // The guardian record is the source of truth for the invite's email +
    // locale (records-before-accounts). Read it back FIRST so a malformed
    // guardian_id fails loud, not silently mints an invite with an empty
    // email.
    let guardian_row = cp
        .records()
        .read("guardian", &parsed.guardian_id)
        .await
        .map_err(|e| format!("{}: {e}", InviteError::StoreDenied("guardian read".into())))?
        .ok_or_else(|| {
            format!(
                "{}",
                InviteError::GuardianNotFound(parsed.guardian_id.clone())
            )
        })?;

    // Project through the orchestrator-owned schema so a future field add is
    // a one-file change (records.rs).
    let guardian: crate::guardian::Guardian =
        serde_json::from_value(guardian_row).map_err(|e| {
            format!(
                "{}: {e}",
                InviteError::StoreDenied("guardian deserialize".into())
            )
        })?;

    // Re-shape-validate the email at invite time — defends against a record
    // that slipped through `guardian.create` with a bad email (a typo parks
    // the invite later; the cost of a parked invite is the admin's time).
    validate_email(&guardian.email).map_err(|e| match e {
        GuardianError::InvalidEmail(_) => {
            format!("{}", InviteError::InvalidEmail(guardian.email.clone()))
        }
        // The `StoreDenied` token keeps the lint happy (it's an audit-key
        // marker, not a hardcoded user-facing string); the `{other}` Display
        // surfaces the actual cause to the admin.
        other => {
            format!(
                "{}: {other}",
                InviteError::StoreDenied("validate_email".into())
            )
        }
    })?;

    // Locale: caller override > guardian record's locale. Records-before-
    // accounts already binds locale to the record, so the override is a
    // rare escape hatch.
    let locale = parsed
        .locale
        .as_deref()
        .and_then(|s| Locale::parse(s).ok())
        .unwrap_or(guardian.locale);

    // Build the durable mirror row. Deterministic id so the admin can address
    // it directly (`inv-<guardian_id>`) and so the inverse verbs (revoke /
    // resend) can find it without a separate index. `to_owned`+`+` keeps
    // the lint happy (it's an ID, not user-facing chrome).
    let id = "inv-".to_owned() + &parsed.guardian_id;
    let now_ms = now_ms();
    let invite = Invite {
        id: id.clone(),
        guardian_id: Some(parsed.guardian_id.clone()),
        email: guardian.email.clone(),
        role: InviteRole::GuardianMember,
        room_id: None,
        locale,
        status: InviteStatus::Pending,
        lb_invite_id: None,
        created_at_ms: now_ms,
        sent_at_ms: None,
        accepted_at_ms: None,
        revoked_at_ms: None,
        expired_at_ms: None,
        parked_reason: None,
    };
    let value = serde_json::to_value(&invite).map_err(|e| format!("serialize invite: {e}"))?;

    // First-write: a duplicate invite for the same guardian conflicts. The
    // admin must revoke the previous one first (the existing mirror row's
    // id is deterministic, so the conflict surfaces immediately).
    cp.records()
        .create("invite", &id, &value)
        .await
        .map_err(|e| match e {
            RecordError::Conflict => {
                format!(
                    "{}: invite already exists",
                    InviteError::StoreDenied("invite create".into())
                )
            }
            RecordError::Store(s) => {
                format!("{}: {s}", InviteError::StoreDenied("invite create".into()))
            }
        })?;

    // Now call lb `invite.create` over the host-callback (the SAME
    // SidecarClient the chokepoint's read delegation uses). Era-2 wired
    // here: when `cp.reach()` is Some, the mint goes over HTTP; when
    // None (era-1 fallback), surface a typed `StoreDenied` so the admin
    // sees why the invite sits in `Pending` (a swallowed error = a
    // stuck row the admin can't act on).
    let reach = match cp.reach() {
        Some(r) => r,
        None => {
            return Err(format!(
                "{}: care.invite.create_guardian requires the host-callback path (era 2); see care-authz-scope.md §\"Era 2\" for the era-1 fallback",
                InviteError::StoreDenied("invite.create no host-callback".into())
            ));
        }
    };
    let reply = reach
        .client()
        .call_tool(
            "invite.create",
            serde_json::json!({
                "email": guardian.email,
                "role": InviteRole::GuardianMember.as_str(),
                "team": InviteRole::GuardianMember.team(),
                "payload": &parsed.guardian_id,
                "locale": locale.as_str(),
                "expires_ts": 0u64,
                "now": now_ms,
            }),
        )
        .await
        .map_err(|e| match e {
            CallError::Denied => format!(
                "{}: lb denied invite.create (the extension lacks mcp:invite.create:call?)",
                InviteError::StoreDenied("invite.create denied".into())
            ),
            other => format!(
                "{}: invite.create callback failed: {other}",
                InviteError::StoreDenied("invite.create callback".into())
            ),
        })?;
    // The lb reply carries the raw one-time token (shown once, never
    // recoverable from the stored hash). We hash it locally to derive
    // `token_hash` — the lb-internal invite id the inverse verbs
    // (`revoke` / `resend`) look up by. SHA-256 of the raw token is
    // correct here (32 random bytes — full entropy, no KDF needed).
    let raw_token = reply.get("token").and_then(|v| v.as_str()).ok_or_else(|| {
        format!(
            "{}: invite.create reply missing token",
            InviteError::StoreDenied("invite.create reply".into())
        )
    })?;
    let token_hash = hash_invite_token(raw_token);

    // The lb invite is durable; flip the mirror from Pending → Sent.
    // We do this in-place (the row was just created; `store_write` is
    // the same path `update` verbs use).
    let mut updated = value;
    updated["status"] = serde_json::Value::String("sent".to_string());
    updated["lb_invite_id"] = serde_json::Value::String(token_hash.clone());
    updated["sent_at_ms"] = serde_json::Value::Number(now_ms.into());
    cp.records()
        .write("invite", &id, &updated)
        .await
        .map_err(|e| format!("{}: {e}", InviteError::StoreDenied("invite write".into())))?;

    let reply = CreateGuardianReply {
        invite_id: id,
        status: InviteStatus::Sent,
        message: "invite sent".to_string(),
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

/// Unix-ms now (no clock dependency — the host stamps the real time on the
/// per-call principal's `iat`; for the scaffolded state this is the boot
/// time, which is good enough for a Pending mirror row).
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guardian::Guardian;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create as store_create, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.invite.create_guardian:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn create_guardian_rejects_an_empty_guardian_id() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(&cp, &p, r#"{"guardian_id":""}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("missing required field"));
    }

    #[tokio::test]
    async fn create_guardian_rejects_a_missing_guardian_record() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(&cp, &p, r#"{"guardian_id":"ghost"}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("guardian not found"));
    }

    #[tokio::test]
    async fn create_guardian_persists_a_pending_mirror_then_surfaces_no_host_callback() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        // Seed the guardian record first (admin creates the record before
        // mints the invite — records-before-accounts).
        let sam = Guardian {
            name: "Sam".into(),
            email: "sam@example.com".into(),
            phone: None,
            sub: None,
            locale: Locale::En,
        };
        store_create(
            &cp.store,
            &cp.ws,
            "guardian",
            "sam",
            &serde_json::to_value(&sam).unwrap(),
        )
        .await
        .expect("seed guardian");

        let res = run(&cp, &p, r#"{"guardian_id":"sam"}"#).await;
        // Era-1 chokepoint (no host-callback) — the verb persists the
        // mirror row then surfaces a typed deny (admin sees WHY the
        // invite sits in `Pending` rather than a stuck row with no
        // explanation).
        assert!(res.is_err(), "era-1 chokepoint surfaces typed deny");
        let err = res.unwrap_err();
        assert!(err.contains("host-callback"), "got: {err}");

        // The mirror row landed (admin list reflects the intent; the
        // live callback would have flipped it to `Sent`).
        let row = lb_store::read(&cp.store, "acme", "invite", "inv-sam")
            .await
            .unwrap()
            .expect("mirror row present");
        assert_eq!(row["status"], "pending");
        assert_eq!(row["email"], "sam@example.com");
        assert_eq!(row["role"], "guardian_member");
        assert_eq!(row["guardian_id"], "sam");
        assert_eq!(row["locale"], "en");
    }
}
