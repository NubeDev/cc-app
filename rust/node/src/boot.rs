//! The cc-app boot ritual — filling lb's `BootConfig` at the binary boundary and
//! driving the embed seam.
//!
//! ## Doctrine (inherited verbatim from rubix-ai's src/boot.rs — sameness)
//!
//! - **Env is a binary concern.** We read `CC_*` (and fall back to defaults) HERE, at the binary
//!   boundary, and fill the struct. No library code below the seam reads env — that is lb's
//!   `embed-node-scope.md` doctrine, and cc-app honours it: everything below `boot_full` comes
//!   from the struct we hand it.
//! - **No special-casing lb (rule 10).** We reach the core only through the generic
//!   `BootConfig`/`boot_full` seam. Embedding grants no extra caps; a host-verb call still goes
//!   through lb's mediated wall, a tokenless HTTP call still 401s.
//! - **Symmetric nodes.** Role = `BootConfig` (gateway on/off by env), never an `if cloud`.
//!
//! ## State dir
//!
//! All on-disk state lives under the **repo-anchored `.cc-app/`** (gitignored, see `.gitignore`)
//! — same posture as rubix-ai's `.rubix-ai/`. Override with `CC_HOME` for a relocated deployment.
//! `CC_STORE_PATH` / `CC_EXT_UI_DIR` override their per-path default; unset ⇒ durable repo-local
//! paths (an installed extension survives a restart — NOT `mem://`).

use std::net::SocketAddr;
use std::path::PathBuf;

use lb_node::{BootConfig, CredentialMode, GatewayMode, RunningNode, SigningKey};

/// The cc-app repo dir — the anchor for all local on-disk state (`.cc-app/`). Every path we hand
/// lb is absolute and anchored HERE, never cwd-relative, so `cargo run` from any directory (and a
/// restart) resolves the same store + ext-UI dir. Resolved at the binary boundary from
/// `CARGO_MANIFEST_DIR` (the crate root), overridable by `CC_HOME` for a relocated deployment.
fn repo_dir() -> PathBuf {
    std::env::var_os("CC_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

/// Create `dir` (best-effort logged) and return it as an absolute string. All installed-extension
/// on-disk state lives under `.cc-app/` (gitignored); we materialise the leaf on boot so lb's store
/// open / ext-UI serve find it already present.
fn ensure_dir(dir: PathBuf) -> String {
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("cc-node: could not create {}: {e}", dir.display());
    }
    dir.to_string_lossy().into_owned()
}

/// The durable node store — holds installed wasm/manifest artifacts + all node state (ONE datastore).
/// `CC_STORE_PATH` overrides; unset ⇒ the repo-anchored `.cc-app/store` (persistent, not `mem://`,
/// so an installed extension survives a restart).
fn store_path() -> String {
    match std::env::var("CC_STORE_PATH")
        .ok()
        .filter(|s| !s.is_empty())
    {
        Some(p) => p,
        None => ensure_dir(repo_dir().join(".cc-app").join("store")),
    }
}

/// Where the gateway serves installed-extension **UI bundles** from (`/extensions/<id>/ui/<file>`).
/// `CC_EXT_UI_DIR` overrides; unset ⇒ the repo-anchored `.cc-app/extensions/installed/ui`.
fn ext_ui_dir() -> String {
    match std::env::var("CC_EXT_UI_DIR")
        .ok()
        .filter(|s| !s.is_empty())
    {
        Some(p) => p,
        None => ensure_dir(
            repo_dir()
                .join(".cc-app")
                .join("extensions")
                .join("installed")
                .join("ui"),
        ),
    }
}

/// The bind address cc-node serves its gateway on. Read from `CC_GATEWAY_ADDR` at the binary
/// boundary; defaults to `127.0.0.1:18099` (distinct from lb's dev `:8080` and rubix-ai's `:8099`,
/// so cc-app + rubix-ai can run side-by-side during co-development).
fn gateway_addr() -> anyhow::Result<SocketAddr> {
    let raw = std::env::var("CC_GATEWAY_ADDR").unwrap_or_else(|_| "127.0.0.1:18099".into());
    raw.parse()
        .map_err(|e| anyhow::anyhow!("CC_GATEWAY_ADDR '{raw}' is not a socket address: {e}"))
}

/// The token-signing key. A stable key from `CC_SIGNING_KEY` (64 hex chars = 32-byte Ed25519 seed)
/// when set — a deployed node wants a key that survives a restart — otherwise a fresh per-boot key.
/// Custody is ours, at the binary boundary; the seed is never logged. Mirrors the rubix-ai shape.
fn signing_key() -> SigningKey {
    let Ok(hex) = std::env::var("CC_SIGNING_KEY") else {
        return SigningKey::generate();
    };
    let hex = hex.trim();
    if hex.len() != 64 {
        eprintln!(
            "CC_SIGNING_KEY: expected 64 hex chars (32-byte seed), got {} — using a fresh key",
            hex.len()
        );
        return SigningKey::generate();
    }
    let mut seed = [0u8; 32];
    for (i, byte) in seed.iter_mut().enumerate() {
        match u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16) {
            Ok(b) => *byte = b,
            Err(_) => {
                eprintln!("CC_SIGNING_KEY: not valid hex — using a fresh key");
                return SigningKey::generate();
            }
        }
    }
    SigningKey::from_seed(&seed)
}

/// Assemble the [`BootConfig`] for a cc-node from the binary-boundary environment.
///
/// Starts from lb's embed-friendly `default()` (the `#[non_exhaustive]` struct forbids a cross-crate
/// literal — `default()`-then-mutate is the supported construction path) and sets the fields cc-app
/// controls: a persistent store (repo-anchored `.cc-app/store`, or `CC_STORE_PATH`), the ext-UI
/// serve dir (`.cc-app/extensions/installed/ui`, or `CC_EXT_UI_DIR`), our signing key, the gateway
/// on our bind addr, reactors on. `hello_demo` stays `false` (a product node does not
/// load lb's demo extension). `seed_user` stays lb's default so a fresh node has an admin member
/// to log in as — an operator overrides via `CC_SEED_USER` (empty ⇒ no seed).
pub fn boot_config() -> anyhow::Result<BootConfig> {
    let mut cfg = BootConfig::default();
    cfg.store_path = Some(store_path());
    cfg.ext_ui_dir = Some(ext_ui_dir());
    cfg.signing_key = signing_key();
    cfg.gateway = GatewayMode::Addr(gateway_addr()?);
    cfg.reactors = true;
    cfg.hello_demo = false;
    cfg.credential_mode = credential_mode();
    if let Ok(user) = std::env::var("CC_SEED_USER") {
        cfg.seed_user = if user.is_empty() { None } else { Some(user) };
    }
    // Seed the dev admin's login credential so a `PasswordHash` node (CC_PASSWORD_LOGIN=1) has a
    // first admin who can log in — otherwise `ada` has no credential and every login 401s (the
    // bootstrap paradox). Read from `CC_SEED_PASSWORD` at the binary boundary; empty/unset ⇒ no
    // credential seeded (correct for the password-less `DevTrustAny` default). Secret-class.
    cfg.seed_credential = std::env::var("CC_SEED_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty());
    if let Ok(ws) = std::env::var("CC_WORKSPACE") {
        if !ws.is_empty() {
            cfg.workspace = ws;
        }
    }
    Ok(cfg)
}

/// Which login credential check the gateway runs, read at the binary boundary from
/// `CC_PASSWORD_LOGIN` (rule 5 — env is a binary concern, mapped here into `BootConfig`).
///
/// Default is **password-less** (`DevTrustAny`): the dev bootstrap handle (`CC_SEED_USER`,
/// `user:ada`) has no credential record, so it can only sign in when passwords aren't checked —
/// this keeps `make dev` + the dev-handle e2e ergonomic. Set `CC_PASSWORD_LOGIN=1` (the
/// `email-login` demo + any real deployment) to select `PasswordHash`: `POST /login` then argon2s
/// the credential a person set at invite-accept, and a wrong/absent secret 401s. Under that mode the
/// dev handle can no longer log in password-less — provision a credential for it, or use an
/// accepted email account (embedder-credential-mode scope, the "absent credential locks out" note).
fn credential_mode() -> CredentialMode {
    match std::env::var("CC_PASSWORD_LOGIN") {
        Ok(v) if !v.trim().is_empty() => CredentialMode::PasswordHash,
        _ => CredentialMode::DevTrustAny,
    }
}

/// The workspace cc-node boots into (and installs the care sidecar in). Read
/// from `CC_WORKSPACE` at the binary boundary, defaulting to lb's `acme` — the
/// SAME value `boot_config()` stamps onto `BootConfig::workspace`, so the
/// install lands in the workspace the node actually serves.
pub fn workspace() -> String {
    std::env::var("CC_WORKSPACE")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "acme".into())
}

/// The gateway URL a spawned sidecar POSTs its host-callbacks to. Read from
/// `CC_GATEWAY_URL` at the binary boundary; this is what `care_mount` mirrors
/// into `LB_GATEWAY_URL` so the care child gets a callback address (Part B).
/// `None` ⇒ no callback address (the sidecar's record I/O would fail cleanly).
pub fn gateway_url() -> Option<String> {
    std::env::var("CC_GATEWAY_URL")
        .ok()
        .filter(|s| !s.is_empty())
}

/// Boot a cc-node through lb's embed seam and hand back the [`RunningNode`]. The caller serves it.
/// Mirrors rubix-ai's `boot()` verbatim, swapping `RUBIX_*` → `CC_*` and the port.
pub async fn boot() -> anyhow::Result<RunningNode> {
    let cfg = boot_config()?;
    println!(
        "cc-node: booting embedded lb node (workspace={}, gateway={:?}, store={})",
        cfg.workspace,
        cfg.gateway,
        cfg.store_path.as_deref().unwrap_or("mem://"),
    );
    lb_node::boot_full(cfg).await
}
