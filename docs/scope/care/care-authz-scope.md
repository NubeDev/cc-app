# Care scope — authz: the guardian-scoping chokepoint

Status: scope (the ask). Promotes to `doc-site/content/public/care/` once shipped.
Owning repo: **this repo** (`rust/extensions/care/src/authz/`), consuming **lb**'s
`auth-caps/entity-scoped-grants-scope.md` as it lands.

The product's defining invariant (CLAUDE.md rule 7): **a guardian reaches only the children
they hold a live guardianship edge to** — in every `get`, `list`, `watch`, channel
membership, media byte, and push audience. This scope pins *how* that is enforced: **one
module, two eras** — extension-enforced today, platform-enforced (lb scoped grants) as soon
as it ships — with the same call sites either way.

## Goals

- **One chokepoint:** `authz/` inside the care extension exposes exactly two calls used by
  every verb — `assert_reach(principal, child_id)` and `reachable_children(principal)`.
  No verb queries edges directly; grep-able rule: `guardianship` is read only inside `authz/`.
- **Staff scoping too:** the same module answers room-scoped staff reach
  (`reachable_rooms`), so there is one place where "who sees what" lives.
- **Era 1 (now):** resolve from `guardianship`/staff-assignment records per call, cached
  per-request only (no cross-request cache — staleness = leak).
- **Era 2 (lb entity-scoped grants):** on `guardianship.link/unlink`, the extension derives
  scoped grants via the granted `grants.*` verbs; `authz/` delegates to the SDK's
  `authz.check_scoped`/`authz.scope_filter`. Call sites unchanged — that is the point of
  the chokepoint.
- **Admins pass, but visibly:** admin reach is a role check through the same module (one
  audit point), never a bypass at the call site.

## Non-goals

- No photo-consent / per-log-type visibility flags here (that's `daily-feed-scope.md`
  policy *on top of* reach).
- No custody *legal* logic — `custody_notes` is display data for staff; reach is edges only.

## Intent / approach

A leak is the existential bug, so the design optimizes for auditability over cleverness:
one folder, a boring synchronous API, and a test harness that enumerates **every** care
verb against the two-family fixture. Rejected: per-verb inline filtering (N copies, the
thing this scope exists to prevent); waiting for lb before building (era 1 is small and the
chokepoint makes era 2 a swap, not a rewrite).

## How it fits

- **Tenancy:** all edge/record reads are workspace-scoped as usual; this narrows *within*.
- **Capabilities:** tool-level caps still gate first (guardian cap set from
  `care-scope.md`); this module is the second, row-level gate. Deny = 403 on `get`/`watch`,
  **empty** (not error) on `list`.
- **Rule 9:** tests on the real store with seeded families; no fixture fakes.
- **Rule 10:** era 2 uses only generic lb seams (`grants.*`, SDK callbacks).

## Example flow

Sam (edges: Leo, Mia) calls `care.log.list` → verb asks `reachable_children(sam)` →
`[leo, mia]` → one indexed query. Ana (edge: Leo) calls `care.log.get(mia_log)` →
`assert_reach` → deny → 403. Admin unlinks Ana↔Leo → era 1: next call resolves empty;
era 2: grants removed in the same handler → wall denies.

## Testing plan

The **cross-family matrix** is the product's mandatory suite: seed Sam(Leo+Mia), Ana(Leo),
Mia's-mum(Mia); run *every* care verb (this file keeps the verb checklist current) for each
principal; assert exact allow/deny/empty. Plus: unlink → immediate deny; staff cross-room
deny; admin allow-with-audit; workspace isolation (second org). Every new verb PR must add
its row to the matrix — a verb without a matrix row fails review.

## Risks & hard problems

- A verb bypassing the chokepoint (the grep rule + matrix are the fence).
- Era-2 grant-derivation drift (edge exists, grant missing → lockout; grant survives
  unlink → leak). The link/unlink handler is transactional with the grant calls.

## Open questions

- Do `watch`/SSE subjects filter at subscribe or at emit in era 1? (Recommend emit-side in
  the feed publisher — one place.)
- Authorized-pickup persons who are *not* guardians (grandma can collect but sees no feed)
  — reach-less contact records; confirm in `attendance-scope.md`.

## Related

`care-scope.md` (§family model, §lb gaps #1) · lb `auth-caps/entity-scoped-grants-scope.md`
· `daily-feed-scope.md` · `messaging-scope.md` · `../../CLAUDE.md` rule 7.
