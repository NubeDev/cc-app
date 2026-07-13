//! `care.enrollment.list` — list enrollments (admin-only this milestone).
//! Cap: `mcp:care.enrollment.list:call`.
//!
//! Rule-7 deny semantic for lists: a non-admin principal gets the empty list
//! (NOT an error). Per-room/per-center staff scoping is a milestone-03
//! follow-up; today admin sees every enrollment, everyone else sees none.
//!
//! Waitlist ordering: rows are sorted so a room's `waitlist` entries appear in
//! FIFO order (`waitlist_seq` ASC) — the stable order stamped at create.

use lb_auth::Principal;

use crate::authz::{reachable_children, Chokepoint};
use crate::enrollment::{Enrollment, EnrollmentStatus};

#[derive(Debug, Default, serde::Deserialize)]
pub struct ListInput {
    #[serde(default)]
    pub room_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    // Admin-only this milestone; a non-admin gets the empty list (rule-7
    // deny-by-empty, never an error).
    let is_admin = principal.role() == lb_auth::Role::WorkspaceAdmin
        || principal.role() == lb_auth::Role::SuperAdmin;
    if !is_admin {
        let _ = reachable_children(cp, principal).await; // touch the chokepoint
        return Ok("[]".to_string());
    }

    // Optional filters — an empty/absent input is "list everything".
    let filter: ListInput = if input.trim().is_empty() {
        ListInput::default()
    } else {
        serde_json::from_str(input)
            .map_err(|e| format!("invalid care.enrollment.list input: {e}"))?
    };
    let status_filter = match &filter.status {
        Some(s) => Some(EnrollmentStatus::parse(s).map_err(|e| format!("{e}"))?),
        None => None,
    };

    let data_rows: Vec<serde_json::Value> = cp
        .records()
        .query_data("enrollment")
        .await
        .map_err(|e| format!("store denied the enrollment list: {e}"))?;

    let mut out: Vec<Enrollment> = Vec::new();
    for row in data_rows {
        let Ok(e) = serde_json::from_value::<Enrollment>(row) else {
            continue;
        };
        if let Some(room) = &filter.room_id {
            if &e.room_id != room {
                continue;
            }
        }
        if let Some(status) = status_filter {
            if e.status != status {
                continue;
            }
        }
        out.push(e);
    }

    // FIFO: order a room's waitlist by its stamped sequence. Group by room,
    // then by waitlist_seq ascending — waitlist entries lead in stable order.
    out.sort_by(|a, b| {
        a.room_id
            .cmp(&b.room_id)
            .then(a.waitlist_seq.cmp(&b.waitlist_seq))
    });

    serde_json::to_string(&out).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enrollment::create as enrollment_create;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.enrollment.create:call".into(),
                "mcp:care.enrollment.list:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    fn staff(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:staff".into(),
            ws: ws.into(),
            role: Role::Member,
            caps: vec!["mcp:care.enrollment.list:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn admin_lists_all_enrollments() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        enrollment_create::run(
            &cp,
            &p,
            r#"{"child_id":"leo","room_id":"possums","status":"enrolled"}"#,
        )
        .await
        .unwrap();
        enrollment_create::run(
            &cp,
            &p,
            r#"{"child_id":"mia","room_id":"koalas","status":"enrolled"}"#,
        )
        .await
        .unwrap();

        let out = run(&cp, &p, "").await.unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(v.len(), 2);
    }

    #[tokio::test]
    async fn non_admin_gets_an_empty_list() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let admin_p = admin(&key, "acme");
        enrollment_create::run(
            &cp,
            &admin_p,
            r#"{"child_id":"leo","room_id":"possums","status":"enrolled"}"#,
        )
        .await
        .unwrap();

        let staff_p = staff(&key, "acme");
        let out = run(&cp, &staff_p, "").await.unwrap();
        assert_eq!(out, "[]", "non-admin list ⇒ empty, not error");
    }

    #[tokio::test]
    async fn waitlist_is_ordered_by_seq() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        // Three waitlisted into the same room, in FIFO join order.
        for child in ["ana", "ben", "cy"] {
            enrollment_create::run(
                &cp,
                &p,
                &format!(r#"{{"child_id":"{child}","room_id":"possums","status":"waitlist"}}"#),
            )
            .await
            .unwrap();
        }

        let out = run(&cp, &p, r#"{"room_id":"possums","status":"waitlist"}"#)
            .await
            .unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(v.len(), 3);
        // FIFO: seq 1, 2, 3 in that order.
        assert_eq!(v[0]["waitlist_seq"], 1);
        assert_eq!(v[0]["child_id"], "ana");
        assert_eq!(v[1]["waitlist_seq"], 2);
        assert_eq!(v[1]["child_id"], "ben");
        assert_eq!(v[2]["waitlist_seq"], 3);
        assert_eq!(v[2]["child_id"], "cy");
    }
}
