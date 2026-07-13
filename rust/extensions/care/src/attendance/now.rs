//! `care.attendance.now` — the DERIVED per-room occupancy `{children, staff,
//! ratio}` (attendance-scope §"Ratio read-out"). Current presence is a QUERY
//! over the append-only ledger, never a mutable counter: we scan every
//! `attendance_event`, fold it to per-room occupancy (`fold_now`, the
//! orchestrator-owned pure function), then REACH-FILTER the result.
//!
//! ## Scope (CLAUDE.md rule 7 — room isolation)
//!
//! - **Admin** (`reachable_rooms` → `["*"]`) sees every room.
//! - **Staff** see ONLY their assigned rooms — we drop any occupancy row whose
//!   `room_id` is not in their `reachable_rooms` set. Deny-by-empty: a staff
//!   member with no assignment gets `[]`, never an error, never a leak.
//! - **Guardian** holds no `now` cap at all — the host wall (cap grant) keeps
//!   them out before this verb runs, so there is no guardian branch here.
//!
//! An optional `room_id` input narrows the reply to a single room, but only if
//! the caller may already see it (the reach filter runs first — the filter
//! cannot be used as a room-existence oracle).

use lb_auth::Principal;

use crate::authz::{reachable_rooms, Chokepoint};
use super::records::AttendanceEvent;
use super::occupancy::fold_now;

/// Optional single-room filter. Absent ⇒ all rooms the caller reaches.
#[derive(Debug, Default, serde::Deserialize)]
struct NowInput {
    #[serde(default)]
    room_id: Option<String>,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    // Tolerate an empty / whitespace body as "no filter" (the verb takes no
    // required input).
    let parsed: NowInput = if input.trim().is_empty() {
        NowInput::default()
    } else {
        serde_json::from_str(input).map_err(|e| format!("parse input: {e}"))?
    };

    // Read the whole ledger and deserialize each row; a row we cannot decode is
    // skipped (never a phantom, never a hard failure of the read-out).
    let rows: Vec<serde_json::Value> = cp
        .records()
        .query_data("attendance_event")
        .await
        .map_err(|e| format!("store denied the attendance read: {e}"))?;
    let events: Vec<AttendanceEvent> = rows
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();

    // The pure fold: one occupancy row per room in the ledger.
    let mut occ = fold_now(&events);

    // Room-scope gate. Admin (`["*"]`) sees every room; staff see only their
    // assigned set. Empty reach ⇒ empty reply (deny-by-empty).
    let reach = reachable_rooms(cp, principal).await;
    let is_admin = reach.iter().any(|r| r == "*");
    if !is_admin {
        occ.retain(|o| reach.iter().any(|r| r == &o.room_id));
    }

    // Optional single-room narrowing — applied AFTER the reach filter so it can
    // only ever shrink an already-authorized set (no existence oracle).
    if let Some(want) = parsed.room_id.as_deref() {
        occ.retain(|o| o.room_id == want);
    }

    serde_json::to_string(&occ).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create as store_create, Store};
    use serde_json::json;
    use std::sync::Arc;

    fn admin(signing: &SigningKey) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: "acme".into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.attendance.now:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    fn staff(signing: &SigningKey, sub: &str) -> Principal {
        let claims = Claims {
            sub: sub.into(),
            ws: "acme".into(),
            role: Role::Member,
            caps: vec!["mcp:care.attendance.now:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    /// Seed one attendance_event via the real store write path.
    async fn seed_event(
        store: &Arc<Store>,
        id: &str,
        kind: &str,
        child: Option<&str>,
        staff_sub: Option<&str>,
        room: &str,
        at: &str,
    ) {
        let mut v = json!({
            "kind": kind,
            "room_id": room,
            "at": at,
            "performed_by": "user:teacher",
        });
        if let Some(c) = child {
            v["child_id"] = json!(c);
        }
        if let Some(s) = staff_sub {
            v["staff_sub"] = json!(s);
        }
        store_create(store, "acme", "attendance_event", id, &v)
            .await
            .unwrap();
    }

    async fn assign_staff(store: &Arc<Store>, sub: &str, room: &str) {
        store_create(
            store,
            "acme",
            "staff_assignment",
            &[sub, room].join("::"),
            &json!({"staff_sub": sub, "room_id": room}),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn admin_now_reports_children_staff_ratio() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_event(&store, "e1", "check_in", Some("leo"), None, "possums", "2026-07-14T08:00:00Z").await;
        seed_event(&store, "e2", "check_in", None, Some("user:t1"), "possums", "2026-07-14T07:50:00Z").await;

        let out = run(&cp, &admin(&key), "").await.expect("admin now");
        let occ: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(occ.len(), 1);
        assert_eq!(occ[0]["room_id"], "possums");
        assert_eq!(occ[0]["children"], 1);
        assert_eq!(occ[0]["staff"], 1);
        assert_eq!(occ[0]["ratio"], 1.0);
    }

    #[tokio::test]
    async fn a_checked_out_child_drops_to_zero() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_event(&store, "e1", "check_in", Some("leo"), None, "possums", "2026-07-14T08:00:00Z").await;
        seed_event(&store, "e2", "check_out", Some("leo"), None, "possums", "2026-07-14T17:00:00Z").await;

        let out = run(&cp, &admin(&key), "").await.expect("admin now");
        let occ: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(occ[0]["children"], 0, "later check_out ⇒ absent");
    }

    #[tokio::test]
    async fn staff_sees_only_their_assigned_room() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        // Two rooms with occupancy; staff assigned to only one.
        seed_event(&store, "e1", "check_in", Some("leo"), None, "possums", "2026-07-14T08:00:00Z").await;
        seed_event(&store, "e2", "check_in", Some("mia"), None, "wombats", "2026-07-14T08:00:00Z").await;
        assign_staff(&store, "user:t1", "possums").await;

        let out = run(&cp, &staff(&key, "user:t1"), "").await.expect("staff now");
        let occ: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(occ.len(), 1, "staff sees only their assigned room");
        assert_eq!(occ[0]["room_id"], "possums");
    }

    #[tokio::test]
    async fn room_id_filter_returns_just_that_room() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        seed_event(&store, "e1", "check_in", Some("leo"), None, "possums", "2026-07-14T08:00:00Z").await;
        seed_event(&store, "e2", "check_in", Some("mia"), None, "wombats", "2026-07-14T08:00:00Z").await;

        let out = run(&cp, &admin(&key), r#"{"room_id":"wombats"}"#).await.expect("filtered now");
        let occ: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(occ.len(), 1);
        assert_eq!(occ[0]["room_id"], "wombats");
    }
}
