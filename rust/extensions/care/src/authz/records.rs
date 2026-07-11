//! The record shapes the chokepoint resolves from the store (era 1).
//!
//! These are the **transient** types the chokepoint's read path returns;
//! the durable record shapes on the store live in the per-verb folders
//! (the `child/`, `guardian/`, `guardianship/` folders of the milestone 03
//! verbs). The chokepoint only needs the minimum it asks the wall — `Edge`
//! for the guardian↔child edge, `StaffAssignment` for the staff↔room
//! mapping. We DO NOT model the full record here; that's the verb's job.

use serde::{Deserialize, Serialize};

/// A `guardianship` edge — the many-to-many `guardian ↔ child` carrying
/// the relationship and per-edge flags. Milestone 03's verbs own the full
/// durable shape; the chokepoint only needs `guardian_sub` + `child_id` +
/// the liveness flag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Edge {
    pub guardian_sub: String,
    pub child_id: String,
    /// `true` while the edge is live (i.e. `link` was called and
    /// `unlink` has not). The chokepoint denies on a missing or
    /// `archived == true` edge — the unlink-on-same-tx semantics from
    /// the care-authz scope.
    pub live: bool,
}

/// A guardian record projection — only the `sub` (the workspace member
/// this guardian links to once they accept an invite). The full durable
/// shape is in milestone 03's `guardian/` folder.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Guardian {
    pub sub: String,
}

/// A staff room assignment — the projection the chokepoint reads to
/// answer `reachable_rooms(staff)`. Full shape lives in the staff
/// vocabulary milestone 03 ships.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StaffAssignment {
    pub staff_sub: String,
    pub room_id: String,
}
