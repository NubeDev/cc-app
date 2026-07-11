//! care.room.* — admin CRUD over rooms (a physical room inside a center).
//!
//! Same shape as `center`: orchestrator-owned records, verb-per-file.
//! `records.rs` (the schema), `create.rs` (admin writes), `list.rs`
//! (holds `get` + `list` — the read paths live together because they
//! share the same era-1 read + role-check logic and stay under the
//! 400-line limit together).

pub mod create;
pub mod list;

mod records;

pub use records::{Room, RoomError};
