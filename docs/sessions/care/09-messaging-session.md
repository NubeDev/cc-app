# Milestone 09 — messaging (provisioning + membership reconciliation) — session

Branch: `build/m08-daily-feed-verbs` (continued) → will land on a m09 branch.

## Entry gate RESOLVED (2026-07-13) — NO lb core work needed

The day-one gating question ("do lb channels support read-only membership?") is
**YES, via split caps** — no additive lb ask:

- lb channels: **post** needs `bus:chan/{cid}:pub`; **read/history/subscribe** need
  `bus:chan/{cid}:sub` (distinct actions/resources — lb `channels-scope.md`
  §"Security invariants"). Read-only = grant `sub` without `pub`.
- Fully expressible with the existing scoped-grant grammar care already uses
  (`grants.assign` over the `SidecarClient`, same path as `store:media/{id}:read`).
- Wildcard-hold: care install adds `bus:chan/care-**:pub` + `bus:chan/care-**:sub`
  to `care_mount::approved_grant` + `extension.toml` + `live_node_support`
  (lock-step; the `store:media/**:read` idiom). Care-side declaration, not lb work.

MCP surface confirmed reachable over the callback: `channel.create`, `channel.post`,
`channel.history`, `channel.list` (host `tool_call.rs` `channel.` arm) + the existing
`grants.assign`/`grants.revoke`.

## Open questions RESOLVED

- **Distinct `messaging` flag** — YES. Added `EdgeFlags::receives_messaging` (a 6th
  flag; custody differs on exactly this vs `receives_daily_feed`). Foundation done.
- Announcements: one per center (`care-center-<id>`), read-only for guardians.
- Removed guardian's authored history STAYS (center record; privacy stance recorded).

## Foundation (orchestrator-owned) — DONE this session

- `guardianship/records.rs` — `receives_messaging` flag added (+ default test).
- `messaging/channel_id.rs` — the channel-id conventions (`care-child/room/center-<id>`)
  + `pub_cap`/`sub_cap` + `ChannelRole::{Full,ReadOnly}::caps` (read-only = sub w/o pub).

## Built this session — m09 CLOSED

1. **Reconciler** ✅ — `authz::channel_members` (`resolve_child/room_channel_members`
   in `authz/scope.rs`, behind the fence) + `messaging/reconcile.rs` (grant/revoke via
   `grants.assign/revoke`, no-client no-op, `reconcile_channel` healing sweep).
2. **Verbs** ✅ — `care.channel.reconcile` (provision `channel.create` + heal),
   `care.announce.post` (read-only-for-guardians enforced twice).
3. **Handler hooks** ✅ — `guardianship.link/unlink/update` grant/revoke channel
   membership in the same breath as the edge.
4. **UI** ✅ (subagent) — `MessagesPage` + `AnnouncementsCompose`, Messages tab wired.
5. **Matrix** ✅ — `matrix_messaging.rs` (derivation-level cross-family sweep).
6. **Wiring** ✅ — call.rs / care_mount / extension.toml / live_node_support lock-step
   (channel.* verbs + `bus:chan/care-**:{pub,sub}` holds; + fixed missing
   `store:media/**:read` hold from m08).

## Deferred to m10 (honest follow-ons)

- **Archive → stop-posts**: provisioning is on-demand (`channel.reconcile`), not eager
  on entity-create/archive — the archive hook lands with entity-archive wiring.
- **Staff room-move reconcile**: no standalone staff-reassignment verb exists to hook
  (assignments minted via invite-accept) — lands when that verb exists. Room-channel
  STAFF derivation is built + tested.
- **ext-UI channel SSE**: the ext-ui-sdk runtime exposes no gateway origin/token, so
  the UI polls `channel.history` for liveness (the m08 FeedPage pattern). TODO(sse)
  in `api/channels.ts` for when the runtime exposes them.
