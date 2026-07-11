# STATUS — cc-app

_The single "where are we" dashboard. Read at the start of a session; update at the end._

**Date:** 2026-07-11

## Current state

**MILESTONES 01 + 02 CLOSED.** Two of eleven milestones are shipped (the
host boot shim + the care extension skeleton + the authz chokepoint +
the cross-family matrix harness). The host boots a real embedded lb
node from a fresh checkout; the authz chokepoint is the ONE place
guardian reach is resolved; `cargo test --workspace` is **17 passed,
0 failed**.

> **Sandbox caveat (read this first).** This work ran in a sandbox
> where `.git` is bind-mounted read-only — the per-milestone commits
> called for by `CLAUDE.md` could NOT land. The full work is on disk
> in uncommitted modifications, in clearly-named files. `git status`
> shows the changeset; re-run the gates from a non-sandbox checkout to
> verify.

## What's real

- Docs: `ABOUT-DOCS.md`, `FILE-LAYOUT.md`, `SCOPE-WRITTING.md`, `HOW-TO-CODE.md`,
  `WORKFLOW-LB.md` (cc-app-adapted mirrors), this dashboard, `scope/README.md`.
- Scopes: the master `scope/care/care-scope.md` **plus the full sub-scope set** —
  `care/{care-authz,enrollment-invites,attendance,daily-feed,menus,messaging}-scope.md`,
  `ui/mobile-shell-scope.md`, `billing/billing-scope.md` (phase-2 placeholder-with-teeth).
  The master's "Scope map" is the build order.
- **Persona layer** (2026-07-11): `scope/personas/{admin,teacher,guardian}/` — one doc per
  use case (6 admin, 4 teacher, 5 guardian), journeys over the feature scopes.
- **Upstream lb gaps IMPLEMENTED** (2026-07-11, in `NubeDev/lb`, branch `updates-to-core`,
  **not yet merged/tagged**): entity-scoped-grants (18c60cb), invites (62a3bf2), media
  (f958f48), push-target (a629378), minimal-shell (3c20433) — 53 tests green. Remaining
  before tag: wire Email/Push relay reactors at boot, rate-limit the public invite-accept
  route. Kiosk = existing lb `api-keys`; cap-freshness folded into invites/access-console.
- **UI stack pinned** (2026-07-11, CLAUDE.md rule 9): shadcn/ui only; mobile-first
  (360px) + laptop-good (~1280px); dark + light mode from day one (host-owned `.dark`
  variable swap, semantic tokens in ext UI). Contract: `scope/ui/mobile-shell-scope.md`
  §"UI stack"; gated in build milestones 04 + 10; theme-seam verification added to 00.
- **Design language pinned** (2026-07-11): **modern iOS on shadcn** — root `PRODUCT.md`
  (strategy/personas/anti-references) + `DESIGN.md` (seed visual system: system font
  stack, large titles, bottom tabs/sheets, OKLCH restrained palette, iOS dark
  elevation). UI milestones build/review via the **impeccable skill**, which auto-loads
  both; `/impeccable document` re-run at milestone 04 captures real tokens.
- **i18n MUST recorded** (2026-07-11): English + Spanish 100% from day one —
  `scope/ui/i18n-scope.md` (CLAUDE.md rule 8), gated per build milestone; lb multi-lang
  coverage verification added to build milestone 00.
- **Repo skeleton scaffolded** (2026-07-11): directory tree under `rust/node/`,
  `rust/extensions/care/` (authz chokepoint + folder-of-verbs per FILE-LAYOUT),
  `rust/extensions/care/ui/`, and `ui/` shell, with per-dir READMEs.
- **Milestone 01 — host boot CLOSED** (2026-07-11): `rust/node/` (the boot
  shim), `BootConfig` filled from `CC_*` env at the binary boundary,
  repo-anchored `.cc-app/` state, `boot_full` → `RunningNode::serve`, a
  real `user:ada` seed, the boot test on `mem://` (gateway health + auth
  round-trip — green, including a live HTTP `POST /login` →
  authed `GET /workspaces` 200 round-trip), `scripts/check-file-size.sh`,
  `.github/workflows/ci.yml`. Session doc:
  [`sessions/node/01-host-boot-session.md`](sessions/node/01-host-boot-session.md).
- **Milestone 02 — care extension skeleton + authz chokepoint CLOSED**
  (2026-07-11): the care extension (native Tier-2, `lb-ext-native` SDK)
  publishes `care.ping`; the **authz chokepoint** is the one-call
  surface every verb uses (`assert_reach` / `reachable_children` /
  `reachable_rooms`); the **cross-family matrix harness** (8 chokepoint
  tests + 4 care.ping tests + 3 ping unit tests, seeded via the real
  write path) is green; the **CI grep fence**
  (`scripts/check-authz-fence.sh`) fails the build on any
  `read`/`list` of `"guardianship"` outside `authz/`. Session doc:
  [`sessions/care/02-care-skeleton-authz-session.md`](sessions/care/02-care-skeleton-authz-session.md).
- **Era-2 delegation seam:** the chokepoint's two calls are stubbed
  behind the same surface as the era-1 path. Both lb verbs
  (`authz.check_scoped` / `authz.scope_filter`) are **already live** in
  the patched lb source — verified by the dev login caps in milestone
  01's HTTP round-trip. The native child tier doesn't yet expose a
  host-callback client in `lb-ext-native`, so the wire-up is a tracked
  TODO (milestone 03 follow-up); the era-2 swap is a one-file fix in
  `rust/extensions/care/src/authz/mod.rs`.

## Deferred (per the milestones, not yet started)

- **Milestone 00 — lb-release**: being executed in a separate session on
  `../lb`. Drop the `[patch]` block in the git-ignored
  `.cargo/config.toml`, pin the new tag in `rust/Cargo.toml`,
  rebuild clean. Two lines. Two checkboxes in `01-host-boot.md` +
  `02-care-skeleton-authz.md` left UNTICKED with a note "pending
  milestone 00 tags".
- **Milestone 03 — enrollment**: next. Records (center/room/child/
  guardian/edge/…) + verb-per-file CRUD + the i18n bootstrap (en/es
  catalog + CI parity + hardcoded-string lint). The orchestrator fixes
  schemas FIRST; subagents handle one verb-family at a time.
- **Milestones 04 + 05 + 06 + 07 + 08 + 09 + 10**: per the build map;
  UI (milestone 04) waits on the in-flight `minimal-shell` lb work
  before it starts; invites (05) wait on `lb/invites` finishing; the
  rest follow.
- **Billing: build LAST** (product decision 2026-07-11). `scope/billing/billing-scope.md`
  stays only as the must-not-preclude ledger; no billing work before phase-1 ships.

## Local-dev posture (the WORKFLOW-LB.md §3 path)

The git-ignored `.cargo/config.toml` carries:

- **zigcc linker wiring** (this box has no system C compiler).
- **`[patch]` block** pointing `lb-node` at `../lb/rust/node` (the
  in-flight `updates-to-core` branch). This lets us build against the
  pre-tag source while milestone 00 executes. Drop the `[patch]`
  block + pin the new tag when milestone 00 lands.
- **ZIG cache redirect** to `/tmp/kilo/zig-cache` (sandbox quirk —
  `/home/user/.cache/zig` is read-only, zig fails with `unable to
  create compilation: ReadOnlyFileSystem` otherwise).

No path/`[patch]` reference is committed — the file is git-ignored.

## Next up

**Milestone 03 — enrollment (records + verbs + i18n bootstrap).**
Schemas first (orchestrator-owned — never a subagent), then one verb
per file with the deny-test + matrix-row shape from milestone 02.

## Non-goals (unchanged)

- No special-casing of lb or any extension (rule 10).
- No vendored lb UI shell — 100% of the product UI is extension UI behind `defineRemote`.
- No billing/payments in phase 1 (scope §Phases).
