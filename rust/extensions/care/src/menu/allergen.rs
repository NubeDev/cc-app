//! The fixed allergen enum + slot enum — ORCHESTRATOR-OWNED (menus-scope
//! §"Fixed allergen enum v1"). The allergen tag set is the SAFETY control's
//! vocabulary: a fixed, reviewed set (the top-9) so a menu item's tag and a
//! child's allergy speak the SAME language and the derivation
//! ([`super::derive`]) can intersect them by key, not by fuzzy string match.
//!
//! Two escape hatches keep the false-negative posture conservative
//! (menus-scope §"Risks"):
//!   - `Other(String)` — a free-text allergen the fixed set can't name; it is
//!     matched by exact (case-folded) label AND always surfaces as taggable.
//!   - An UNTAGGABLE menu item (empty tag list) is never treated as "safe":
//!     the derivation flags it conservatively for every child who has ANY
//!     allergy (see `derive`).
//!
//! The words render per-locale from the i18n catalog under `allergen.<key>`
//! (CLAUDE.md rule 8): a Spanish-speaking guardian must read "cacahuete/maní",
//! never "peanut". Records store the KEY (`peanut`), catalogs store the WORD.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The fixed top-9 allergen set (US "big-9" — the reviewed v1 vocabulary),
/// plus an `Other` free-text escape. Serialized lowercase; `Other` serializes
/// as `other:<label>` so a round-trip is lossless and the catalog key for the
/// fixed arms stays a bare word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Allergen {
    Peanut,
    TreeNut,
    Milk,
    Egg,
    Wheat,
    Soy,
    Fish,
    Shellfish,
    Sesame,
    /// A free-text allergen the fixed set can't name. Matched by exact
    /// case-folded label. Always taggable → never a silent safe.
    Other(String),
}

impl Allergen {
    /// The fixed-set members (not `Other`) — for the UI's tag picker.
    pub const FIXED: &'static [Allergen] = &[
        Allergen::Peanut,
        Allergen::TreeNut,
        Allergen::Milk,
        Allergen::Egg,
        Allergen::Wheat,
        Allergen::Soy,
        Allergen::Fish,
        Allergen::Shellfish,
        Allergen::Sesame,
    ];

    /// Parse a wire tag. A fixed word maps to its arm; anything else becomes
    /// `Other(<trimmed, lowercased label>)` — NEVER rejected (a food-safety
    /// tag is always accepted so garbage flags conservatively, never drops).
    /// An `other:foo` prefix is honored so a re-serialized tag round-trips.
    pub fn parse(s: &str) -> Allergen {
        let raw = s.trim();
        let bare = raw.strip_prefix("other:").unwrap_or(raw);
        match bare.to_ascii_lowercase().as_str() {
            "peanut" => Allergen::Peanut,
            "tree_nut" | "treenut" | "tree-nut" => Allergen::TreeNut,
            "milk" | "dairy" => Allergen::Milk,
            "egg" => Allergen::Egg,
            "wheat" | "gluten" => Allergen::Wheat,
            "soy" => Allergen::Soy,
            "fish" => Allergen::Fish,
            "shellfish" => Allergen::Shellfish,
            "sesame" => Allergen::Sesame,
            other => Allergen::Other(other.to_string()),
        }
    }

    /// The stable wire/storage key (`peanut`, `other:mango`). This is what a
    /// record stores and the derivation compares on — never the display word.
    pub fn key(&self) -> String {
        match self {
            Allergen::Peanut => "peanut".into(),
            Allergen::TreeNut => "tree_nut".into(),
            Allergen::Milk => "milk".into(),
            Allergen::Egg => "egg".into(),
            Allergen::Wheat => "wheat".into(),
            Allergen::Soy => "soy".into(),
            Allergen::Fish => "fish".into(),
            Allergen::Shellfish => "shellfish".into(),
            Allergen::Sesame => "sesame".into(),
            Allergen::Other(label) => ["other:", label].concat(),
        }
    }

    /// The i18n catalog key for the display word (`allergen.peanut`). `Other`
    /// has no fixed catalog word — the free-text label IS the display (never
    /// translated), so the derivation surfaces its label verbatim.
    pub fn catalog_key(&self) -> Option<&'static str> {
        Some(match self {
            Allergen::Peanut => "allergen.peanut",
            Allergen::TreeNut => "allergen.tree_nut",
            Allergen::Milk => "allergen.milk",
            Allergen::Egg => "allergen.egg",
            Allergen::Wheat => "allergen.wheat",
            Allergen::Soy => "allergen.soy",
            Allergen::Fish => "allergen.fish",
            Allergen::Shellfish => "allergen.shellfish",
            Allergen::Sesame => "allergen.sesame",
            Allergen::Other(_) => return None,
        })
    }
}

impl Serialize for Allergen {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.key())
    }
}

impl<'de> Deserialize<'de> for Allergen {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(Allergen::parse(&s))
    }
}

impl fmt::Display for Allergen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key())
    }
}

/// The meal slot a menu row is for. Fixed enum, lowercase-serialized; the
/// display word renders per-locale (`slot.<key>`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    Breakfast,
    AmSnack,
    Lunch,
    PmSnack,
}

impl Slot {
    pub const ALL: &'static [Slot] = &[Slot::Breakfast, Slot::AmSnack, Slot::Lunch, Slot::PmSnack];

    /// Parse a wire slot key. Rejects anything outside the fixed set.
    pub fn parse(s: &str) -> Option<Slot> {
        match s.trim().to_ascii_lowercase().as_str() {
            "breakfast" => Some(Slot::Breakfast),
            "am_snack" | "amsnack" => Some(Slot::AmSnack),
            "lunch" => Some(Slot::Lunch),
            "pm_snack" | "pmsnack" => Some(Slot::PmSnack),
            _ => None,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            Slot::Breakfast => "breakfast",
            Slot::AmSnack => "am_snack",
            Slot::Lunch => "lunch",
            Slot::PmSnack => "pm_snack",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dairy_and_gluten_alias_onto_the_fixed_set() {
        assert_eq!(Allergen::parse("dairy"), Allergen::Milk);
        assert_eq!(Allergen::parse("gluten"), Allergen::Wheat);
        assert_eq!(Allergen::parse("PEANUT"), Allergen::Peanut);
    }

    #[test]
    fn unknown_allergen_becomes_other_never_dropped() {
        assert_eq!(Allergen::parse("mango"), Allergen::Other("mango".into()));
        // round-trips through the `other:` prefix
        assert_eq!(
            Allergen::parse(&Allergen::Other("mango".into()).key()),
            Allergen::Other("mango".into())
        );
    }

    #[test]
    fn slot_parses_the_fixed_set_only() {
        assert_eq!(Slot::parse("lunch"), Some(Slot::Lunch));
        assert_eq!(Slot::parse("am_snack"), Some(Slot::AmSnack));
        assert_eq!(Slot::parse("brunch"), None);
    }
}
