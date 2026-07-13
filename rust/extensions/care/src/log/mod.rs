//! care.log.* — staff write daily-feed entries; guardians read (via authz).
//! Milestone 08 (docs/build/08-daily-feed.md).
//!
//! Verb-per-file (FILE-LAYOUT §2): `records.rs` (the `daily_log` schema + the
//! 8 log types + the type-specific payloads + the bus subject + the push
//! policy) is ORCHESTRATOR-OWNED; `add` / `list` / `correct` / `day` each own
//! their file. The live-feed SSE + push motion halves live in `feed/` and
//! `push/` respectively (they consume the records + subject defined here).
//!
//! - `add` (staff; multi-child): fan out to per-child rows atomically, attach
//!   photos (consent-checked at write), emit the per-child bus event, dispatch
//!   push per the type policy.
//! - `list` (admin all / staff room-scoped / guardian own-children via authz):
//!   the feed, cursor-paged.
//! - `correct`: append a COMPENSATING entry (`correction_of`), never an edit.
//! - `day`: the "Leo's day" rollup the UI AND the AI consume.
//!
//! Every read verb routes through the authz chokepoint (CLAUDE.md rule 7). The
//! staff write verbs are cap-gated at the host wall (guardians hold no
//! `log.add` cap — deny-tested).

pub mod add;
pub mod correct;
pub mod day;
pub mod list;

mod payload;
mod records;
mod validate;

pub use payload::{IncidentPayload, MealPayload, MedicationPayload, NapPayload};
pub use records::{entry_id, feed_subject, DailyLog, LogKind, PushPolicy};
pub use validate::{validate_timestamp, LogError};
