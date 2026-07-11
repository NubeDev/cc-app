# cc-app

A **childcare-management product** (working name `cc-app` — branding TBD) built as a
**product host/node** that embeds the [lb](https://github.com/NubeDev/lb) core as a Rust
library via lb's `BootConfig`/`NodeBuilder` embed seam — the same posture as
`NubeIO/rubix-ai`, the reference embedder.

> **Visibility: PRIVATE (intended).** Proprietary product repository.

## What it is

A brightwheel/lillio-class childcare management platform for **owners/admins of one or more
childcare centers**, the **staff** in their rooms, and the **parents/guardians** of enrolled
children. Mobile-first. The lb core provides identity, the capability wall, the SurrealDB
store, the Zenoh bus, channels (messaging), and the extension runtime; **all childcare
domain logic and 100% of the product UI are extensions** built on the published SDKs
(`lb-ext-sdk` / `@nube/ext-ui-sdk`, Tailwind + shadcn tokens). This repo does **not** vendor
lb's UI shell — `ui/` is a thin mobile-first shell whose only jobs are auth + mounting the
extension UI.

The product scope — domain model, tenancy, features, phases, and the lb gaps it surfaces —
lives at **[`docs/scope/care/care-scope.md`](docs/scope/care/care-scope.md)**. Read it first.

## Current status: SCOPED, NOT BUILT

This repo is at the **scope stage**: the docs are real, the code directories are
deliberately empty. `docs/STATUS.md` is the "where are we" dashboard.

## Repository layout

- `rust/node/` — the host binary: fill `BootConfig` from env at the boundary, `boot_full`, serve. *(empty — pending build)*
- `rust/extensions/` — the product extensions (`rust/extensions/<id>/`), built **against the published SDKs only**. *(empty)*
- `ui/` — the thin mobile-first web shell (login + federation mount; no admin chrome). *(empty)*
- `docs/` — scope → sessions → debugging, plus the family playbooks (mirrored from lb):
  `ABOUT-DOCS.md`, `FILE-LAYOUT.md`, `SCOPE-WRITTING.md`, `HOW-TO-CODE.md`, and
  **`WORKFLOW-LB.md`** (cross-repo workflow — read before any task that might belong upstream).
- `doc-site/content/public/` — shipped, reader-facing truth (MDX).

## The family

`cc-app → lb` (git dep by `node-v*` tag), `extensions → SDKs` (by `sdk-v*`/`ui-v*` tags).
Nothing upstream depends on or special-cases this product (lb rule 10). Unlike rubix-ai, the
extensions live **in this repo** — the product *is* its extensions; see
`docs/WORKFLOW-LB.md` §1.
