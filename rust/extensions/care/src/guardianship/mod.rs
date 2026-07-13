//! care.guardianship.* — edge CRUD (guardian ↔ child) with the five per-edge
//! flags. THE authz source of truth: a live edge = reach. This is where
//! era-2 scoped-grant derivation lives (`link` derives the grant, `unlink`
//! removes it — transactionally with the edge write, `care-authz-scope.md`).
//!
//! Verb-per-file (FILE-LAYOUT §2): `records.rs` is the orchestrator-owned
//! schema; `link` / `unlink` / `update` each own their file.

pub mod link;
pub mod unlink;
pub mod update;

mod records;

pub use records::{edge_id, EdgeFlags, Guardianship, GuardianshipError, Relationship};
