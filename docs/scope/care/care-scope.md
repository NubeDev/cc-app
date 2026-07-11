# Care scope — the childcare-management product on lb

Status: scope (the ask). Promotes to `doc-site/content/public/care/` once shipped.

We want a **brightwheel/lillio-class childcare management platform** — attendance, daily
reports, parent messaging, menus, enrollment, and (later) billing — for owners/admins of one
or more childcare centers, their staff, and the guardians of enrolled children. It is built
**on lb**: the core provides identity, the capability wall, the SurrealDB store, channels,
and the extension runtime; **all childcare domain logic and 100% of the product UI are
extensions** built on the published SDKs. Mobile-first — a parent lives in this on a phone.
Where the product needs something lb doesn't have, we **fix lb generically** (see §lb gaps)
rather than working around it here.

## Market reference (what "this class of app" means)

From brightwheel (mybrightwheel.com), lillio (lillio.com, ex-HiMama), and the 2026 field
(kidsday et al.), the table-stakes feature set is:

- **Attendance**: check-in/out (staff-assisted + kiosk/QR), who dropped off / picked up,
  room rosters, staff-ratio visibility.
- **Daily activity feed**: meals, naps, diapers/bathroom, activities, photos/videos, notes,
  incident reports — pushed to guardians in near-real-time.
- **Messaging**: guardian↔staff per child, room broadcasts, center-wide announcements.
- **Menus/food**: lunch & snack plans per room per day, per-child dietary
  substitutions/allergy awareness, (later) CACFP food-program reporting.
- **Enrollment/admissions**: profiles, waitlist, documents/forms, immunization records.
- **Billing**: invoicing, autopay, subsidy tracking (a differentiator, but phase-2+).
- **Learning** (phase-3): lesson plans, milestones/assessments, portfolios.
- **Multi-center**: one owner sees all their centers in one place.

The 2026 trend the roundups call out is *simplicity* — the winners are the apps a
distracted parent and a busy room-leader can use one-handed. That is our UI bar.

## Goals

- One product that serves the three personas: **owner/admin** (possibly multi-center),
  **staff** (room-scoped), **guardian** (child-scoped, mobile-first).
- Phase 1 usable by a real center: enrollment, attendance, daily feed with photos, menus,
  messaging, invites.
- Everything reachable as MCP tools (`care.*`) so AI agents can drive the same surface
  (e.g. "summarize Leo's week for his mum").
- **English + Spanish from day one — a MUST, not a stretch goal.** Every user-facing
  surface (UI, invite emails, push notifications, domain-generated text) ships 100% in
  `en` and `es`, via lb's multi-lang seam. Binding contract:
  [`../ui/i18n-scope.md`](../ui/i18n-scope.md); enforced per build-milestone exit gate.
- Every gap found in lb becomes an **upstream lb improvement**, never a product-side hack.

## Non-goals (deferred, not rejected)

- Billing/payments, subsidy management (phase 2 — external provider behind one trait).
- Lesson plans / assessments / curriculum (phase 3).
- Staff payroll/scheduling optimization (phase 3).
- A native app-store app. Phase 1 is the mobile-first web shell (installable PWA); an RN
  shell (lb's `app/shell` line) is a later decision.
- Government reporting (CACFP etc.) — data model must not preclude it; reports deferred.

## Tenancy — workspace = the childcare organization

**One lb workspace per childcare organization** (the owner's business). Centers and rooms
are **records inside** the workspace, not workspaces themselves.

- An owner/admin of 3 centers has **one** workspace; a `center` field scopes queries and
  staff assignments. Cross-center dashboards are just workspace queries.
- Staff and guardians are **workspace members** with narrow capability sets (see below).
- A guardian with children at two *unrelated* organizations is a member of **two
  workspaces** — lb's normal multi-workspace login covers this.

**Rejected: workspace-per-center.** It gives hard isolation between centers of one owner —
which nobody asked for — at the cost of the owner juggling N workspaces, cross-center
queries becoming federation problems, and a guardian with siblings in two rooms of two
centers needing two memberships to one business. The hard wall belongs between
*organizations*; inside one, `center`/`room` are just scoping fields.

## The family model — guardianship is an edge, not a household

The stated requirement: *a parent can have 2 children from different moms/dads*. So there is
no "family" primary key. The model is:

- **`child`** — profile, DOB, room/enrollment, allergies/dietary, medical notes,
  immunizations, emergency contacts, authorized-pickup list.
- **`guardian`** — a person; links 1:1 to a workspace member (`sub`) once they accept an
  invite. Exists as a record *before* they have an account (admin can add a child +
  guardians, invite later).
- **`guardianship`** — the many-to-many **edge** `guardian ↔ child` carrying the
  relationship (`mother`/`father`/`grandparent`/`nanny`/…) and per-edge flags:
  `can_pickup`, `receives_daily_feed`, `receives_billing`, `emergency_contact`,
  `custody_notes`. Dad sees both his kids (two edges); each mum sees only her own child
  (one edge each). Nothing is inferred from a shared surname or household.
- **`household`** (optional, phase 2) — a billing-only grouping; never used for data access.

**Everything a guardian may read or receive is derived from live guardianship edges** — the
single most important invariant in the product (CLAUDE.md rule 7).

## Personas → capability sets

Per-persona use-case docs (the journeys over these cap sets) live in
[`../personas/`](../personas/README.md) — `admin/`, `teacher/` (=staff), `guardian/`.

lb capability-first, folded at login (like lb's `reach:` nav gating):

- **owner/admin** — full `care.*` verbs + member/invite management + `reach:` to the admin
  surfaces. Multi-center admins are just admins (center is data, not a grant).
- **staff** — room-scoped operational verbs: attendance, daily-log write, menu read,
  child read *for their assigned room(s)*, channel posting. No enrollment/billing/member
  admin. Deny path: `care.child.update` → 403.
- **guardian** — the narrow set: read own children (via edges), read daily feed/menus for
  own children, check-in/out *ack* (not perform), message in their child's channels,
  update own contact info. Deny path: any list/read of a child without an edge → 403/empty.
- **kiosk** (phase 1.5) — a device principal with exactly `care.attendance.*`; see lb gaps.

The *within-workspace, per-record* part of guardian/staff scoping is today **not
expressible in lb's cap grammar** (caps gate tools, not rows) — see **lb gap #1**. Until
that lands, the care extension enforces it in **one** `authz/` module every read/write verb
passes through.

## Intent / approach

- **Host (`rust/node/`)**: a boot shim exactly like rubix-ai — fill `BootConfig` from
  `CC_*` env at the binary boundary, `boot_full`, state under `.cc-app/`. No product logic.
- **One backend extension `care` (`rust/extensions/care/`)** owning the whole domain:
  tools namespaced `care.*`, all guardian scoping through its single `authz/` module.
  *Rejected: one extension per feature area* (attendance, menus, …) — it would duplicate
  the authz module across extensions (each re-deriving guardian→child edges = N chances to
  leak) and multiply publish/cap-grant overhead. Split later along the natural seam:
  `care-billing` as its own extension in phase 2 (external provider, own secrets).
- **UI = 100% extension** (`care` ships pages + widgets via a single `defineRemote`),
  Tailwind + shadcn tokens via `extTailwindPreset()`, **mobile-first** (bottom tab bar,
  one-handed reach, feed-first guardian home). We do **not** vendor lb's shell.
- **Thin shell (`ui/`)**: login + workspace pick + full-screen mount of the extension's
  page + SSE wiring + PWA manifest. No sidebar, no admin console, no dock. *Rejected:
  vendoring the lb shell like rubix-ai* — it is desktop-admin-shaped; stripping it is more
  work than a thin shell, and it drags surfaces (channels UI, dashboards) we'd have to
  hide. Long-term this thin shell should come **from lb** as a package (lb gap #2).
- **Messaging = lb channels, not a new system.** The care extension *provisions* channels
  (per room, per child, announcements) and manages membership from guardianship/staff
  records; message transport, history, and SSE are core lb. The child channel membership
  reconciles when edges change (mum removed from an edge → removed from the channel).

## Data (SurrealDB — state) — all records workspace-scoped

`center`, `room`, `child`, `guardian`, `guardianship` (edge), `enrollment`
(child↔room, schedule, status: waitlist|enrolled|withdrawn), `attendance_event`
(check_in|check_out, by-whom, guardian-or-staff, timestamp), `daily_log` (typed:
meal|nap|diaper|activity|photo|note|incident|medication; child id; author; media refs),
`menu` (date × room × meal slot; items; per-child substitutions derived from allergies),
`invite` (email, role, guardian id, token, status), phase 2: `invoice`, `payment`,
`household`.

Motion (Zenoh): channel messages (core), `care.feed.<child>` bus subjects backing the SSE
live feed for daily-log/attendance events. State lives in SurrealDB; the bus only moves it.

Media (photos): stored via lb's store/bucket path, **not** inline tool payloads — see lb
gap #4 for the upload/thumbnail seam.

## MCP surface (`care.*`) — API shapes per SCOPE-WRITTING §6.1

CRUD + get/list where a caller mutates; live feed where a caller watches; jobs for long
batches. Illustrative, not exhaustive — the build session finalizes per-verb files:

- `care.center.*`, `care.room.*` — CRUD + list (admin).
- `care.child.create/update/get/list` — admin writes; staff read (room-scoped); guardian
  read (edge-scoped). `archive` not `delete` (regulatory retention).
- `care.guardian.*` + `care.guardianship.link/unlink/update` (admin) — edge flags above.
- `care.invite.create/revoke/list` (admin), `care.invite.accept` (pre-auth via token — lb gap #3).
- `care.enrollment.*` — incl. waitlist ordering.
- `care.attendance.check_in/check_out/list` — staff/kiosk; guardian gets the event on the feed.
- `care.log.add/list` — staff write; guardian read (edge-scoped). Photo attach via media seam.
- `care.feed.watch` — SSE live feed per child/room (guardian home screen).
- `care.menu.set/get/week` — admin/staff write, everyone reads (guardian sees own child's
  room + substitutions).
- Batch: `care.enrollment.import` (CSV of children/guardians) → **a job** (lb jobs), per-item
  results, never a blocking loop.

No public write verb without a capability; every verb gets a deny-test.

## Example flow — invite a guardian, live in 5 minutes

1. Admin creates `child:leo` (room Possums, allergy: peanuts) and `guardian` records for
   dad Sam and mum Ana; links two `guardianship` edges. Sam also has `child:mia` from a
   previous relationship — one more edge, nothing else special.
2. Admin `care.invite.create` for Sam → invite record + token; email delivered via an
   outbox target (lb gap #3).
3. Sam opens the link on his phone → thin shell pre-auth accept page → account + workspace
   membership with the **guardian** cap set; `care.invite.accept` binds his `sub` to the
   guardian record. He now sees **both Leo and Mia** — and Ana sees only Leo.
4. Staff check Leo in at 08:02; Sam's feed shows it live (`care.feed.watch` over SSE).
5. 11:30: staff logs lunch from today's `menu` — Leo's entry shows the peanut-free
   substitution; a photo of the fingerpainting posts to the feed.
6. Ana messages the room-leader in Leo's channel; the room-leader answers from the staff
   view. Mia's mum never sees any of it.

## Testing plan (per lb `testing-scope.md` — real infra, no mocks)

- **Capability-deny** (mandatory): staff denied `care.child.update`; guardian denied every
  admin/staff verb; kiosk principal denied everything but `care.attendance.*`.
- **Workspace isolation** (mandatory): two organizations seeded; admin of A gets
  403/empty on every B record.
- **Cross-family isolation (the product-critical one, run per verb):** seed Sam(Leo+Mia),
  Ana(Leo), Mia's-mum(Mia); assert Ana can never read/list/watch/message anything of Mia's
  — including via `list` filters, the SSE feed, and channel membership after an
  `unlink`.
- **Edge-change reconciliation:** unlink an edge → feed stops, channel membership revoked,
  cached reads gone.
- **Integration:** invite→accept→login→feed E2E against a real booted node; enrollment
  import as a real job with partial-failure results.
- Externals (email, later Stripe/push) are the *sanctioned* fake: one trait, one named file.

## Risks & hard problems

- **A cross-family data leak** is the existential bug. Mitigation: one authz chokepoint,
  per-verb deny-tests, and pushing lb gap #1 so the platform enforces it below the ext.
- **Media at daily-photo volume** (every child, every day) through SurrealDB — size,
  thumbnails, mobile upload on flaky networks. Needs the lb media seam designed, not improvised.
- **Pre-auth surfaces** (invite accept) widen the attack surface; must land with lb's
  login hardening, not before.
- **Channel-membership drift** vs guardianship edges — reconciliation must be idempotent
  and event-driven, not a cron hope.
- **Regulatory posture** (child data!): retention (archive-not-delete), audit trail of who
  saw what, immunization/allergy correctness. Cheap now, brutal to retrofit.

## Scope map (build order across the sub-scopes)

The master scope splits into per-feature sub-scopes; `../README.md` carries the full table.
Build order — each lb dependency ships (tag) before its consumer starts:

1. lb `auth-caps/entity-scoped-grants-scope.md` → [`care-authz-scope.md`](care-authz-scope.md)
   (the chokepoint can start ext-enforced in parallel — its call sites don't change).
2. Host boot (`rust/node/`) + lb `frontend/minimal-shell-scope.md` →
   [`../ui/mobile-shell-scope.md`](../ui/mobile-shell-scope.md) (interim thin host allowed).
3. lb `auth-caps/invites-scope.md` → [`enrollment-invites-scope.md`](enrollment-invites-scope.md).
4. [`attendance-scope.md`](attendance-scope.md) (kiosk rides shipped lb api-keys) and
   [`menus-scope.md`](menus-scope.md) (no upstream dep) — parallelizable.
5. lb `files/media-scope.md` + `inbox-outbox/push-target-scope.md` →
   [`daily-feed-scope.md`](daily-feed-scope.md).
6. [`messaging-scope.md`](messaging-scope.md) (shipped lb channels; one possible small ask).
7. **Last — after everything above ships:** [`../billing/billing-scope.md`](../billing/billing-scope.md)
   (own extension; explicitly deferred — see that file's status line).

## lb gaps (fix upstream, in this order)

Rule-10-clean: every one is a generic seam, none names this product. **Status 2026-07-11:
the five real gaps now have written lb scopes** (in `NubeDev/lb` → `docs/scope/…`); two of
the original seven turned out to be covered by existing lb scopes and were folded in.

1. **Entity-scoped grants (row-level authorization).** Caps gate *tools*; this product
   needs *per-record* reach (guardian→their children). → **written:** lb
   `auth-caps/entity-scoped-grants-scope.md` — an additive `scope` selector on the
   authz-grants record + `check_scoped`/`scope_filter` via SDK host-callback. Until it
   ships, `care-authz-scope.md`'s chokepoint enforces it ext-side. **The blocker.**
2. **Minimal shell package.** → **written:** lb `frontend/minimal-shell-scope.md` — the
   publishable host-side contract (auth + invite-accept + `ext.list` mount + SSE + theme +
   PWA), retiring vendor-the-whole-shell (the rubix-ai compromise).
3. **Invite-by-email onboarding.** → **written:** lb `auth-caps/invites-scope.md` —
   single-use invite records with role/team intent + opaque payload, outbox email delivery,
   one pre-auth accept route, atomic identity+membership+grants with caps live on first
   login (also absorbs the re-login-for-caps friction for joiners).
4. **Media/blob seam.** → **written:** lb `files/media-scope.md` — resumable
   begin/chunk/commit upload, variant jobs (thumbs), capability-checked streaming serve
   with Range/ETag, on SurrealDB buckets (rule 2 intact).
5. **Push notifications.** → **written:** lb `inbox-outbox/push-target-scope.md` — push as
   one more outbox `Target`: per-member device records, FCM/APNs/WebPush behind one
   `PushProvider` trait, quiet-hours prefs gate.
6. **Device/kiosk principals — no new gap.** Covered by lb's existing
   `auth-caps/api-keys-scope.md` (machine principals, hashed bearer, instant revoke);
   `attendance-scope.md` consumes it and verifies its shipped state.
7. **Cap refresh without re-login — narrowed.** lb's `builtin-role-freshness-scope.md`
   fixed stale built-in roles; the invites scope makes first-login caps live. Remaining
   slice (mid-session grant changes → live sessions) is tracked in the invites/access
   console freshness levers, not a new scope.

## Open questions

- Product/brand name (`cc-app` is a working name) and org placement (NubeIO? new org?).
- PWA-only for phase 1, or commit to the RN shell (lb `app/shell` line) earlier?
- Billing provider (Stripe assumed) and whether subsidy handling is phase 2 or 3.
- Offline posture for staff tablets (lb sync exists — how much do rooms need offline?).
- Photo consent model per child (some guardians forbid photos — a `guardianship`/`child`
  flag the feed and media path must honor?).

## Related

- lb: `README.md` §3 (rules), `docs/scope/extensions/ext-out-of-tree-scope.md`,
  `docs/scope/node-roles/embed-node-scope.md`, `docs/scope/auth-caps/login-hardening-scope.md`,
  `docs/scope/testing/testing-scope.md`.
- rubix-ai: `docs/WORKFLOW-LB.md` (the family workflow this repo mirrors),
  `docs/extensions/README.md` (build→publish loop).
- This repo: `../../WORKFLOW-LB.md`, `../../STATUS.md`,
  `../../../doc-site/content/public/care/care.md` (stub).
- Market: mybrightwheel.com · lillio.com · kidsday.com "10 best childcare management apps of 2026".
