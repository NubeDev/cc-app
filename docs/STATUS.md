# STATUS — cc-app

_The single "where are we" dashboard. Read at the start of a session; update at the end._

**Date:** 2026-07-12

## Current state

**MILESTONES 00 + 01 + 02 + 03 + 04 CLOSED, MILESTONE 05 NEXT.** The
mobile shell + the first real care admin screens ship: every m03 verb
(center / room / child / guardian / guardianship / enrollment) is
wired onto the live `Tools::call` dispatcher (the "honest half-wired
vs whole-contract" gap from m03 step C is closed), and the thin
`ui/` shell mounts the care ext with `defineRemote(...)` over a
phone-first + laptop-good shadcn surface in dark + light, en + es.
The host-owned `:root{}`/`.dark{}` shadcn variable swap propagates into
the ext through the SDK CSS-isolation seam. `cargo test --workspace`
is **89 passed, 0 failed, 1 ignored** (the ignored test is the
regression for the upstream lb `grants.*` fix — runs with `--ignored`
once the patch is on disk). All 4 CI fences hard-green.

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
  DERIVATION half (minting scoped grants on `guardianship.link`/`unlink` via
  `grants.assign`/`revoke`) is **wired but blocked** by an lb gap: `/mcp/call`
  routes only `authz.*`, not `grants.*`, so a native ext can't mint a grant
  over the callback yet. **Until lb routes `grants.*`, era-1 (store edges) is
  the live reach path**
  (`docs/debugging/authz/grants-verbs-not-on-mcp-callback-surface.md`;
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
  `child.list` reach/id-form lockout (allow-case tests added). Session doc:
  [`sessions/care/03-enrollment-session.md`](sessions/care/03-enrollment-session.md).
- **Milestone 04 — mobile-shell CLOSED** (2026-07-12): the **wire-in** lands
  every m03 verb onto the live `Tools::call` dispatcher via `Care::boot(env)`
  (reads the supervisor-injected `LB_EXT_*` env, builds the `SidecarClient`,
  opens the store). A booted-node integration test (`tests/live_wire.rs`,
  4 green) proves `care.ping` + `care.center.create` round-trip end to end
  through the wired-in constructor. The **shell** (`ui/`) implements the
  login → workspace-pick → `ExtMountPage` flow with shadcn-styled inputs,
  a host-owned `:root{}`/`.dark{}` shadcn variable swap, and a top-bar
  EN/ES + light/dark toggle that propagates into the ext. The **care ext
  UI** (`rust/extensions/care/ui/`) now ships four admin surfaces against
  the m03 verbs: **Centers/Rooms list + create**, **Child editor** (safety
  data: DOB + allergies + medical notes + photo consent, with a `⚠` row
  badge for any allergies), **Family/Edges editor** (the five flags:
  `can_pickup` / `receives_daily_feed` / `receives_billing` /
  `emergency_contact` / `custody_notes`), **Waitlist** (FIFO per room
  ordered by `waitlist_seq`). **en + es parity** at 96 keys each (i18n
  gate hard-green for both shell + ext), **mobile + laptop** viewports
  encoded via content max-widths + bottom-tab layout, **dark + light**
  via the host-owned `.dark` variable swap that propagates through the
  SDK CSS-isolation seam into the ext. Session doc:
  [`sessions/care/04-mobile-shell-session.md`](sessions/care/04-mobile-shell-session.md).
- **lb follow-up — patch written, awaiting PR**: the upstream additive fix
  (one `else if` arm in `rust/crates/host/src/tool_call.rs` routing
  `grants.*` / `roles.*` / `teams.*` to `call_authz_tool`) is at
  `docs/debugging/authz/lb-grants-routing.patch`. A local lb worktree
  under `/tmp/kilo/lb-workdir/lb-fix` applies the patch and `cargo
  build -p lb-host` is green (3m04s). A `#[ignore]`d regression test
  (`tests/matrix_era2_write.rs::era2_write_grants_assign_over_callback_works`)
  runs against the patched lb (with the temporary `[patch]` block in
  `.cargo/config.toml` pointing at the worktree); it proves the era-2
  WRITE half of `guardianship.link` / `unlink` round-trips over the
  callback. **Sandbox git dir is read-only** — the upstream PR + tag
  dance lands from a writable clone; the patch + the test are ready.

## Deferred (per the milestones, not yet started)

- **Milestone 00 — lb-release: DONE** (2026-07-12). Pinned `node-v0.3.0` /
  `sdk-v0.3.0`; dropped the `[patch]` block from the git-ignored
  `.cargo/config.toml`; `cargo build`/`test --workspace` clean FROM TAGS ALONE.
- **lb follow-up (upstream, blocks era-2 write path)**: route `grants.*` /
  `roles.*` / `teams.*` through lb's `/mcp/call` dispatcher (one additive arm
  in `tool_call.rs`) so a native extension can mint scoped grants over the
  host-callback. Patch is at `docs/debugging/authz/lb-grants-routing.patch`;
  a local worktree proves it green. Open the lb PR + tag `node-v0.3.1` + drop
  the `[patch]` from `.cargo/config.toml` + flip the `#[ignore]`d test live.
  See `docs/debugging/authz/grants-verbs-not-on-mcp-callback-surface.md`.
- **Milestone 03 — `care.enrollment.import`**: the lb/jobs CSV integration
  (deferred this session; records/verbs it lands into are all shipped).
  Accepts children+guardians+edges, per-item results, hard-fail on medical
  fields, idempotent on natural keys; 40-row fixture, 2 bad rows → 38 land.
- **Milestones 05 + 06 + 07 + 08 + 09 + 10**: per the build map. m05
  (invites-golden-path) is the E2E gate — invite→accept→feed; it
  exercises the full path the m04 ships.
- **Billing: build LAST** (product decision 2026-07-11). `scope/billing/billing-scope.md`
  stays only as the must-not-preclude ledger; no billing work before phase-1 ships.

## Local-dev posture (the WORKFLOW-LB.md §3 path)

The git-ignored `.cargo/config.toml` now carries ONLY:

- **zigcc linker wiring** (this box has no system C compiler).
- **ZIG cache redirect** to `/tmp/kilo/zig-cache` (sandbox quirk —
  `/home/user/.cache/zig` is read-only).
- `jobs = 4` (the RAM-heavy link step OOM-killed at 6 with the editor resident).
- (TEMPORARY) A `[patch."https://github.com/NubeDev/lb"]` block pointing
  at the local `node-v0.3.x + grants.* routing fix` worktree under
  `/tmp/kilo/lb-workdir/lb-fix`, so the era-2 WRITE regression test runs
  green. **Drop the patch + bump the lb pin to `node-v0.3.1`** when the
  upstream PR lands.

The lb `[patch]` block stays local until the upstream PR ships
(see the session doc for the precise commit sequence).

## Next up

**Milestone 05 — invites-golden-path** (care ext + `ui/`): the E2E
gate — invite → accept → feed. Reuses the m04 admin surfaces (the
guardian editor + the family/edges editor + the waitlist per child)
and exercises `care.invite.*` + the pre-auth accept page + the
guardian feed surface end to end. The `care.invite_guardian` verb
needs a `<care>` extension to mint an invite (records-before-accounts
binds on `invite.accepted`).

## Non-goals (unchanged)

- No special-casing of lb or any extension (rule 10).
- No vendored lb UI shell — 100% of the product UI is extension UI behind `defineRemote`.
- No billing/payments in phase 1 (scope §Phases).
