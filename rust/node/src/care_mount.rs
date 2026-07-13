//! Boot-time install of the `care` native sidecar — the cc-app analog of lb's
//! `control_engine::mount`. This is the fix for Part A: the cc-node binary now
//! SPAWNS + REGISTERS the care Tier-2 sidecar at boot via lb's real native
//! install path (`install_native` + `lb_supervisor::OsLauncher`), so its caps
//! are granted and every `care.*` verb is reachable over `POST /mcp/call`.
//! Without this the tool catalog exposes zero care tools and every `care.*`
//! call 403s (no cap, no sidecar to serve it).
//!
//! ## Doctrine
//!
//! - **Not publish-ext / lb-pack.** Those pack a WASM artifact; care is a
//!   NATIVE binary, so we use the native install seam directly (the same one
//!   lb's `control_engine` role uses). `exec` in `extension.toml` resolves
//!   against the install dir we hand `install_native` (the workspace target
//!   dir where `cargo build` writes the `care` binary).
//! - **Rule 5 — env is a binary concern.** We read `CC_*` here, at the binary
//!   boundary, and thread values in. `install_native` injects the sidecar's
//!   callback address from `LB_GATEWAY_URL`; cc-app exposes the gateway as
//!   `CC_GATEWAY_URL`, so we mirror it into `LB_GATEWAY_URL` here (once) so the
//!   spawned child gets a callback address and its record I/O reaches the
//!   node's durable store (Part B).
//! - **Rule 10 — no special-casing.** `install_native` is generic; the manifest
//!   is the contract. cc-node names no lb internals beyond the public install
//!   seam.
//!
//! ## Idempotent
//!
//! `install_native` is idempotent on ext id (a second install stops the running
//! child and re-spawns). A re-boot re-installs from the same manifest + binary.

use std::path::PathBuf;
use std::sync::Arc;

use lb_auth::{mint, verify, Claims, Principal, Role, SigningKey};
use lb_host::{install_native, Node};
use lb_supervisor::OsLauncher;

/// The care extension manifest — compiled in so the binary needs no file at
/// this path at run time (the same source `make pack` would sign). Its
/// `[capabilities].request` + `[[tools]]` are the full care verb surface.
const MANIFEST: &str = include_str!("../../extensions/care/extension.toml");

/// The care extension id (matches `[extension] id` in the manifest).
const CARE_ID: &str = "care";

/// Wall-clock seconds since the Unix epoch — the install's `now`, read at the
/// binary boundary (the no-wall-clock rule keeps time out of the core crates;
/// a binary may read it).
fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// The admin service principal the install acts as: it holds exactly the native
/// install gate (`mcp:native.install:call`). The install grant the sidecar ends
/// up with is `manifest.request ∩ admin_approved` — computed by `install_native`
/// from [`approved_grant`] below, NOT from this principal's caps. (A real admin
/// session replaces this bootstrap identity later, as lb's own mounts note.)
fn admin_principal(ws: &str) -> Principal {
    let key = SigningKey::generate();
    let claims = Claims {
        sub: "ext:care-bootstrap".into(),
        ws: ws.into(),
        role: Role::WorkspaceAdmin,
        caps: vec!["mcp:native.install:call".into()],
        iat: 0,
        exp: u64::MAX,
        constraint: None,
        run_id: None,
    };
    verify(&key, &mint(&key, &claims), 1).expect("freshly minted bootstrap token verifies")
}

/// Resolve the directory holding the built `care` binary. `cargo run`/`cargo
/// build` writes it to the workspace `target/<profile>/` dir; a release run
/// uses `release`. Overridable with `CC_CARE_DIR` for a packaged deployment
/// (where the binary lives beside the node, not in a cargo target dir).
fn care_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CC_CARE_DIR") {
        return PathBuf::from(dir);
    }
    // node/ is a workspace member; the shared target/ is one level up from the
    // crate manifest dir (`rust/node/` → `rust/target/`).
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    manifest_dir.join("..").join("target").join(profile)
}

/// The dev-bootstrap approved grant: mirror the manifest's requested surface so
/// `install_native`'s `requested ∩ approved` intersection grants the full care
/// verb + store + reach set on this node. A real deployment narrows this per an
/// admin approval flow; the demo node approves everything the manifest asks for.
///
/// Kept in lock-step with `extension.toml`'s `request` — a cap requested there
/// but absent here would be silently dropped from the grant (the sidecar would
/// then 403 on that callback). The care crate's `Tools::TOOLS` handshake catches
/// a verb/tool mismatch; this list is the cap mirror of the same contract.
fn approved_grant() -> Vec<String> {
    let mut approved = Vec::new();
    // 1. per-verb wall caps
    for verb in [
        "ping",
        "center.create",
        "center.get",
        "center.list",
        "room.create",
        "room.get",
        "room.list",
        "child.create",
        "child.get",
        "child.list",
        "child.update",
        "child.archive",
        "guardian.create",
        "guardian.get",
        "guardian.list",
        "guardianship.link",
        "guardianship.unlink",
        "guardianship.update",
        "enrollment.create",
        "enrollment.list",
        "enrollment.update",
        "invite.create_guardian",
        "invite.create_staff",
        "invite.list",
        "invite.resend",
        "invite.revoke",
        // milestone 06 — attendance
        "attendance.check_in",
        "attendance.check_out",
        "attendance.list",
        "attendance.now",
        "attendance.correct",
        // milestone 07 — menus
        "menu.set",
        "menu.get",
        "menu.week",
        "menu.copy_week",
        // milestone 08 — daily feed
        "log.add",
        "log.list",
        "log.correct",
        "log.day",
        "feed.watch",
        // milestone 09 — messaging
        "channel.reconcile",
        "announce.post",
    ] {
        approved.push(format!("mcp:care.{verb}:call"));
    }
    // 2. generic store surface (Part B) — outer verb caps + per-table inner gate
    for verb in ["store.write", "store.query", "store.delete"] {
        approved.push(format!("mcp:{verb}:call"));
    }
    for table in [
        "center",
        "room",
        "child",
        "guardian",
        "guardianship",
        "enrollment",
        "invite",
        "attendance_event",
        "menu",
        "daily_log",
    ] {
        approved.push(format!("store:{table}:write"));
    }
    // milestone 08 — the motion seams (bus emit + push + media photo path). The
    // guardian SSE (`bus.watch`) is opened by the UI with its own token; the
    // sidecar holds it so the reach-checked `feed.watch` authorization is coherent.
    for verb in [
        "bus.publish",
        "bus.watch",
        "notify.send",
        "media.upload_begin",
        "media.upload_commit",
        "media.get",
        // milestone 09 — the channel host surface (provision + post over the
        // callback; membership is granted via grants.assign below).
        "channel.create",
        "channel.post",
        "channel.history",
        "channel.list",
    ] {
        approved.push(format!("mcp:{verb}:call"));
    }
    // milestone 09 — the channel wildcard HOLDS. lb's grants_assign no-widening
    // rule requires the care sidecar to HOLD a cap matching what it grants, so it
    // holds `bus:chan/care-**:{pub,sub}` (scoped to the `care-` channel prefix) to
    // mint per-channel `bus:chan/care-child-<id>:sub` / `:pub` for members — the
    // same idiom as the `store:media/**:read` media serve-grant hold.
    approved.push("bus:chan/care-**:pub".to_string());
    approved.push("bus:chan/care-**:sub".to_string());
    // milestone 08 — the media serve-grant wildcard hold (requested in
    // extension.toml). Same no-widening reason: care holds `store:media/**:read`
    // so it can grant a per-photo `store:media/{id}:read` to feed recipients.
    approved.push("store:media/**:read".to_string());
    // 3. era-2 reach + grant derivation + invite host-callback verbs. Includes
    //    `care.reach.child` — lb's `grants.assign` no-widening rule requires the
    //    granter to HOLD the cap it grants, and `guardianship.link` grants
    //    `mcp:care.reach.child:call` scoped to a child (`authz/caps.rs`
    //    REACH_CAP). Without it the derive step 403s and the link rolls back.
    for verb in [
        "authz.check_scoped",
        "authz.scope_filter",
        // The delegation marker (native-caller-identity scope, node-v0.4.0):
        // lets the chokepoint name a `subject` (the frame caller's guardian)
        // on the reach verbs so rule 7 is enforced ABOUT the caller. Mirrors
        // `extension.toml`'s request — a drift would 403 every guardian read.
        "authz.delegate_reach",
        "grants.assign",
        "grants.revoke",
        "care.reach.child",
        "invite.create",
        "invite.resend",
        "invite.revoke",
    ] {
        approved.push(format!("mcp:{verb}:call"));
    }
    approved
}

/// Mount the care extension on `node`: install + supervise the native `care`
/// sidecar with the full care grant, so its verbs are reachable over the
/// gateway immediately. Best-effort with a clear log on failure — a node that
/// fails to install care still serves the host layer (login/wall), so an
/// operator sees WHY care is unreachable rather than a silent zero-tool catalog.
///
/// `ws` is the workspace to install into (cc-node's `CC_WORKSPACE`).
/// `gateway_url` is the node's own gateway address — mirrored into
/// `LB_GATEWAY_URL` so `install_native` injects it as the sidecar's callback
/// address (the child POSTs its `store.*` / `authz.*` calls back here).
pub async fn mount(node: Arc<Node>, ws: &str, gateway_url: Option<&str>) {
    // `install_native` reads `LB_GATEWAY_URL` from the host env to tell the
    // spawned child where to POST its callbacks. cc-app carries the gateway as
    // `CC_GATEWAY_URL`; mirror it once here (rule 5 — env at the boundary) so
    // the child's record I/O (Part B) reaches the node's durable store. Without
    // it the sidecar spawns with no callback address and every record write
    // fails cleanly (visible, not a silent private store).
    if let Some(url) = gateway_url {
        if std::env::var("LB_GATEWAY_URL").is_err() {
            std::env::set_var("LB_GATEWAY_URL", url);
        }
    }

    let dir = care_dir();
    let bin = dir.join(CARE_ID);
    if !bin.exists() {
        eprintln!(
            "care: sidecar binary not found at {} — build it with `cargo build -p care` \
             (or `make build-be`); skipping install. care.* verbs will 403 until it is present.",
            bin.display()
        );
        return;
    }
    let dir_str = dir.to_string_lossy().into_owned();

    let admin = admin_principal(ws);
    let approved = approved_grant();
    let now = unix_seconds();

    match install_native(
        &node,
        &OsLauncher,
        &admin,
        ws,
        MANIFEST,
        &dir_str,
        &approved,
        now,
    )
    .await
    {
        Ok(s) => println!(
            "care: installed native sidecar in '{ws}' ({} tools, {} caps granted)",
            s.tools.len(),
            s.granted_caps.len(),
        ),
        Err(e) => eprintln!(
            "care: sidecar install FAILED in '{ws}': {e} — care.* verbs will be unreachable"
        ),
    }
}
