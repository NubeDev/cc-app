//! `daily_log` write-boundary validation — the typed [`LogError`], the
//! per-`LogKind` REQUIRED-field enforcement ([`LogKind::validate`]), and the
//! ISO-8601 timestamp shape guard ([`validate_timestamp`]). Split out of
//! `records.rs` (FILE-LAYOUT §one responsibility per file): the record + enum
//! shapes live in `records.rs`; the rules that decide a well-formed write live
//! here. Called by `log::add` at the write boundary so a regulated entry
//! (incident/medication) can never land half-formed.

use super::records::{DailyLog, LogKind};
use std::fmt;

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
    use super::super::payload::{IncidentPayload, MedicationPayload};
    use super::super::records::{DailyLog, LogKind};
    use super::*;

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
