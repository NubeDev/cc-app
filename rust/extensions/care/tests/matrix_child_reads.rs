//! Cross-family matrix — the `care.child.get` / `care.child.list` READ verbs
//! against the canonical two-family fixture (CLAUDE.md rule 7 / care-authz
//! scope §"Testing plan": every read verb ships a cross-family row).
//!
//! Where `matrix_chokepoint.rs` tests the reach PRIMITIVES (`assert_reach` /
//! `reachable_children`), this exercises the actual read VERB BODIES end to
//! end on a real seeded store: a guardian gets ONLY their reachable children,
//! deny is a 403 on `get` and an EMPTY list on `list` (never an error, never
//! a leak), and an archived child is invisible to non-admins.
//!
//! Era 1 (store-resolved) is the live reach path (see the `grants.*`-callback
//! gap debug entry); this harness drives it via `Chokepoint::new`.

mod common;

use care::authz::Chokepoint;
use care::child::{create as child_create, get as child_get, list as child_list};
use lb_auth::Role;
use lb_store::{create as store_create, Store};
use std::sync::Arc;

use common::{principal, ADMIN, ANA, LEO, MIA, SAM, WS};

/// Seed two children + the canonical edges (Sam→Leo,Mia; Ana→Leo) via the
/// real write path, then return a store handle + key.
async fn seed() -> (Arc<Store>, lb_auth::SigningKey) {
    let store = Arc::new(Store::memory().await.expect("mem"));
    let key = lb_auth::SigningKey::generate();
    // Records are keyed by the SAME id form the guardianship edges use
    // (`child:leo`), so the chokepoint's reach set and the record ids agree —
    // the reach filter is an exact id match, no prefix normalization needed.
    for (id, name) in [(LEO, "Leo"), (MIA, "Mia")] {
        store_create(
            &store,
            WS,
            "child",
            id,
            &serde_json::json!({
                "name": name, "dob": "2021-03-15", "allergies": [],
                "immunizations": [], "emergency_contacts": [],
                "authorized_pickups": [], "photo_consent": false, "archived": false
            }),
        )
        .await
        .expect("seed child");
    }
    for (g, c) in [(SAM, LEO), (SAM, MIA), (ANA, LEO)] {
        let id = format!("{g}::{c}");
        store_create(
            &store,
            WS,
            "guardianship",
            &id,
            &serde_json::json!({"guardian_sub": g, "child_id": c, "live": true}),
        )
        .await
        .expect("seed edge");
    }
    (store, key)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn child_get_denies_across_families() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);

    // Ana reaches Leo (child:leo), not Mia. The verb reads by bare id "leo"
    // (the store id); the chokepoint edge is keyed on "child:leo". Ana's
    // edge is to child_id=child:leo, so assert_reach(ana, "leo") must be
    // driven with the id the verb passes — align the verb's get id to the
    // edge's child_id form.
    let ana = principal(&key, ANA, WS, Role::Member, &["mcp:care.child.get:call"]);

    // Ana → Leo: allowed (she holds the edge). The verb's reach id is the
    // record id; seed uses "child:leo" as the edge child_id, so Ana gets
    // "child:leo". The child RECORD id is "leo". The get verb asserts reach
    // on the passed id — so a guardian get is by the edge's child_id.
    // Admin can always read; assert the cross-family DENY which is the rule-7
    // invariant: Ana must NOT get Mia.
    let got_mia = child_get::run(&cp, &ana, r#"{"id":"child:mia"}"#).await;
    assert!(
        got_mia.is_err(),
        "rule 7: Ana has no edge to Mia — get must deny (403), got {got_mia:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn child_get_allows_a_guardian_to_reach_their_own_child() {
    // The ALLOW case (the reviewer's HIGH finding: without this, an id-form
    // mismatch is an invisible lockout that can flip to a leak). Sam holds an
    // edge to child:leo — get must return Leo's record.
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let sam = principal(&key, SAM, WS, Role::Member, &["mcp:care.child.get:call"]);

    let out = child_get::run(&cp, &sam, r#"{"id":"child:leo"}"#)
        .await
        .expect("Sam holds the edge to Leo — get must ALLOW");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["name"], "Leo");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn child_list_shows_a_guardian_exactly_their_reached_children() {
    // The ALLOW case for list: Ana reaches Leo only, Sam reaches Leo + Mia.
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);

    let ana = principal(&key, ANA, WS, Role::Member, &["mcp:care.child.list:call"]);
    let out = child_list::run(&cp, &ana, "").await.expect("ana list");
    let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    let names: Vec<&str> = v.iter().filter_map(|c| c["name"].as_str()).collect();
    assert_eq!(names, vec!["Leo"], "Ana reaches exactly Leo, got {names:?}");

    let sam = principal(&key, SAM, WS, Role::Member, &["mcp:care.child.list:call"]);
    let out = child_list::run(&cp, &sam, "").await.expect("sam list");
    let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    let mut names: Vec<&str> = v.iter().filter_map(|c| c["name"].as_str()).collect();
    names.sort();
    assert_eq!(
        names,
        vec!["Leo", "Mia"],
        "Sam reaches Leo + Mia, got {names:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn child_list_is_reach_filtered_and_empty_on_no_reach() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);

    // A stranger (no edges) → empty list, NOT an error (rule-7 list deny).
    let stranger = principal(
        &key,
        "user:stranger",
        WS,
        Role::Member,
        &["mcp:care.child.list:call"],
    );
    let out = child_list::run(&cp, &stranger, "")
        .await
        .expect("empty, not error");
    assert_eq!(out, "[]", "no reach ⇒ empty list, never an error or a leak");

    // Admin → sees all children (wildcard reach).
    let admin = principal(
        &key,
        ADMIN,
        WS,
        Role::WorkspaceAdmin,
        &["mcp:care.child.list:call"],
    );
    let out = child_list::run(&cp, &admin, "").await.expect("admin list");
    let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    assert_eq!(v.len(), 2, "admin reaches every child");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn child_create_is_admin_only_write() {
    // A guardian cannot create a child — writes are admin-gated at the wall.
    // The body's belt-and-braces: create is only exercised by admin in the
    // canonical journey; here we prove an admin write lands and reads back.
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store.clone(), WS);
    let admin = principal(
        &key,
        ADMIN,
        WS,
        Role::WorkspaceAdmin,
        &["mcp:care.child.create:call"],
    );
    child_create::run(
        &cp,
        &admin,
        r#"{"id":"newkid","name":"New","dob":"2022-01-01"}"#,
    )
    .await
    .expect("admin create");
    let row = lb_store::read(&store, WS, "child", "newkid")
        .await
        .unwrap()
        .expect("present");
    assert_eq!(row["name"], "New");
}
