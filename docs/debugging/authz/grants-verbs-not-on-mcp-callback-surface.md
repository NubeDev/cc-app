# `grants.*` verbs are not reachable over the native host-callback

- **Area:** authz / era-2 scoped-grant derivation
- **Date:** 2026-07-12 (milestone 03, Step C)
- **Status:** OPEN — needs an upstream **lb** fix; tracked here, worked around
  by keeping era-1 as the live reach path (see "Current posture").

## Symptom

Wiring era-2 (Step C), the care chokepoint's grant-derivation path
(`care.guardianship.link` → `authz::grant::derive_reach` →
`SidecarClient::call_tool("grants.assign", …)`) returns `CallError::Denied`
(HTTP 403) against a **real** booted gateway, even with a token that holds
`mcp:grants.assign:call` (and `mcp:care.reach.child:call` for anti-widen).

The READ half of the same callback works fine: `authz.scope_filter` and
`authz.check_scoped` round-trip over the identical `SidecarClient` and return
correct results (proven in `tests/matrix_era2.rs`).

## Root cause (found by reading lb `node-v0.3.0`, read-only)

lb's MCP dispatcher `call_tool_at_depth`
(`rust/crates/host/src/tool_call.rs`) routes tool families to their handlers
by prefix. It routes **`authz.*`** to `call_authz_tool`:

```rust
} else if qualified_tool.starts_with("authz.") {
    crate::call_authz_tool(&node.store, principal, ws, qualified_tool, &input).await?
}
```

…but there is **no arm for `grants.*` / `roles.* / teams.*`**. Those verbs are
*implemented* in `call_authz_tool` (it matches `"grants.assign"`,
`"grants.revoke"`, …) but the dispatcher never routes a `grants.*` call to it,
so a `grants.assign` sent through `POST /mcp/call` falls through to the generic
extension-registry path and is denied (no such registered tool).

Consequence: `grants.*` is reachable only via the dedicated REST routes
(`POST /admin/grants`, `/admin/grants/revoke`) that the admin console uses —
NOT via the generic `/mcp/call` bridge that a native (Tier-2) sidecar reaches
the host through. So a native extension can *read* the scoped-grant surface
(`authz.check_scoped` / `authz.scope_filter`) but cannot *mint* a scoped grant
over the callback. The entity-scoped-grants scope's promise — "a domain event
(a guardianship edge linked/unlinked) can create/remove scoped grants through
the normal granted `grants.*` verbs" — is not yet true for the native tier.

## The fix (upstream lb — do NOT work around in care, rule 10)

Route `grants.*` (and `roles.*` / `teams.*`) through the MCP dispatcher the
same way `authz.*` is routed — one added arm in `call_tool_at_depth`:

```rust
} else if qualified_tool.starts_with("grants.")
    || qualified_tool.starts_with("roles.")
    || qualified_tool.starts_with("teams.") {
    crate::call_authz_tool(&node.store, principal, ws, qualified_tool, &input).await?
}
```

`call_authz_tool` already handles every one of those verbs, gated by the
existing admin caps (`mcp:grants.assign:call`, …) — so this is additive, no new
verb, no grammar change, no WIT bump. It belongs in lb, gets a `node-v*` tag,
and cc-app bumps the pin (WORKFLOW-LB.md §4). Filed as the milestone-03 → lb
follow-up.

## Current posture in cc-app (until lb ships the routing fix)

- **Era-2 READ delegation is LIVE and proven** — the chokepoint delegates
  `assert_reach` / `reachable_children` to `authz.check_scoped` /
  `authz.scope_filter` when a `ReachClient` is present. Proven end-to-end over
  real HTTP in `tests/matrix_era2.rs` (grant→reach, cross-family deny,
  revoke→grant-physically-gone, workspace isolation), with grants seeded via
  lb's real in-process write path (`lb_host::grants_assign`) since the callback
  mint is blocked.
- **Era-2 WRITE derivation is WIRED but not the live path** — `authz::grant::{
  derive_reach, remove_reach}` and the transactional link/unlink/update call
  sites are complete and correct against the verb contract; they activate the
  moment lb routes `grants.*`. Until then, calling them over the callback
  returns `Denied`, which the verbs surface fail-closed (the edge write is
  rolled back), so an era-2 chokepoint is **not** wired into the live `Care`
  dispatcher yet.
- **Era 1 (store-resolved edges) is the LIVE reach path** for now — exactly the
  `care-authz-scope.md` §"Era 2" contract: "keep the era-1 resolution as the
  documented fallback path … if lb's verbs aren't reachable." lb's *write*
  verbs aren't reachable over the callback, so era-1 is correctly the live
  path. The matrix harness (`matrix_chokepoint.rs`) exercises it, including
  `unlink_immediately_denies`.

## Regression test

`tests/matrix_era2.rs::era2_grant_then_reach_and_revoke_removes_it` is the
forward-looking regression: it proves the read delegation + the
grant-gone-after-revoke invariant over real HTTP. When lb routes `grants.*`,
swap its in-process `seed_reach_grant`/`revoke_reach_grant` for the
`SidecarClient` `grants.assign`/`grants.revoke` callbacks and delete this
work-around note — the assertion body stays identical.
