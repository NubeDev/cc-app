//! `RecordStore` — the ONE seam every care verb reads and writes domain
//! records through, with two interchangeable backends behind an identical
//! API. This is the Part-B chokepoint: the node's durable store is the single
//! source of truth, and a spawned sidecar owns NO durable store of its own.
//!
//! ## Why this exists
//!
//! A native Tier-2 sidecar runs in its OWN process. The supervisor injects its
//! identity + callback address (`LB_EXT_WS` / `LB_EXT_TOKEN` / `LB_GATEWAY_URL`)
//! but NOT a shared store URL — by design (native-callback-transport scope). A
//! sidecar that opened its own `lb_store::Store` would write to a PRIVATE store
//! the node/gateway never sees, and that store dies on restart. So the correct
//! long-term architecture is: the sidecar reaches records over the host
//! callback, using lb's generic `store.*` MCP surface (`store.write` /
//! `store.query` / `store.delete`, workspace-scoped + cap-gated), exactly as
//! any other tool call. The node's durable store answers; data survives a
//! restart and is visible to admin reads (rule 10 — a generic seam, no
//! special-casing).
//!
//! ## Two backends, one API
//!
//! - [`RecordStore::Callback`] — the PRODUCTION path. Every op is a
//!   `SidecarClient::call_tool` to the host `store.*` verb. The node's store is
//!   the source of truth.
//! - [`RecordStore::Local`] — the era-1 / unit-test path. Wraps
//!   `Arc<lb_store::Store>` and calls `lb_store::{read,create,write,delete,
//!   query_ws}` directly (the in-process store the matrix harness seeds through
//!   the real write path). Identical observable behaviour, so a verb body reads
//!   the same whichever backend the chokepoint carries.
//!
//! The two backends return byte-identical shapes: `read` unwraps the `{ data }`
//! envelope exactly like `lb_store::read`; `query_data` yields the inner record
//! values (the `SELECT data` rows); `create` preserves the first-settle
//! `AlreadyExists` conflict semantic (the host `store.write` UPSERTs, so the
//! callback path reads-then-checks to reproduce it).

use std::sync::Arc;

use lb_ext_native::{CallError, SidecarClient};
use serde_json::{json, Value};

/// The typed outcome of a record op — mapped by verb bodies onto their own
/// `StoreDenied` / `AlreadyExists` / `NotFound` surfaces. One error enum so a
/// verb never has to know which backend answered.
#[derive(Debug)]
pub enum RecordError {
    /// A first-write conflict — the id already exists (the `lb_store::create`
    /// first-settle semantic, reproduced on the callback path by a pre-read).
    Conflict,
    /// The store/callback refused or faulted (cap denial, transport, decode).
    /// Carries a short reason for the verb's audit string; opaque to callers.
    Store(String),
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordError::Conflict => write!(f, "conflict"),
            RecordError::Store(s) => write!(f, "{s}"),
        }
    }
}

/// The record read/write seam. Cloneable and cheap (both variants hold an
/// `Arc`/pooled client); constructed once on the [`super::Chokepoint`] and
/// shared across every dispatch.
#[derive(Clone)]
pub enum RecordStore {
    /// In-process store (era-1 fallback + unit/integration tests).
    Local {
        store: Arc<lb_store::Store>,
        ws: String,
    },
    /// Host-callback store (production sidecar) — the node's durable store.
    Callback { client: SidecarClient, ws: String },
}

impl RecordStore {
    /// The workspace every op is scoped to (diagnostics; the wire ws is the
    /// token's on the callback path, the namespace on the local path).
    pub fn ws(&self) -> &str {
        match self {
            RecordStore::Local { ws, .. } => ws,
            RecordStore::Callback { ws, .. } => ws,
        }
    }

    /// Read `table:id`, unwrapping the `{ data }` envelope. `None` if absent in
    /// this workspace's namespace — byte-identical to `lb_store::read`.
    pub async fn read(&self, table: &str, id: &str) -> Result<Option<Value>, RecordError> {
        match self {
            RecordStore::Local { store, ws } => lb_store::read(store, ws, table, id)
                .await
                .map_err(|e| RecordError::Store(e.to_string())),
            RecordStore::Callback { client, .. } => {
                // `store.query` bounds + parse-allowlists a single SELECT; bind
                // the table/id as `$`-vars (never string interpolation). The
                // host wraps records under `{ data }`, so ask for `data` and
                // unwrap the single row exactly as `lb_store::read` does.
                let out = query(
                    client,
                    "SELECT data FROM ONLY type::thing($tb, $id)",
                    json!({ "tb": table, "id": id }),
                )
                .await?;
                // `store.query` returns `{ columns, rows }`; ONLY yields at most
                // one row: `{ "data": <record> }`. Absent ⇒ empty rows.
                let rows = out.get("rows").and_then(Value::as_array);
                match rows.and_then(|r| r.first()) {
                    Some(row) => Ok(row.get("data").cloned()),
                    None => Ok(None),
                }
            }
        }
    }

    /// First-write `value` at `table:id`. A duplicate id is [`RecordError::Conflict`]
    /// — the `lb_store::create` first-settle semantic. On the callback path the
    /// host `store.write` UPSERTs (no conflict), so we pre-read to reproduce the
    /// conflict; this keeps every `create` verb's `AlreadyExists` reply intact.
    pub async fn create(&self, table: &str, id: &str, value: &Value) -> Result<(), RecordError> {
        match self {
            RecordStore::Local { store, ws } => lb_store::create(store, ws, table, id, value)
                .await
                .map_err(|e| match e {
                    lb_store::StoreError::Conflict => RecordError::Conflict,
                    other => RecordError::Store(other.to_string()),
                }),
            RecordStore::Callback { client, .. } => {
                if self.read(table, id).await?.is_some() {
                    return Err(RecordError::Conflict);
                }
                write(client, table, id, value).await
            }
        }
    }

    /// UPSERT `value` at `table:id` (the `update`/mirror-write path — no
    /// conflict). Mirrors `lb_store::write`.
    pub async fn write(&self, table: &str, id: &str, value: &Value) -> Result<(), RecordError> {
        match self {
            RecordStore::Local { store, ws } => lb_store::write(store, ws, table, id, value)
                .await
                .map_err(|e| RecordError::Store(e.to_string())),
            RecordStore::Callback { client, .. } => write(client, table, id, value).await,
        }
    }

    /// Erase `table:id` (idempotent). Mirrors `lb_store::delete`.
    pub async fn delete(&self, table: &str, id: &str) -> Result<(), RecordError> {
        match self {
            RecordStore::Local { store, ws } => lb_store::delete(store, ws, table, id)
                .await
                .map_err(|e| RecordError::Store(e.to_string())),
            RecordStore::Callback { client, .. } => client
                .call_tool("store.delete", json!({ "table": table, "id": id }))
                .await
                .map(|_| ())
                .map_err(call_err),
        }
    }

    /// Every record's inner `data` value in `table` (the admin-path list). The
    /// same rows `SELECT data FROM <table>` yields via `query_ws` on the local
    /// path — a `Vec<Value>` of the unwrapped records.
    pub async fn query_data(&self, table: &str) -> Result<Vec<Value>, RecordError> {
        match self {
            RecordStore::Local { store, ws } => {
                let mut resp = store
                    .query_ws(
                        ws,
                        "SELECT data FROM type::table($tb)",
                        vec![("tb".into(), json!(table))],
                    )
                    .await
                    .map_err(|e| RecordError::Store(e.to_string()))?;
                let rows: Vec<Value> = resp.take::<Vec<Value>>((0, "data")).unwrap_or_default();
                Ok(rows)
            }
            RecordStore::Callback { client, .. } => {
                let out = query(
                    client,
                    "SELECT data FROM type::table($tb)",
                    json!({ "tb": table }),
                )
                .await?;
                let rows = out
                    .get("rows")
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(|r| r.get("data").cloned()).collect())
                    .unwrap_or_default();
                Ok(rows)
            }
        }
    }
}

/// One host `store.query` call, returning the raw `{ columns, rows }` object.
async fn query(client: &SidecarClient, sql: &str, vars: Value) -> Result<Value, RecordError> {
    client
        .call_tool("store.query", json!({ "sql": sql, "vars": vars }))
        .await
        .map_err(call_err)
}

/// One host `store.write` (UPSERT) call.
async fn write(
    client: &SidecarClient,
    table: &str,
    id: &str,
    value: &Value,
) -> Result<(), RecordError> {
    client
        .call_tool(
            "store.write",
            json!({ "table": table, "id": id, "value": value }),
        )
        .await
        .map(|_| ())
        .map_err(call_err)
}

/// Map a host-callback [`CallError`] onto a [`RecordError::Store`] with a short
/// reason for the verb's audit string. A `Denied` (403) means the sidecar's own
/// token lacks the `store.*` cap — a misconfiguration, surfaced as a store
/// refusal (fail closed), never silently swallowed.
fn call_err(e: CallError) -> RecordError {
    match e {
        CallError::Denied => RecordError::Store("store callback denied".into()),
        other => RecordError::Store(format!("store callback: {other}")),
    }
}
