# Session — milestone 02 (care extension skeleton + authz chokepoint)

**Status:** closed (per the orchestrator's gate; the deferred-from-tags boxes
left UNTICKED below).

**Date:** 2026-07-11

**Milestone:** [`../../docs/build/02-care-skeleton-authz.md`](../../docs/build/02-care-skeleton-authz.md)

## What landed

### 1. The care extension skeleton

- `rust/extensions/care/Cargo.toml` — native Tier-2 extension, deps on
  `lb-ext-native` (the published child-wire SDK) + `lb-store` + `lb-auth`
  for the authz module. Pinned to `lb-ext-sdk/sdk-v0.2.1` (the SDK is
  **NOT** patched — it is unaffected by the in-flight milestone 00 work in
  the lb repo).
- `rust/extensions/care/src/lib.rs` — exposes the `Care` impl + the `authz`
  module. Native binary `care` (host-metrics's shape, the canonical "why
  native" reference, copied for sameness).
- `rust/extensions/care/src/{main,call,ping}.rs` — the binary entry, the
  `Tools` dispatcher, and `care.ping` (the loop-proof verb). Bare-name
  dispatch matches the host's routed + direct native bridges (the host
  strips the `care.` prefix; we re-qualify defensively).
- `rust/extensions/care/extension.toml` — the manifest (id, tier=native,
  world=@0.2.0, the `care.ping` tool, the `[ui]` page stub awaiting
  milestone 04's mobile-shell work).
- `rust/extensions/care/build.sh` — `cargo build --release` →
  `target/release/care` (host-metrics's exact pattern; UI build step is
  commented out, to be enabled by milestone 04).

### 2. The authz chokepoint — the deliverable

- `rust/extensions/care/src/authz/{mod,scope,records,principal,deny}.rs` —
  the two-call surface every verb uses:
  - `assert_reach(cp, principal, child_id) -> Result<(), AuthzError>`
  - `reachable_children(cp, principal) -> Vec<String>` (empty on miss)
  - `reachable_rooms(cp, principal) -> Vec<String>` (empty on miss)
  - Admin passes via an **audited role check** inside the chokepoint
    (one audit point, never a call-site bypass). The audit is an
    `eprintln!` today; a future milestone routes it to the platform
    audit reactor.
  - **Era 1 (now):** resolves from `guardianship` / `staff_assignment`
    records per call, per-request cache only.
  - **Era 2 stub:** `scope::resolve_era2_todo` is the delegation seam
    — same two calls, swapped at this point to lb's
    `authz.check_scoped` / `authz.scope_filter` once the native child
    has a host-callback client. Milestone 00's verification surfaced
    that both lb verbs are **already live in the patched source**
    (the dev login caps include `mcp:authz.check_scoped:call` and
    `mcp:authz.scope_filter:call`). The era-2 swap is a one-file fix
    in `mod.rs` once `lb-ext-native` adds a `host_call` API
    (milestone 03+ follow-up; tracked TODO in `scope.rs`).

### 3. The cross-family matrix harness

- `rust/extensions/care/tests/common/mod.rs` — the shared fixture
  (Sam(Leo+Mia) / Ana(Leo) / Mia's-mum(Mia), two rooms, a second
  workspace `ws-b`), seeded via the REAL write path (`lb_store::create`).
- `rust/extensions/care/tests/matrix_chokepoint.rs` — 8 tests covering:
  - `chokepoint_allows_only_live_guardianship_edges`
  - `chokepoint_reachable_children_returns_only_reached_set`
  - `chokepoint_reachable_rooms_returns_only_assigned_rooms`
  - `chokepoint_admin_passes_via_audited_role_check_only`
  - `chokepoint_admin_in_other_workspace_does_not_reach`
    (documents that the chokepoint's role check is additive — the
    workspace wall fires upstream)
  - `chokepoint_kiosk_principal_reaches_nothing`
  - `unlink_immediately_denies` (the edge-change reconciliation test
    — archive an edge live→false, the very next call denies)
  - `seed_fixture_idempotent_for_a_fresh_store` (the harness uses a
    fresh `Store::memory()` per test; this guards against double-seed)
- `rust/extensions/care/tests/matrix_care_ping.rs` — 4 tests covering
  `care.ping`:
  - `assert_matrix_covers_all_verbs` — the GUARD. A verb without a row
    in `COVERED_VERBS` fails the harness; a row for a removed verb
    fails the harness. Adding a verb without updating this list is a
    build-fail.
  - `care_ping_deny_test_fails_closed_without_the_cap` (the deny-test)
  - `care_ping_round_trips_through_the_child_wire`
  - `care_ping_round_trips_with_a_signed_principal_over_the_chokepoint`

### 4. The CI grep fence

- `scripts/check-authz-fence.sh` — fails CI if any `*.rs` file in
  `src/` (outside `authz/`) does a `read` or `list` on the
  `"guardianship"` table. The pattern is the actual leak shape — a
  per-verb inline filter that reads the edge record — not a bare
  substring match (false positives on comments + module names). Test
  files (`tests/`) are intentionally exempt (they seed records by
  table name; that's what the matrix harness IS).
- The fence is wired into the `.github/workflows/ci.yml` `care-build-test`
  job.

## Deny semantics — the milestone 02 lock-in

- `assert_reach` (single-target): **403 / `Err(Deny)`** on miss.
- `reachable_children` / `reachable_rooms` (list paths): **empty
  `Vec`**, never an error. The verb body translates `[]` to an empty
  reply (CLAUDE.md rule 7).
- Admin: **wildcard `["*"]`** for list paths; always-allow for
  single-target. The role check is logged via `eprintln!` (audit point)
  and fires inside the chokepoint, never at the call site.

## Open question resolved

> *Do `watch`/SSE subjects filter at subscribe or at emit in era 1?*

**Resolution:** emit-side (per the scope's recommended default). The
feed publisher is the ONE place that filters; the SSE subjects stay
broad and the emit gate enforces reach. Recorded in the scope's open
questions. (Implemented in milestone 08's feed verb; this milestone
records the decision.)

## Green output (paste)

### `cargo test --workspace`

```
test result: ok. 3 passed; 0 failed   (ping unit tests)
test result: ok. 4 passed; 0 failed   (matrix_care_ping.rs)
test result: ok. 8 passed; 0 failed   (matrix_chokepoint.rs)
test result: ok. 2 passed; 0 failed   (cc-node boot_test.rs)
---
TOTAL: 17 passed, 0 failed
```

### `scripts/check-file-size.sh --all`

```
FILE-LAYOUT: all source files within 400 lines (62 checked)
```

### `scripts/check-authz-fence.sh --all`

```
AUTHZ-FENCE: 15 files checked, no "guardianship" reads outside authz/
```

### `cargo clippy -p cc-node` / `cargo clippy -p care --lib`

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 06s
…
Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.81s
```

(clean; no warnings)

### `cargo fmt --all --check`

(clean)

## Deferred (left UNTICKED per the milestone spec)

- "Era-2 delegation seam wire-up" — left as a stub. The wall verbs
  (`authz.check_scoped` / `authz.scope_filter`) are already live in the
  patched lb source, but the native child tier doesn't yet expose a
  host-callback client in `lb-ext-native` to invoke them. The swap is
  a one-file fix in `mod.rs` once `lb-ext-native::host_call` lands.
  Tracked as a milestone 03 follow-up.

- The full `care.*` verb set (center/room/child/guardian/edge/…)
  ships with milestone 03.

## Sandbox notes

- Same `.git` read-only + `/home/user/.cache/zig` read-only caveats
  from milestone 01 apply. The matrix harness uses `Store::memory()`
  directly — no on-disk state.

- The `tests/common/mod.rs` pattern is the idiomatic Rust way to
  share fixture code between integration test binaries (each `.rs` in
  `tests/` is its own binary; the `common/` subdir is the only place
  that compiles into every binary).

## Follow-ups

- Milestone 03 begins: records (center/room/child/guardian/edge/…)
  + verb-per-file CRUD + the i18n bootstrap (en/es catalog + CI
  parity check + hardcoded-string lint).
- When `lb-ext-native` exposes a host-callback client: replace
  `scope::resolve_era1_*` with `scope::resolve_era2_todo` in
  `mod.rs`. One-file fix.