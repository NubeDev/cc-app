# Admin use case — set up centers, rooms, and staff assignments

**Goal:** from an empty workspace to a structure the rest of the product hangs off:
centers → rooms → staff assigned to rooms.

## Journey

1. Admin signs in (first member of the workspace) and lands on the admin surface.
2. Creates a center (name, address, contact, licensed capacity); repeats per center —
   all inside the one workspace.
3. Creates rooms per center (name, age band, capacity, target staff ratio).
4. Invites staff ([invites-onboarding.md](invites-onboarding.md)) and assigns each to
   room(s); assignments drive staff reach and channel membership.
5. Provisioning side-effects happen automatically: each room/center gets its channel
   (messaging scope), the ratio dashboard has its denominators.

## Verbs & screens

- `care.center.create/update/list`, `care.room.create/update/list` — admin CRUD.
- Staff room assignment (enrollment/invites scope records) — the single source
  the authz chokepoint (`reachable_rooms`) and the channel reconciler both read.
- Screen: admin → Centers/Rooms (mobile-shell scope §Admin).

## Deny / edge cases

- Staff and guardians: every `center/room` write verb → 403 (cap-deny tests).
- Room archive with enrolled children → blocked until children are moved (no orphan
  enrollments).
- Staff moved between rooms → reach and channel membership follow the assignment
  (messaging scope reconciler; asserted, not eventual).

## Source scopes

[`../../care/care-scope.md`](../../care/care-scope.md) §Tenancy ·
[`../../care/enrollment-invites-scope.md`](../../care/enrollment-invites-scope.md) ·
[`../../care/messaging-scope.md`](../../care/messaging-scope.md) (provisioning) ·
[`../../care/attendance-scope.md`](../../care/attendance-scope.md) (ratios).
