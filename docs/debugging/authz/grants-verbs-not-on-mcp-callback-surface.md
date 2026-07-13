# `grants.*` verbs are not reachable over the native host-callback

- **Area:** authz / era-2 scoped-grant derivation
- **Date:** 2026-07-12 (milestone 03, Step C — opened)
- **Date:** 2026-07-12 (milestone 04 — patch authored, awaiting upstream PR)
- **Date:** 2026-07-12 (milestone 04 — verified STILL PRESENT in `node-v0.3.1`)
- **Date:** 2026-07-12 (milestone 05 prep — newest published tag is STILL
  `node-v0.3.1`; the routing fix has NOT shipped. Per HARD RULE 2 of
  the m05 prep session, did the non-blocked m05 work: scaffolded the
  `care.invite.*` verb files WITHOUT wiring the SidecarClient
  `invite.create` / `invite.revoke` calls. The verbs validate input +
  persist a local mirror row, then return `InviteError::NotImplemented`
  pointing back at THIS debug entry.)
- **Date:** 2026-07-12 — **CLOSED.** `node-v0.3.2` shipped the routing
  fix (`rust/crates/host/src/tool_call.rs` gains the
  `grants.` / `roles.` / `teams.` arm into `call_authz_tool`); cc-app
  pin bumped to `node-v0.3.3` (which carries the pack-toolchain side of
  the same release — see `docs/debugging/build/make-dev-lb-pack-not-found.md`).
  Era-2 WRITE is live: `authz::grant::{derive_reach, remove_reach}` go
  through `cp.reach().client().call_tool("grants.assign" | "grants.revoke", …)`,
  the chokepoint delegates reach resolution over HTTP, and
  `tests/matrix_era2_write.rs::era2_write_grants_assign_over_callback_works`
  runs (un-`#[ignore]`d) green against a real booted gateway. Plus:
  `tests/matrix_era2_write.rs::era2_cross_family_deny_over_live_callback`
  + `tests/matrix_era2_write.rs::era2_first_sign_in_deny_over_live_callback`
  assert the cross-family deny + the invite→accept→first-read boundary
  via the live callback (CLAUDE.md rule 7, sacred). All live, no in-
  process fallback.

> **Status:** CLOSED 2026-07-12.

## Symptom (history — kept for the record)

Wiring era-2 (Step C), the care chokepoint's grant-derivation path
(`care.guardianship.link` → `authz::grant::derive_reach` →
`SidecarClient::call_tool("grants.assign", …)`) returned `CallError::Denied`
(HTTP 403) against a **real** booted gateway, even with a token that holds
`mcp:grants.assign:call` (and `mcp:care.reach.child:call` for anti-widen).

The READ half of the same callback worked fine: `authz.scope_filter` and
`authz.check_scoped` round-tripped over the identical `SidecarClient` and
returned correct results (proven in `tests/matrix_era2.rs`).

## Root cause (found by reading lb `node-v0.3.0`, read-only)

lb's MCP dispatcher `call_tool_at_depth`
(`rust/crates/host/src/tool_call.rs`) routed tool families to their
handlers by prefix. It routed **`authz.*`** to `call_authz_tool`:

```rust
} else if qualified_tool.starts_with("authz.") {
    crate::call_authz_tool(&node.store, principal, ws, qualified_tool, &input).await?
}
```

…but there was no arm for `grants.*` / `roles.* / teams.*`. Those verbs
were *implemented* in `call_authz_tool` (it matched `"grants.assign"`,
`"grants.revoke"`, …) but the dispatcher never routed a `grants.*` call
to it, so a `grants.assign` sent through `POST /mcp/call` fell through
to the generic extension-registry path and was denied (no such registered
tool).

Consequence: `grants.*` was reachable only via the dedicated REST routes
(`POST /admin/grants`, `/admin/grants/revoke`) that the admin console
uses — NOT via the generic `/mcp/call` bridge a native (Tier-2) sidecar
reaches the host through. So a native extension could *read* the
scoped-grant surface (`authz.check_scoped` / `authz.scope_filter`) but
could not *mint* a scoped grant over the callback.

## The fix (shipped as `node-v0.3.2`, additive one-arm)

Route `grants.*` (and `roles.*` / `teams.*`) through the MCP dispatcher
the same way `authz.*` is routed — one added arm in `call_tool_at_depth`:

```rust
} else if qualified_tool.starts_with("grants.")
    || qualified_tool.starts_with("roles.")
    || qualified_tool.starts_with("teams.") {
    crate::call_authz_tool(&node.store, principal, ws, qualified_tool, &input).await?
}
```

`call_authz_tool` already handles every one of those verbs, gated by the
existing admin caps (`mcp:grants.assign:call`, …) — so this is additive,
no new verb, no grammar change, no WIT bump. Tagged `node-v0.3.2`; pinned
in cc-app via `rust/Cargo.toml` + `rust/extensions/care/Cargo.toml`.

### Cap-alias nuance (care only uses `grants.assign` / `grants.revoke`)

Over `/mcp/call` the outer gate for `grants.revoke` rides
`mcp:grants.assign:call` (assign + revoke share the cap, per lb's gate
table). Care only ever mints with `grants.assign` and revokes with
`grants.revoke`, so a token holding `mcp:grants.assign:call` + the reach
cap (`mcp:care.reach.child:call`, needed for `derive_reach`'s scope's
anti-widen) is the correct pair.

## Live posture in cc-app (after `node-v0.3.2`)

- **Era-2 READ delegation is LIVE** — the chokepoint delegates
  `assert_reach` / `reachable_children` to `authz.check_scoped` /
  `authz.scope_filter` when a `ReachClient` is present. The seed path in
  `tests/matrix_era2.rs` (and the cross-family deny / workspace isolation
  rows) goes through `SidecarClient::call_tool("grants.assign" | "grants.revoke", …)`
  — no in-process `lb_host::grants_assign` fallback. The assertion bodies
  are identical to before; only the seed path moved.
- **Era-2 WRITE derivation is LIVE** — `authz::grant::{derive_reach, remove_reach}`
  are the live call sites for `guardianship.link` / `unlink` (and
  `update` when it re-affirms an edge). Edge-and-grant stay all-or-
  nothing on a derivation failure (the chokepoint rolls the edge back so
  the store never holds a live edge without its grant).
- **Era-1 (store-resolved edges) remains the documented fallback** —
  `care-authz-scope.md` §"Era 2": "keep the era-1 resolution as the
  documented fallback path … if lb's verbs aren't reachable." When
  `Care::boot(env)` lacks the `LB_EXT_TOKEN` + `LB_GATEWAY_URL` env the
  chokepoint falls back to era-1 (so a hand-rolled integration test, or
  a node that hasn't yet enabled native extensions, still boots). Same
  binary, both postures.
- **m05 invites (`care.invite.*`) wired** — `create_guardian` /
  `create_staff` / `revoke` / `resend` each mint / revoke / resend over
  the host-callback `SidecarClient::call_tool("invite.create" | "invite.revoke" | "invite.resend")`,
  hashing the raw token locally (SHA-256, the same primitive lb uses)
  to derive `lb_invite_id` = `token_hash` (the lb-internal id the
  inverse verbs look up by). `list` reads the local mirror. Verb files
  added to `Tools::TOOLS` (the dispatcher is the WHOLE contract;
  m04-style). See session doc for the wire shapes + the mirror-row
  bookkeeping.

## Patch (upstream lb — shipped, archived for the record)

The additive one-arm fix lives at
`docs/debugging/authz/lb-grants-routing.patch` (historical artifact).
Apply it on top of `node-v0.3.1` and you'd get the exact commit that
shipped as `node-v0.3.2` (commit `0304acd`); cc-app bumped the pin in
the milestone-05 session and the gate went green.

## Regression test (live, over the callback)

`tests/matrix_era2_write.rs` is the live-WRITE-half regression: it
proves era-2 mint/revoke over the callback against a real booted
gateway. Three rows:

1. `era2_write_grants_assign_over_callback_works` — mints a sam → leo
   scoped reach grant over `SidecarClient::call_tool("grants.assign", …)`,
   reads it back via `authz.scope_filter`, then revokes via
   `SidecarClient::call_tool("grants.revoke", …)` and asserts the read
   returns empty (the grant is physically gone, not merely that a read
   denied — a grant surviving unlink is the existential cross-family
   leak).
2. `era2_cross_family_deny_over_live_callback` — sam linked to leo,
   queried with an era-2 chokepoint against mia (a child in ANOTHER
   family); `scope_filter` returns `[child:leo]`, `assert_reach`
   denies `child:mia`. **CLAUDE.md rule 7 is sacred — the chokepoint
   exists to prevent this leak.**
3. `era2_first_sign_in_deny_over_live_callback` — a freshly-bound
   guardian (no grants yet — the "first sign-in" posture), queried with
   the era-2 chokepoint against every child in the workspace; deny on
   every one (the invite → accept → first-read boundary).

`tests/matrix_era2.rs` stayed the live READ-path regression, with its
seed swapped from the in-process `lb_host::grants_assign` to
`SidecarClient::call_tool("grants.assign" | "grants.revoke", …)` —
assertion bodies identical, only the seed path moves.
