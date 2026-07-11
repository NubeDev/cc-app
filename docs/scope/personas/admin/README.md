# Admin persona — the owner/director's product

The owner/admin runs the business: one or many centers in **one workspace** (center is a
data field, not a grant — master scope §Tenancy). Full `care.*` verbs, member/invite
management, and `reach:` to the admin surfaces. Desktop-friendly UI, still responsive
([`../../ui/mobile-shell-scope.md`](../../ui/mobile-shell-scope.md) §Admin).

**Admin reach is never a bypass**: it passes through the same `authz/` chokepoint as every
other principal, as an audited role check ([`../../care/care-authz-scope.md`](../../care/care-authz-scope.md)).

## Use cases (rough lifecycle order)

1. [setup-center.md](setup-center.md) — centers, rooms, staff room assignments.
2. [enroll-family.md](enroll-family.md) — children, guardians, guardianship edges,
   enrollment/waitlist, CSV import.
3. [invites-onboarding.md](invites-onboarding.md) — invite guardians and staff by email;
   handle mismatches and re-sends.
4. [operations-oversight.md](operations-oversight.md) — occupancy/ratio dashboard,
   attendance corrections, audited pickup overrides.
5. [menu-planning.md](menu-planning.md) — plan the week, resolve allergy substitutions.
6. [announcements.md](announcements.md) — center-wide read-only broadcast channel.

Deferred (not use cases yet): billing/invoicing (phase 2,
[`../../billing/billing-scope.md`](../../billing/billing-scope.md) — **build last**),
regulatory reports/exports, staff scheduling, admissions forms/documents.

## Open questions (admin-specific)

- Does the multi-center owner get a center-switcher or an all-centers rollup as the
  dashboard default?
- Audit trail surface: the scopes record who-did-what (corrections, overrides, admin
  reads) — where does the admin *view* it v1? (Recommend: per-record history first, a
  global audit page later.)
- Second-admin / owner-vs-director split: one admin cap set v1 (recommended), or a
  reduced "director" set per center?
