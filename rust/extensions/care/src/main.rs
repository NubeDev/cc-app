//! `care` — the native Tier-2 sidecar binary (milestone 04, docs/build/04-mobile-shell.md).
//!
//! A real host-platform child the host spawns + supervises over stdio. It
//! speaks the `lb-ext-native` child wire (init/health/call/shutdown) and
//! serves the `care.*` tool set — every m03 verb (center / room / child /
//! guardian / guardianship / enrollment) plus `ping`.
//!
//! ## Wire-in (milestone 04 prerequisite)
//!
//! At sidecar start we read the supervisor-injected identity from the
//! environment (`LB_EXT_WS` / `LB_EXT_TOKEN` / `LB_GATEWAY_URL` /
//! `LB_EXT_ID` / `LB_EXT_STORE_URL`) and build a `Care` impl whose
//! authz chokepoint carries a real [`lb_ext_native::SidecarClient`]
//! — verb bodies reach the host store + chokepoint over the callback.
//! A booted-node integration test (`tests/live_wire.rs`) proves one verb
//! round-trips end to end through this exact code path (the host booted
//! from `lb-node`'s `boot_full`, the care dispatcher, the real
//! `/mcp/call` bridge).

use care::Care;
use lb_ext_native::serve_stdio;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    // Build the impl from the supervisor-injected env at the BINARY
    // boundary (CLAUDE.md rule 5). The lib exposes `Care::boot(env)`
    // so the sidecar hands the env map across one responsibility
    // boundary (env-reading here; everything below lives off the impl).
    let env: std::collections::HashMap<String, String> = std::env::vars().collect();
    let care = Care::boot(&env)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Stamp the synthetic admin once the boot resolves. The host's
    // future per-call `LB_EXT_PRINCIPAL_JSON` stamp overrides this per
    // dispatch; the boot-time integration test paths in through here.
    eprintln!(
        "care: sidecar ready (ws={}, tools={})",
        care.ws,
        <Care as lb_ext_native::Tools>::tools(&care).len()
    );

    serve_stdio(care).await
}