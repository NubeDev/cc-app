# `rust/extensions/care/ui/`

The care extension's frontend — **pages + widgets** surfaced to the host via a
single `defineRemote({ id, styles, page, widgets })` from `@nube/ext-ui-sdk`.

## Rules

- **Never hand-write `remoteEntry`** (CLAUDE.md rule 6 / HOW-TO-CODE §4b). A single
  `defineRemote(...)` call; the SDK owns the scoped mount, React root, widget
  dispatch, and CSS isolation.
- Pair with `defineExtConfig({ entry: "src/remoteEntry.tsx" })`,
  `extTailwindPreset()`, and a `tokens.css` with no `@tailwind base`/`:root{}`/`.dark{}`.
- English + Spanish from day one (CLAUDE.md rule 8). Catalogs under
  `src/locales/`; CI gate fails any key missing in either language.

Reference: [`../../../../CLAUDE.md`](../../../../CLAUDE.md) rule 6,
[`../../../../docs/HOW-TO-CODE.md`](../../../../docs/HOW-TO-CODE.md) §4b.

## Owner

Filled by build milestones 02 (skeleton), 04 (mobile shell hosts it), and the
per-feature UI screens shipped by 03, 05, 06, 07, 08, 09.
Governed by [`../../../../docs/scope/care/care-scope.md`](../../../../docs/scope/care/care-scope.md) §Intent
and [`../../../../docs/scope/ui/mobile-shell-scope.md`](../../../../docs/scope/ui/mobile-shell-scope.md).

## Reference

`NubeIO/rubix-ai` `host-metrics` / `proof-panel` are the canonical shapes
(per HOW-TO-CODE §4b).