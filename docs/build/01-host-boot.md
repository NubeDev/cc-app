# Milestone 01 — boot the host (`rust/node/`)

The product's binary: a **boot shim, no product logic** — rubix-ai's proven shape, copied.
Reference implementation: `NubeIO/rubix-ai` host crate + lb
`docs/scope/node-roles/embed-node-scope.md`.

## Entry gate

- [ ] Milestone 00 closed (an `lb-node` **tag** to pin).
- [ ] Machine-local `.cargo/config.toml` in place (zigcc linker; `WORKFLOW-LB.md` §3) —
      create from the rubix-ai example, keep git-ignored.

## Work items

- [ ] `rust/` workspace `Cargo.toml`; `rust/node/` crate pinned to the lb tag.
- [ ] `BootConfig` fill from `CC_*` env **at the binary boundary only** (CLAUDE.md rule:
      env is a binary concern); every field defaulted so bare `cargo run` boots.
- [ ] Repo-anchored state under `.cc-app/` (store path, installed ext bundles) — threaded
      as `BootConfig` fields, same posture as `.rubix-ai/`. Git-ignore `.cc-app/`.
- [ ] `boot_full` → `RunningNode`; clean shutdown on signal.
- [ ] Seed path for dev: a real admin user seeded via the real write path (log in as a
      seeded user, **never `dev`**).
- [ ] `scripts/check-file-size.sh` + CI stub (fmt, clippy, test, file-size) — cheap now.

## Exit gate

- [ ] `cargo run` boots a node from a fresh checkout (plus the documented
      `.cargo/config.toml`); gateway answers; login works as the seeded admin.
- [ ] A boot test: node boots on `mem://`, gateway health + auth round-trip asserted.
- [ ] No committed `path`/`[patch]`; `cargo build` clean from tags alone.
- [ ] STATUS.md moved.

## Notes for the session

Small milestone — one session, little fan-out. Copy rubix-ai's file shapes rather than
inventing: the value here is *sameness* with the reference embedder. Any missing generic
boot option = an lb ask (additive `BootConfig` field), not a fork.

## Sources

`../WORKFLOW-LB.md` · `../scope/care/care-scope.md` §Intent (host bullet) · rubix-ai
host crate · lb `embed-node-scope.md`.
