//! care.messaging — provision lb channels + keep membership DERIVED from domain
//! records. Milestone 09 (docs/build/09-messaging.md).
//!
//! No transport is built: lb channels own history/SSE/post/read. care owns
//! WHO IS IN WHICH CHANNEL — membership is derived from guardianship edges (the
//! `receives_messaging` flag) + staff room assignments, never hand-managed
//! (`messaging-scope.md` §"Derived membership, one reconciler"). The whole
//! design is rule-10-shaped: care reaches channels only through the generic
//! granted `channel.*` + `grants.*` verbs; ids are conventions, membership is
//! derivation.
//!
//! - `channel_id` — the convention-named channel ids + the pub/sub cap builders
//!   (ORCHESTRATOR-OWNED; read-only = `sub` without `pub`, the posting policy).
//! - the reach-fenced membership derivation lives in `authz::channel_members`
//!   (it reads `guardianship` — a fence concern; see `authz/`).

pub mod announce;
pub mod channel_id;
pub mod reconcile;
pub mod reconcile_verb;

pub use channel_id::{
    center_channel, child_channel, pub_cap, room_channel, sub_cap, ChannelRole, CARE_CHANNEL_PREFIX,
};
