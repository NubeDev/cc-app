# Session — milestone 03 (enrollment — partial, orchestrator schemas + 2 nouns)

**Status:** PARTIAL — orchestrator-owned schemas + 2 nouns (center, room)
+ i18n bootstrap. The remaining nouns (child, guardian, guardianship,
enrollment) ship with the next session.

**Date:** 2026-07-11

**Milestone:** [`../../docs/build/03-enrollment.md`](../../docs/build/03-enrollment.md)

## What landed (this session — a focused bottom-up subset)

### 1. Orchestrator-owned record schemas

The verb-folder shape that the milestone 03 plan calls for. The
orchestrator (this session) fixed the shapes; subagents will extend
verb-per-file in the next session.

- `rust/extensions/care/src/center/records.rs` — `Center` (name,
  address/phone/email, `default_locale`, `archived`) + `Locale` enum
  (en/es launch set, `parse` rejects unknown locales with a typed
  error) + `CenterError` (typed, Display, Error).
- `rust/extensions/care/src/room/records.rs` — `Room` (name, center_id,
  archived) + `RoomError`.

Both files are `pub mod records` barrels re-exported from the verb's
`mod.rs`. Adding new fields is one-file; verb files import the shape.

### 2. Verbs (a focused subset — 5 verbs shipped this session)

Per FILE-LAYOUT §2: one verb per file. Same shape across the nouns:
**validate → resolve → store → audit → reply**.

- `care.center.create` — admin. First-write (`lb_store::create`
  Conflict ⇒ `AlreadyExists`). Validates id length + locale.
- `care.center.get` — admin-wildcard (audit via `assert_reach`). Returns
  the record verbatim (admin gets archived too; the list verb filters).
- `care.center.list` — admin gets the full set; non-admin gets `[]`
  (empty, never error — CLAUDE.md rule 7). Staff/guardian reaches land
  in milestone 03's full follow-up.
- `care.room.create` — admin. First-write. Validates id length.
- `care.room.get` + `care.room.list` — admin wildcard; staff filtered
  to `reachable_rooms` (the chokepoint's staff-resolution path,
  exercised live by `staff_list_filters_to_assigned_rooms`).

Each verb has a unit test in its `#[cfg(test)] mod tests`:
- create (write + duplicate-conflict + invalid-locale)
- get (round-trip + not-found)
- list (admin wildcard + non-admin empty + staff filtered)

That's **13 lib tests**, on top of the 17 from milestones 01+02 =
**30 total**.

### 3. The i18n bootstrap (CLAUDE.md rule 8 / i18n-scope §Enforcement)

- `i18n/en.json` + `i18n/es.json` — the launch catalogs (23 leaf
  keys, parity-checked). The schema is "section → key" with optional
  `{{name}}` interpolation slots. Records store KEYS (the enum-ish
  `kind: "linked"`), not translated strings — the catalog lookup
  happens at the edge (UI / outbox template) per i18n-scope §"the rule".
- `scripts/check-i18n-parity.sh` — the **catalog completeness gate**.
  Walks every JSON catalog in `i18n/`, asserts every leaf key is in
  every catalog, asserts `_meta.locale` matches the filename. **A key
  added in one catalog without the other fails the build.** Wired into
  `.github/workflows/ci.yml`.
- `scripts/check-hardcoded-strings.sh` — the **no-literal guard**.
  Flags `format!` / `println!` / `eprintln!` / `print!` / `panic!`
  calls that embed a quoted string literal (the actual user-facing
  surface). **Emits warnings today** (exit 0) because the catalog
  wire-up is a milestone 03 follow-up — after that, the lint flips to
  `exit 1` and the gate becomes hard.

### 4. SurrealDB query pattern (reusable across verbs)

The `SELECT * FROM <table>` + `(0, "data")` field-access take
pattern is the canonical "list every row" idiom for the verb files.
Captured in `center/list.rs` and `room/list.rs`; future verbs reuse it
verbatim. The earlier debug (`Serialization("invalid type: enum...")`)
is recorded in the file's commit history (when commits resume after
the sandbox) — taking by `(0usize, &str "data")` shape is the right
form for the SurrealDB Row envelope.

## What's pending (the rest of milestone 03)

- `care.child.*` (create / update / get / list / archive — archive
  never delete; full safety data: DOB, allergies, immunizations,
  photo-consent flag).
- `care.guardian.*` (create / get / list — pre-account records, the
  `sub` field binds once invite.accept lands in milestone 05).
- `care.guardianship.link / unlink / update` (the edge verb family;
  carries the 5 per-edge flags: `can_pickup`, `receives_daily_feed`,
  `receives_billing`, `emergency_contact`, `custody_notes`). The
  **era-2 grant derivation** lives here — when the matrix harness's
  `unlink_immediately_denies` test needs to fire the wall denial, the
  link/unlink handlers derive the scoped grants in the same
  transaction.
- `care.enrollment.*` (create / update / list + waitlist ordering).
- `care.enrollment.import` — the CSV job. Out of scope this session
  per the milestone's explicit "skip for now: import job UI polish".
- The admin UI slice — explicitly deferred (milestone 04's mobile-
  shell work, depends on lb's minimal-shell landing).
- The i18n **catalog wire-up**: a `t(locale, key, vars)` helper the
  verb bodies call instead of `format!("hardcoded English...")`. The
  lint flips from warning to hard failure once this lands.

## Open questions (resolved this session)

- *"Authorized-pickup persons who are not guardians"* (enrollment-
  invites-scope) — deferred to the child record schema (a separate
  `authorized_pickup` field, not a first-class contact record). v1
  recommended default; recorded.
- *"Waitlist: FIFO per room v1 (recommended) or priority tiers?"* —
  FIFO. Recorded.

## Open questions (carried to the next session)

- *"Import is where garbage enters"* (enrollment-invites-scope §"Risks
  & hard problems") — the import job's per-item validation (hard-fail
  on medical fields) needs the `lb/jobs` integration. Tracking.
- The i18n helper `t(locale, key, vars)` shape and where it lives
  (the care extension re-implements it OR consumes lb's multi-lang
  seam). The next-session work.

## Green output (paste)

### `cargo test --workspace`

```
test result: ok. 16 passed; 0 failed   (care lib: ping + center + room)
test result: ok. 0  passed; 0 failed
test result: ok. 4  passed; 0 failed   (matrix_care_ping.rs)
test result: ok. 8  passed; 0 failed   (matrix_chokepoint.rs)
test result: ok. 0  passed; 0 failed
test result: ok. 2  passed; 0 failed   (cc-node boot_test.rs)
test result: ok. 0  passed; 0 failed
---
TOTAL: 30 passed, 0 failed
```

### `cargo clippy -p care --lib -- -D warnings`

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.81s
```

(clean; no warnings)

### `scripts/check-file-size.sh --all`

```
FILE-LAYOUT: all source files within 400 lines (69 checked)
```

### `scripts/check-authz-fence.sh --all`

```
AUTHZ-FENCE: 22 files checked, no "guardianship" reads outside authz/
```

### `scripts/check-i18n-parity.sh`

```
i18n: 2 catalogs, 23 leaf keys, parity OK
```

### `scripts/check-hardcoded-strings.sh`

```
::warning::7 file(s) embed a raw user-facing string in a macro — route through i18n/<locale>.json (CLAUDE.md rule 8); catalog wire-up pending
```

(warning today; the catalog wire-up turns this into a hard gate)

## Follow-ups (clear runway for the next session)

1. Add `child/`, `guardian/`, `guardianship/`, `enrollment/` record
   shapes (orchestrator-owned) + their verb-per-file CRUD.
2. Wire the i18n `t(locale, key, vars)` helper into the verb bodies
   (catalog lookups for the user-facing error messages). Flip the
   hardcoded-string lint to `exit 1` once green.
3. The `care.enrollment.import` job — `lb/jobs` integration.
4. Update `COVERED_VERBS` in `tests/matrix_care_ping.rs` with each new
   verb (the guard asserts it).
5. The session-doc note for the era-2 grant derivation: link/
   unlink handlers derive scoped grants in the same transaction.

## Sandbox caveat (still in effect)

`.git` is bind-mounted read-only — the milestone-03 work is on disk
in uncommitted modifications, alongside milestones 01 and 02. The
gates above (tests / clippy / fmt / file-size / authz-fence / i18n-
parity) all run green from a fresh `cargo test --workspace` + the
scripts; re-running from a non-sandbox checkout will land the same
result and let the per-milestone commits close the gap.