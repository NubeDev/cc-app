# Build blocked: cc-app cannot build from published tags (unpushed `*-v0.4.1`/`v0.4.2`)

**Date:** 2026-07-13
**Status:** OPEN — blocks milestone 08 (and any `cargo build`/`cargo test`) in a
fresh environment.
**Owning repos:** `NubeDev/lb` + `NubeDev/lb-ext-sdk` first (push the tags), then
this repo (drop the recreated `[patch]`).

## Symptom

A fresh session (no git-ignored `.cargo/config.toml`, sibling `lb` on an
unrelated branch, no `lb-ext-sdk` sibling) fails at dependency resolution:

```
failed to find tag `node-v0.4.2`
reference 'refs/remotes/origin/tags/node-v0.4.2' not found
```

## Root cause (two layers)

1. **The `*-v0.4.1` / `*-v0.4.2` tags were never pushed.** STATUS.md's
   "Release ritual pending" (admin-marker `*-v0.4.1`, credential-mode
   `node-v0.4.2`) was cut only as LOCAL tags in sibling checkouts + proven via a
   local `[patch]`. `git ls-remote` confirms the remotes hold ONLY:
   - `NubeDev/lb` → `node-v0.4.0`
   - `NubeDev/lb-ext-sdk` → `sdk-v0.4.0`

   The declared pins in `rust/Cargo.toml` + `rust/extensions/care/Cargo.toml` +
   `Makefile LB_TAG` say `node-v0.4.2`, so with no `[patch]` present cargo tries
   to fetch a tag that does not exist.

2. **cc-app's shipped source depends on an API that exists only in the unpushed
   `sdk-v0.4.1`.** `rust/extensions/care/src/lib.rs::principal_from_caller`
   reads `caller.admin`. Probing the published SDK:

   ```
   git clone --branch sdk-v0.4.0 https://github.com/NubeDev/lb-ext-sdk
   # crates/lb-ext-native/src/wire.rs — struct Caller { sub, ws, role, delegated }
   #   → NO `admin` field.
   ```

   So even after repointing every pin to `node-v0.4.0` / `sdk-v0.4.0`, the care
   crate would not compile (`no field admin on Caller`). The `admin` marker is
   the `sdk-v0.4.1` addition (host derives it from caps — see STATUS 2026-07-12
   admin-marker banner).

**Why not just derive admin from `caller.role`?** STATUS is explicit: lb mints
EVERY session as `role: member`; admin power rides caps, never the role enum.
Reading `role` to decide the chokepoint admin-pass reintroduces exactly the
admin-pass bug `*-v0.4.1` fixed (bootstrap admin treated as a guardian →
`center.list → []`, `child.get → 403`) AND is fail-OPEN if lb ever did set the
role. Not an acceptable workaround for rule 7.

## The fix (in owning-repo order — WORKFLOW-LB.md §4)

1. **`NubeDev/lb-ext-sdk`:** push the branch carrying the `admin: bool` addition
   on `Caller` and tag it `sdk-v0.4.1`. (Its non-goal: the `admin` marker is
   host-derived from caps, mirrored onto the frame `Caller`.)
2. **`NubeDev/lb`:** push the `native-caller-admin-marker` branch and tag
   `node-v0.4.1` (host derives `admin` from `caps_hold_admin`) + `node-v0.4.2`
   (additive `BootConfig::credential_mode` + `seed_credential`).
3. **cc-app:** with the tags pushed, a clean `cargo build --workspace` with NO
   `[patch]` resolves — that IS the "am I on releases?" check.

## Local unblock (until the tags are pushed) — recreate the `[patch]`

The git-ignored `rust/.cargo/config.toml` (or `.cargo/config.toml` at the crate
root) must carry, alongside the zigcc/ZIG-cache wiring:

```toml
[patch."https://github.com/NubeDev/lb"]
lb-node        = { path = "/home/user/code/rust/lb/rust/node" }
lb-host        = { path = "/home/user/code/rust/lb/rust/crates/host" }
lb-supervisor  = { path = "/home/user/code/rust/lb/rust/crates/supervisor" }
lb-store       = { path = "/home/user/code/rust/lb/rust/crates/store" }
lb-auth        = { path = "/home/user/code/rust/lb/rust/crates/auth" }
lb-role-gateway= { path = "/home/user/code/rust/lb/rust/role/gateway" }

[patch."https://github.com/NubeDev/lb-ext-sdk"]
lb-ext-native  = { path = "/home/user/code/rust/lb-ext-sdk/crates/lb-ext-native" }
```

PREREQUISITE (not satisfied in the 2026-07-13 session): the sibling checkouts
must exist AND carry the admin-marker + credential-mode work —
`~/code/rust/lb` on branch `native-caller-admin-marker` (this session found it on
branch `cal` at an unrelated HEAD, and `lb-ext-native` had moved out of that
tree entirely), and `~/code/rust/lb-ext-sdk` present (this session had no such
directory). Without correct siblings the `[patch]` paths do not resolve either.

## What this session did NOT do

Did not repoint pins to `node-v0.4.0` (would still fail to compile per cause 2),
did not derive admin from `role` (rule-7 regression per above), and did not push
any tag (cross-repo release is a deliberate, reviewed step — WORKFLOW-LB.md). The
milestone-08 code written this session (`log/records.rs` schema + the
`feed_recipients` chokepoint resolver) is committed as un-compiled source pending
a restored build.
