# Admin use case — enroll a child and their family

**Goal:** the roster is real before any parent has an account: child profile (with the
safety data), guardian records, guardianship edges with per-edge flags, enrollment into a
room or the waitlist.

## Journey

1. Admin creates the child: DOB, allergies/dietary, medical notes, immunizations,
   emergency contacts, authorized-pickup list, photo-consent flag.
2. Creates guardian records (no account yet) and links guardianship edges — relationship
   plus flags: `can_pickup`, `receives_daily_feed`, `receives_billing`,
   `emergency_contact`, `custody_notes`. Blended families are just more edges (Sam→Leo,
   Sam→Mia; Ana→Leo only).
3. Enrolls the child into a room with a schedule, or onto the waitlist.
4. Later custody change: admin updates/unlinks an edge — feed, channels, and grants
   follow immediately (the reconciliation invariant).
5. September intake: CSV import of 40 families as a job; per-row results; fixes the two
   bad rows and re-runs (idempotent).

## Verbs & screens

- `care.child.create/update/get/list/archive` (archive, never delete — retention),
  `care.guardian.*`, `care.guardianship.link/unlink/update`,
  `care.enrollment.create/update/list`, `care.enrollment.import` (job).
- Screen: admin → Enrollment (child editor, family/edges editor, waitlist ordering,
  import with per-row results).

## Deny / edge cases

- Staff: `care.child.update` → 403 (the canonical cap-deny). Guardians: all of it → 403.
- Import validation is hard-fail per row — an allergy typo is a safety bug, never
  "best-effort" (enrollment scope §Risks).
- Archived child: invisible to guardians, recoverable by admin.
- Edge unlink → the ex-guardian's next read denies (cross-family matrix + reconciliation
  tests).

## Source scopes

[`../../care/enrollment-invites-scope.md`](../../care/enrollment-invites-scope.md) ·
[`../../care/care-scope.md`](../../care/care-scope.md) §Family model ·
[`../../care/care-authz-scope.md`](../../care/care-authz-scope.md) ·
[`../../care/menus-scope.md`](../../care/menus-scope.md) (allergies feed substitutions).
