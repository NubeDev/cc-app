//! `care.log.list` — the daily-feed LEDGER, ROLE-FILTERED (CLAUDE.md rule 7),
//! CURSOR-PAGED. Cap: `mcp:care.log.list:call`.
//!
//! ## Three role scopes over ONE `daily_log` scan (same idiom as attendance)
//!
//! Scan once (`query_rows`), then keep only the rows the caller is entitled to:
//! - **Admin** (`reachable_children` yields `["*"]`) → every entry.
//! - **Staff** → entries whose `room_id` is in `reachable_rooms`.
//! - **Guardian** → entries whose `child_id` is in `reachable_children`; a
//!   guardian who reaches nothing ⇒ EMPTY (never an error — the list deny semantic).
//!
//! Optional `room_id` / `child_id` / `since` / `until` filters apply AFTER the
//! role scope, so a filter only NARROWS the authorized set. `since`/`until`
//! compare the fixed-width ISO `at` lexically (a byte compare is chronological).
//!
//! ## Cursor paging — stable `(at, row_id)` compound cursor
//!
//! `daily_log` rows carry no id in the body and a fan-out writes N rows at one
//! `at`, so paging on `at` alone is ambiguous. We page on the COMPOUND key
//! `(at, row_id)` (`row_id` from `query_rows`, sorted `at` ASC then `row_id` ASC:
//! a total, stable timeline order). The `after` cursor is the opaque
//! `"<at>|<row_id>"` of the previous page's last row; we return rows strictly
//! greater under that order. `next_cursor` is `Some(...)` when more rows remain,
//! `None` on the last page. `limit` defaults to 50, capped at 200.

use lb_auth::Principal;

use crate::authz::{reachable_children, reachable_rooms, Chokepoint};
use crate::log::DailyLog;

/// Default page size when the caller omits `limit`.
const DEFAULT_LIMIT: usize = 50;
/// Hard cap on page size — a caller can never ask for more than this.
const MAX_LIMIT: usize = 200;

#[derive(Debug, Default, serde::Deserialize)]
pub struct ListInput {
    /// Keep only entries in this room (exact match). Optional.
    #[serde(default)]
    pub room_id: Option<String>,
    /// Keep only entries for this child (exact match). Optional.
    #[serde(default)]
    pub child_id: Option<String>,
    /// Keep only entries with `at >= since` (inclusive, ISO string compare).
    #[serde(default)]
    pub since: Option<String>,
    /// Keep only entries with `at <= until` (inclusive, ISO string compare).
    #[serde(default)]
    pub until: Option<String>,
    /// Cursor: return only rows AFTER this `"<at>|<row_id>"` key. Optional.
    #[serde(default)]
    pub after: Option<String>,
    /// Page size (default 50, capped at 200).
    #[serde(default)]
    pub limit: Option<usize>,
}

/// One authorized `daily_log` row + its stable store key (the paging tiebreaker).
struct Row {
    id: String,
    entry: DailyLog,
}

impl Row {
    /// The compound cursor key for this row: `"<at>|<row_id>"`.
    fn cursor(&self) -> String {
        [self.entry.at.as_str(), self.id.as_str()].join("|")
    }
}

#[derive(serde::Serialize)]
struct ListReply {
    entries: Vec<DailyLog>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    // Empty input ⇒ no filters / default paging (all fields optional). A
    // malformed body is a caller bug, surfaced as an error.
    let params: ListInput = if input.trim().is_empty() {
        ListInput::default()
    } else {
        serde_json::from_str(input).map_err(|e| format!("invalid care.log.list input: {e}"))?
    };

    // Resolve the caller's reach ONCE. Admin ⇒ `["*"]`; guardian gets a
    // (possibly empty) child set; staff gets a room set.
    let reach_children = reachable_children(cp, principal).await;
    let is_admin = reach_children.iter().any(|r| r == "*");

    // One table scan — the whole `daily_log`, then filtered down by role.
    let all = all_rows(cp).await?;

    let scoped: Vec<Row> = if is_admin {
        all
    } else {
        let reach_rooms = reachable_rooms(cp, principal).await;
        // A non-admin is EITHER staff (rooms) OR a guardian (children); a row
        // is authorized if it lands in the caller's room scope OR its child is
        // in the caller's reached set. Deny-by-empty falls out naturally: a
        // guardian who reaches nothing keeps zero rows.
        all.into_iter()
            .filter(|r| is_authorized(&r.entry, &reach_rooms, &reach_children))
            .collect()
    };

    // Filters only NARROW the authorized set (never widen it).
    let mut rows: Vec<Row> = scoped
        .into_iter()
        .filter(|r| passes_filters(&r.entry, &params))
        .collect();

    // Timeline order: `at` ASC then the stable row id (total + stable order).
    rows.sort_by(|a, b| a.entry.at.cmp(&b.entry.at).then_with(|| a.id.cmp(&b.id)));

    // Skip everything up to and including the `after` cursor.
    if let Some(after) = &params.after {
        rows.retain(|r| r.cursor().as_str() > after.as_str());
    }

    // Take one page; `next_cursor` is Some iff rows remain past the page.
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let has_more = rows.len() > limit;
    let next_cursor = if has_more {
        rows.get(limit - 1).map(Row::cursor)
    } else {
        None
    };
    rows.truncate(limit);

    let reply = ListReply {
        entries: rows.into_iter().map(|r| r.entry).collect(),
        next_cursor,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

/// Is `entry` authorized for a NON-admin caller? Staff reach is by room; a
/// guardian's reach is by child. A row's `child_id` is always set (per-child
/// rows), so a guardian with the reaching edge matches; a staff with the room
/// assignment matches.
fn is_authorized(entry: &DailyLog, reach_rooms: &[String], reach_children: &[String]) -> bool {
    let room_ok = reach_rooms.iter().any(|r| r == &entry.room_id);
    let child_ok = reach_children.iter().any(|c| c == &entry.child_id);
    room_ok || child_ok
}

/// Apply the optional input filters (exact match on room/child; inclusive
/// lexical bounds on the ISO `at` string). A `None` filter is a no-op.
fn passes_filters(entry: &DailyLog, f: &ListInput) -> bool {
    if let Some(room) = &f.room_id {
        if &entry.room_id != room {
            return false;
        }
    }
    if let Some(child) = &f.child_id {
        if &entry.child_id != child {
            return false;
        }
    }
    if let Some(since) = &f.since {
        if entry.at.as_str() < since.as_str() {
            return false;
        }
    }
    if let Some(until) = &f.until {
        if entry.at.as_str() > until.as_str() {
            return false;
        }
    }
    true
}

/// Read the whole `daily_log` table as `(row_id, entry)` pairs. Malformed rows
/// are skipped (a garbage row can't leak, and can't fail an authorized read).
async fn all_rows(cp: &Chokepoint) -> Result<Vec<Row>, String> {
    let rows: Vec<(String, serde_json::Value)> = cp
        .records()
        .query_rows("daily_log")
        .await
        .map_err(|e| format!("store denied the log list: {e}"))?;
    let mut out = Vec::with_capacity(rows.len());
    for (id, value) in rows {
        if let Ok(entry) = serde_json::from_value::<DailyLog>(value) {
            out.push(Row { id, entry });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::child::create as child_create;
    use crate::guardianship::link as guardianship_link;
    use crate::log::add as log_add;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    /// One parametric principal builder — role + sub + caps.
    fn principal(
        signing: &SigningKey,
        sub: &str,
        ws: &str,
        role: Role,
        caps: &[&str],
    ) -> Principal {
        let claims = Claims {
            sub: sub.into(),
            ws: ws.into(),
            role,
            caps: caps.iter().map(|c| c.to_string()).collect(),
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }
    fn admin(k: &SigningKey, ws: &str) -> Principal {
        let caps = [
            "mcp:care.child.create:call",
            "mcp:care.guardianship.link:call",
            "mcp:care.log.list:call",
        ];
        principal(k, "user:admin", ws, Role::WorkspaceAdmin, &caps)
    }
    fn staff(k: &SigningKey, ws: &str) -> Principal {
        principal(
            k,
            "user:teacher",
            ws,
            Role::Member,
            &["mcp:care.log.add:call", "mcp:care.log.list:call"],
        )
    }
    fn member(k: &SigningKey, sub: &str, ws: &str) -> Principal {
        principal(k, sub, ws, Role::Member, &["mcp:care.log.list:call"])
    }

    async fn seed_child(cp: &Chokepoint, a: &Principal, id: &str) {
        let input =
            format!(r#"{{"id":"{id}","name":"{id}","dob":"2021-03-15","photo_consent":true}}"#);
        child_create::run(cp, a, &input).await.expect("seed child");
    }

    /// Add one `note` entry for `child_id` in `room_id` at `at`, via the real
    /// write path (`log::add`), using a staff principal.
    async fn seed_entry(
        cp: &Chokepoint,
        p: &Principal,
        entry_id: &str,
        child_id: &str,
        room_id: &str,
        at: &str,
    ) {
        let input = format!(
            r#"{{"entry_id":"{entry_id}","child_ids":["{child_id}"],"room_id":"{room_id}","kind":"note","at":"{at}","note":"hi"}}"#
        );
        log_add::run(cp, p, &input).await.expect("seed entry");
    }

    /// Two families: Sam→(Leo, Mia); Ana→Leo. Leo in Possums, Mia in Wombats.
    /// Entries: 2 for Leo, 1 for Mia.
    async fn seed_two_families() -> (Arc<Store>, SigningKey) {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");
        let s = staff(&key, "acme");

        seed_child(&cp, &a, "child:leo").await;
        seed_child(&cp, &a, "child:mia").await;

        for input in [
            r#"{"guardian_sub":"user:sam","child_id":"child:leo","relationship":"father"}"#,
            r#"{"guardian_sub":"user:sam","child_id":"child:mia","relationship":"father"}"#,
            r#"{"guardian_sub":"user:ana","child_id":"child:leo","relationship":"mother"}"#,
        ] {
            guardianship_link::run(&cp, &a, input).await.expect("link");
        }

        seed_entry(
            &cp,
            &s,
            "log:leo:1",
            "child:leo",
            "room:possums",
            "2026-07-13T08:02:00Z",
        )
        .await;
        seed_entry(
            &cp,
            &s,
            "log:leo:2",
            "child:leo",
            "room:possums",
            "2026-07-13T15:30:00Z",
        )
        .await;
        seed_entry(
            &cp,
            &s,
            "log:mia:1",
            "child:mia",
            "room:wombats",
            "2026-07-13T08:10:00Z",
        )
        .await;

        (store, key)
    }

    fn entries_of(out: &str) -> Vec<serde_json::Value> {
        let v: serde_json::Value = serde_json::from_str(out).unwrap();
        v["entries"].as_array().cloned().unwrap_or_default()
    }

    #[tokio::test]
    async fn admin_lists_all_entries() {
        let (store, key) = seed_two_families().await;
        let cp = Chokepoint::new(store, "acme");
        let a = admin(&key, "acme");

        let out = run(&cp, &a, "").await.unwrap();
        let e = entries_of(&out);
        assert_eq!(e.len(), 3, "admin sees every family's entries");
        // Sorted ascending by `at`.
        assert_eq!(e[0]["at"], "2026-07-13T08:02:00Z");
        assert_eq!(e[1]["at"], "2026-07-13T08:10:00Z");
        assert_eq!(e[2]["at"], "2026-07-13T15:30:00Z");
    }

    /// RULE 7 CROSS-FAMILY ROW: Ana reaches Leo only — she must see Leo's two
    /// entries and NEVER Mia's. A leak here is the worst bug this product has.
    #[tokio::test]
    async fn guardian_sees_only_reached_childrens_entries() {
        let (store, key) = seed_two_families().await;
        let cp = Chokepoint::new(store, "acme");
        let ana = member(&key, "user:ana", "acme");

        let out = run(&cp, &ana, "").await.unwrap();
        let e = entries_of(&out);
        assert_eq!(e.len(), 2, "Ana sees Leo's two entries only");
        for row in &e {
            assert_eq!(row["child_id"], "child:leo");
            assert_ne!(
                row["child_id"], "child:mia",
                "MUST NOT leak Mia across families"
            );
        }
    }

    #[tokio::test]
    async fn guardian_with_no_reach_gets_empty_not_error() {
        let (store, key) = seed_two_families().await;
        let cp = Chokepoint::new(store, "acme");
        // A guardian with NO edge to any child reaches nothing.
        let stranger = member(&key, "user:stranger", "acme");

        let out = run(&cp, &stranger, "").await.expect("empty, not error");
        assert_eq!(entries_of(&out).len(), 0, "deny-by-empty, never an error");
    }

    #[tokio::test]
    async fn cursor_is_stable_across_pages() {
        // Five entries for Leo, all reachable by admin; page by 2.
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");
        let s = staff(&key, "acme");
        seed_child(&cp, &a, "child:leo").await;
        for (i, at) in [
            "2026-07-13T08:00:00Z",
            "2026-07-13T09:00:00Z",
            "2026-07-13T10:00:00Z",
            "2026-07-13T11:00:00Z",
            "2026-07-13T12:00:00Z",
        ]
        .iter()
        .enumerate()
        {
            let base = ["log:leo:", &i.to_string()].concat();
            seed_entry(&cp, &s, &base, "child:leo", "room:possums", at).await;
        }

        // Page 1: limit 2 → 2 rows + a next_cursor.
        let out1 = run(&cp, &a, r#"{"limit":2}"#).await.unwrap();
        let v1: serde_json::Value = serde_json::from_str(&out1).unwrap();
        let e1 = v1["entries"].as_array().unwrap();
        assert_eq!(e1.len(), 2);
        assert_eq!(e1[0]["at"], "2026-07-13T08:00:00Z");
        assert_eq!(e1[1]["at"], "2026-07-13T09:00:00Z");
        let cursor1 = v1["next_cursor"].as_str().expect("more rows remain");

        // Page 2: follow the cursor → next 2 rows + another cursor.
        let out2 = run(&cp, &a, &format!(r#"{{"limit":2,"after":"{cursor1}"}}"#))
            .await
            .unwrap();
        let v2: serde_json::Value = serde_json::from_str(&out2).unwrap();
        let e2 = v2["entries"].as_array().unwrap();
        assert_eq!(e2.len(), 2);
        assert_eq!(e2[0]["at"], "2026-07-13T10:00:00Z");
        assert_eq!(e2[1]["at"], "2026-07-13T11:00:00Z");
        let cursor2 = v2["next_cursor"].as_str().expect("one row remains");

        // Page 3 (last): the final row, next_cursor None.
        let out3 = run(&cp, &a, &format!(r#"{{"limit":2,"after":"{cursor2}"}}"#))
            .await
            .unwrap();
        let v3: serde_json::Value = serde_json::from_str(&out3).unwrap();
        let e3 = v3["entries"].as_array().unwrap();
        assert_eq!(e3.len(), 1);
        assert_eq!(e3[0]["at"], "2026-07-13T12:00:00Z");
        assert!(
            v3.get("next_cursor").is_none(),
            "last page has no next_cursor"
        );
    }

    #[tokio::test]
    async fn child_id_filter_narrows_within_authorized_set() {
        let (store, key) = seed_two_families().await;
        let cp = Chokepoint::new(store, "acme");
        let sam = member(&key, "user:sam", "acme");

        // Sam reaches both Leo and Mia (3 entries); filter to Leo → 2 rows.
        let unfiltered = run(&cp, &sam, "").await.unwrap();
        assert_eq!(
            entries_of(&unfiltered).len(),
            3,
            "Sam reaches both children"
        );

        let out = run(&cp, &sam, r#"{"child_id":"child:leo"}"#).await.unwrap();
        let e = entries_of(&out);
        assert_eq!(e.len(), 2, "filter narrows to Leo");
        assert!(e.iter().all(|r| r["child_id"] == "child:leo"));
    }
}
