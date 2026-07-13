//! `care.invite.list` — admin sees the invite queue (pending / sent /
//! accepted / revoked / expired / parked). Cap:
//! `mcp:care.invite.list:call`. Admin-only.
//!
//! ## Source of truth split
//!
//! - The extension's mirror rows (this store, table `"invite"`) carry the
//!   statuses the extension OWNS: `Pending` (set on `create_*`), `Revoked`
//!   (set on `care.invite.revoke`).
//! - lb owns `Sent` / `Accepted` / `Expired` / `Parked` — the extension
//!   mirrors those transitions when the milestone-05 `invite.accepted`
//!   bind hook + the invite relay report them. Today (scaffolded state)
//!   those transitions are TODOs in `create_guardian.rs` /
//!   `create_staff.rs` (the SidecarClient `invite.create` call) and the
//!   bind hook is its own milestone-05 deliverable.
//!
//! So this verb reads what the extension knows TODAY (Pending + Revoked)
//! and labels each row with its mirror state; the milestone-05 session
//! adds the live `Sent` / `Accepted` / `Expired` / `Parked` transitions
//! in the same wire.
//!
//! Filter parameters (`status`, `role`) are optional — admin passes one to
//! narrow the queue view.

use lb_auth::Principal;
use serde::Deserialize;

use crate::authz::Chokepoint;
use crate::invite::{InviteError, InviteRole, InviteStatus};

#[derive(Debug, Deserialize)]
pub struct ListInput {
    /// Optional status filter — `"pending" | "sent" | "accepted" |
    /// "revoked" | "expired" | "parked"`. Absent ⇒ every row.
    #[serde(default)]
    pub status: Option<String>,
    /// Optional role filter — `"guardian_member" | "staff_member"`.
    /// Absent ⇒ every role.
    #[serde(default)]
    pub role: Option<String>,
    /// Locale the reply summary renders in (admin's, or workspace default).
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct InviteRow {
    pub id: String,
    pub guardian_id: Option<String>,
    pub email: String,
    pub role: InviteRole,
    pub room_id: Option<String>,
    pub status: InviteStatus,
    pub locale: String,
    pub created_at_ms: u64,
    pub accepted_at_ms: Option<u64>,
    pub revoked_at_ms: Option<u64>,
    pub parked_reason: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ListReply {
    pub invites: Vec<InviteRow>,
    pub count: usize,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let _ = principal; // admin-gated at the wall; role audited in the chokepoint

    let parsed: ListInput = serde_json::from_str(input).unwrap_or(ListInput {
        status: None,
        role: None,
        locale: None,
    });

    // Validate filters up front so a typo'd `"accept"` ⇒ typed InvalidRole,
    // not silent-empty. Status is a free-form lowercased string from the
    // catalog key; role is the enum.
    if let Some(r) = parsed.role.as_deref() {
        match r {
            "guardian_member" => {}
            "staff_member" => {}
            other => return Err(format!("{}", InviteError::InvalidRole(other.to_string()))),
        }
    }
    let status_filter: Option<InviteStatus> = match parsed.status.as_deref() {
        None => None,
        Some("pending") => Some(InviteStatus::Pending),
        Some("sent") => Some(InviteStatus::Sent),
        Some("accepted") => Some(InviteStatus::Accepted),
        Some("revoked") => Some(InviteStatus::Revoked),
        Some("expired") => Some(InviteStatus::Expired),
        Some("parked") => Some(InviteStatus::Parked),
        Some(other) => {
            return Err(format!(
                "{}: {other:?}",
                InviteError::StoreDenied("invalid status filter".into())
            ))
        }
    };

    // Read every row in the workspace's `invite` table. The SurrealDB
    // list idiom — same shape every verb uses (see `center/list.rs`,
    // `room/list.rs`).
    let data_rows: Vec<serde_json::Value> = cp
        .records()
        .query_data("invite")
        .await
        .map_err(|e| format!("{}: {e}", InviteError::StoreDenied("invite list".into())))?;

    let mut invites: Vec<InviteRow> = Vec::with_capacity(data_rows.len());
    for value in data_rows {
        // Filter by role / status before the projection (cheaper + the
        // reply surface stays minimal).
        let row_role = value
            .get("role")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if let Some(r) = parsed.role.as_deref() {
            if row_role != r {
                continue;
            }
        }
        let row_status = value
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if let Some(s) = status_filter {
            if row_status != s.as_str() {
                continue;
            }
        }

        invites.push(InviteRow {
            id: value
                .get("id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
            guardian_id: value
                .get("guardian_id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            email: value
                .get("email")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
            role: match row_role {
                "guardian_member" => InviteRole::GuardianMember,
                "staff_member" => InviteRole::StaffMember,
                _ => InviteRole::GuardianMember, // typed enum, not freeform
            },
            room_id: value
                .get("room_id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            status: match row_status {
                "pending" => InviteStatus::Pending,
                "sent" => InviteStatus::Sent,
                "accepted" => InviteStatus::Accepted,
                "revoked" => InviteStatus::Revoked,
                "expired" => InviteStatus::Expired,
                "parked" => InviteStatus::Parked,
                _ => InviteStatus::Pending,
            },
            locale: value
                .get("locale")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("en")
                .to_string(),
            created_at_ms: value
                .get("created_at_ms")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            accepted_at_ms: value
                .get("accepted_at_ms")
                .and_then(serde_json::Value::as_u64),
            revoked_at_ms: value
                .get("revoked_at_ms")
                .and_then(serde_json::Value::as_u64),
            parked_reason: value
                .get("parked_reason")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
        });
    }

    // Sort newest-first (admin scans the top of the queue). Stable on
    // `created_at_ms` desc, then `id` asc as a tie-breaker.
    invites.sort_by(|a, b| {
        b.created_at_ms
            .cmp(&a.created_at_ms)
            .then_with(|| a.id.cmp(&b.id))
    });

    let reply = ListReply {
        count: invites.len(),
        invites,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize list reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invite::{Invite, InviteRole, InviteStatus, Locale};
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create as store_create, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.invite.list:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    /// Async seed (NOT `futures::executor::block_on` inside a tokio runtime —
    /// that deadlocks because the store's inner future needs the runtime
    /// the calling thread is parked on).
    async fn seed_invite(store: &lb_store::Store, ws: &str, id: &str, invite: &Invite) {
        let value = serde_json::to_value(invite).unwrap();
        store_create(store, ws, "invite", id, &value)
            .await
            .expect("seed invite");
    }

    fn pending_invite(id: &str, role: InviteRole, created_at_ms: u64) -> Invite {
        Invite {
            id: id.into(),
            guardian_id: if matches!(role, InviteRole::GuardianMember) {
                Some(id.trim_start_matches("inv-").into())
            } else {
                None
            },
            email: "x@example.com".into(),
            role,
            room_id: if matches!(role, InviteRole::StaffMember) {
                Some("room:possums".into())
            } else {
                None
            },
            locale: Locale::En,
            status: InviteStatus::Pending,
            lb_invite_id: None,
            created_at_ms,
            sent_at_ms: None,
            accepted_at_ms: None,
            revoked_at_ms: None,
            expired_at_ms: None,
            parked_reason: None,
        }
    }

    #[tokio::test]
    async fn list_returns_every_mirror_row() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        seed_invite(
            &cp.store,
            &cp.ws,
            "inv-sam",
            &pending_invite("inv-sam", InviteRole::GuardianMember, 100),
        )
        .await;
        seed_invite(
            &cp.store,
            &cp.ws,
            "inv-staff-a",
            &pending_invite("inv-staff-a", InviteRole::StaffMember, 200),
        )
        .await;

        let out = run(&cp, &p, r#"{}"#).await.expect("list");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["count"], 2);
        let ids: Vec<&str> = v["invites"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        // Newest-first ordering.
        assert_eq!(ids, vec!["inv-staff-a", "inv-sam"]);
    }

    #[tokio::test]
    async fn list_filters_by_role() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");
        seed_invite(
            &cp.store,
            &cp.ws,
            "inv-sam",
            &pending_invite("inv-sam", InviteRole::GuardianMember, 100),
        )
        .await;
        let out = run(&cp, &p, r#"{"role":"staff_member"}"#)
            .await
            .expect("list");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["count"], 0);
    }

    #[tokio::test]
    async fn list_rejects_an_unknown_role() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(&cp, &p, r#"{"role":"principal"}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("invalid role"));
    }
}
