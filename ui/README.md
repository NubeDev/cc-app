# `ui/`

The **thin mobile-first shell**. Its only jobs: auth, workspace pick, full-screen
mount of the extension's page, SSE wiring, and a PWA manifest. **Not** a vendored
copy of lb's shell.

## Owners

- Filled by build milestone [`../docs/build/04-mobile-shell.md`](../docs/build/04-mobile-shell.md).
- Long-term, this shell should come **from lb** as a package (care-scope §lb gap #2:
  `frontend/minimal-shell-scope.md`). Until then: thin host, allowed.

## Rules

- CLAUDE.md rule 6: this is not where extensions live — the **care** extension
  ships its own UI behind `defineRemote` and the shell mounts it.
- CLAUDE.md rule 8: en+es from day one; locale pick before first render.
- [`../docs/FILE-LAYOUT.md`](../docs/FILE-LAYOUT.md), [`../docs/HOW-TO-CODE.md`](../docs/HOW-TO-CODE.md).
- No admin chrome, no sidebar, no dock. Bottom tab bar, one-handed reach
  (care-scope §Intent).