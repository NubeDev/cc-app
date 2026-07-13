//! The substitution DERIVATION — the food-safety control at the heart of the
//! menus milestone. ORCHESTRATOR-OWNED (menus-scope §"The derivation module is
//! the one file that matters"; the reviewer's brief is literally "make the
//! derivation miss an allergen").
//!
//! ## What it computes
//!
//! Given ONE child's allergy set (from the child record — the single source of
//! truth) and ONE `(date, room, slot)` [`Menu`], produce the per-child
//! substitution rows for that menu: for every planned item whose allergen tags
//! intersect the child's allergies, a [`DerivedRow`] naming the offending item,
//! the matched allergen, and the substitute IF one was entered (else
//! `unresolved`). The guardian read returns ONLY the asking child's rows —
//! never the room's, never another child's allergen data (menus-scope §"How it
//! fits"; the medical-leak class the matrix tests).
//!
//! ## Why it can't miss (the conservative posture)
//!
//! Three false-negative guards (menus-scope §"Risks"):
//!   1. **Case/alias-folded key match** — `Allergen::parse` folds `dairy`→milk,
//!      `gluten`→wheat, case, and `other:` prefix, so a child's `"Dairy"` and
//!      an item's `"milk"` tag intersect. The intersection is on the STABLE
//!      key, never the raw string.
//!   2. **Untaggable item flags for ANY allergy** — an item with an EMPTY
//!      allergen tag list can't be proven safe, so it is flagged (as
//!      `Untaggable`) for every child who has ANY allergy. A staff member who
//!      forgets to tag an item does not silently poison a child.
//!   3. **Unresolved is loud** — a matched allergen with no substitute entered
//!      yields `substitute: None` (`resolved == false`); the plan view renders
//!      it red at plan time, not discovered at lunch.
//!
//! Pure function, no I/O — the verb bodies fetch the child + menus and call
//! this. That keeps the safety logic in one heavily-tested place.

use serde::Serialize;

use super::allergen::Allergen;
use super::records::Menu;

/// One derived substitution row for a single child against a single menu item.
/// `resolved == substitute.is_some()`; an unresolved row is the red flag.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DerivedRow {
    /// The offending item's name (verbatim — never translated).
    pub item: String,
    /// Why it was flagged: a specific matched allergen KEY, or the
    /// conservative `untaggable` sentinel. Rendered per-locale in the UI
    /// (`allergen.<key>`); `untaggable` and `other:` labels render as-is.
    pub reason: String,
    /// The substitute dish IF one was entered for this restriction, else
    /// `None` — an UNRESOLVED substitution (the red flag at plan time).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substitute: Option<String>,
    /// Convenience flag mirroring `substitute.is_some()` so the UI can filter
    /// unresolved rows without inspecting the option (the safety highlight).
    pub resolved: bool,
}

/// Derive the substitution rows for ONE child (their allergy keys) against ONE
/// menu. Returns an empty vec when nothing matches. The caller passes the
/// child's allergies already parsed to canonical keys via [`allergy_keys`].
///
/// The match is: for each item, intersect its allergen keys with the child's
/// allergy keys. Any intersection → a row per matched allergen. An untaggable
/// item (no allergen keys) → one conservative row IF the child has any allergy.
pub fn derive_for_child(menu: &Menu, child_allergy_keys: &[String]) -> Vec<DerivedRow> {
    let mut rows = Vec::new();
    let has_any_allergy = !child_allergy_keys.is_empty();
    // The child's restrictions the fixed vocabulary CANNOT reason about — a
    // free-text allergy that folded to `other:*` (e.g. `other:mango`). An item
    // tagged only with fixed allergens can't be PROVEN clear of these, so they
    // force a conservative per-item flag (guard 3, below). A menu item can only
    // be proven safe against a restriction the tag vocabulary can express.
    let has_unknown_restriction = child_allergy_keys.iter().any(|k| k.starts_with("other:"));

    for item in &menu.items {
        if item.allergens.is_empty() {
            // Guard 2: an untaggable item cannot be proven safe. Flag it
            // conservatively for a child with ANY allergy so a missing tag is
            // never a silent safe. No substitute is inferable → unresolved.
            if has_any_allergy {
                rows.push(DerivedRow {
                    item: item.name.clone(),
                    reason: "untaggable".to_string(),
                    substitute: None,
                    resolved: false,
                });
            }
            continue;
        }

        let mut matched_fixed = false;
        for tag in &item.allergens {
            let tag_key = tag.key();
            // Guard 1: the intersection is on the canonical key (alias/case
            // folded by `Allergen::parse` on both sides), never the raw string.
            if child_allergy_keys.iter().any(|k| k == &tag_key) {
                matched_fixed = true;
                let substitute = substitute_for(menu, tag);
                rows.push(DerivedRow {
                    item: item.name.clone(),
                    reason: tag_key,
                    resolved: substitute.is_some(),
                    substitute,
                });
            }
        }

        // Guard 3: the child has a free-text (`other:*`) restriction the fixed
        // tags can't express. This tagged item was NOT matched by a fixed key,
        // so it CANNOT be proven clear of that unknown restriction — flag it
        // conservatively (unresolved). A child whose allergy the vocabulary
        // can't name is never silently served a dish that "looked safe" only
        // because the tag set couldn't describe the danger.
        if has_unknown_restriction && !matched_fixed {
            rows.push(DerivedRow {
                item: item.name.clone(),
                reason: "untaggable".to_string(),
                substitute: None,
                resolved: false,
            });
        }
    }
    rows
}

/// Whether a menu has ANY unresolved substitution for a child — the plan-time
/// red flag (menus-scope §"unresolved-substitution flagged at plan time").
/// True when at least one derived row has no substitute.
pub fn has_unresolved(menu: &Menu, child_allergy_keys: &[String]) -> bool {
    derive_for_child(menu, child_allergy_keys)
        .iter()
        .any(|r| !r.resolved)
}

/// Canonicalize a child's raw allergy strings (as stored on the child record)
/// to the stable allergen keys the derivation compares on. This is the ONE
/// place the child side is folded, so the child's `"Dairy"` and a menu's
/// `"milk"` tag are guaranteed to meet (menus-scope §"Risks" — alias folding).
pub fn allergy_keys(raw: &[String]) -> Vec<String> {
    raw.iter().map(|a| Allergen::parse(a).key()).collect()
}

/// The substitute entered for a restriction, matched on the canonical key.
fn substitute_for(menu: &Menu, restriction: &Allergen) -> Option<String> {
    let want = restriction.key();
    menu.substitutions
        .iter()
        .find(|s| s.restriction.key() == want)
        .map(|s| s.substitute.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::allergen::Slot;
    use crate::menu::records::{MenuItem, Substitution};

    fn menu(items: Vec<MenuItem>, subs: Vec<Substitution>) -> Menu {
        Menu {
            date: "2026-07-14".into(),
            room_id: "room:possums".into(),
            slot: Slot::Lunch,
            items,
            substitutions: subs,
        }
    }

    #[test]
    fn satay_peanut_fixture_flags_leo() {
        // The scope's canonical fixture: peanut satay flags a peanut-allergic
        // child, unresolved when no substitute is entered.
        let m = menu(
            vec![MenuItem {
                name: "Peanut satay".into(),
                allergens: vec![Allergen::Peanut],
            }],
            vec![],
        );
        let rows = derive_for_child(&m, &allergy_keys(&["peanut".into()]));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].item, "Peanut satay");
        assert_eq!(rows[0].reason, "peanut");
        assert!(!rows[0].resolved, "no substitute entered → unresolved");
        assert!(has_unresolved(&m, &allergy_keys(&["peanut".into()])));
    }

    #[test]
    fn entering_a_substitute_resolves_it() {
        let m = menu(
            vec![MenuItem {
                name: "Peanut satay".into(),
                allergens: vec![Allergen::Peanut],
            }],
            vec![Substitution {
                restriction: Allergen::Peanut,
                substitute: "Sunflower satay".into(),
            }],
        );
        let rows = derive_for_child(&m, &allergy_keys(&["peanut".into()]));
        assert_eq!(rows[0].substitute.as_deref(), Some("Sunflower satay"));
        assert!(rows[0].resolved);
        assert!(!has_unresolved(&m, &allergy_keys(&["peanut".into()])));
    }

    #[test]
    fn alias_and_case_still_match_the_medical_key() {
        // The child stored "Dairy" (free text on the record); the item is
        // tagged "milk". They MUST intersect (guard 1) — a miss here is the
        // worst outcome.
        let m = menu(
            vec![MenuItem {
                name: "Cheese pizza".into(),
                allergens: vec![Allergen::Milk],
            }],
            vec![],
        );
        let rows = derive_for_child(&m, &allergy_keys(&["Dairy".into()]));
        assert_eq!(rows.len(), 1, "dairy(child) must meet milk(menu)");
        assert_eq!(rows[0].reason, "milk");
    }

    #[test]
    fn untaggable_item_flags_conservatively_for_an_allergic_child() {
        // Guard 2: an item with no allergen tags can't be proven safe.
        let m = menu(
            vec![MenuItem {
                name: "Mystery casserole".into(),
                allergens: vec![],
            }],
            vec![],
        );
        // Allergic child → flagged (conservative), unresolved.
        let allergic = derive_for_child(&m, &allergy_keys(&["egg".into()]));
        assert_eq!(allergic.len(), 1);
        assert_eq!(allergic[0].reason, "untaggable");
        assert!(!allergic[0].resolved);
        // Child with NO allergies → no flag (nothing to protect against).
        let none = derive_for_child(&m, &[]);
        assert!(none.is_empty());
    }

    #[test]
    fn plural_and_spaced_free_text_allergies_still_flag() {
        // FINDING 1 (the adversarial reviewer's CRITICAL): a child's allergy
        // typed as the NATURAL plural "peanuts" (or "Tree Nuts", "peanut
        // allergy") must still meet a `peanut`/`tree_nut` menu tag. A miss here
        // is the worst outcome.
        let m = menu(
            vec![MenuItem {
                name: "Peanut satay".into(),
                allergens: vec![Allergen::Peanut],
            }],
            vec![],
        );
        for spelling in ["peanuts", "Peanuts", "PEANUT", "peanut allergy", "allergic to peanuts"] {
            let rows = derive_for_child(&m, &allergy_keys(&[spelling.into()]));
            assert_eq!(rows.len(), 1, "spelling {spelling:?} must flag the peanut item");
            assert_eq!(rows[0].reason, "peanut");
        }
    }

    #[test]
    fn an_unknown_free_text_restriction_flags_tagged_items_conservatively() {
        // FINDING 1 guard 3: a child whose allergy the fixed vocabulary can't
        // name (`other:mango`) can't have a peanut-tagged item PROVEN safe
        // against mango — so it flags conservatively (unresolved), never a
        // silent safe.
        let m = menu(
            vec![MenuItem {
                name: "Fruit salad".into(),
                allergens: vec![Allergen::Peanut], // tagged, but not for mango
            }],
            vec![],
        );
        let rows = derive_for_child(&m, &allergy_keys(&["mango".into()]));
        assert_eq!(rows.len(), 1, "an unknown restriction flags the item conservatively");
        assert!(!rows[0].resolved);
        // A child whose restriction the vocabulary DOES express (peanut) does
        // NOT get the conservative flag on an item clear of peanut.
        let clear = menu(
            vec![MenuItem {
                name: "Rice".into(),
                allergens: vec![Allergen::Milk],
            }],
            vec![],
        );
        assert!(
            derive_for_child(&clear, &allergy_keys(&["peanut".into()])).is_empty(),
            "a known restriction doesn't over-flag a clearly-safe item"
        );
    }

    #[test]
    fn a_child_only_sees_rows_for_their_own_allergens() {
        // Leo (peanut) against a menu that also carries egg. Only the peanut
        // row derives — the egg tag is invisible to Leo (no leak of another
        // restriction's relevance).
        let m = menu(
            vec![
                MenuItem {
                    name: "Peanut satay".into(),
                    allergens: vec![Allergen::Peanut],
                },
                MenuItem {
                    name: "Egg custard".into(),
                    allergens: vec![Allergen::Egg],
                },
            ],
            vec![],
        );
        let rows = derive_for_child(&m, &allergy_keys(&["peanut".into()]));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reason, "peanut");
    }
}
