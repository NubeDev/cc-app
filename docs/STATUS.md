# STATUS — cc-app

_The single "where are we" dashboard. Read at the start of a session; update at the end._

**Date:** 2026-07-13 (real EMAIL + PASSWORD login LIVE — `node-v0.4.2`, local tag;
rule 7 enforced in-sidecar — `node-v0.4.0`/`sdk-v0.4.0`; admin-pass fixed via the
caller `admin` marker — `node-v0.4.1`/`sdk-v0.4.1`. Release ritual pending: `*-v0.4.x`
tags cut locally + proven via `[patch]`, NOT yet pushed)

> **EMAIL + PASSWORD LOGIN LIVE (2026-07-13) — the last non-real piece of auth is closed.**
> A person now signs in with an email + a password THEY set (invite→accept→set-password),
> not the dev bootstrap handle, and a WRONG password is rejected. **Root gap (found by
> driving the live node):** `lb_node::boot_full` built its gateway via `Gateway::new_live`,
> which hardwires the password-less `DevTrustAny` check — so an embedded node's `POST /login`
> accepted ANY password (the `PasswordHash` check login-hardening shipped was unreachable
> below the embed seam). **Fixed generically in lb** (`node-v0.4.2`, rule 10): additive
> `BootConfig::credential_mode` (applied via the existing `Gateway::with_credential_check`
> seam) + `BootConfig::seed_credential` (the boot seed argon2-sets the dev admin's password,
> fixing the bootstrap paradox — `identity.set_credential` needs an admin token, unavailable
> before any credential exists). Scope:
> `lb/docs/scope/auth-caps/embedder-credential-mode-scope.md`; lb test
> `rust/node/tests/credential_mode_test.rs` (4 green). **cc-app wired it in:** cc-node reads
> `CC_PASSWORD_LOGIN` + `CC_SEED_PASSWORD` (`boot.rs`); `make dev` now defaults to REAL
> PasswordHash mode (`PASSWORD_LOGIN=1`, `ADMIN_PASSWORD=cc-admin-1234`); the shell's accept
> page sets a password (verify + `/public/invite/accept` — the seam path was wrong before);
> `seed.sh` prints working email-login instructions + a ready accept link (token recovered
> from the outbox effect). **Proven GREEN:** `ui/e2e/email-login.spec.ts` (guardian
> accepts → sets password → email+password login lands on `/workspaces`; wrong password →
> 401 → stays on `/login`), the existing `login.spec.ts` bumped to the seeded admin password,
> `make e2e-ui` 4/4, `cargo test --workspace` green (rule 7 `live_node.rs` intact). Set
> `PASSWORD_LOGIN=` for the old password-less dev mode.
> **Release ritual pending:** push the `*-v0.4.x` branch+tags, bump the declared pins (already
> say `node-v0.4.2`), drop the `[patch]` (WORKFLOW-LB.md §4). Still building via the local
> `[patch]` → sibling checkouts.
>
> **ALSO FIXED 2026-07-13 — the care ext UI blank render at `/ext/<ws>`.** `remoteEntry.tsx`
> passed lazy `() => import(...)` thunks to `defineRemote`; the SDK renders `page(ctx,bridge)`
> directly, so a Promise child threw "Objects are not valid as a React child (found:
> [object Promise])" and the mount showed only skeletons. Rewrote to the SDK contract (static
> imports, `?inline` CSS string, page wrapped in the ext's `LocaleProvider`, widgets as a keyed
> map) + fixed `vite.config.ts` (`extUiSdk` → `defineExtConfig` — a config fragment, not a
> plugin — unblocking `pnpm build`/`make pack`). Guard: `ui/e2e/ext-mount.spec.ts`. Debug doc:
> `docs/debugging/ui/care-ext-blank-render-remoteentry-promise-child.md`. NOTE: the care ext UI
> tree still has a large set of prior-session uncommitted changes + a pre-existing
> `api/care.ts` TS error unrelated to this fix.

> **FOLLOW-ON (2026-07-12) — admin-pass fixed via a caller `admin` marker (`*-v0.4.1`).**
> Driving the REAL seed/e2e flow (not just the synthetic-token tests) surfaced that the
> chokepoint's admin-pass read the JWT `role` enum — but lb mints EVERY session as
> `role: member` (admin power rides caps, never the role enum; `dev_claims` is explicit:
> "the check path reads caps, never role"). So on a real node the bootstrap admin (`user:ada`,
> admin *caps* but `role=member`) was treated as a guardian: `center.list → []`,
> `child.get leo → 403`. The frame's minimal `{sub,ws,role,delegated}` projection carried no
> caps, so the sidecar couldn't tell admin from guardian. **Fixed generically in lb** (rule 10):
> added an additive `admin: bool` to the native-caller frame, host-derived from the caller's
> caps (`lb_host::caps_hold_admin`, the admin-only cap delta) — `node-v0.4.1` (host +
> supervisor + runtime) + `sdk-v0.4.1` (`lb-ext-native::Caller` mirror). cc-app's
> `principal_from_caller` now maps `caller.admin ⇒ WorkspaceAdmin` (ignoring the cosmetic
> role). **Proven on the live node:** admin reads see the roster, `child.get leo → 200`,
> and rule 7 still holds (`tests/live_node.rs`: stranger→leo 403, 0 kids — with the admin
> token now carrying an admin-only cap so `admin` derives true, mirroring production).
> **Release ritual pending:** `node-v0.4.1`/`sdk-v0.4.1` are tagged in the sibling checkouts
> and proven via a local `.cargo/config.toml` `[patch]`; push the tags, bump the cc-app pins
> to `*-v0.4.1`, then drop the `[patch]` (WORKFLOW-LB.md §4). Also fixed: `seed.sh`/`e2e.sh`
> probed `care.ping` (a verb the admin holds no cap for) as the reachability check → false
> "care unreachable"; switched to `care.center.list`.

## Current state

> **RULE 7 ENFORCED IN-SIDECAR (2026-07-12) — the correction banner is resolved.**
> The native wire-in is complete AND guardian isolation (CLAUDE.md rule 7) is now
> enforced end to end on a real spawned sidecar. cc-node boot installs the care
> sidecar via `lb_host::install_native` (`rust/node/src/care_mount.rs`); all record
> I/O flows over the host `store.*` callback to the node's DURABLE store
> (`authz/store.rs` `RecordStore`); and the row-level chokepoint now sees the REAL
> caller. lb shipped native-caller-identity in **`sdk-v0.4.0`** (the additive
> `caller` on the native call frame) + **`node-v0.4.0`** (the `subject`-parameterized
> reach verbs, gated by `mcp:authz.delegate_reach:call`). cc-app bumped the pins,
> threaded the frame caller through `Care::call_with_caller` (retiring the dead
> `LB_EXT_PRINCIPAL_JSON` seam), and the chokepoint asks reach ABOUT the caller
> (`subject = caller.sub`) behind the care install's delegation grant. Proven GREEN
> by `tests/live_node.rs`: install → seed → admin reads see data → **a LINKED
> guardian (Ana) reaches her child, a STRANGER guardian (Mallory) is DENIED (403 on
> get, EMPTY on list)** → data SURVIVES A RESTART. The guardian UI is no longer
> gated. Full write-up + Gap 3 CLOSED:
> `docs/debugging/authz/native-sidecar-not-spawned-and-caller-identity-not-propagated.md`.

> **MILESTONES 06 (attendance) + 07 (menus) CLOSED (2026-07-13).** Both open,
> both no-lb-dependency, built in one session as parallel vertical slices.
> **m07 menus:** the fixed top-9 allergen enum + slot enum (`menu/allergen.rs`),
> the `menu` record (`menu/records.rs`), and the **substitution derivation**
> (`menu/derive.rs` — the food-safety control, three false-negative guards:
> alias/plural fold, untaggable-flags-conservatively, unresolved-is-loud) drive
> `menu.set/get/week/copy_week`. `menu.week` is the guardian medical-leak
> surface — `assert_reach` first, only the asking child's derived rows.
> **m06 attendance:** the append-only `attendance_event` ledger, the **pickup
> gate** (`attendance/gate.rs` pure decision + `authz/pickup.rs` resolver behind
> the fence — custody-hold-first, admin-capped audited override, localized
> `PickupDeny` enum), and the derived `now` occupancy fold drive
> `attendance.check_in/check_out/list/now/correct`. Kiosk = an lb api-key `key:`
> principal (api-keys verified shipped). **UI:** 6 role-aware shadcn screens
> (menu planner/serving/guardian-week; attendance kiosk-roster/dashboard),
> semantic tokens, en+es, wired into a role-aware TabBar (the ext mount context
> now threads the caller's workspace role — it previously dropped it). **Green:**
> 160 care lib + 11 new matrix rows + cc-node tests; all 4 fences; `tsc` + `vite
> build`; **`make e2e-ui` 8/8 incl. an es-locale flow on a real seeded node.**
> An adversarial rule-7 pass found + we fixed 3 issues (a CRITICAL derivation
> miss on plural allergy spellings, a HIGH custody-hold fail-open, a LOW menu
> write room-scope). Session doc:
> [`sessions/care/06-07-attendance-menus-session.md`](sessions/care/06-07-attendance-menus-session.md).
> **Bus-event emit for the feed is DEFERRED → m08** (no feed consumer yet).

**MILESTONES 00 + 01 + 02 + 03 + 04 + 05 + 06 + 07 CLOSED.** Era-2 WRITE is LIVE
(lb `node-v0.3.3` shipped both the `grants.*`/`roles.*`/`teams.*` MCP
routing fix AND the published pack toolchain — `node-v0.3.2` carried the
routing fix; `node-v0.3.3` carried the pack toolchain publish). The
care chokepoint delegates reach resolution over the host-callback AND
mints/revokes scoped grants through the same client on
`care.guardianship.link` / `unlink` / `update`. Era-2 is the wire
construction's default; era-1 stays as the documented fallback (`Care::boot`
falls back when the `LB_EXT_TOKEN` + `LB_GATEWAY_URL` env pair isn't
present — the integration-test boot path). `cargo test --workspace`
exercises the matrix_era2 + matrix_era2_write halves over a real booted
gateway with no in-process fallback (the gate went green with the
`SidecarClient::call_tool("grants.assign" | "invite.create", …)`
call sites — closure proven by `docs/debugging/authz/grants-verbs-not-on-mcp-callback-surface.md`).
Milestone 05 (the invites golden path) is LIVE: `care.invite.create_guardian` /
`create_staff` / `list` / `revoke` / `resend` mint/revoke/resend over the
same `SidecarClient`. The pre-auth accept page (`ui/src/auth/InviteAcceptPage.tsx`)
follows the guardian record's `locale` so a Spanish-speaking Ana gets a
Spanish accept page (CLAUDE.md rule 8). The dev extension packaging flow
now installs the published `lb-pack` at the pinned lb tag
(`cargo install --git https://github.com/NubeDev/lb --tag node-v0.3.3 lb-pack`;
see `Makefile` §LB_TAG + §trusted-pubkey).

## What's real

- Docs: `ABOUT-DOCS.md`, `FILE-LAYOUT.md`, `SCOPE-WRITING.md`, `HOW-TO-CODE.md`,
  `WORKFLOW-LB.md` (cc-app-adapted mirrors), this dashboard, `scope/README.md`.
- Scopes: the master `scope/care/care-scope.md` **plus the full sub-scope set** —
  `care/{care-authz,enrollment-invites,attendance,daily-feed,menus,messaging}-scope.md`,
  `ui/mobile-shell-scope.md`, `billing/billing-scope.md` (phase-2 placeholder-with-teeth).
  The master's "Scope map" is the build order.
- **Persona layer** (2026-07-11): `scope/personas/{admin,teacher,guardian}/` — one doc per
  use case (6 admin, 4 teacher, 5 guardian), journeys over the feature scopes.
- **Upstream lb gaps IMPLEMENTED** (2026-07-11, in `NubeDev/lb`, branch `updates-to-core`,
  tagged `node-v0.3.0`): entity-scoped-grants (18c60cb), invites (62a3bf2), media
  (f958f48), push-target (a629378), minimal-shell (3c20433) — 53 tests green.
- **lb routing fix SHIPPED as `node-v0.3.2`** (2026-07-12, in `NubeDev/lb`, branch
  `authz-verbs-mcp-dispatch`, commit `7988360c` / tag merge `0304acd7`): the additive
  one-arm in `rust/crates/host/src/tool_call.rs` routes `grants.*` / `roles.*` /
  `teams.*` through `call_authz_tool` so a native extension can mint scoped grants
  over the callback. **Era-2 WRITE live in cc-app** (the additive patch + the pin
  bump + the seed swap + the chokepoint's live wire construction — gate proven by
  `tests/matrix_era2_write.rs` three rows over a real booted gateway).
- **lb native-caller-identity SHIPPED as `sdk-v0.4.0` + `node-v0.4.0`** (2026-07-12,
  `lb/docs/scope/extensions/native-caller-identity-scope.md`): GAP A — the additive
  `caller` projection (`{sub, ws, role, delegated}`) on the native call frame
  (`lb-ext-native::CallParams`), threaded by the host, handed to the child via
  `Tools::call_with_caller` (`sdk-v0.4.0`); GAP B — an optional `subject` on
  `authz.check_scoped`/`scope_filter`, gated by `mcp:authz.delegate_reach:call`, so a
  granted extension resolves a NAMED subject's reach (`node-v0.4.0`). **cc-app wired it
  in:** pins bumped (`Makefile` `LB_TAG=node-v0.4.0`, `rust/Cargo.toml` + care crate),
  `Care::call_with_caller` projects the frame caller (the dead `LB_EXT_PRINCIPAL_JSON`
  seam retired), the chokepoint passes `subject = caller.sub` to the reach verbs, and the
  care install requests + is granted the delegation cap (`extension.toml` +
  `care_mount::approved_grant` + `live_node_support::approved_grant`, lock-step). Rule 7
  GREEN over a real spawned sidecar (`tests/live_node.rs`).
- **lb pack toolchain SHIPPED as `node-v0.3.3`** (2026-07-12, in `NubeDev/lb`, branch
  `pack-toolchain-publish`, commit `86f98c2d` / tag merge `a02c353`): `lb-devkit` +
  `lb-pack` drop `publish = false`. The artifact packager + the Ed25519 sign / key /
  trust-line idiom are now `cargo install --git …lb --tag <LB_TAG> lb-pack` —
  consumable by any embedder. **cc-app follow-on in `Makefile`** (the
  `cargo build -p lb-pack` workaround is gone; `LB_TAG = node-v0.3.3` drives a
  one-line re-pin when lb ships the next bump).
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
  **DERIVATION half** (minting scoped grants on `guardianship.link`/`unlink`
  via `grants.assign`/`revoke`) is **LIVE as of milestone 05** (the
  `node-v0.3.3` pin — routing fix in `node-v0.3.2`, pack toolchain in
  `node-v0.3.3`). Era-1 remains the documented fallback (`care-authz-scope.md`).
  The `tests/matrix_era2_write.rs::era2_write_grants_assign_over_callback_works`
  proves the era-2 WRITE half of `guardianship.link` / `unlink`
  round-trips over the callback; `era2_cross_family_deny_over_live_callback`
  + `era2_first_sign_in_deny_over_live_callback` assert the sacred invariants
  (CLAUDE.md rule 7). The era-2 chokepoint wires into `Care::boot`
  whenever the sidecar env ships `LB_EXT_TOKEN` + `LB_GATEWAY_URL`
  (the documented era-2 contract); the same binary boots era-1 when
  those vars are absent (the integration-test path).
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
- **Milestone 04 — mobile-shell CLOSED** (2026-07-12; **wire-in corrected
  2026-07-12**): `Care::boot(env)` reads the supervisor-injected `LB_EXT_*`
  env, builds the `SidecarClient`, and constructs the dispatcher.
  `tests/live_wire.rs` proves the constructor IN-PROCESS. **The REAL native
  wire-in** (cc-node spawning the sidecar + durable record I/O) was NOT part
  of m04 as first written — it is the fix landed 2026-07-12: `care_mount.rs`
  installs the sidecar via `install_native`, `authz/store.rs` routes all
  record I/O over the host `store.*` callback to the node's durable store, and
  `tests/live_node.rs` proves install + seed + admin-read + restart-durability
  on a REAL node — AND (as of the `node-v0.4.0`/`sdk-v0.4.0` pin) rule 7 in the
  sidecar is ENFORCED: a linked guardian reaches her child, a stranger is denied
  (403 on get, EMPTY on list). See the banner at the top + the CLOSED debugging
  entry. The **shell** (`ui/`) implements the
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
- **Milestone 05 — invites golden path CLOSED** (2026-07-12): the `care.invite.*`
  verbs live. `care.invite.create_guardian` mints a `invite.create` over the
  host-callback (the same `SidecarClient` the era-2 chokepoint reads from),
  persists the local mirror row, derives `lb_invite_id = hash(token)` locally
  (SHA-256, the same primitive lb uses), flips the mirror to `Sent`.
  `care.invite.create_staff` does the same for staff (slot + room). `care.invite.list`
  reads the local mirror (Pending + Revoked are the statuses the extension
  OWNS). `care.invite.revoke` flips the mirror to Revoked FIRST (admin list
  reflects the intent immediately) then calls `invite.revoke` over the
  callback (the lb durable kill). `care.invite.resend` calls lb
  `invite.resend` (which atomically rotates the token + the TTL + enqueues
  a fresh email effect — no born-expired links) and records the new
  `lb_invite_id`. The pre-auth accept surface (`ui/src/auth/InviteAcceptPage.tsx`)
  follows the guardian record's `locale` so a Spanish-speaking Ana gets a
  Spanish accept page (CLAUDE.md rule 8; en + es i18n parity verified by
  `node scripts/i18n-check.mjs` in both `ui/` and `rust/extensions/care/ui/`).
  All 5 invite verbs are in `Tools::TOOLS` (the dispatcher is the WHOLE
  contract). Era-1 fallback surfaces a typed `StoreDenied` when the chokepoint
  has no host-callback client (the integration-test path). Session doc:
  [`sessions/care/05-era2-write-and-invites-live.md`](sessions/care/05-era2-write-and-invites-live.md).

## Deferred (per the milestones, not yet started)

- **Milestone 00 — lb-release: DONE** (2026-07-12). Pinned `node-v0.3.0` /
  `sdk-v0.3.0`; dropped the `[patch]` block from the git-ignored
  `.cargo/config.toml`; `cargo build`/`test --workspace` clean FROM TAGS ALONE.
- **lb routing fix — DONE** (2026-07-12, shipped as `node-v0.3.2`).
- **lb pack toolchain publish — DONE** (2026-07-12, shipped as `node-v0.3.3`).
- **Milestone 03 — `care.enrollment.import`**: the lb/jobs CSV integration
  (deferred this session; records/verbs it lands into are all shipped).
  Accepts children+guardians+edges, per-item results, hard-fail on medical
  fields, idempotent on natural keys; 40-row fixture, 2 bad rows → 38 land.
- **Milestones 06 + 07 CLOSED (2026-07-13)** — attendance + menus (see the
  banner above). **08 + 09 + 10** remain: m08 (daily-feed) is NEXT UP (below).
- **Billing: build LAST** (product decision 2026-07-11). `scope/billing/billing-scope.md`
  stays only as the must-not-preclude ledger; no billing work before phase-1 ships.

## Local-dev posture (the WORKFLOW-LB.md §3 path)

The git-ignored `.cargo/config.toml` now carries ONLY:

- **zigcc linker wiring** (this box has no system C compiler).
- **ZIG cache redirect** to `/tmp/kilo/zig-cache` (sandbox quirk —
  `/home/user/.cache/zig` is read-only).
- `jobs = 4` (the RAM-heavy link step OOM-killed at 6 with the editor resident).

`.cargo/config.toml` CURRENTLY carries a local lb `[patch]` block (WORKFLOW-LB.md
§3) redirecting `lb-node`/`lb-host`/`lb-supervisor`/`lb-store`/`lb-auth`/`lb-role-gateway`
+ `lb-ext-native` to the sibling checkouts (`~/code/rust/lb`, `~/code/rust/lb-ext-sdk`).
Those siblings carry LOCALLY-tagged `node-v0.4.1`/`node-v0.4.2`/`sdk-v0.4.1` on branch
`native-caller-admin-marker`, NOT pushed. The declared pins in `rust/Cargo.toml` +
`rust/extensions/care/Cargo.toml` + `Makefile LB_TAG` say `node-v0.4.2`, but the `[patch]`
is what actually resolves. **Release ritual to finish:** push the branch + `*-v0.4.x` tags,
then drop the `[patch]` — a clean `cargo build --workspace` with NO `[patch]` is the "am I on
releases?" check (WORKFLOW-LB.md §4). (Milestone 00 originally dropped the patch at
`node-v0.3.0`; the `*-v0.4.x` local-tag posture was reintroduced for the admin-marker +
credential-mode work and is intentional until those tags are pushed.)

## ✅ BUILD BLOCK RESOLVED (2026-07-13) — builds clean from published tags

The `*-v0.4.1`/`v0.4.2` tags **are now pushed** to the remotes (`git ls-remote`
confirms `node-v0.4.0/0.4.1/0.4.2` on `NubeDev/lb` and `sdk-v0.4.0/0.4.1` on
`NubeDev/lb-ext-sdk`). The real remaining blocker was the **missed SDK pin**:
`rust/Cargo.toml` still declared `lb-ext-native = sdk-v0.4.0`, which lacks the
`Caller.admin` field that shipped `principal_from_caller` reads (`lib.rs:200`).
**Fixed by the one-line pin bump `sdk-v0.4.0 → sdk-v0.4.1`** (care ext inherits
via `workspace = true`). `cargo build --workspace` + `cargo test -p care` are
GREEN from tags with **NO `.cargo/config.toml` and NO `[patch]`** — a clean
"am I on releases?" pass (WORKFLOW-LB.md §4). (This box has a system C compiler,
so the zigcc wiring the earlier local-dev posture described is not needed here.)
Full history: [`debugging/build/unbuildable-from-releases-unpushed-v0.4.1-tags.md`](debugging/build/unbuildable-from-releases-unpushed-v0.4.1-tags.md).

## Milestone 08 (daily-feed) — STARTED (foundation only, un-compiled)

The orchestrator-owned foundation is written, **COMPILED, and fence-clean**
(authz + hardcoded + file-size gates green; `cargo test -p care` = 174 lib +
all matrix suites green from published tags). The foundation was split to
satisfy the ≤400-line FILE-LAYOUT limit (`records.rs` was 504): now
`log/records.rs` (the core `DailyLog` record + `LogKind` + `feed_subject` +
`PushPolicy`, 278L), `log/payload.rs` (the 4 type-specific payload structs,
71L), `log/validate.rs` (`LogError` + `LogKind::validate` + `validate_timestamp`
+ their tests, 207L):
- `log/records.rs` — the `daily_log` schema (8 types, type-specific payloads:
  nap start/end, meal slot+portion, incident required what/where/action + ack,
  medication dose+witness), `LogKind::validate` (regulated-field enforcement),
  `PushPolicy` (incident/medication = Always, else FeedThenPrefs), the
  `feed_subject(child)` = `care.feed.<child>` bus contract, `validate_timestamp`.
- `authz::feed_recipients` (in `authz/scope.rs` behind the grep fence) — the
  LIVE-`receives_daily_feed`-edge holder set for a child (the emit + push
  recipient resolution; a `false`/unlinked edge gets neither).
- `push/mod.rs` — the pure type→notification `decide()` policy + the
  `notify.send` catalog-key/deep-link shape (localization is lb's job per
  recipient via `title_key`/`body_key` — the both-languages exit gate).

**NEXT (once the build is unblocked):** the verbs (`log.add` multi-child atomic
fan-out + bus emit + push dispatch; `log.list` cursor-paged role-filtered;
`log.correct`; `log.day` rollup; `feed.watch` SSE), the media photo path
(begin/commit + reach-checked serve + consent-at-write), the attendance→feed bus
emit (m06 deferral), dispatcher/TOOLS/caps wiring, the en+es catalog keys, the
guardian Feed + staff two-tap UI, and the matrix rows (incl. the media-URL leak).
The wire seams are confirmed present in lb: `bus.publish`, `notify.send`,
`media.upload_begin/commit/get` — all reachable over the same `SidecarClient`.

## Next up

**Milestone 08 — daily-feed** (care ext + `ui/`): the family-facing feed a
guardian sees (photo + note + allergy check + which guardians receive it — the
per-edge `receives_daily_feed` flag). It DEPENDS on m06 + m07 (both now closed):
the feed composes from `menu.*` × `child.allergies` (m07 derivation) and
consumes the attendance bus event (m06's deferred emit lands HERE — agree the
subject shape and wire the ledger append to it). The chokepoint's
`reachable_children` anchors the guardian read; SSE + push are the motion half.
Then m09 (messaging), m10 (hardening / PWA install path). All chained off the
chokepoint.

**One deferral carried out of m06:** the attendance→feed bus event emit (no feed
consumer existed at m06). Do it first thing in m08.

## Non-goals (unchanged)

- No special-casing of lb or any extension (rule 10).
- No vendored lb UI shell — 100% of the product UI is extension UI behind `defineRemote`.
- No billing/payments in phase 1 (scope §Phases).
