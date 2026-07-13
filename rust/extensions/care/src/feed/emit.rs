//! The MOTION seam — `bus.publish` (a new entry exists) + `notify.send` (the
//! push) over the host callback. Owned here so the I/O lives in ONE named file
//! (FILE-LAYOUT) and `log::add` calls two intention-revealing functions instead
//! of hand-rolling `call_tool` payloads inline.
//!
//! ## State is the record; motion is best-effort re-derivable
//!
//! The durable `daily_log` row is the source of truth (it lands via the record
//! store BEFORE we get here). The bus event and the push are MOTION: "a new
//! entry exists". On the era-1 / unit-test path the chokepoint carries no host
//! client (`cp.host_client() == None`), so these are no-ops that return `Ok` —
//! the record still lands, and the feed/push simply don't fan out (there is no
//! gateway to fan them onto). A spawned production sidecar always carries the
//! client, so the emit + push fire.
//!
//! ## Localization is lb's job (the both-languages exit gate)
//!
//! `notify.send` takes `title_key` / `body_key` / `args` — lb renders each
//! recipient's locale server-side from their prefs (push-target scope). So one
//! `notify.send` for an incident yields English for Sam and Spanish for Ana.
//! cc-app supplies CATALOG KEYS, never words (CLAUDE.md rule 8). The decision
//! (who/whether/which keys) is [`crate::push::decide`]; this module only
//! performs the resolved call.
//!
//! ## bus.watch is NOT here
//!
//! The live subscribe side (`care.feed.watch`) is an HTTP SSE route on the
//! gateway (`GET /bus/{subject}/stream`), NOT a `call_tool` — lb rejects a
//! synchronous `bus.watch`. So emit (publish) is a callback verb and lives
//! here; subscribe (watch) is the gateway stream the guardian UI opens after
//! the reach-checked `feed.watch` authorization. See `feed/watch.rs`.

use lb_ext_native::SidecarClient;
use serde_json::json;

use crate::push::Decision;

/// Publish "a new entry exists" onto the child's per-child bus subject
/// (`crate::log::feed_subject`). The payload is the entry's JSON (the same shape
/// the SSE feed appends). Fire-and-forget: a publish failure is NON-fatal to the
/// write (the record already landed) — we swallow it so a transient bus fault
/// never fails an authorized log. `None` client (era-1/tests) ⇒ no-op `Ok`.
pub async fn publish_entry(
    client: Option<&SidecarClient>,
    subject: &str,
    entry: &serde_json::Value,
) {
    let Some(client) = client else { return };
    // Fire-and-forget: the record is the source of truth; a bus fault must not
    // fail the write. A future milestone routes the failure to the audit reactor.
    let _ = client
        .call_tool("bus.publish", json!({ "subject": subject, "payload": entry }))
        .await;
}

/// Send the push for one entry per the resolved [`Decision`]. A no-push
/// decision (`recipients` empty — feed-only, or nobody opted in) is a no-op.
/// `args` carries the `{{child}}` interpolation for the catalog keys (lb
/// localizes per recipient). An incident/medication is must-deliver → mapped to
/// `priority: "high"` (lb has no `must_deliver` field; priority is the urgency
/// knob). Returns the outbox `effect_id` on a real send (so the caller/tests can
/// assert the enqueue), `None` when there was no push or no client.
///
/// A push enqueue failure IS surfaced to the caller (unlike the bus publish):
/// an incident push is must-deliver, and a caller may want to log the failure —
/// but it is still not fatal to the already-landed record (the verb decides).
pub async fn send_push(
    client: Option<&SidecarClient>,
    decision: &Decision,
    child_id: &str,
) -> Option<String> {
    if !decision.is_push() {
        return None;
    }
    let client = client?;
    // must_deliver (incident/medication) → high priority; lb has no dedicated
    // must-deliver flag, and quiet hours are honored by lb per recipient prefs
    // (not suppressed here). collapse_key groups repeat pushes for one entry.
    let priority = if decision.must_deliver { "high" } else { "normal" };
    let out = client
        .call_tool(
            "notify.send",
            json!({
                "to": decision.recipients,
                "title_key": decision.title_key,
                "body_key": decision.body_key,
                "args": { "child": child_id },
                "deep_link": decision.deep_link,
                "priority": priority,
                "collapse_key": decision.deep_link,
            }),
        )
        .await
        .ok()?;
    out.get("effect_id")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}
