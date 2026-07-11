# Teacher use case — serve a meal safely

**Goal:** at each slot, the teacher sees exactly which children need a substitution and
what to serve them — the allergy safety control at the moment it matters.

## Journey

1. 11:25: staff open the serving view for lunch → today's menu with every affected child
   flagged in red and their substitute item shown (planned at
   [`../admin/menu-planning.md`](../admin/menu-planning.md) — an unresolved substitution
   never survives to serving time).
2. Leo (peanut allergy, satay day) shows "sunflower satay" — staff serve it.
3. Staff log the meal ([daily-logging.md](daily-logging.md)); the `daily_log` entry is
   the *served* record (menus are the plan; the log is what happened — CACFP-ready
   later).

## Verbs & screens

- `care.menu.get` (day, room) — room-scoped staff read; `care.log.add` for the served
  record.
- Screen: staff → Serving view (per-slot list, red flags + substitutes).

## Deny / edge cases

- The derivation is computed from the child's allergy record × item allergen tags —
  never hand-copied (drift is a safety bug, menus scope §Intent).
- A menu edited mid-morning re-derives flags before serving.
- Staff see allergy flags for their room only; a guardian never sees other children's
  flags (that leak class is tested in the menus matrix rows, not here).

## Source scopes

[`../../care/menus-scope.md`](../../care/menus-scope.md) ·
[`../../care/daily-feed-scope.md`](../../care/daily-feed-scope.md) (meal log) ·
[`../../care/enrollment-invites-scope.md`](../../care/enrollment-invites-scope.md)
(allergy truth).
