# Milestone 10 — hardening & phase-1 launch — session

Date: 2026-07-13. Branch: master (small, gated commits). The final phase-1 milestone:
prove the product holds together under the rules, land the accumulated deferrals, and
promote the docs. Only fixes ship here — with one exception forced by a bug found
mid-drill (below).

## Entry gate

Master green at session start (verified): `cargo fmt --check`, `cargo test -p care`
(213 lib + all matrix), all 4 fences, care-UI `tsc` + i18n + `pnpm build`. Milestones
00–09 closed.

## What shipped

### 1. lb pin `node-v0.4.2 → node-v0.4.3` + `[patch]` retired

`node-v0.4.3` (NubeDev/lb#49) ships subject-scoped `bus:<subject>:watch` grants
(present ⇒ required, absent ⇒ open — back-compat) + revoke-terminates-stream (a 3s
`WatchRecheck` tick). Additive, no SDK change (`sdk-v0.4.1` stays). Bumped all sites
(`rust/Cargo.toml`, `care/Cargo.toml`, `Makefile LB_TAG`) and dropped the local
`[patch]` block from `.cargo/config.toml` — `cargo build --workspace` clean from the
pushed tags (the WORKFLOW-LB §4 "am I on releases?" check). Lockfile resolves
`node-v0.4.3` + `sdk-v0.4.1`.

### 2. `feed.watch` → full platform stream isolation

`feed::watch_grant` mints the narrow `bus:care.feed.<child>:watch` for each daily-feed
guardian on `guardianship.link` (iff `receives_daily_feed`), revokes it on `unlink`
(which per lb#49 Gap-2 ALSO terminates the open SSE stream within a tick), and
re-derives on a `receives_daily_feed`/`live` flip in `update`. The media
serve-grant / channel-membership idiom. Sidecar HOLDS `bus:care.feed.**:watch`
(lock-step across `extension.toml` + `care_mount::approved_grant` +
`live_node_support`). `feed.watch`'s reach-check is now defence-in-depth atop the
platform gate. Debug doc `bus-watch-unscoped-and-no-midstream-revoke.md` → RESOLVED.

### 3. The edge-change drill — the m10 existential-bug exit gate

`matrix_edge_change.rs` (live spawned sidecar, `--ignored`): seed Ana LINKED to Leo
(feed + messaging), assert full access, then `guardianship.unlink` and assert EVERY
surface collapses — `child.get` 403, `feed.watch` 403 (feed-watch grant revoked →
WatchRecheck terminates the stream), `log.list` empty, reach + feed-watch grants
revoked over the live callback. **PASSES.** Channel collapse covered by
`matrix_messaging`'s unlink row + the drill's messaging link/unlink.

### 4. Full matrix-sweep completeness gate

`matrix_completeness.rs` enumerates `care::call::TOOLS` (all 44 served verbs) and
fails if any verb lacks a declared cross-family coverage (GuardianRead ⇒ a reach deny
row; AdminWrite ⇒ a capability deny), naming the covering test. A second test pins the
guardian-read (rule-7 leak) census so the sacred set can never shrink silently. A new
verb without coverage fails CI.

### 5. archive → stop-posts (m09 deferral, closed)

An archived child's channel is frozen: `resolve_child_channel_members` returns NO
members for an archived (or missing/faulted) child, and `child.archive` captures the
live members before flipping the flag then revokes each over the callback. Unit-tested
(member before archive, gone after).

## Two real bugs found via the drill (both cc-app-owned, both fixed)

1. **Channel wildcard never matched.** `bus:chan/care-**:sub` could not authorize a
   `care-child-<id>` channel grant — lb's cap grammar splits on `/` and `.` but NOT
   `-`, so `care-**` was one literal segment. Every live channel grant 403'd. **Fixed:**
   switched channel ids to `.`-separators (`care.child.<id>` etc.), held as
   `bus:chan/care.**` — the terminal `**` now covers the entity-id tail (mirrors the
   `.`-joined `feed_subject`, already correct). Proven: the drill now links with
   `receives_messaging=true` (grant lands, 200 — 403'd before) and unlinks (revoke).
2. **Four verbs unrouted.** `channel.reconcile`, `announce.post`, `media.begin`,
   `media.commit` were in `Tools::TOOLS` + the grant but had NO `[[tools]]` manifest
   block, so the host never registered them → "no such tool" on a live node. **Fixed:**
   added the blocks + added `media.begin`/`commit` to both `approved_grant` sites.

## One lb gap found + filed (fix lb generically — WORKFLOW-LB)

**NubeDev/lb#52 — `channel.create` is not wired into lb's MCP dispatch.**
`call_channel_tool` has no `create` arm → `NotFound` before any cap check
(`channel_create` exists only in-process). Root-caused by dispatch trace (a subagent),
confirmed against lb source. Filed with scope + the additive one-arm fix (reuses the
channel `pub` gate, no new cap/ABI/SDK). Until it ships, live-node channel
PROVISIONING is blocked; the leak-critical membership grant/revoke + the drill are
unaffected. Debug doc:
`docs/debugging/authz/channel-wildcard-hold-does-not-match-hyphen-ids.md`.

## Doc promotion

`doc-site/content/public/care/care.md` and `ui/ui.md` filled with the shipped phase-1
product (personas, features, guardian isolation, en/es, shadcn/modern-iOS) per each
scope's "Promotes to" header. Billing stays a phase-2 placeholder.

## Green (pasted output)

- `cargo fmt --all --check` — clean.
- 4 fences — authz / hardcoded / file-size (200 files ≤400) / i18n-parity — all OK.
- `cargo test -p care` — 17 test binaries all `ok` (215 lib + all matrix, incl.
  `matrix_completeness`).
- `--ignored` live: `live_node` (1 ok) + `matrix_edge_change` (1 ok — the drill).
- `cargo build --workspace` — clean from tags (no `[patch]`).
- care UI — `tsc --noEmit` clean, i18n gate OK, `pnpm build` OK.

## Carried to phase-2 / launch checklist (need a device or a human, or new scope)

- **lb#52** — `channel.create` MCP arm → live-node channel provisioning (bump pin when
  shipped).
- **Staff room-move reconcile** — needs a new `staff.reassign` verb; NEW scope, not a
  fix, so out of m10 (assignments are minted only via invite-accept today).
- Persona acceptance pass on a real 360px phone + 1280px laptop in both themes.
- Human Spanish-speaker `es`-catalog review.
- Photo-heavy-feed perf pass; cold-PWA / flaky-network pass.
- ext-UI channel SSE (needs ext-ui-sdk to expose a gateway origin/token; polls
  `channel.history` today).
- Master-scope open questions (naming/branding, PWA-vs-RN, staff-tablet offline,
  billing provider) — carry, don't block.
