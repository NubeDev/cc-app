//! The durable `daily_log` record + its typed error + the bus-subject and
//! push-policy shapes — ORCHESTRATOR-OWNED schema (daily-feed-scope §Goals;
//! subagents never decide record shapes). This is the foundation the whole of
//! milestone 08 hangs off, so it is written first (08-daily-feed §"Subagent
//! notes": fix the `daily_log` schema + bus subject shape FIRST).
//!
//! ## One entry per (type, child) — the fan-out is per-child rows
//!
//! A staff member taps "lunch" for eight children in one gesture; that fans out
//! to eight `daily_log` rows, one per child (daily-feed-scope §Goals: "one tap
//! can log lunch for the whole room — fan-out to per-child entries"). Each row
//! is independently addressable, correctable, reach-filtered, and pushed. The
//! feed is per-CHILD, so the record is per-child; the multi-child add is a
//! bounded synchronous fan-out in `log::add`, not a shared multi-child row.
//!
//! ## Append + compensating correction — never an edit (like attendance)
//!
//! A `daily_log` row is written once. A wrong entry is fixed by a COMPENSATING
//! correction row (`correction_of` points at the corrected entry), never by
//! mutating the row — the same audit posture as the attendance ledger
//! (`attendance/records.rs`). Regulators and the center's incident file read
//! this table.
//!
//! ## Type-specific payload — one flat record, optional fields per type
//!
//! Rather than a tagged union per type (eight structs), the record is one flat
//! shape whose optional fields are populated per `LogKind` (nap carries
//! start/end; meal carries portion; incident carries the required what/where/
//! action + the guardian-ack flag; medication carries dose + witness). The verb
//! layer validates the REQUIRED fields per kind at write (`LogKind::validate`),
//! so an incident can never land without its regulated fields.

use serde::{Deserialize, Serialize};
use std::fmt;

/// One `daily_log` entry (workspace-scoped, append-only). Always for exactly
/// ONE child — the multi-child add fans out to N of these.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailyLog {
    /// The entry type — drives which payload fields are required + the push
    /// policy (`LogKind::push_policy`).
    pub kind: LogKind,
    /// The child this entry is for (a `child` id). Always set (per-child rows).
    pub child_id: String,
    /// The room the entry was logged in (a `room` id) — the staff room scope
    /// the list verb filters by, and the fan-out's shared context.
    pub room_id: String,
    /// The staff author's auth `sub` (audit; who wrote it).
    pub author: String,
    /// The entry time, ISO-8601 (`YYYY-MM-DDTHH:MM:SSZ`); caller-supplied so the
    /// sidecar stays clock-free (no `Date::now` in the child — same posture as
    /// attendance) and an offline room-tablet stamps the true wall time.
    pub at: String,
    /// The free-text note (never translated — recorded as-is). Optional for
    /// most types; the incident required fields live in `incident` below.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Attached photo `media_id`s (from the lb media path). Empty unless the
    /// child's `photos_allowed` consent held at write (enforced in `log::add`,
    /// NOT at render — daily-feed-scope §"Photo consent"). Photos only v1
    /// (video is a scope non-goal; enforced at attach).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub media_ids: Vec<String>,
    /// Nap payload (`start`/`end` ISO strings) — only for [`LogKind::Nap`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nap: Option<NapPayload>,
    /// Meal payload (slot + portion eaten) — only for [`LogKind::Meal`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meal: Option<MealPayload>,
    /// Incident payload (the REQUIRED what/where/action + the guardian-ack
    /// flag) — only for [`LogKind::Incident`]. Regulated; validated hard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incident: Option<IncidentPayload>,
    /// Medication payload (dose + witness) — only for [`LogKind::Medication`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medication: Option<MedicationPayload>,
    /// A compensating correction: the id of the entry this one corrects.
    /// `Some` ⇒ this is a correction row (never an edit — the corrected row
    /// stays for audit, same as attendance).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction_of: Option<String>,
}

/// The eight entry types (daily-feed-scope §Goals). The enum is EXTENSIBLE —
/// phase-3 learning/milestone tagging rides the same record additively (scope
/// non-goal §"Learning/milestone tagging"). Each maps to an
/// `log.type.<key>` catalog key (rendered per-locale in the feed + push).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogKind {
    Meal,
    Nap,
    Diaper,
    Activity,
    Photo,
    Note,
    Incident,
    Medication,
}

impl LogKind {
    pub fn parse(s: &str) -> Option<LogKind> {
        match s.trim().to_ascii_lowercase().as_str() {
            "meal" => Some(LogKind::Meal),
            "nap" => Some(LogKind::Nap),
            "diaper" => Some(LogKind::Diaper),
            "activity" => Some(LogKind::Activity),
            "photo" => Some(LogKind::Photo),
            "note" => Some(LogKind::Note),
            "incident" => Some(LogKind::Incident),
            "medication" => Some(LogKind::Medication),
            _ => None,
        }
    }

    /// The stable key (wire + record + catalog-key suffix).
    pub fn key(&self) -> &'static str {
        match self {
            LogKind::Meal => "meal",
            LogKind::Nap => "nap",
            LogKind::Diaper => "diaper",
            LogKind::Activity => "activity",
            LogKind::Photo => "photo",
            LogKind::Note => "note",
            LogKind::Incident => "incident",
            LogKind::Medication => "medication",
        }
    }

    /// The i18n catalog key for this type's label (`log.type.<key>`), rendered
    /// per-locale in the feed AND server-side in each recipient's push
    /// (daily-feed-scope §Push: "log-type labels are enum keys rendered per
    /// locale").
    pub fn label_key(&self) -> String {
        ["log.type", self.key()].join(".")
    }

    /// The push policy for this type (daily-feed-scope §Push: "entry types map
    /// to notification policy"). An INCIDENT always pushes (must-deliver, never
    /// gated); a MEDICATION always pushes (a dose given is a parent-must-know);
    /// every other type is feed-only unless the guardian opted into per-type
    /// push (the `receives_daily_feed` edge + prefs — resolved in
    /// `push::policy`). This is the ONE place the type→urgency rule lives.
    pub fn push_policy(&self) -> PushPolicy {
        match self {
            // Always — a scraped knee or a dose given is never merely "feed".
            LogKind::Incident | LogKind::Medication => PushPolicy::Always,
            // Feed-first — pushed only if the guardian opted in per prefs.
            _ => PushPolicy::FeedThenPrefs,
        }
    }
}

/// The per-type push urgency (daily-feed-scope §Push). `Always` bypasses the
/// per-guardian prefs + quiet hours (an incident wakes the phone);
/// `FeedThenPrefs` pushes only when the guardian's prefs allow it and outside
/// quiet hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushPolicy {
    /// Always push — incident / medication. Must-deliver via the outbox;
    /// quiet hours do NOT suppress it (safety over convenience).
    Always,
    /// Push only if the guardian's per-type pref allows AND it is outside their
    /// quiet hours; otherwise feed-only.
    FeedThenPrefs,
}

/// Nap payload — start/end ISO strings (either may be open while the child
/// sleeps; the pair is closed by a correction/second entry per the scope's
/// nap start/end shape).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NapPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

/// Meal payload — which slot + how much was eaten (an enum, rendered per
/// locale; never free text so a Spanish parent reads "la mayoría").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MealPayload {
    /// The meal slot key (`breakfast|lunch|snack|dinner`) — matches
    /// `menu::Slot::key()` so the feed can cross-reference the day's menu +
    /// the child's derived substitution (daily-feed-scope §"composes from
    /// menu.* × child.allergies").
    pub slot: String,
    /// How much was eaten — an enum key (`all|most|some|none`), rendered per
    /// locale (`log.portion.<key>`).
    pub portion: String,
}

/// Incident payload — the REQUIRED regulated fields (daily-feed-scope §Goals:
/// "incidents carry required fields (what/where/action) and a
/// guardian-acknowledgement flag"). All three strings are validated non-empty
/// at write; the ack flag is set false at write, flipped by the guardian ack.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncidentPayload {
    /// WHAT happened (required, non-empty; never translated — recorded as-is).
    pub what: String,
    /// WHERE it happened (required, non-empty).
    #[serde(rename = "where")]
    pub where_: String,
    /// The ACTION taken (required, non-empty).
    pub action: String,
    /// Whether a guardian has acknowledged reading this incident. Set `false`
    /// at write; flipped by the ack path (best-effort v1 — the resolved open
    /// question; recorded for the center's file).
    #[serde(default)]
    pub acknowledged: bool,
}

/// Medication payload — dose + witness (daily-feed-scope §Goals: "medications
/// record dose + witness"). Both required non-empty at write.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MedicationPayload {
    /// The dose administered (required, non-empty; free text — never translated).
    pub dose: String,
    /// The witnessing staff member's `sub` or name (required, non-empty).
    pub witness: String,
}

/// The per-child bus subject a new entry publishes onto (daily-feed-scope
/// §"Intent": "one bus subject per child, filtered at emit"). The SSE feed
/// (`care.feed.watch`) subscribes to this subject via the gateway stream route.
///
/// **Filtered at emit** (the 02 decision, daily-feed-scope §"How it fits"):
/// because the subject embeds the child id, the reach check is applied when a
/// guardian SUBSCRIBES (the watch verb asserts reach on the child before
/// opening the stream) — the emit fans one payload onto the child's subject,
/// and only reach-holders can subscribe. This keeps the emit side dumb and the
/// authz decision in the one chokepoint.
pub fn feed_subject(child_id: &str) -> String {
    // A join, not a `format!` with a literal, so this is pure key construction
    // (distinct from user-facing chrome — rule 8 lint). `care.feed.<child>`.
    ["care.feed", child_id].join(".")
}

/// Typed errors the verb layer maps to the MCP `ToolError` shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogError {
    /// The id was empty / whitespace / too long.
    InvalidId(String),
    /// The `kind` value is outside the eight-type set.
    InvalidKind(String),
    /// The `at` timestamp was empty or malformed.
    InvalidTimestamp(String),
    /// A required field for this kind was empty (incident what/where/action,
    /// medication dose/witness, or a core field like child_id/room_id).
    MissingField(&'static str),
    /// A photo attach was attempted for a child whose `photos_allowed` consent
    /// is not set — blocked AT WRITE (daily-feed-scope §"Photo consent").
    PhotoConsentDenied(String),
    /// The entry already exists (append is first-write on the entry id).
    AlreadyExists(String),
    /// The referenced record (child / corrected entry) was not found.
    NotFound(String),
    /// The store denied the read or write.
    StoreDenied(String),
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogError::InvalidId(s) => write!(f, "invalid id: {s:?}"),
            LogError::InvalidKind(s) => write!(f, "invalid kind: {s:?}"),
            LogError::InvalidTimestamp(s) => write!(f, "invalid timestamp: {s:?}"),
            LogError::MissingField(s) => write!(f, "missing required field: {s}"),
            LogError::PhotoConsentDenied(c) => {
                write!(f, "photo consent denied for child: {c}")
            }
            LogError::AlreadyExists(s) => write!(f, "daily log entry already exists: {s}"),
            LogError::NotFound(s) => write!(f, "not found: {s}"),
            LogError::StoreDenied(what) => write!(f, "store denied: {what}"),
        }
    }
}

impl std::error::Error for LogError {}

impl LogKind {
    /// Validate the REQUIRED payload fields for this kind (daily-feed-scope
    /// §Goals). An incident MUST carry non-empty what/where/action; a
    /// medication MUST carry non-empty dose/witness; a meal MUST carry a
    /// slot + portion. The other types have no required payload. Called by
    /// `log::add` at the write boundary so a regulated entry can never land
    /// half-formed.
    pub fn validate(&self, log: &DailyLog) -> Result<(), LogError> {
        match self {
            LogKind::Incident => {
                let i = log
                    .incident
                    .as_ref()
                    .ok_or(LogError::MissingField("incident"))?;
                if i.what.trim().is_empty() {
                    return Err(LogError::MissingField("incident.what"));
                }
                if i.where_.trim().is_empty() {
                    return Err(LogError::MissingField("incident.where"));
                }
                if i.action.trim().is_empty() {
                    return Err(LogError::MissingField("incident.action"));
                }
                Ok(())
            }
            LogKind::Medication => {
                let m = log
                    .medication
                    .as_ref()
                    .ok_or(LogError::MissingField("medication"))?;
                if m.dose.trim().is_empty() {
                    return Err(LogError::MissingField("medication.dose"));
                }
                if m.witness.trim().is_empty() {
                    return Err(LogError::MissingField("medication.witness"));
                }
                Ok(())
            }
            LogKind::Meal => {
                let m = log.meal.as_ref().ok_or(LogError::MissingField("meal"))?;
                if m.slot.trim().is_empty() {
                    return Err(LogError::MissingField("meal.slot"));
                }
                if m.portion.trim().is_empty() {
                    return Err(LogError::MissingField("meal.portion"));
                }
                Ok(())
            }
            // Nap/diaper/activity/photo/note carry no hard-required payload.
            _ => Ok(()),
        }
    }
}

/// Validate an ISO-8601 timestamp shape (`YYYY-MM-DDTHH:MM...`). Shape-only —
/// the device supplies real wall time; we reject the obviously-malformed so a
/// garbage `at` never fragments the feed's time ordering. Mirrors the
/// attendance ledger's `validate_timestamp` (same posture, one guard per table).
pub fn validate_timestamp(s: &str) -> Result<(), LogError> {
    let b = s.as_bytes();
    let ok = b.len() >= 19
        && b[4] == b'-'
        && b[7] == b'-'
        && (b[10] == b'T' || b[10] == b' ')
        && b[13] == b':'
        && b[16] == b':'
        && b[..10]
            .iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
    if ok {
        Ok(())
    } else {
        Err(LogError::InvalidTimestamp(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_kind_parses_its_set_only() {
        assert_eq!(LogKind::parse("meal"), Some(LogKind::Meal));
        assert_eq!(LogKind::parse("INCIDENT"), Some(LogKind::Incident));
        assert_eq!(LogKind::parse("telepathy"), None);
    }

    #[test]
    fn incident_and_medication_always_push_others_dont() {
        assert_eq!(LogKind::Incident.push_policy(), PushPolicy::Always);
        assert_eq!(LogKind::Medication.push_policy(), PushPolicy::Always);
        assert_eq!(LogKind::Meal.push_policy(), PushPolicy::FeedThenPrefs);
        assert_eq!(LogKind::Photo.push_policy(), PushPolicy::FeedThenPrefs);
    }

    #[test]
    fn feed_subject_embeds_the_child_id() {
        assert_eq!(feed_subject("child:leo"), "care.feed.child:leo");
    }

    #[test]
    fn label_key_is_the_catalog_suffix() {
        assert_eq!(LogKind::Nap.label_key(), "log.type.nap");
    }

    #[test]
    fn incident_requires_all_three_regulated_fields() {
        let mut log = base_log(LogKind::Incident);
        // No payload at all → missing incident.
        assert_eq!(
            LogKind::Incident.validate(&log),
            Err(LogError::MissingField("incident"))
        );
        // Empty `where` → missing incident.where.
        log.incident = Some(IncidentPayload {
            what: "scraped knee".into(),
            where_: "  ".into(),
            action: "cleaned + plaster".into(),
            acknowledged: false,
        });
        assert_eq!(
            LogKind::Incident.validate(&log),
            Err(LogError::MissingField("incident.where"))
        );
        // All three present → ok.
        log.incident = Some(IncidentPayload {
            what: "scraped knee".into(),
            where_: "playground".into(),
            action: "cleaned + plaster".into(),
            acknowledged: false,
        });
        assert!(LogKind::Incident.validate(&log).is_ok());
    }

    #[test]
    fn medication_requires_dose_and_witness() {
        let mut log = base_log(LogKind::Medication);
        assert_eq!(
            LogKind::Medication.validate(&log),
            Err(LogError::MissingField("medication"))
        );
        log.medication = Some(MedicationPayload {
            dose: "5ml".into(),
            witness: "".into(),
        });
        assert_eq!(
            LogKind::Medication.validate(&log),
            Err(LogError::MissingField("medication.witness"))
        );
    }

    #[test]
    fn timestamp_shape_is_enforced() {
        assert!(validate_timestamp("2026-07-14T11:30:00Z").is_ok());
        assert!(validate_timestamp("lunchtime").is_err());
    }

    #[test]
    fn incident_where_uses_the_reserved_json_key() {
        let log = {
            let mut l = base_log(LogKind::Incident);
            l.incident = Some(IncidentPayload {
                what: "scraped knee".into(),
                where_: "playground".into(),
                action: "cleaned".into(),
                acknowledged: false,
            });
            l
        };
        let v = serde_json::to_value(&log).unwrap();
        // `where` is a JSON key (not the Rust reserved word `where_`).
        assert_eq!(v["incident"]["where"], "playground");
        let back: DailyLog = serde_json::from_value(v).unwrap();
        assert_eq!(back, log);
    }

    #[test]
    fn daily_log_round_trips_through_json() {
        let log = DailyLog {
            kind: LogKind::Meal,
            child_id: "child:leo".into(),
            room_id: "room:possums".into(),
            author: "user:teacher".into(),
            at: "2026-07-14T11:30:00Z".into(),
            note: None,
            media_ids: vec!["media:1".into()],
            nap: None,
            meal: Some(MealPayload {
                slot: "lunch".into(),
                portion: "most".into(),
            }),
            incident: None,
            medication: None,
            correction_of: None,
        };
        let v = serde_json::to_value(&log).unwrap();
        assert_eq!(v["kind"], "meal");
        assert_eq!(v["meal"]["portion"], "most");
        assert_eq!(v["media_ids"][0], "media:1");
        assert!(v.get("nap").is_none(), "nap omitted when None");
        let back: DailyLog = serde_json::from_value(v).unwrap();
        assert_eq!(back, log);
    }

    /// A minimal well-formed log of `kind` with no type payload (for the
    /// validate tests to fill in).
    fn base_log(kind: LogKind) -> DailyLog {
        DailyLog {
            kind,
            child_id: "child:leo".into(),
            room_id: "room:possums".into(),
            author: "user:teacher".into(),
            at: "2026-07-14T11:30:00Z".into(),
            note: None,
            media_ids: Vec::new(),
            nap: None,
            meal: None,
            incident: None,
            medication: None,
            correction_of: None,
        }
    }
}
