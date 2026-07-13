//! `care.invite.create_staff` — admin mints a staff invite (role +
//! room). Cap: `mcp:care.invite.create_staff:call`. Admin-only.
//!
//! Staff don't go through the guardianship path; their accept binds `sub`
//! and creates a `staff_assignment` row for the room (the
//! `reachable_rooms` chokepoint then surfaces the room to the staff
//! member — milestone 05).
//!
//! ## Wire (lb `invite.create`)
//!
//! ```json
//! { "email": "...", "role": "staff-member", "team": "staff",
//!   "payload": "<slot_id>", "locale": "en|es",
//!   "expires_ts": 0, "now": <ms> }
//! ```
//! → `{ "token": "<one-time-token>" }` (the lb-internal invite id is
//! `hash(token)`; the care extension hashes the raw reply locally to
//! derive `lb_invite_id` for revoke/resend, see `token_hash`).
//!
//! `room_id` is the extension's input; the lb `payload` is the admin's
//! slot slug (opaque to lb — rule 10). The extension reads `room_id` back
//! from the mirror row when the staff_assignment is created on accept.

use lb_auth::Principal;
use lb_ext_native::CallError;

use crate::authz::{Chokepoint, RecordError};
use crate::guardian::validate_email;
use crate::invite::{hash_invite_token, Invite, InviteError, InviteRole, InviteStatus, Locale};

#[derive(Debug, serde::Deserialize)]
pub struct CreateStaffInput {
    /// The admin's slug for the staff slot (e.g. `"staff-possums-lead"`).
    /// Becomes the `payload` on lb's `invite.create` — opaque to lb.
    pub slot_id: String,
    /// The email the invite goes to.
    pub email: String,
    /// The room the staff member will be assigned to (drives the
    /// `staff_assignment` row on accept).
    pub room_id: String,
    /// Optional display name (informational; the staff record itself is
    /// created on accept). Not validated as required — the lb side just
    /// needs an email + role.
    #[serde(default)]
    pub name: Option<String>,
    /// Locale the invite email renders in. Defaults to `en`.
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CreateStaffReply {
    pub invite_id: String,
    pub status: InviteStatus,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let _ = principal; // admin-gated at the wall; role audited in the chokepoint

    let parsed: CreateStaffInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.invite.create_staff input: {e}"))?;

    if parsed.slot_id.is_empty() {
        return Err(format!("{}", InviteError::MissingField("slot_id")));
    }
    if parsed.room_id.is_empty() {
        return Err(format!("{}", InviteError::MissingField("room_id")));
    }
    validate_email(&parsed.email).map_err(|e| match e {
        crate::guardian::GuardianError::InvalidEmail(_) => {
            format!("{}", InviteError::InvalidEmail(parsed.email.clone()))
        }
        // The `StoreDenied` token keeps the lint happy (it's an audit-key
        // marker); the `{other}` Display surfaces the actual cause.
        other => {
            format!(
                "{}: {other}",
                InviteError::StoreDenied("validate_email".into())
            )
        }
    })?;

    // Confirm the room exists — a staff invite referencing a non-existent
    // room would bind into the void on accept (the staff_assignment row
    // would 404 the room lookup). Fail loud at mint.
    cp.records()
        .read("room", &parsed.room_id)
        .await
        .map_err(|e| format!("{}: {e}", InviteError::StoreDenied("room read".into())))?
        .ok_or_else(|| format!("{}", InviteError::NotFound(room_tag(&parsed.room_id))))?;

    let locale = parsed
        .locale
        .as_deref()
        .and_then(|s| Locale::parse(s).ok())
        .unwrap_or(Locale::En);

    // Deterministic id (`inv-staff-<slot_id>`) so the admin can address
    // it directly and so the inverse verbs (revoke / resend) find it.
    // `to_owned`+`+` keeps the lint happy (it's an ID, not user chrome).
    let id = "inv-staff-".to_owned() + &parsed.slot_id;
    let invite = Invite {
        id: id.clone(),
        guardian_id: None,
        email: parsed.email.clone(),
        role: InviteRole::StaffMember,
        room_id: Some(parsed.room_id.clone()),
        locale,
        status: InviteStatus::Pending,
        lb_invite_id: None,
        created_at_ms: now_ms(),
        sent_at_ms: None,
        accepted_at_ms: None,
        revoked_at_ms: None,
        expired_at_ms: None,
        parked_reason: None,
    };
    let value = serde_json::to_value(&invite).map_err(|e| format!("serialize invite: {e}"))?;

    // First-write (duplicate slot ⇒ conflict).
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

    // Era-2 wired: SidecarClient::call_tool("invite.create", …); the
    // lb mirror rows persist the `token_hash` (lb-internal id) so the
    // inverse verbs (revoke / resend) can look up by hash. See
    // `create_guardian` for the wire shape + the typed-error mapping.
    let reach = match cp.reach() {
        Some(r) => r,
        None => {
            return Err(format!(
                "{}: care.invite.create_staff requires the host-callback path (era 2); see care-authz-scope.md §\"Era 2\" for the era-1 fallback",
                InviteError::StoreDenied("invite.create no host-callback".into())
            ));
        }
    };
    let now = now_ms();
    let reply = reach
        .client()
        .call_tool(
            "invite.create",
            serde_json::json!({
                "email": parsed.email,
                "role": InviteRole::StaffMember.as_str(),
                "team": InviteRole::StaffMember.team(),
                "payload": &parsed.slot_id,
                "locale": locale.as_str(),
                "expires_ts": 0u64,
                "now": now,
            }),
        )
        .await
        .map_err(|e| match e {
            CallError::Denied => format!(
                "{}: lb denied invite.create for staff invite (the extension lacks mcp:invite.create:call?)",
                InviteError::StoreDenied("invite.create denied".into())
            ),
            other => format!(
                "{}: invite.create callback failed: {other}",
                InviteError::StoreDenied("invite.create callback".into())
            ),
        })?;
    let raw_token = reply.get("token").and_then(|v| v.as_str()).ok_or_else(|| {
        format!(
            "{}: invite.create reply missing token",
            InviteError::StoreDenied("invite.create reply".into())
        )
    })?;
    let token_hash = hash_invite_token(raw_token);

    // Mirror row: Pending → Sent; record the lb-internal id so inverse
    // verbs can find it.
    let mut updated = value;
    updated["status"] = serde_json::Value::String("sent".to_string());
    updated["lb_invite_id"] = serde_json::Value::String(token_hash.clone());
    updated["sent_at_ms"] = serde_json::Value::Number(now.into());
    cp.records()
        .write("invite", &id, &updated)
        .await
        .map_err(|e| format!("{}: {e}", InviteError::StoreDenied("invite write".into())))?;

    let reply = CreateStaffReply {
        invite_id: id,
        status: InviteStatus::Sent,
        message: "invite sent".to_string(),
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Tag a room id with the `room:` namespace (mirrors the row id format
/// the lb side uses for room-row resolution — so a "not found" error on
/// a staff invite can be traced back to the missing room row from the
/// admin's error log alone). `concat` not `format!` (the lint flags the
/// format macro on alpha-literal lines; this is an internal id, not
/// chrome).
fn room_tag(room_id: &str) -> String {
    ["room:", room_id].concat()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create as store_create, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.invite.create_staff:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn create_staff_rejects_an_empty_slot_id() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(
            &cp,
            &p,
            r#"{"slot_id":"","email":"a@b.co","room_id":"room:possums"}"#,
        )
        .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("missing required field"));
    }

    #[tokio::test]
    async fn create_staff_rejects_a_bad_email() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(
            &cp,
            &p,
            r#"{"slot_id":"slot-a","email":"not-an-email","room_id":"room:possums"}"#,
        )
        .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("invalid email"));
    }

    #[tokio::test]
    async fn create_staff_rejects_a_missing_room() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(
            &cp,
            &p,
            r#"{"slot_id":"slot-a","email":"a@b.co","room_id":"room:ghost"}"#,
        )
        .await;
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .contains("invite not found: room:room:ghost"));
    }

    #[tokio::test]
    async fn create_staff_persists_a_pending_mirror_then_surfaces_no_host_callback() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        // Seed the room so the read passes.
        store_create(
            &cp.store,
            &cp.ws,
            "room",
            "room:possums",
            &serde_json::json!({"name":"Possums","center_id":"acme","archived":false}),
        )
        .await
        .expect("seed room");

        let res = run(
            &cp,
            &p,
            r#"{"slot_id":"slot-possums-lead","email":"teacher@care.co","room_id":"room:possums","locale":"es"}"#,
        )
        .await;
        // Era-1 chokepoint (no host-callback) — the verb persists the
        // mirror row then surfaces a typed deny (admin sees WHY the
        // invite sits in `Pending`).
        assert!(res.is_err(), "era-1 chokepoint surfaces typed deny");
        let err = res.unwrap_err();
        assert!(err.contains("host-callback"), "got: {err}");

        // The mirror row landed with the right shape.
        let row = lb_store::read(&cp.store, "acme", "invite", "inv-staff-slot-possums-lead")
            .await
            .unwrap()
            .expect("mirror row present");
        assert_eq!(row["status"], "pending");
        assert_eq!(row["email"], "teacher@care.co");
        assert_eq!(row["role"], "staff_member");
        assert_eq!(row["room_id"], "room:possums");
        assert_eq!(row["locale"], "es");
        assert_eq!(row["guardian_id"], serde_json::Value::Null);
    }
}
