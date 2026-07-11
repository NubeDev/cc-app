# Scope docs — the ask, before any work

One file per feature/ask, under a topic subfolder: `scope/<topic>/<name>-scope.md`.
Write it by following [`../SCOPE-WRITTING.md`](../SCOPE-WRITTING.md). Then build it by
following [`../HOW-TO-CODE.md`](../HOW-TO-CODE.md). A scope here often describes work that
actually lands **upstream in lb** — name the owning repo (see [`../WORKFLOW-LB.md`](../WORKFLOW-LB.md)).

Start at **[`care/care-scope.md`](care/care-scope.md)** — the product master scope; its
"Scope map" section is the build order across everything below.

## Topics

| Topic / scope | What it covers | Owning repo / upstream dependency |
|---|---|---|
| [`care/care-scope.md`](care/care-scope.md) | **The product master scope**: tenancy (workspace = organization), the family model (guardianship edges), personas/caps, phases, and the lb-gaps ledger | this repo; gaps land in `NubeDev/lb` (each now has a written lb scope — see the master's §lb gaps) |
| [`care/care-authz-scope.md`](care/care-authz-scope.md) | The guardian/staff **reach chokepoint** (`authz/`): ext-enforced now, lb entity-scoped grants when shipped; the cross-family test matrix | this repo ← lb `auth-caps/entity-scoped-grants-scope.md` |
| [`care/enrollment-invites-scope.md`](care/enrollment-invites-scope.md) | Children, guardians, edges, rooms, waitlist, invites, CSV import job | this repo ← lb `auth-caps/invites-scope.md` (**must ship first**) |
| [`care/attendance-scope.md`](care/attendance-scope.md) | Check-in/out ledger, pickup authorization, kiosk devices, ratios | this repo ← lb `auth-caps/api-keys-scope.md` |
| [`care/daily-feed-scope.md`](care/daily-feed-scope.md) | Typed daily logs, photos, live SSE feed, push policy | this repo ← lb `files/media-scope.md` + `inbox-outbox/push-target-scope.md` (**must ship first**) |
| [`care/menus-scope.md`](care/menus-scope.md) | Menu plans, allergen tags, **derived substitutions** (safety) | this repo (no upstream dep) |
| [`care/messaging-scope.md`](care/messaging-scope.md) | Channel provisioning + derived membership over shipped lb channels | this repo (possible small additive lb ask: read-only channel membership) |
| [`ui/mobile-shell-scope.md`](ui/mobile-shell-scope.md) | The thin shell instance + the care extension's three persona surfaces (100% ext UI) | this repo ← lb `frontend/minimal-shell-scope.md` (preferred path) |
| [`ui/i18n-scope.md`](ui/i18n-scope.md) | **MUST, phase 1:** English + Spanish 100% from day one — UI, invite emails, push, domain text; CI catalog gate | this repo ← lb multi-lang seam (verify coverage; holes = lb gaps) |
| [`personas/`](personas/README.md) | **The per-user layer**: one folder per persona (`admin/`, `teacher/`, `guardian/`), one doc per use case — journeys over the feature scopes, never a second domain model | this repo (docs only) |
| [`billing/billing-scope.md`](billing/billing-scope.md) | **DEFERRED — build LAST.** Phase 2: households, invoices, provider-behind-a-trait. Kept only so phase-1 data decisions don't preclude it (edge flags, schedules, no implicit families) — no billing work before every phase-1 use case ships | this repo (own extension `care-billing`, later) |
