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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Boot the embedded lb node through the generic seam (CC_* env filled
    // into BootConfig at the binary boundary — CLAUDE.md rule 5). `serve()`
    // blocks serving the gateway until the process is stopped.
    let running = boot::boot().await?;
    running.serve().await
}
