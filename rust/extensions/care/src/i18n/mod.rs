//! i18n — the extension's user-facing string catalog (CLAUDE.md rule 8).
//!
//! `catalog.rs` owns the ONE resolver `t(locale, key, vars)`; the embedded
//! `en`/`es` catalogs are the repo-root `i18n/*.json` (parity-checked in CI).
//! Every user-facing verb string flows through `t` — records store keys, the
//! catalog stores the words.

mod catalog;

pub use catalog::t;
