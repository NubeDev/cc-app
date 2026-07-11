# Milestone 07 — menus & substitutions (parallelizable with 06)

Week plans per room, allergen tags, **derived** substitutions — the food-safety surface.
No upstream lb dependency. Scope: [`../scope/care/menus-scope.md`](../scope/care/menus-scope.md).

## Entry gate

- [ ] Milestone 03 closed (child allergy records are the derivation input).

## Work items

- [ ] `menu` records (date × room × slot; items with allergen tags from the fixed top-9
      set + conservative `other:`), copy-last-week verb.
- [ ] Verbs: `care.menu.set` (admin/staff), `care.menu.get`, `care.menu.week`
      (guardian read = own child's room + **only that child's** computed substitution
      rows).
- [ ] **Derivation, not entry:** allergy truth lives on the child record only; the
      allergen × tag intersection computes at read/serve time; substitutes entered once
      per (menu, restriction) pair; unresolved substitution loud at plan time.
- [ ] Allergy edit on a child re-derives across every planned menu.
- [ ] Admin UI: week-grid planner with red unresolved flags
      ([`../scope/personas/admin/menu-planning.md`](../scope/personas/admin/menu-planning.md)).
      Staff UI: serving view, red flags + substitutes
      ([`../scope/personas/teacher/serving-meals.md`](../scope/personas/teacher/serving-meals.md)).
      Guardian UI: week view per child
      ([`../scope/personas/guardian/menus.md`](../scope/personas/guardian/menus.md)).

## Exit gate

- [ ] Cap-deny (guardian `menu.set` → 403) + matrix rows on `week/get` — **Ana must not
      see Mia's substitutions or that Mia exists**; a guardian read never includes
      another child's allergen data (the medical-leak class).
- [ ] Derivation tests: satay/peanut fixture flags Leo; allergy edit re-derives;
      unresolved flagged at plan time; copy-week idempotent; false-negative posture
      (untaggable item → conservative flag) asserted.
- [ ] All three persona journeys pass through the UI on a real node, in `en` **and**
      `es` — allergen names and slot names are enum keys rendered per locale (a
      Spanish-speaking guardian must read "cacahuete/maní", not "peanut"; this is the
      safety surface, so the i18n rule bites hardest here).
- [ ] Open questions resolved: top-9 fixed enum v1; dietary preference (halal/veg) as a
      parallel non-safety tag (recommended); copy-week only, no rotations v1.
- [ ] STATUS.md moved.

## Subagent notes

The derivation module is the one file that matters — single agent, heavy tests. Verbs and
three UI slices fan out cleanly. Reviewer brief: make the derivation miss an allergen.

## Sources

`../scope/care/menus-scope.md` · `../scope/care/enrollment-invites-scope.md` (allergy
source) · the three persona docs above.
