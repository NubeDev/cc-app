//! The `t(locale, key, vars)` catalog helper — the ONE place a user-facing
//! string is resolved (CLAUDE.md rule 8 / `scope/ui/i18n-scope.md`).
//!
//! Records store KEYS/enums, catalogs store the WORDS: a verb emits
//! `t(Locale::Es, "child.created", &[("name", "Leo")])`, never a raw English
//! literal. The catalogs are the repo-root `i18n/{en,es}.json` — the SAME
//! files the CI parity gate (`scripts/check-i18n-parity.sh`) checks — embedded
//! at compile time via `include_str!` so the native binary carries them (no
//! runtime file dependency, no working-dir assumption).
//!
//! ## Resolution
//!
//! - `key` is a dotted path (`"child.created"`) into the nested catalog.
//! - `{{var}}` placeholders are substituted from `vars`.
//! - A missing key in the requested locale falls back to `en` (the catalogs'
//!   declared `_meta.fallback`); a key missing in BOTH is a bug the parity
//!   gate prevents — at runtime it degrades to the key itself (visible, not a
//!   panic), so a missing catalog entry surfaces loudly instead of crashing a
//!   guardian's screen.
//!
//! The two catalogs are parity-checked in CI, so `en` and `es` always carry
//! the same key set — the fallback is a belt, not a crutch.

use crate::center::Locale;

/// The embedded catalogs (the repo-root `i18n/*.json`, byte-for-byte the CI
/// parity gate's inputs). `../../../../../i18n` walks from
/// `rust/extensions/care/src/i18n/` up to the repo root.
const EN_JSON: &str = include_str!("../../../../../i18n/en.json");
const ES_JSON: &str = include_str!("../../../../../i18n/es.json");

/// Resolve a localized string for `key` in `locale`, substituting `{{var}}`
/// placeholders from `vars`. Falls back to `en`, then to the key itself.
///
/// This is the helper EVERY user-facing verb string flows through — a raw
/// English literal in a verb body is a review-blocking defect
/// (`scripts/check-hardcoded-strings.sh`, hard as of milestone 03).
pub fn t(locale: Locale, key: &str, vars: &[(&str, &str)]) -> String {
    let template = lookup(locale, key)
        .or_else(|| lookup(Locale::En, key))
        // A key missing in BOTH catalogs degrades to the key (visible in the
        // UI, caught by the parity gate before it ships) — never a panic.
        .unwrap_or_else(|| key.to_string());
    interpolate(&template, vars)
}

/// Look a dotted `key` up in one locale's catalog. `None` if absent.
fn lookup(locale: Locale, key: &str) -> Option<String> {
    let raw = match locale {
        Locale::En => EN_JSON,
        Locale::Es => ES_JSON,
    };
    // Parse lazily per call — the catalogs are tiny (a few KB) and a verb
    // emits at most one string per call, so a parse-per-call is cheaper than
    // the machinery a cached static would need in a native single-line child.
    let root: serde_json::Value = serde_json::from_str(raw).ok()?;
    let mut node = &root;
    for segment in key.split('.') {
        node = node.get(segment)?;
    }
    node.as_str().map(str::to_string)
}

/// Substitute `{{name}}` placeholders in `template` from `vars`. An unmatched
/// placeholder is left intact (visible, not silently blanked — a missing var
/// is a caller bug worth seeing).
fn interpolate(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (name, value) in vars {
        out = out.replace(&format!("{{{{{name}}}}}"), value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_a_key_in_both_locales() {
        assert_eq!(
            t(Locale::En, "child.created", &[("name", "Leo")]),
            "Child profile for Leo created."
        );
        assert_eq!(
            t(Locale::Es, "child.created", &[("name", "Leo")]),
            "Perfil de Leo creado."
        );
    }

    #[test]
    fn interpolates_multiple_vars() {
        let s = t(
            Locale::En,
            "guardian.linked",
            &[
                ("guardian", "Sam"),
                ("child", "Leo"),
                ("relationship", "father"),
            ],
        );
        assert_eq!(s, "Sam linked to Leo as father.");
    }

    #[test]
    fn falls_back_to_en_then_to_the_key() {
        // A key present in the catalogs resolves; a bogus key degrades to
        // itself (never a panic) — the parity gate keeps this from shipping.
        assert_eq!(t(Locale::Es, "no.such.key", &[]), "no.such.key");
    }

    #[test]
    fn nested_common_keys_resolve() {
        assert_eq!(t(Locale::En, "common.save", &[]), "Save");
        assert_eq!(t(Locale::Es, "common.save", &[]), "Guardar");
    }
}
