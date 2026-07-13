//! care.feed.watch — the live per-child SSE feed (daily-feed-scope §"Intent":
//! "one bus subject per child, filtered at emit"). Milestone 08.
//!
//! The subject shape + the emit contract are ORCHESTRATOR-OWNED and already
//! fixed in [`crate::log::feed_subject`] (`care.feed.<child_id>`): `log::add`
//! publishes one payload onto the child's subject via `bus.publish`, and this
//! verb (next session) opens the guardian's SSE subscription AFTER an
//! `authz::assert_reach` on the child — the reach check at SUBSCRIBE is the
//! "filtered at emit" guarantee (only reach-holders can subscribe to a child's
//! subject). Unlink mid-stream must terminate the OPEN stream, not just future
//! subscribes (exit gate) — the watch loop re-checks reach or the platform
//! drops the grant-scoped subscription.
//!
//! The emit half (`bus.publish` + `notify.send` over the host callback) lives
//! in `emit.rs` — the MOTION seam `log::add` calls. The subscribe half
//! (`care.feed.watch`) lives in `watch.rs`: a reach-checked authorization that
//! hands the guardian UI the gateway SSE stream (`GET /bus/{subject}/stream`),
//! since lb's `bus.watch` is an HTTP stream, not a `call_tool` verb.

pub mod emit;
pub mod watch;
pub mod watch_grant;

pub use emit::{publish_entry, send_push};
pub use watch_grant::{grant_feed_watch, revoke_feed_watch, FEED_WATCH_CAP_HELD};
