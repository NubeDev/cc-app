//! care.enrollment.* — child ↔ room enrollment incl. waitlist (FIFO per
//! room). CSV import (`care.enrollment.import`) is an lb/jobs integration,
//! DEFERRED past this session (tracked in the 03 session doc).
//!
//! Verb-per-file (FILE-LAYOUT §2): `records.rs` is the orchestrator-owned
//! schema; `create` / `update` / `list` each own their file. `create` stamps
//! the monotonic per-room `waitlist_seq`; `list` orders a room's waitlist by
//! it (FIFO, stable across withdrawals).

pub mod create;
pub mod list;
pub mod update;

mod records;

pub use records::{Enrollment, EnrollmentError, EnrollmentStatus, Weekday};
