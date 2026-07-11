//! `care.enrollment.create` — admin enrolls (or waitlists) a child in a room.
//! Cap: `mcp:care.enrollment.create:call`. Admin-only.
//!
//! Deterministic id: `"{child_id}::{room_id}"` — one child↔room pair is one
//! enrollment (first-write; a duplicate pair ⇒ `AlreadyExists`).
//!
//! WAITLIST FIFO PER ROOM (`enrollment-invites-scope.md`): when the new
//! enrollment is a `waitlist` entry, its `waitlist_seq` is stamped as one past
//! the highest `waitlist_seq` already held by a `waitlist` row in the SAME
//! room — a monotonic per-room sequence. It is stable across withdrawals: a
//! mid-list withdrawal never renumbers the rest. A directly-`enrolled` /
//! `withdrawn` row carries `waitlist_seq = 0`.
//!
//! All validation runs BEFORE the store call so a malformed enrollment never
//! lands.

use lb_auth::Principal;
use lb_store::{create as store_create, StoreError};

use crate::authz::{assert_reach, Chokepoint};
use crate::center::Locale;
use crate::enrollment::{Enrollment, EnrollmentError, EnrollmentStatus, Weekday};
use crate::i18n::t;

#[derive(Debug, serde::Deserialize)]
pub struct CreateInput {
    pub child_id: String,
    pub room_id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub schedule: Vec<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CreateReply {
    pub id: String,
    pub status: &'static str,
    pub waitlist_seq: u64,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: CreateInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.enrollment.create input: {e}"))?;

    // Required identifiers (a child↔room pair is the enrollment's identity).
    if parsed.child_id.trim().is_empty() {
        return Err(format!("{}", EnrollmentError::MissingField("child_id")));
    }
    if parsed.room_id.trim().is_empty() {
        return Err(format!("{}", EnrollmentError::MissingField("room_id")));
    }

    // Parse status (default: waitlist) and schedule (each entry a weekday key).
    let status = match &parsed.status {
        Some(s) => EnrollmentStatus::parse(s).map_err(|e| format!("{e}"))?,
        None => EnrollmentStatus::Waitlist,
    };
    let mut schedule: Vec<Weekday> = Vec::with_capacity(parsed.schedule.len());
    for day in &parsed.schedule {
        schedule.push(Weekday::parse(day).map_err(|e| format!("{e}"))?);
    }

    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // FIFO per room: stamp the next monotonic `waitlist_seq` only for a
    // waitlist entry — one past the room's current highest. Enrolled/withdrawn
    // carry 0.
    let waitlist_seq = if status == EnrollmentStatus::Waitlist {
        next_waitlist_seq(cp, &parsed.room_id).await?
    } else {
        0
    };

    let enrollment = Enrollment {
        child_id: parsed.child_id.clone(),
        room_id: parsed.room_id.clone(),
        status,
        schedule,
        waitlist_seq,
        start_date: parsed.start_date,
    };
    let value =
        serde_json::to_value(&enrollment).map_err(|e| format!("serialize enrollment: {e}"))?;

    let id = format!("{}::{}", parsed.child_id, parsed.room_id);

    // First-write (a duplicate child↔room pair ⇒ AlreadyExists).
    store_create(&cp.store, &cp.ws, "enrollment", &id, &value)
        .await
        .map_err(|e| match e {
            StoreError::Conflict => format!("{}", EnrollmentError::AlreadyExists(id.clone())),
            other => format!("{}: {other}", EnrollmentError::StoreDenied("create".into())),
        })?;

    // Admin audit through the chokepoint (one audit point; the wall already
    // denied staff/guardian by cap).
    let _ = assert_reach(cp, principal, &parsed.child_id).await;

    let message = if status == EnrollmentStatus::Waitlist {
        t(
            locale,
            "enrollment.waitlisted",
            &[
                ("child", &parsed.child_id),
                ("room", &parsed.room_id),
                ("position", &waitlist_seq.to_string()),
            ],
        )
    } else {
        t(
            locale,
            "enrollment.created",
            &[("child", &parsed.child_id), ("room", &parsed.room_id)],
        )
    };

    let reply = CreateReply {
        id,
        status: status.as_str(),
        waitlist_seq,
        message,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

/// Compute the next FIFO waitlist sequence for `room_id`: one past the highest
/// `waitlist_seq` held by an existing `waitlist` row in that room. Monotonic
/// and stable — a withdrawal frees no number, so later joins never reuse it.
async fn next_waitlist_seq(cp: &Chokepoint, room_id: &str) -> Result<u64, String> {
    let mut resp = cp
        .store
        .query_ws(&cp.ws, "SELECT * FROM enrollment", vec![])
        .await
        .map_err(|e| format!("{}: {e}", EnrollmentError::StoreDenied("waitlist scan".into())))?;
    let data_rows: Vec<serde_json::Value> = resp
        .take::<Vec<serde_json::Value>>((0, "data"))
        .unwrap_or_default();

    let mut max_seq: u64 = 0;
    for row in data_rows {
        let Ok(e) = serde_json::from_value::<Enrollment>(row) else {
            continue;
        };
        if e.room_id == room_id && e.status == EnrollmentStatus::Waitlist {
            max_seq = max_seq.max(e.waitlist_seq);
        }
    }
    Ok(max_seq + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{read, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.enrollment.create:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn create_enrolled_stamps_zero_seq() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        let out = run(
            &cp,
            &p,
            r#"{"child_id":"leo","room_id":"possums","status":"enrolled","schedule":["mon","wed"]}"#,
        )
        .await
        .expect("create");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["id"], "leo::possums");
        assert_eq!(v["status"], "enrolled");
        assert_eq!(v["waitlist_seq"], 0);

        let row = read(&store, "acme", "enrollment", "leo::possums")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row["status"], "enrolled");
        assert_eq!(row["schedule"][1], "wed");
    }

    #[tokio::test]
    async fn create_waitlist_stamps_seq_one() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let out = run(&cp, &p, r#"{"child_id":"leo","room_id":"possums"}"#)
            .await
            .expect("create");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        // Default status is waitlist.
        assert_eq!(v["status"], "waitlist");
        assert_eq!(v["waitlist_seq"], 1);
        assert!(v["message"].as_str().unwrap().contains("position 1"));
    }

    #[tokio::test]
    async fn waitlist_is_fifo_two_children_get_seq_one_and_two() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let a = run(
            &cp,
            &p,
            r#"{"child_id":"ana","room_id":"possums","status":"waitlist"}"#,
        )
        .await
        .unwrap();
        let b = run(
            &cp,
            &p,
            r#"{"child_id":"ben","room_id":"possums","status":"waitlist"}"#,
        )
        .await
        .unwrap();

        let va: serde_json::Value = serde_json::from_str(&a).unwrap();
        let vb: serde_json::Value = serde_json::from_str(&b).unwrap();
        assert_eq!(va["waitlist_seq"], 1);
        assert_eq!(vb["waitlist_seq"], 2);
    }

    #[tokio::test]
    async fn create_is_first_write_a_duplicate_pair_conflicts() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let input = r#"{"child_id":"leo","room_id":"possums","status":"enrolled"}"#;
        run(&cp, &p, input).await.expect("first write");
        let res = run(&cp, &p, input).await;
        assert!(res.is_err(), "second create of the same pair must fail");
        assert!(res.unwrap_err().contains("already exists"));
    }

    #[tokio::test]
    async fn create_rejects_a_bad_weekday() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(
            &cp,
            &p,
            r#"{"child_id":"leo","room_id":"possums","schedule":["funday"]}"#,
        )
        .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("invalid weekday"));
    }
}
