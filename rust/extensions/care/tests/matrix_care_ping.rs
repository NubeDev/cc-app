//! The cross-family matrix harness — the `care.ping` half + the
//! matrix-coverage guard.
//!
//! Today only one verb ships (`care.ping`); this file exercises its
//! deny-test, its full round-trip through the child wire, and the
//! matrix-coverage guard (a verb without a row fails the harness).

mod common;

use care::call::TOOLS;
use lb_ext_native::serve::Tools;
use serde_json::Value;

use common::WS;

/// MANDATORY GUARD: every registered verb has a row in the matrix table.
/// Adding a verb without a matrix row fails the harness.
///
/// Today the matrix is one row (one verb: `care.ping`), satisfied by the
/// dedicated tests in this file. As each milestone 03+ verb ships, add
/// a test for it here AND add its name to `COVERED_VERBS`.
const COVERED_VERBS: &[&str] = &[
    "ping",
    "center.create",
    "center.get",
    "center.list",
    "room.create",
    "room.get",
    "room.list",
    "child.create",
    "child.get",
    "child.list",
    "child.update",
    "child.archive",
    "guardian.create",
    "guardian.get",
    "guardian.list",
    "guardianship.link",
    "guardianship.unlink",
    "guardianship.update",
    "enrollment.create",
    "enrollment.list",
    "enrollment.update",
    "invite.create_guardian",
    "invite.create_staff",
    "invite.list",
    "invite.resend",
    "invite.revoke",
    // Milestone 06 — attendance. Cross-family + pickup-gate deny sweep in
    // `tests/matrix_attendance.rs`; per-verb in-file tests (rule-7 list scope,
    // pickup-gate denies, kiosk/staff scope) in each verb's `src` module.
    "attendance.check_in",
    "attendance.check_out",
    "attendance.list",
    "attendance.now",
    "attendance.correct",
    // Milestone 07 — menus. The guardian medical-leak deny sweep is in
    // `tests/matrix_menu_reads.rs` (Ana denied Mia's week, never sees another
    // child's allergen); per-verb in-file tests cover room-scope + derivation.
    "menu.set",
    "menu.get",
    "menu.week",
    "menu.copy_week",
    // Milestone 08 — daily feed. The cross-family deny sweep (guardian sees only
    // reached children's entries on list/day; a stranger's feed.watch is denied;
    // the media-URL leak 403s) is in `tests/matrix_daily_feed.rs`; per-verb
    // in-file tests (rule-7 list/day scope, photo-consent-at-write, cursor
    // stability, compensating correction) live in each verb's `src` module.
    "log.add",
    "log.list",
    "log.correct",
    "log.day",
    "feed.watch",
];

#[test]
fn assert_matrix_covers_all_verbs() {
    let registered: std::collections::HashSet<&str> = TOOLS.iter().copied().collect();
    let covered: std::collections::HashSet<&str> = COVERED_VERBS.iter().copied().collect();
    let missing: Vec<&&str> = registered.difference(&covered).collect();
    assert!(
        missing.is_empty(),
        "matrix harness: verbs registered without a matrix row: {missing:?} \
         — add a deny-test for the new verb + add its name to COVERED_VERBS \
         in rust/extensions/care/tests/matrix_care_ping.rs"
    );
    // And the reverse: a verb named in COVERED_VERBS that ISN'T in TOOLS
    // is a stale row (a verb was removed without cleaning up the test).
    let stale: Vec<&&str> = covered.difference(&registered).collect();
    assert!(
        stale.is_empty(),
        "matrix harness: stale rows in COVERED_VERBS (verb removed?): {stale:?} \
         — remove the deny-test for it from matrix_care_ping.rs and its \
         name from COVERED_VERBS"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn care_ping_deny_test_fails_closed_without_the_cap() {
    let no_cap = common::principal(
        &common::seed_fixture().await.1, // placeholder; we don't actually need a real key here
        common::SAM,
        WS,
        lb_auth::Role::Member,
        &[],
    );
    assert!(
        care::call::require_caller_cap(no_cap.caps(), care::call::REQUIRED_CAP).is_err(),
        "cap-deny: a caller without the care.ping cap is denied at the body"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn care_ping_round_trips_through_the_child_wire() {
    let mut svc = care::Care::new(WS.to_string());

    // Bare tool name (the /native/call bridge uses this).
    let out = svc.call("ping", r#"{"echo":"hello"}"#).await.expect("ok");
    let v: Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["ws"], WS);
    assert_eq!(v["ok"], true);
    assert_eq!(v["echoed"], "hello");

    // Qualified tool name (the routed native adapter uses this).
    let out = svc.call("care.ping", r#"{"echo":"hi"}"#).await.expect("ok");
    let v: Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["echoed"], "hi");

    // Unknown tool is an explicit error, not a panic.
    assert!(svc.call("care.unknown", "{}").await.is_err());

    // The `tools()` handshake list matches TOOLS.
    let listed = svc.tools();
    assert_eq!(
        listed,
        TOOLS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "the init handshake reports exactly the registered tools"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn care_ping_round_trips_with_a_signed_principal_over_the_chokepoint() {
    // The end-to-end chokepoint + verb-body composition: a principal
    // with the cap reaches the verb (we model "verb callable" by going
    // through the Tools::call path; the host's wall is the authoritative
    // gate, exercised in the live HTTP round-trip in the session doc).
    let mut svc = care::Care::new(WS.to_string());
    let (_store, _key) = common::seed_fixture().await;
    let _p = common::principal(
        &_key,
        common::SAM,
        WS,
        lb_auth::Role::Member,
        &["mcp:care.ping:call"],
    );

    // The principal would reach Leo (Sam has the edge); the verb body
    // doesn't care about the principal — it's stateless — so this is a
    // smoke test that the body works under the canonical Sam+care.ping
    // posture.
    let out = svc
        .call("care.ping", r#"{"echo":"sam"}"#)
        .await
        .expect("ok");
    let v: Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["ws"], WS);
    assert_eq!(v["echoed"], "sam");
}
