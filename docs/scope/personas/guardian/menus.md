# Guardian use case — see the week's menu

**Goal:** "what's Leo eating this week?" — the room's plan with *his* substitutions
inline, and nothing about any other child.

## Journey

1. Sunday: Sam opens the Menu tab → `care.menu.week` → Possums' week; Tuesday shows
   Leo's peanut-free substitution inline on the satay row.
2. Mia's tab shows *her* room's menu with her rows — same verb, other edge.
3. Tuesday's meal log entry in the feed later confirms what was actually served.

## Verbs & screens

- `care.menu.week` / `care.menu.get` — edge-scoped: own children's rooms only, own
  child's computed substitution rows only.
- Screen: guardian → Menu tab (week view per child).

## Deny / edge cases

- `care.menu.set` → 403.
- The read returns only the guardian's child's substitutions — never the room's allergy
  roster; another child's allergen flag reaching a guardian is exactly the medical-info
  leak class the cross-family matrix tests (menus scope §Risks).
- No edge to any enrolled child → empty, not error.

## Source scopes

[`../../care/menus-scope.md`](../../care/menus-scope.md) ·
[`../../care/care-authz-scope.md`](../../care/care-authz-scope.md) ·
[`../teacher/serving-meals.md`](../teacher/serving-meals.md) (the other side of the plan).
