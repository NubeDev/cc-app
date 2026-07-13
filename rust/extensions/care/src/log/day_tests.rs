use super::*;
use crate::child::create as child_create;
use crate::guardianship::link as guardianship_link;
use crate::log::add as log_add;
use lb_auth::{mint, verify, Claims, Role, SigningKey};
use lb_store::Store;
use std::sync::Arc;

/// One parametric principal builder — role + sub + caps.
fn principal(signing: &SigningKey, sub: &str, ws: &str, role: Role, caps: &[&str]) -> Principal {
    let claims = Claims {
        sub: sub.into(),
        ws: ws.into(),
        role,
        caps: caps.iter().map(|c| c.to_string()).collect(),
        iat: 0,
        exp: u64::MAX,
        constraint: None,
        run_id: None,
    };
    verify(signing, &mint(signing, &claims), 1).expect("verify")
}
fn staff(k: &SigningKey, ws: &str) -> Principal {
    principal(
        k,
        "user:teacher",
        ws,
        Role::Member,
        &["mcp:care.log.add:call"],
    )
}
fn admin(k: &SigningKey, ws: &str) -> Principal {
    let caps = [
        "mcp:care.child.create:call",
        "mcp:care.guardianship.link:call",
        "mcp:care.log.day:call",
    ];
    principal(k, "user:admin", ws, Role::WorkspaceAdmin, &caps)
}
fn guardian(k: &SigningKey, sub: &str, ws: &str) -> Principal {
    principal(k, sub, ws, Role::Member, &["mcp:care.log.day:call"])
}

async fn seed_child(cp: &Chokepoint, a: &Principal, id: &str) {
    let input = format!(r#"{{"id":"{id}","name":"{id}","dob":"2021-03-15","photo_consent":true}}"#);
    child_create::run(cp, a, &input).await.expect("seed child");
}

/// Seed one entry via `log::add`; the row lands at `<base>::<child_id>`.
async fn seed_entry(
    cp: &Chokepoint,
    p: &Principal,
    base: &str,
    child_id: &str,
    kind: &str,
    at: &str,
    extra: &str,
) {
    let input = format!(
        r#"{{"entry_id":"{base}","child_ids":["{child_id}"],"room_id":"room:possums","kind":"{kind}","at":"{at}"{extra}}}"#
    );
    log_add::run(cp, p, &input).await.expect("seed entry");
}

#[tokio::test]
async fn day_rollup_for_admin_counts_by_type() {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store, "acme");
    let a = admin(&key, "acme");
    seed_child(&cp, &a, "child:leo").await;

    let s = staff(&key, "acme");
    // A meal + a nap + an incident for Leo on 2026-07-14.
    seed_entry(
        &cp,
        &s,
        "log:meal:1",
        "child:leo",
        "meal",
        "2026-07-14T11:30:00Z",
        r#","meal":{"slot":"lunch","portion":"most"}"#,
    )
    .await;
    seed_entry(
        &cp,
        &s,
        "log:nap:1",
        "child:leo",
        "nap",
        "2026-07-14T13:00:00Z",
        r#","nap":{"start":"2026-07-14T13:00:00Z"}"#,
    )
    .await;
    seed_entry(
        &cp,
        &s,
        "log:inc:1",
        "child:leo",
        "incident",
        "2026-07-14T15:10:00Z",
        r#","incident":{"what":"scraped knee","where":"playground","action":"cleaned"}"#,
    )
    .await;

    let out = run(&cp, &a, r#"{"child_id":"child:leo","date":"2026-07-14"}"#)
        .await
        .expect("rollup");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["entries"].as_array().unwrap().len(), 3);
    assert_eq!(v["summary"]["meal"], 1);
    assert_eq!(v["summary"]["nap"], 1);
    assert_eq!(v["summary"]["incident"], 1);
    // Sparse: a kind with no rows is absent from the tally.
    assert!(v["summary"].get("diaper").is_none());
    // Timeline order (ascending by `at`).
    assert_eq!(v["entries"][0]["kind"], "meal");
    assert_eq!(v["entries"][2]["kind"], "incident");
}

/// RULE 7: a guardian with NO edge to Leo is DENIED (never a phantom empty
/// rollup); a linked guardian gets the rollup.
#[tokio::test]
async fn stranger_guardian_is_denied() {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store, "acme");
    let a = admin(&key, "acme");
    seed_child(&cp, &a, "child:leo").await;

    let s = staff(&key, "acme");
    seed_entry(
        &cp,
        &s,
        "log:note:1",
        "child:leo",
        "note",
        "2026-07-14T09:00:00Z",
        r#","note":"settled in well""#,
    )
    .await;

    // A stranger (no edge) is denied — fail closed, never an empty rollup.
    let stranger = guardian(&key, "user:stranger", "acme");
    let denied = run(
        &cp,
        &stranger,
        r#"{"child_id":"child:leo","date":"2026-07-14"}"#,
    )
    .await;
    assert!(denied.is_err(), "a stranger MUST be denied, not shown []");

    // Link Ana → Leo; now she reaches the rollup.
    guardianship_link::run(
        &cp,
        &a,
        r#"{"guardian_sub":"user:ana","child_id":"child:leo","relationship":"mother"}"#,
    )
    .await
    .expect("link ana");
    let ana = guardian(&key, "user:ana", "acme");
    let out = run(&cp, &ana, r#"{"child_id":"child:leo","date":"2026-07-14"}"#)
        .await
        .expect("linked guardian sees the rollup");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["entries"].as_array().unwrap().len(), 1);
    assert_eq!(v["summary"]["note"], 1);
}

#[tokio::test]
async fn only_the_asked_date_is_included() {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store, "acme");
    let a = admin(&key, "acme");
    seed_child(&cp, &a, "child:leo").await;

    let s = staff(&key, "acme");
    seed_entry(
        &cp,
        &s,
        "log:d1",
        "child:leo",
        "note",
        "2026-07-14T09:00:00Z",
        r#","note":"today""#,
    )
    .await;
    // A row on a DIFFERENT date — must be excluded.
    seed_entry(
        &cp,
        &s,
        "log:d2",
        "child:leo",
        "note",
        "2026-07-15T09:00:00Z",
        r#","note":"tomorrow""#,
    )
    .await;

    let out = run(&cp, &a, r#"{"child_id":"child:leo","date":"2026-07-14"}"#)
        .await
        .expect("rollup");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v["entries"].as_array().unwrap().len(),
        1,
        "only the asked date"
    );
    assert_eq!(v["entries"][0]["note"], "today");
}

#[tokio::test]
async fn only_the_asked_child_is_included() {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store, "acme");
    let a = admin(&key, "acme");
    seed_child(&cp, &a, "child:leo").await;
    seed_child(&cp, &a, "child:mia").await;

    let s = staff(&key, "acme");
    seed_entry(
        &cp,
        &s,
        "log:leo",
        "child:leo",
        "note",
        "2026-07-14T09:00:00Z",
        r#","note":"leo""#,
    )
    .await;
    // A row for Mia — Leo's rollup must exclude it.
    seed_entry(
        &cp,
        &s,
        "log:mia",
        "child:mia",
        "note",
        "2026-07-14T09:00:00Z",
        r#","note":"mia""#,
    )
    .await;

    let out = run(&cp, &a, r#"{"child_id":"child:leo","date":"2026-07-14"}"#)
        .await
        .expect("rollup");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["entries"].as_array().unwrap().len(), 1, "only Leo's rows");
    assert_eq!(v["entries"][0]["child_id"], "child:leo");
}

/// A correction supersedes its original (dropped from the net timeline; the
/// compensating row remains). Seeded directly via the store by writing a row
/// whose `correction_of` points at the original's derived id.
#[tokio::test]
async fn a_corrected_original_is_dropped_from_the_net_view() {
    use crate::log::entry_id;
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store, "acme");
    let a = admin(&key, "acme");
    seed_child(&cp, &a, "child:leo").await;

    let s = staff(&key, "acme");
    // Original meal ("some") lands at `log:meal:orig::child:leo`.
    seed_entry(
        &cp,
        &s,
        "log:meal:orig",
        "child:leo",
        "meal",
        "2026-07-14T11:30:00Z",
        r#","meal":{"slot":"lunch","portion":"some"}"#,
    )
    .await;
    let original_id = entry_id("log:meal:orig", "child:leo");

    // A compensating correction row referencing the original's id.
    let correction = DailyLog {
        kind: LogKind::Meal,
        child_id: "child:leo".into(),
        room_id: "room:possums".into(),
        author: "user:teacher".into(),
        at: "2026-07-14T11:45:00Z".into(),
        note: None,
        media_ids: Vec::new(),
        nap: None,
        meal: Some(crate::log::MealPayload {
            slot: "lunch".into(),
            portion: "all".into(),
        }),
        incident: None,
        medication: None,
        correction_of: Some(original_id),
    };
    let cv = serde_json::to_value(&correction).unwrap();
    cp.records()
        .create("daily_log", "log:meal:fix::child:leo", &cv)
        .await
        .expect("seed correction");

    let out = run(&cp, &a, r#"{"child_id":"child:leo","date":"2026-07-14"}"#)
        .await
        .expect("rollup");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    // The original is superseded; only the compensating row survives.
    assert_eq!(
        v["entries"].as_array().unwrap().len(),
        1,
        "original dropped"
    );
    assert_eq!(v["entries"][0]["meal"]["portion"], "all");
    assert_eq!(v["summary"]["meal"], 1);
}
