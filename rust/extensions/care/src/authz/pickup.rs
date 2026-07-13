//! Pickup-roster resolution — the ONE place the `guardianship` edge's
//! `can_pickup` flag + `custody_notes` are read for the attendance pickup gate.
//! Lives in `authz/` because the fence (`check-authz-fence.sh`) forbids reading
//! `guardianship` anywhere else, and because "who may collect this child" is a
//! reach-adjacent authorization question (rule 7's neighbour).
//!
//! The attendance check-out verb calls [`pickup_roster`] to get the resolved
//! guardian half, merges in the child record's `authorized_pickups` names, and
//! hands the whole [`super::super::attendance::gate::PickupRoster`] to the pure
//! gate. The SAFETY DECISION lives in the gate; the guardianship READ lives
//! here, behind the fence. Neither half is duplicated in a verb.

use lb_store::list;
use serde::Deserialize;

use super::Chokepoint;

/// The guardian-side pickup facts for one child, resolved from the live
/// `guardianship` edges: the subjects + display names of guardians with
/// `can_pickup`, and whether ANY live edge carries custody notes (the hold).
#[derive(Debug, Clone, Default)]
pub struct GuardianPickupFacts {
    pub can_pickup_subs: Vec<String>,
    /// Guardian display names for `can_pickup` edges. Resolved from the
    /// `guardian` record keyed by the edge's `guardian_sub` (best-effort —
    /// a missing name just means that guardian only authorizes by sub).
    pub can_pickup_names: Vec<String>,
    pub custody_hold: bool,
}

/// The minimal edge projection the pickup resolver needs. `#[serde(default)]`
/// on the flags so an old edge without them reads as all-false (conservative).
#[derive(Debug, Deserialize)]
struct EdgeRow {
    guardian_sub: String,
    #[serde(default)]
    live: bool,
    #[serde(default)]
    can_pickup: bool,
    #[serde(default)]
    custody_notes: Option<String>,
}

/// Resolve the guardian-side pickup facts for `child_id` from the live
/// `guardianship` edges (era-1 store read — the same durable rows the
/// chokepoint resolves reach from). Only LIVE edges count; a `custody_notes`
/// on any live edge sets the hold (conservative — the note must be read).
///
/// Guardian display names are looked up from the `guardian` record so a
/// staff-selected NAME authorizes; a missing name is skipped (sub still works).
pub async fn pickup_roster(cp: &Chokepoint, child_id: &str) -> GuardianPickupFacts {
    // The fence-sanctioned read: `guardianship` rows for this child. `list`
    // filters by an indexed field; we filter by `child_id` and keep the live,
    // can_pickup ones. A store error yields an empty roster (fail-closed —
    // the gate then denies as unknown, never allows).
    let rows = match list(&cp.store, &cp.ws, "guardianship", "child_id", child_id).await {
        Ok(rs) => rs,
        Err(_) => return GuardianPickupFacts::default(),
    };

    let mut facts = GuardianPickupFacts::default();
    for row in rows {
        let edge: EdgeRow = match serde_json::from_value(row) {
            Ok(e) => e,
            // FAIL CLOSED (child-safety): an edge we cannot fully read might be
            // the one carrying the custody hold. Skipping it would drop the
            // hold and could release a held child to a `can_pickup` guardian a
            // court order restrains. So an undecodable edge forces the custody
            // hold — the gate then denies until an admin reads the record and
            // overrides. A malformed edge never silently opens the gate.
            Err(_) => {
                facts.custody_hold = true;
                continue;
            }
        };
        if !edge.live {
            continue;
        }
        if edge.custody_notes.as_deref().map(str::trim).map_or(false, |s| !s.is_empty()) {
            facts.custody_hold = true;
        }
        if edge.can_pickup {
            facts.can_pickup_subs.push(edge.guardian_sub.clone());
            if let Some(name) = guardian_name(cp, &edge.guardian_sub).await {
                facts.can_pickup_names.push(name);
            }
        }
    }
    facts
}

/// Best-effort guardian display name from the `guardian` record. `None` if the
/// record is absent or nameless (the sub still authorizes the collector).
async fn guardian_name(cp: &Chokepoint, guardian_sub: &str) -> Option<String> {
    let v = cp.records().read("guardian", guardian_sub).await.ok()??;
    v.get("name")
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
}
