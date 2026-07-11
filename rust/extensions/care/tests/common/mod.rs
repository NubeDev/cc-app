//! Shared fixture + helpers for the matrix harness (care/tests/). The
//! canonical fixture (care-authz-scope §Testing plan) seeds Sam, Ana,
//! Mia's-mum, two rooms, and a second workspace into a fresh
//! `Store::memory()` via the real write path (`lb_store::create`). Every
//! matrix test in this directory uses it.
//!
//! This is the `tests/common/mod.rs` pattern (idiomatic Rust for sharing
//! code between integration test binaries): each top-level file in
//! `tests/` declares `mod common;` and gets the helpers from here.

#![allow(dead_code)] // each test binary only uses a subset of the helpers

use std::sync::Arc;

use lb_auth::{mint, verify, Claims, Principal, Role, SigningKey};
use lb_store::{create as store_create, Store};

// -- Workspace + identity constants --------------------------------------

pub const WS: &str = "ws-a";
pub const WS_B: &str = "ws-b";

pub const SAM: &str = "user:sam";
pub const ANA: &str = "user:ana";
pub const MIAS_MUM: &str = "user:mia-mum";
pub const KIOSK: &str = "user:kiosk";
pub const ADMIN: &str = "user:admin-a";
pub const ADMIN_B: &str = "user:admin-b";

pub const LEO: &str = "child:leo";
pub const MIA: &str = "child:mia";

pub const POSS: &str = "room:possums";
pub const KOAL: &str = "room:koalas";

/// Seed the canonical fixture into a fresh `Store::memory()` and return
/// the shared store handle + a fresh signing key. Helper for every
/// matrix test.
pub async fn seed_fixture() -> (Arc<Store>, SigningKey) {
    let store = Arc::new(Store::memory().await.expect("mem store"));
    let key = SigningKey::generate();

    // --- ws-a: the canonical two-family setup ----------------------
    // Children.
    store_create(
        &store,
        WS,
        "child",
        "leo",
        &serde_json::json!({"name":"Leo","room":POSS}),
    )
    .await
    .expect("seed child leo");
    store_create(
        &store,
        WS,
        "child",
        "mia",
        &serde_json::json!({"name":"Mia","room":KOAL}),
    )
    .await
    .expect("seed child mia");
    // Guardians (pre-account records — the durable shape ships in M03).
    store_create(
        &store,
        WS,
        "guardian",
        SAM,
        &serde_json::json!({"name":"Sam","sub":SAM}),
    )
    .await
    .expect("seed guardian sam");
    store_create(
        &store,
        WS,
        "guardian",
        ANA,
        &serde_json::json!({"name":"Ana","sub":ANA}),
    )
    .await
    .expect("seed guardian ana");
    store_create(
        &store,
        WS,
        "guardian",
        MIAS_MUM,
        &serde_json::json!({"name":"Mia's Mum","sub":MIAS_MUM}),
    )
    .await
    .expect("seed guardian mia-mum");
    // The edges (the row shape the chokepoint reads).
    let sam_leo = format!("{SAM}::{LEO}");
    let sam_mia = format!("{SAM}::{MIA}");
    let ana_leo = format!("{ANA}::{LEO}");
    let mum_mia = format!("{MIAS_MUM}::{MIA}");
    for (id, g, c) in [
        (&sam_leo, SAM, LEO),
        (&sam_mia, SAM, MIA),
        (&ana_leo, ANA, LEO),
        (&mum_mia, MIAS_MUM, MIA),
    ] {
        store_create(
            &store,
            WS,
            "guardianship",
            id,
            &serde_json::json!({"guardian_sub":g, "child_id":c, "live":true}),
        )
        .await
        .expect("seed edge");
    }
    // Staff room assignments (Sam is also staff in both rooms for the
    // staff-matrix half — a parent who is also staff, the real-world case).
    for (id, r) in [
        (format!("{SAM}::{POSS}"), POSS),
        (format!("{SAM}::{KOAL}"), KOAL),
    ] {
        store_create(
            &store,
            WS,
            "staff_assignment",
            &id,
            &serde_json::json!({"staff_sub":SAM, "room_id":r}),
        )
        .await
        .expect("seed staff");
    }

    // --- ws-b: a second workspace for the isolation row -------------
    // No children/edges — just a sentinel so admin-A can try and fail.
    store_create(
        &store,
        WS_B,
        "child",
        "child-in-b",
        &serde_json::json!({"name":"DifferentOrg"}),
    )
    .await
    .expect("seed ws-b child");

    (store, key)
}

/// Mint + verify a principal with `role` + `caps` for `ws`. Returns the
/// verified `Principal` ready to hand to the chokepoint.
pub fn principal(
    signing: &SigningKey,
    sub: &str,
    ws: &str,
    role: Role,
    caps: &[&str],
) -> Principal {
    let claims = Claims {
        sub: sub.into(),
        ws: ws.into(),
        role,
        caps: caps.iter().map(|s| s.to_string()).collect(),
        iat: 0,
        exp: u64::MAX,
        constraint: None,
        run_id: None,
    };
    verify(signing, &mint(signing, &claims), 1).expect("token verifies")
}
