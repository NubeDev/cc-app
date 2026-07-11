# BUILD — the execution runbook

This directory turns the scopes into shipped code. It is written to be **handed to a fresh
session** (human or agent) that has never seen this repo: read this file, pick the first
milestone whose entry gate is open, execute it, close its exit gate, update STATUS, stop.
Repeat until [10-hardening-launch.md](10-hardening-launch.md) is closed.

**A milestone doc never restates the domain** — the scopes
([`../scope/`](../scope/README.md)) stay the source of truth for verbs, data, and tests;
milestone docs sequence the work and pin the gates.

## Read before executing anything

1. [`../../CLAUDE.md`](../../CLAUDE.md) — the binding rules (rule 10, no mocks, rule 7).
2. [`../STATUS.md`](../STATUS.md) — where the project actually is (trust it over this dir
   if they disagree — then fix the disagreement).
3. [`../WORKFLOW-LB.md`](../WORKFLOW-LB.md) — which repo owns a change; `[patch]` dev;
   PR→tag→bump.
4. [`../HOW-TO-CODE.md`](../HOW-TO-CODE.md) + [`../FILE-LAYOUT.md`](../FILE-LAYOUT.md) —
   the per-session procedure and file rules. **Every milestone session follows
   HOW-TO-CODE's procedure and definition-of-done**; this runbook only adds ordering.

## Milestone map (dependency order)

```
00 lb-release ──► 01 host-boot ──► 02 care-skeleton-authz ──► 03 enrollment ──► 05 invites-golden-path
                        │                                          │                    │
                        └────────► 04 mobile-shell ────────────────┘                    │
                                                                                        ▼
                                              06 attendance ─┐   ┌─ (06 ∥ 07 parallel) │
                                              07 menus ──────┤   │                      │
                                                             ▼   ▼                      │
                                                        08 daily-feed ──► 09 messaging ─┴─► 10 hardening-launch
```

| # | Milestone | Repo(s) | Gate it opens |
|---|---|---|---|
| 00 | [lb-release](00-lb-release.md) | `NubeDev/lb` | released tags cc-app can pin |
| 01 | [host-boot](01-host-boot.md) | this repo `rust/node/` | a booting node |
| 02 | [care-skeleton-authz](02-care-skeleton-authz.md) | `rust/extensions/care/` | the authz chokepoint + matrix harness |
| 03 | [enrollment](03-enrollment.md) | care ext | the roster (children/guardians/edges/rooms) |
| 04 | [mobile-shell](04-mobile-shell.md) | `ui/` | login → mounted ext on a phone |
| 05 | [invites-golden-path](05-invites-golden-path.md) | care ext + `ui/` | **the E2E gate: invite→accept→feed** |
| 06 | [attendance](06-attendance.md) | care ext | check-in/out + kiosk + ratios |
| 07 | [menus](07-menus.md) | care ext | plans + derived substitutions |
| 08 | [daily-feed](08-daily-feed.md) | care ext | logs, photos, SSE, push |
| 09 | [messaging](09-messaging.md) | care ext | provisioned channels + reconciler |
| 10 | [hardening-launch](10-hardening-launch.md) | all | phase-1 ship |

**06 ∥ 07 may run in parallel** (independent scopes). Everything else is sequential at the
milestone level; parallelism lives *inside* milestones (below).
**Billing is not on this map — deliberately.** It is deferred to last
([`../scope/billing/billing-scope.md`](../scope/billing/billing-scope.md)); do not start it.

## Execution model (sessions + subagents)

- **One milestone = one session** (or a few, for 08). The session is the orchestrator: it
  reads the milestone doc + its scopes, fans work out to subagents, integrates, runs the
  exit gate itself. Never let a subagent claim a gate closed — the orchestrator verifies.
- **Every subagent prompt must carry the rules preamble:** the paths of `CLAUDE.md`,
  `docs/FILE-LAYOUT.md`, the owning scope doc(s), and the exact files/verbs it owns.
  Subagents inherit no context — repeat the constraints (no mocks; verb-per-file ≤400
  lines; guardian reads only via `authz/`; never hand-write `remoteEntry`).
- **Good fan-out shapes:** one subagent per verb-file + its tests (after the orchestrator
  fixes the record shapes and the `authz/` API); one per UI screen; one adversarial
  reviewer per milestone hunting cross-family leaks and chokepoint bypasses
  (`grep` rule: `guardianship` read outside `authz/` = finding).
- **Bad fan-out shapes:** parallel agents editing the same file; a subagent deciding a
  record schema or a cap name alone; UI before its verbs exist.
- **Cross-repo work** (milestone 00, and any lb gap discovered later): follow
  `WORKFLOW-LB.md` — prove locally with `[patch]`, release upstream, bump, **never commit
  a path/patch**.

## Per-milestone ritual (compresses HOW-TO-CODE §3)

1. Confirm the **entry gate** of the milestone doc. Not open → do the blocking milestone.
2. Open `sessions/<topic>/<milestone>-session.md` (status `in-progress`).
3. Execute the work items — vertical slices, whole contract, deny-test per verb.
4. Run the **exit gate** checklist yourself; paste green output in the session doc.
5. Tick the milestone's checkboxes in its doc, update `STATUS.md`, resolve/refresh scope
   open questions, promote shipped docs. HOW-TO-CODE §4 is the definition of done.
6. If anything broke non-trivially: `debugging/<area>/<symptom>.md`.

## Non-negotiables the whole build hangs on

- **English + Spanish from day one (CLAUDE.md rule 8,
  [`../scope/ui/i18n-scope.md`](../scope/ui/i18n-scope.md)) — a phase-1 MUST.** Every
  milestone that ships a user-facing surface (03 onward) exits only if that surface
  renders 100% in `en` **and** `es`: no hardcoded strings, catalogs complete (CI gate),
  and the milestone's E2E run once with an `es`-locale user. Invite emails (05) and push
  bodies (08) are localized server-side. Retrofitting i18n is misery — it is cheaper at
  every milestone than at the end, which is why it gates each one.
- **UI stack is pinned (CLAUDE.md rule 9):** shadcn/ui components only; mobile-first
  (360px) **and** laptop-good (~1280px); dark + light mode (system default, persisted
  toggle, host-owned `.dark` variable swap, semantic tokens only in ext UI). Every UI
  milestone's Playwright pass runs **both viewports and both themes**; a hardcoded color
  or a non-shadcn widget for something shadcn ships fails review. **Design language:
  modern iOS** — root `PRODUCT.md` + `DESIGN.md` are the binding contracts. UI work runs
  through the **impeccable skill**: `/impeccable craft <screen>` to build,
  `/impeccable critique` + `polish` on the milestone's screens **before** its UI exit
  gate is claimed (it auto-loads PRODUCT.md/DESIGN.md; re-run
  `/impeccable document` at milestone 04 to replace the seed DESIGN.md with real
  tokens).
- **Rule 7 (guardian isolation) is enforced from milestone 02 on** — no care verb ever
  lands without its cross-family matrix row. A verb without a matrix row fails the gate.
- **Open questions get resolved at the milestone that hits them**, recorded in the scope
  doc (each milestone doc lists which ones it must close). Recommended defaults are in the
  scopes — take them unless there's a reason not to; log the decision either way.
- Tests on real infra (`mem://` store, real bus, real gateway, real booted extensions) —
  the only sanctioned fakes: EmailProvider / PushProvider / (later) PaymentProvider.
- If lb is missing something: **stop, fix lb generically, tag, bump** — never a care-side
  workaround (rule 10). The persona use-case docs ([`../scope/personas/`](../scope/personas/README.md))
  are the acceptance checklist for what "done" feels like per user.
