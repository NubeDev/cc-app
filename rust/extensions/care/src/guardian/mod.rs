//! care.guardian.* — admin CRUD over guardian records (records-before-
//! accounts: the record exists before the person has an account; an invite
//! binds a `sub` later — milestone 05).
//!
//! Verb-per-file (FILE-LAYOUT §2): `records.rs` is the orchestrator-owned
//! schema; `create` writes, `list` holds `get` + `list` (shared read path).

pub mod create;
pub mod list;

mod records;

pub use records::{validate_email, Guardian, GuardianError};
