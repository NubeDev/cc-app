//! Cross-family + pickup-gate + kiosk deny sweep for `care.attendance.*` (m06 —
//! the rule-7 + child-safety gate). On a REAL `Store::memory()` seeded via the
//! real write path: `list` (rule-7 cross-family + staff room-scope), `check_out`
//! (the pickup gate — stranger denied, named/`can_pickup` allowed, admin override
//! audited), `now` (the in→out fold). Guardian writes are cap-gated at the wall.

mod common;

use care::attendance::{check_out, list, now};
use care::authz::Chokepoint;
use lb_auth::Role;
use lb_store::{create as store_create, Store};
use std::sync::Arc;

use common::{principal, ANA, KOAL, LEO, MIA, POSS, SAM, WS};

const ADMIN: &str = "user:admin-a";
const TEACH: &str = "user:teacher";

/// Seed: Leo (Sam+Ana), Mia (Sam); edges (Sam→Leo pickup, Ana→Leo NOT); Leo has an authorized-pickup "Grandma Jo"; teacher→Possums.
async fn seed() -> (Arc<Store>, lb_auth::SigningKey) {
    let store = Arc::new(Store::memory().await.expect("mem store"));
    let key = lb_auth::SigningKey::generate();

    store_create(
        &store,
        WS,
        "child",
        LEO,
        &serde_json::json!({
            "name": "Leo", "dob": "2021-03-15", "room": POSS,
            "authorized_pickups": [{"name": "Grandma Jo"}],
            "archived": false
        }),
    )
    .await
    .expect("seed leo");
    store_create(
        &store,
        WS,
        "child",
        MIA,
        &serde_json::json!({
            "name": "Mia", "dob": "2021-06-01", "room": KOAL,
            "authorized_pickups": [], "archived": false
        }),
    )
    .await
    .expect("seed mia");

    store_create(
        &store,
        WS,
        "guardian",
        SAM,
        &serde_json::json!({"name":"Sam","sub":SAM}),
    )
    .await
    .expect("seed sam");
    store_create(
        &store,
        WS,
        "guardian",
        ANA,
        &serde_json::json!({"name":"Ana","sub":ANA}),
    )
    .await
    .expect("seed ana");

    // Edges: Sam→Leo {live,can_pickup}; Ana→Leo {live, NOT}; Sam→Mia {live,pickup}.
    for (g, c, can) in [(SAM, LEO, true), (ANA, LEO, false), (SAM, MIA, true)] {
        let id = format!("{g}::{c}");
        store_create(
            &store,
            WS,
            "guardianship",
            &id,
            &serde_json::json!({"guardian_sub": g, "child_id": c, "live": true, "can_pickup": can}),
        )
        .await
        .expect("seed edge");
    }

    store_create(
        &store,
        WS,
        "staff_assignment",
        &format!("{TEACH}::{POSS}"),
        &serde_json::json!({"staff_sub": TEACH, "room_id": POSS}),
    )
    .await
    .expect("seed staff");

    (store, key)
}

/// Seed one attendance event straight through the real store write path.
async fn seed_event(
    store: &Arc<Store>,
    id: &str,
    kind: &str,
    child_id: &str,
    room_id: &str,
    at: &str,
) {
    store_create(
        store,
        WS,
        "attendance_event",
        id,
        &serde_json::json!({
            "kind": kind, "child_id": child_id, "room_id": room_id,
            "at": at, "performed_by": TEACH,
        }),
    )
    .await
    .expect("seed event");
}

/// Ana reaches Leo only: `attendance.list` returns ONLY Leo's events, never Mia's (worst-bug deny row).
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn guardian_cannot_check_in() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store.clone(), WS);
    seed_event(
        &store,
        "ev:leo:1",
        "check_in",
        LEO,
        POSS,
        "2026-07-14T08:00:00Z",
    )
    .await;
    seed_event(
        &store,
        "ev:leo:2",
        "check_out",
        LEO,
        POSS,
        "2026-07-14T17:00:00Z",
    )
    .await;
    seed_event(
        &store,
        "ev:mia:1",
        "check_in",
        MIA,
        KOAL,
        "2026-07-14T08:05:00Z",
    )
    .await;

    let ana = principal(
        &key,
        ANA,
        WS,
        Role::Member,
        &["mcp:care.attendance.list:call"],
    );
    let out = list::run(&cp, &ana, "").await.expect("ana list ok");
    let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    assert_eq!(v.len(), 2, "Ana sees Leo's two events only, got {v:?}");
    for row in &v {
        assert_eq!(row["child_id"], LEO);
        assert_ne!(row["child_id"], MIA, "MUST NOT leak Mia across families");
    }
}

fn checkout(collector_name: &str, collector_sub: Option<&str>, override_: bool) -> String {
    let sub = collector_sub
        .map(|s| format!(r#","collector_sub":"{s}""#))
        .unwrap_or_default();
    let ovr = if override_ {
        r#","pickup_override":true"#
    } else {
        ""
    };
    format!(
        r#"{{"event_id":"co:1","child_id":"{LEO}","room_id":"{POSS}","at":"2026-07-14T17:20:00Z","collector_name":"{collector_name}"{sub}{ovr}}}"#
    )
}

/// A non-`can_pickup`, non-named collector is HARD-DENIED — no event written.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn stranger_pickup_is_hard_denied() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let admin = principal(
        &key,
        ADMIN,
        WS,
        Role::WorkspaceAdmin,
        &["mcp:care.attendance.check_out:call"],
    );

    let res = check_out::run(
        &cp,
        &admin,
        &checkout("Mallory", Some("user:mallory"), false),
    )
    .await;
    assert!(res.is_err(), "a stranger collector must be hard-denied");
    assert!(
        cp.records()
            .read("attendance_event", "co:1")
            .await
            .unwrap()
            .is_none(),
        "no check_out event may be written on a deny"
    );
}

/// A named authorized-pickup ("Grandma Jo") is allowed — the event is written.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn authorized_pickup_by_name_allowed() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let admin = principal(
        &key,
        ADMIN,
        WS,
        Role::WorkspaceAdmin,
        &["mcp:care.attendance.check_out:call"],
    );

    let out = check_out::run(&cp, &admin, &checkout("Grandma Jo", None, false))
        .await
        .expect("named authorized pickup allowed");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["event_id"], "co:1");
    let ev = cp
        .records()
        .read("attendance_event", "co:1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ev["kind"], "check_out");
}

/// Ana has a live edge to Leo but `can_pickup:false` — the gate denies, no event.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn non_can_pickup_guardian_denied() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let admin = principal(
        &key,
        ADMIN,
        WS,
        Role::WorkspaceAdmin,
        &["mcp:care.attendance.check_out:call"],
    );

    let res = check_out::run(&cp, &admin, &checkout("Ana", Some(ANA), false)).await;
    assert!(res.is_err(), "a can_pickup:false guardian must be denied");
    assert!(
        cp.records()
            .read("attendance_event", "co:1")
            .await
            .unwrap()
            .is_none(),
        "no event on a denied non-can_pickup guardian"
    );
}

/// An admin releases past a failed gate with `pickup_override:true` (audited on the event).
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn admin_override_is_audited() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let admin = principal(
        &key,
        ADMIN,
        WS,
        Role::WorkspaceAdmin,
        &["mcp:care.attendance.check_out:call"],
    );

    // Mallory is a stranger — denied by the gate — but the admin overrides.
    let out = check_out::run(
        &cp,
        &admin,
        &checkout("Mallory", Some("user:mallory"), true),
    )
    .await
    .expect("admin override allowed");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["event_id"], "co:1");
    let ev = cp
        .records()
        .read("attendance_event", "co:1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        ev["pickup_override"], true,
        "override must be audited on the event"
    );
}

/// check_in Leo ⇒ children:1; check_out ⇒ children:0 (the derived occupancy fold).
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn now_reflects_in_then_out() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store.clone(), WS);
    let admin_now = principal(
        &key,
        ADMIN,
        WS,
        Role::WorkspaceAdmin,
        &["mcp:care.attendance.now:call"],
    );

    seed_event(
        &store,
        "in:leo",
        "check_in",
        LEO,
        POSS,
        "2026-07-14T08:00:00Z",
    )
    .await;

    let out = now::run(&cp, &admin_now, r#"{"room_id":"room:possums"}"#)
        .await
        .expect("now after in");
    let occ: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    assert_eq!(occ.len(), 1, "one occupied room after check-in");
    assert_eq!(occ[0]["children"], 1, "Leo present ⇒ children:1");

    let admin_co = principal(
        &key,
        ADMIN,
        WS,
        Role::WorkspaceAdmin,
        &["mcp:care.attendance.check_out:call"],
    );
    check_out::run(&cp, &admin_co, &checkout("Grandma Jo", None, false))
        .await
        .expect("gated check-out");

    let out = now::run(&cp, &admin_now, r#"{"room_id":"room:possums"}"#)
        .await
        .expect("now after out");
    let occ: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    // The room may drop out of the fold entirely, or report children:0.
    let children = occ
        .first()
        .and_then(|o| o["children"].as_i64())
        .unwrap_or(0);
    assert_eq!(children, 0, "later check_out ⇒ Leo absent ⇒ children:0");
}

/// A staff member in Possums sees Possums events, never Koalas (staff rule-7 filter).
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn staff_scoped_list() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store.clone(), WS);
    seed_event(
        &store,
        "ev:poss",
        "check_in",
        LEO,
        POSS,
        "2026-07-14T08:00:00Z",
    )
    .await;
    seed_event(
        &store,
        "ev:koal",
        "check_in",
        MIA,
        KOAL,
        "2026-07-14T08:05:00Z",
    )
    .await;

    let teacher = principal(
        &key,
        TEACH,
        WS,
        Role::Member,
        &["mcp:care.attendance.list:call"],
    );
    let out = list::run(&cp, &teacher, "").await.expect("staff list ok");
    let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v.len(),
        1,
        "teacher assigned to Possums sees one room's events, got {v:?}"
    );
    assert_eq!(v[0]["room_id"], POSS);
    for row in &v {
        assert_ne!(
            row["room_id"], KOAL,
            "MUST NOT leak Koalas to a Possums-only teacher"
        );
    }
}
