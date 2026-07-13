# care ext renders blank at `/ext/<ws>` — `remoteEntry` passed lazy imports (Promise children)

**Status:** RESOLVED 2026-07-13.

## Symptom

Login worked and landed on `/ext/acme`, but the care extension UI showed only skeleton
placeholder bars — no real DOM. The browser console showed, repeatedly:

```
Uncaught Error: Objects are not valid as a React child (found: [object Promise]).
  If you meant to render a collection of children, use an array instead.
The above error occurred in the <Context.Provider> component:
    at RuntimeProvider (…/lb-ext-ui-sdk/dist/runtime.js)
```

The prior handover guessed "the SDK mount needs a session provider the shell's bridge doesn't
pass." That was wrong — the SDK's `RuntimeProvider` already supplies the session; the crash was
upstream of that.

## Root cause

`rust/extensions/care/ui/src/remoteEntry.tsx` handed `defineRemote` **lazy dynamic-import
thunks** instead of the render functions / CSS string the SDK contract requires:

```tsx
// WRONG — every field is a thunk returning a Promise
defineRemote({
  id: "care",
  styles: () => import("./styles/tokens.css"),        // Promise, not a CSS string
  page:   () => import("./pages/Home"),               // Promise, not (ctx,bridge)=>ReactNode
  widgets: () => [ import("./widgets/…"), … ],         // array of Promises, not a keyed map
});
```

The SDK's `defineRemote` (`lb-ext-ui-sdk/src/remote.tsx`) does `page(ctx, bridge)` and renders
the result **directly** inside `RuntimeProvider`. `page()` returned a `Promise`, so React tried to
render a Promise as a child → the thrown error, caught by React, leaving the mount empty (skeletons).

The SDK contract (`RemoteDef`):
- `styles?: string` — a CSS **string** (`import styles from "./x.css?inline"`).
- `page?: (ctx, bridge) => ReactNode` — a render function returning JSX.
- `widgets?: Record<widgetId, (ctx, bridge) => ReactNode>` — a keyed map of render functions.

The same mistake produced the stale `remoteEntry.tsx` TS errors the shell lint had been emitting
("Type `() => Promise<…>` is not assignable to type `ReactNode`") — a compile-time signal of the
runtime crash that was being ignored.

## Fix

Rewrote `remoteEntry.tsx` to the canonical shape: static imports, `?inline` CSS string, the page
wrapped in the ext's own `LocaleProvider` (the i18n/theme context `useT` reads — the SDK supplies
`RuntimeProvider`/session, the ext supplies its own locale provider), and widgets as a keyed map.
Added `src/vite-env.d.ts` declaring `*.css?inline` so the `?inline` import type-checks.

Also fixed `vite.config.ts`: it imported a non-existent `extUiSdk` plugin; the SDK's Vite export is
`defineExtConfig`, a **config fragment spread into `defineConfig`** (not a plugin). This unblocked
`pnpm build` / `make pack` (the production `remoteEntry.js` bundle).

## Proof

`ui/e2e/ext-mount.spec.ts` — a real browser signs in, enters the workspace, and asserts the scoped
ext root (`[data-ext-root='care']`) is visible with real content (the Today/Children bottom tabs)
and **zero** page errors. `make e2e-ui` green (5/5). The ext bundle builds
(`dist/remoteEntry.js`).

## Lesson

A `remoteEntry` is generated boilerplate around the SDK contract — `styles: string`,
`page:/widgets:` render functions. Lazy `() => import(...)` thunks are a regression (they resolve to
Promises the SDK renders verbatim). When `remoteEntry.tsx` throws TS "not assignable to ReactNode",
that is the runtime blank-render bug at compile time — fix it, don't silence it.
