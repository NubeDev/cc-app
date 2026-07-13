//! `care.log.correct` — APPEND a compensating `daily_log` entry that corrects a
//! prior one. Cap: `mcp:care.log.correct:call` (admin/staff; guardians hold no
//! such cap and never reach this body — they read + acknowledge, never author).
//!
//! ## A correction is an APPEND, never an edit — the feed is an audit ledger
//!
//! A `daily_log` row is written once and read by regulators and the center's
//! incident file (records.rs §"Append + compensating correction"). A wrong
//! entry is NOT patched in place: we append a NEW row whose `correction_of`
//! points at the offending one, and the original row stays untouched forever
//! for audit. Same posture as the attendance ledger (`attendance/correct.rs`).
//!
//! ## Same child + room + kind as the original — a correction can't retarget
//!
//! A correction refers to the SAME child, room, and entry TYPE as the row it
//! corrects — the caller cannot silently move an entry to a different child or
//! turn a nap into an incident. We read the original, copy its `child_id`/
//! `room_id`/`kind` into the new row, and stamp `author = principal.sub()` for
//! the audit trail (WHO issued the correction).
//!
//! ## Payload override decision — replace-or-copy per field
//!
//! The corrected fields ARE the caller's point: an incident logged with the
//! wrong `action`, a meal with the wrong portion. So the input MAY carry
//! replacement type-payloads (nap/meal/incident/medication) and `media_ids` to
//! supersede the original's; each is OPTIONAL and, when ABSENT, copies the
//! original's value (a note-only fix keeps the original payload intact). The
//! resulting row is re-validated with `kind.validate` so a correction can never
//! DROP a regulated field (an incident correction must still carry
//! what/where/action).
//!
//! ## First-write on the NEW id — the original is read, not overwritten
//!
//! `create("daily_log", &entry_id, ...)` is a first-write on a FRESH id; the
//! corrected row's id (`correction_of`) is only READ. A duplicate new id is a
//! Conflict → AlreadyExists (the ledger never overwrites).

use lb_auth::Principal;

use crate::authz::{Chokepoint, RecordError};
use crate::center::Locale;
use crate::i18n::t;
use crate::log::{
    validate_timestamp, DailyLog, IncidentPayload, LogError, MealPayload, MedicationPayload,
    NapPayload,
};

#[derive(Debug, serde::Deserialize)]
pub struct CorrectInput {
    /// The id of the NEW compensating entry (first-write key).
    pub entry_id: String,
    /// The id of the entry being corrected (read-only reference).
    pub correction_of: String,
    /// The correction's timestamp (device wall time, ISO-8601).
    pub at: String,
    #[serde(default)]
    pub note: Option<String>,
    /// Optional replacement payloads — when absent, the original's are copied.
    #[serde(default)]
    pub nap: Option<NapPayload>,
    #[serde(default)]
    pub meal: Option<MealPayload>,
    #[serde(default)]
    pub incident: Option<IncidentPayload>,
    #[serde(default)]
    pub medication: Option<MedicationPayload>,
    /// Optional replacement media ids — when absent, the original's are copied.
    #[serde(default)]
    pub media_ids: Option<Vec<String>>,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CorrectReply {
    pub entry_id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: CorrectInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.log.correct input: {e}"))?;

    // Validate the NEW entry id (first-write key) before touching the store.
    if parsed.entry_id.is_empty() || parsed.entry_id.len() > 64 {
        return Err(format!("{}", LogError::InvalidId(parsed.entry_id.clone())));
    }
    if parsed.correction_of.trim().is_empty() {
        return Err(format!("{}", LogError::MissingField("correction_of")));
    }
    validate_timestamp(&parsed.at).map_err(|e| format!("{e}"))?;

    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // Read the ORIGINAL entry — it is only READ (never written); NotFound if the
    // caller points at an entry that isn't in the ledger.
    let original = cp
        .records()
        .read("daily_log", &parsed.correction_of)
        .await
        .map_err(|e| match e {
            RecordError::Store(s) => {
                format!("{}: {s}", LogError::StoreDenied("log.correct".into()))
            }
            RecordError::Conflict => format!("{}", LogError::StoreDenied("log.correct".into())),
        })?
        .ok_or_else(|| format!("{}", LogError::NotFound(parsed.correction_of.clone())))?;
    let original: DailyLog = serde_json::from_value(original)
        .map_err(|e| format!("deserialize corrected entry: {e}"))?;

    // WHO issued the correction (audit) — a staff/admin sub.
    let author = principal.sub().to_string();
    // The kind is CANONICAL — copied from the original; a correction can't
    // change the entry type (records.rs §"Same child + room + kind").
    let kind = original.kind;

    // Copy child/room/kind from the original; replace payloads + media where the
    // caller supplied them, else copy the original's (a note-only fix keeps the
    // original payload intact). `correction_of` marks this as a correction row.
    let row = DailyLog {
        kind,
        child_id: original.child_id,
        room_id: original.room_id,
        author,
        at: parsed.at,
        note: parsed.note,
        media_ids: parsed.media_ids.unwrap_or(original.media_ids),
        nap: parsed.nap.or(original.nap),
        meal: parsed.meal.or(original.meal),
        incident: parsed.incident.or(original.incident),
        medication: parsed.medication.or(original.medication),
        correction_of: Some(parsed.correction_of),
    };
    // Re-validate the RESULT — a correction can never DROP a regulated field
    // (an incident correction must still carry what/where/action).
    kind.validate(&row).map_err(|e| format!("{e}"))?;

    let value = serde_json::to_value(&row).map_err(|e| format!("serialize entry: {e}"))?;

    // First-write append on the NEW id — the original stays untouched. A
    // duplicate new id is a Conflict (the ledger never overwrites).
    cp.records()
        .create("daily_log", &parsed.entry_id, &value)
        .await
        .map_err(|e| match e {
            RecordError::Conflict => {
                format!("{}", LogError::AlreadyExists(parsed.entry_id.clone()))
            }
            RecordError::Store(s) => {
                format!("{}: {s}", LogError::StoreDenied("log.correct".into()))
            }
        })?;

    let reply = CorrectReply {
        message: t(
            locale,
            "log.corrected",
            &[("type", &t(locale, &kind.label_key(), &[]))],
        ),
        entry_id: parsed.entry_id,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::child::create as child_create;
    use crate::log::{add, entry_id};
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{read, Store};
    use std::sync::Arc;

    /// A staff principal holding the log.add + log.correct caps.
    fn staff(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:teacher".into(),
            ws: ws.into(),
            role: Role::Member,
            caps: vec![
                "mcp:care.log.add:call".into(),
                "mcp:care.log.correct:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    /// An admin (to seed children through the real write path).
    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.child.create:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    async fn seed_child(cp: &Chokepoint, a: &Principal, id: &str) {
        let input =
            format!(r#"{{"id":"{id}","name":"{id}","dob":"2021-03-15","photo_consent":true}}"#);
        child_create::run(cp, a, &input).await.expect("seed child");
    }

    #[tokio::test]
    async fn correction_appends_and_leaves_the_original_untouched() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");
        seed_child(&cp, &a, "child:leo").await;

        // Seed an ORIGINAL note entry via the real write path.
        let p = staff(&key, "acme");
        add::run(
            &cp,
            &p,
            r#"{"entry_id":"log:1","child_ids":["child:leo"],"room_id":"room:possums","kind":"note","at":"2026-07-14T09:00:00Z","note":"typo"}"#,
        )
        .await
        .expect("seed add");
        let orig_id = entry_id("log:1", "child:leo");

        // Correct it — a note-only fix (no replacement payload).
        let out = run(
            &cp,
            &p,
            &format!(
                r#"{{"entry_id":"log:2","correction_of":"{orig_id}","at":"2026-07-14T09:05:00Z","note":"fixed"}}"#
            ),
        )
        .await
        .expect("correct");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["entry_id"], "log:2");

        // The ORIGINAL is unchanged — still a note, no correction_of, same
        // child/room.
        let orig = read(&store, "acme", "daily_log", &orig_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(orig["kind"], "note");
        assert_eq!(orig["child_id"], "child:leo");
        assert_eq!(orig["room_id"], "room:possums");
        assert!(
            orig.get("correction_of").is_none(),
            "original is not a correction"
        );

        // The NEW row carries correction_of + copied child/room/kind + the
        // corrector as author.
        let corr = read(&store, "acme", "daily_log", "log:2")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(corr["kind"], "note", "kind copied from original");
        assert_eq!(corr["correction_of"], orig_id);
        assert_eq!(corr["child_id"], "child:leo", "copied from original");
        assert_eq!(corr["room_id"], "room:possums", "copied from original");
        assert_eq!(corr["author"], "user:teacher");
    }

    #[tokio::test]
    async fn correcting_a_missing_entry_is_not_found() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = staff(&key, "acme");

        let res = run(
            &cp,
            &p,
            r#"{"entry_id":"log:2","correction_of":"log:ghost","at":"2026-07-14T09:05:00Z"}"#,
        )
        .await;
        assert!(res.is_err(), "correcting a non-existent entry must fail");
        assert!(res.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn a_correction_dropping_a_regulated_incident_field_is_rejected() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");
        seed_child(&cp, &a, "child:leo").await;

        // Seed a well-formed ORIGINAL incident via the real write path.
        let p = staff(&key, "acme");
        add::run(
            &cp,
            &p,
            r#"{"entry_id":"log:inc","child_ids":["child:leo"],"room_id":"room:possums","kind":"incident","at":"2026-07-14T15:10:00Z","incident":{"what":"scraped knee","where":"playground","action":"cleaned"}}"#,
        )
        .await
        .expect("seed incident");
        let orig_id = entry_id("log:inc", "child:leo");

        // Correct it supplying an incident payload with an EMPTY action — the
        // correction would DROP a regulated field, so it must reject and no new
        // row must land.
        let res = run(
            &cp,
            &p,
            &format!(
                r#"{{"entry_id":"log:inc:fix","correction_of":"{orig_id}","at":"2026-07-14T15:12:00Z","incident":{{"what":"scraped knee","where":"playground","action":""}}}}"#
            ),
        )
        .await;
        assert!(
            res.is_err(),
            "a correction dropping a regulated field must reject"
        );
        assert!(res.unwrap_err().contains("incident.action"));
        assert!(
            read(&store, "acme", "daily_log", "log:inc:fix")
                .await
                .unwrap()
                .is_none(),
            "no row lands on a rejected correction"
        );
    }
}
