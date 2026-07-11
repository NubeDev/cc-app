//! care.center.* — admin CRUD over centers.
//!
//! A `center` is a physical location belonging to a workspace (one
//! workspace per childcare organization, per care-scope §Tenancy — so
//! `center` is a SCOPING FIELD inside the workspace, never a separate
//! workspace). Admin-only writes; staff/guardian reads via the authz
//! chokepoint (they reach the records of their room's center).
//!
//! ## Records (milestone 03)
//!
//! - `id` — `"<name-slug>"` (the durable key, e.g. `"main"`).
//! - `name` — display name.
//! - `address`, `phone`, `email` — contact info (free-text strings, never
//!   translated — recorded as the admin enters them).
//! - `default_locale` — the workspace default language for this center
//!   (`en` | `es`, per i18n-scope §"Data & defaults").
//! - `archived` — true ⇒ invisible to guardians, recoverable by admin.
//!
//! ## Verbs (milestone 03)
//!
//! - `create` (admin, gated by `mcp:care.center.create:call`)
//! - `get`    (admin/staff/guardian, gated by `mcp:care.center.get:call`,
//!   returns the center's own row — guardians reach their child's center)
//! - `list`   (admin/staff/guardian, gated by `mcp:care.center.list:call`,
//!   filters to centers the principal reaches — admin wildcard)
//!
//! Verb-per-file (FILE-LAYOUT §2): `create.rs`, `get.rs`, `list.rs`.

pub mod create;
pub mod get;
pub mod list;

mod records;

pub use records::{Center, CenterError, Locale};
