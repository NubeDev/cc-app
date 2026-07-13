# UI — the thin shell + the care extension surfaces

The product UI is split in two, and **100% of it is extension UI** — there is no
vendored lb shell:

- **The shell** (`ui/`) is a thin, mobile-first host: login, workspace pick, and the
  federation mount point. It owns the theme (the `:root{}` / `.dark{}` variable swap)
  and the top-bar language + light/dark toggles, and threads the signed-in person's
  workspace role into the mounted extension.
- **The care extension UI** (`rust/extensions/care/ui/`) is the whole product: every
  screen a person actually uses — the roster, the child and family editors, the
  waitlist, the attendance kiosk and dashboard, the menu planner and guardian week,
  the daily feed and the staff two-tap logger, messages and announcements — mounted
  behind a single `defineRemote(...)` entry from the UI SDK.

> **Status: phase 1 shipped.** The ask of record is
> [`docs/scope/ui/mobile-shell-scope.md`](../../../../docs/scope/ui/mobile-shell-scope.md).

## Stack

- **shadcn/ui only**, on Tailwind via the extension Tailwind preset — the one component
  library. The extension UI uses **semantic tokens only**; a hardcoded color is a
  review-blocking defect, so the host's theme swap flows through to every extension
  screen.
- **Mobile-first + laptop-good** — a phone at 360px is the design target, and every
  screen is also designed at ~1280px.
- **Dark + light** — follows the system with a persisted user toggle.
- **Design language: modern iOS** — large-title hierarchy, bottom tabs and sheets, the
  system font stack, soft depth.

## Bilingual

Every screen renders fully in **English and Spanish**, selected per user and propagated
from the shell into the extension. No hardcoded user-facing strings — the catalogs hold
the words, and a CI completeness gate fails any key missing in either language. A screen
that can't render in both languages is not done.

## The federation seam

The shell never hand-writes an extension's remote entry: the care UI ships a single
`defineRemote({ id, styles, page, widgets })` from the UI SDK, which owns the scoped
mount, the React root, widget dispatch, and CSS isolation. The shell dynamic-imports
that remote and mounts it with the caller's role context.
