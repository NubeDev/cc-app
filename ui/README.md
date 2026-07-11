# `ui/`

The thin mobile-first shell. Only jobs: auth, workspace pick, full-screen mount
of the extension's page, SSE wiring, PWA manifest. **Not** a vendored copy of
lb's shell.

## Layout

```
ui/
├── package.json
├── tsconfig.json
├── vite.config.ts          # vite-plugin-pwa manifest
├── tailwind.config.ts
├── index.html
├── scripts/i18n-check.mjs  # CI gate (CLAUDE.md rule 8)
└── src/
    ├── main.tsx            # React root
    ├── App.tsx             # LocaleProvider + Shell + Router
    ├── router.tsx          # routes
    ├── api/                # gateway + auth.ts
    ├── auth/               # LoginPage, InviteAcceptPage
    ├── components/         # one per file
    ├── hooks/              # useT, useWorkspaces
    ├── lib/                # locale.ts
    ├── locales/            # en.json + es.json (CI parity gated)
    ├── pages/              # WorkspacePicker, ExtMount, Offline
    └── styles/index.css    # @tailwind base/components/utilities (host owns these)
```

## Routes

- `/` → redirect `/workspaces`
- `/login` → `LoginPage`
- `/invite/:token` → `InviteAcceptPage` (pre-auth, lb gap #3)
- `/workspaces` → `WorkspacePickerPage`
- `/ext/:workspaceId/*` → `ExtMountPage` (mounts the `care` extension)
- `*` → `OfflinePage`

## Rules

- No admin chrome, no sidebar, no dock — bottom tab bar lives **inside** the
  extension (one-handed reach, care-scope §Intent).
- `tokens.css`-style rules apply here **only** for the host chrome. The
  extension's UI is what carries the per-extension tokens (see
  `rust/extensions/care/ui/README.md`).
- CLAUDE.md rule 6: `remoteEntry.tsx` lives in the extension, not here.
- CLAUDE.md rule 8: en+es from day one; `pnpm i18n:check` is the CI gate.
- CLAUDE.md rule 9: shadcn/ui components; **the host owns `:root{}`/`.dark{}`**
  and the shadcn variable swap — dark/light = system default + persisted user
  toggle, propagated to the mounted extension via the SDK token contract.
  Mobile-first (360px) but laptop-good (~1280px). Design language: **modern iOS**
  — root [`PRODUCT.md`](../PRODUCT.md) + [`DESIGN.md`](../DESIGN.md); build/review
  UI with the impeccable skill (`/impeccable craft|critique|polish`).

## Owner

Filled by build milestone [`../docs/build/04-mobile-shell.md`](../docs/build/04-mobile-shell.md).
Long-term: this shell should come **from lb** as a package (care-scope §lb gap
#2: `frontend/minimal-shell-scope.md`). Until then: thin host, allowed.