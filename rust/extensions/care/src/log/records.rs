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

use super::payload::{IncidentPayload, MealPayload, MedicationPayload, NapPayload};
use serde::{Deserialize, Serialize};

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

/// The per-child entry id for a multi-child add fan-out — `<base>::<child_id>`.
/// `log::add` takes ONE caller-supplied `entry_id` base (the gesture id) and
/// derives one deterministic row id per tapped child, so the eight rows of a
/// "lunch for the room" tap are independently addressable, correctable, and
/// first-write idempotent (a re-tapped gesture with the same base conflicts,
/// never silently double-logs). A join, not a `format!` with a literal — pure
/// key construction (rule 8 lint). Same `::` separator as `authz::edge_id`.
pub fn entry_id(base: &str, child_id: &str) -> String {
    [base, child_id].join("::")
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
