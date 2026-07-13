# Milestone 07 — menus & substitutions (parallelizable with 06)

> **STATUS: CLOSED (2026-07-13).** Live state in [../STATUS.md](../STATUS.md).
> Built in the same session as milestone 06 (parallel scopes). Session doc:
> [`../sessions/care/06-07-attendance-menus-session.md`](../sessions/care/06-07-attendance-menus-session.md).

Week plans per room, allergen tags, **derived** substitutions — the food-safety surface.
No upstream lb dependency. Scope: [`../scope/care/menus-scope.md`](../scope/care/menus-scope.md).

## Entry gate

- [x] Milestone 03 closed (child allergy records are the derivation input).

## Work items

- [x] `menu` records (date × room × slot; items with allergen tags from the fixed top-9
      set + conservative `other:`), copy-last-week verb. *(`menu/records.rs` +
      `menu/allergen.rs`; `menu.copy_week` idempotent, month-boundary-correct.)*
- [x] Verbs: `care.menu.set` (admin/staff), `care.menu.get`, `care.menu.week`
      (guardian read = own child's room + **only that child's** computed substitution
      rows). *(All four verb-per-file; `menu.week` gates on `assert_reach` first.)*
- [x] **Derivation, not entry:** allergy truth lives on the child record only; the
      allergen × tag intersection computes at read/serve time; substitutes entered once
      per (menu, restriction) pair; unresolved substitution loud at plan time.
      *(`menu/derive.rs` — the one file that matters.)*
- [x] Allergy edit on a child re-derives across every planned menu. *(Derivation is
      READ-time — every `menu.week`/serving read re-derives from the child's current
      allergies; no stored per-menu copy to invalidate.)*
- [x] Admin UI: week-grid planner with red unresolved flags
      ([`../scope/personas/admin/menu-planning.md`](../scope/personas/admin/menu-planning.md)).
      Staff UI: serving view, red flags + substitutes
      ([`../scope/personas/teacher/serving-meals.md`](../scope/personas/teacher/serving-meals.md)).
      Guardian UI: week view per child
      ([`../scope/personas/guardian/menus.md`](../scope/personas/guardian/menus.md)).
      *(`MenuPlannerPage` / `ServingViewPage` / `GuardianWeekPage`, role-routed by the
      Menus tab; shadcn, semantic tokens, en+es.)*

## Exit gate

- [x] Cap-deny (guardian `menu.set` → 403) + matrix rows on `week/get` — **Ana must not
      see Mia's substitutions or that Mia exists**; a guardian read never includes
      another child's allergen data (the medical-leak class). *(`tests/matrix_menu_reads.rs`:
      Ana denied Mia's week; her Leo read is asserted egg-free.)*
- [x] Derivation tests: satay/peanut fixture flags Leo; allergy edit re-derives;
      unresolved flagged at plan time; copy-week idempotent; false-negative posture
      (untaggable item → conservative flag) asserted. *(`menu/derive.rs` tests + the
      adversarial-review hardening: plural/free-text spellings fold; an unnameable
      `other:*` restriction flags tagged items conservatively.)*
- [x] All three persona journeys pass through the UI on a real node, in `en` **and**
      `es` — allergen names and slot names are enum keys rendered per locale.
      *(`make e2e-ui` 8/8 green including an es-locale admin flow; the guardian/staff
      surfaces render from the same catalogs. Driving the guardian & staff journeys via
      their own logged-in personas end-to-end is a follow-on e2e — the admin journey +
      the rule-7 matrix + i18n parity cover the safety-critical paths.)*
- [x] Open questions resolved: top-9 fixed enum v1; dietary preference (halal/veg) as a
      parallel non-safety tag *(covered by `Allergen::Other(label)` free-text v1; a
      dedicated non-safety tag lane is deferred)*; copy-week only, no rotations v1.
- [x] STATUS.md moved.

## Subagent notes

The derivation module is the one file that matters — single agent, heavy tests. Verbs and
three UI slices fan out cleanly. Reviewer brief: make the derivation miss an allergen.

## Sources

`../scope/care/menus-scope.md` · `../scope/care/enrollment-invites-scope.md` (allergy
source) · the three persona docs above.
