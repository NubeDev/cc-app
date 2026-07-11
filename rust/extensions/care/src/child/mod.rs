//! care.child.* — admin CRUD over child profiles; staff/guardian read via
//! the authz chokepoint (guardians only for children they hold a live edge
//! to — CLAUDE.md rule 7). Archive, never delete (retention).
//!
//! Verb-per-file (FILE-LAYOUT §2): `records.rs` is the orchestrator-owned
//! schema; `create` / `update` / `get` / `list` / `archive` each own their
//! file. Reads (`get`/`list`) share the era-1 read + role-check pattern.

pub mod archive;
pub mod create;
pub mod get;
pub mod list;
pub mod update;

mod records;

pub use records::{validate_dob, Child, ChildError, EmergencyContact, PickupPerson};