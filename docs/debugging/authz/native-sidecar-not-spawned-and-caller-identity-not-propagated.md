# The care native sidecar was never spawned; its writes never reached the node store; and rule 7 is unenforceable in-sidecar

- **Area:** boot wiring (native install) / native record I/O (Part B) / authz caller identity (rule 7)
- **Date:** 2026-07-12 — opened + Parts A & B FIXED; the caller-identity half is scoped to lb (open).

> **Status:** ALL CLOSED. Gaps 1 + 2 **FIXED** (cc-app, 2026-07-12). Gap 3 **CLOSED
> (2026-07-12)** — lb shipped native-caller-identity in `sdk-v0.4.0` (the `caller` frame
> field) + `node-v0.4.0` (the `subject`-parameterized reach verbs, gated by
> `mcp:authz.delegate_reach:call`); cc-app bumped the pins and wired the frame caller through
> the sidecar. Rule 7 is now GREEN over a real spawned sidecar (`tests/live_node.rs`:
> stranger→leo 403, stranger child.list 0 kids, Ana→leo 200, restart-durable). A fourth issue
> found while proving Gaps 1+2 (a subject-form mismatch) is **FIXED** (cc-app).

## Context

Milestone 04's STATUS.md read as if every m03 verb was "wired onto the live dispatcher." That was
true only for the IN-PROCESS test path (`tests/live_wire.rs` calls `Care::boot` inside the test); it
was **false for a real node**. Standing the sidecar up on a live cc-node (`tests/live_node.rs`, which
boots a real gateway + store, installs care via `install_native`, and drives `care.*` over real
`/mcp/call`) surfaced three distinct gaps + one bug.

## Gap 1 — the cc-node binary never spawned/registered the care sidecar (FIXED)

**Symptom.** On a live node, every `care.*` call returned 403 and the tool catalog exposed zero care
tools. The login/wall/credential host layer was fully green; only care was unreachable.

**Cause.** `rust/node/` had no `lb-host` dependency and never called lb's native install path. It
booted the embedded lb node and served the gateway, full stop — nothing spawned the `care` Tier-2
binary, so no `mcp:care.*:call` caps were granted and no sidecar existed to serve the verbs.
`make publish-ext` could not fix it: it runs `lb-pack` with a **wasm** signature, but care is a
**native** binary, so `pack` errors.

**Fix (cc-app).**
- `rust/node/src/care_mount.rs` — a boot-time install that mirrors lb's own
  `control_engine::mount`: resolve the built `care` binary dir, build an admin principal holding
  `mcp:native.install:call`, compute the approved grant (mirroring the manifest request), and call
  `lb_host::install_native(&node, &OsLauncher, …)`. Called from `main.rs` between `boot()` and
  `serve()`. Idempotent on ext id (re-install on reboot).
- `rust/node/Cargo.toml` gains `lb-host` + `lb-supervisor` + `lb-auth` at the SAME tag as `lb-node`
  (`node-v0.3.3`, no `[patch]`) — cargo unifies the shared `lb-store`/`lb-auth` instances across the
  whole workspace, so there is NO duplicate-`Store`-type wall (the old Cargo.toml comment warning of
  that was about a `[patch]`/path situation, not a tag build).
- `extension.toml` expanded from 1 declared verb (`ping`) to all 26 (`center`/`room`/`child`/
  `guardian`/`guardianship`/`enrollment`/`invite`), each in `[capabilities].request` + `[[tools]]`,
  plus the `store.*` / `authz.*` / `grants.*` / `invite.*` callback caps the bodies need.

**Verified.** `tests/live_node.rs`: `install_native` spawns the real `care` OS child (26 tools), and
`care.ping` / `care.center.create` / … round-trip over real HTTP `/mcp/call`.

## Gap 2 — every verb wrote to the sidecar's PRIVATE store, invisible to the node (FIXED)

**Symptom.** Even when spawned, a sidecar's writes would be invisible to admin reads and would die on
restart — the node/gateway never sees them.

**Cause.** Every verb body wrote to `cp.store` via `lb_store::{create,read,write,delete}`. lb's
`build_spec` injects `LB_EXT_WS`/`ID`/`TOKEN`/`GATEWAY_URL` into a spawned child but **NOT** a shared
store URL (by design — native-callback-transport scope). So a spawned sidecar opened its OWN private
`mem://` store; records it wrote were invisible to the node and lost on restart. The in-process tests
"worked" only because they shared `node.store` via an `Arc<Store>` — impossible across a real process
boundary.

**Fix (cc-app).** A new `authz/store.rs` `RecordStore` seam with two backends behind ONE API:
- `RecordStore::Callback` (production) — every op is a `SidecarClient::call_tool` to lb's generic
  `store.*` MCP surface (`store.write` = create/upsert, `store.query` = read/list, `store.delete`),
  over the SAME host callback the era-2 reach client already rides. The node's durable store is the
  single source of truth.
- `RecordStore::Local` (era-1 / unit tests) — wraps `Arc<lb_store::Store>` exactly as before.

The seam returns byte-identical shapes (unwraps the `{data}` envelope like `lb_store::read`;
reproduces `create`'s first-settle `Conflict` on the callback path via a pre-read since `store.write`
UPSERTs). Every verb body switched from `cp.store` to `cp.records()`; the chokepoint constructs the
right backend once (`Chokepoint::with_host_callback` → `Callback`, `Chokepoint::new` → `Local`). No
verb decides the backend. **No lb change needed** — the `store.*` surface already exists at
`node-v0.3.3` (host `store_mutate` / `store_query`, cap-gated, callable by a native sidecar).

**Verified.** `tests/live_node.rs` phase 1 seeds a roster; an admin read (`care.center.list`,
`care.child.get`) sees it (proving it landed in the NODE store, visible to a different reader). Phase
2 drops the node + sidecar, re-opens the SAME on-disk store in a fresh node, re-installs care, and the
roster is STILL readable — **data survives a restart.**

## Bug — guardian subject form mismatch broke reach (FIXED)

**Symptom (found while proving Gap 2).** `care.guardianship.link {guardian_sub:"ana"}` failed the
era-2 grant derivation with `grants.assign → Denied`, AND the era-1 edge read would miss.

**Cause.** The seed (and any caller) passes a bare guardian id (`ana`), but a guardian's auth
subject (token `sub`) is `user:ana`. `derive_reach` passed the bare `"ana"` as the grant `subject`;
lb's `Subject::parse("ana")` rejects a colon-less string, so `grants.assign` denied. The era-1 edge
read (`scope.rs`) looks up `edge_id(principal.sub(), child)` = `user:ana::…` while the seeded edge id
was `ana::…` — a silent miss.

**Fix (cc-app).** `authz::canonical_subject` normalizes a guardian subject to the `user:<x>` auth
form (idempotent on an already-prefixed subject). Applied at the top of `guardianship.link` /
`unlink` / `update`, so the edge id, the era-1 lookup, and the era-2 grant subject all address the
SAME identity. One owner of the rule (a drift = a lockout or a leak).

## Gap 3 — rule 7 (guardian isolation) is unenforceable in the native sidecar (CLOSED — fixed by `sdk-v0.4.0` + `node-v0.4.0`)

**Symptom (was).** A stranger guardian (`mallory`, no edge to `leo`) reads Leo: `care.child.get
{id:"leo"}` → **200**, and `care.child.list` → **1 kid**. Both must deny/empty. This was a rule-7
LEAK — the existential bug for this product.

**Resolution (2026-07-12).** lb closed both compounding gaps generically (rule 10 — no
cc-app special-casing), per `lb/docs/scope/extensions/native-caller-identity-scope.md`:
- **GAP A — caller in the frame (`sdk-v0.4.0`).** `lb-ext-native::wire::CallParams` gained an
  additive `caller: Option<Caller>` (`{sub, ws, role, delegated}`); the host stamps the
  authorized principal; `serve_stdio` hands it to `Tools::call_with_caller`. cc-app:
  `Care::call_with_caller` projects it per dispatch (`principal_from_caller`), and the dead
  `LB_EXT_PRINCIPAL_JSON` env seam (which lb never stamped) is RETIRED — the frame is the source now.
- **GAP B — reach ABOUT the caller (`node-v0.4.0`).** `authz.check_scoped` / `authz.scope_filter`
  gained an optional `subject`; a caller holding `mcp:authz.delegate_reach:call` may name it, and
  the host resolves THAT subject's scoped grants. cc-app: the chokepoint passes
  `subject = <frame caller's sub>` (`ReachClient::reaches`/`reachable`), and the care install
  requests + is granted the delegation cap (`extension.toml` + `care_mount::approved_grant` +
  `live_node_support::approved_grant`, all lock-step). No chokepoint call-site changed — its
  two-era shape was already built.

**Verified GREEN.** `tests/live_node.rs` over a real spawned sidecar:
`RULE-7 … Ana→leo=200, stranger→leo=403, stranger child.list=200 with 0 kids` + restart-durable.
`cargo test --workspace` + fmt + file-size + authz-fence + clippy all green.

---

### Historical detail (the state before the fix)

**Symptom.** As above (200 / 1 kid — the leak).

**Cause (platform, not cc-app).** Two lb-side gaps compound:
1. lb's native call frame (`lb-ext-native::wire::CallParams { tool, input }`) carries **no caller
   principal**. The host's `SidecarDispatch` passes only the tool + input to the child. So the
   sidecar cannot know WHO called; `Care::principal_for_call` falls back to a synthetic
   `WorkspaceAdmin`, and the chokepoint's admin-pass fires — bypassing the row filter. (The
   `LB_EXT_PRINCIPAL_JSON` seam the care code hopefully reads is **never stamped by lb** — grep lb:
   it exists only in cc-app.)
2. Even if the caller were known, lb's `authz.check_scoped` / `scope_filter` only answer about the
   **caller's own token**; the sidecar holds the EXTENSION token, not the guardian's, and cannot mint
   a guardian token. So it cannot ask "does `user:ana` reach `child:leo`?" over the callback.

The host's outer `mcp:care.<verb>:call` wall still fires correctly — this is specifically the
row-level second gate (the chokepoint) that cannot see the caller.

**Fix — lb, per WORKFLOW-LB.md.** Scoped upstream in
`lb/docs/scope/extensions/native-caller-identity-scope.md`: (a) propagate the caller's principal into
the native call ABI (additive `caller` field on `CallParams` + host dispatch threads it +
`serve_stdio` hands it to `Tools::call`), and (b) a `subject`-parameterized (or delegated-reach) verb
so a granted extension can resolve the CALLER's row scope. When it ships + cc-app bumps the pin, the
care chokepoint enforces rule 7 with **no call-site change** (its two-era shape is ready).

**Resolved.** The rule-7 stranger-deny assertions in `tests/live_node.rs` are back ON
(`assert_ne!(str_c, 200, …)` + `assert!(stranger_kids == 0, …)`) and GREEN. The
**guardian-facing UI is no longer gated** — a guardian session routed through the sidecar now
reaches only their linked children (rule 7 enforced in-sidecar), so the `children` tab +
reach-filtered reads are safe for guardians as well as admins. (`Ana-reaches-her-own-child`
still asserts, as before.)

## Also corrected

STATUS.md's milestone 04 "wired-in" claim (it described the in-process `live_wire.rs` path as if it
were a real node) was rewritten to state the real wire-in status: cc-node installs the sidecar and
record I/O is durable. As of 2026-07-12 rule 7 in the sidecar is ENFORCED (the lb scope above
shipped as `sdk-v0.4.0` + `node-v0.4.0`); STATUS.md's correction banner is resolved.
