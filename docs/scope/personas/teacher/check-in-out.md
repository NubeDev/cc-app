# Teacher use case — check-in, check-out, and the pickup gate

**Goal:** an accurate, auditable ledger of who is in the room and who took each child
home — with the child-safety control (pickup authorization) unbypassable.

## Journey

1. 08:02, arrivals: room roster on the tablet; tap Leo present, pick the drop-off person
   (guardian or authorized-pickup entry) → event appended, guardians' feeds/push fire.
2. Through the day the roster shows live occupancy; staff check themselves in too (their
   presence feeds the ratio).
3. 17:20, pickup: staff tap check-out and select the collector. Grandma (on the
   authorized list) → allowed and named in the event. An unlisted person → **hard deny**
   with the reason on screen; `custody_notes` surface here; only an admin's audited
   override can proceed ([`../admin/operations-oversight.md`](../admin/operations-oversight.md)).
4. Wrong tap → `care.attendance.correct` (compensating event, never an edit).

## Verbs & screens

- `care.attendance.check_in/check_out/correct`, room-scoped `care.attendance.list`.
- Screen: staff → Room roster (tap in/out, collector picker) — one-handed, two taps for
  the happy path.

## Deny / edge cases

- Guardians cannot perform check-in/out (they *ack* via the feed); cross-room staff
  denied by reach.
- The deny must be loud in the UI and impossible to skip past — this is the product's
  child-safety control (attendance scope §Risks).
- Kiosk self-check-in (phase 1.5) appends the same events under the device principal +
  guardian PIN.

## Source scopes

[`../../care/attendance-scope.md`](../../care/attendance-scope.md) ·
[`../../care/enrollment-invites-scope.md`](../../care/enrollment-invites-scope.md)
(authorized-pickup list) · [`../../care/daily-feed-scope.md`](../../care/daily-feed-scope.md)
(guardian fan-out).
