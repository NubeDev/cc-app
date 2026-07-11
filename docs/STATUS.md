# STATUS — cc-app

_The single "where are we" dashboard. Read at the start of a session; update at the end._

**Date:** 2026-07-12

## Current state

**MILESTONES 00 + 01 + 02 + 03 CLOSED, MILESTONE 04 NEXT.** Four of eleven
milestones are shipped — lb-release (the pin bump), the host boot shim, the
care extension skeleton + the authz chokepoint, and the full enrollment roster
(children, guardians, guardianship edges + the five flags, centers, rooms,
enrollment + waitlist FIFO). The host boots a real embedded lb node from a
fresh checkout; the authz chokepoint is the ONE place guardian reach is
resolved and now **delegates its READ path to lb's entity-scoped grants over
the native host-callback** (era-2, proven over a real gateway); the i18n `t()`
catalog resolves en/es and the hardcoded-string lint is hard. `cargo test
--workspace` is **87 passed, 0 failed**, clean from git tags alone (no
`[patch]`).

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
- **Era-2 delegation — READ path LIVE** (2026-07-12): the chokepoint delegates
  `assert_reach`/`reachable_children` to lb's `authz.check_scoped` /
  `authz.scope_filter` over the node-v0.3.0 native host-callback
  (`SidecarClient`, re-exported from `lb-ext-native`), call sites unchanged.
  Proven end to end over a REAL booted gateway (`tests/matrix_era2.rs`). The
  DERIVATION half (minting scoped grants on `guardianship.link/unlink` via
  `grants.assign`/`revoke`) is **wired but blocked** by an lb gap: `/mcp/call`
  routes only `authz.*`, not `grants.*`, so a native ext can't mint a grant
  over the callback yet. **Until lb routes `grants.*`, era-1 (store edges) is
  the live reach path** (`docs/debugging/authz/grants-verbs-not-on-mcp-callback-surface.md`;
  an additive one-arm lb fix → tag → pin bump).
- **Milestone 03 — enrollment CLOSED** (2026-07-12): the full roster ships —
  orchestrator-owned schemas + verb-per-file bodies (≤400 lines) for
  `care.center.*`, `care.room.*`, `care.child.create|update|get|list|archive`
  (archive not delete; reach-filtered reads), `care.guardian.create|get|list`
  (records-before-accounts), `care.guardianship.link|unlink|update` (the five
  edge flags; era-2 grant derivation), `care.enrollment.create|update|list`
  (waitlist FIFO per room). **i18n `t(locale,key,vars)`** resolves en/es from
  the embedded catalogs (33 leaf keys, parity-checked); the hardcoded-string
  lint is now **hard** (`exit 1`, scoped to genuine chrome). Era-2 read
  delegation is live (above). An adversarial review found + we fixed a
  `child.list` reach/id-form lockout (allow-case tests added). **DEFERRED:**
  `care.enrollment.import` (lb/jobs), the admin UI (m04). `cargo test
  --workspace` **87 passed**. Session doc:
  [`sessions/care/03-enrollment-session.md`](sessions/care/03-enrollment-session.md).

## Deferred (per the milestones, not yet started)

- **Milestone 00 — lb-release: DONE** (2026-07-12). Pinned `node-v0.3.0` /
  `sdk-v0.3.0`; dropped the `[patch]` block from the git-ignored
  `.cargo/config.toml`; `cargo build`/`test --workspace` clean FROM TAGS ALONE.
- **lb follow-up (upstream, blocks era-2 write path)**: route `grants.*` /
  `roles.*` / `teams.*` through lb's `/mcp/call` dispatcher (one additive arm
  in `tool_call.rs`) so a native extension can mint scoped grants over the
  host-callback. Then tag + bump the pin here. See
  `docs/debugging/authz/grants-verbs-not-on-mcp-callback-surface.md`.
- **Milestone 03 — `care.enrollment.import`**: the lb/jobs CSV integration
  (deferred this session; records/verbs it lands into are all shipped).
- **Milestone 04 — mobile-shell**: NEXT. `minimal-shell-v0.2.0` shipped, so it
  can start. Login → mounted ext on a phone; the backend verbs + i18n are ready.
- **Milestones 05 + 06 + 07 + 08 + 09 + 10**: per the build map.
- **Billing: build LAST** (product decision 2026-07-11). `scope/billing/billing-scope.md`
  stays only as the must-not-preclude ledger; no billing work before phase-1 ships.

## Local-dev posture (the WORKFLOW-LB.md §3 path)

The git-ignored `.cargo/config.toml` now carries ONLY:

- **zigcc linker wiring** (this box has no system C compiler).
- **ZIG cache redirect** to `/tmp/kilo/zig-cache` (sandbox quirk —
  `/home/user/.cache/zig` is read-only).
- `jobs = 4` (the RAM-heavy link step OOM-killed at 6 with the editor resident).

The lb `[patch]` block is **GONE** — cc-app builds straight from the published
git tags (`node-v0.3.0` / `sdk-v0.3.0`). A clean `cargo build --workspace` with
no path/patch is the "am I on releases?" check. No path/`[patch]` is committed.

## Next up

**Milestone 04 — mobile-shell** (`ui/`): login → federation-mounted care ext on
a phone (360px) + laptop-good, dark/light, shadcn only, en + es. Build the first
real admin screens (Centers/Rooms, child editor, family/edges editor, waitlist)
against the shipped m03 verbs via the impeccable skill; `remoteEntry.tsx` = one
`defineRemote(...)`. In parallel/after: land the lb `grants.*`-routing fix
(unblocks era-2 write) and `care.enrollment.import`.

## Non-goals (unchanged)

- No special-casing of lb or any extension (rule 10).
- No vendored lb UI shell — 100% of the product UI is extension UI behind `defineRemote`.
- No billing/payments in phase 1 (scope §Phases).
