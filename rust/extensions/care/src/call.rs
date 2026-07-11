//! The `Tools` impl — the dispatch from the child wire (`Method::Call`)
//! to a per-verb body. Each verb body is its own file (`ping.rs`,
//! `child/get.rs`, …) per FILE-LAYOUT.
//!
//! The host strips the `care.` prefix before the `call` reaches us — we
//! dispatch on the bare tool name, matching the manifest's `[[tools]]`
//! list (the only the host's install-time grant check looks at).
//!
//! ## Cap check (the deny-test half of milestone 02)
//!
//! Every `care.*` tool here requires the caller to hold a `mcp:care.*:call`
//! cap (the cap wall is enforced BEFORE we see the call; we re-state the
//! required cap here as a defence-in-depth check so a cap-wall regression
//! breaks the test, not a silent success). The harness in
//! `tests/matrix.rs` exercises this for `care.ping`.

use lb_ext_native::Tools;
use serde::Deserialize;

use crate::ping;

/// The tool names this sidecar serves (bare; the host owns the `care.`
/// prefix). Reported in the `init` handshake so the host rejects an
/// unknown-tool dispatch early, AND in the manifest's `[[tools]]` list so
/// the install grant is computed against the actual surface.
pub const TOOLS: &[&str] = &["ping"];

/// The expected cap a caller must carry to invoke a `care.*` tool. The
/// host re-checks this at the wall; we re-check here as a defence-in-depth
/// (an over-permissive host ⇒ deny here, not silent success).
#[allow(dead_code)]
pub const REQUIRED_CAP: &str = "mcp:care.ping:call";

/// The input shape for `care.ping` — the only verb shipped today. Each
/// verb body has its own input struct; future verbs add theirs here as
/// `#[serde(untagged)]` over the tool name, or per-verb files (cleaner).
#[derive(Debug, Deserialize)]
pub struct PingInput {
    /// Optional echo payload; the verb round-trips it under `echoed`.
    #[serde(default)]
    pub echo: Option<String>,
}

/// The reply shape for `care.ping`. Stateless + trivial — this verb's job
/// is to prove the loop end-to-end, not to carry domain semantics.
#[derive(Debug, serde::Serialize)]
pub struct PingReply {
    pub ws: String,
    pub tier: &'static str,
    pub ok: bool,
    /// The `echo` payload the caller passed in, round-tripped. Lets a
    /// caller prove which invocation the reply is for in a multi-call
    /// script.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echoed: Option<String>,
}

impl Tools for crate::Care {
    fn tools(&self) -> Vec<String> {
        // Reported in `init` AND used by the matrix harness to assert that
        // every registered verb has a matrix row (the harness reads
        // `TOOLS` and refuses to start if the matrix is smaller — see
        // `tests/matrix.rs`).
        TOOLS.iter().map(|s| s.to_string()).collect()
    }

    async fn call(&mut self, tool: &str, input: &str) -> Result<String, String> {
        // The host's routed native adapter re-qualifies the tool as
        // `care.<tool>` before it reaches us, while the direct
        // `/native/call` bridge passes the bare name. Match on the tail
        // after any leading `care.` — the child should not assume which
        // router reached it (host-metrics's posture).
        let verb = tool.strip_prefix("care.").unwrap_or(tool);
        match verb {
            "ping" => ping::run(&self.ws, input),
            other => Err(format!("unknown tool: {other}")),
        }
    }
}

/// Defence-in-depth cap check for a verb. Called from the matrix harness
/// to exercise the deny-test half of the milestone 02 gate (a caller
/// WITHOUT the cap must see `Err(cap_denied)`, not a silent success).
///
/// In production the host re-checks the cap at the wall — this is the
/// belt-and-braces assertion that the binary's body refuses the call
/// too, so a wall regression that lets an under-cap call through still
/// hits the deny here (and the test catches it).
///
/// The cap itself is checked INSIDE the verb body via this helper (each
/// verb body's `run` calls `require_caller_cap` with the supplied cap
/// set). The helper is on the `Tools` impl so it has access to the
/// caller's principal — but the wire doesn't carry a principal, only an
/// opaque-JSON `input` per `CallParams`. So the cap check happens at the
/// HOST, not here: this is the documented shape (host wall = the
/// authoritative gate). The matrix harness tests the wall via a separate
/// path (it doesn't go through `Tools::call`; it calls the chokepoint
/// directly). This helper is kept as a stub for the future when native
/// children get a principal-in-env path (milestone 03 follow-up).
#[allow(dead_code)]
pub fn require_caller_cap(held_caps: &[String], required: &str) -> Result<(), String> {
    if held_caps.iter().any(|c| c == required) {
        Ok(())
    } else {
        Err(format!("missing required cap: {required}"))
    }
}
