# Session — milestones 06 (attendance) + 07 (menus)

Status: in-progress → CLOSING. Both milestones executed in one session as
parallel vertical slices (independent scopes, no upstream lb dependency — the
build map marks 06 ∥ 07 parallel).

## What shipped

### Milestone 07 — menus & derived substitutions (food safety)

Orchestrator-owned safety core (heavily unit-tested before any verb):

- `menu/allergen.rs` — the fixed top-9 allergen enum + slot enum, with
  alias/case folding (`dairy`→milk, `gluten`→wheat) and an `Other(label)`
  escape that is never dropped. Records store keys, catalogs store words
  (rule 8): `allergen.<key>` / `slot.<key>` in en+es.
- `menu/records.rs` — the `menu` record (`date × room × slot`, items with
  allergen tags, per-restriction substitutes), id = `date::room::slot`
  (deterministic; `set` upserts on the natural key, copy-week idempotent).
- `menu/derive.rs` — **the substitution derivation, the one file that
  matters**. Three false-negative guards: (1) intersection on the canonical
  allergen KEY (alias/case folded on both sides), (2) an untaggable item
  (no tags) flags conservatively for any child with any allergy — never a
  silent safe, (3) an unresolved substitution (matched allergen, no substitute)
  is loud (`resolved=false`).

Verbs (verb-per-file ≤400 lines, in-file real-store tests):
`menu.set` (upsert, admin/staff), `menu.get` (room-scoped, staff sees only
assigned rooms), `menu.week` (**the guardian rule-7 medical-leak surface** —
`assert_reach` FIRST, returns only the asking child's derived rows, never the
room's other-child data), `menu.copy_week` (idempotent, month-boundary-correct
date math).

### Milestone 06 — attendance (ledger, pickup gate, kiosk, ratios)

Orchestrator-owned cores:

- `attendance/records.rs` — the append-only `attendance_event` ledger
  (`check_in`/`check_out`, child OR staff subject, `correction_of` for
  compensating corrections) + the `PickupDeny` reason ENUM (localizable:
  `attendance.deny.<key>` in en+es — a Spanish teacher reads WHY the gate
  refused).
- `attendance/gate.rs` — **the pickup gate, a child-safety control**. Pure
  decision: custody-hold denies FIRST (strongest), then a can_pickup guardian
  (by sub) or an authorized-pickup/guardian NAME is allowed; anything else is a
  hard deny with a reason. Fail-closed; an empty collector name never matches
  an empty roster slot.
- `attendance/occupancy.rs` — the derived `now` fold (last-event-wins per
  subject in time order, correction-aware; children/staff/ratio per room; a
  zero-staff room yields `ratio=None`, an alert, not a divide-by-zero).
- `authz/pickup.rs` — resolves the `can_pickup`/`custody_notes` facts from the
  live `guardianship` edges BEHIND the authz fence (rule 7 / rule 10), so the
  gate stays pure and no verb reads `guardianship` directly. Fail-closed on a
  store error (empty roster → deny).

Verbs: `attendance.check_in` (staff/kiosk/staff-presence append),
`attendance.check_out` (**runs the pickup gate**; admin-capped audited override;
localized deny), `attendance.list` (rule-7 reach-filtered: guardian own-children
only, staff room-scoped), `attendance.now` (room-scoped derived read),
`attendance.correct` (append-only compensating event; original untouched).

Kiosk: an lb API key (machine principal, `key:` subject — lb api-keys verified
shipped: `admin_apikeys.rs` + `key:` subject in the native wire + `key:`
handled by `authz::canonical_subject`) granted exactly the two check verbs;
guardian-PIN self-check-in is phase 1.5 (deferred, recorded).

## Rule 7 gates (the sacred invariant)

Every new READ verb ships a cross-family deny-test:
- In-file: `menu::week` (Ana denied Mia), `attendance::list` (guardian sees only
  reached children), plus the gate/derivation/occupancy safety unit tests.
- Integration matrices:
  - `tests/matrix_menu_reads.rs` (4) — Ana sees Leo's peanut substitution, is
    denied Mia's week, her Leo read never contains egg (Mia's restriction).
  - `tests/matrix_attendance.rs` (7) — list scoping + the full pickup deny
    sweep (stranger hard-denied no-event, authorized-by-name allowed,
    non-can_pickup denied, admin override audited, now in/out, staff room-scoped).

The `check-authz-fence.sh` grep fence stays green — no `guardianship` read
outside `authz/` (the pickup resolver lives in `authz/pickup.rs`).

## UI (shadcn, semantic tokens, en+es, modern iOS)

- Menus: guardian week view (per-child, unresolved substitutions loud/red),
  admin week × slot planner (copy-week, allergen-tag chips, substitute entry),
  staff serving view (red allergen flags + substitutes).
- Attendance: staff/kiosk room roster (two-tap check-in/out; pickup sheet;
  unmissable deny banner + admin-only override), admin occupancy dashboard
  (children/staff/ratio per room, no-staff/ratio warnings).

## Exit-gate output

```
# care lib + integration (run per-package; the box OOMs on --workspace at jobs=4)
cargo test -p care --lib      → 160 passed; 0 failed
cargo test -p care --tests    → matrix_attendance 7, matrix_menu_reads 4,
                                 matrix_chokepoint 8, matrix_child_reads 5,
                                 matrix_era2 3, matrix_era2_write 4,
                                 matrix_care_ping 4 (coverage guard incl. all
                                 9 new verbs), live_wire 4 — all green
cargo test -p cc-node         → 2 passed; 0 failed

# fences
scripts/check-authz-fence.sh       → 58 files, no guardianship reads outside authz/
scripts/check-file-size.sh         → all source files within 400 lines
scripts/check-hardcoded-strings.sh → no raw user-facing strings in macros
scripts/check-i18n-parity.sh       → 2 catalogs, 62 leaf keys, parity OK
node scripts/i18n-check.mjs (ui/ + care ui/) → i18n gate OK

# UI
npx tsc --noEmit (care ui/)  → 0 errors
npx vite build (care ui/)    → dist/remoteEntry.js 329.71 kB, built OK

# live e2e on a real seeded node (make dev + make seed)
make e2e-ui → 8 passed (login, email-login, ext-mount, care-menus-attendance
              incl. the es-locale admin flow)
```

## Adversarial review (one reviewer subagent per the runbook)

Ran a rule-7 / pickup-bypass / derivation-miss adversarial pass. Three real
findings, all FIXED + tested:
- **CRITICAL** — the derivation silently missed non-canonical allergy spellings
  (`"peanuts"` plural → `Other`, no match with a `peanut` tag). Fixed:
  `Allergen::parse` now folds plurals/phrasing; a child allergy that stays
  `other:*` forces a conservative per-item flag (guard 3). Its own create test
  even seeded `"peanuts"` — a live miss.
- **HIGH** — `authz/pickup.rs` skipped an undecodable edge via `continue`,
  dropping a custody hold (fail-open). Fixed: a bad edge now forces the hold.
- **LOW** — `menu.set`/`copy_week` didn't room-scope staff writes. Fixed with
  `reachable_rooms`, mirroring `menu.get`.

The authz fence was also tightened to flag guardianship READS only (not
test-fixture seeds) — a `store_create` seed is not the leak the rule guards.

## Deferred

- Guardian-PIN self-check-in at the kiosk → phase 1.5 (attendance-scope open
  question; recorded, not built).
- Menu templates/rotations (4-week cycle) → copy-week only in v1 (scope default).
- Dietary-preference (halal/veg) as a parallel non-safety tag → the allergen
  enum's `Other(label)` covers free-text v1; a dedicated non-safety tag lane is
  a later slice.
