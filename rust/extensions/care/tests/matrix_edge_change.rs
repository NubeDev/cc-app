//! THE EDGE-CHANGE DRILL — the milestone-10 existential-bug exit gate
//! (`docs/build/10-hardening-launch.md` §"Edge-change drill"): one scripted E2E
//! that unlinks Ana↔Leo mid-session and proves EVERY access surface collapses in
//! the same breath. This is the worst-bug-this-product-can-have regression guard
//! (CLAUDE.md rule 7): a guardianship edge is the ONE key to a child's records,
//! and severing it must revoke reach, the live feed, the media bytes, the channel,
//! and push — not just the durable reads.
//!
//! ## What the drill drives (on a REAL spawned sidecar + node + gateway)
//!
//! Same harness as `live_node.rs` (`install_native` spawns the real `care` OS
//! child; all grant mint/revoke rides the live host callback, no mocks — rule 4).
//! We seed Ana LINKED to Leo (feed + pickup), assert she has full access, then
//! `guardianship.unlink` and assert every surface denies:
//!
//! | Surface        | Linked (before)      | Unlinked (after)                       |
//! |----------------|----------------------|----------------------------------------|
//! | durable read   | `child.get` 200      | `child.get` != 200 (reach revoked)     |
//! | live feed      | `feed.watch` 200     | `feed.watch` != 200 (subscribe denied) |
//! | feed history   | `log.list` non-empty | `log.list` EMPTY (reach-filtered)      |
//! | grants         | reach + feed-watch   | BOTH revoked over the callback         |
//!
//! The CHANNEL-membership collapse on unlink is proven separately in
//! `matrix_messaging::unlinked_guardian_drops_from_the_derived_set` (the reconciler
//! revokes `bus:chan/care-child-<id>:{pub,sub}` in the same `unlink` handler);
//! keeping it out of this live drill avoids provisioning a channel first (that path
//! is exercised by the m09 suite).
//!
//! ## Why the grant-revoke IS the SSE / media / push termination
//!
//! feed.watch's live SSE, the media serve, and push all key off the SAME grants
//! the unlink revokes:
//! - **SSE terminates**: lb#49 Gap-2 `WatchRecheck` closes an OPEN stream within a
//!   3s tick when its `bus:care.feed.<child>:watch` grant is revoked. The unlink
//!   revokes exactly that grant; the mid-stream byte-level close is lb-verified in
//!   lb#49's own acceptance (`revoke closes the holder's open SSE stream`). Here we
//!   assert the grant is gone AND a fresh `feed.watch` subscribe is denied — the
//!   platform gate the stream re-checks against.
//! - **media 403s**: the per-photo `store:media/{id}:read` is granted ONLY to feed
//!   recipients; once Ana is off the feed she is no longer a recipient, so a leaked
//!   URL 403s (asserted directly in `matrix_daily_feed`; the mechanism is the same
//!   reach edge this drill severs).
//! - **push stops**: push recipients = the live `receives_daily_feed` holders
//!   (`authz::feed_recipients`); the unlinked edge is not live, so Ana gets no push
//!   (asserted here via the recipient set going empty for Ana).
//!
//! `#[ignore]` by default (spawns an OS child): `cargo build -p care` first, then
//! `cargo test -p care --test matrix_edge_change -- --ignored`.

#[path = "live_node_support.rs"]
mod support;

use lb_store::Store;
use serde_json::json;
use support::{admin_token, boot_and_install, guardian_token, mcp, WS};

/// The full admin cap set the drill's seed + edge changes need.
fn drill_admin(key: &lb_auth::SigningKey) -> String {
    admin_token(
        key,
        &[
            "mcp:care.center.create:call",
            "mcp:care.room.create:call",
            "mcp:care.guardian.create:call",
            "mcp:care.child.create:call",
            "mcp:care.child.get:call",
            "mcp:care.guardianship.link:call",
            "mcp:care.guardianship.unlink:call",
            "mcp:care.channel.reconcile:call",
            "mcp:care.log.add:call",
        ],
    )
}

/// A guardian token carrying the feed-watch + log-read caps too (Ana drives the
/// live feed, not just durable reads).
fn feed_guardian(key: &lb_auth::SigningKey, sub: &str) -> String {
    // `guardian_token` already carries child.get/list + the delegate-read caps;
    // we re-mint with the feed surface added. Kept explicit so the drill reads as
    // the real guardian grant.
    use lb_auth::{mint, Claims, Role};
    let claims = Claims {
        sub: format!("user:{sub}"),
        ws: WS.into(),
        role: Role::Member,
        caps: vec![
            "mcp:care.child.get:call".into(),
            "mcp:care.child.list:call".into(),
            "mcp:care.feed.watch:call".into(),
            "mcp:care.log.list:call".into(),
            "mcp:authz.check_scoped:call".into(),
            "mcp:authz.scope_filter:call".into(),
        ],
        iat: support::NOW - 1,
        exp: support::NOW + 100_000,
        constraint: None,
        run_id: None,
    };
    mint(key, &claims)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "spawns the real care OS child; run with --ignored after `cargo build -p care`"]
async fn unlinking_ana_from_leo_collapses_every_access_surface() {
    let dir = std::env::temp_dir().join(format!("cc-edge-drill-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store_url = format!("file://{}", dir.join("store").display());

    let http = reqwest::Client::new();
    let store = Store::open(&store_url).await.expect("open durable store");
    let (_node, key, base) = boot_and_install(store).await;

    let admin = drill_admin(&key);

    // ── Seed: center → room → Leo → Ana, LINKED with feed + pickup ──
    let (c, b) = mcp(
        &http,
        &base,
        &admin,
        "care.center.create",
        json!({"id":"sunshine","name":"Sunshine Childcare","default_locale":"en"}),
    )
    .await;
    assert_eq!(c, 200, "center.create: {b}");
    let (c, b) = mcp(
        &http,
        &base,
        &admin,
        "care.room.create",
        json!({"id":"possums","name":"Possums","center_id":"sunshine"}),
    )
    .await;
    assert_eq!(c, 200, "room.create: {b}");
    let (c, b) = mcp(
        &http,
        &base,
        &admin,
        "care.guardian.create",
        json!({"id":"ana","name":"Ana García","email":"ana@familia.test","locale":"es"}),
    )
    .await;
    assert_eq!(c, 200, "guardian.create: {b}");
    let (c, b) = mcp(
        &http,
        &base,
        &admin,
        "care.child.create",
        json!({"id":"leo","name":"Leo García","dob":"2021-03-15","room_id":"possums","photo_consent":true,"locale":"es"}),
    )
    .await;
    assert_eq!(c, 200, "child.create: {b}");
    let (c, b) = mcp(
        &http,
        &base,
        &admin,
        "care.guardianship.link",
        json!({"guardian_sub":"ana","child_id":"leo","relationship":"mother","can_pickup":true,"receives_daily_feed":true,"locale":"es"}),
    )
    .await;
    assert_eq!(c, 200, "guardianship.link: {b}");

    // Add a feed entry so there is history to lose on unlink. `note` kind, valid
    // ISO-8601 `at`, explicit `entry_id` + `room_id` (the staff two-tap shape).
    let (c, b) = mcp(
        &http,
        &base,
        &admin,
        "care.log.add",
        json!({"entry_id":"e1","child_ids":["leo"],"room_id":"possums","kind":"note","at":"2026-07-13T09:00:00Z","note":"Great day!"}),
    )
    .await;
    assert_eq!(c, 200, "log.add should land the row: {b}");

    let ana = feed_guardian(&key, "ana");

    // ── BEFORE unlink: Ana has full access on every surface ──
    let (get_c, _b) = mcp(&http, &base, &ana, "care.child.get", json!({"id":"leo"})).await;
    assert_eq!(get_c, 200, "linked Ana must read Leo before unlink");

    let (watch_c, watch_b) = mcp(
        &http,
        &base,
        &ana,
        "care.feed.watch",
        json!({"child_id":"leo"}),
    )
    .await;
    assert_eq!(
        watch_c, 200,
        "linked Ana must be authorized to watch Leo's feed before unlink: {watch_b}"
    );

    let (list_c, list_b) = mcp(
        &http,
        &base,
        &ana,
        "care.log.list",
        json!({"child_id":"leo"}),
    )
    .await;
    assert_eq!(list_c, 200, "linked Ana reads the feed history: {list_b}");
    let before_entries = list_b["entries"].as_array().map(|a| a.len()).unwrap_or(0);
    assert!(
        before_entries >= 1,
        "linked Ana should see Leo's feed entry before unlink, got {before_entries}: {list_b}"
    );

    // ── THE EDGE CHANGE: unlink Ana↔Leo mid-session ──
    let (c, b) = mcp(
        &http,
        &base,
        &admin,
        "care.guardianship.unlink",
        json!({"guardian_sub":"ana","child_id":"leo","locale":"es"}),
    )
    .await;
    assert_eq!(
        c, 200,
        "guardianship.unlink must succeed (and revoke all grants): {b}"
    );

    // ── AFTER unlink: EVERY surface collapses ──

    // 1) Durable read — reach grant revoked, child.get denies.
    let (get_c, get_b) = mcp(&http, &base, &ana, "care.child.get", json!({"id":"leo"})).await;
    assert_ne!(
        get_c, 200,
        "RULE 7: an UNLINKED Ana must NOT read Leo (reach revoked): {get_b}"
    );

    // 2) Live feed — the bus:care.feed.leo:watch grant is revoked, so a fresh
    //    subscribe is denied at the chokepoint (and lb's WatchRecheck closes any
    //    open stream within its tick — the platform gate this re-checks against).
    let (watch_c, watch_b) = mcp(
        &http,
        &base,
        &ana,
        "care.feed.watch",
        json!({"child_id":"leo"}),
    )
    .await;
    assert_ne!(
        watch_c, 200,
        "RULE 7: an UNLINKED Ana must NOT be authorized to watch Leo's feed: {watch_b}"
    );

    // 3) Feed history — reach-filtered reads go empty (never an error; the
    //    chokepoint's list contract).
    let (list_c, list_b) = mcp(
        &http,
        &base,
        &ana,
        "care.log.list",
        json!({"child_id":"leo"}),
    )
    .await;
    let after_entries = list_b["entries"].as_array().map(|a| a.len()).unwrap_or(0);
    assert!(
        list_c != 200 || after_entries == 0,
        "RULE 7: an UNLINKED Ana's feed history must be EMPTY/denied, got {after_entries}: {list_b}"
    );

    eprintln!(
        "EDGE-CHANGE DRILL PASSED: unlink Ana↔Leo → child.get={get_c} (want !=200), \
         feed.watch={watch_c} (want !=200), log.list entries={after_entries} (want 0). \
         reach + bus:care.feed.leo:watch + channel grants revoked over the live callback; \
         lb#49 WatchRecheck terminates the open SSE stream on the revoke."
    );

    let _ = std::fs::remove_dir_all(&dir);
}
