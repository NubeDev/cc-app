# Session — milestone 03: enrollment (the roster) + era-2 reach

- **Milestone:** [`../../build/03-enrollment.md`](../../build/03-enrollment.md)
- **Scopes:** [`enrollment-invites-scope.md`](../../scope/care/enrollment-invites-scope.md),
  [`care-authz-scope.md`](../../scope/care/care-authz-scope.md)
- **Status:** shipped (2026-07-12). Milestone 03 CLOSED (verbs + era-2 read
  delegation + i18n). Two items explicitly DEFERRED (import → lb/jobs; UI → m04).
- **Tests:** `cargo test --workspace` → **87 passed, 0 failed**, clean from
  git tags alone (no `[patch]`). All four CI fences green.

## What this session did (A → CLOSE)

### A — genuine tag bump to `node-v0.3.0` / `sdk-v0.3.0`

lb released the native host-callback client (`SidecarClient`, re-exported from
`lb-ext-native`). Pinned `lb-node`/`lb-store`/`lb-auth`/`lb-host` → `node-v0.3.0`
and `lb-ext-native` → `sdk-v0.3.0`; DROPPED the `[patch]` block from the
git-ignored `.cargo/config.toml` (kept only the zigcc linker + zig-cache
wiring). `cargo build`/`test --workspace` are clean FROM TAGS ALONE — every lb
crate compiles from `tag=node-v0.3.0`, the sidecar client from `tag=sdk-v0.3.0`.
This closed the "fake-green from a local patch" risk the prior session flagged.

### B — CI fences

Fixed the known authz-fence false-positive: it excluded file paths against the
ABSOLUTE `$AUTHZ_DIR` while the `git ls-files` branch (CI posture) emits
REPO-RELATIVE paths, so `authz/scope.rs` (the blessed chokepoint) was flagged.
Now excludes a repo-relative pattern for both branches. Verified: clean tree →
0; inject a `guardianship` read in a non-authz file → 1; revert → 0. All three
fences (authz, file-size, i18n-parity) green; the fourth (hardcoded-strings) is
flipped hard in Step D.

### C — era-2 (platform-enforced reach)

The chokepoint now delegates reach to lb's entity-scoped grants via the
node-v0.3.0 native host-callback, with call sites UNCHANGED:
- `authz/host_callback.rs` (`ReachClient`) → `authz.check_scoped` /
  `authz.scope_filter`.
- `authz/caps.rs` — the one orchestrator-owned `REACH_CAP`
  (`mcp:care.reach.child:call`) + `REACH_TABLE` the derive + read paths share.
- `authz/grant.rs` — `derive_reach`/`remove_reach` over `grants.assign` /
  `grants.revoke`, called transactionally from `guardianship.link/unlink/update`
  (edge AND grant, or neither; a failed rollback surfaces the divergence via a
  typed `GrantDerivationDiverged` error, never a silent swallow).
- `Chokepoint::with_host_callback` carries an optional `ReachClient`;
  `assert_reach`/`reachable_children` delegate when present, else fall back to
  era-1 store resolution. The era-2 stub in `scope.rs` is deleted.

**Proven live (read path):** `tests/matrix_era2.rs` boots a REAL `lb-role-gateway`
on a real TCP port and drives the chokepoint's `assert_reach` /
`reachable_children` through a REAL `SidecarClient` over HTTP — grant→reach,
cross-family deny, revoke→**grant-physically-gone** (asserted via
`scope_filter` returning no ids, not merely a denied read), workspace
isolation. No mocks; scoped grants seeded via lb's real in-process write path.

**Tracked lb gap (blocks the era-2 WRITE path):** lb's `/mcp/call` dispatcher
routes only `authz.*` to the authz verbs — NOT `grants.*`/`roles.*`/`teams.*`
— so a native extension can *read* the scoped-grant surface but cannot *mint* a
grant over the callback. The care derivation code is wired-and-correct against
the verb contract; it goes live the moment lb routes those verbs. **Until then
era-1 (store edges) is the LIVE reach path** — exactly `care-authz-scope.md`'s
"keep era-1 as the documented fallback if lb's verbs aren't reachable." Filed:
[`../../debugging/authz/grants-verbs-not-on-mcp-callback-surface.md`](../../debugging/authz/grants-verbs-not-on-mcp-callback-surface.md).
This is the milestone-03 → lb follow-up (a one-arm additive fix in
`tool_call.rs`, then a `node-v*` tag + a pin bump here — WORKFLOW-LB.md).

### D — milestone 03 verbs + i18n

Orchestrator owned every schema first (`records.rs` per noun); verbs fanned to
subagents (guardian, enrollment) + written directly (child, guardianship),
each verb-per-file (≤400 lines) with real-store unit tests:
- `care.child.create|update|get|list|archive` — safety data (DOB validated
  hard, allergies, medical, immunizations, emergency contacts, authorized
  pickups, photo-consent); archive not delete (archived invisible to
  guardians, admin-recoverable); reads reach-filtered (rule 7).
- `care.guardian.create|get|list` — records-before-accounts, `locale`
  pre-account (invites need it in m05).
- `care.guardianship.link|unlink|update` — relationship + the five edge flags
  (`can_pickup`, `receives_daily_feed`, `receives_billing`,
  `emergency_contact`, `custody_notes`); the era-2 grant derivation lives here.
- `care.enrollment.create|update|list` — `waitlist|enrolled|withdrawn` +
  waitlist FIFO per room (monotonic `waitlist_seq`, stable across withdrawals).
- `i18n/catalog.rs` — `t(locale, key, vars)` resolves en/es from the embedded
  repo-root `i18n/*.json`, interpolates `{{var}}`, en fallback. Shipped verbs'
  user-facing strings flow through it; `scripts/check-hardcoded-strings.sh`
  flipped to `exit 1` (refined to exempt developer-facing error context + the
  i18n engine, so it blocks only genuine chrome — verified it still catches an
  injected user sentence).

### Adversarial review + fixes

An adversarial reviewer swept for chokepoint bypasses / missing matrix rows.
Findings actioned:
- **HIGH (fixed):** `child.list` had (a) a `take()`-consumed-twice bug that
  dropped every id and (b) a `leo` vs `child:leo` id-form mismatch between the
  reach set and record ids — a latent lockout that could flip to a leak, with
  NO allow-case test. Fixed: non-admin list now reads the reached ids directly
  (records keyed in the edge's `child_id` form), and `matrix_child_reads.rs`
  adds guardian ALLOW cases through the `child.get`/`child.list` bodies on the
  two-family fixture.
- **HIGH (fixed):** link/unlink/update rollback-of-rollback was a silent
  `let _ = …`; now surfaces the edge/grant divergence via a typed error.
- **MEDIUM (noted):** center/room `get` reach-checks via `assert_reach` on a
  center/room id (fail-closed but semantically a child-edge check) — pre-existing
  shipped code; staff center/room scoping is a documented later slice.
- **MEDIUM (addressed for the rule-7 surface):** child reads have cross-family
  allow+deny rows; guardian/enrollment/center/room reads are admin-only this
  milestone (non-admin → empty), covered by their admin-all + non-admin-empty
  unit tests — no guardian reaches them, so the rule-7 cross-family concern
  (guardian-reachable child records) is the one with dedicated matrix rows.

## Test output (paste)

```
cargo build --workspace   -> Finished (all lb crates from tag=node-v0.3.0)
cargo test  --workspace   -> 87 passed, 0 failed
  care lib                16 + 50 = 66   (schemas, verb bodies, i18n)
  matrix_care_ping         4
  matrix_child_reads       5   (cross-family allow + deny through verb bodies)
  matrix_chokepoint        8
  matrix_era2              2   (era-2 read delegation over a real gateway)
  boot_test                2
```

All four fences: `check-authz-fence.sh` 0, `check-file-size.sh` 0,
`check-i18n-parity.sh` 0 (33 leaf keys, en/es parity), `check-hardcoded-strings.sh` 0 (now hard).

## Deferred (tracked, not this session)

- **`care.enrollment.import`** — the lb/jobs CSV integration (children +
  guardians + edges, per-item results, idempotent on natural keys). A milestone
  03 exit-gate line, deferred by the session brief; it lands when the roster
  UI/import need it. TODO carried here.
- **Admin UI** (Centers/Rooms, child editor, family/edges editor, waitlist,
  import) — milestone 04 (mobile-shell), awaiting the shipped minimal-shell.
- **Era-2 grant derivation over the callback** — blocked on the lb `grants.*`
  dispatcher-routing fix (debug entry above); era-1 is the live path meanwhile.
