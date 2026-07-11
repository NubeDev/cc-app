# Milestone 05 — invites: the golden path E2E

The product's proof-of-life: **admin invites Sam → email → accept on a phone → signed in
with live caps → sees Leo and Mia, and nothing else.** This E2E doubles as the acceptance
test of all five lb features at once — it is the single most important test in the repo
and it stays green forever after.
Scopes: [`../scope/care/enrollment-invites-scope.md`](../scope/care/enrollment-invites-scope.md)
(invite half) · [`../scope/personas/guardian/join-by-invite.md`](../scope/personas/guardian/join-by-invite.md)
· [`../scope/personas/admin/invites-onboarding.md`](../scope/personas/admin/invites-onboarding.md).

## Entry gate

- [ ] Milestones 03 (roster + edges) and 04 (shell + accept page) closed.
- [ ] lb invite relay verified delivering (00's exit gate).

## Work items

- [ ] `care.invite_guardian(guardian_id, email)` → lb `invite.create` with role
      `guardian-member`, `payload = guardian_id`; staff variant with role + room
      assignment.
- [ ] `invite.accepted` handler: bind `sub → guardian` (idempotent), derive scoped grants
      from existing edges — caps live on first login, no re-login.
- [ ] **Mismatch parking:** accept email ≠ guardian record email → parked for admin
      review, never bound.
- [ ] Admin UI: invite list (pending/accepted/revoked/parked), re-send, revoke, park
      queue.
- [ ] Guardian landing: first screen after accept = the (still minimal) child view — a
      placeholder feed is fine pre-08, but edge-scoped listing must be real.
- [ ] **Localized onboarding:** the invite email renders in the guardian record's
      `locale`; the pre-auth accept page follows the same locale; accept copies it to the
      member preference. Spanish-speaking Ana gets a Spanish email and a Spanish accept
      page — her first touchpoint, in her language.

## Exit gate

- [ ] **The golden-path Playwright E2E:** boot node → seed admin → create Leo/Mia/Sam/Ana
      + edges → invite Sam → capture the email via the recording EmailProvider fake →
      open the accept link in the browser → account created → Sam sees Leo *and* Mia;
      **in the same run** Ana accepts and sees Leo only, and Mia's-mum's view shows no
      trace of Leo. Green output pasted.
- [ ] Expired/revoked token → clean dead-end (no partial account); double-accept
      idempotent; mismatch parks (all tested).
- [ ] Matrix rows for the invite verbs; cap-deny (staff/guardian can't mint invites).
- [ ] **The golden path also runs in Spanish:** an `es`-locale guardian's email, accept
      page, and landing screen asserted localized in the E2E.
- [ ] STATUS.md moved.

## Subagent notes

The handler + binding logic is one careful agent (transactional, idempotent — don't
parallelize it). Fan out: admin UI, the E2E harness, negative-path tests. Adversarial
reviewer targets the bind: can a crafted accept bind to someone else's guardian record?

## Sources

`../scope/care/enrollment-invites-scope.md` · lb `auth-caps/invites-scope.md` ·
`../scope/personas/guardian/join-by-invite.md` · `../scope/personas/admin/invites-onboarding.md`.
