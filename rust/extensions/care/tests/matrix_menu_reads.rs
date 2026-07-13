//! Cross-family matrix — the `care.menu.week` GUARDIAN FOOD VIEW read verb
//! against the canonical two-family fixture (CLAUDE.md rule 7 / menus-scope
//! §"the medical-leak class": every read verb ships a cross-family row, and
//! this is THE medical-leak surface — milestone 07's mandatory rule-7 gate).
//!
//! `care.menu.week` returns a child's ROOM's week of menus PLUS ONLY the
//! asking child's derived substitution rows. The leak class it must never
//! commit: returning another child's room plan (Ana asking for Mia's
//! `child_id`), or surfacing another child's allergen in a room the two
//! children share. This harness drives the verb end to end on a REAL seeded
//! `Store::memory()` and asserts the deny is a 403 with NO egg/koalas/Mia
//! leakage, and that even a legitimate read of Leo's week never contains
//! Mia's allergen word.
//!
//! Era 1 (store-resolved) is the live reach path; this harness drives it via
//! `Chokepoint::new` — the same posture as `matrix_child_reads.rs`.

mod common;

use care::authz::Chokepoint;
use care::menu::week;
use lb_auth::Role;
use lb_store::{create as store_create, Store};
use std::sync::Arc;

use common::{principal, ANA, KOAL, LEO, MIA, MIAS_MUM, POSS, WS};

/// A Monday whose week (Mon..Sun) contains the seeded menu dates.
const MON: &str = "2026-07-13";

/// The `mcp:care.menu.week:call` cap every caller here needs at the wall.
const CAP: &str = "mcp:care.menu.week:call";

/// Seed the two-family food fixture via the REAL write path:
///   - Leo: allergies ["peanut"], room possums, reached by Ana.
///   - Mia: allergies ["egg"],    room koalas,  reached ONLY by Mia's mum.
///   - guardianship edges {guardian_sub, child_id, live:true} for Ana→Leo and
///     mum→Mia (Ana holds NO edge to Mia — the leak she must not exploit).
///   - a peanut-satay lunch in possums, an egg-custard lunch in koalas, both
///     dated on MON (inside the requested week). No substitute entered on
///     either → the peanut row derives UNRESOLVED (the red flag the guardian
///     sees), and the egg tag lives only in the koalas cell.
async fn seed() -> (Arc<Store>, lb_auth::SigningKey) {
    let store = Arc::new(Store::memory().await.expect("mem"));
    let key = lb_auth::SigningKey::generate();

    // Children — the child record is the SINGLE source of allergy truth the
    // derivation intersects on; `room_id` places the child in a room's plan.
    for (id, name, allergen, room) in [(LEO, "Leo", "peanut", POSS), (MIA, "Mia", "egg", KOAL)] {
        store_create(
            &store,
            WS,
            "child",
            id,
            &serde_json::json!({
                "name": name,
                "dob": "2021-03-15",
                "allergies": [allergen],
                "room_id": room,
                "immunizations": [], "emergency_contacts": [],
                "authorized_pickups": [], "photo_consent": false, "archived": false
            }),
        )
        .await
        .expect("seed child");
    }

    // The live guardianship edges (the row shape the chokepoint reads). Ana
    // reaches Leo; Mia's mum reaches Mia. Ana ↛ Mia is the whole point.
    for (g, c) in [(ANA, LEO), (MIAS_MUM, MIA)] {
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

    // The menus — one lunch cell per room on MON. The menu id is the natural
    // key `<date>::<room>::<slot>`, so the week verb addresses these directly.
    // Possums (Leo's room): peanut-tagged satay, NO substitute → unresolved.
    seed_menu(&store, POSS, "Peanut satay", "peanut").await;
    // Koalas (Mia's room): egg-tagged custard — the OTHER family's allergen,
    // which must never reach Ana through any read of Leo's week.
    seed_menu(&store, KOAL, "Egg custard", "egg").await;

    (store, key)
}

/// Seed one `(MON, room, lunch)` menu cell with a single allergen-tagged item
/// and no substitute (unresolved) — the id matches `Menu::id`.
async fn seed_menu(store: &Arc<Store>, room: &str, item: &str, allergen: &str) {
    let id = format!("{MON}::{room}::lunch");
    store_create(
        store,
        WS,
        "menu",
        &id,
        &serde_json::json!({
            "date": MON,
            "room_id": room,
            "slot": "lunch",
            "items": [{"name": item, "allergens": [allergen]}],
            "substitutions": [],
        }),
    )
    .await
    .expect("seed menu");
}

/// The verb input for a child's week anchored on MON.
fn week_input(child_id: &str) -> String {
    serde_json::json!({"child_id": child_id, "week_start": MON}).to_string()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn ana_sees_leos_peanut_substitution() {
    // The ALLOW case: Ana holds the edge to Leo → she gets Leo's room's week
    // with the peanut substitution row derived (unresolved, no substitute).
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let ana = principal(&key, ANA, WS, Role::Member, &[CAP]);

    let out = week::run(&cp, &ana, &week_input(LEO))
        .await
        .expect("Ana reaches Leo — week must ALLOW");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert_eq!(v["child_id"], LEO);
    assert_eq!(v["room_id"], POSS);
    assert_eq!(v["days"].as_array().unwrap().len(), 7);

    // Day 0 (MON) carries the seeded possums lunch cell.
    let slot = &v["days"][0]["slots"][0];
    assert_eq!(slot["slot"], "lunch");
    assert_eq!(slot["items"][0]["name"], "Peanut satay");
    // Leo's peanut row derives, unresolved (no substitute entered).
    assert_eq!(slot["substitutions"][0]["reason"], "peanut");
    assert_eq!(slot["substitutions"][0]["resolved"], false);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn ana_cannot_see_mias_week() {
    // THE MANDATORY CROSS-FAMILY ROW: Ana has NO edge to Mia → 403, and the
    // error carries NO egg / koalas / Mia leakage. Without the reach gate,
    // asking for Mia's child_id would return Mia's room's (koalas) plan.
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let ana = principal(&key, ANA, WS, Role::Member, &[CAP]);

    let err = week::run(&cp, &ana, &week_input(MIA))
        .await
        .expect_err("rule 7: Ana has no edge to Mia — week must DENY (403)");

    assert!(!err.contains(KOAL), "must not leak Mia's room: {err}");
    assert!(!err.contains("egg"), "must not leak Mia's allergen: {err}");
    assert!(!err.contains("Egg"), "must not leak Mia's menu item: {err}");
    assert!(!err.contains("Mia"), "must not leak the child: {err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn ana_week_never_contains_another_childs_allergen() {
    // Even the LEGITIMATE read of Leo's week must never surface Mia's
    // restriction. Leo's room (possums) and Mia's (koalas) differ here, but
    // the invariant is stronger than room separation: a guardian read returns
    // ONLY the asking child's derived rows + Leo's room's item names — never
    // another child's allergen. Assert the whole response is egg-free.
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let ana = principal(&key, ANA, WS, Role::Member, &[CAP]);

    let out = week::run(&cp, &ana, &week_input(LEO))
        .await
        .expect("Ana reaches Leo");

    assert!(
        !out.to_lowercase().contains("egg"),
        "Leo's week must never contain Mia's allergen (egg): {out}"
    );
    assert!(
        !out.contains(KOAL),
        "Leo's week must never reference Mia's room (koalas): {out}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn guardian_with_no_reach_is_denied() {
    // A stranger guardian (no edges at all) asking for ANY child → 403. Reach
    // fails closed; a caller with no edge is never handed a week.
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let stranger = principal(&key, "user:stranger", WS, Role::Member, &[CAP]);

    let leo = week::run(&cp, &stranger, &week_input(LEO)).await;
    assert!(leo.is_err(), "stranger ↛ Leo → deny, got {leo:?}");
    let mia = week::run(&cp, &stranger, &week_input(MIA)).await;
    assert!(mia.is_err(), "stranger ↛ Mia → deny, got {mia:?}");
}
