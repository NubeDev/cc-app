use super::*;
use lb_auth::{mint, verify, Claims, Role, SigningKey};
use lb_store::{create as store_create, Store};
use serde_json::json;
use std::sync::Arc;

fn principal(signing: &SigningKey, sub: &str, ws: &str, role: Role) -> Principal {
    let claims = Claims {
        sub: sub.into(),
        ws: ws.into(),
        role,
        caps: vec!["mcp:care.attendance.check_out:call".into()],
        iat: 0,
        exp: u64::MAX,
        constraint: None,
        run_id: None,
    };
    verify(signing, &mint(signing, &claims), 1).expect("verify")
}
fn admin(k: &SigningKey, ws: &str) -> Principal {
    principal(k, "user:admin", ws, Role::WorkspaceAdmin)
}
fn member(k: &SigningKey, sub: &str, ws: &str) -> Principal {
    principal(k, sub, ws, Role::Member)
}

async fn seed_child(store: &Arc<Store>, ws: &str, id: &str, pickups: serde_json::Value) {
    let child = json!({
        "name": "Leo",
        "dob": "2021-03-15",
        "authorized_pickups": pickups,
    });
    store_create(store, ws, "child", id, &child).await.unwrap();
}

async fn seed_edge(
    store: &Arc<Store>,
    ws: &str,
    edge_id: &str,
    guardian_sub: &str,
    child_id: &str,
    can_pickup: bool,
    custody_notes: Option<&str>,
) {
    let mut edge = json!({
        "guardian_sub": guardian_sub,
        "child_id": child_id,
        "live": true,
        "can_pickup": can_pickup,
    });
    if let Some(n) = custody_notes {
        edge["custody_notes"] = json!(n);
    }
    store_create(store, ws, "guardianship", edge_id, &edge)
        .await
        .unwrap();
    // Guardian record so the name resolves for name-based authorization.
    let g = json!({ "name": "Sam Parent" });
    store_create(store, ws, "guardian", guardian_sub, &g)
        .await
        .unwrap();
}

#[tokio::test]
async fn can_pickup_guardian_by_sub_is_allowed() {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store.clone(), "acme");
    seed_child(&store, "acme", "leo", json!([])).await;
    seed_edge(&store, "acme", "e1", "user:sam", "leo", true, None).await;

    let p = admin(&key, "acme");
    let out = run(
            &cp,
            &p,
            r#"{"event_id":"ev1","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Sam Parent","collector_sub":"user:sam"}"#,
        )
        .await
        .expect("check-out allowed");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["event_id"], "ev1");
    // Event was appended.
    let ev = cp
        .records()
        .read("attendance_event", "ev1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ev["kind"], "check_out");
    assert_eq!(ev["pickup_override"], false);
}

#[tokio::test]
async fn authorized_pickup_by_name_is_allowed() {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store.clone(), "acme");
    seed_child(&store, "acme", "leo", json!([{"name": "Grandma Jo"}])).await;

    let p = admin(&key, "acme");
    let out = run(
            &cp,
            &p,
            r#"{"event_id":"ev2","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"grandma jo"}"#,
        )
        .await
        .expect("name-authorized check-out allowed");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["event_id"], "ev2");
    assert!(cp
        .records()
        .read("attendance_event", "ev2")
        .await
        .unwrap()
        .is_some());
}

#[tokio::test]
async fn a_stranger_is_hard_denied_with_localized_reason() {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store.clone(), "acme");
    seed_child(&store, "acme", "leo", json!([{"name": "Grandma Jo"}])).await;
    seed_edge(&store, "acme", "e1", "user:sam", "leo", true, None).await;

    let p = admin(&key, "acme");
    // Stranger in English.
    let err_en = run(
            &cp,
            &p,
            r#"{"event_id":"evX","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Mallory","collector_sub":"user:mallory","locale":"en"}"#,
        )
        .await
        .expect_err("stranger denied");
    // No event appended on a deny.
    assert!(cp
        .records()
        .read("attendance_event", "evX")
        .await
        .unwrap()
        .is_none());

    // Stranger in Spanish — the message MUST differ (a Spanish teacher reads why).
    let err_es = run(
            &cp,
            &p,
            r#"{"event_id":"evY","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Mallory","collector_sub":"user:mallory","locale":"es"}"#,
        )
        .await
        .expect_err("stranger denied es");

    assert_ne!(err_en, err_es, "deny reason must localize");
    assert!(
        err_es.contains("autorizada"),
        "spanish not_authorized text: {err_es}"
    );
    assert!(cp
        .records()
        .read("attendance_event", "evY")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn custody_hold_denies_even_can_pickup_guardian() {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store.clone(), "acme");
    seed_child(&store, "acme", "leo", json!([])).await;
    seed_edge(
        &store,
        "acme",
        "e1",
        "user:sam",
        "leo",
        true,
        Some("court order"),
    )
    .await;

    // A can_pickup guardian — but a custody hold denies unless admin override.
    let member_p = member(&key, "user:sam", "acme");
    let denied = run(
            &cp,
            &member_p,
            r#"{"event_id":"evH","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Sam Parent","collector_sub":"user:sam"}"#,
        )
        .await;
    assert!(denied.is_err(), "custody hold denies without override");
    assert!(cp
        .records()
        .read("attendance_event", "evH")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn admin_override_on_denied_collector_is_allowed_and_audited() {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store.clone(), "acme");
    seed_child(&store, "acme", "leo", json!([])).await;
    seed_edge(
        &store,
        "acme",
        "e1",
        "user:sam",
        "leo",
        true,
        Some("court order"),
    )
    .await;

    let p = admin(&key, "acme");
    let out = run(
            &cp,
            &p,
            r#"{"event_id":"evO","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Sam Parent","collector_sub":"user:sam","pickup_override":true}"#,
        )
        .await
        .expect("admin override allowed");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["event_id"], "evO");
    let ev = cp
        .records()
        .read("attendance_event", "evO")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ev["pickup_override"], true);
    assert_eq!(ev["override_reason"], "custody_hold");
}

#[tokio::test]
async fn non_admin_override_is_still_denied() {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store.clone(), "acme");
    seed_child(&store, "acme", "leo", json!([])).await;
    seed_edge(
        &store,
        "acme",
        "e1",
        "user:sam",
        "leo",
        true,
        Some("court order"),
    )
    .await;

    // A Member sets pickup_override:true — the override is admin-capped, so
    // they are STILL denied and no event is written.
    let member_p = member(&key, "user:sam", "acme");
    let denied = run(
            &cp,
            &member_p,
            r#"{"event_id":"evN","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Sam Parent","collector_sub":"user:sam","pickup_override":true}"#,
        )
        .await;
    assert!(denied.is_err(), "non-admin override must be denied");
    assert!(cp
        .records()
        .read("attendance_event", "evN")
        .await
        .unwrap()
        .is_none());
}
