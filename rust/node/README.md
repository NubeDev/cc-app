# `rust/node/`

The host binary — a **boot shim, no product logic**. Posture copied from
`NubeIO/rubix-ai` (the reference embedder) and lb's `embed-node-scope.md`.

## Responsibilities (at the binary boundary only)

- Read `CC_*` env, fill `BootConfig` (CLAUDE.md rule 5: env is a binary concern).
- `boot_full` → `RunningNode`; serve; clean shutdown on signal.
- Anchor on-disk state under `.cc-app/` (git-ignored).

## Owner

Filled by build milestone [`../../docs/build/01-host-boot.md`](../../docs/build/01-host-boot.md).
Governed by [`../../docs/scope/care/care-scope.md`](../../docs/scope/care/care-scope.md) §Intent
(host bullet). Cross-repo dependency bumps follow
[`../../docs/WORKFLOW-LB.md`](../../docs/WORKFLOW-LB.md).

## Rules

- [`../../docs/FILE-LAYOUT.md`](../../docs/FILE-LAYOUT.md), [`../../docs/HOW-TO-CODE.md`](../../docs/HOW-TO-CODE.md).
- Never fork lb: a missing boot option is an additive `BootConfig` field upstream.