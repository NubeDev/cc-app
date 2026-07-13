//! `care.log.day` — the "Leo's day" rollup (UI + AI). Cap: `mcp:care.log.day:call`.
//!
//! A SINGLE-child read, so it `assert_reach`es on the child BEFORE any store read
//! (same idiom as `child::get`; rule 7) — a miss is a 403, never a phantom empty
//! rollup. Admin passes via the chokepoint's audited role check; a linked guardian
//! passes; a stranger is denied.
//!
//! The NET timeline drops corrected originals (append-only + compensating rows):
//! a row is superseded iff some OTHER row's `correction_of` equals its id, so we
//! scan via `query_rows` (`(id, data)` pairs), drop superseded originals, keep
//! corrections. Reply `summary` is a SPARSE per-kind tally (only non-zero kinds).

use std::collections::{BTreeMap, HashSet};

use lb_auth::Principal;
use serde::Serialize;

use crate::authz::{assert_reach, Chokepoint};
use crate::log::{DailyLog, LogKind};

#[derive(Debug, serde::Deserialize)]
pub struct DayInput {
    /// The child to roll up (required). Reach-gated FIRST.
    pub child_id: String,
    /// The ISO date (`YYYY-MM-DD`, required) — keep entries whose `at` starts
    /// with this prefix (`at` is `YYYY-MM-DDTHH:MM:SS...`).
    pub date: String,
}

#[derive(Debug, Serialize)]
struct DayReply {
    child_id: String,
    date: String,
    /// The net timeline (superseded originals dropped), `at` ascending.
    entries: Vec<DailyLog>,
    /// Per-kind tally over `entries` — only kinds with count > 0.
    summary: BTreeMap<String, usize>,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: DayInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.log.day input: {e}"))?;

    // Reach check FIRST — 403 on miss (never a phantom empty rollup).
    assert_reach(cp, principal, &parsed.child_id)
        .await
        .map_err(|e| format!("{e}"))?;

    // Scan with ids (query_rows) so we can resolve `correction_of` references.
    let rows: Vec<(String, serde_json::Value)> = cp
        .records()
        .query_rows("daily_log")
        .await
        .map_err(|e| format!("store denied the day rollup: {e}"))?;

    // The set of row ids that some OTHER row corrects — the superseded originals.
    let mut superseded: HashSet<String> = HashSet::new();
    for (_id, data) in &rows {
        if let Some(target) = data.get("correction_of").and_then(|v| v.as_str()) {
            superseded.insert(target.to_string());
        }
    }

    // Keep rows for THIS child on THIS date that are not superseded; malformed
    // rows are skipped (same posture as attendance::list).
    let mut entries: Vec<DailyLog> = Vec::with_capacity(rows.len());
    for (id, data) in rows {
        if superseded.contains(&id) {
            continue;
        }
        let log: DailyLog = match serde_json::from_value(data) {
            Ok(l) => l,
            Err(_) => continue,
        };
        if log.child_id != parsed.child_id {
            continue;
        }
        if !log.at.starts_with(&parsed.date) {
            continue;
        }
        entries.push(log);
    }

    // Timeline order: the ISO-8601 `at` sorts chronologically as a byte string.
    entries.sort_by(|a, b| a.at.cmp(&b.at));

    // Per-kind tally over the net set (sparse — only non-zero kinds).
    let mut summary: BTreeMap<String, usize> = BTreeMap::new();
    for e in &entries {
        *summary.entry(kind_key(e.kind).to_string()).or_insert(0) += 1;
    }

    let reply = DayReply {
        child_id: parsed.child_id,
        date: parsed.date,
        entries,
        summary,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

/// The stable enum key for a kind (`meal`/`nap`/...) — the `summary` map key,
/// matching the record's serialized `kind` value.
fn kind_key(kind: LogKind) -> &'static str {
    kind.key()
}

#[cfg(test)]
#[path = "day_tests.rs"]
mod tests;
