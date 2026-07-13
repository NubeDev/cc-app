# Milestone 03 — enrollment: the roster

Children, guardians, guardianship edges, centers, rooms, enrollment/waitlist, CSV import.
After this milestone the domain is real and every later feature hangs records off it.
Scope: [`../scope/care/enrollment-invites-scope.md`](../scope/care/enrollment-invites-scope.md)
(the invite *flow* half is milestone 05 — this one builds the records + admin CRUD).

> **STATUS: CLOSED (2026-07-12).** Live state in [`../STATUS.md`](../STATUS.md). Two exit-gate
> items were explicitly **deferred forward** (not skipped): the `es` admin UI E2E landed in
> **m04** (mobile shell), and `care.enrollment.import` is scheduled with **m05** invites — both
> tracked below and in the m03 session doc.

## Entry gate

- [x] Milestone 02 closed (chokepoint + matrix harness live).

## Work items

- [x] Records: `center`, `room`, `child` (full safety data incl. photo-consent flag),
      `guardian`, `guardianship` edge (relationship + the five flags), `enrollment`
      (schedule, `waitlist|enrolled|withdrawn`), staff room assignments. All
      workspace-scoped.
- [x] Verbs (verb-per-file): `care.center.*`, `care.room.*`,
      `care.child.create/update/get/list/archive` (archive, never delete),
      `care.guardian.*`, `care.guardianship.link/unlink/update`,
      `care.enrollment.create/update/list` + waitlist ordering.
- [x] **Era-2 grant derivation** in `link`/`unlink` (transactional with the edge write) —
      wired; era-1 was the live path this milestone (era-2 WRITE goes live at m05 with the
      node-v0.3.3 grants-routing fix).
- [ ] `care.enrollment.import` as a real lb **job**: CSV children+guardians+edges,
      per-item results, hard-fail rows on medical fields, idempotent on natural keys.
      *(DEFERRED → scheduled with m05; records/verbs it imports into are all shipped.)*
- [x] Admin UI slice (first real screens): Centers/Rooms, child editor, family/edges
      editor, waitlist —
      [`../scope/personas/admin/setup-center.md`](../scope/personas/admin/setup-center.md) +
      [`enroll-family.md`](../scope/personas/admin/enroll-family.md) are the acceptance
      journeys. `remoteEntry.tsx` = one `defineRemote(...)`. *(Shipped in m04.)*
- [x] **i18n bootstrap (first UI = the moment it starts):** the `en` + `es` catalog
      structure via lb's mechanism, the CI catalog-completeness gate, the
      hardcoded-string lint, and `locale` on the `guardian` record (pre-account — invites
      need it in 05). Workspace default locale a settable field.

## Exit gate

- [x] Every verb above shipped end-to-end with its cap-deny test (staff
      `care.child.update` → 403 is the canonical one) + its **matrix row**.
      *(2026-07-12: DONE — center + room (prior) plus child (create, update,
      get, list, archive), guardian (create, get, list), guardianship (link,
      unlink, update), enrollment (create, update, list). Real-store unit
      tests per verb; cross-family allow+deny for the rule-7 child reads in
      `tests/matrix_child_reads.rs`; the chokepoint primitives every read verb
      uses are covered in `tests/matrix_chokepoint.rs`. 87 workspace tests.)*
- [x] Unlink → immediate deny (era-2 grants asserted removed in the same transaction).
      *(2026-07-12: DONE — `unlink_immediately_denies` green (era-1 live path);
      era-2 `tests/matrix_era2.rs` additionally asserts the scoped grant is
      PHYSICALLY GONE after revoke — `scope_filter` returns no ids, not merely
      a denied read — over a real gateway. `guardianship.unlink` removes the
      grant transactionally with the edge archive; a failed rollback surfaces
      the divergence via a typed error.)*
- [x] Archive semantics: invisible to guardians, recoverable by admin.
      *(2026-07-12: DONE — `care.child.archive` sets `archived`; `child.get`
      hides an archived child from non-admin, `child.list` filters it; admin
      sees it (audit); `restore:true` recovers it. Tested.)*
- [ ] Import: 40-row fixture, 2 bad rows → 38 land + per-item errors; re-run duplicates
      nothing.
      *(DEFERRED (not this session) — `care.enrollment.import` is an `lb/jobs`
      integration; the session brief deferred it. Tracked in the 03 session
      doc's "Deferred" section. Records/verbs it imports into are all shipped.)*
- [ ] Admin can do the two persona journeys on a real node through the UI — **and the
      screens render fully in `es`** (catalog CI gate green; E2E once as an `es` user).
      *(DEFERRED — UI is milestone 04 (mobile-shell). The backend half is ready:
      i18n `t()` resolves en/es (catalog parity CI green), all admin verbs ship.)*
- [x] Open questions resolved: authorized-pickup as child-record entries v1 (recommended),
      waitlist FIFO v1 (recommended).
      *(2026-07-11: resolved — authorized-pickup is a child-record field v1;
      waitlist is FIFO per room v1. Both implemented this session.)*
- [x] STATUS.md moved.
      *(2026-07-12: moved to "milestone 03 CLOSED, milestone 04 next".)*

## Subagent notes

Orchestrator fixes record schemas + cap names **first** (subagents never decide schemas),
then fan out one agent per verb-family with its tests; one agent on the import job; UI
agents after verbs are callable. Adversarial reviewer sweeps for chokepoint bypasses and
matrix gaps.

## Sources

`../scope/care/enrollment-invites-scope.md` · `../scope/care/care-authz-scope.md` ·
`../scope/personas/admin/{setup-center,enroll-family}.md` · lb jobs scope.
