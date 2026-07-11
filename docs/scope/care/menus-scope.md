# Care scope — menus & lunches (plans, allergies, substitutions)

Status: scope (the ask). Promotes to `doc-site/content/public/care/` once shipped.
Owning repo: **this repo** (`rust/extensions/care/`). No upstream lb dependency.

"Show lunches and so on": the week's food plan per room, visible to guardians, with each
child's allergy-driven substitution computed — the safety half — and the meal log
(`daily-feed-scope.md`) recording what was actually served.

## Goals

- **`menu`** records: date × room × slot (`breakfast|am_snack|lunch|pm_snack`), items
  (name + allergen tags from a small fixed allergen set: peanut, dairy, egg, gluten, …),
  authored by admin/staff, plannable weeks ahead; copy-last-week.
- **Substitutions computed, not hand-entered:** a child's `allergies`/dietary field
  (enrollment scope) crossed with item allergen tags → per-child substitution requirement
  on that day's menu; staff enter the substitute item once per (menu, restriction) pair.
- **Verbs:** `care.menu.set` (admin/staff, deny-tested for guardians), `care.menu.get`
  (day, room), `care.menu.week` (the guardian view: *their child's* room + that child's
  substitutions only — via authz reach).
- **Safety surface:** the staff serving view flags every affected child in red per slot
  (the two-tap serving flow in the UI scope); an unresolved substitution (allergen match,
  no substitute entered) is visible at plan time, not discovered at lunch.

## Non-goals

- CACFP / food-program reporting (phase 3 — the allergen-tagged, per-child-served data
  model must not preclude it; the meal `daily_log` entries are the served-record).
- Inventory/ordering, recipes, nutrition math.

## Intent / approach

Menus are plain workspace state — the one deliberate design point is **derived
substitutions**: allergy truth lives on the child record only (one place to update), menus
carry allergen *tags* only, and the intersection is computed at read/serve time. Rejected:
per-child menu overrides entered by hand (drifts from the allergy record — the drift is a
safety bug).

## How it fits

- **Capabilities:** write verbs staff/admin (deny-tested); guardian reads scoped by authz
  (a guardian sees their child's room's menu + only their child's substitutions — not other
  children's medical info; an allergen flag on another child is exactly the leak class the
  matrix tests).
- **API shape:** set (upsert per date/room/slot) + get/week reads. No live feed (menus
  change slowly; the feed announces "today's lunch" via the meal log). Batch: copy-week is
  a small bounded synchronous verb.
- **Data:** state only; no motion beyond normal cache invalidation.

## Example flow

1. Friday: admin plans next week for Possums; Tuesday lunch = peanut satay → the plan view
   immediately flags Leo (peanut) as unresolved → staff enter "sunflower satay" once.
2. Sam's app, Sunday: `care.menu.week` shows Possums' week with Leo's Tuesday substitution
   inline. Mia's row shows *her* room's menu (Sam sees both children, per edges).
3. Tuesday 11:30: serving view lists Leo in red with the substitute; the meal log entry
   records what he was actually served.

## Testing plan

Cap-deny (guardian `menu.set` → 403), workspace isolation, cross-family matrix on
`week/get` (Ana must not see Mia's substitutions or existence), substitution derivation
(allergy edit on the child re-derives every planned menu), unresolved-substitution
flagged at plan time, copy-week idempotent.

## Risks & hard problems

- **The substitution derivation is a safety control** — a false negative (missed allergen)
  is the worst outcome; the allergen tag set is fixed + reviewed, free-text items always
  taggable as `other:` and flagged conservatively.
- Guardians inferring other children's allergies from staff-view artifacts — the guardian
  read returns only their child's computed rows, never the room's.

## Open questions

- Fixed allergen enum v1 (recommended: the top-9 set) — is free-text dietary preference
  (halal, vegetarian) in the same mechanism or a parallel non-safety tag?
- Menu templates/rotations (4-week cycle) v1 or just copy-week? (Recommend copy-week.)

## Related

`care-scope.md` · `enrollment-invites-scope.md` (the allergy source of truth) ·
`daily-feed-scope.md` (meal log = served record) · `care-authz-scope.md`.
