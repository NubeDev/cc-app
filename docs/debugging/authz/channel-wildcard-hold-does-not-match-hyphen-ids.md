# m09 bug — the channel wildcard hold `bus:chan/care-**:sub` never matches a `care-child-<id>` channel

**Status:** OPEN (found 2026-07-13 during the m10 edge-change drill). cc-app-owned
(NOT an lb gap — lb's cap grammar is correct). Fix lands in cc-app messaging.

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
