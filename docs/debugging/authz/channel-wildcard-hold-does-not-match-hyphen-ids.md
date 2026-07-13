# m09 bug — the channel wildcard hold `bus:chan/care-**:sub` never matches a `care-child-<id>` channel

**Status:** RESOLVED 2026-07-13 (found + fixed during the m10 edge-change drill).
cc-app-owned (NOT an lb gap — lb's cap grammar is correct). See **Resolution** below.

## Symptom

On a REAL node, `care.guardianship.link` with `receives_messaging: true` returns
403:

```
guardianship.link: "extension error: child returned an error: channel membership
grant failed (retry via reconcile): host denied the call (capability/workspace gate)"
```

The store-only m09 matrix suite (`matrix_messaging.rs`) is GREEN because it exercises
the derivation directly with no live `grants.assign` callback — the wildcard-hold
match is never evaluated there. The live drill is the first place the real
`grants.assign` no-widening check runs against the channel cap.

## Root cause — the cap grammar splits on `/` and `.`, NOT `-`

lb's `grants_assign` no-widening rule (`host/src/authz/grants.rs:38`) requires the
GRANTER to HOLD a cap matching what it grants (`lb_caps::grammar::matches`). The
matcher (`caps/src/grammar.rs:76`) splits both the pattern and the resource on
`['/', '.']` — **`-` is NOT a separator**. So:

- held pattern `bus:chan/care-**:sub` → resource segments `["care-**"]` (ONE literal
  segment; the `**` is embedded in a literal, not its own segment).
- minted cap `bus:chan/care-child-leo:sub` → resource segments `["care-child-leo"]`.
- `"care-**"` is a literal segment (not the terminal `**` wildcard), so it matches
  ONLY the literal string `care-**`, never `care-child-leo`. ⇒ `holds_cap` = false ⇒
  `AuthzError::Widen` ⇒ 403.

The channel-id convention (`channel_id.rs`) joins with `-`
(`["care-child-", child_id].concat()`), so the whole id is one grammar segment and
`**` can't reach into it.

## Why the m10 feed-watch cap is NOT affected

`feed::watch_grant` builds `bus:care.feed.<child>:watch` and holds
`bus:care.feed.**:watch`. `feed_subject` joins with `.`
(`["care.feed", child].join(".")`), so the resource splits to
`["care", "feed", "child:leo"]` and the held pattern to `["care", "feed", "**"]` —
the terminal `**` matches the `child:leo` tail. The live edge-change drill PROVES
this works (link grants the feed-watch cap, 200; unlink revokes it). The bug is
specific to the `-`-joined channel ids.

## Fix options (cc-app messaging, phase-1)

1. **Change the channel-id separator** so the entity id sits in its own grammar
   segment: `care/child/<id>` or `care.child.<id>`, held as `bus:chan/care/**:sub`
   / `bus:chan/care.**:sub`. Cleanest — the `**` tail then covers every care
   channel. Touches `channel_id.rs` (all 3 builders + the pub/sub cap builders) and
   the three wildcard-hold sites (`extension.toml`, `care_mount::approved_grant`,
   `live_node_support`). No lb change.
2. Hold the exact per-channel caps — rejected (unbounded, one per channel).

Option 1 is the fix. Tracked as an m10 hardening item alongside the matrix-sweep
completeness gate.

## Resolution — `.`-separated channel ids (2026-07-13)

Applied **option 1**. `messaging/channel_id.rs` now joins with `.`:
`care.child.<id>` / `care.room.<id>` / `care.center.<id>`, and the three
wildcard-hold sites hold `bus:chan/care.**:{pub,sub}` (was `care-**`). The cap
grammar splits the resource on `.`, so `care.child.leo` → segments
`[chan, care, child, leo]` and the held `bus:chan/care.**:sub` terminal `**`
matches the `[child, leo]` tail. `CARE_CHANNEL_PREFIX` is now `care.`.

Lock-step sites updated: `channel_id.rs` (3 id builders), `extension.toml`,
`care_mount::approved_grant`, `live_node_support::approved_grant`, plus the
comment/test literals in `reconcile.rs`, `reconcile_verb.rs`, `announce.rs`,
`authz/scope.rs`.

**Proven on the live node:** `matrix_edge_change.rs` now links Ana↔Leo with
`receives_messaging: true` and gets **200** (the channel-membership grant lands
over the callback; it 403'd before this fix), and `unlink` revokes it (200) —
both grant + revoke of `bus:chan/care.child.leo:{pub,sub}` succeed under the
no-widening rule. `cargo test -p care` (store-path derivation) + the 4 fences +
`pnpm build` remain green.

### Follow-on 1 — FIXED: four verbs had no `[[tools]]` manifest block (routing)

The drill also surfaced that `care.channel.reconcile` routed as **"no such tool"**
over `POST /mcp/call` on the live node. Root cause: four verbs
(`channel.reconcile`, `announce.post`, `media.begin`, `media.commit`) were in
`Tools::TOOLS` and the install grant but had **no `[[tools]]` block** in
`extension.toml`, so the host never registered them for routing (the `tools()`
handshake did not reject the mismatch — it is not the guard the manifest comment
claimed). **Fixed:** added the four `[[tools]]` blocks + added `media.begin` /
`media.commit` to both `approved_grant` sites (they were missing entirely). The
drill now proves `care.channel.reconcile` ROUTES (returns a tool-level result, not
"no such tool"). This was a real reachability bug for all four verbs on a live node.

### Follow-on 2 — ROOT-CAUSED → lb gap (NubeDev/lb#52), not a cc-app bug

With routing fixed, `care.channel.reconcile` reaches lb `channel.create`, which
fails. **Root cause (dispatch-traced):** `channel.create` is NOT wired into lb's
MCP dispatch. `call_channel_tool` (`lb host/src/channel/tool.rs:27`) has arms only
for `post | history | edit | delete | list`; `"channel.create"` falls through to
`_ => Err(ToolError::NotFound)` (`tool.rs:91`). The outer cap gate
(`mcp:channel.create:call`, which care holds) PASSES; the call dies at NotFound
BEFORE any `authorize_channel(..., Pub)` bus-cap check runs. So the `.`-separator
fix + the wildcard hold are both correct and never the deciding factor here — the
channel-registry helper `channel_create` exists only in-process
(`channel_registry/create.rs:17`, re-exported `lib.rs:156`) and was never given an
MCP arm.

`grants.assign` over the SAME callback + principal works (it routes to the
fully-implemented `call_authz_tool`), which is how we isolated the gap to the
missing channel dispatch arm — not caps, principal, or workspace.

**Owned by lb** (WORKFLOW-LB — fix lb generically): filed as **NubeDev/lb#52** with
scope + the one-arm additive fix (add a `channel.create` arm calling the existing
`channel_create`; it reuses the channel `pub` gate, no new cap, no ABI/SDK change).
Until it ships, guardian↔staff channel PROVISIONING on a live node is blocked; the
leak-critical paths (membership grant/revoke on link/unlink, the derivation) all
work, and the m10 edge-change drill passes. A cc-app create-on-first-post workaround
is possible but rejected (rule 10) — no working around a core gap. **Launch-checklist
carry:** bump the lb pin once #52 ships, then the drill can assert full channel
provision + read + post + unlink-revoke.
