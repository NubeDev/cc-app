# Milestone 06 — attendance (parallelizable with 07)

Check-in/out ledger, the pickup safety gate, kiosk device, ratio read-out.
Scope: [`../scope/care/attendance-scope.md`](../scope/care/attendance-scope.md).

## Entry gate

- [ ] Milestone 03 closed (roster, authorized-pickup list on the child record).
- [ ] lb `api-keys` verified shipped as the scope assumes (machine principals, hashed
      bearer, instant revoke) — verify first, file the lb ask if reality differs.

## Work items

- [ ] `attendance_event` append-only ledger (corrections = compensating events with
      `correction_of`, never edits); staff presence in the same table (recommended —
      resolve the open question).
- [ ] Verbs: `care.attendance.check_in/check_out` (staff + kiosk),
      `care.attendance.list` (admin all / staff room / guardian edges),
      `care.attendance.now` (per-room `{children, staff, ratio}`),
      `care.attendance.correct`, the admin-capped audited pickup-override.
- [ ] **Pickup gate:** collector must be a `can_pickup` guardian or an authorized-pickup
      entry → else hard deny + staff-visible reason + `custody_notes` surfaced; override
      is admin-capped and audited.
- [ ] Kiosk: an lb API key granted exactly the two check verbs + minimal roster read;
      guardian PIN self-check-in is phase 1.5 — record the decision, don't build it now.
- [ ] Emit the bus event the feed will consume (subject shape agreed with milestone 08 —
      coordinate if running in parallel).
- [ ] Staff UI: room roster tap in/out + collector picker
      ([`../scope/personas/teacher/check-in-out.md`](../scope/personas/teacher/check-in-out.md));
      Admin UI: occupancy/ratio dashboard + history + corrections
      ([`../scope/personas/admin/operations-oversight.md`](../scope/personas/admin/operations-oversight.md)).

## Exit gate

- [ ] Full deny sweep: guardian can't check in; kiosk key denied everything beyond its
      two verbs; cross-room staff denied; matrix rows on `list`.
- [ ] Unauthorized pickup denied + audited override path tested; `now` correct across
      in/out/correct sequences; kiosk key revocation kills access instantly (E2E).
- [ ] Both persona journeys pass on a real node through the UI, in `en` **and** `es`
      (the pickup **deny reason** is an enum rendered per locale — a Spanish-speaking
      teacher must read why the gate refused; catalog CI gate green).
- [ ] Open questions resolved: PIN per guardian (recommended), staff same-table
      (recommended).
- [ ] STATUS.md moved.

## Subagent notes

Ledger + gate = one careful agent. Fan out verbs/tests/UI after. The pickup deny is a
child-safety control: the adversarial reviewer's brief is "bypass the gate" (UI path,
direct verb call, kiosk key, correction abuse).

## Sources

`../scope/care/attendance-scope.md` · lb `auth-caps/api-keys-scope.md` · teacher/admin
persona docs above.
