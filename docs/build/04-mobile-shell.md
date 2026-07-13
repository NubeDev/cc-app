# Milestone 04 — the thin mobile shell (`ui/`)

> **STATUS: CLOSED (2026-07-12).** Shipped; live state in [`../STATUS.md`](../STATUS.md).
> **As-built note (differs from the plan below):** `ui/` is a **thin hand-built shell over
> `@nube/ext-ui-sdk`** (`App.tsx` · `router.tsx` · `Shell.tsx` · `LoginPage.tsx` ·
> `InviteAcceptPage.tsx` · `useT.tsx`), mounting the care ext via `defineRemote(...)` — it does
> **not** consume a `@nube/minimal-shell` package (that seam wasn't the path taken; the ui-sdk
> dep is currently a `0.0.0-placeholder` pin). The exit-gate outcomes below were met against
> that shell. The checkboxes are ticked to reflect the close; read the plan text as historical.

Login → workspace pick → full-screen mount of the care extension's page, on a phone, as an
installable PWA. *(Original plan: consume lb's `@nube/minimal-shell` package; the as-built note
above records what actually shipped.)*
Scope: [`../scope/ui/mobile-shell-scope.md`](../scope/ui/mobile-shell-scope.md).

## Entry gate

- [x] Milestone 01 closed (a gateway to talk to).
- [x] Extension SDK available: `@nube/ext-ui-sdk` (the `minimal-shell` package route was not
      taken — see the as-built note; the thin shell was built directly over the ui-sdk).
- [x] Milestone 02's `care.ping`-level extension exists to mount (03's admin UI is the
      real payload; either works for bring-up).

## Work items

- [x] `ui/` pnpm workspace: thin shell over `@nube/ext-ui-sdk` — gateway URL,
      `VITE_HOME_EXT=care`, branding blob, PWA manifest.
- [x] Login + workspace pick + full-screen `defineRemote(...)` mount + SSE hub + theme
      (`App.tsx` · `router.tsx` · `Shell.tsx`).
- [x] The invite-accept page wiring (`auth/InviteAcceptPage.tsx`; milestone 05 exercises it).
- [x] Mobile discipline pass: 360px viewport, no blank screens (skeleton + retry),
      installability.
- [x] **Shell chrome in `en` + `es`** (login, workspace pick, errors, accept page) via
      lb's catalog mechanism (`useT.tsx` · `lib/locale.ts` · `locales/`); locale = user
      preference → browser language → `en`.
- [x] **Theme foundation (CLAUDE.md rule 9):** shadcn/ui for the shell chrome; dark/light
      system default + persisted user toggle; host-side `:root{}`/`.dark{}` shadcn variable
      swap propagates into the mounted extension through the SDK token contract (seam verified
      — every later UI milestone builds on it).

## Exit gate

- [x] Phone browser → login as seeded admin → care ext page mounted full-screen; zero lb
      chrome; PWA installs.
- [x] Playwright (real node): login → mount → SSE connected; CSS isolation (host styles
      byte-identical after ext mount — the SDK contract test); **both themes** (toggle
      flips shell + mounted ext together; persists across reload) at **360px and 1280px**.
- [x] `ui/` is the thin shell over `@nube/ext-ui-sdk` (as-built note above); product UI is
      all extension UI behind `defineRemote(...)`.
- [x] STATUS.md moved.

## Subagent notes

Small; one session. Most "work" is configuration + the Playwright harness (which later
milestones reuse — build it well: boot node, seed fixture, drive browser).

## Sources

`../scope/ui/mobile-shell-scope.md` · lb `frontend/minimal-shell-scope.md` · CLAUDE.md
rule 6.
