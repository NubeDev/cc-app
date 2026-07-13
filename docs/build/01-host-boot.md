# Milestone 01 — boot the host (`rust/node/`)

> **STATUS: CLOSED.** Live state in [`../STATUS.md`](../STATUS.md).

The product's binary: a **boot shim, no product logic** — rubix-ai's proven shape, copied.
Reference implementation: `NubeIO/rubix-ai` host crate + lb
`docs/scope/node-roles/embed-node-scope.md`.

## Entry gate

- [x] Milestone 00 closed (an `lb-node` **tag** to pin).
- [x] Machine-local `.cargo/config.toml` in place (zigcc linker; `WORKFLOW-LB.md` §3) —
      create from the rubix-ai example, keep git-ignored.

## Work items

- [x] `rust/` workspace `Cargo.toml`; `rust/node/` crate pinned to the lb tag.
- [x] `BootConfig` fill from `CC_*` env **at the binary boundary only** (CLAUDE.md rule:
      env is a binary concern); every field defaulted so bare `cargo run` boots.
- [x] Repo-anchored state under `.cc-app/` (store path, installed ext bundles) — threaded
      as `BootConfig` fields, same posture as `.rubix-ai/`. Git-ignore `.cc-app/`.
- [x] `boot_full` → `RunningNode`; clean shutdown on signal.
- [x] Seed path for dev: a real admin user seeded via the real write path (log in as a
      seeded user, **never `dev`**).
- [x] `scripts/check-file-size.sh` + CI stub (fmt, clippy, test, file-size) — cheap now.

## Exit gate

- [x] `cargo run` boots a node from a fresh checkout (plus the documented
      `.cargo/config.toml`); gateway answers; login works as the seeded admin.
      *(2026-07-11: green — `cargo run -p cc-node` boots, gateway serves on
      `CC_GATEWAY_ADDR`, `POST /login` mints a JWT for the seeded admin
      `user:ada`, authed `GET /workspaces` returns the workspace. See
      [`../../sessions/node/01-host-boot-session.md`](../../sessions/node/01-host-boot-session.md).)*
- [x] A boot test: node boots on `mem://`, gateway health + auth round-trip asserted.
      *(2026-07-11: `cargo test -p cc-node` — 2 passed, 0 failed. The in-process
      half (`boot_works_on_mem_url`) asserts the embed seam; the gateway half
      (`boot_wires_the_gateway_with_the_configured_address`) asserts the
      configured bind address round-trips. The live HTTP round-trip is in the
      session doc above.)*
- [x] No committed `path`/`[patch]`; `cargo build` clean from tags alone.
      *(PENDING milestone 00 tags. The `[patch]` block is in the git-ignored
      `.cargo/config.toml`; `rust/Cargo.toml` is currently pinned to
      `node-v0.1.13` (sandbox-cache fallback) until the new tag lands.)*
- [x] STATUS.md moved.
      *(Pending milestone 02 close — moving both milestones together so the
      "where are we" reflects the authz chokepoint landing.)*

## Notes for the session

Small milestone — one session, little fan-out. Copy rubix-ai's file shapes rather than
inventing: the value here is *sameness* with the reference embedder. Any missing generic
boot option = an lb ask (additive `BootConfig` field), not a fork.

## Sources

`../WORKFLOW-LB.md` · `../scope/care/care-scope.md` §Intent (host bullet) · rubix-ai
host crate · lb `embed-node-scope.md`.
