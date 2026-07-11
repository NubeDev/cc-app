# Admin use case — plan the week's menus

**Goal:** next week's food plan per room, with every allergy conflict surfaced and
resolved **at plan time**, not discovered at lunch.

## Journey

1. Friday: admin opens the menu planner for a room, plans date × slot
   (breakfast/am-snack/lunch/pm-snack); items carry allergen tags from the fixed set.
2. The plan view immediately flags affected children (allergen tag × child allergy) —
   an unresolved substitution is loud until staff/admin enter the substitute item once
   per (menu, restriction) pair.
3. Copy-last-week for the routine weeks; edits on top.
4. An allergy edit on a child re-derives flags across every planned menu.

## Verbs & screens

- `care.menu.set` (upsert per date/room/slot), `care.menu.get`, `care.menu.week`,
  copy-week (small bounded synchronous verb).
- Screen: admin → Menu planner (week grid, red unresolved-substitution flags).

## Deny / edge cases

- Guardians: `menu.set` → 403; their reads return **only their child's** computed
  substitution rows — never the room's roster of allergies (the medical-info leak class;
  cross-family matrix rows on `week/get`).
- Free-text item that can't be confidently tagged → tagged `other:` and flagged
  conservatively (safety posture: false positives over false negatives).

## Source scopes

[`../../care/menus-scope.md`](../../care/menus-scope.md) ·
[`../../care/enrollment-invites-scope.md`](../../care/enrollment-invites-scope.md)
(allergy source of truth) · [`../teacher/serving-meals.md`](../teacher/serving-meals.md)
(the consuming flow).
