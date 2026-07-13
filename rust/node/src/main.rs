//! cc-node — the cc-app host binary. Thin boot shim (docs/build/01-host-boot.md).
//!
//! Fills lb's `BootConfig` at the binary boundary via [`boot`] (CC_* env),
//! boots the embedded lb node through the supported `boot_full` seam, and
//! serves the gateway. No product logic — the domain is the `care` extension.
//!
//! Embedding grants no extra caps (CLAUDE.md rule 10): a host-verb call still
//! goes through lb's mediated wall, a tokenless HTTP call still 401s. We
//! reach the core only through the generic `BootConfig`/`boot_full` seam,
//! the same way `NubeIO/rubix-ai` does — sameness is the point (the milestone
//! 01 doc explicitly says "copy rubix-ai's file shapes rather than inventing").

mod boot;
mod care_mount;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Boot the embedded lb node through the generic seam (CC_* env filled
    // into BootConfig at the binary boundary — CLAUDE.md rule 5).
    let running = boot::boot().await?;

    // Install + supervise the care native sidecar so its verbs are reachable
    // over `/mcp/call` (Part A). Without this the node serves a zero-care-tool
    // catalog and every `care.*` call 403s. Best-effort: a failed install logs
    // WHY and the node still serves the host layer. The sidecar reaches the
    // node's DURABLE store over the host callback (Part B) — the node is the
    // single source of truth; the child owns no durable store of its own.
    care_mount::mount(
        running.node.clone(),
        &boot::workspace(),
        boot::gateway_url().as_deref(),
    )
    .await;

    // `serve()` blocks serving the gateway until the process is stopped.
    running.serve().await
}
