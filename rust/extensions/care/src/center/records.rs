//! The durable `center` record + its typed error.
//!
//! Schemas are ORCHESTRATOR-OWNED (milestone 03 §Subagent notes):
//! subagents never decide record shapes. Sub-verb files import from here.
//!
//! Validation rules live next to the shape so a new verb can't drift from
//! the contract (e.g. forgetting to check `archived` before listing).

use serde::{Deserialize, Serialize};
use std::fmt;

/// The `center` record (workspace-scoped). All fields are admin-managed;
/// staff/guardian sees the row via the authz chokepoint but never edits
/// it (the verbs `create` + `update` are admin-only).
///
/// Field guide (the docs-bound contract — see i18n-scope.md §"Data &
/// defaults" for the `default_locale` semantic):
/// - `default_locale` is the **workspace default** for invites + chrome,
///   overridable per-user. Stored here because a center IS the unit of
///   physical operation — a center in El Paso defaults `es`, a center in
///   Toronto defaults `en`.
/// - `archived` is the soft-delete flag (CLAUDE.md / scope §"archive,
///   never delete"). The verb layer filters archived rows from guardian
///   reads but keeps them in the store for retention.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Center {
    /// Display name (admin-entered; never translated — recorded as-is).
    pub name: String,
    /// Free-text street address (never translated).
    pub address: Option<String>,
    /// Free-text phone (never translated).
    pub phone: Option<String>,
    /// Free-text email (never translated).
    pub email: Option<String>,
    /// Workspace default locale (`"en"` | `"es"`); per-user overrides win.
    pub default_locale: Locale,
    /// Soft-delete flag. `true` ⇒ invisible to guardians, recoverable by admin.
    pub archived: bool,
}

/// The two launch locales (per i18n-scope §"Two locales at launch").
/// A new locale is a catalog drop, not a code change — this enum grows
/// only when the catalog grows.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    En,
    Es,
}

impl Locale {
    /// Parse from the wire string. Returns `Err(CenterError::InvalidLocale)`
    /// for any value outside the launch set — a typo at the boundary
    /// fails fast (a guard against a stray `"EN"` / `"english"` slipping
    /// into the catalog lookup path).
    pub fn parse(s: &str) -> Result<Self, CenterError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Ok(Locale::En),
            "es" | "spanish" | "español" | "espanol" => Ok(Locale::Es),
            other => Err(CenterError::InvalidLocale(other.to_string())),
        }
    }
    /// Wire form (`"en"` / `"es"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Es => "es",
        }
    }
}

/// Typed errors the verb layer maps to the MCP `ToolError` shape (the host
/// wall decides on the wire form — `Err(InvalidLocale)` becomes a
/// `ToolError::InvalidArgs` upstream, `Err(NotFound)` a `ToolError::Denied`
/// after the authz check, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CenterError {
    /// The id was empty / whitespace / too long (durability guard — ids
    /// are URLs in the served UI, so they MUST be safe).
    InvalidId(String),
    /// The `default_locale` field is outside the launch set.
    InvalidLocale(String),
    /// The record already exists (`care.center.create` is first-write, not
    /// upsert — mirroring `lb_store::create`'s first-settle semantic).
    AlreadyExists(String),
    /// The store denied the read or write (the verb layer never bubbles
    /// store errors — it maps them to `Denied` after the authz check).
    StoreDenied(String),
}

impl fmt::Display for CenterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CenterError::InvalidId(s) => write!(f, "invalid id: {s:?}"),
            CenterError::InvalidLocale(s) => write!(f, "invalid locale: {s:?}"),
            CenterError::AlreadyExists(s) => write!(f, "center already exists: {s}"),
            CenterError::StoreDenied(what) => write!(f, "store denied: {what}"),
        }
    }
}

impl std::error::Error for CenterError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_parse_accepts_launch_set() {
        assert_eq!(Locale::parse("en").unwrap(), Locale::En);
        assert_eq!(Locale::parse("es").unwrap(), Locale::Es);
        // Tolerate case + common synonyms (i18n-scope §"es variant").
        assert_eq!(Locale::parse("EN").unwrap(), Locale::En);
        assert_eq!(Locale::parse("español").unwrap(), Locale::Es);
    }

    #[test]
    fn locale_parse_rejects_unknown_locales() {
        assert!(matches!(
            Locale::parse("fr"),
            Err(CenterError::InvalidLocale(s)) if s == "fr"
        ));
        assert!(Locale::parse("").is_err());
    }
}
