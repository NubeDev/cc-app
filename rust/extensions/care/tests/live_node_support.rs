//! Harness for the live-node wire-in proof (`live_node.rs`) — real node + real
//! gateway + real `install_native` (spawns the `care` OS child), and the HTTP
//! helpers to drive it over `/mcp/call`. Split out of the scenario file so each
//! stays one responsibility (FILE-LAYOUT ≤400).
//!
//! Included by `live_node.rs` via `#[path = "live_node_support.rs"] mod support`.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use lb_auth::{mint, verify, Claims, Role, SigningKey};
use lb_host::{install_native, Node};
use lb_role_gateway::{router, Gateway};
use lb_store::Store;
use serde_json::{json, Value};

pub const NOW: u64 = 1000;
pub const WS: &str = "acme";

/// The care manifest — the same source cc-node's `care_mount` compiles in.
const MANIFEST: &str = include_str!("../extension.toml");

/// The approved grant cc-node's `care_mount::approved_grant` hands `install_native`
/// (mirrored here so the test installs with the SAME grant production does — a
/// drift would let the test pass while production 403s). Kept in lock-step with
/// `extension.toml`'s `request`.
pub fn approved_grant() -> Vec<String> {
    let mut g = Vec::new();
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
        // milestone 06/07/08
        "attendance.check_in",
        "attendance.check_out",
        "attendance.list",
        "attendance.now",
        "attendance.correct",
        "menu.set",
        "menu.get",
        "menu.week",
        "menu.copy_week",
        "log.add",
        "log.list",
        "log.correct",
        "log.day",
        "feed.watch",
        "media.begin",
        "media.commit",
        "channel.reconcile",
        "announce.post",
    ] {
        g.push(format!("mcp:care.{verb}:call"));
    }
    for verb in ["store.write", "store.query", "store.delete"] {
        g.push(format!("mcp:{verb}:call"));
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
        g.push(format!("store:{table}:write"));
    }
    for verb in [
        "authz.check_scoped",
        "authz.scope_filter",
        "authz.delegate_reach",
        "grants.assign",
        "grants.revoke",
        "care.reach.child",
        "invite.create",
        "invite.resend",
        "invite.revoke",
        // milestone 08 — motion seams
        "bus.publish",
        "bus.watch",
        "notify.send",
        "media.upload_begin",
        "media.upload_commit",
        "media.get",
        // milestone 09 — channel host surface
        "channel.create",
        "channel.post",
        "channel.history",
        "channel.list",
    ] {
        g.push(format!("mcp:{verb}:call"));
    }
    // wildcard holds (media serve-grant + channel membership grants — the
    // no-widening rule; lock-step with care_mount::approved_grant + extension.toml)
    g.push("store:media/**:read".to_string());
    g.push("bus:chan/care.**:pub".to_string());
    g.push("bus:chan/care.**:sub".to_string());
    // milestone 10 — the feed-watch wildcard hold (lb#49 / node-v0.4.3).
    g.push("bus:care.feed.**:watch".to_string());
    g
}

/// The care binary the host spawns — the workspace `target/<profile>/care`
/// (built by `cargo build -p care`). Absolute, resolved from this crate.
fn care_install_dir() -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // extensions/care/ → ../../target/<profile>
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    manifest_dir
        .join("..")
        .join("..")
        .join("target")
        .join(profile)
        .to_string_lossy()
        .into_owned()
}

/// An admin token the gateway verifies (signed with the node key) carrying the
/// per-verb care caps the roster writes + reads need, PLUS an admin-only cap
/// (`mcp:members.manage:call`) so the host's caller projection derives
/// `admin = true` (native-caller-identity scope). This mirrors PRODUCTION:
/// lb mints every session with `role: Member` (the role enum is cosmetic), and
/// admin authority rides caps — so the token's role is `Member` here too, and
/// the chokepoint's admin-pass fires off the host-derived `admin` marker, NOT
/// the role. Without an admin-only cap this token would be treated as a
/// guardian and DENIED (the exact bug the live seed surfaced).
pub fn admin_token(key: &SigningKey, caps: &[&str]) -> String {
    let mut all: Vec<String> = caps.iter().map(|s| s.to_string()).collect();
    // The authoritative admin signal in lb (an `ADMIN_ONLY_CAPS` member). Its
    // presence is what makes the host stamp `caller.admin = true`.
    all.push("mcp:members.manage:call".into());
    let claims = Claims {
        sub: "user:ada".into(),
        ws: WS.into(),
        role: Role::Member,
        caps: all,
        iat: NOW - 1,
        exp: NOW + 100_000,
        constraint: None,
        run_id: None,
    };
    mint(key, &claims)
}

/// A guardian (Member) token carrying the reach-read caps — the exact grant a
/// guardian's session token holds. The chokepoint resolves THIS principal's
/// scoped grants over the callback (rule 7).
pub fn guardian_token(key: &SigningKey, sub: &str) -> String {
    let claims = Claims {
        sub: format!("user:{sub}"),
        ws: WS.into(),
        role: Role::Member,
        caps: vec![
            "mcp:care.child.get:call".into(),
            "mcp:care.child.list:call".into(),
            "mcp:authz.check_scoped:call".into(),
            "mcp:authz.scope_filter:call".into(),
        ],
        iat: NOW - 1,
        exp: NOW + 100_000,
        constraint: None,
        run_id: None,
    };
    mint(key, &claims)
}

/// The admin service principal `install_native` acts as (holds the native
/// install gate). Mirrors `care_mount::admin_principal`.
fn install_principal() -> lb_auth::Principal {
    let key = SigningKey::generate();
    let claims = Claims {
        sub: "ext:care-bootstrap".into(),
        ws: WS.into(),
        role: Role::WorkspaceAdmin,
        caps: vec!["mcp:native.install:call".into()],
        iat: 0,
        exp: u64::MAX,
        constraint: None,
        run_id: None,
    };
    verify(&key, &mint(&key, &claims), 1).expect("bootstrap token verifies")
}

/// Boot a real node over `store` + a real gateway on an ephemeral port, install
/// the care sidecar, and return `(node, node-key, base-url)`. The gateway key IS
/// the node key so it verifies the child's callback token (minted with the node
/// key by `install_native`). `LB_GATEWAY_URL` is set to the gateway base so the
/// spawned child knows where to POST its `store.*`/`authz.*` callbacks.
pub async fn boot_and_install(store: Store) -> (Arc<Node>, Arc<SigningKey>, String) {
    let node = Arc::new(Node::boot_with_store(store).await.expect("node boots"));
    let key = node.key();
    let gw = Gateway::new(node.clone(), (*key).clone(), NOW);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr: SocketAddr = listener.local_addr().unwrap();
    let base = format!("http://{addr}");
    let app = router(gw);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // The child POSTs its callbacks here — mirror the gateway base into
    // `LB_GATEWAY_URL` the way cc-node's `care_mount` does (rule 5, boundary).
    std::env::set_var("LB_GATEWAY_URL", &base);

    let admin = install_principal();
    let dir = care_install_dir();
    install_native(
        &node,
        &lb_supervisor::OsLauncher,
        &admin,
        WS,
        MANIFEST,
        &dir,
        &approved_grant(),
        NOW,
    )
    .await
    .expect("care sidecar installs + spawns");

    // The child needs a beat to connect its callback client + answer the init
    // handshake before the first routed call. A short poll on `care.ping`
    // over HTTP is the readiness gate (no fixed sleep).
    let http = reqwest::Client::new();
    let tok = admin_token(&key, &["mcp:care.ping:call"]);
    for _ in 0..50 {
        let code = mcp_status(&http, &base, &tok, "care.ping", json!({})).await;
        if code == 200 {
            return (node, key, base);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("care sidecar did not become reachable (care.ping never 200)");
}

/// One `POST /mcp/call`; returns `(status, body)`. The gateway returns the
/// tool's JSON output on success, but a PLAIN-TEXT error string (not JSON) on any
/// tool error → 403; we parse-or-wrap so assert messages show the real cause.
pub async fn mcp(
    http: &reqwest::Client,
    base: &str,
    token: &str,
    tool: &str,
    args: Value,
) -> (u16, Value) {
    let resp = http
        .post(format!("{base}/mcp/call"))
        .bearer_auth(token)
        .json(&json!({ "tool": tool, "args": args }))
        .send()
        .await
        .expect("mcp call sends");
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    let body = serde_json::from_str::<Value>(&text).unwrap_or(Value::String(text));
    (status, body)
}

/// Just the status of a `POST /mcp/call` (for the readiness poll).
async fn mcp_status(
    http: &reqwest::Client,
    base: &str,
    token: &str,
    tool: &str,
    args: Value,
) -> u16 {
    http.post(format!("{base}/mcp/call"))
        .bearer_auth(token)
        .json(&json!({ "tool": tool, "args": args }))
        .send()
        .await
        .map(|r| r.status().as_u16())
        .unwrap_or(0)
}

/// Seed the minimal roster over HTTP `/mcp/call` — the same calls `scripts/seed.sh`
/// makes. Panics on any non-200 (a real seed must land).
pub async fn seed_roster(http: &reqwest::Client, base: &str, token: &str) {
    let (c, b) = mcp(
        http,
        base,
        token,
        "care.center.create",
        json!({"id":"sunshine","name":"Sunshine Childcare","default_locale":"en"}),
    )
    .await;
    assert_eq!(c, 200, "center.create failed: {b}");

    let (c, b) = mcp(
        http,
        base,
        token,
        "care.room.create",
        json!({"id":"possums","name":"Possums","center_id":"sunshine"}),
    )
    .await;
    assert_eq!(c, 200, "room.create failed: {b}");

    let (c, b) = mcp(
        http,
        base,
        token,
        "care.guardian.create",
        json!({"id":"ana","name":"Ana García","email":"ana@familia.test","locale":"es"}),
    )
    .await;
    assert_eq!(c, 200, "guardian.create failed: {b}");

    let (c, b) = mcp(http, base, token, "care.child.create",
        json!({"id":"leo","name":"Leo García","dob":"2021-03-15","room_id":"possums","allergies":["peanuts"],"photo_consent":true,"locale":"es"})).await;
    assert_eq!(c, 200, "child.create failed: {b}");

    let (c, b) = mcp(http, base, token, "care.guardianship.link",
        json!({"guardian_sub":"ana","child_id":"leo","relationship":"mother","can_pickup":true,"receives_daily_feed":true,"emergency_contact":true,"locale":"es"})).await;
    assert_eq!(c, 200, "guardianship.link failed: {b}");
}
