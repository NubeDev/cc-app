# Guardian use case — join by invite

**Goal:** from an email on a phone to signed-in on your child's feed, in minutes, with
the right reach from the first screen (no re-login for caps).

## Journey

1. Sam gets the invite email (admin flow:
   [`../admin/invites-onboarding.md`](../admin/invites-onboarding.md)) and taps the link
   on his phone.
2. Thin-shell pre-auth accept page → sets a password/identity → atomically: account +
   workspace membership + guardian cap set; the care extension binds `sub → guardian`
   and derives scoped grants from his existing edges.
3. First screen: the feed — Leo *and* Mia (both edges), nothing else. He installs the
   PWA from the browser prompt.
4. Ana accepts her own invite → sees Leo only.

## Verbs & screens

- lb `POST /public/invite/accept` (pre-auth, token) → `invite.accepted` → extension bind.
- Screens: shell accept page → login → care ext guardian home (feed).

## Deny / edge cases

- Expired/revoked token → clear dead-end page, nothing created.
- Accept email ≠ guardian record email → invite parks for admin review; never binds the
  wrong person to a child (the wrong-person binding risk).
- Double-accept idempotent; deep link after accept lands on the feed, not a blank shell.
- Caps live on first login — a guardian who accepts and sees an empty/denied home is a
  bug (the invites scope absorbed the re-login-for-caps friction).

## Source scopes

[`../../care/enrollment-invites-scope.md`](../../care/enrollment-invites-scope.md) ·
lb `auth-caps/invites-scope.md` · lb `frontend/minimal-shell-scope.md` (accept page) ·
[`../../ui/mobile-shell-scope.md`](../../ui/mobile-shell-scope.md).
