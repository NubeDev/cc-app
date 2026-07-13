//! care.attendance.* — the check-in/out LEDGER, the pickup safety gate, the
//! kiosk device path, and the per-room ratio read-out. Milestone 06.
//!
//! Verb-per-file (FILE-LAYOUT §2): `records.rs` (the `attendance_event`
//! schema + the `PickupDeny` reason enum), `gate.rs` (the pure pickup-gate
//! decision — the child-safety control), and `occupancy.rs` (the derived
//! "who's here now" fold) are ORCHESTRATOR-OWNED; `check_in` / `check_out` /
//! `list` / `now` / `correct` each own their file.
//!
//! - `check_in` / `check_out` (staff + kiosk): APPEND a ledger event. Check-out
//!   runs the pickup gate (`gate` + `authz::pickup_roster`); a failed gate is a
//!   hard deny unless an admin `pickup_override` is set (audited).
//! - `list` (admin all / staff room-scoped / guardian own-children via authz):
//!   the ledger, time/room filtered.
//! - `now`: the DERIVED per-room occupancy `{children, staff, ratio}` — a fold
//!   over the ledger, never a mutable counter (no drift).
//! - `correct`: append a COMPENSATING event (`correction_of`), never an edit.
//!
//! The kiosk is an lb API key (machine principal, `key:` subject) granted
//! EXACTLY `check_in`/`check_out` + a minimal roster read — deny-tested for
//! everything else (attendance-scope §"Kiosk mode"; lb api-keys shipped).

pub mod check_in;
pub mod check_out;
pub mod correct;
pub mod list;
pub mod now;

pub mod gate;
pub mod occupancy;
mod records;

pub use gate::{decide as pickup_decide, Collector, PickupRoster};
pub use occupancy::{fold_now, RoomOccupancy};
pub use records::{validate_timestamp, AttendanceError, AttendanceEvent, EventKind, PickupDeny};
