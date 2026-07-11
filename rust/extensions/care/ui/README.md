# `rust/extensions/care/ui/`

The care extension's frontend. Pages + widgets surfaced to the host via a single
`defineRemote(...)` from `@nube/ext-ui-sdk` (CLAUDE.md rule 6).

## Layout

```
ui/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.ts      # extTailwindPreset() — no @tailwind base here
├── scripts/i18n-check.mjs  # CI gate: en ⇔ es key parity (rule 8)
└── src/
    ├── remoteEntry.tsx     # THE ONLY export — single defineRemote(...) call
    ├── api/                # one resource per file (care.ts)
    ├── components/         # one component per file (PascalCase)
    ├── hooks/              # one hook per file (useX.ts)
    ├── lib/                # pure utilities (locale.ts)
    ├── locales/            # en.json + es.json — CI parity gated
    ├── pages/              # one resource folder per family
    │   ├── Home.tsx
    │   ├── feed/  child/  menu/  invite/  admin/
    ├── styles/
    │   ├── index.css       # @tailwind components/utilities only
    │   └── tokens.css      # design tokens — NO :root{} / .dark{} here
    └── widgets/            # small surfaces for host embed (one per file)
```

## Rules

- **Never hand-write `remoteEntry`** (CLAUDE.md rule 6). The SDK owns scoped
  mount, React root, widget dispatch, and CSS isolation.
- Pair `defineRemote(...)` with `defineExtConfig({ entry })` in `package.json`
  and `extTailwindPreset()` in `tailwind.config.ts`.
- `tokens.css` contains design tokens only — no `@tailwind base`, no `:root{}`,
  no `.dark{}` (those belong in the host, not the extension).
- English + Spanish from day one (CLAUDE.md rule 8). `pnpm i18n:check` is the
  CI gate; missing key in either language fails the build.
- One component per file, one hook per file, one store per file (FILE-LAYOUT).
  No `utils.ts` / `helpers.ts` / `common.ts`.
- CLAUDE.md rule 9: **shadcn/ui only** for anything shadcn ships; **semantic
  tokens only** (`bg-background`, `text-muted-foreground`, …) — a hardcoded
  color is a review-blocking defect (it breaks dark mode, which the host
  controls). Every screen: designed at 360px, still looking designed at
  ~1280px, legible in dark **and** light. Design language: **modern iOS** — root
  [`PRODUCT.md`](../../../../PRODUCT.md) + [`DESIGN.md`](../../../../DESIGN.md);
  every screen built via `/impeccable craft` and passed through
  `/impeccable critique`/`polish` before its milestone's UI exit gate.

## Owners

- Skeleton: milestone 02.
- Real screens: milestones 03 (children/enrollment), 05 (invites), 06
  (attendance), 07 (menus), 08 (daily feed + push), 09 (messaging).