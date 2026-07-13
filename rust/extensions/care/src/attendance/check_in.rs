//! `care.attendance.check_in` — APPEND a check-in event to the ledger.
//! Cap: `mcp:care.attendance.check_in:call` (staff OR kiosk key).
//!
//! ## A staff/kiosk WRITE, not a guardian read — no reach gate here
//!
//! The check-in is performed by a staff member or a lobby kiosk key; both hold
//! the `check_in` cap at the host wall (attendance-scope §"How it fits": staff
//! verbs deny-tested for guardians, kiosk key scoped to exactly its two verbs).
//! A guardian holds no such cap, so they never reach this body. There is thus
//! NO `assert_reach` here — that gate protects guardian READS of child records,
//! not a staff/kiosk write. `performed_by` is `principal.sub()` (the staff sub
//! or the kiosk `key:` id) for the audit trail.
//!
//! ## Exactly one of child_id / staff_sub
//!
//! One ledger carries both child check-ins and staff-presence events
//! (records.rs §"Staff presence in the SAME table"): a child event sets
//! `child_id`, a staff-presence event sets `staff_sub`. Exactly one must be
//! present — both or neither is a malformed tap (MissingField).
//!
//! ## First-write append — the ledger never overwrites
//!
//! `cp.records().create` is a first-write; a duplicate `event_id` is a
//! Conflict → AlreadyExists. A wrong tap is fixed by a compensating correction
//! event (`correction_of`), never by re-writing this id (records.rs §"An
//! append-only ledger, never edits").

use lb_auth::Principal;

use crate::attendance::{validate_timestamp, AttendanceError, AttendanceEvent, EventKind};
use crate::authz::{Chokepoint, RecordError};
use crate::center::Locale;
use crate::feed::publish_entry;
use crate::i18n::t;
use crate::log::feed_subject;

#[derive(Debug, serde::Deserialize)]
pub struct CheckInInput {
    pub event_id: String,
    #[serde(default)]
    pub child_id: Option<String>,
    #[serde(default)]
    pub staff_sub: Option<String>,
    pub room_id: String,
    pub at: String,
    #[serde(default)]
    pub person: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CheckInReply {
    pub event_id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: CheckInInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.attendance.check_in input: {e}"))?;

    // Validate the event id (first-write key) before anything touches the store.
    if parsed.event_id.is_empty() || parsed.event_id.len() > 64 {
        return Err(format!(
            "{}",
            AttendanceError::InvalidId(parsed.event_id.clone())
        ));
    }
    // EXACTLY ONE of child_id / staff_sub — a child check-in XOR a staff-presence
    // event. Both or neither is a malformed tap.
    let child_id = parsed.child_id.filter(|s| !s.trim().is_empty());
    let staff_sub = parsed.staff_sub.filter(|s| !s.trim().is_empty());
    match (&child_id, &staff_sub) {
        (Some(_), Some(_)) | (None, None) => {
            return Err(format!(
                "{}",
                AttendanceError::MissingField("child_id|staff_sub")
            ));
        }
        _ => {}
    }
    validate_timestamp(&parsed.at).map_err(|e| format!("{e}"))?;
    if parsed.room_id.trim().is_empty() {
        return Err(format!("{}", AttendanceError::MissingField("room_id")));
    }

    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // WHO tapped — a staff sub or a kiosk `key:` id (audit; host wall already
    // proved they hold the cap).
    let performed_by = principal.sub().to_string();

    // Best-effort display for the confirmation string: the named person if the
    // staff selected one, else the subject of the event.
    let display = parsed
        .person
        .as_deref()
        .or(child_id.as_deref())
        .or(staff_sub.as_deref())
        .unwrap_or("")
        .to_string();

    let event = AttendanceEvent {
        kind: EventKind::CheckIn,
        child_id,
        staff_sub,
        room_id: parsed.room_id,
        at: parsed.at,
        performed_by,
        person: parsed.person,
        correction_of: None,
        pickup_override: false,
        override_reason: None,
        note: parsed.note,
    };
    let value = serde_json::to_value(&event).map_err(|e| format!("serialize event: {e}"))?;

    // First-write append — a duplicate event id is a Conflict (the ledger never
    // overwrites; a wrong tap is corrected, not clobbered).
    cp.records()
        .create("attendance_event", &parsed.event_id, &value)
        .await
        .map_err(|e| match e {
            RecordError::Conflict => {
                format!(
                    "{}",
                    AttendanceError::AlreadyExists(parsed.event_id.clone())
                )
            }
            RecordError::Store(s) => {
                format!("{}: {s}", AttendanceError::StoreDenied("check_in".into()))
            }
        })?;

    // Attendance → feed emit (the m06 deferral, wired at m08): a CHILD arrival
    // appears in the guardian's live feed onto the same per-child subject the
    // daily-feed entries use (`feed_subject`). Best-effort (the ledger row is the
    // source of truth; a bus fault never fails the tap); staff-presence events
    // (no child) never emit — they are not a family-facing feed item.
    if let Some(child_id) = event.child_id.as_deref() {
        publish_entry(cp.host_client(), &feed_subject(child_id), &value).await;
    }

    let reply = CheckInReply {
        message: t(locale, "attendance.checked_in", &[("name", &display)]),
        event_id: parsed.event_id,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{read, Store};
    use std::sync::Arc;

    /// A staff/kiosk principal holding exactly the check_in cap (Role::Member —
    /// staff, not admin; the cap, not the role, opens this verb).
    fn staff(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:teacher".into(),
            ws: ws.into(),
            role: Role::Member,
            caps: vec!["mcp:care.attendance.check_in:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn child_check_in_appends_and_round_trips() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = staff(&key, "acme");

        let out = run(
            &cp,
            &p,
            r#"{"event_id":"evt:1","child_id":"child:leo","room_id":"room:possums","at":"2026-07-14T08:02:00Z","person":"Sam"}"#,
        )
        .await
        .expect("check_in");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["event_id"], "evt:1");

        let row = read(&store, "acme", "attendance_event", "evt:1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row["kind"], "check_in");
        assert_eq!(row["child_id"], "child:leo");
        assert_eq!(row["room_id"], "room:possums");
        assert_eq!(row["performed_by"], "user:teacher");
        assert!(
            row.get("staff_sub").is_none(),
            "staff_sub omitted for a child event"
        );
    }

    #[tokio::test]
    async fn staff_presence_check_in_appends() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = staff(&key, "acme");

        run(
            &cp,
            &p,
            r#"{"event_id":"evt:s1","staff_sub":"user:ana","room_id":"room:possums","at":"2026-07-14T07:55:00Z"}"#,
        )
        .await
        .expect("staff check_in");

        let row = read(&store, "acme", "attendance_event", "evt:s1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row["kind"], "check_in");
        assert_eq!(row["staff_sub"], "user:ana");
        assert!(
            row.get("child_id").is_none(),
            "child_id omitted for a staff event"
        );
    }

    #[tokio::test]
    async fn both_child_and_staff_is_rejected() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = staff(&key, "acme");
        let res = run(
            &cp,
            &p,
            r#"{"event_id":"evt:2","child_id":"child:leo","staff_sub":"user:ana","room_id":"room:possums","at":"2026-07-14T08:02:00Z"}"#,
        )
        .await;
        assert!(res.is_err(), "both child_id and staff_sub set must reject");
    }

    #[tokio::test]
    async fn neither_child_nor_staff_is_rejected() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = staff(&key, "acme");
        let res = run(
            &cp,
            &p,
            r#"{"event_id":"evt:3","room_id":"room:possums","at":"2026-07-14T08:02:00Z"}"#,
        )
        .await;
        assert!(
            res.is_err(),
            "neither child_id nor staff_sub set must reject"
        );
    }

    #[tokio::test]
    async fn duplicate_event_id_conflicts() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = staff(&key, "acme");
        let input = r#"{"event_id":"evt:dup","child_id":"child:leo","room_id":"room:possums","at":"2026-07-14T08:02:00Z"}"#;
        run(&cp, &p, input).await.expect("first append");
        let res = run(&cp, &p, input).await;
        assert!(
            res.is_err(),
            "the ledger never overwrites — a dup event_id conflicts"
        );
        assert!(res.unwrap_err().contains("already exists"));
    }
}
