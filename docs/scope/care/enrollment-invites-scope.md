# Care scope — enrollment & invites (children, guardians, edges, onboarding)

Status: scope (the ask). Promotes to `doc-site/content/public/care/` once shipped.
Owning repo: **this repo** (`rust/extensions/care/`), consuming **lb**'s
`auth-caps/invites-scope.md` (upstream — must ship first) and `global-identity` (shipped).

The admin's setup surface: create children and guardians **before anyone has an account**,
link guardianship edges, enroll into rooms (or the waitlist), then invite guardians/staff by
email — with the guardian landing signed-in on their child's feed in one flow.

## Goals

- **Records + verbs:** `care.child.create/update/get/list/archive` (never hard-delete —
  retention), `care.guardian.*`, `care.guardianship.link/unlink/update` (relationship +
  per-edge flags: `can_pickup`, `receives_daily_feed`, `receives_billing`,
  `emergency_contact`, `custody_notes`), `care.enrollment.create/update/list` (child↔room,
  schedule, `waitlist|enrolled|withdrawn`), waitlist ordering.
- **Child profile carries the safety data:** DOB, allergies/dietary (the input to
  `menus-scope.md` substitutions), medical notes, immunization records, emergency contacts,
  authorized-pickup list (persons, not necessarily guardians).
- **Invite flow:** `care.invite_guardian(guardian_id, email)` → mints an lb `invite.create`
  with role `guardian-member`, team `guardians`, and `payload = guardian_id`; on
  `invite.accepted` the extension binds `sub → guardian` and (era 2) derives scoped grants
  from the existing edges. Staff invites identical with the staff role + room assignment.
- **Bulk import:** `care.enrollment.import` (CSV children+guardians+edges) → an lb **job**,
  per-item results, idempotent on natural keys (re-upload doesn't duplicate).

## Non-goals

- Admissions forms / e-signature / document collection (later topic, on lb document-store).
- Billing linkage (`receives_billing` is stored now, consumed by `../billing/billing-scope.md`).
- Immunization *reminders* (record now; reminder rules are a phase-2 slice).

## Intent / approach

Records-before-accounts is the deliberate shape: the center's roster is real on day one and
invites merely *bind* people to it (the `payload` correlation). Rejected: account-first
(sign up, then admin links you) — it makes the admin's setup depend on parents' promptness
and invites typo-matching the wrong person to a child.

## How it fits

- **Capabilities:** all verbs admin-gated except reads (staff room-scoped, guardian
  edge-scoped via `care-authz-scope.md`). Deny-tests per verb.
- **API shape:** CRUD + list (filter by room/status); import is a **job** (partial-failure
  per item); no live feed (roster refetch).
- **Data:** all records workspace-scoped; edges are the authz source of truth.
- **Rule 10:** the extension consumes `invite.*` as normal granted MCP tools; the core
  never sees "guardian" — `payload` is opaque.

## Example flow

1. Admin creates `child:leo` (allergies: peanuts) + guardians Sam and Ana; links two edges;
   enrolls Leo in Possums.
2. `care.invite_guardian(sam, "sam@…")` → lb invite email → Sam accepts on his phone →
   `invite.accepted` → extension binds `sam.sub`, derives grants → Sam's first screen is
   Leo's (and Mia's) feed.
3. September intake: `care.enrollment.import` with 40 rows → job id → admin watches
   progress; 2 rows fail validation (bad DOB) → per-item errors, 38 land.

## Testing plan

Mandatory cap-deny (staff → `care.child.update` 403; guardian → any admin verb 403) and
workspace isolation. Plus: archive-not-delete (archived child invisible to guardians,
recoverable by admin), edge flags respected downstream (a `receives_daily_feed: false` edge
gets no feed — asserted in daily-feed tests), invite bind idempotency (double `accepted`
event safe), import job partial failure + idempotent re-run, waitlist ordering stable.
Cross-family matrix rows for every read verb (per `care-authz-scope.md`).

## Risks & hard problems

- **Wrong-person binding** — the invite email must match the guardian record's email at
  accept; a mismatch parks the invite for admin review instead of binding.
- **Import is where garbage enters** (allergy typos are safety bugs) — validate hard,
  reject loudly, never "best-effort" a medical field.

## Open questions

- Are authorized-pickup persons plain child-record entries v1 (recommended) or first-class
  contact records shared across siblings?
- Waitlist: FIFO per room v1 (recommended) or priority tiers?

## Related

`care-scope.md` · `care-authz-scope.md` · lb `auth-caps/invites-scope.md` ·
lb `jobs/jobs-scope.md` · `attendance-scope.md` (pickup list consumer) ·
`../billing/billing-scope.md`.
