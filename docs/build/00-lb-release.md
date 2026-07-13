# Milestone 00 — release the lb work (upstream)

> **STATUS: CLOSED.** Live state in [`../STATUS.md`](../STATUS.md). The initial five features
> shipped as `node-v0.2.0` etc.; two later lb follow-ups (grants-routing + the published pack
> toolchain) shipped as **`node-v0.3.3`** — see [`../STATUS.md`](../STATUS.md) and the closed
> entries under [`../debugging/`](../debugging/).

**Repo: `NubeDev/lb`** (sibling checkout `../lb`). Nothing in cc-app can start until lb's
five features exist as **tags**, not a branch. Status at writing (2026-07-11): all five are
implemented and green on lb branch `updates-to-core` (grants 18c60cb, invites 62a3bf2,
media f958f48, push a629378, minimal-shell 3c20433) — unmerged, untagged.

## Entry gate

- [x] `../lb` checkout exists and `updates-to-core` builds green.

## Work items

- [x] **Close the two pre-tag remainders** (found in the 2026-07-11 review):
  - [x] Wire `spawn_relay_reactors` with `EmailTarget` and `PushTarget` at boot — done
        generically: `RouterTarget` + `BootConfig.outbox_providers` (logging no-op default);
        drain-at-boot proven in lb `rust/node/tests/relay_boot_test.rs`.
  - [x] Rate-limit `POST /public/invite/accept` — done (`FixedWindowLimiter`, also applied to
        the new pre-auth `GET /public/invite/verify`).
- [x] **Verify lb's multi-lang coverage against cc-app's i18n MUST** — all four fixed generically
      in lb: (a) invite `locale` + member `language` pref + pre-auth `GET /public/invite/verify`;
      (b) invite email through the prefs catalog (`invite.email.*`, en+es); (c) `notify.send`
      key+args with per-recipient render in `PushTarget`; (d) `@nube/ext-ui-sdk` 0.7.0 i18n seam
      (`resolveLocale`/`makeTranslator`/`catalogParity`) + fully-catalogued minimal-shell with a
      CI key-parity gate.
      **Original MUST text:**
      ([`../scope/ui/i18n-scope.md`](../scope/ui/i18n-scope.md)): (a) a locale
      field/preference on the member + readable pre-auth, (b) locale-aware outbox
      **email templates** (invites), (c) locale-aware **push** rendering, (d) an SDK
      string-catalog mechanism for extension UI + shell. Any hole is an lb gap — fix it
      generically **while in this repo**, before the tag (retro-tagging for i18n later is
      exactly the churn this milestone exists to avoid).
- [x] **Verify the theme seam** — verified + hardened in lb: minimal-shell `ThemeProvider` now
      defaults to the system scheme when no choice is persisted, persists the user toggle, and
      stamps the shadcn-compatible `.dark` class on `documentElement` (plus `color-scheme` +
      `data-theme-mode`) so a host toggle cascades into mounted extension UI.
      **Original MUST text:** the
      minimal-shell theme provider does dark/light with system default + persisted user
      toggle, and the ui-sdk token contract is CSS-variable/semantic so a host-side
      `.dark` swap propagates into mounted extension UI (shadcn-compatible). Holes =
      generic upstream fixes, before the tag.
- [x] PR `updates-to-core` per lb's own HOW-TO-CODE (kept as one PR — the five features are
      already separate commits on the branch); merged to `master`.
- [x] Tag: **`node-v0.2.0`** (minor bump — new verb surface: `invite.verify`, `invite.create
      locale`, `notify.send` catalog keys, `BootConfig.outbox_providers`). SDK tag:
      **lb-ext-ui-sdk `ui-v0.7.0`** (the i18n seam; no WIT/Rust-SDK change).
- [x] Publish/tag `@nube/minimal-shell` → **`minimal-shell-v0.2.0`** (package version 0.2.0;
      consumed via git tag / link — not pushed to npm).
- [x] In cc-app: record the pinned tags here → `lb-node = { git, tag = "node-v0.2.0" }`,
      minimal-shell version `0.2.0` (tag `minimal-shell-v0.2.0`), `@nube/ext-ui-sdk` `0.7.0`
      (tag `ui-v0.7.0`).

## Exit gate

- [x] All five features reachable from **tags** (no branch refs) — `node-v0.2.0`.
- [x] lb test suite green on the tag (one allowed pre-existing failure: `lb-cli reminder_test`,
      logged at lb `docs/debugging/cli/reminder-create-denied-in-cli-round-trip-test.md`).
- [x] lb's own harness proves the wiring end to end: `rust/node/tests/relay_boot_test.rs` boots a
      real node and observes the invite email AND a push land on the recording providers via the
      spawned reactor (not a direct relay call).
- [x] cc-app `docs/STATUS.md` updated with the tags.

## Notes for the session

Deferred-not-blocking (track in lb, don't do now): media Range requests (**required before
cc-app milestone 08 ships video — photos-only until then**), orphaned-upload housekeeping,
real WebPush VAPID provider (fake provider is fine through milestone 08's tests), FCM/APNs.

## Sources

`../WORKFLOW-LB.md` §4 · lb `docs/scope/auth-caps/{entity-scoped-grants,invites}-scope.md`,
`files/media-scope.md`, `inbox-outbox/push-target-scope.md`, `frontend/minimal-shell-scope.md`.
