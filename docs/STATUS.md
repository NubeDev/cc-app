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
- **Upstream lb scopes written** (2026-07-11, in `NubeDev/lb`): `auth-caps/entity-scoped-grants`,
  `auth-caps/invites`, `files/media`, `inbox-outbox/push-target`, `frontend/minimal-shell`
  (+ lb `scope/README.md` indexed). Kiosk = existing lb `api-keys`; cap-freshness folded
  into invites/access-console.
- Session log: `sessions/care/care-scope-session.md`.

## Next up (in order — mirrors the master scope's "Scope map")

1. **Build the lb gaps upstream**: entity-scoped grants (the blocker), then invites,
   media, push-target, minimal-shell — each PR+tag per lb's HOW-TO-CODE, then bump here.
2. **Boot the host** (`rust/node/`): `Cargo.toml` + `BootConfig` fill from `CC_*` env,
   repo-anchored `.cc-app/` state — copy rubix-ai's proven shape.
3. **`care` extension slice 1** (`rust/extensions/care/`): children + guardianship + rooms
   CRUD with the `authz/` chokepoint (era 1) and the cross-family test matrix.
4. **Thin mobile shell** (`ui/`): lb minimal-shell package if shipped, else the ~15-file
   interim host per `scope/ui/mobile-shell-scope.md`.

## Non-goals (unchanged)

- No special-casing of lb or any extension (rule 10).
- No vendored lb UI shell — 100% of the product UI is extension UI behind `defineRemote`.
- No billing/payments in phase 1 (scope §Phases).
