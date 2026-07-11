# Admin use case — invite guardians and staff

**Goal:** turn roster records into logged-in people: email invites that land a guardian on
their child's feed (or a teacher in their room) in one flow, with caps live on first login.

## Journey

1. From a guardian record: `care.invite_guardian(guardian_id, email)` → lb invite with
   role `guardian-member` and `payload = guardian_id`; email goes out via the outbox.
2. Guardian accepts on their phone (guardian journey:
   [`../guardian/join-by-invite.md`](../guardian/join-by-invite.md)); on `invite.accepted`
   the extension binds `sub → guardian` and derives scoped grants from the existing edges.
3. Staff invites: identical, with the staff role + room assignment attached.
4. Admin monitors: invite list with status (pending/accepted/revoked/parked), re-send,
   revoke.
5. Mismatch path: accept email ≠ guardian record email → the invite **parks for admin
   review** instead of binding — admin resolves (fix the record or re-issue).

## Verbs & screens

- `care.invite_guardian` / staff equivalent; lb `invite.create/list/revoke/resend`
  consumed as granted tools.
- Screen: admin → Invites (status list, park queue, re-send/revoke).

## Deny / edge cases

- Staff/guardians cannot mint or revoke invites (403).
- Double `invite.accepted` is idempotent (bind-once).
- Revoked/expired token at the accept page → clear dead-end, no account created.
- The accept route is pre-auth and public — rate-limiting is an **lb-side must** before
  this ships (flagged on the lb invites work).

## Source scopes

[`../../care/enrollment-invites-scope.md`](../../care/enrollment-invites-scope.md) ·
lb `auth-caps/invites-scope.md` (upstream — must ship first) ·
[`../../care/care-authz-scope.md`](../../care/care-authz-scope.md) (grant derivation on bind).
