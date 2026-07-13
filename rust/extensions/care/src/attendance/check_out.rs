//! `care.attendance.check_out` — APPEND a child check-out event, GATED by the
//! pickup safety control. Cap: `mcp:care.attendance.check_out:call`.
//!
//! ## The child-safety gate (attendance-scope §"Pickup authorization")
//!
//! Check-out is where a child LEAVES the building — the single most
//! safety-critical tap in the product. The verb takes the collecting person and
//! runs them through the pure pickup gate (`gate::decide`): a collector is
//! allowed IFF they are a `can_pickup` guardian for this child OR a named
//! authorized-pickup entry on the child record, AND the child is not under a
//! custody hold. Anything else is a HARD DENY, returned to the staff device as
//! the LOCALIZED reason (a Spanish teacher must read WHY the gate refused).
//!
//! The gate is UNBYPASSABLE except by the audited admin override: an admin
//! (WorkspaceAdmin/SuperAdmin) may set `pickup_override:true` to release past a
//! failed gate, and the event then records `pickup_override:true` +
//! `override_reason` (the reason it bypassed) for the audit trail. A NON-admin
//! passing `pickup_override:true` is STILL denied — the override is admin-capped
//! (custody disputes land exactly here).
//!
//! ## Where each half lives (rule 7 + the authz fence)
//!
//! The `can_pickup` flag + `custody_notes` live on the `guardianship` edge,
//! which only `authz/` may read (`check-authz-fence.sh`). So the verb resolves
//! the guardian half through `authz::pickup_roster`, merges the child record's
//! `authorized_pickups` names, and hands the whole roster to the pure gate. The
//! safety LOGIC is in the gate; the guardianship READ is behind the fence.

use lb_auth::Principal;

use crate::attendance::{pickup_decide, AttendanceEvent, Collector, EventKind, PickupRoster};
use crate::authz::{pickup_roster, Chokepoint};
use crate::center::Locale;
use crate::child::Child;
use crate::feed::publish_entry;
use crate::i18n::t;
use crate::log::feed_subject;

/// The check-out request. `child_id` is required — check_out is child-only (a
/// child leaves); staff-presence check-out is out of scope here.
#[derive(Debug, serde::Deserialize)]
pub struct CheckOutInput {
    /// Unique event id — the append is first-write on this id (a ledger never
    /// overwrites; a duplicate id is a conflict).
    pub event_id: String,
    /// The child leaving (a `child` id).
    pub child_id: String,
    /// The room the child is checked out from (the ratio read-out is per-room).
    pub room_id: String,
    /// Event time, ISO-8601 — the staff/kiosk device stamps real wall time.
    pub at: String,
    /// WHO is collecting — the display name the staff selected (always present).
    pub collector_name: String,
    /// The collector's auth subject, if they authenticated as a guardian.
    #[serde(default)]
    pub collector_sub: Option<String>,
    /// Admin-capped, audited override past a failed gate (ignored for non-admins).
    #[serde(default)]
    pub pickup_override: bool,
    /// Optional free-text note (never translated).
    #[serde(default)]
    pub note: Option<String>,
    /// Requested locale for the deny/confirm message (`"en"` | `"es"`).
    #[serde(default)]
    pub locale: Option<String>,
}

/// The check-out reply — the appended event id + a localized confirmation
/// (`attendance.checked_out`, or `attendance.override_recorded` when overridden).
#[derive(Debug, serde::Serialize)]
pub struct CheckOutReply {
    pub event_id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: CheckOutInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.attendance.check_out input: {e}"))?;

    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // Validate the required fields non-empty (+ timestamp shape). A garbage `at`
    // must never fragment the ledger's time ordering.
    if parsed.event_id.trim().is_empty() {
        return Err("event_id is required".to_string());
    }
    if parsed.child_id.trim().is_empty() {
        return Err("child_id is required".to_string());
    }
    if parsed.room_id.trim().is_empty() {
        return Err("room_id is required".to_string());
    }
    if parsed.collector_name.trim().is_empty() {
        return Err("collector_name is required".to_string());
    }
    crate::attendance::validate_timestamp(&parsed.at).map_err(|e| format!("{e}"))?;

    // BUILD THE ROSTER. The guardian half comes through the authz chokepoint
    // (behind the fence); the authorized-pickup names come from the child record.
    let facts = pickup_roster(cp, &parsed.child_id).await;

    let child_value = cp
        .records()
        .read("child", &parsed.child_id)
        .await
        .map_err(|_| "store denied the child read".to_string())?
        .ok_or_else(|| "child not found".to_string())?;
    let child: Child =
        serde_json::from_value(child_value).map_err(|e| format!("deserialize child: {e}"))?;
    let authorized_pickup_names: Vec<String> = child
        .authorized_pickups
        .iter()
        .map(|p| p.name.clone())
        .collect();

    let roster = PickupRoster {
        can_pickup_guardians: facts.can_pickup_subs,
        can_pickup_names: facts.can_pickup_names,
        authorized_pickup_names,
        custody_hold: facts.custody_hold,
    };

    // RUN THE GATE.
    let collector = Collector {
        sub: parsed.collector_sub.clone(),
        name: parsed.collector_name.clone(),
    };

    let is_admin = principal.role() == lb_auth::Role::WorkspaceAdmin
        || principal.role() == lb_auth::Role::SuperAdmin;

    // The gate decides ALLOW / DENY-with-reason. On a DENY, only an ADMIN
    // `pickup_override` proceeds (audited); everyone else — including a non-admin
    // who set `pickup_override:true` — is hard-denied with the LOCALIZED reason.
    let (pickup_override, override_reason) = match pickup_decide(&collector, &roster) {
        Ok(()) => (false, None),
        Err(reason) => {
            if parsed.pickup_override && is_admin {
                // Audited admin release: allow, but record WHY the gate refused.
                (true, Some(reason))
            } else {
                // HARD DENY — the staff device reads the localized reason.
                return Err(t(
                    locale,
                    reason.catalog_key(),
                    &[("name", &parsed.child_id)],
                ));
            }
        }
    };

    // APPEND the check-out event (first-write on the event id — the ledger never
    // overwrites; a wrong tap is fixed by a compensating `correct`, not an edit).
    let event = AttendanceEvent {
        kind: EventKind::CheckOut,
        child_id: Some(parsed.child_id.clone()),
        staff_sub: None,
        room_id: parsed.room_id.clone(),
        at: parsed.at.clone(),
        performed_by: principal.sub().to_string(),
        person: Some(parsed.collector_name.clone()),
        correction_of: None,
        pickup_override,
        override_reason,
        note: parsed.note.clone(),
    };
    let value = serde_json::to_value(&event).map_err(|e| format!("serialize event: {e}"))?;
    cp.records()
        .create("attendance_event", &parsed.event_id, &value)
        .await
        .map_err(|e| format!("append check-out event: {e}"))?;

    // Attendance → feed emit (the m06 deferral, wired at m08): a child departure
    // appears in the guardian's live feed onto the same per-child subject the
    // daily-feed entries use. Best-effort (the ledger row is the source of truth).
    publish_entry(cp.host_client(), &feed_subject(&parsed.child_id), &value).await;

    let msg_key = if pickup_override {
        "attendance.override_recorded"
    } else {
        "attendance.checked_out"
    };
    let reply = CheckOutReply {
        event_id: parsed.event_id.clone(),
        message: t(locale, msg_key, &[("name", &child.name)]),
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
#[path = "check_out_tests.rs"]
mod tests;
