# Admin use case — operations oversight (occupancy, ratios, corrections, overrides)

**Goal:** the director's live view of the building — who is in, whether rooms are in
ratio — plus the two audited interventions: attendance corrections and pickup overrides.

## Journey

1. Dashboard: `care.attendance.now` per room → `{children, staff, ratio}`, amber at the
   configured threshold; multi-center admin sees all centers (workspace query — center is
   a filter, not a boundary).
2. A wrong tap yesterday: admin (or staff) issues `care.attendance.correct` — a
   compensating event referencing the original, never an edit; occupancy re-derives.
3. Pickup exception: collector fails the authorization check (not a `can_pickup`
   guardian, not on the authorized list) → staff see a hard deny; admin performs the
   **audited override** verb when the real world requires it (custody notes visible).
4. Reviews history: attendance list filtered by room/date — the ledger regulators read.

## Verbs & screens

- `care.attendance.now`, `care.attendance.list`, `care.attendance.correct`, the
  admin-capped pickup-override verb.
- Screen: admin → Dashboard (occupancy/ratio) + Attendance history.

## Deny / edge cases

- Guardians can't reach `now`/full `list` (edge-scoped only); staff scoped to their rooms;
  kiosk key denied everything beyond its two check verbs.
- Correction chains: `now` stays correct across in/out/correct sequences (attendance
  scope tests).
- Override without admin cap → 403; every override lands in the audit trail with actor,
  reason, and the denied collector.

## Source scopes

[`../../care/attendance-scope.md`](../../care/attendance-scope.md) ·
[`../../care/care-authz-scope.md`](../../care/care-authz-scope.md) (admin = audited role
check, no bypass) · lb `auth-caps/api-keys-scope.md` (kiosk).
