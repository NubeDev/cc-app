# `rust/extensions/care/src/authz/`

**The chokepoint.** Every read and write verb in the care extension passes through
this module. Two-call surface (per the build milestone 02 spec):

- `assert_reach(principal, child_id)` — for `get`/`update`/`watch` paths.
- `reachable_children(principal)` / `reachable_rooms(staff)` — for `list` paths.

Admin passes via an **audited role check**, never a call-site bypass.

## Era-1 / era-2

- **Era 1 (milestone 02):** resolve from `guardianship` / staff-assignment records
  per call, per-request cache only. Pure ext-side enforcement.
- **Era 2:** same two calls, swapped at the delegation point to lb's
  `authz.check_scoped` / `authz.scope_filter` once that ships (lb gap #1:
  `entity-scoped-grants-scope.md`). Call sites do not change.

## Owner

Filled by build milestone
[`../../../../../docs/build/02-care-skeleton-authz.md`](../../../../../docs/build/02-care-skeleton-authz.md).
Governed by
[`../../../../../docs/scope/care/care-authz-scope.md`](../../../../../docs/scope/care/care-authz-scope.md)
and the master [`../../../../../docs/scope/care/care-scope.md`](../../../../../docs/scope/care/care-scope.md) §Personas.

## Rules

- CLAUDE.md rule 7 (sacred — see [`../../../../../CLAUDE.md`](../../../../../CLAUDE.md)).
  A `guardianship` read outside this module fails CI (grep fence, milestone 02 exit gate).
- Every new verb ships a cross-family deny-test (the matrix harness seeds
  Sam(Leo+Mia) / Ana(Leo) / Mia's-mum(Mia) and asserts isolation).