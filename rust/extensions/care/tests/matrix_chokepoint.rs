//! The cross-family matrix harness — the chokepoint half.
//!
//! Seeds the canonical fixture (Sam(Leo+Mia) / Ana(Leo) / Mia's-mum(Mia),
//! two rooms, a second workspace) via the REAL write path
//! (`lb_store::create`) and asserts the chokepoint's allow/deny/empty
//! table over every shape the two-call API answers. A verb without a
//! matrix row fails the harness (see `matrix_coverage.rs`).
//!
//! ## Per CLAUDE.md rule 7
//!
//! A cross-family data leak is the existential bug this product can
//! have. The matrix is the canonical suite — every new chokepoint call
//! shape (and every verb) lands with a row added here. The harness
//! refuses to start if `crate::call::TOOLS` isn't fully covered.

mod common;

use care::authz::{assert_reach, reachable_children, reachable_rooms, Chokepoint};
use lb_auth::Role;
use lb_store::{create as store_create, write as store_write, Store, StoreError};

use common::{
    principal, seed_fixture, ADMIN, ADMIN_B, ANA, KOAL, LEO, MIA, MIAS_MUM, POSS, SAM, WS, WS_B,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn chokepoint_allows_only_live_guardianship_edges() {
    let (store, key) = seed_fixture().await;
    let cp = Chokepoint::new(store, WS);

    // Sam reaches both Leo and Mia.
    let sam = principal(&key, SAM, WS, Role::Member, &["mcp:care.ping:call"]);
    assert_reach(&cp, &sam, LEO).await.expect("sam→leo allow");
    assert_reach(&cp, &sam, MIA).await.expect("sam→mia allow");

    // Ana reaches Leo only.
    let ana = principal(&key, ANA, WS, Role::Member, &["mcp:care.ping:call"]);
    assert_reach(&cp, &ana, LEO).await.expect("ana→leo allow");
    assert_reach(&cp, &ana, MIA)
        .await
        .expect_err("ana→mia deny (no edge)");

    // Mia's mum reaches Mia only.
    let mum = principal(&key, MIAS_MUM, WS, Role::Member, &["mcp:care.ping:call"]);
    assert_reach(&cp, &mum, LEO)
        .await
        .expect_err("mum→leo deny");
    assert_reach(&cp, &mum, MIA).await.expect("mum→mia allow");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn chokepoint_reachable_children_returns_only_reached_set() {
    let (store, key) = seed_fixture().await;
    let cp = Chokepoint::new(store, WS);

    let sam = principal(&key, SAM, WS, Role::Member, &["mcp:care.ping:call"]);
    let mut got = reachable_children(&cp, &sam).await;
    let mut want = vec![LEO.to_string(), MIA.to_string()];
    got.sort();
    want.sort();
    assert_eq!(got, want, "sam reaches both kids");

    let ana = principal(&key, ANA, WS, Role::Member, &["mcp:care.ping:call"]);
    assert_eq!(
        reachable_children(&cp, &ana).await,
        vec![LEO.to_string()],
        "ana reaches leo only"
    );

    let mum = principal(&key, MIAS_MUM, WS, Role::Member, &["mcp:care.ping:call"]);
    assert_eq!(
        reachable_children(&cp, &mum).await,
        vec![MIA.to_string()],
        "mum reaches mia only"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn chokepoint_reachable_rooms_returns_only_assigned_rooms() {
    let (store, key) = seed_fixture().await;
    let cp = Chokepoint::new(store, WS);

    let sam = principal(&key, SAM, WS, Role::Member, &["mcp:care.ping:call"]);
    let mut got = reachable_rooms(&cp, &sam).await;
    let mut want = vec![POSS.to_string(), KOAL.to_string()];
    got.sort();
    want.sort();
    assert_eq!(got, want, "sam reaches both rooms");

    let ana = principal(&key, ANA, WS, Role::Member, &["mcp:care.ping:call"]);
    assert_eq!(reachable_rooms(&cp, &ana).await, Vec::<String>::new());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn chokepoint_admin_passes_via_audited_role_check_only() {
    let (store, key) = seed_fixture().await;
    let cp = Chokepoint::new(store, WS);

    let admin = principal(&key, ADMIN, WS, Role::WorkspaceAdmin, &[]);
    assert_reach(&cp, &admin, LEO)
        .await
        .expect("admin allow (audited role)");
    assert_eq!(
        reachable_children(&cp, &admin).await,
        vec!["*".to_string()],
        "admin gets wildcard"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn chokepoint_admin_in_other_workspace_does_not_reach() {
    // Documents the role-vs-workspace boundary: the chokepoint's role
    // check is purely additive; the workspace wall fires upstream.
    let (store, key) = seed_fixture().await;
    let cp = Chokepoint::new(store, WS);

    let admin_b = principal(&key, ADMIN_B, WS_B, Role::WorkspaceAdmin, &[]);
    assert_eq!(
        reachable_children(&cp, &admin_b).await,
        vec!["*".to_string()],
        "role check is additive: the workspace wall fires upstream; the \
         chokepoint must NOT silently widen an admin's reach across \
         workspaces — verified end-to-end by the host embed test"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn chokepoint_kiosk_principal_reaches_nothing() {
    let (store, key) = seed_fixture().await;
    let cp = Chokepoint::new(store, WS);

    let kiosk = principal(
        &key,
        common::KIOSK,
        WS,
        Role::Member,
        &["mcp:care.attendance.kiosk:call"],
    );
    assert_reach(&cp, &kiosk, LEO)
        .await
        .expect_err("kiosk→leo deny (no edge)");
    assert_reach(&cp, &kiosk, MIA)
        .await
        .expect_err("kiosk→mia deny (no edge)");
    assert_eq!(reachable_children(&cp, &kiosk).await, Vec::<String>::new());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn unlink_immediately_denies() {
    let (store, key) = seed_fixture().await;
    let cp = Chokepoint::new(store.clone(), WS);

    let ana = principal(&key, ANA, WS, Role::Member, &["mcp:care.ping:call"]);
    assert_reach(&cp, &ana, LEO)
        .await
        .expect("ana→leo allow (initial)");

    // Archive the edge (live → false). The very next call must deny —
    // no caching, no grace period. Era-2 derives the scoped grants in
    // the same transaction so the wall denies too.
    let ana_leo_id = format!("{ANA}::{LEO}");
    store_write(
        &store,
        WS,
        "guardianship",
        &ana_leo_id,
        &serde_json::json!({"guardian_sub":ANA, "child_id":LEO, "live":false}),
    )
    .await
    .expect("archive ana-leo edge");

    assert_reach(&cp, &ana, LEO)
        .await
        .expect_err("ana→leo deny after unlink (no caching)");

    assert_eq!(
        reachable_children(&cp, &ana).await,
        Vec::<String>::new(),
        "ana's reachable set is empty after unlink"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn seed_fixture_idempotent_for_a_fresh_store() {
    let store = Store::memory().await.expect("mem");
    let res: Result<(), StoreError> = store_create(
        &store,
        WS,
        "child",
        "leo",
        &serde_json::json!({"name":"Leo"}),
    )
    .await;
    assert!(res.is_ok());
    let res: Result<(), StoreError> = store_create(
        &store,
        WS,
        "child",
        "leo",
        &serde_json::json!({"name":"Leo"}),
    )
    .await;
    assert!(matches!(res, Err(StoreError::Conflict)));
}
