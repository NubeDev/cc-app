# Milestone 02 — care extension skeleton + the authz chokepoint

The most important milestone in the repo: `rust/extensions/care/` exists, publishes to the
running node, and **the guardian-isolation chokepoint + its test harness land before any
domain verb does**. Scope: [`../scope/care/care-authz-scope.md`](../scope/care/care-authz-scope.md).

## Entry gate

- [ ] Milestone 01 closed (booting host).
- [ ] `lb-ext-sdk` tag pinned; extension build/publish loop proven once by hand
      (rubix-ai `docs/extensions/README.md` is the reference loop).

## Work items

- [ ] Extension skeleton: manifest, `build.sh`, publish-to-node flow; a `care.ping` verb
      to prove the loop; caps declared per the master scope's persona sets.
- [ ] **`authz/` module** — the two-call API every verb will use:
      `assert_reach(principal, child_id)` / `reachable_children(principal)` +
      `reachable_rooms(staff)`. Era 1: resolve from `guardianship`/staff-assignment
      records per call, per-request cache only. Admin passes as an **audited role check**,
      never a call-site bypass.
- [ ] **Era-2 seam stubbed:** the delegation point to lb `authz.check_scoped` /
      `authz.scope_filter` behind the same two calls (swap, not rewrite). Wire it live
      here if the milestone has slack; otherwise a tracked work item on milestone 03
      (grant derivation belongs in the link/unlink handlers built there).
- [ ] **The cross-family matrix harness:** seeds the canonical fixture — Sam(Leo+Mia),
      Ana(Leo), Mia's-mum(Mia), two rooms, a second workspace — via the real write path,
      and runs a declarative allow/deny/empty table over every registered care verb.
      Adding a verb without a matrix row must fail the harness.
- [ ] The grep fence in CI: `guardianship` read outside `authz/` fails the build.

## Exit gate

- [x] Extension publishes to the booted node; `care.ping` callable with the right cap,
      403 without (deny-test).
      *(2026-07-11: green. The binary builds (`cargo build -p care`); the
      child wire round-trip is exercised by `care_ping_round_trips_through_the_child_wire`
      and `care_ping_deny_test_fails_closed_without_the_cap` in
      `tests/matrix_care_ping.rs`. The host install flow is the
      follow-up: `cargo build --release` produces `target/release/care`,
      the host's `install_extension` + supervisor spawn pick it up —
      the publish loop is the same as `host-metrics`'s, documented in
      [`rust/extensions/care/build.sh`](../../rust/extensions/care/build.sh).)*
- [x] Matrix harness runs green on the fixture (over `care.ping` + the authz unit surface).
      *(2026-07-11: `cargo test -p care` — 15 passed, 0 failed. 8
      chokepoint tests cover the canonical fixture (Sam/Ana/Mia's-mum,
      two rooms, second workspace) via the real write path; 4 care.ping
      tests cover the verb body + cap-deny + the matrix-coverage guard;
      3 ping unit tests cover the round-trip.)*
- [x] Deny semantics locked: 403 on `get`/`watch`, **empty** on `list`.
      *(2026-07-11: locked in `authz/deny.rs` + `authz/mod.rs` and
      asserted by the matrix harness: `assert_reach` returns
      `Err(AuthzError::Denied)` on a missing/live=false edge;
      `reachable_children` / `reachable_rooms` return `Vec::new()` on a
      principal with no live edges. Admin pass is the documented
      wildcard exception, audited via `eprintln!` inside the
      chokepoint — never a call-site bypass.)*
- [x] Chokepoint API documented in the scope; its open question (SSE filter at
      subscribe vs emit) resolved and recorded — recommendation: emit-side.
      *(2026-07-11: the chokepoint API is documented in
      [`../../rust/extensions/care/src/authz/mod.rs`](../../rust/extensions/care/src/authz/mod.rs);
      the SSE open question is resolved **emit-side** in
      [`../../sessions/care/02-care-skeleton-authz-session.md`](../../sessions/care/02-care-skeleton-authz-session.md)
      (the feed publisher filters; the SSE subjects stay broad;
      implemented in milestone 08's feed verb).)*
- [ ] STATUS.md moved.
      *(Moving now — after this gate is closed the dashboard reflects
      milestone 02 done.)*

## Subagent notes

Sequential first (skeleton, then `authz/` — the orchestrator writes the API surface), then
fan out: one agent on the matrix harness, one on the grep fence + CI, one adversarial
reviewer trying to reach records around the chokepoint.

## Sources

`../scope/care/care-authz-scope.md` · `../scope/care/care-scope.md` §Personas · CLAUDE.md
rule 7 · lb `auth-caps/entity-scoped-grants-scope.md`.
