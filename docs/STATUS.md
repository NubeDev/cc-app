# STATUS — cc-app

_The single "where are we" dashboard. Read at the start of a session; update at the end._

**Date:** 2026-07-11

## Current state

**SCOPE STAGE — no code yet.** The repo skeleton exists (`rust/node/`, `rust/extensions/`,
`ui/`, `docs/`, `doc-site/`), the family playbooks are mirrored from lb/rubix-ai, and the
product scope is written: **`docs/scope/care/care-scope.md`** — the domain model
(children / guardians / guardianship edges / rooms / centers), tenancy (workspace = the
childcare organization), the phased feature plan, the extension + UI split, and the **lb
gaps** this product needs fixed upstream.

## What's real

- Docs: `ABOUT-DOCS.md`, `FILE-LAYOUT.md`, `SCOPE-WRITTING.md`, `HOW-TO-CODE.md`,
  `WORKFLOW-LB.md` (cc-app-adapted mirrors), this dashboard, `scope/README.md`.
- Scopes: the master `scope/care/care-scope.md` **plus the full sub-scope set** —
  `care/{care-authz,enrollment-invites,attendance,daily-feed,menus,messaging}-scope.md`,
  `ui/mobile-shell-scope.md`, `billing/billing-scope.md` (phase-2 placeholder-with-teeth).
  The master's "Scope map" is the build order.
- **Persona layer** (2026-07-11): `scope/personas/{admin,teacher,guardian}/` — one doc per
  use case (6 admin, 4 teacher, 5 guardian), journeys over the feature scopes.
- **Upstream lb gaps IMPLEMENTED** (2026-07-11, in `NubeDev/lb`, branch `updates-to-core`,
  **not yet merged/tagged**): entity-scoped-grants (18c60cb), invites (62a3bf2), media
  (f958f48), push-target (a629378), minimal-shell (3c20433) — 53 tests green. Remaining
  before tag: wire Email/Push relay reactors at boot, rate-limit the public invite-accept
  route. Kiosk = existing lb `api-keys`; cap-freshness folded into invites/access-console.
- **i18n MUST recorded** (2026-07-11): English + Spanish 100% from day one —
  `scope/ui/i18n-scope.md` (CLAUDE.md rule 8), gated per build milestone; lb multi-lang
  coverage verification added to build milestone 00.
- **Repo skeleton scaffolded** (2026-07-11): directory tree under `rust/node/`,
  `rust/extensions/care/` (authz chokepoint + folder-of-verbs per FILE-LAYOUT),
  `rust/extensions/care/ui/`, and `ui/` shell, with per-dir READMEs. No source code.
- Session log: `sessions/care/care-scope-session.md`.

## Explicitly deferred

- **Billing: build LAST** (product decision 2026-07-11). `scope/billing/billing-scope.md`
  stays only as the must-not-preclude ledger; no billing work before phase-1 ships.

## Next up — execute the build runbook

**The build plan is written: [`build/README.md`](build/README.md)** — 11 gated milestones
(00 lb-release → 10 hardening-launch) with entry/exit checklists, subagent fan-out notes,
and the parallelism map. A fresh session starts there: pick the first milestone whose
entry gate is open (right now: **00-lb-release** — merge/tag `updates-to-core`, close the
two pre-tag remainders, bump pins here), execute it per `HOW-TO-CODE.md`, tick its gates,
move this file.

Milestone progress: none started. 00 is next.

## Non-goals (unchanged)

- No special-casing of lb or any extension (rule 10).
- No vendored lb UI shell — 100% of the product UI is extension UI behind `defineRemote`.
- No billing/payments in phase 1 (scope §Phases).
