//! The type-specific `daily_log` payloads (daily-feed-scope §Goals) — the
//! optional per-`LogKind` fields carried on [`DailyLog`](super::records::DailyLog).
//! ORCHESTRATOR-OWNED schema, split out of `records.rs` (FILE-LAYOUT §one
//! responsibility per file): the core record + enum + errors live in
//! `records.rs`; each type's payload shape lives here.
//!
//! Each payload is populated only when its `LogKind` is set (nap carries
//! start/end; meal carries slot + portion; incident carries the required
//! what/where/action + the guardian-ack flag; medication carries dose +
//! witness). The REQUIRED-field enforcement per kind is `LogKind::validate`
//! (in `records.rs`), which reads these shapes at the write boundary.

use serde::{Deserialize, Serialize};

/// Nap payload — start/end ISO strings (either may be open while the child
/// sleeps; the pair is closed by a correction/second entry per the scope's
/// nap start/end shape). Only for [`LogKind::Nap`](super::records::LogKind::Nap).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NapPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

/// Meal payload — which slot + how much was eaten (an enum, rendered per
/// locale; never free text so a Spanish parent reads "la mayoría"). Only for
/// [`LogKind::Meal`](super::records::LogKind::Meal).
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
/// Only for [`LogKind::Incident`](super::records::LogKind::Incident).
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
/// record dose + witness"). Both required non-empty at write. Only for
/// [`LogKind::Medication`](super::records::LogKind::Medication).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MedicationPayload {
    /// The dose administered (required, non-empty; free text — never translated).
    pub dose: String,
    /// The witnessing staff member's `sub` or name (required, non-empty).
    pub witness: String,
}
