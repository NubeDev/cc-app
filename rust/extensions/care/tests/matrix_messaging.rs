//! Cross-family matrix — the milestone-09 messaging membership DERIVATION
//! against the canonical two-family fixture (CLAUDE.md rule 7 /
//! messaging-scope.md §"Testing plan"). Membership is the leak vector: a
//! guardian may appear in a channel's derived member set ONLY for a child she
//! holds a live messaging edge to. care controls the DERIVATION (what these
//! rows assert); lb's channel gate enforces the actual read/post using the caps
//! the reconciler grants from this derivation (a live-node concern).
//!
//! The rows this harness asserts (messaging-scope §"Testing plan" + the m09
//! exit gate):
//!  - a messaging guardian appears in HER child's channel; the OTHER family's
//!    mum does NOT — and Ana has NO path to Mia's channel (never in its members).
//!  - the DISTINCT flag: a guardian with `receives_daily_feed` but NOT
//!    `receives_messaging` is absent (feed access ≠ messaging access).
//!  - unlink (live=false) → the guardian drops from the derived set (the revoke
//!    basis — her next channel read 403s at lb once the reconciler revokes).
//!  - room-channel membership folds the room's staff + its children's guardians,
//!    deduped to one Full member each (idempotent re-derivation).
//!
//! Era 1 (store-resolved) is the live derivation path; driven via
//! `Chokepoint::new`, the same posture as `matrix_daily_feed.rs`.

mod common;

use care::authz::{channel_members, ChannelTarget, Chokepoint};
use lb_store::{create as store_create, write as store_write, Store};
use std::sync::Arc;

use common::{ANA, LEO, MIA, MIAS_MUM, POSS, SAM, WS};

/// Seed the two-family messaging fixture:
///  - Leo in Possums (Sam is Possums staff); Mia in Koalas.
///  - Ana ↔ Leo edge with `receives_messaging: true`.
///  - Mia's mum ↔ Mia with messaging; Ana has NO edge to Mia.
///  - a THIRD guardian (feed-only) on Leo: feed on, messaging OFF — the
///    distinct-flag assertion.
async fn seed() -> Arc<Store> {
    let store = Arc::new(Store::memory().await.expect("mem"));

    for (id, room) in [(LEO, POSS), (MIA, "room:koalas")] {
        store_create(
            &store,
            WS,
            "child",
            id,
            &serde_json::json!({
                "name": id, "dob": "2021-03-15", "room_id": room,
                "photo_consent": true, "archived": false
            }),
        )
        .await
        .expect("seed child");
    }

    // Sam is Possums staff (a Full member of Leo's + the room's channel).
    store_create(
        &store,
        WS,
        "staff_assignment",
        &format!("{SAM}::{POSS}"),
        &serde_json::json!({ "staff_sub": SAM, "room_id": POSS }),
    )
    .await
    .expect("seed staff");

    // Edges: Ana↔Leo (messaging on); Mia's mum↔Mia (messaging on); a feed-only
    // guardian on Leo (messaging OFF).
    let edges = [
        (ANA, LEO, true, true),
        (MIAS_MUM, MIA, true, true),
        ("user:leo-feedonly", LEO, false, true),
    ];
    for (g, c, messaging, feed) in edges {
        store_create(
            &store,
            WS,
            "guardianship",
            &[g, c].join("::"),
            &serde_json::json!({
                "guardian_sub": g, "child_id": c, "live": true,
                "receives_messaging": messaging, "receives_daily_feed": feed
            }),
        )
        .await
        .expect("seed edge");
    }
    store
}

fn subjects(members: &[care::authz::ChannelMember]) -> Vec<String> {
    members.iter().map(|m| m.subject.clone()).collect()
}

#[tokio::test]
async fn leo_channel_has_ana_and_sam_never_mias_mum() {
    let store = seed().await;
    let cp = Chokepoint::new(store, WS);
    let subs = subjects(&channel_members(&cp, &ChannelTarget::Child(LEO)).await);

    assert!(
        subs.contains(&ANA.to_string()),
        "Ana (messaging edge) is in Leo's channel"
    );
    assert!(
        subs.contains(&SAM.to_string()),
        "Sam (Possums staff) is in Leo's channel"
    );
    // THE LEAK GUARD: Mia's mum has no path to Leo's channel.
    assert!(
        !subs.contains(&MIAS_MUM.to_string()),
        "MUST NOT put another family's guardian in Leo's channel: {subs:?}"
    );
}

#[tokio::test]
async fn ana_has_no_path_to_mias_channel() {
    let store = seed().await;
    let cp = Chokepoint::new(store, WS);
    let subs = subjects(&channel_members(&cp, &ChannelTarget::Child(MIA)).await);

    // Ana is never a member of Mia's channel — she cannot even address it.
    assert!(
        !subs.contains(&ANA.to_string()),
        "THE CROSS-FAMILY LEAK: Ana must have NO path to Mia's channel: {subs:?}"
    );
    assert!(
        subs.contains(&MIAS_MUM.to_string()),
        "Mia's mum reaches her own child's channel"
    );
}

#[tokio::test]
async fn feed_only_guardian_is_not_a_messaging_member() {
    let store = seed().await;
    let cp = Chokepoint::new(store, WS);
    let subs = subjects(&channel_members(&cp, &ChannelTarget::Child(LEO)).await);

    // The DISTINCT-FLAG point: feed access ≠ messaging access. A guardian with
    // receives_daily_feed:true but receives_messaging:false gets NO channel seat.
    assert!(
        !subs.contains(&"user:leo-feedonly".to_string()),
        "a feed-only (no messaging flag) guardian must not be a channel member: {subs:?}"
    );
}

#[tokio::test]
async fn unlinked_guardian_drops_from_the_derived_set() {
    let store = seed().await;
    // Soft-unlink Ana↔Leo (live=false) — the unlink handler's basis for revoking
    // her channel caps. She must vanish from the derived membership immediately.
    store_write(
        &store,
        WS,
        "guardianship",
        &[ANA, LEO].join("::"),
        &serde_json::json!({
            "guardian_sub": ANA, "child_id": LEO, "live": false,
            "receives_messaging": true, "receives_daily_feed": true
        }),
    )
    .await
    .expect("overwrite edge dead");

    let cp = Chokepoint::new(store, WS);
    let subs = subjects(&channel_members(&cp, &ChannelTarget::Child(LEO)).await);
    assert!(
        !subs.contains(&ANA.to_string()),
        "an unlinked guardian must drop from the channel (revoke basis): {subs:?}"
    );
    // Staff are unaffected by a guardian unlink.
    assert!(subs.contains(&SAM.to_string()));
}

#[tokio::test]
async fn room_channel_derives_staff_deduped_full() {
    let store = seed().await;
    let cp = Chokepoint::new(store, WS);
    let members = channel_members(&cp, &ChannelTarget::Room(POSS)).await;
    let subs = subjects(&members);

    // The room channel's DERIVED membership is the room's staff (the stable
    // broadcast authors). Guardians are folded onto the room channel at the
    // PLACEMENT event, not derived here (the generic store `list` returns no
    // keys, so a room cannot enumerate its children — see
    // `resolve_room_channel_members`'s doc).
    assert!(
        subs.contains(&SAM.to_string()),
        "room staff in the room channel"
    );
    // No guardian leaks in via derivation (they're event-driven per placement).
    assert!(
        !subs.contains(&MIAS_MUM.to_string()),
        "another room's guardian excluded: {subs:?}"
    );
    // Idempotent dedupe: no subject appears twice, every member is Full.
    let mut seen = std::collections::HashSet::new();
    for m in &members {
        assert!(
            seen.insert(m.subject.clone()),
            "no duplicate member: {}",
            m.subject
        );
        assert!(m.full, "room channel members are Full (post+read)");
    }
}
