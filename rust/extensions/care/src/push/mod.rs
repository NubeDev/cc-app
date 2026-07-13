//! care push policy — the type→notification decision + the `notify.send`
//! payload shape (daily-feed-scope §Push). Milestone 08.
//!
//! ## What lives here (and what does NOT)
//!
//! This module owns the PURE policy decision: given a `daily_log` entry's type
//! and the resolved feed-recipient set, decide whether to push and with what
//! catalog keys / deep-link. It does NOT hold the `notify.send` call itself —
//! that is one host-callback call issued by `log::add` (the write verb that
//! already holds the `SidecarClient`), keeping the I/O at the verb boundary and
//! the decision unit-testable here (no store, no client).
//!
//! ## Localization is lb's job, per recipient (the exit-gate requirement)
//!
//! We NEVER render push title/body strings here. `notify.send` takes
//! `title_key` / `body_key` / `args`, and lb resolves each recipient's locale
//! server-side from their device/prefs (push-target scope §i18n). So for ONE
//! incident, Sam (en) gets English and Ana (es) gets Spanish from a SINGLE
//! `notify.send` — the exit gate's both-languages assertion. cc-app supplies
//! ENUM KEYS (`push.title.<kind>` / `push.body.<kind>`), never words
//! (CLAUDE.md rule 8).
//!
//! ## Policy (daily-feed-scope §Push)
//!
//! - INCIDENT / MEDICATION → `PushPolicy::Always`: push to every
//!   `receives_daily_feed` holder, must-deliver via the outbox, quiet hours do
//!   NOT suppress (safety over convenience). The incident push is the
//!   never-drop case the outbox-retry test asserts.
//! - Every other type → `PushPolicy::FeedThenPrefs`: feed-only v1 (per-type
//!   guardian push prefs + quiet hours are a `profile`-tab slice — recorded as
//!   the resolved v1 posture, additive later). The entry still lands in the
//!   feed + emits on the bus; it just does not push.
//!
//! The recipient set itself (which guardians hold a LIVE `receives_daily_feed`
//! edge) is resolved by the authz chokepoint (`authz::feed_recipients`) — a
//! `false` or unlinked edge is already excluded before we get here, so a
//! non-feed guardian never appears in `Decision::recipients`.

use crate::log::{LogKind, PushPolicy};

/// The push decision for one entry — what `log::add` hands to `notify.send`.
/// `recipients` is empty ⇒ no push (feed-only, or nobody opted in); the verb
/// skips the call entirely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// The guardian subjects to notify (a subset — or all — of the entry's
    /// `receives_daily_feed` holders, already reach-filtered by the chokepoint).
    /// Empty ⇒ do not call `notify.send`.
    pub recipients: Vec<String>,
    /// The push title catalog key (`push.title.<kind>`) — lb renders it in each
    /// recipient's locale. Empty when there is no push.
    pub title_key: String,
    /// The push body catalog key (`push.body.<kind>`).
    pub body_key: String,
    /// The deep-link the push opens (the specific feed entry —
    /// daily-feed-scope §Push: "push deep-links to the entry").
    pub deep_link: String,
    /// Whether this is a must-deliver push that ignores quiet hours (incident /
    /// medication). Carried so the verb can set the outbox priority + the test
    /// can assert the always-push path.
    pub must_deliver: bool,
}

impl Decision {
    /// No push — feed-only. `log::add` checks `recipients.is_empty()` and skips
    /// the `notify.send` call.
    pub fn none() -> Self {
        Decision {
            recipients: Vec::new(),
            title_key: String::new(),
            body_key: String::new(),
            deep_link: String::new(),
            must_deliver: false,
        }
    }

    pub fn is_push(&self) -> bool {
        !self.recipients.is_empty()
    }
}

/// The deep-link an entry's push opens — the feed entry by id
/// (`care://feed/<entry_id>`; the shell routes it to the guardian Feed tab
/// scrolled to the entry). A pure key construction (join, not a formatted
/// literal — rule 8 lint).
pub fn entry_deep_link(entry_id: &str) -> String {
    ["care://feed", entry_id].join("/")
}

/// The catalog key for a kind's push title / body (`push.title.<kind>` /
/// `push.body.<kind>`). lb renders the words per recipient locale.
pub fn title_key(kind: LogKind) -> String {
    ["push.title", kind.key()].join(".")
}
pub fn body_key(kind: LogKind) -> String {
    ["push.body", kind.key()].join(".")
}

/// Decide the push for one entry. `kind` drives the policy; `recipients` is the
/// already-reach-filtered `receives_daily_feed` set (from
/// `authz::feed_recipients`); `entry_id` builds the deep-link.
///
/// - `Always` (incident / medication) → push to ALL recipients, must-deliver.
/// - `FeedThenPrefs` (everything else) → feed-only v1 → `Decision::none()`.
///
/// An empty `recipients` always yields `none()` regardless of policy — even an
/// incident has nobody to notify if no guardian holds a feed edge (a truly
/// private child). The entry still lands + emits; only the push is skipped.
pub fn decide(kind: LogKind, recipients: &[String], entry_id: &str) -> Decision {
    if recipients.is_empty() {
        return Decision::none();
    }
    match kind.push_policy() {
        PushPolicy::Always => Decision {
            recipients: recipients.to_vec(),
            title_key: title_key(kind),
            body_key: body_key(kind),
            deep_link: entry_deep_link(entry_id),
            must_deliver: true,
        },
        // v1: feed-only for the non-urgent types (per-guardian push prefs +
        // quiet hours are the deferred profile-tab slice). Recorded as the
        // resolved open-question posture, additive later.
        PushPolicy::FeedThenPrefs => Decision::none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incident_pushes_to_all_recipients_must_deliver() {
        let d = decide(
            LogKind::Incident,
            &["user:sam".into(), "user:ana".into()],
            "log:leo:incident:1",
        );
        assert!(d.is_push());
        assert!(d.must_deliver, "an incident ignores quiet hours");
        assert_eq!(d.recipients.len(), 2);
        assert_eq!(d.title_key, "push.title.incident");
        assert_eq!(d.body_key, "push.body.incident");
        assert_eq!(d.deep_link, "care://feed/log:leo:incident:1");
    }

    #[test]
    fn medication_also_always_pushes() {
        let d = decide(LogKind::Medication, &["user:ana".into()], "log:1");
        assert!(d.is_push() && d.must_deliver);
    }

    #[test]
    fn meal_is_feed_only_v1() {
        let d = decide(LogKind::Meal, &["user:sam".into()], "log:1");
        assert!(!d.is_push(), "a meal does not push v1 (feed-only)");
        assert_eq!(d, Decision::none());
    }

    #[test]
    fn no_recipients_never_pushes_even_for_an_incident() {
        // A truly private child (no feed-edge holder) → even an incident has
        // nobody to notify. The entry still lands; only the push is skipped.
        let d = decide(LogKind::Incident, &[], "log:1");
        assert!(!d.is_push());
    }

    #[test]
    fn photo_never_pushes_v1() {
        assert!(!decide(LogKind::Photo, &["user:sam".into()], "log:1").is_push());
    }
}
