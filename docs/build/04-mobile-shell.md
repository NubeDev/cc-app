# Milestone 04 — the thin mobile shell (`ui/`)

Login → workspace pick → full-screen mount of the care extension's page, on a phone, as an
installable PWA. **Consume lb's `@nube/minimal-shell` package** (shipped in milestone 00);
`ui/` is the shell *instance* + config only.
Scope: [`../scope/ui/mobile-shell-scope.md`](../scope/ui/mobile-shell-scope.md).

## Entry gate

- [ ] Milestone 01 closed (a gateway to talk to).
- [ ] `@nube/minimal-shell` tag available (milestone 00). If it genuinely lags: the
      interim host is allowed — **hard ~15-file budget**, every part contributed upstream
      (a loan to lb, not a fork) — but with 00 done first this branch shouldn't trigger.
- [ ] Milestone 02's `care.ping`-level extension exists to mount (03's admin UI is the
      real payload; either works for bring-up).

## Work items

- [ ] `ui/` pnpm workspace: minimal-shell as the app, config only — gateway URL,
      `VITE_HOME_EXT=care`, branding blob, PWA manifest.
- [ ] Login + workspace pick + full-screen `ext.list`-discovered mount + SSE hub + theme —
      all from the package; anything missing is an upstream minimal-shell PR, never local
      shell code.
- [ ] The invite-accept page wiring (the package has it; milestone 05 exercises it).
- [ ] Mobile discipline pass: 360px viewport, no blank screens (skeleton + retry),
      installability.
- [ ] **Shell chrome in `en` + `es`** (login, workspace pick, errors, accept page) via
      lb's catalog mechanism; locale = user preference → browser language → `en`.

## Exit gate

- [ ] Phone browser → login as seeded admin → care ext page mounted full-screen; zero lb
      chrome; PWA installs.
- [ ] Playwright (real node): login → mount → SSE connected; CSS isolation (host styles
      byte-identical after ext mount — the SDK contract test).
- [ ] `ui/` contains **no** shell logic beyond config (review the diff against that
      sentence).
- [ ] STATUS.md moved.

## Subagent notes

Small; one session. Most "work" is configuration + the Playwright harness (which later
milestones reuse — build it well: boot node, seed fixture, drive browser).

## Sources

`../scope/ui/mobile-shell-scope.md` · lb `frontend/minimal-shell-scope.md` · CLAUDE.md
rule 6.
