# `rust/`

Cargo workspace for the host binary and the in-repo product extensions. No code lives
elsewhere.

## Layout

- `node/` — the host binary. A boot shim, no product logic.
  See [`../docs/build/01-host-boot.md`](../docs/build/01-host-boot.md).
- `extensions/` — product extensions built **against the published SDKs only**
  (`lb-ext-sdk` / `@nube/ext-ui-sdk`). Today: `care/` — the single backend extension
  owning all childcare domain logic and 100% of the product UI.
  See [`../docs/build/02-care-skeleton-authz.md`](../docs/build/02-care-skeleton-authz.md).

## Rules that bind this tree

- [`../docs/FILE-LAYOUT.md`](../docs/FILE-LAYOUT.md) — one verb per file, ≤400 lines,
  no `utils`/`helpers`/`common`.
- [`../docs/HOW-TO-CODE.md`](../docs/HOW-TO-CODE.md) — the per-session procedure.
- [`../CLAUDE.md`](../CLAUDE.md) — the binding rules (rule 10: no special-casing,
  rule 4: no mocks, rule 7: guardian isolation, rule 8: en+es from day one).
- Cross-repo work follows [`../docs/WORKFLOW-LB.md`](../docs/WORKFLOW-LB.md).

Status: SCOPED, not built. Build work begins at milestone `01-host-boot`.