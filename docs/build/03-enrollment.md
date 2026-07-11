# Milestone 03 — enrollment: the roster

Children, guardians, guardianship edges, centers, rooms, enrollment/waitlist, CSV import.
After this milestone the domain is real and every later feature hangs records off it.
Scope: [`../scope/care/enrollment-invites-scope.md`](../scope/care/enrollment-invites-scope.md)
(the invite *flow* half is milestone 05 — this one builds the records + admin CRUD).

## Entry gate

- [ ] Milestone 02 closed (chokepoint + matrix harness live).

## Work items

- [ ] Records: `center`, `room`, `child` (full safety data incl. photo-consent flag),
      `guardian`, `guardianship` edge (relationship + the five flags), `enrollment`
      (schedule, `waitlist|enrolled|withdrawn`), staff room assignments. All
      workspace-scoped.
- [ ] Verbs (verb-per-file): `care.center.*`, `care.room.*`,
      `care.child.create/update/get/list/archive` (archive, never delete),
      `care.guardian.*`, `care.guardianship.link/unlink/update`,
      `care.enrollment.create/update/list` + waitlist ordering.
- [ ] **Era-2 grant derivation** in `link`/`unlink` (transactional with the edge write) if
      stubbed in 02 — the chokepoint delegates from here on.
- [ ] `care.enrollment.import` as a real lb **job**: CSV children+guardians+edges,
      per-item results, hard-fail rows on medical fields, idempotent on natural keys.
- [ ] Admin UI slice (first real screens): Centers/Rooms, child editor, family/edges
      editor, waitlist, import with per-row results —
      [`../scope/personas/admin/setup-center.md`](../scope/personas/admin/setup-center.md) +
      [`enroll-family.md`](../scope/personas/admin/enroll-family.md) are the acceptance
      journeys. `remoteEntry.tsx` = one `defineRemote(...)` — no exceptions.
- [ ] **i18n bootstrap (first UI = the moment it starts):** the `en` + `es` catalog
      structure via lb's mechanism, the CI catalog-completeness gate, the
      hardcoded-string lint, and `locale` on the `guardian` record (pre-account — invites
      need it in 05). Workspace default locale a settable field.

## Exit gate

- [ ] Every verb above shipped end-to-end with its cap-deny test (staff
      `care.child.update` → 403 is the canonical one) + its **matrix row**.
- [ ] Unlink → immediate deny (era-2 grants asserted removed in the same transaction).
- [ ] Archive semantics: invisible to guardians, recoverable by admin.
- [ ] Import: 40-row fixture, 2 bad rows → 38 land + per-item errors; re-run duplicates
      nothing.
- [ ] Admin can do the two persona journeys on a real node through the UI — **and the
      screens render fully in `es`** (catalog CI gate green; E2E once as an `es` user).
- [ ] Open questions resolved: authorized-pickup as child-record entries v1 (recommended),
      waitlist FIFO v1 (recommended).
- [ ] STATUS.md moved.

## Subagent notes

Orchestrator fixes record schemas + cap names **first** (subagents never decide schemas),
then fan out one agent per verb-family with its tests; one agent on the import job; UI
agents after verbs are callable. Adversarial reviewer sweeps for chokepoint bypasses and
matrix gaps.

## Sources

`../scope/care/enrollment-invites-scope.md` · `../scope/care/care-authz-scope.md` ·
`../scope/personas/admin/{setup-center,enroll-family}.md` · lb jobs scope.
