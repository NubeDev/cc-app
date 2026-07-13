use super::*;
use crate::child::create as child_create;
use crate::menu::allergen::Allergen;
use crate::menu::{MenuItem, Substitution};
use lb_auth::{mint, verify, Claims, Role, SigningKey};
use lb_store::{create as store_create, Store};
use serde_json::json;
use std::sync::Arc;

const WS: &str = "acme";
const MON: &str = "2026-07-13"; // a Monday
const ROOM: &str = "room:possums";

fn admin(signing: &SigningKey) -> Principal {
    principal(signing, "user:admin", Role::WorkspaceAdmin)
}
fn guardian(signing: &SigningKey, sub: &str) -> Principal {
    principal(signing, sub, Role::Member)
}
fn principal(signing: &SigningKey, sub: &str, role: Role) -> Principal {
    let caps = vec![
        "mcp:care.child.create:call".into(),
        "mcp:care.menu.week:call".into(),
    ];
    let claims = Claims {
        sub: sub.into(),
        ws: WS.into(),
        role,
        caps,
        iat: 0,
        exp: u64::MAX,
        constraint: None,
        run_id: None,
    };
    verify(signing, &mint(signing, &claims), 1).expect("verify")
}

/// Seed a LIVE guardianship edge via the real store write path (the authz
/// source of truth the chokepoint reads).
async fn seed_edge(store: &Arc<Store>, guardian_sub: &str, child_id: &str) {
    let id = [guardian_sub, child_id].join("::");
    let row = json!({ "guardian_sub": guardian_sub, "child_id": child_id, "relationship": "mother", "live": true });
    store_create(store, WS, "guardianship", &id, &row)
        .await
        .unwrap();
}

/// Seed one menu cell for the possums room on Monday lunch.
async fn seed_menu(store: &Arc<Store>, subs: Vec<Substitution>) {
    let menu = Menu {
        date: MON.into(),
        room_id: ROOM.into(),
        slot: Slot::Lunch,
        items: vec![MenuItem {
            name: "Peanut satay".into(),
            allergens: vec![Allergen::Peanut],
        }],
        substitutions: subs,
    };
    let id = Menu::id(MON, ROOM, Slot::Lunch);
    store_create(
        store,
        WS,
        "menu",
        &id,
        &serde_json::to_value(&menu).unwrap(),
    )
    .await
    .unwrap();
}

fn week_input(child_id: &str) -> String {
    json!({ "child_id": child_id, "week_start": MON }).to_string()
}

/// Seed the two-family fixture: Sam→Leo,Mia; Ana→Leo. Leo is peanut-allergic
/// and roomed in possums. Returns the store + signing key.
async fn fixture() -> (Arc<Store>, SigningKey, Chokepoint) {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store.clone(), WS);
    let a = admin(&key);

    child_create::run(
        &cp,
        &a,
        &json!({
            "id": "leo",
            "name": "Leo",
            "dob": "2021-03-15",
            "allergies": ["peanut"],
            "room_id": ROOM,
        })
        .to_string(),
    )
    .await
    .unwrap();
    child_create::run(&cp, &a, r#"{"id":"mia","name":"Mia","dob":"2020-06-01"}"#)
        .await
        .unwrap();

    // Edges: Sam reaches Leo + Mia; Ana reaches only Leo.
    seed_edge(&store, "user:sam", "leo").await;
    seed_edge(&store, "user:sam", "mia").await;
    seed_edge(&store, "user:ana", "leo").await;

    (store, key, cp)
}

#[tokio::test]
async fn ana_reaches_leo_sees_his_peanut_substitution_row() {
    let (store, key, cp) = fixture().await;
    seed_menu(&store, vec![]).await; // no substitute entered → unresolved

    let ana = guardian(&key, "user:ana");
    let out = run(&cp, &ana, &week_input("leo")).await.expect("week");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert_eq!(v["child_id"], "leo");
    assert_eq!(v["room_id"], ROOM);
    assert_eq!(v["days"].as_array().unwrap().len(), 7);

    // Day 0 (Monday) has the seeded lunch cell.
    let day0 = &v["days"][0];
    assert_eq!(day0["date"], MON);
    let slot = &day0["slots"][0];
    assert_eq!(slot["slot"], "lunch");
    assert_eq!(slot["items"][0]["name"], "Peanut satay");
    // The item's allergen tags are NOT surfaced (leak guard).
    assert!(slot["items"][0].get("allergens").is_none());
    // Leo's peanut substitution row IS present, unresolved.
    assert_eq!(slot["substitutions"][0]["reason"], "peanut");
    assert_eq!(slot["substitutions"][0]["resolved"], false);
}

#[tokio::test]
async fn rule7_ana_cannot_read_mia_the_cross_family_deny() {
    // MANDATORY cross-family row: Ana has NO edge to Mia → 403, no data.
    let (store, key, cp) = fixture().await;
    seed_menu(&store, vec![]).await;

    let ana = guardian(&key, "user:ana");
    let err = run(&cp, &ana, &week_input("mia"))
        .await
        .expect_err("Ana must be DENIED Mia — the leak gate");
    // Denied, and no room/menu data leaked.
    assert!(!err.contains(ROOM), "must not leak the room: {err}");
    assert!(!err.contains("Peanut"), "must not leak menu items: {err}");
}

#[tokio::test]
async fn a_resolved_substitution_shows_resolved_true() {
    let (store, key, cp) = fixture().await;
    seed_menu(
        &store,
        vec![Substitution {
            restriction: Allergen::Peanut,
            substitute: "Sunflower satay".into(),
        }],
    )
    .await;

    let ana = guardian(&key, "user:ana");
    let out = run(&cp, &ana, &week_input("leo")).await.expect("week");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let sub = &v["days"][0]["slots"][0]["substitutions"][0];
    assert_eq!(sub["resolved"], true);
    assert_eq!(sub["substitute"], "Sunflower satay");
}

#[tokio::test]
async fn child_with_no_room_gets_an_empty_week() {
    // Mia has no room_id → Sam (who reaches her) gets an empty week, not err.
    let (_store, key, cp) = fixture().await;
    let sam = guardian(&key, "user:sam");
    let out = run(&cp, &sam, &week_input("mia"))
        .await
        .expect("empty week");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["room_id"], serde_json::Value::Null);
    assert_eq!(v["days"].as_array().unwrap().len(), 0);
}

#[test]
fn add_days_walks_the_calendar_across_boundaries() {
    assert_eq!(add_days("2026-07-13", 0).unwrap(), "2026-07-13");
    assert_eq!(add_days("2026-07-13", 6).unwrap(), "2026-07-19");
    // month boundary
    assert_eq!(add_days("2026-07-30", 3).unwrap(), "2026-08-02");
    // leap year: 2024-02-28 + 1 = 2024-02-29
    assert_eq!(add_days("2024-02-28", 1).unwrap(), "2024-02-29");
    // non-leap: 2026-02-28 + 1 = 2026-03-01
    assert_eq!(add_days("2026-02-28", 1).unwrap(), "2026-03-01");
    // year boundary
    assert_eq!(add_days("2026-12-31", 1).unwrap(), "2027-01-01");
}
