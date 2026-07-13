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
#[path = "list_tests.rs"]
mod tests;
