//! `care.attendance.correct` — APPEND a compensating event that corrects a
//! prior one. Cap: `mcp:care.attendance.correct:call` (admin/staff; guardians
//! hold no such cap and never reach this body).
//!
//! ## A correction is an APPEND, never an edit — the ledger is sacred
//!
//! Attendance is an append-only ledger read by regulators (records.rs §"An
//! append-only ledger, never edits"). A wrong tap is NOT patched in place: we
//! append a NEW event whose `correction_of` points at the offending one, and
//! the original row stays untouched forever for audit. `fold_now` folds both
//! events in time order, so a wrong check-in corrected by a compensating
//! check-out nets to absent (occupancy.rs §"correction-aware").
//!
//! ## Same subject + room as the original
//!
//! A correction refers to the SAME child/staff subject and room as the event
//! it corrects — the caller only supplies the corrected DIRECTION (`kind`) and
//! the time. We read the original, copy its `child_id`/`staff_sub`/`room_id`/
//! `person` into the new event, and stamp `performed_by = principal.sub()` for
//! the audit trail (WHO issued the correction). This keeps the subject/room
//! canonical — a correction can never silently retarget a different child.
//!
//! ## First-write on the NEW id — the original is read, not overwritten
//!
//! `create("attendance_event", &event_id, ...)` is a first-write on a FRESH id;
//! the corrected event's id (`correction_of`) is only READ. A duplicate new id
//! is a Conflict → AlreadyExists (the ledger never overwrites).

use lb_auth::Principal;

use crate::attendance::{validate_timestamp, AttendanceEvent, AttendanceError, EventKind};
use crate::authz::{Chokepoint, RecordError};
use crate::center::Locale;
use crate::i18n::t;

#[derive(Debug, serde::Deserialize)]
pub struct CorrectInput {
    /// The id of the NEW compensating event (first-write key).
    pub event_id: String,
    /// The id of the event being corrected (read-only reference).
    pub correction_of: String,
    /// The corrected direction: `check_in` | `check_out`.
    pub kind: String,
    /// The correction's timestamp (device wall time).
    pub at: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CorrectReply {
    pub event_id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: CorrectInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.attendance.correct input: {e}"))?;

    // Validate the NEW event id (first-write key) before touching the store.
    if parsed.event_id.is_empty() || parsed.event_id.len() > 64 {
        return Err(format!("{}", AttendanceError::InvalidId(parsed.event_id.clone())));
    }
    if parsed.correction_of.trim().is_empty() {
        return Err(format!("{}", AttendanceError::MissingField("correction_of")));
    }
    // The corrected DIRECTION — reject anything outside {check_in, check_out}.
    let kind = EventKind::parse(&parsed.kind)
        .ok_or_else(|| format!("{}", AttendanceError::InvalidKind(parsed.kind.clone())))?;
    validate_timestamp(&parsed.at).map_err(|e| format!("{e}"))?;

    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // Read the ORIGINAL event — it is only READ (never written); NotFound if the
    // caller points at an event that isn't in the ledger.
    let original = cp
        .records()
        .read("attendance_event", &parsed.correction_of)
        .await
        .map_err(|e| match e {
            RecordError::Store(s) => {
                format!("{}: {s}", AttendanceError::StoreDenied("correct".into()))
            }
            RecordError::Conflict => {
                format!("{}", AttendanceError::StoreDenied("correct".into()))
            }
        })?
        .ok_or_else(|| format!("{}", AttendanceError::NotFound(parsed.correction_of.clone())))?;
    let original: AttendanceEvent = serde_json::from_value(original)
        .map_err(|e| format!("deserialize corrected event: {e}"))?;

    // WHO issued the correction (audit) — a staff/admin sub.
    let performed_by = principal.sub().to_string();

    // Best-effort display for the confirmation string: the original's named
    // person, else the subject (child or staff) of the corrected event.
    let display = original
        .person
        .as_deref()
        .or(original.child_id.as_deref())
        .or(original.staff_sub.as_deref())
        .unwrap_or("")
        .to_string();

    // Copy the subject + room from the original — a correction refers to the
    // SAME subject and room; only the direction/time (and audit who) change.
    let event = AttendanceEvent {
        kind,
        child_id: original.child_id,
        staff_sub: original.staff_sub,
        room_id: original.room_id,
        at: parsed.at,
        performed_by,
        person: original.person,
        correction_of: Some(parsed.correction_of),
        pickup_override: false,
        override_reason: None,
        note: parsed.note,
    };
    let value = serde_json::to_value(&event).map_err(|e| format!("serialize event: {e}"))?;

    // First-write append on the NEW id — the original stays untouched. A
    // duplicate new id is a Conflict (the ledger never overwrites).
    cp.records()
        .create("attendance_event", &parsed.event_id, &value)
        .await
        .map_err(|e| match e {
            RecordError::Conflict => {
                format!("{}", AttendanceError::AlreadyExists(parsed.event_id.clone()))
            }
            RecordError::Store(s) => {
                format!("{}: {s}", AttendanceError::StoreDenied("correct".into()))
            }
        })?;

    let reply = CorrectReply {
        message: t(locale, "attendance.corrected", &[("name", &display)]),
        event_id: parsed.event_id,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attendance::check_in;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{read, Store};
    use std::sync::Arc;

    /// A staff/admin principal holding the check_in + correct caps.
    fn staff(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:teacher".into(),
            ws: ws.into(),
            role: Role::Member,
            caps: vec![
                "mcp:care.attendance.check_in:call".into(),
                "mcp:care.attendance.correct:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn correction_appends_and_leaves_the_original_untouched() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = staff(&key, "acme");

        // Seed an ORIGINAL check-in via the real write path.
        check_in::run(
            &cp,
            &p,
            r#"{"event_id":"evt:1","child_id":"child:leo","room_id":"room:possums","at":"2026-07-14T08:00:00Z","person":"Sam"}"#,
        )
        .await
        .expect("seed check_in");

        // Correct it with a compensating check-out.
        let out = run(
            &cp,
            &p,
            r#"{"event_id":"evt:2","correction_of":"evt:1","kind":"check_out","at":"2026-07-14T08:01:00Z"}"#,
        )
        .await
        .expect("correct");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["event_id"], "evt:2");

        // BOTH events exist. The ORIGINAL is unchanged (still a check_in, no
        // correction_of, same subject/room).
        let orig = read(&store, "acme", "attendance_event", "evt:1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(orig["kind"], "check_in", "original direction untouched");
        assert_eq!(orig["child_id"], "child:leo");
        assert_eq!(orig["room_id"], "room:possums");
        assert!(orig.get("correction_of").is_none(), "original is not a correction");

        // The NEW event carries correction_of + copied subject/room.
        let corr = read(&store, "acme", "attendance_event", "evt:2")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(corr["kind"], "check_out");
        assert_eq!(corr["correction_of"], "evt:1");
        assert_eq!(corr["child_id"], "child:leo", "copied from original");
        assert_eq!(corr["room_id"], "room:possums", "copied from original");
        assert_eq!(corr["performed_by"], "user:teacher");
    }

    #[tokio::test]
    async fn correcting_a_missing_event_is_not_found() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = staff(&key, "acme");

        let res = run(
            &cp,
            &p,
            r#"{"event_id":"evt:2","correction_of":"evt:ghost","kind":"check_out","at":"2026-07-14T08:01:00Z"}"#,
        )
        .await;
        assert!(res.is_err(), "correcting a non-existent event must fail");
        assert!(res.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn an_invalid_kind_is_rejected() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = staff(&key, "acme");

        // Seed an original so the failure is provably the kind, not NotFound.
        check_in::run(
            &cp,
            &p,
            r#"{"event_id":"evt:1","child_id":"child:leo","room_id":"room:possums","at":"2026-07-14T08:00:00Z"}"#,
        )
        .await
        .expect("seed check_in");

        let res = run(
            &cp,
            &p,
            r#"{"event_id":"evt:2","correction_of":"evt:1","kind":"teleport","at":"2026-07-14T08:01:00Z"}"#,
        )
        .await;
        assert!(res.is_err(), "an invalid kind must reject");
        assert!(res.unwrap_err().contains("invalid kind"));
    }
}
