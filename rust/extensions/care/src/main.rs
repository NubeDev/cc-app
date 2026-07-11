//! `care` — the native Tier-2 sidecar binary (milestone 02, docs/build/02-care-skeleton-authz.md).
//!
//! A real host-platform child the host spawns + supervises over stdio. It
//! speaks the `lb-ext-native` child wire (init/health/call/shutdown) and
//! serves the `care.*` tool set. Today that set is just `care.ping` — the
//! loop proof. Per-verb folders (center/room/child/…) wire up in later
//! milestones; the authz chokepoint is the only verb-level surface this
//! milestone ships (milestone 02's deliverable).

use care::Care;
use lb_ext_native::serve_stdio;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    // The host spawns us with our scoped identity in env (native-tier
    // injection). We carry it on the impl so every verb body is workspace-
    // scoped without plumbing it through params (host-metrics's shape).
    let ws = std::env::var("LB_EXT_WS").unwrap_or_default();
    serve_stdio(Care::new(ws)).await
}
