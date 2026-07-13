//! Cross-family matrix — the milestone-08 daily-feed READ verbs against the
//! canonical two-family fixture (CLAUDE.md rule 7 / daily-feed-scope §"Testing
//! plan": every read verb ships a cross-family row). The feed is the guardian's
//! highest-frequency surface and carries photos + incidents + medications — a
//! leak across families here is the worst bug this product can have.
//!
//! The rows this harness asserts (daily-feed-scope §"Testing plan" + the m08
//! exit gate):
//!  - `log.list` — a guardian sees ONLY her reached children's entries; a
//!    stranger gets EMPTY (deny-by-empty), never another family's rows.
//!  - `log.day` — a guardian's rollup of HER child is allowed; the OTHER
//!    family's child is DENIED (403), with no leak in the error.
//!  - `feed.watch` — a stranger's subscribe is DENIED (never handed a subject).
//!  - THE MEDIA-URL LEAK — a record filtered out of a stranger's list/day must
//!    also keep its photo `media_id` out of that caller's reach: we assert the
//!    other family's `media_id` never appears in a stranger's list/day output
//!    (the guessable-URL class — the durable side of the reach-checked serve).
//!  - The incident push keys resolve in BOTH `en` and `es` (the both-languages
//!    exit gate; localization is lb's job per recipient, so cc-app asserts the
//!    catalog renders `push.{title,body}.incident` in each language and that
//!    `push::decide` targets both an en and an es recipient must-deliver).
//!
//! Era 1 (store-resolved) is the live reach path; driven via `Chokepoint::new`,
//! the same posture as `matrix_menu_reads.rs`.

mod common;

use care::authz::Chokepoint;
use care::center::Locale;
use care::i18n::t;
use care::log::{day, list, LogKind};
use care::push::decide;
use care::feed::watch;
use lb_auth::Role;
use lb_store::{create as store_create, Store};
use std::sync::Arc;

use common::{principal, ANA, LEO, MIA, MIAS_MUM, POSS, WS};

const DAY: &str = "2026-07-14";
const LEO_MEDIA: &str = "media:leo-photo";
const MIA_MEDIA: &str = "media:mia-photo";

/// Seed the two-family feed fixture via the store write path:
///  - Leo reached by Ana; Mia reached ONLY by Mia's mum (Ana ↛ Mia).
///  - one photo entry per child on DAY, each carrying a DISTINCT `media_id`
///    (so a leak of Mia's row would surface `media:mia-photo` to Ana).
///  - one incident entry for Leo (the always-push case).
async fn seed() -> (Arc<Store>, lb_auth::SigningKey) {
    let store = Arc::new(Store::memory().await.expect("mem"));
    let key = lb_auth::SigningKey::generate();

    for (id, room) in [(LEO, POSS), (MIA, "room:koalas")] {
        store_create(
            &store,
            WS,
            "child",
            id,
            &serde_json::json!({
                "name": id, "dob": "2021-03-15", "allergies": [], "room_id": room,
                "immunizations": [], "emergency_contacts": [], "authorized_pickups": [],
                "photo_consent": true, "archived": false
            }),
        )
        .await
        .expect("seed child");
    }

    // Live guardianship edges (feed edge on for both). Ana↛Mia is the point.
    for (g, c) in [(ANA, LEO), (MIAS_MUM, MIA)] {
        store_create(
            &store,
            WS,
            "guardianship",
            &[g, c].join("::"),
            &serde_json::json!({
                "guardian_sub": g, "child_id": c, "live": true,
                "receives_daily_feed": true
            }),
        )
        .await
        .expect("seed edge");
    }

    // One photo entry per child (distinct media ids) + a Leo incident.
    seed_entry(&store, "log:leo:photo::child:leo", LEO, POSS, "photo", &[LEO_MEDIA]).await;
    seed_entry(&store, "log:mia:photo::child:mia", MIA, "room:koalas", "photo", &[MIA_MEDIA]).await;
    seed_incident(&store, "log:leo:inc::child:leo", LEO, POSS).await;

    (store, key)
}

async fn seed_entry(store: &Arc<Store>, id: &str, child: &str, room: &str, kind: &str, media: &[&str]) {
    let at = [DAY, "T10:00:00Z"].concat();
    store_create(
        store,
        WS,
        "daily_log",
        id,
        &serde_json::json!({
            "kind": kind, "child_id": child, "room_id": room, "author": "user:teacher",
            "at": at, "media_ids": media
        }),
    )
    .await
    .expect("seed entry");
}

async fn seed_incident(store: &Arc<Store>, id: &str, child: &str, room: &str) {
    let at = [DAY, "T15:10:00Z"].concat();
    store_create(
        store,
        WS,
        "daily_log",
        id,
        &serde_json::json!({
            "kind": "incident", "child_id": child, "room_id": room, "author": "user:teacher",
            "at": at,
            "incident": {"what": "bump", "where": "gym", "action": "iced", "acknowledged": false}
        }),
    )
    .await
    .expect("seed incident");
}

// ---- log.list -------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn guardian_list_sees_only_her_childs_entries() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let ana = principal(&key, ANA, WS, Role::Member, &["mcp:care.log.list:call"]);

    let out = list::run(&cp, &ana, "{}").await.expect("Ana's list");
    // Ana reaches Leo only: she must see Leo's rows and NEVER Mia's media id.
    assert!(out.contains(LEO), "Ana must see Leo's entries");
    assert!(!out.contains(MIA), "MUST NOT leak Mia's entries across families: {out}");
    assert!(!out.contains(MIA_MEDIA), "MUST NOT leak Mia's photo media id: {out}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn stranger_list_is_empty() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let stranger = principal(&key, "user:stranger", WS, Role::Member, &["mcp:care.log.list:call"]);

    let out = list::run(&cp, &stranger, "{}").await.expect("empty, not error");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["entries"].as_array().unwrap().len(), 0, "deny-by-empty");
    assert!(!out.contains(LEO_MEDIA) && !out.contains(MIA_MEDIA), "no media leak: {out}");
}

// ---- log.day + THE MEDIA-URL LEAK ----------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn guardian_day_allows_her_child_denies_the_other() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let ana = principal(&key, ANA, WS, Role::Member, &["mcp:care.log.day:call"]);

    // ALLOW: Ana's own child's rollup, and it carries Leo's media id (she reaches it).
    let leo = day::run(&cp, &ana, &day_input(LEO)).await.expect("Ana reaches Leo");
    assert!(leo.contains(LEO_MEDIA), "Ana's Leo rollup carries Leo's photo");

    // DENY: Mia's rollup is a 403 with NO leak of Mia's media id / room / child.
    let err = day::run(&cp, &ana, &day_input(MIA))
        .await
        .expect_err("rule 7: Ana ↛ Mia — day must DENY");
    assert!(!err.contains(MIA_MEDIA), "THE MEDIA-URL LEAK: Mia's photo id must not leak: {err}");
    assert!(!err.contains("koalas"), "must not leak Mia's room: {err}");
}

// ---- feed.watch -----------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn stranger_feed_watch_is_denied() {
    let (store, key) = seed().await;
    let cp = Chokepoint::new(store, WS);
    let stranger = principal(&key, "user:stranger", WS, Role::Member, &["mcp:care.feed.watch:call"]);

    let res = watch::run(&cp, &stranger, &serde_json::json!({"child_id": LEO}).to_string()).await;
    assert!(res.is_err(), "a stranger's subscribe must be denied (never handed a subject)");
    // A linked guardian IS handed the subject.
    let ana = principal(&key, ANA, WS, Role::Member, &["mcp:care.feed.watch:call"]);
    let ok = watch::run(&cp, &ana, &serde_json::json!({"child_id": LEO}).to_string())
        .await
        .expect("Ana reaches Leo — watch authorized");
    assert!(ok.contains("care.feed.child:leo"), "Ana gets Leo's subject");
}

// ---- the incident push, both languages ------------------------------------

/// The both-languages exit gate. Localization is lb's job per recipient, so the
/// cc-app assertion is: (1) `push::decide` for an incident targets BOTH an en
/// and an es recipient, must-deliver, with the incident catalog keys; (2) those
/// keys render in BOTH `en` and `es` — proving the same incident yields Sam
/// English and Ana Spanish from ONE `notify.send` (lb renders each side).
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn incident_push_targets_both_languages_with_resolvable_keys() {
    // Sam (en) + Ana (es) both hold the feed edge → both are push recipients.
    let recipients = vec!["user:sam".to_string(), "user:ana".to_string()];
    let d = decide(LogKind::Incident, &recipients, "log:leo:inc::child:leo");
    assert!(d.is_push() && d.must_deliver, "an incident is a must-deliver push");
    assert_eq!(d.recipients.len(), 2, "both language recipients targeted from one send");
    assert_eq!(d.title_key, "push.title.incident");
    assert_eq!(d.body_key, "push.body.incident");

    // The keys resolve in BOTH languages (the words differ — proving each
    // recipient's locale renders server-side; cc-app only ships the keys).
    let en_title = t(Locale::En, &d.title_key, &[("child", "Leo")]);
    let es_title = t(Locale::Es, &d.title_key, &[("child", "Leo")]);
    let en_body = t(Locale::En, &d.body_key, &[("child", "Leo")]);
    let es_body = t(Locale::Es, &d.body_key, &[("child", "Leo")]);
    assert!(en_title.contains("Leo") && es_title.contains("Leo"), "child interpolated both langs");
    assert_ne!(en_title, es_title, "en and es incident titles differ (real localization)");
    assert_ne!(en_body, es_body, "en and es incident bodies differ");
    // Not the raw key (a missing catalog entry degrades to the key itself).
    assert_ne!(en_body, d.body_key, "en body resolved to words, not the key");
    assert_ne!(es_body, d.body_key, "es body resolved to words, not the key");
}

fn day_input(child: &str) -> String {
    serde_json::json!({"child_id": child, "date": DAY}).to_string()
}
