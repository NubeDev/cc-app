# CLAUDE.md — cc-app

Guidance for Claude Code (and any AI agent) working in this repository.

## What this is

**cc-app** (working name) is a childcare-management product host that **embeds the lb
core** (`NubeDev/lb`) via the `lb-node` seam — same posture as `NubeIO/rubix-ai`, the
reference embedder. All childcare domain logic and **100% of the product UI** are
**extensions** (in-repo under `rust/extensions/<id>/`) built against the **published SDKs
only** (`lb-ext-sdk` / `@nube/ext-ui-sdk`). The host binary (`rust/node/`) is a boot shim;
`ui/` is a thin mobile-first shell (auth + federation mount), **not** a vendored copy of
lb's shell.

**Status: SCOPED, NOT BUILT.** Read `docs/STATUS.md` first, then the product scope
`docs/scope/care/care-scope.md` — it is the source of truth for the domain model, tenancy,
and phases.

> **You are in ONE repo of a FAMILY.** Many tasks are not a cc-app edit at all — they land
> in `lb` or an SDK, get released (a git tag), and this repo bumps a pin. **Read
> `docs/WORKFLOW-LB.md`** and decide the owning repo BEFORE coding. The lb gaps named in the
> scope (entity-scoped grants, invites, push, media, minimal shell) are **lb work first** —
> always fix lb generically rather than working around it here.

## Rules that bind this repo

lb's non-negotiables apply downstream:

1. **Rule 10 — no special-casing.** Reach the core and every extension only through the
   generic mediated seams (`BootConfig`, MCP tool resolution, the capability grammar, the
   outbox `Target` trait, `ext.list`). Embedding grants no extra capabilities.
2. **Symmetric nodes.** Role = `BootConfig`, never a code branch.
3. **One responsibility per file** (`docs/FILE-LAYOUT.md`): ≤400 lines hard, ~100 typical;
   never `utils`/`helpers`/`common`.
4. **No mocks, no fake backends.** Tests boot the real store (`mem://`), bus, gateway, and
   extensions, seeded via the real write path. Fakes only for a true external (Stripe, an
   email/push provider), behind one trait in one named file.
5. **Env is a binary concern.** Read env only at the binary boundary, fill `BootConfig`.
6. **Never hand-write an extension's `remoteEntry`.** It is a single
   `defineRemote({ id, styles, page, widgets })` from `@nube/ext-ui-sdk`; the SDK owns the
   scoped mount, React root, widget dispatch, and CSS isolation. A hand-rolled entry is a
   regression — reject it.
7. **Guardian data isolation is sacred.** A guardian may only ever see records for children
   they have a live guardianship edge to. Every read verb in the care extension goes through
   the one `authz/` scoping module, and every new verb ships a cross-family deny-test
   (`docs/scope/care/care-scope.md` §Testing). A leak across families is the worst bug this
   product can have.

## Where docs live

- `docs/STATUS.md` — the "where are we" dashboard. Read first, update last.
- `docs/scope/care/care-scope.md` — the product scope (domain, tenancy, phases, lb gaps).
- `docs/scope/` / `docs/sessions/` / `docs/debugging/` — the ask / the working log / the
  append-only issue history, per `docs/ABOUT-DOCS.md`.
- `docs/SCOPE-WRITTING.md` → write a scope; `docs/HOW-TO-CODE.md` → build one;
  `docs/FILE-LAYOUT.md` → before writing any code.
- `docs/WORKFLOW-LB.md` — the cross-repo workflow (owning repo, local `[patch]`, tag→bump).

## Build / test commands

None yet — the code directories are intentionally empty at scope stage. When the build
starts: `cargo build`/`cargo run` under `rust/` (needs the git-ignored `.cargo/config.toml`
zigcc + `[patch]` file), per-extension `build.sh`, `pnpm` in `ui/`. On-disk state will be
repo-anchored under `.cc-app/`.
