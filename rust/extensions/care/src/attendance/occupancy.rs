//! The DERIVED "who's here now" fold — ORCHESTRATOR-OWNED (attendance-scope
//! §"Ratio read-out" + §"Intent": current occupancy is a QUERY over the
//! ledger, never a mutable counter). A pure function so it is heavily tested
//! and the `now` verb stays thin.
//!
//! ## The rule (last-event-wins per subject, correction-aware)
//!
//! For each child and each staff member, the CURRENT presence is decided by
//! their LAST event in time order: a `check_in` with no later `check_out`
//! means present; a `check_out` (or nothing) means absent. A correction event
//! (`correction_of`) is just another append that participates in the ordering
//! — a wrong check-in corrected by a compensating check-out nets to absent, so
//! `now` is right after any in/out/correct sequence (the scope's test). We
//! fold per (subject, room) and count the present.
//!
//! Ratio is `children / staff` per room (display + threshold only — no
//! enforcement here). A room with children but zero staff yields a `None`
//! ratio (division guard) which the UI renders as an alert.

use std::collections::HashMap;

use serde::Serialize;

use super::records::{AttendanceEvent, EventKind};

/// Per-room occupancy for the `now` read-out.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RoomOccupancy {
    pub room_id: String,
    /// Present children (last event is a check_in).
    pub children: usize,
    /// Present staff (last event is a check_in).
    pub staff: usize,
    /// children / staff, or `None` when staff is zero (a division guard the UI
    /// renders as "no staff present" — an amber/red alert).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratio: Option<f64>,
}

/// Fold a ledger slice into per-room occupancy. The events may be in any
/// order; we sort by `at` (string ISO-8601 sorts chronologically) so the
/// last-event-wins rule is stable. Returns one [`RoomOccupancy`] per room that
/// appears in the ledger, sorted by room id for a stable read.
pub fn fold_now(events: &[AttendanceEvent]) -> Vec<RoomOccupancy> {
    // (subject-key, room) → last (at, kind). Subject-key distinguishes a child
    // from a staff member so a child and a staff with colliding ids can't
    // clobber each other.
    let mut last: HashMap<(String, String), (&str, EventKind)> = HashMap::new();

    let mut ordered: Vec<&AttendanceEvent> = events.iter().collect();
    ordered.sort_by(|a, b| a.at.cmp(&b.at));

    for e in ordered {
        let subject = match (&e.child_id, &e.staff_sub) {
            (Some(c), _) => ["child:", c.as_str()].concat(),
            (_, Some(s)) => ["staff:", s.as_str()].concat(),
            // An event with neither subject is malformed — skip it (never
            // count a phantom).
            (None, None) => continue,
        };
        last.insert((subject, e.room_id.clone()), (&e.at, e.kind));
    }

    // Count present per room, split by subject kind.
    let mut children: HashMap<String, usize> = HashMap::new();
    let mut staff: HashMap<String, usize> = HashMap::new();
    for ((subject, room), (_, kind)) in &last {
        if *kind != EventKind::CheckIn {
            continue; // last event was a check_out ⇒ absent
        }
        if subject.starts_with("child:") {
            *children.entry(room.clone()).or_default() += 1;
        } else {
            *staff.entry(room.clone()).or_default() += 1;
        }
    }

    // Every room that appears ANYWHERE in the ledger shows in `now` — a room
    // whose occupants have all checked out reads as `0/0`, not a vanished row
    // (the dashboard shows the room emptied, not that it never existed).
    let mut rooms: Vec<String> = events.iter().map(|e| e.room_id.clone()).collect();
    rooms.sort();
    rooms.dedup();

    rooms
        .into_iter()
        .map(|room| {
            let c = *children.get(&room).unwrap_or(&0);
            let s = *staff.get(&room).unwrap_or(&0);
            let ratio = if s == 0 {
                None
            } else {
                Some(c as f64 / s as f64)
            };
            RoomOccupancy {
                room_id: room,
                children: c,
                staff: s,
                ratio,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(
        kind: EventKind,
        child: Option<&str>,
        staff: Option<&str>,
        room: &str,
        at: &str,
    ) -> AttendanceEvent {
        AttendanceEvent {
            kind,
            child_id: child.map(String::from),
            staff_sub: staff.map(String::from),
            room_id: room.into(),
            at: at.into(),
            performed_by: "user:teacher".into(),
            person: None,
            correction_of: None,
            pickup_override: false,
            override_reason: None,
            note: None,
        }
    }

    #[test]
    fn a_check_in_without_a_later_out_is_present() {
        let events = vec![
            ev(
                EventKind::CheckIn,
                Some("leo"),
                None,
                "possums",
                "2026-07-14T08:00:00Z",
            ),
            ev(
                EventKind::CheckIn,
                None,
                Some("user:t1"),
                "possums",
                "2026-07-14T07:50:00Z",
            ),
        ];
        let now = fold_now(&events);
        assert_eq!(now.len(), 1);
        assert_eq!(now[0].children, 1);
        assert_eq!(now[0].staff, 1);
        assert_eq!(now[0].ratio, Some(1.0));
    }

    #[test]
    fn a_later_check_out_makes_a_child_absent() {
        let events = vec![
            ev(
                EventKind::CheckIn,
                Some("leo"),
                None,
                "possums",
                "2026-07-14T08:00:00Z",
            ),
            ev(
                EventKind::CheckOut,
                Some("leo"),
                None,
                "possums",
                "2026-07-14T17:00:00Z",
            ),
        ];
        assert_eq!(fold_now(&events)[0].children, 0);
    }

    #[test]
    fn a_correcting_checkout_nets_to_absent_regardless_of_append_order() {
        // A wrong check-in at 08:00, corrected by a compensating check-out at
        // 08:01 — even if the correction is appended (listed) first, time
        // ordering makes the child absent.
        let events = vec![
            {
                let mut c = ev(
                    EventKind::CheckOut,
                    Some("leo"),
                    None,
                    "possums",
                    "2026-07-14T08:01:00Z",
                );
                c.correction_of = Some("evt-1".into());
                c
            },
            ev(
                EventKind::CheckIn,
                Some("leo"),
                None,
                "possums",
                "2026-07-14T08:00:00Z",
            ),
        ];
        assert_eq!(
            fold_now(&events)[0].children,
            0,
            "correction nets to absent"
        );
    }

    #[test]
    fn children_with_zero_staff_yield_no_ratio() {
        let events = vec![ev(
            EventKind::CheckIn,
            Some("leo"),
            None,
            "possums",
            "2026-07-14T08:00:00Z",
        )];
        assert_eq!(
            fold_now(&events)[0].ratio,
            None,
            "no staff → alert, not a divide-by-zero"
        );
    }

    #[test]
    fn re_check_in_after_checkout_is_present_again() {
        let events = vec![
            ev(
                EventKind::CheckIn,
                Some("leo"),
                None,
                "possums",
                "2026-07-14T08:00:00Z",
            ),
            ev(
                EventKind::CheckOut,
                Some("leo"),
                None,
                "possums",
                "2026-07-14T12:00:00Z",
            ),
            ev(
                EventKind::CheckIn,
                Some("leo"),
                None,
                "possums",
                "2026-07-14T13:00:00Z",
            ),
        ];
        assert_eq!(fold_now(&events)[0].children, 1);
    }
}
