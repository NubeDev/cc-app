# Milestone 00 — release the lb work (upstream)

**Repo: `NubeDev/lb`** (sibling checkout `../lb`). Nothing in cc-app can start until lb's
five features exist as **tags**, not a branch. Status at writing (2026-07-11): all five are
implemented and green on lb branch `updates-to-core` (grants 18c60cb, invites 62a3bf2,
media f958f48, push a629378, minimal-shell 3c20433) — unmerged, untagged.

## Entry gate

- [ ] `../lb` checkout exists and `updates-to-core` builds green.

## Work items

- [ ] **Close the two pre-tag remainders** (found in the 2026-07-11 review):
  - [ ] Wire `spawn_relay_reactors` with `EmailTarget` and `PushTarget` at boot — invites
        and push currently enqueue to an outbox nothing drains.
  - [ ] Rate-limit `POST /public/invite/accept` (pre-auth public route; lb's
        login-hardening posture applies).
- [ ] **Verify lb's multi-lang coverage against cc-app's i18n MUST**
      ([`../scope/ui/i18n-scope.md`](../scope/ui/i18n-scope.md)): (a) a locale
      field/preference on the member + readable pre-auth, (b) locale-aware outbox
      **email templates** (invites), (c) locale-aware **push** rendering, (d) an SDK
      string-catalog mechanism for extension UI + shell. Any hole is an lb gap — fix it
      generically **while in this repo**, before the tag (retro-tagging for i18n later is
      exactly the churn this milestone exists to avoid).
- [ ] PR `updates-to-core` per lb's own HOW-TO-CODE (split entity-scoped-grants out first
      if review size demands); merge.
- [ ] Tag: `node-vX.Y.Z` (and SDK tags only if the SDKs changed — the grants work chose
      MCP host-callback precisely to avoid a WIT change).
- [ ] Publish/tag `@nube/minimal-shell` so `ui/` can depend on it.
- [ ] In cc-app: record the pinned tags here → `lb-node = { git, tag = "____" }`,
      minimal-shell version `____`.

## Exit gate

- [ ] All five features reachable from **tags** (no branch refs).
- [ ] lb test suite green on the tag.
- [ ] A scratch consumer (or lb's own harness) proves: invite email actually delivered by
      the reactor; a push actually handed to the (fake) provider — i.e. the wiring items
      above are observed working, not just compiled.
- [ ] cc-app `docs/STATUS.md` updated with the tags.

## Notes for the session

Deferred-not-blocking (track in lb, don't do now): media Range requests (**required before
cc-app milestone 08 ships video — photos-only until then**), orphaned-upload housekeeping,
real WebPush VAPID provider (fake provider is fine through milestone 08's tests), FCM/APNs.

## Sources

`../WORKFLOW-LB.md` §4 · lb `docs/scope/auth-caps/{entity-scoped-grants,invites}-scope.md`,
`files/media-scope.md`, `inbox-outbox/push-target-scope.md`, `frontend/minimal-shell-scope.md`.
