# Teacher use case — the two-tap daily log

**Goal:** record the room's day (meals, naps, diapers, activities, photos, notes,
incidents, medications) fast enough to actually happen in a busy room — the make-or-break
interaction of the whole product (mobile-shell scope §Risks: prototype this first).

## Journey

1. 11:30: staff pick 8 children → pick "meal" → done. One action fans out to 8
   per-child `daily_log` entries; each guardian's feed updates live.
2. Photo: camera → resumable upload (lb media: begin/chunks/commit) → `media_id` attached;
   feed renders the thumb variant. Upload survives flaky room wifi.
3. Nap: start on lie-down, end on wake (type-specific payload).
4. 15:10 incident (scraped knee): required fields — what/where/action — then **always
   push** to guardians; the guardian acknowledgement lands back on the record.
5. Medication: dose + witness recorded.
6. Mis-log → `care.log.correct` (compensating, audited).

## Verbs & screens

- `care.log.add` (multi-child), `care.log.correct`, room-scoped `care.log.list`,
  media upload path.
- Screen: staff → room view, the two-tap flow (children multi-select → type → optional
  detail).

## Deny / edge cases

- Guardians can't `log.add` (403); staff can't log outside assigned rooms (reach).
- Photo-consent flag on the child blocks the media **attach at write** (not at render).
- Multi-child fan-out is atomic (all 8 or none); incident push is must-deliver via the
  outbox — a missed incident notification is not acceptable.

## Source scopes

[`../../care/daily-feed-scope.md`](../../care/daily-feed-scope.md) ·
lb `files/media-scope.md` · lb `inbox-outbox/push-target-scope.md` ·
[`../../ui/mobile-shell-scope.md`](../../ui/mobile-shell-scope.md) (the two-tap flow).
