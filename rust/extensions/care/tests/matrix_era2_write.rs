//! Era-2 WRITE path — the regression that proves the lb `grants.*` routing
//! fix is live and end to end (no mocks — CLAUDE.md rule 4).
//!
//! `node-v0.3.2` shipped the upstream additive fix that routes
//! `grants.*` / `roles.*` / `teams.*` through the MCP dispatcher (the
//! patch in `docs/debugging/authz/lb-grants-routing.patch`). A native
//! (Tier-2) extension can now mint scoped grants over the callback by
//! calling `SidecarClient::call_tool("grants.assign", …)` — the SAME
//! client that reads them back via `authz.scope_filter`.
//! `node-v0.3.3` carried the fix to the cc-app pin (the unpinned
//! version of this test ran with the patch + a temporary `[patch]`
//! override; the pin is now `node-v0.3.3`).
//!
//! ## What it asserts
//!
//! 1. **grant mint over the callback works** — a `SidecarClient` whose
//!    token holds `mcp:grants.assign:call` succeeds against a real booted
//!    gateway (now `node-v0.3.3`'s `/mcp/call`).
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

use care::authz::{assert_reach, reachable_children, Chokepoint, ReachClient, ReachFilter};
use lb_auth::{mint, verify, Claims, Principal, Role, SigningKey};
use lb_ext_native::SidecarClient;
use lb_host::{Node, Role as NodeRole};
use lb_role_gateway::{router, Gateway};
use serde_json::json;

const NOW: u64 = 1000;
const WS: &str = "ws-a";
const REACH_CAP: &str = "mcp:care.reach.child:call";

/// Verify a `Member` guardian principal (the chokepoint reads its role).
fn guardian(key: &SigningKey, sub: &str, ws: &str) -> Principal {
    let claims = Claims {
        sub: format!("user:{sub}"),
        ws: ws.into(),
        role: Role::Member,
        caps: vec!["mcp:care.child.get:call".into()],
        iat: NOW - 1,
        exp: NOW + 100_000,
        constraint: None,
        run_id: None,
    };
    verify(key, &mint(key, &claims), NOW).expect("verifies")
}

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

/// The care EXTENSION's callback token for the reach-read path: holds the two
/// reach verbs' caps PLUS the delegation marker (`mcp:authz.delegate_reach:call`)
/// so it may name a guardian `subject` on `authz.check_scoped`/`scope_filter`
/// (native-caller-identity scope, node-v0.4.0). This is the exact callback token
/// a real spawned care sidecar carries; the chokepoint supplies the subject.
fn ext_reach_token(key: &SigningKey, ws: &str) -> String {
    admin_token(
        key,
        ws,
        "care-ext",
        &[
            "mcp:authz.check_scoped:call",
            "mcp:authz.scope_filter:call",
            "mcp:authz.delegate_reach:call",
        ],
    )
}

async fn serve() -> (Arc<Node>, SigningKey, String) {
    let node = Arc::new(Node::boot_as(NodeRole::Hub).await.expect("node"));
    let key = SigningKey::generate();
    let gw = Gateway::new(node.clone(), key.clone(), NOW);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr: SocketAddr = listener.local_addr().unwrap();
    let app = router(gw);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (node, key, format!("http://{addr}"))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn era2_write_grants_assign_over_callback_works() {
    let (_node, key, base) = serve().await;

    // A native sidecar with the `grants.assign` cap. With `node-v0.3.2+`,
    // lb dispatches `grants.assign` to `call_authz_tool`; on earlier tags
    // this exact call returned `Denied` (see
    // docs/debugging/authz/grants-verbs-not-on-mcp-callback-surface.md).
    //
    // The `mcp:grants.assign:call` cap aliases `mcp:grants.revoke:call`
    // (assign/revoke share the gate) per lb's cap-alias idiom — that is
    // the documented pair care uses; the reach cap is added so the
    // anti-widen check passes for `derive_reach`'s scope.
    let ext_token = admin_token(
        &key,
        WS,
        "care-ext",
        &[
            "mcp:grants.assign:call",
            "mcp:authz.scope_filter:call",
            REACH_CAP,
        ],
    );
    let cfg =
        lb_ext_native::Config::new(base.clone(), ext_token, WS.to_string(), "care".to_string());
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
        .expect("grants.assign over the callback MUST succeed when lb routes grants.*");
    assert_eq!(assign_out["ok"], true);

    // Read it back via the read-delegation path (already proven in
    // matrix_era2.rs; here we re-assert it composes with the write path). The
    // reach client is the EXTENSION's callback token (holds the delegation cap,
    // native-caller-identity scope) naming `user:sam` as the subject — exactly
    // what the chokepoint does in production.
    let ext_reach_token = admin_token(
        &key,
        WS,
        "care-ext",
        &[
            "mcp:authz.scope_filter:call",
            "mcp:authz.delegate_reach:call",
        ],
    );
    let sam_cfg = lb_ext_native::Config::new(
        base.clone(),
        ext_reach_token,
        WS.to_string(),
        "care".to_string(),
    );
    let reach = ReachClient::new(SidecarClient::with_config(sam_cfg));
    match reach.reachable("user:sam").await.expect("scope_filter") {
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
    match reach.reachable("user:sam").await.expect("scope_filter") {
        ReachFilter::Ids(ids) => {
            assert!(ids.is_empty(), "scope_filter empty after revoke: {ids:?}")
        }
        ReachFilter::All => panic!("guardian must never resolve to All"),
    }

    // The host-side `Node` is consumed by `serve()` to build the gateway;
    // it stays bound here so the gateway process isn't dropped mid-test.
    let _: NodeRole = NodeRole::Hub;
}

/// Cross-family deny over the LIVE callback (CLAUDE.md rule 7 — the
/// sacred invariant). A guardian linked to `child:leo` via the live
/// `grants.assign` callback, queried with an era-2 chokepoint against
/// `child:mia` (a child in ANOTHER family), MUST be denied — the grant
/// isn't there, `scope_filter` returns just `child:leo`, `assert_reach`
/// denies `child:mia`. This is the existential bug the chokepoint
/// exists to prevent (a cross-family leak is the worst kind of bug
/// this product can ship).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn era2_cross_family_deny_over_live_callback() {
    let (node, key, base) = serve().await;

    // Mint sam → child:leo over the live callback (`derive_reach` in prod).
    let ext_token = admin_token(&key, WS, "care-ext", &["mcp:grants.assign:call", REACH_CAP]);
    let ext_cfg =
        lb_ext_native::Config::new(base.clone(), ext_token, WS.to_string(), "care".to_string());
    let ext_client = SidecarClient::with_config(ext_cfg);
    let out = ext_client
        .call_tool(
            "grants.assign",
            json!({
                "subject": "user:sam",
                "cap": REACH_CAP,
                "scope": { "kind": "ids", "table": "child", "ids": ["child:leo"] },
            }),
        )
        .await
        .expect("grants.assign over the live callback");
    assert_eq!(out["ok"], true);

    // The era-2 chokepoint: resolve reach through the platform over HTTP. The
    // reach client is the CARE EXTENSION's callback token holding the delegation
    // cap (native-caller-identity scope); the chokepoint names the guardian
    // (`principal.sub()` = `user:sam`) as the `subject`, so the platform
    // resolves SAM's grants — exactly the production shape.
    let ext_token = ext_reach_token(&key, WS);
    let ext_cfg =
        lb_ext_native::Config::new(base.clone(), ext_token, WS.to_string(), "care".to_string());
    let sam_cp = Chokepoint::with_host_callback(
        Arc::new(node.store.clone()),
        WS,
        ReachClient::new(SidecarClient::with_config(ext_cfg)),
    );
    let sam = guardian(&key, "sam", WS);

    // Allowed for the child the grant covers.
    assert_reach(&sam_cp, &sam, "child:leo")
        .await
        .expect("allow: sam reaches leo via the platform grant");

    // Cross-family deny: sam has no grant for mia. The chokepoint
    // delegates to `authz.check_scoped`, the platform returns `false`,
    // and the chokepoint surfaces `Denied` — never a silent success,
    // never a leak. THIS is the rule-7 invariant.
    assert_reach(&sam_cp, &sam, "child:mia")
        .await
        .expect_err("cross-family deny: sam does NOT reach mia");

    // And `reachable_children` returns exactly the granted set
    // (no `mia`, no wildcard `*` — a guardian must never resolve to All).
    let reached = reachable_children(&sam_cp, &sam).await;
    assert_eq!(
        reached,
        vec!["child:leo".to_string()],
        "era-2 scope_filter scopes exactly to the granted child"
    );
}

/// Guardian's FIRST sign-in lands seeing ONLY the child they hold a
/// live edge to (the invite → accept → first-read boundary). A
/// freshly-bound guardian — one whose scope-filter is empty BEFORE
/// `guardianship.link` runs (the scariest moment for a leak: a
/// guardian with a freshly-bound `sub` and no grants yet), queried
/// with an era-2 chokepoint against ANY child in the workspace, MUST
/// be denied. The grant derives on `guardianship.link`'s
/// `derive_reach` callback; until that callback lands, the platform
/// has nothing to read back, so the chokepoint denies.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn era2_first_sign_in_deny_over_live_callback() {
    let (node, key, base) = serve().await;

    // The era-2 chokepoint: the caller is sam, no grants yet (the "first
    // sign-in" posture — the scariest leak window). The reach client is the
    // extension's delegation-cap token; the chokepoint names `user:sam` as the
    // subject, and the platform has nothing to read back → deny.
    let ext_token = ext_reach_token(&key, WS);
    let ext_cfg =
        lb_ext_native::Config::new(base.clone(), ext_token, WS.to_string(), "care".to_string());
    let sam_cp = Chokepoint::with_host_callback(
        Arc::new(node.store.clone()),
        WS,
        ReachClient::new(SidecarClient::with_config(ext_cfg)),
    );
    let sam = guardian(&key, "sam", WS);

    // Reach set is empty (no grants yet).
    let reached = reachable_children(&sam_cp, &sam).await;
    assert!(
        reached.is_empty(),
        "no grants ⇒ reach set is empty, got {reached:?}"
    );

    // Deny on EVERY child in the workspace — no grants ⇒ every child
    // is unreachable. This is the first-read boundary the invite
    // golden path tests: a guardian lands on the app, sees only the
    // children they hold a live edge to.
    assert_reach(&sam_cp, &sam, "child:leo")
        .await
        .expect_err("no grants ⇒ deny leo");
    assert_reach(&sam_cp, &sam, "child:mia")
        .await
        .expect_err("no grants ⇒ deny mia");
}
