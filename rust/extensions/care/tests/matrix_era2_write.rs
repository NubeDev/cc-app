//! Era-2 WRITE path — the regression that proves the lb `grants.*` routing
//! fix is live and end to end (no mocks — CLAUDE.md rule 4).
//!
//! This is the forward-looking test: when the lb-side dispatcher routes
//! `grants.*` to `call_authz_tool` (the upstream fix at
//! `docs/debugging/authz/lb-grants-routing.patch`), a native extension can
//! mint scoped grants over the callback by calling
//! `SidecarClient::call_tool("grants.assign", …)` — the SAME client that
//! reads them back via `authz.scope_filter`. Before the fix, this exact
//! call returns `CallError::Denied` (the dispatcher's
//! `call_tool_at_depth` falls through to the generic extension-registry
//! path and denies — see `docs/debugging/authz/grants-verbs-not-on-mcp-callback-surface.md`).
//!
//! ## What it asserts
//!
//! 1. **grant mint over the callback works** — a `SidecarClient` whose
//!    token holds `mcp:grants.assign:call` succeeds against a real booted
//!    gateway (the patched lb's `/mcp/call`).
//! 2. **reach reads pick up the newly-minted grant** —
//!    `authz.scope_filter` for the same `(subject, cap, scope)` returns
//!    the grant (era-2 read delegation stays green).
//! 3. **revoke over the callback works** — the symmetric `grants.revoke`
//!    through the same client removes the grant and the read returns
//!    empty.
//!
//! The seed_admin / read-delegation / revoke paths in `matrix_era2.rs`
//! stay unchanged — this file is purely the WRITE-half regression that
//! activates the moment the lb routing fix lands.

use std::net::SocketAddr;
use std::sync::Arc;

use care::authz::{Chokepoint, ReachClient, ReachFilter};
use lb_auth::{mint, verify, Claims, Principal, Role, SigningKey};
use lb_ext_native::SidecarClient;
use lb_host::{grants_assign as host_grants_assign, grants_revoke as host_grants_revoke, Node, Role as NodeRole, Scope, Subject};
use lb_role_gateway::{router, Gateway};
use serde_json::json;

const NOW: u64 = 1000;
const WS: &str = "ws-a";
const REACH_CAP: &str = "mcp:care.reach.child:call";

fn admin_token(key: &SigningKey, ws: &str, sub: &str, caps: &[&str]) -> String {
    let claims = Claims {
        sub: format!("user:{sub}"),
        ws: ws.into(),
        role: Role::WorkspaceAdmin,
        caps: caps.iter().map(|s| s.to_string()).collect(),
        iat: NOW - 1,
        exp: NOW + 100_000,
        constraint: None,
        run_id: None,
    };
    mint(key, &claims)
}

fn guardian_token(key: &SigningKey, ws: &str, sub: &str, caps: &[&str]) -> String {
    let claims = Claims {
        sub: format!("user:{sub}"),
        ws: ws.into(),
        role: Role::Member,
        caps: caps.iter().map(|s| s.to_string()).collect(),
        iat: NOW - 1,
        exp: NOW + 100_000,
        constraint: None,
        run_id: None,
    };
    mint(key, &claims)
}

fn verify_p(key: &SigningKey, token: &str) -> Principal {
    verify(key, token, NOW).expect("verify")
}

async fn serve() -> (Arc<Node>, SigningKey, String) {
    let node = Arc::new(Node::boot_as(NodeRole::Hub).await.expect("node"));
    let key = SigningKey::generate();
    let gw = Gateway::new(node.clone(), key.clone(), NOW);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr: SocketAddr = listener.local_addr().unwrap();
    let app = router(gw);
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });
    (node, key, format!("http://{addr}"))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires the patched lb (see docs/debugging/authz/lb-grants-routing.patch) — \
            set CC_APPLY_LB_GRANTS_ROUTING_PATCH=1 after dropping the [patch] into \
            .cargo/config.toml (milestone 04 session doc)"]
async fn era2_write_grants_assign_over_callback_works() {
    let (node, key, base) = serve().await;

    // A native sidecar with the `grants.assign` cap. The patched lb
    // dispatches `grants.assign` to `call_authz_tool` (the upstream
    // fix); without the patch, this exact call returns `Denied`.
    let ext_token = admin_token(
        &key,
        WS,
        "care-ext",
        &["mcp:grants.assign:call", "mcp:authz.scope_filter:call", REACH_CAP],
    );
    let cfg = lb_ext_native::Config::new(base.clone(), ext_token, WS.to_string(), "care".to_string());
    let client = SidecarClient::with_config(cfg);

    // Mint a sam → leo scoped reach grant via the callback (the sidecar's
    // `authz::grant::derive_reach` does this in production on
    // `care.guardianship.link`).
    let assign_out = client
        .call_tool(
            "grants.assign",
            json!({
                "subject": "user:sam",
                "cap": REACH_CAP,
                "scope": { "kind": "ids", "table": "child", "ids": ["child:leo"] },
            }),
        )
        .await
        .expect("grants.assign over the callback MUST succeed when lb routes grants.* (the upstream fix)");
    assert_eq!(assign_out["ok"], true);

    // Read it back via the read-delegation path (already proven in
    // matrix_era2.rs; here we re-assert it composes with the write path).
    let sam_token = guardian_token(&key, WS, "sam", &["mcp:authz.scope_filter:call"]);
    let sam_cfg = lb_ext_native::Config::new(base.clone(), sam_token, WS.to_string(), "care".to_string());
    let reach = ReachClient::new(SidecarClient::with_config(sam_cfg));
    match reach.reachable().await.expect("scope_filter") {
        ReachFilter::Ids(ids) => assert!(
            ids.iter().any(|x| x == "child:leo"),
            "scope_filter sees the just-minted grant: {ids:?}"
        ),
        ReachFilter::All => panic!("guardian must never resolve to All"),
    }

    // Revoke the same grant via the callback (the sidecar's
    // `authz::grant::remove_reach` does this in production on
    // `care.guardianship.unlink`). Same shape as the mint.
    let rev_out = client
        .call_tool(
            "grants.revoke",
            json!({
                "subject": "user:sam",
                "cap": REACH_CAP,
                "scope": { "kind": "ids", "table": "child", "ids": ["child:leo"] },
            }),
        )
        .await
        .expect("grants.revoke over the callback MUST succeed");
    assert_eq!(rev_out["ok"], true);

    // The grant is physically gone.
    match reach.reachable().await.expect("scope_filter") {
        ReachFilter::Ids(ids) => assert!(
            ids.is_empty(),
            "scope_filter empty after revoke: {ids:?}"
        ),
        ReachFilter::All => panic!("guardian must never resolve to All"),
    }

    // Silence the unused-import lint when the host-path helpers aren't
    // called directly (we keep them imported because the surrounding
    // era-2 module re-exports them and they're useful for parity).
    let _ = (host_grants_assign, host_grants_revoke);
    let _: Subject = Subject::User("user:sam".into());
    let _: Scope = Scope::Ids { table: "child".into(), ids: vec!["child:leo".into()] };
    let _ = verify_p;
    let _: NodeRole = NodeRole::Hub;
}