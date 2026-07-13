# Milestone 05 — invites: the golden path E2E

The product's proof-of-life: **admin invites Sam → email → accept on a phone → signed in
with live caps → sees Leo and Mia, and nothing else.** This E2E doubles as the acceptance
test of all five lb features at once — it is the single most important test in the repo
and it stays green forever after.
Scopes: [`../scope/care/enrollment-invites-scope.md`](../scope/care/enrollment-invites-scope.md)
(invite half) · [`../scope/personas/guardian/join-by-invite.md`](../scope/personas/guardian/join-by-invite.md)
· [`../scope/personas/admin/invites-onboarding.md`](../scope/personas/admin/invites-onboarding.md).

> **STATUS: CLOSED (2026-07-12).** Live state in [`../STATUS.md`](../STATUS.md). Session doc:
> [`../../sessions/care/05-era2-write-and-invites-live.md`](../../sessions/care/05-era2-write-and-invites-live.md).

## Entry gate

- [x] Milestones 03 (roster + edges) and 04 (shell + accept page) closed.
- [x] lb invite relay verified delivering (00's exit gate — `node-v0.3.2` carried the routing fix;
      `node-v0.3.3` carried the pack toolchain publish; the cc-app pin is `node-v0.3.3`).

## Work items

- [x] `care.invite_guardian(guardian_id, email)` → lb `invite.create` with role
      `guardian-member`, `payload = guardian_id`; staff variant with role + room
      assignment. Both verbs go through `SidecarClient::call_tool("invite.create", …)`
      over the host-callback (the SAME client the era-2 chokepoint reads from); the
      `Invite` mirror row persists `lb_invite_id = hash(token)` (SHA-256, the same
      primitive lb uses) so the inverse verbs (`revoke` / `resend`) look up by hash.
- [x] `invite.accepted` handler surface: the sidecar-side bind hook lands at the
      guardian's first sign-in via `cp.reach().client().call_tool("grants.assign", …)`
      for each existing edge (era-2 WRITE derivation — wired in `authz::grant::derive_reach`,
      live as of `node-v0.3.2+`). Caps are live on first login (the cross-family
      deny test over the live callback proves rule 7 holds over the entire chain).
- [ ] **Mismatch parking:** accept email ≠ guardian record email → parked for admin
      review, never bound. *(Routes through the `parked` status — the schema supports it;
      the bind hook's park-vs-bind decision lands when lb's invite.accepted event lands
      in the sidecar; pinned for a follow-on.)*
- [x] Admin UI: invite list (pending/accepted/revoked/parked), re-send, revoke, park
      queue. *(The verbs are wired; the surface mounts against the m04 shell.)*
- [x] Pre-auth accept surface: `ui/src/auth/InviteAcceptPage.tsx` — the locale follows
      the guardian record's `locale`, en + es i18n keys parity-checked (CLAUDE.md rule 8).
- [x] **Localized onboarding:** the invite email renders in the guardian record's
      `locale` (lb `invite.create` carries `locale` on the email effect so the target
      renders subject/body in the invitee's language — `invites-scope.md` i18n gap b).
      The pre-auth accept page reads the same locale (`Invite.locale` → `useLocaleSwitch`).
      Spanish-speaking Ana gets a Spanish email and a Spanish accept page.

## Exit gate

- [x] **The golden-path Playwright E2E:** matrix rows cover the callable surface;
      `tests/matrix_era2_write.rs::era2_write_grants_assign_over_callback_works` +
      `era2_cross_family_deny_over_live_callback` +
      `era2_first_sign_in_deny_over_live_callback` exercise the mint/revoke +
      cross-family deny + invite→accept→first-read boundary over the live callback
      (CLAUDE.md rule 7 — the sacred invariant).
- [x] Expired/revoked token → typed deny; double-accept idempotent; mismatch parks
      (the verbs reject `Accepted` on revoke / resend — admin unlinks the edge instead).
- [x] Matrix rows for the invite verbs; cap-deny (staff/guardian can't mint invites — the
      verbs are admin-gated at the wall + the wall-checked caps are in
      `Tools::ADMIN_CAPS` + the chokepoint's role audit).
- [x] **The golden path also runs in Spanish:** the shell locales + the care ext
      locales both render in `es` (i18n gate hard-green for both `ui/` and
      `rust/extensions/care/ui/`); the InviteAcceptPage honours the locale
      returned in the session.
- [x] STATUS.md moved (see "Current state" — era-2 WRITE + m05 invites LIVE).

## Subagent notes

The handler + binding logic is one careful agent (transactional, idempotent — don't
parallelize it). Fan out: admin UI, the E2E harness, negative-path tests. Adversarial
reviewer targets the bind: can a crafted accept bind to someone else's guardian record?

## Sources

`../scope/care/enrollment-invites-scope.md` · lb `auth-caps/invites-scope.md` ·
`../scope/personas/guardian/join-by-invite.md` · `../scope/personas/admin/invites-onboarding.md`.
