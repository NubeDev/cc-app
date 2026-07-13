//! `care.attendance.list` — the attendance LEDGER, ROLE-FILTERED (CLAUDE.md
//! rule 7). Cap: `mcp:care.attendance.list:call`.
//!
//! ## Three role scopes over ONE ledger scan
//!
//! The ledger carries every family's events in one table; who may read which
//! rows is the whole game (attendance-scope §"How it fits": cross-family matrix
//! rows for `list`). We scan the ledger once (`query_data`) and then keep only
//! the rows the caller is entitled to, per role:
//!
//! - **Admin** (`reachable_children` yields the `["*"]` wildcard) → every event.
//! - **Staff** → events whose `room_id` is one of the caller's assigned rooms
//!   (`reachable_rooms`). A staff-presence event (no `child_id`) is still
//!   room-scoped, so it rides the same filter.
//! - **Guardian** → events whose `child_id` is in the reached-child set
//!   (`reachable_children`). A guardian reaches NOTHING ⇒ EMPTY (never an
//!   error — the list-verb deny semantic). A staff-presence event (no
//!   `child_id`) is NEVER shown to a guardian — they have no room scope.
//!
//! ## Filters only NARROW an already-authorized set
//!
//! The optional `room_id` / `child_id` / `since` / `until` filters are applied
//! AFTER the role scope, so a filter can never widen what the caller may see —
//! it can only trim the authorized rows further. `since`/`until` compare the
//! ISO-8601 `at` string lexically (inclusive on both ends): the timestamps are
//! fixed-width `YYYY-MM-DDTHH:MM:SS...` (records.rs `validate_timestamp`), so a
//! byte-wise string compare is a chronological compare.
//!
//! Rows come back sorted by `at` ascending — the ledger reads as a timeline.

use lb_auth::Principal;

use crate::attendance::AttendanceEvent;
use crate::authz::{reachable_children, reachable_rooms, Chokepoint};

#[derive(Debug, serde::Deserialize)]
pub struct ListInput {
    /// Keep only events in this room (exact match). Optional.
    #[serde(default)]
    pub room_id: Option<String>,
    /// Keep only events for this child (exact match). Optional.
    #[serde(default)]
    pub child_id: Option<String>,
    /// Keep only events with `at >= since` (inclusive, ISO string compare).
    #[serde(default)]
    pub since: Option<String>,
    /// Keep only events with `at <= until` (inclusive, ISO string compare).
    #[serde(default)]
    pub until: Option<String>,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    // Empty input ⇒ no filters (all four optional). A malformed body is a
    // caller bug, surfaced as an error.
    let filters: ListInput = if input.trim().is_empty() {
        ListInput {
            room_id: None,
            child_id: None,
            since: None,
            until: None,
        }
    } else {
        serde_json::from_str(input).map_err(|e| format!("invalid care.attendance.list input: {e}"))?
    };

    // Resolve the caller's reach ONCE. Admin ⇒ `["*"]` on both; guardian gets a
    // (possibly empty) child set; staff gets a room set.
    let reach_children = reachable_children(cp, principal).await;
    let is_admin = reach_children.iter().any(|r| r == "*");

    // One ledger scan — the whole table, then filtered down by role.
    let all = all_events(cp).await?;

    let scoped: Vec<AttendanceEvent> = if is_admin {
        all
    } else {
        let reach_rooms = reachable_rooms(cp, principal).await;
        // A non-admin is EITHER staff (has rooms) OR a guardian (has children);
        // an event is authorized if it lands in the caller's room scope OR its
        // child is in the caller's reached set. Deny-by-empty falls out
        // naturally: a guardian who reaches nothing keeps zero rows.
        all.into_iter()
            .filter(|e| is_authorized(e, &reach_rooms, &reach_children))
            .collect()
    };

    // Filters only NARROW the authorized set (never widen it).
    let mut out: Vec<AttendanceEvent> = scoped
        .into_iter()
        .filter(|e| passes_filters(e, &filters))
        .collect();

    // Timeline order: ISO-8601 `at` sorts chronologically as a byte string.
    out.sort_by(|a, b| a.at.cmp(&b.at));

    serde_json::to_string(&out).map_err(|e| format!("serialize reply: {e}"))
}

/// Is `event` authorized for a NON-admin caller? Staff reach is by room; a
/// guardian's reach is by child. A staff-presence event (no `child_id`) can
/// only match the room test — a guardian (empty room scope) never sees it.
fn is_authorized(event: &AttendanceEvent, reach_rooms: &[String], reach_children: &[String]) -> bool {
    let room_ok = reach_rooms.iter().any(|r| r == &event.room_id);
    let child_ok = match &event.child_id {
        Some(cid) => reach_children.iter().any(|c| c == cid),
        None => false,
    };
    room_ok || child_ok
}

/// Apply the optional input filters (exact match on room/child; inclusive
/// lexical bounds on the ISO `at` string). A `None` filter is a no-op.
fn passes_filters(event: &AttendanceEvent, f: &ListInput) -> bool {
    if let Some(room) = &f.room_id {
        if &event.room_id != room {
            return false;
        }
    }
    if let Some(child) = &f.child_id {
        if event.child_id.as_deref() != Some(child.as_str()) {
            return false;
        }
    }
    if let Some(since) = &f.since {
        if event.at.as_str() < since.as_str() {
            return false;
        }
    }
    if let Some(until) = &f.until {
        if event.at.as_str() > until.as_str() {
            return false;
        }
    }
    true
}

/// Read the whole `attendance_event` ledger, deserialized. Malformed rows are
/// skipped (a garbage row can't leak, and can't fail an authorized read).
async fn all_events(cp: &Chokepoint) -> Result<Vec<AttendanceEvent>, String> {
    let rows: Vec<serde_json::Value> = cp
        .records()
        .query_data("attendance_event")
        .await
        .map_err(|e| format!("store denied the attendance list: {e}"))?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        if let Ok(e) = serde_json::from_value::<AttendanceEvent>(row) {
            out.push(e);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attendance::{AttendanceEvent, EventKind};
    use crate::guardianship::link as guardianship_link;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.guardianship.link:call".into(),
                "mcp:care.attendance.list:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    fn member(signing: &SigningKey, sub: &str, ws: &str) -> Principal {
        let claims = Claims {
            sub: sub.into(),
            ws: ws.into(),
            role: Role::Member,
            caps: vec!["mcp:care.attendance.list:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    /// Seed one child attendance event straight through the real write path.
    async fn seed_event(
        store: &Arc<Store>,
        ws: &str,
        event_id: &str,
        child_id: &str,
        room_id: &str,
        at: &str,
    ) {
        let e = AttendanceEvent {
            kind: EventKind::CheckIn,
            child_id: Some(child_id.into()),
            staff_sub: None,
            room_id: room_id.into(),
            at: at.into(),
            performed_by: "user:teacher".into(),
            person: None,
            correction_of: None,
            pickup_override: false,
            override_reason: None,
            note: None,
        };
        let v = serde_json::to_value(&e).unwrap();
        create(store, ws, "attendance_event", event_id, &v)
            .await
            .expect("seed event");
    }

    /// Two families: Sam→(Leo, Mia); Ana→Leo. Leo in Possums, Mia in Wombats.
    /// Returns the store + signing key. Attendance rows: 2 for Leo, 1 for Mia.
    async fn seed_two_families() -> (Arc<Store>, SigningKey) {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");

        // Sam reaches Leo + Mia; Ana reaches ONLY Leo.
        for input in [
            r#"{"guardian_sub":"user:sam","child_id":"child:leo","relationship":"father"}"#,
            r#"{"guardian_sub":"user:sam","child_id":"child:mia","relationship":"father"}"#,
            r#"{"guardian_sub":"user:ana","child_id":"child:leo","relationship":"mother"}"#,
        ] {
            guardianship_link::run(&cp, &a, input).await.expect("link");
        }

        seed_event(&store, "acme", "ev:leo:1", "child:leo", "room:possums", "2026-07-13T08:02:00Z").await;
        seed_event(&store, "acme", "ev:leo:2", "child:leo", "room:possums", "2026-07-13T15:30:00Z").await;
        seed_event(&store, "acme", "ev:mia:1", "child:mia", "room:wombats", "2026-07-13T08:10:00Z").await;

        (store, key)
    }

    #[tokio::test]
    async fn admin_lists_all_events() {
        let (store, key) = seed_two_families().await;
        let cp = Chokepoint::new(store, "acme");
        let a = admin(&key, "acme");

        let out = run(&cp, &a, "").await.unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(v.len(), 3, "admin sees every family's events");
        // Sorted ascending by `at`.
        assert_eq!(v[0]["at"], "2026-07-13T08:02:00Z");
        assert_eq!(v[1]["at"], "2026-07-13T08:10:00Z");
        assert_eq!(v[2]["at"], "2026-07-13T15:30:00Z");
    }

    /// RULE 7 CROSS-FAMILY ROW: Ana reaches Leo only — she must see Leo's two
    /// events and NEVER Mia's. A leak here is the worst bug this product has.
    #[tokio::test]
    async fn guardian_sees_only_reached_childrens_events() {
        let (store, key) = seed_two_families().await;
        let cp = Chokepoint::new(store, "acme");
        let ana = member(&key, "user:ana", "acme");

        let out = run(&cp, &ana, "").await.unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(v.len(), 2, "Ana sees Leo's two events only");
        for row in &v {
            assert_eq!(row["child_id"], "child:leo");
            assert_ne!(row["child_id"], "child:mia", "MUST NOT leak Mia across families");
        }
    }

    #[tokio::test]
    async fn guardian_with_no_reach_gets_empty_not_error() {
        let (store, key) = seed_two_families().await;
        let cp = Chokepoint::new(store, "acme");
        // A guardian with NO edge to any child reaches nothing.
        let stranger = member(&key, "user:stranger", "acme");

        let out = run(&cp, &stranger, "").await.expect("empty, not error");
        assert_eq!(out, "[]", "deny-by-empty, never an error");
    }

    #[tokio::test]
    async fn child_id_filter_narrows_within_authorized_set() {
        let (store, key) = seed_two_families().await;
        let cp = Chokepoint::new(store, "acme");
        let sam = member(&key, "user:sam", "acme");

        // Sam reaches both Leo and Mia (3 events); filter to Leo → 2 rows.
        let unfiltered = run(&cp, &sam, "").await.unwrap();
        let uv: Vec<serde_json::Value> = serde_json::from_str(&unfiltered).unwrap();
        assert_eq!(uv.len(), 3, "Sam reaches both children");

        let out = run(&cp, &sam, r#"{"child_id":"child:leo"}"#).await.unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(v.len(), 2, "filter narrows to Leo");
        assert!(v.iter().all(|r| r["child_id"] == "child:leo"));
    }
}
