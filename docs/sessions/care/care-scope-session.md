# Care — product scope setup (session)

- Date: 2026-07-11
- Scope: ../../scope/care/care-scope.md
- Status: done (scope-stage session — docs only, no code by design)

## Goal

Stand the `cc-app` repo up to scope stage, the same way rubix-ai was: mirror the family
playbooks, write the product scope for a brightwheel/lillio-class childcare app on lb, lay
out the core directories (empty), and surface the lb gaps to fix upstream.

## What changed

- Repo docs: `README.md`, `CLAUDE.md`, `docs/STATUS.md`, `docs/WORKFLOW-LB.md` (cc-app
  adaptation of rubix-ai's), mirrored playbooks (`ABOUT-DOCS.md`, `FILE-LAYOUT.md`,
  `SCOPE-WRITTING.md`, `HOW-TO-CODE.md` — copied from rubix-ai, names/paths adapted).
- The product scope: `docs/scope/care/care-scope.md` + `docs/scope/README.md` index +
  public stub `doc-site/content/public/care/care.md`.
- Dir skeleton (empty, `.gitkeep` only): `rust/node/`, `rust/extensions/`, `ui/`,
  `docs/debugging/`, `docs/vision/`.

## Decisions & alternatives

- **Workspace = organization** (centers/rooms are records) over workspace-per-center —
  multi-center owners get one pane; the hard wall belongs between businesses.
- **Guardianship as a many-to-many edge** with per-edge flags over any household/family
  grouping — handles blended families (one dad, two kids, different mums) natively.
- **One `care` extension** over per-feature extensions — a single `authz/` guardian-scoping
  chokepoint instead of N copies; billing splits out in phase 2.
- **Thin mobile shell in `ui/`** over vendoring lb's shell (rubix-ai's choice) — the lb
  shell is desktop-admin-shaped; the product is guardian-phone-first. Filed lb gap #2
  (minimal shell package) so this eventually comes from lb.
- **Extensions in-repo** (`rust/extensions/`) over a sibling `-extensions` repo — the
  product *is* its extensions; still SDK-only discipline.

## Tests

N/A — scope-stage session, no code. The scope's Testing plan defines the mandatory
categories (cap-deny, workspace isolation, **cross-family isolation per verb**, edge-change
reconciliation) the build sessions must satisfy.

## Debugging

None (no code).

## Public / scope updates

Public stub created at `doc-site/content/public/care/care.md` (TODO until first ship).

## Session 2 (same day) — scope expansion + upstream lb scopes

- Split the master into the full sub-scope set: `care/{care-authz,enrollment-invites,
  attendance,daily-feed,menus,messaging}-scope.md`, `ui/mobile-shell-scope.md`,
  `billing/billing-scope.md`; rewrote `scope/README.md` as the topic table; added the
  "Scope map" (build order) to the master.
- Wrote the five upstream lb scopes (entity-scoped-grants, invites, media, push-target,
  minimal-shell) in `NubeDev/lb` + indexed them there; lb session:
  `lb/docs/sessions/auth-caps/cc-app-platform-gaps-session.md`.
- Gap ledger corrected: kiosk → covered by lb `api-keys-scope.md`; cap-refresh → narrowed
  into invites/access-console freshness. Master scope §lb gaps updated with statuses.
- New public stubs: `public/ui/ui.md`, `public/billing/billing.md`. STATUS.md moved.

## Follow-ups

- The lb gaps (scope §"lb gaps") each need an upstream lb scope; #1 entity-scoped grants is
  the blocker.
- Resolve open questions: product name/org, PWA vs RN shell, billing provider, photo consent.
- STATUS.md written with the build order (host boot → care slice 1 → shell).
