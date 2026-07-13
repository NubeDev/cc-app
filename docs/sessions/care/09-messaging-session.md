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

## Remaining (build order)

1. **Reconciler** (leak vector — careful, direct): `authz/messaging.rs` behind the
   fence — `channel_members(cp, channel)` derives the (subject, ChannelRole) set from
   `guardianship` (live + `receives_messaging`) + `staff_assignment` (room staff). Then
   `messaging/reconcile.rs` = grant/revoke the delta via `grants.assign`/`revoke`.
   Called from every edge/assignment handler + an idempotent sweep verb.
2. **Provisioning**: child/room/center create → `channel.create` (idempotent); archive
   → archive channel (history retained). Hook into existing create/archive verbs.
3. **Handler hooks**: `guardianship.link/unlink/update`, staff assignment create/remove,
   room-move → call reconcile in the same handler (same transaction discipline as grants).
4. **Posting policy**: falls out of the grant (Full=pub+sub, ReadOnly=sub). Announcements
   compose verb for admin/staff.
5. **UI**: guardian Messages tab, staff room/child threads, admin announcements compose
   (lb channel widgets where possible).
6. **Matrix**: `matrix_messaging.rs` — post as Ana in Leo's channel → Mia's-mum never
   sees it; Ana has no path to Mia's channel; unlink → next read denied; reconciler
   idempotent under double events; sweep repairs; archive retains history stops posts;
   guardian announcement post denied.
7. Caps/TOOLS/`approved_grant`/`extension.toml` wiring (lock-step) + en/es catalog keys.
