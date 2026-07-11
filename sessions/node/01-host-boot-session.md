# Session — milestone 01 (host boot)

**Status:** closed (per the orchestrator's gate; the deferred-from-tags boxes
left UNTICKED below).

**Date:** 2026-07-11

**Milestone:** [`../../docs/build/01-host-boot.md`](../../docs/build/01-host-boot.md)

## What landed

The cc-app host shim, modeled verbatim after `NubeIO/rubix-ai`'s host crate
(pattern copied for sameness — not invented):

- `rust/Cargo.toml` — the real workspace, `lb-node` pinned to
  `node-v0.1.13` (sandbox-cache fallback while milestone 00's tag ships in the
  sibling `../lb` repo; the git-ignored `.cargo/config.toml` `[patch]` redirects
  it at the in-flight `../lb/rust/node` checkout per `WORKFLOW-LB.md` §3).
- `rust/node/Cargo.toml` + `src/{main,boot}.rs` — `CC_*` env at the binary
  boundary → `BootConfig` → `lb_node::boot_full` → `RunningNode::serve`. Same
  defaulting posture as rubix-ai (every field defaulted so bare `cargo run`
  boots); durable repo-anchored state under `.cc-app/`, overridable via
  `CC_HOME` / `CC_STORE_PATH` / `CC_EXT_UI_DIR`; signing-key custody at the
  binary boundary (no logging); gateway on `127.0.0.1:18099` by default
  (distinct from lb's `:8080` and rubix-ai's `:8099`).
- `rust/node/tests/boot_test.rs` — the boot test (exit gate). Two tests:
  `boot_works_on_mem_url` (boot on `mem://` succeeds + the store opens under
  the right scope) and `boot_wires_the_gateway_with_the_configured_address`
  (gateway returns the configured bind addr — the socket binds later in
  `serve()`). Both green.
- `scripts/check-file-size.sh` — the FILE-LAYOUT ≤400-line guard, modeled
  after lb's `rust/scripts/check-file-size.sh`. Tracked human `*.rs`/`*.ts`/
  `*.tsx` over 400 lines fail.
- `.github/workflows/ci.yml` — the CI stub: fmt + clippy + test + file-size.
- `.gitignore` — `.cc-app/` + `.cargo/config.toml` + build outputs.
- `.cargo/config.toml` (gitignored) — zigcc linker wiring + the `[patch]`
  pointing `lb-node` at `../lb/rust/node` (the in-flight
  `updates-to-core` branch). **Machine-local only** — never committed.
  Drop the `[patch]` block and pin the new tag in `rust/Cargo.toml` when
  milestone 00's tag lands. **Two-line swap.**

## Sandbox-specific notes (recorded so this session is reproducible)

- The `.git` directory in this sandbox is bind-mounted read-only → I cannot
  commit. The full scaffolding changeset (the planning session's output)
  stays as uncommitted modifications on disk; the per-milestone work above
  is also on disk only. STATUS.md captures this explicitly.
- The zig cache dir `/home/user/.cache/zig` is also read-only → zig fails
  with `unable to create compilation: ReadOnlyFileSystem`. The fix is in
  the git-ignored `.cargo/config.toml`'s `[env]` block: redirect
  `ZIG_LOCAL_CACHE_DIR` / `ZIG_GLOBAL_CACHE_DIR` to writable scratch under
  `$TMPDIR` (`/tmp/kilo/zig-cache`). This is local-quirk-only and lives with
  the linker wiring.
- No network → cargo can't fetch a fresh `node-v*` tag. The dep is pinned to
  `node-v0.1.13` (the highest tag in the local cargo git cache) so the
  source resolves offline. The `[patch]` redirects to the live
  `../lb/rust/node` regardless. When this leaves the sandbox, the placeholder
  tag pattern in WORKFLOW-LB.md §4 is what we use.

## Green output (paste)

### `cargo test -p cc-node`

```
running 2 tests
test boot_wires_the_gateway_with_the_configured_address ... ok
test boot_works_on_mem_url ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
```

### `cargo run -p cc-node` + HTTP login round-trip

The binary boots, the dev seed prints, the gateway serves on
`http://127.0.0.1:19998`:

```
cc-node: booting embedded lb node (workspace=cc-test, gateway=Addr(127.0.0.1:19998), store=/tmp/kilo/cc-test-store)
boot seed: user:ada is a workspace-admin member of cc-test
…
agent: serving routed agent.invoke
gateway: serving on http://127.0.0.1:19998
```

The full HTTP round-trip — unauth → 401; login → JWT; authed → 200 — is the
exit gate's "gateway health + auth round-trip":

```
$ curl -s -X POST http://127.0.0.1:19998/login \
       -H "Content-Type: application/json" \
       -d '{"user":"user:ada","workspace":"cc-test"}'
# ⇒ {"token":"eyJ…","principal":"user:ada","workspace":"cc-test","caps":[…]}
#    (real signed Ed25519 JWT carrying the seeded admin's caps)

$ curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:19998/workspaces
# ⇒ [{"ws":"cc-test","name":"cc-test","kind":"workspace","status":"active","ts":1783770591}]
#    HTTP 200

$ curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:19998/workspaces   # no token
# ⇒ 401   (the wall is intact — tokenless request refused)
```

## Surprises worth recording

- The dev identity's caps already include `mcp:authz.check_scoped:call` and
  `mcp:authz.scope_filter:call` in the patched lb source. The **era-2
  delegation seam is wired up at the wall**, not behind a separate grant.
  That means milestone 02 can wire the chokepoint to the real lb verbs
  immediately — the "leave the stub if the surface looks unstable" branch
  is **not** taken. Recorded as the resolution to the milestone 02
  "wire it live if the surface looks stable" gate.
- The gateway port stays as-configured through `boot_full` — the actual
  bind happens in `RunningNode::serve()`. The boot test was originally
  written to assert the OS-assigned port; corrected to assert the
  configured port round-trips (the structural guarantee we actually own).

## Follow-ups

- Milestone 02 builds on top of this — the boot is wired, the auth path
  round-trips, the era-2 seam is live.
- When the milestone 00 tag lands, drop `.cargo/config.toml`'s `[patch]`
  block + replace `node-v0.1.13` in `rust/Cargo.toml` with the new tag.
  Two lines.