//! THE MATRIX-SWEEP COMPLETENESS GATE (milestone 10,
//! `docs/build/10-hardening-launch.md` §"Full matrix sweep": "every registered
//! `care.*` verb has its cross-family matrix row … the harness enumerates and
//! fails on gaps").
//!
//! Every other `matrix_*.rs` proves a SLICE of the cross-family invariant (rule
//! 7). This test proves the SET is COMPLETE: it enumerates `care::call::TOOLS` —
//! the whole served verb surface, the one contract — and asserts EVERY verb is
//! declared in the [`COVERAGE`] table below with the test that exercises its
//! cross-family / capability deny. A newly-added verb that ships without a matrix
//! row fails THIS test at CI time, before it can leak.
//!
//! ## The two coverage kinds (rule 7 has two shapes)
//!
//! - [`Coverage::GuardianRead`] — a reach-gated READ (the leak surface). Its row
//!   asserts a STRANGER guardian is denied (403 on a `get`, EMPTY on a `list`) and
//!   a LINKED guardian reaches only their own child. This is the sacred set.
//! - [`Coverage::AdminWrite`] — a capability-gated WRITE/admin verb with no
//!   guardian reach (creates, edits, links, invites, kiosk ledger). Its deny is a
//!   capability deny (a guardian holds no cap; the wall 403s) — proven by the
//!   chokepoint/era-2 suites + the per-milestone matrix files.
//!
//! Classifying a verb is a DELIBERATE act: adding a verb to `TOOLS` forces a line
//! here, and the author must state which kind it is and where its deny lives. That
//! is the gate — not a heuristic that could silently mis-bucket a new read verb as
//! a write and skip its cross-family row.

use care::call::TOOLS;
use std::collections::BTreeMap;

/// What kind of cross-family guarantee a verb carries, and therefore what its
/// matrix coverage must prove.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Coverage {
    /// A reach-gated guardian-visible READ — the sacred leak surface. MUST have a
    /// cross-family deny row (stranger denied, linked reaches only own child).
    GuardianRead,
    /// A capability-gated admin/staff/kiosk WRITE with no guardian reach — deny is
    /// a capability deny (guardian holds no cap).
    AdminWrite,
}

/// The coverage declaration for EVERY served verb: `verb → (kind, covering test)`.
/// This map's key set MUST equal `TOOLS` exactly (asserted below) — a verb in
/// `TOOLS` but absent here, or here but absent from `TOOLS`, fails the gate.
fn coverage() -> BTreeMap<&'static str, (Coverage, &'static str)> {
    use Coverage::*;
    // (verb, kind, the test module that proves its deny)
    let rows: &[(&str, Coverage, &str)] = &[
        ("ping", AdminWrite, "matrix_care_ping"),
        // enrollment roster — admin writes.
        ("center.create", AdminWrite, "matrix_chokepoint"),
        ("center.get", GuardianRead, "matrix_chokepoint"),
        ("center.list", GuardianRead, "matrix_chokepoint"),
        ("room.create", AdminWrite, "matrix_chokepoint"),
        ("room.get", GuardianRead, "matrix_chokepoint"),
        ("room.list", GuardianRead, "matrix_chokepoint"),
        ("child.create", AdminWrite, "matrix_child_reads"),
        ("child.get", GuardianRead, "matrix_child_reads"),
        ("child.list", GuardianRead, "matrix_child_reads"),
        ("child.update", AdminWrite, "matrix_child_reads"),
        ("child.archive", AdminWrite, "matrix_child_reads"),
        ("guardian.create", AdminWrite, "matrix_chokepoint"),
        ("guardian.get", GuardianRead, "matrix_chokepoint"),
        ("guardian.list", GuardianRead, "matrix_chokepoint"),
        ("guardianship.link", AdminWrite, "matrix_era2_write"),
        ("guardianship.unlink", AdminWrite, "matrix_edge_change"),
        ("guardianship.update", AdminWrite, "matrix_era2_write"),
        ("enrollment.create", AdminWrite, "matrix_chokepoint"),
        ("enrollment.list", GuardianRead, "matrix_chokepoint"),
        ("enrollment.update", AdminWrite, "matrix_chokepoint"),
        ("invite.create_guardian", AdminWrite, "matrix_chokepoint"),
        ("invite.create_staff", AdminWrite, "matrix_chokepoint"),
        ("invite.list", AdminWrite, "matrix_chokepoint"),
        ("invite.resend", AdminWrite, "matrix_chokepoint"),
        ("invite.revoke", AdminWrite, "matrix_chokepoint"),
        // milestone 06 — attendance.
        ("attendance.check_in", AdminWrite, "matrix_attendance"),
        ("attendance.check_out", AdminWrite, "matrix_attendance"),
        ("attendance.list", GuardianRead, "matrix_attendance"),
        ("attendance.now", GuardianRead, "matrix_attendance"),
        ("attendance.correct", AdminWrite, "matrix_attendance"),
        // milestone 07 — menus (menu.week is the medical-leak read surface).
        ("menu.set", AdminWrite, "matrix_menu_reads"),
        ("menu.get", GuardianRead, "matrix_menu_reads"),
        ("menu.week", GuardianRead, "matrix_menu_reads"),
        ("menu.copy_week", AdminWrite, "matrix_menu_reads"),
        // milestone 08 — daily feed.
        ("log.add", AdminWrite, "matrix_daily_feed"),
        ("log.list", GuardianRead, "matrix_daily_feed"),
        ("log.correct", AdminWrite, "matrix_daily_feed"),
        ("log.day", GuardianRead, "matrix_daily_feed"),
        ("feed.watch", GuardianRead, "matrix_edge_change"),
        ("media.begin", AdminWrite, "matrix_daily_feed"),
        ("media.commit", AdminWrite, "matrix_daily_feed"),
        // milestone 09 — messaging (channel derivation is the leak vector).
        ("channel.reconcile", AdminWrite, "matrix_messaging"),
        ("announce.post", AdminWrite, "matrix_messaging"),
    ];
    rows.iter().map(|(v, k, t)| (*v, (*k, *t))).collect()
}

/// THE GATE: the coverage table's verb set == `TOOLS` exactly. A new verb in
/// `TOOLS` with no coverage line (or a stale coverage line for a removed verb)
/// fails here — the enumerate-and-fail-on-gaps contract.
#[test]
fn every_served_verb_has_a_declared_matrix_coverage() {
    let cov = coverage();
    let tools: std::collections::BTreeSet<&str> = TOOLS.iter().copied().collect();
    let covered: std::collections::BTreeSet<&str> = cov.keys().copied().collect();

    let missing: Vec<&str> = tools.difference(&covered).copied().collect();
    let stale: Vec<&str> = covered.difference(&tools).copied().collect();

    assert!(
        missing.is_empty(),
        "care.* verbs served in Tools::TOOLS but with NO cross-family matrix \
         coverage declared (add a COVERAGE row + its deny test): {missing:?}"
    );
    assert!(
        stale.is_empty(),
        "COVERAGE declares verbs no longer in Tools::TOOLS (remove the stale \
         rows): {stale:?}"
    );
    assert_eq!(
        cov.len(),
        TOOLS.len(),
        "coverage count must equal the served verb count"
    );
}

/// Every guardian-visible READ (the sacred leak surface) is named + attributed to
/// a concrete cross-family deny test. This is the rule-7 census: if this list ever
/// shrinks silently, a read verb lost its deny row. Printed so a reviewer can eyeball
/// the sacred set in one place.
#[test]
fn the_guardian_read_surface_is_the_expected_census() {
    let cov = coverage();
    let mut reads: Vec<&str> = cov
        .iter()
        .filter(|(_, (k, _))| *k == Coverage::GuardianRead)
        .map(|(v, _)| *v)
        .collect();
    reads.sort();

    // The reach-gated read surface as of milestone 10. Any change here is a
    // deliberate edit to the sacred set (rule 7) — never an accident.
    let expected = [
        "attendance.list",
        "attendance.now",
        "center.get",
        "center.list",
        "child.get",
        "child.list",
        "enrollment.list",
        "feed.watch",
        "guardian.get",
        "guardian.list",
        "log.day",
        "log.list",
        "menu.get",
        "menu.week",
        "room.get",
        "room.list",
    ];
    assert_eq!(
        reads, expected,
        "the guardian-read (rule-7 leak) surface changed — every entry MUST have a \
         cross-family deny row; update deliberately, never silently"
    );
}
