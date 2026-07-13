//! care.menu.* — week food plans per room, allergen-tagged items, and
//! DERIVED per-child substitutions (the food-safety surface). Milestone 07.
//!
//! Verb-per-file (FILE-LAYOUT §2): `records.rs` (the `menu` schema) +
//! `allergen.rs` (the fixed allergen/slot enums) + `derive.rs` (the
//! substitution derivation — the one file that matters) are ORCHESTRATOR-OWNED
//! schema/logic; `set` / `get` / `week` / `copy_week` each own their file.
//!
//! - `set` (admin/staff): upsert one `(date, room, slot)` cell.
//! - `get` (reach-audited): read one cell.
//! - `week` (guardian view): a room's week + ONLY the asking child's derived
//!   substitution rows (never the room's — rule 7 / the medical-leak class).
//! - `copy_week`: copy a room's whole week to a target week (idempotent).
//!
//! Allergy TRUTH lives on the child record only; menus carry allergen TAGS;
//! the intersection is computed at read time (`derive`). One place to update
//! an allergy, every planned menu re-derives (menus-scope §"Derivation").

pub mod copy_week;
pub mod get;
pub mod set;
pub mod week;

pub mod allergen;
pub mod derive;
mod records;

pub use allergen::{Allergen, Slot};
pub use derive::{allergy_keys, derive_for_child, has_unresolved, DerivedRow};
pub use records::{validate_date, Menu, MenuError, MenuItem, Substitution};
