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
//! Verb body lands next session; the subject/emit contract it depends on is
//! complete in `log::records`.
