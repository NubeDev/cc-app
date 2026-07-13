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
use crate::i18n::t;

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
    let authorized_pickup_names: Vec<String> =
        child.authorized_pickups.iter().map(|p| p.name.clone()).collect();

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
                return Err(t(locale, reason.catalog_key(), &[("name", &parsed.child_id)]));
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
mod tests {
    use super::*;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create as store_create, Store};
    use serde_json::json;
    use std::sync::Arc;

    fn principal(signing: &SigningKey, sub: &str, ws: &str, role: Role) -> Principal {
        let claims = Claims {
            sub: sub.into(),
            ws: ws.into(),
            role,
            caps: vec!["mcp:care.attendance.check_out:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }
    fn admin(k: &SigningKey, ws: &str) -> Principal {
        principal(k, "user:admin", ws, Role::WorkspaceAdmin)
    }
    fn member(k: &SigningKey, sub: &str, ws: &str) -> Principal {
        principal(k, sub, ws, Role::Member)
    }

    async fn seed_child(store: &Arc<Store>, ws: &str, id: &str, pickups: serde_json::Value) {
        let child = json!({
            "name": "Leo",
            "dob": "2021-03-15",
            "authorized_pickups": pickups,
        });
        store_create(store, ws, "child", id, &child).await.unwrap();
    }

    async fn seed_edge(
        store: &Arc<Store>,
        ws: &str,
        edge_id: &str,
        guardian_sub: &str,
        child_id: &str,
        can_pickup: bool,
        custody_notes: Option<&str>,
    ) {
        let mut edge = json!({
            "guardian_sub": guardian_sub,
            "child_id": child_id,
            "live": true,
            "can_pickup": can_pickup,
        });
        if let Some(n) = custody_notes {
            edge["custody_notes"] = json!(n);
        }
        store_create(store, ws, "guardianship", edge_id, &edge).await.unwrap();
        // Guardian record so the name resolves for name-based authorization.
        let g = json!({ "name": "Sam Parent" });
        store_create(store, ws, "guardian", guardian_sub, &g).await.unwrap();
    }

    #[tokio::test]
    async fn can_pickup_guardian_by_sub_is_allowed() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_child(&store, "acme", "leo", json!([])).await;
        seed_edge(&store, "acme", "e1", "user:sam", "leo", true, None).await;

        let p = admin(&key, "acme");
        let out = run(
            &cp,
            &p,
            r#"{"event_id":"ev1","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Sam Parent","collector_sub":"user:sam"}"#,
        )
        .await
        .expect("check-out allowed");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["event_id"], "ev1");
        // Event was appended.
        let ev = cp.records().read("attendance_event", "ev1").await.unwrap().unwrap();
        assert_eq!(ev["kind"], "check_out");
        assert_eq!(ev["pickup_override"], false);
    }

    #[tokio::test]
    async fn authorized_pickup_by_name_is_allowed() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_child(&store, "acme", "leo", json!([{"name": "Grandma Jo"}])).await;

        let p = admin(&key, "acme");
        let out = run(
            &cp,
            &p,
            r#"{"event_id":"ev2","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"grandma jo"}"#,
        )
        .await
        .expect("name-authorized check-out allowed");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["event_id"], "ev2");
        assert!(cp.records().read("attendance_event", "ev2").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn a_stranger_is_hard_denied_with_localized_reason() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_child(&store, "acme", "leo", json!([{"name": "Grandma Jo"}])).await;
        seed_edge(&store, "acme", "e1", "user:sam", "leo", true, None).await;

        let p = admin(&key, "acme");
        // Stranger in English.
        let err_en = run(
            &cp,
            &p,
            r#"{"event_id":"evX","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Mallory","collector_sub":"user:mallory","locale":"en"}"#,
        )
        .await
        .expect_err("stranger denied");
        // No event appended on a deny.
        assert!(cp.records().read("attendance_event", "evX").await.unwrap().is_none());

        // Stranger in Spanish — the message MUST differ (a Spanish teacher reads why).
        let err_es = run(
            &cp,
            &p,
            r#"{"event_id":"evY","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Mallory","collector_sub":"user:mallory","locale":"es"}"#,
        )
        .await
        .expect_err("stranger denied es");

        assert_ne!(err_en, err_es, "deny reason must localize");
        assert!(err_es.contains("autorizada"), "spanish not_authorized text: {err_es}");
        assert!(cp.records().read("attendance_event", "evY").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn custody_hold_denies_even_can_pickup_guardian() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_child(&store, "acme", "leo", json!([])).await;
        seed_edge(&store, "acme", "e1", "user:sam", "leo", true, Some("court order")).await;

        // A can_pickup guardian — but a custody hold denies unless admin override.
        let member_p = member(&key, "user:sam", "acme");
        let denied = run(
            &cp,
            &member_p,
            r#"{"event_id":"evH","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Sam Parent","collector_sub":"user:sam"}"#,
        )
        .await;
        assert!(denied.is_err(), "custody hold denies without override");
        assert!(cp.records().read("attendance_event", "evH").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn admin_override_on_denied_collector_is_allowed_and_audited() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_child(&store, "acme", "leo", json!([])).await;
        seed_edge(&store, "acme", "e1", "user:sam", "leo", true, Some("court order")).await;

        let p = admin(&key, "acme");
        let out = run(
            &cp,
            &p,
            r#"{"event_id":"evO","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Sam Parent","collector_sub":"user:sam","pickup_override":true}"#,
        )
        .await
        .expect("admin override allowed");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["event_id"], "evO");
        let ev = cp.records().read("attendance_event", "evO").await.unwrap().unwrap();
        assert_eq!(ev["pickup_override"], true);
        assert_eq!(ev["override_reason"], "custody_hold");
    }

    #[tokio::test]
    async fn non_admin_override_is_still_denied() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_child(&store, "acme", "leo", json!([])).await;
        seed_edge(&store, "acme", "e1", "user:sam", "leo", true, Some("court order")).await;

        // A Member sets pickup_override:true — the override is admin-capped, so
        // they are STILL denied and no event is written.
        let member_p = member(&key, "user:sam", "acme");
        let denied = run(
            &cp,
            &member_p,
            r#"{"event_id":"evN","child_id":"leo","room_id":"possums","at":"2026-07-14T17:20:00Z","collector_name":"Sam Parent","collector_sub":"user:sam","pickup_override":true}"#,
        )
        .await;
        assert!(denied.is_err(), "non-admin override must be denied");
        assert!(cp.records().read("attendance_event", "evN").await.unwrap().is_none());
    }
}
