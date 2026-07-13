# lb gap — `bus.watch` is workspace-wide + no mid-stream termination on revoke

**Status:** OPEN lb gap (record; do NOT work around in cc-app). Found 2026-07-13
while building m08 `care.feed.watch`. Verified against lb source at
`node-v0.4.2` (`/home/user/code/rust/lb/rust/...`).

## What we needed

`care.feed.watch` is the guardian's live SSE feed over the per-child bus subject
`care.feed.<child>` (daily-feed-scope §"Intent": "filtered at emit"). Two exit-gate
requirements from `docs/build/08-daily-feed.md`:

1. **Per-child isolation on the stream** — a guardian may only watch a child they
   hold a live `receives_daily_feed`/reach edge to (rule 7 on the motion half).
2. **Unlink mid-stream terminates the OPEN stream**, not just future subscribes.

## What lb actually provides (both are gaps)

### Gap 1 — the generic `bus.watch` surface is NOT subject-scoped

Authorizing a subscribe checks exactly TWO things (lb `bus/authorize.rs` →
`caps/check.rs`): the workspace matches, and the caller holds the workspace-wide
cap `mcp:bus.watch:call`. **The subject string never reaches the cap check** — it
is only namespace-walled (`wall_subject`: reserved-prefix + cross-ws guard, then
`ext/{subject}`). So within a workspace, ANY holder of `mcp:bus.watch:call` can
watch ANY `ext/*` subject, including another family's `care.feed.<child>`.

A per-subject cap grammar (`bus:chan/*:sub`) exists but ONLY for the separate
`channel` service — the generic `bus.watch` deliberately maps its row to the
unscoped `mcp:bus.watch:call` (lb `bus/subject.rs`). The entity-scoped-grants
scope selector is `{table, ids}` — it scopes record/list/get verbs, NOT bus
subjects. lb's own answer for watch verbs is **"filter-at-emit in the extension
for v1 (no scoped subscription helper)"** (lb
`entity-scoped-grants-scope.md` §Watch verbs).

### Gap 2 — no mid-stream termination on `grants.revoke`

The subscribe gate runs EXACTLY ONCE, before the stream opens (`bus/watch.rs`).
The SSE route (`gateway/routes/bus.rs`, `session/events/hub.rs`) is a plain
`unfold`/`while stream.next()` loop that exits only when the payload stream closes
or the client disconnects. No grant re-check, no revocation signal, no heartbeat
re-auth. Revoking a scoped grant blocks the NEXT subscribe/call — it does NOT
close a currently-open SSE stream. lb's freshness guarantee ("revoke → immediate
deny") is documented to apply to `check_scoped`/`scope_filter` at call/list time,
not to open subscriptions.

## cc-app decision for m08 (documented, not a workaround)

`care.feed.watch` v1 does the ONE thing the extension CAN enforce correctly: a
**reach-check-at-subscribe** (`authz::assert_reach` on the child) before it hands
the guardian UI the authorized subject + stream descriptor. This is the same
chokepoint decision every read verb uses, so a stranger's `feed.watch` is denied
(matrix row asserts it).

What v1 does NOT get, and why it is acceptable short-term:
- **Full stream-level isolation** against a same-workspace member who forges a
  raw `GET /bus/stream?subject=care.feed.<otherchild>`: NOT enforceable at the
  bus today (Gap 1). Mitigation v1: the guardian UI only ever opens subjects the
  reach-checked `feed.watch` returned; the record-level reads (`log.list`/`day`)
  and media serve ARE reach-checked (the durable leak surfaces are closed). The
  live bus is a best-effort convenience channel over already-authorized data.
- **Mid-stream termination on unlink** (Gap 2): NOT possible today. An unlinked
  guardian's open stream survives until they disconnect; their NEXT subscribe is
  denied, and every durable read is denied immediately. Recorded as the residual
  risk.

## The genuine fix is lb-owned (WORKFLOW-LB.md — fix lb generically first)

Two additive lb pieces, in priority order:
1. **Subject-scoped bus grants** — let an extension mint a `bus:<subject>:watch`
   scoped grant on `guardianship.link` (mirroring the reach grant) that the
   `bus.watch` gate honors, so per-child subscribe authz is platform-enforced.
2. **Revoke-terminates-stream** — a subscription invalidation signal so
   `grants.revoke` closes matching open streams (a re-check tick or a
   revoke→close push in `hub.rs`).

Until those ship, cc-app stays on reach-check-at-subscribe + the note above. See
lb `entity-scoped-grants-scope.md` §Watch verbs and the m08 exit-gate rows.
