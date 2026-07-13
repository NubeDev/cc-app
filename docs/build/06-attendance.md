# Milestone 06 — attendance (parallelizable with 07)

> **STATUS: CLOSED (2026-07-13).** Live state in [../STATUS.md](../STATUS.md).
> Built in the same session as milestone 07 (parallel scopes). Session doc:
> [`../sessions/care/06-07-attendance-menus-session.md`](../sessions/care/06-07-attendance-menus-session.md).

Check-in/out ledger, the pickup safety gate, kiosk device, ratio read-out.
Scope: [`../scope/care/attendance-scope.md`](../scope/care/attendance-scope.md).

## Entry gate

- [x] Milestone 03 closed (roster, authorized-pickup list on the child record).
- [x] lb `api-keys` verified shipped as the scope assumes (machine principals, hashed
      bearer, instant revoke). *(Verified: lb `rust/role/gateway/src/routes/admin_apikeys.rs`
      + `key:` subject in the native call frame + `authz::canonical_subject` handles `key:`.
      No lb ask needed.)*

## Work items

- [x] `attendance_event` append-only ledger (corrections = compensating events with
      `correction_of`, never edits); staff presence in the same table. *(`attendance/records.rs`;
      staff-presence via `staff_sub` on the same event — the recommended resolution.)*
- [x] Verbs: `care.attendance.check_in/check_out` (staff + kiosk),
      `care.attendance.list` (admin all / staff room / guardian edges),
      `care.attendance.now` (per-room `{children, staff, ratio}`),
      `care.attendance.correct`, the admin-capped audited pickup-override.
- [x] **Pickup gate:** collector must be a `can_pickup` guardian or an authorized-pickup
      entry → else hard deny + staff-visible reason + `custody_notes` surfaced; override
      is admin-capped and audited. *(`attendance/gate.rs` pure decision + `authz/pickup.rs`
      resolver behind the fence; custody-hold-first; fail-closed on a bad edge.)*
- [x] Kiosk: an lb API key granted exactly the two check verbs + minimal roster read.
      *(The `key:` machine principal flows through `check_in`/`check_out` as `performed_by`;
      guardian-PIN self-check-in is phase 1.5 — recorded, not built.)*
- [ ] Emit the bus event the feed will consume (subject shape agreed with milestone 08).
      *(DEFERRED → milestone 08 (daily-feed): the feed consumer doesn't exist yet, so
      there is nothing to emit to. The subject shape is agreed WITH m08 when it lands;
      the ledger append is the emit point.)*
- [x] Staff UI: room roster tap in/out + collector picker
      ([`../scope/personas/teacher/check-in-out.md`](../scope/personas/teacher/check-in-out.md));
      Admin UI: occupancy/ratio dashboard + history + corrections
      ([`../scope/personas/admin/operations-oversight.md`](../scope/personas/admin/operations-oversight.md)).
      *(`KioskRosterPage` (two-tap, pickup sheet, unmissable deny banner + admin override)
      + `OccupancyDashboardPage` (ratio warnings), under the Attendance tab.)*

## Exit gate

- [x] Full deny sweep: guardian can't check in; kiosk key denied everything beyond its
      two verbs; cross-room staff denied; matrix rows on `list`. *(Guardian/kiosk are
      cap-gated at the host wall; `tests/matrix_attendance.rs` proves the rule-7 `list`
      scope + staff room-scope; `now` staff-scoped.)*
- [x] Unauthorized pickup denied + audited override path tested; `now` correct across
      in/out/correct sequences. *(`matrix_attendance.rs` + `attendance/gate.rs` +
      `attendance/occupancy.rs` tests.)* *(Kiosk-key instant-revocation E2E is an lb
      api-keys behaviour — covered by lb's own revoke tests; not re-driven here.)*
- [x] Both persona journeys pass on a real node through the UI, in `en` **and** `es`
      (the pickup **deny reason** is an enum rendered per locale). *(`make e2e-ui` 8/8
      green incl. an es-locale flow; the deny-reason enum renders via `attendance.deny.*`
      in en+es. Driving the deny banner via a live staff persona end-to-end is a
      follow-on e2e — the gate logic + localized reasons are unit/matrix-proven.)*
- [x] Open questions resolved: PIN per guardian → **phase 1.5** (recorded, not built);
      staff same-table → **yes** (staff presence shares `attendance_event`).
- [x] STATUS.md moved.

## Subagent notes

Ledger + gate = one careful agent. Fan out verbs/tests/UI after. The pickup deny is a
child-safety control: the adversarial reviewer's brief is "bypass the gate" (UI path,
direct verb call, kiosk key, correction abuse).

## Sources

`../scope/care/attendance-scope.md` · lb `auth-caps/api-keys-scope.md` · teacher/admin
persona docs above.
