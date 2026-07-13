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
fn admin(k: &SigningKey, ws: &str) -> Principal {
    let caps = [
        "mcp:care.child.create:call",
        "mcp:care.guardianship.link:call",
        "mcp:care.log.list:call",
    ];
    principal(k, "user:admin", ws, Role::WorkspaceAdmin, &caps)
}
fn staff(k: &SigningKey, ws: &str) -> Principal {
    principal(
        k,
        "user:teacher",
        ws,
        Role::Member,
        &["mcp:care.log.add:call", "mcp:care.log.list:call"],
    )
}
fn member(k: &SigningKey, sub: &str, ws: &str) -> Principal {
    principal(k, sub, ws, Role::Member, &["mcp:care.log.list:call"])
}

async fn seed_child(cp: &Chokepoint, a: &Principal, id: &str) {
    let input = format!(r#"{{"id":"{id}","name":"{id}","dob":"2021-03-15","photo_consent":true}}"#);
    child_create::run(cp, a, &input).await.expect("seed child");
}

/// Add one `note` entry for `child_id` in `room_id` at `at`, via the real
/// write path (`log::add`), using a staff principal.
async fn seed_entry(
    cp: &Chokepoint,
    p: &Principal,
    entry_id: &str,
    child_id: &str,
    room_id: &str,
    at: &str,
) {
    let input = format!(
        r#"{{"entry_id":"{entry_id}","child_ids":["{child_id}"],"room_id":"{room_id}","kind":"note","at":"{at}","note":"hi"}}"#
    );
    log_add::run(cp, p, &input).await.expect("seed entry");
}

/// Two families: Sam→(Leo, Mia); Ana→Leo. Leo in Possums, Mia in Wombats.
/// Entries: 2 for Leo, 1 for Mia.
async fn seed_two_families() -> (Arc<Store>, SigningKey) {
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store.clone(), "acme");
    let a = admin(&key, "acme");
    let s = staff(&key, "acme");

    seed_child(&cp, &a, "child:leo").await;
    seed_child(&cp, &a, "child:mia").await;

    for input in [
        r#"{"guardian_sub":"user:sam","child_id":"child:leo","relationship":"father"}"#,
        r#"{"guardian_sub":"user:sam","child_id":"child:mia","relationship":"father"}"#,
        r#"{"guardian_sub":"user:ana","child_id":"child:leo","relationship":"mother"}"#,
    ] {
        guardianship_link::run(&cp, &a, input).await.expect("link");
    }

    seed_entry(
        &cp,
        &s,
        "log:leo:1",
        "child:leo",
        "room:possums",
        "2026-07-13T08:02:00Z",
    )
    .await;
    seed_entry(
        &cp,
        &s,
        "log:leo:2",
        "child:leo",
        "room:possums",
        "2026-07-13T15:30:00Z",
    )
    .await;
    seed_entry(
        &cp,
        &s,
        "log:mia:1",
        "child:mia",
        "room:wombats",
        "2026-07-13T08:10:00Z",
    )
    .await;

    (store, key)
}

fn entries_of(out: &str) -> Vec<serde_json::Value> {
    let v: serde_json::Value = serde_json::from_str(out).unwrap();
    v["entries"].as_array().cloned().unwrap_or_default()
}

#[tokio::test]
async fn admin_lists_all_entries() {
    let (store, key) = seed_two_families().await;
    let cp = Chokepoint::new(store, "acme");
    let a = admin(&key, "acme");

    let out = run(&cp, &a, "").await.unwrap();
    let e = entries_of(&out);
    assert_eq!(e.len(), 3, "admin sees every family's entries");
    // Sorted ascending by `at`.
    assert_eq!(e[0]["at"], "2026-07-13T08:02:00Z");
    assert_eq!(e[1]["at"], "2026-07-13T08:10:00Z");
    assert_eq!(e[2]["at"], "2026-07-13T15:30:00Z");
}

/// RULE 7 CROSS-FAMILY ROW: Ana reaches Leo only — she must see Leo's two
/// entries and NEVER Mia's. A leak here is the worst bug this product has.
#[tokio::test]
async fn guardian_sees_only_reached_childrens_entries() {
    let (store, key) = seed_two_families().await;
    let cp = Chokepoint::new(store, "acme");
    let ana = member(&key, "user:ana", "acme");

    let out = run(&cp, &ana, "").await.unwrap();
    let e = entries_of(&out);
    assert_eq!(e.len(), 2, "Ana sees Leo's two entries only");
    for row in &e {
        assert_eq!(row["child_id"], "child:leo");
        assert_ne!(
            row["child_id"], "child:mia",
            "MUST NOT leak Mia across families"
        );
    }
}

#[tokio::test]
async fn guardian_with_no_reach_gets_empty_not_error() {
    let (store, key) = seed_two_families().await;
    let cp = Chokepoint::new(store, "acme");
    // A guardian with NO edge to any child reaches nothing.
    let stranger = member(&key, "user:stranger", "acme");

    let out = run(&cp, &stranger, "").await.expect("empty, not error");
    assert_eq!(entries_of(&out).len(), 0, "deny-by-empty, never an error");
}

#[tokio::test]
async fn cursor_is_stable_across_pages() {
    // Five entries for Leo, all reachable by admin; page by 2.
    let store = Arc::new(Store::memory().await.unwrap());
    let key = SigningKey::generate();
    let cp = Chokepoint::new(store.clone(), "acme");
    let a = admin(&key, "acme");
    let s = staff(&key, "acme");
    seed_child(&cp, &a, "child:leo").await;
    for (i, at) in [
        "2026-07-13T08:00:00Z",
        "2026-07-13T09:00:00Z",
        "2026-07-13T10:00:00Z",
        "2026-07-13T11:00:00Z",
        "2026-07-13T12:00:00Z",
    ]
    .iter()
    .enumerate()
    {
        let base = ["log:leo:", &i.to_string()].concat();
        seed_entry(&cp, &s, &base, "child:leo", "room:possums", at).await;
    }

    // Page 1: limit 2 → 2 rows + a next_cursor.
    let out1 = run(&cp, &a, r#"{"limit":2}"#).await.unwrap();
    let v1: serde_json::Value = serde_json::from_str(&out1).unwrap();
    let e1 = v1["entries"].as_array().unwrap();
    assert_eq!(e1.len(), 2);
    assert_eq!(e1[0]["at"], "2026-07-13T08:00:00Z");
    assert_eq!(e1[1]["at"], "2026-07-13T09:00:00Z");
    let cursor1 = v1["next_cursor"].as_str().expect("more rows remain");

    // Page 2: follow the cursor → next 2 rows + another cursor.
    let out2 = run(&cp, &a, &format!(r#"{{"limit":2,"after":"{cursor1}"}}"#))
        .await
        .unwrap();
    let v2: serde_json::Value = serde_json::from_str(&out2).unwrap();
    let e2 = v2["entries"].as_array().unwrap();
    assert_eq!(e2.len(), 2);
    assert_eq!(e2[0]["at"], "2026-07-13T10:00:00Z");
    assert_eq!(e2[1]["at"], "2026-07-13T11:00:00Z");
    let cursor2 = v2["next_cursor"].as_str().expect("one row remains");

    // Page 3 (last): the final row, next_cursor None.
    let out3 = run(&cp, &a, &format!(r#"{{"limit":2,"after":"{cursor2}"}}"#))
        .await
        .unwrap();
    let v3: serde_json::Value = serde_json::from_str(&out3).unwrap();
    let e3 = v3["entries"].as_array().unwrap();
    assert_eq!(e3.len(), 1);
    assert_eq!(e3[0]["at"], "2026-07-13T12:00:00Z");
    assert!(
        v3.get("next_cursor").is_none(),
        "last page has no next_cursor"
    );
}

#[tokio::test]
async fn child_id_filter_narrows_within_authorized_set() {
    let (store, key) = seed_two_families().await;
    let cp = Chokepoint::new(store, "acme");
    let sam = member(&key, "user:sam", "acme");

    // Sam reaches both Leo and Mia (3 entries); filter to Leo → 2 rows.
    let unfiltered = run(&cp, &sam, "").await.unwrap();
    assert_eq!(
        entries_of(&unfiltered).len(),
        3,
        "Sam reaches both children"
    );

    let out = run(&cp, &sam, r#"{"child_id":"child:leo"}"#).await.unwrap();
    let e = entries_of(&out);
    assert_eq!(e.len(), 2, "filter narrows to Leo");
    assert!(e.iter().all(|r| r["child_id"] == "child:leo"));
}
