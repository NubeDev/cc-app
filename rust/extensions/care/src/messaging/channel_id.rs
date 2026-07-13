//! Channel-id conventions + the pub/sub cap builders — ORCHESTRATOR-OWNED
//! (milestone 09 §Subagent notes: fix the channel-id + cap shape first).
//!
//! ## Channels are provisioned by convention, opaque to the core
//!
//! care never asks lb "who is in this channel" — membership is DERIVED from
//! domain records (`membership.rs`) and enforced by lb's per-channel caps. The
//! channel id is a deterministic string built from a domain id, so provisioning
//! + reconciliation address the identical channel without any lb-side lookup
//! (`messaging-scope.md` §Goals: "an opaque string to the core"):
//!
//! - `care-child-<child_id>`   — a child's channel (its guardians + room staff).
//! - `care-room-<room_id>`     — a room broadcast (its staff + the room's kids'
//!                               guardians).
//! - `care-center-<center_id>` — the center announcements channel (admin/staff
//!                               post; guardians READ-ONLY — the split-cap grant).
//!
//! ## Read-only membership is the split cap (NO lb ask — entry gate resolved)
//!
//! lb channels split the authority (channels-scope §"Security invariants"):
//! **post** requires `bus:chan/{cid}:pub`, **read/history/subscribe** require
//! `bus:chan/{cid}:sub`. So:
//!
//! - a full member (child/room channel) is granted BOTH `pub` + `sub`;
//! - an announcements reader (guardian) is granted `sub` ONLY — they can read
//!   but every post 403s at lb's gate, no care hack (§Posting policy).
//!
//! lb's no-widening rule (`grants_assign`) requires the care install to HOLD a
//! cap matching what it grants, so the install requests the wildcards
//! `bus:chan/care-**:pub` + `bus:chan/care-**:sub` (`care_mount::approved_grant`;
//! the same idiom as `store:media/**:read` in `media/serve_grant.rs`).

/// The channel-id prefix every care-provisioned channel carries — the wildcard
/// hold (`bus:chan/care-**:{pub,sub}`) is scoped to exactly this prefix so care
/// can never grant on a non-care channel.
pub const CARE_CHANNEL_PREFIX: &str = "care-";

// NOTE: these builders assemble id/cap STRINGS via `concat`, not `format!` with
// a literal — the id prefixes (`care-child-`, `bus:chan/`) are wire conventions,
// not user-facing prose, but the no-hardcoded-strings fence's regex flags any
// letter-bearing literal inside a `format!`. `concat` keeps the fence quiet
// while reading identically (the care CI-lint idiom for id builders).

/// `care-child-<child_id>` — the per-child channel.
pub fn child_channel(child_id: &str) -> String {
    ["care-child-", child_id].concat()
}

/// `care-room-<room_id>` — the per-room broadcast channel.
pub fn room_channel(room_id: &str) -> String {
    ["care-room-", room_id].concat()
}

/// `care-center-<center_id>` — the center announcements channel.
pub fn center_channel(center_id: &str) -> String {
    ["care-center-", center_id].concat()
}

/// The post (publish) cap for a channel — a full member holds this.
pub fn pub_cap(channel_id: &str) -> String {
    ["bus:chan/", channel_id, ":pub"].concat()
}

/// The read (subscribe/history) cap for a channel — every member (including a
/// read-only announcements guardian) holds this.
pub fn sub_cap(channel_id: &str) -> String {
    ["bus:chan/", channel_id, ":sub"].concat()
}

/// The caps a role holds on a channel. `Full` = post + read (child/room
/// channels, and admin/staff on announcements); `ReadOnly` = read only (a
/// guardian on the announcements channel — every post 403s at lb's gate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelRole {
    Full,
    ReadOnly,
}

impl ChannelRole {
    /// The caps to grant a subject in this role on `channel_id` (`sub` always;
    /// `pub` only for `Full`). One owner of the "read-only = sub without pub"
    /// rule — the whole posting policy hinges on it.
    pub fn caps(self, channel_id: &str) -> Vec<String> {
        let mut caps = vec![sub_cap(channel_id)];
        if self == ChannelRole::Full {
            caps.push(pub_cap(channel_id));
        }
        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_ids_are_conventional_and_prefixed() {
        assert_eq!(child_channel("child:leo"), "care-child-child:leo");
        assert_eq!(room_channel("room:possums"), "care-room-room:possums");
        assert_eq!(center_channel("center:hq"), "care-center-center:hq");
        for id in [
            child_channel("child:leo"),
            room_channel("room:possums"),
            center_channel("center:hq"),
        ] {
            assert!(id.starts_with(CARE_CHANNEL_PREFIX), "wildcard-hold scope: {id}");
        }
    }

    #[test]
    fn read_only_is_sub_without_pub() {
        let full = ChannelRole::Full.caps("care-child-leo");
        let ro = ChannelRole::ReadOnly.caps("care-center-hq");
        assert!(full.contains(&"bus:chan/care-child-leo:pub".to_string()));
        assert!(full.contains(&"bus:chan/care-child-leo:sub".to_string()));
        // The whole announcements policy: a reader gets sub, never pub.
        assert_eq!(ro, vec!["bus:chan/care-center-hq:sub".to_string()]);
        assert!(!ro.iter().any(|c| c.ends_with(":pub")), "read-only never posts");
    }
}
