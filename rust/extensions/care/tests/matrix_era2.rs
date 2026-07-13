//! The era-2 matrix — the chokepoint's PLATFORM-ENFORCED reach RESOLUTION,
//! end to end over REAL HTTP (no mocks; CLAUDE.md rule 4 / `care-authz-scope.md`).
//!
//! Era 1 resolves reach from `guardianship` records; era 2 delegates to lb's
//! entity-scoped grants (`authz.check_scoped` / `authz.scope_filter`) via the
//! native host-callback (`SidecarClient`, `node-v0.3.0`). This test proves the
//! LIVE READ path: a real `Node` + a real `lb-role-gateway` on a real TCP
//! port, the real `SidecarClient` making real callbacks, the real MCP cap
//! gate. It exercises exactly the surface the chokepoint's `assert_reach` /
//! `reachable_children` use in production.
//!
//! ## Scope of what this proves (and the lb gap it does NOT work around)
//!
//! The **reach-resolution** half of era 2 (the read: `check_scoped` /
//! `scope_filter`) is LIVE and proven here. As of `node-v0.3.2` the
//! `grants.*` / `roles.*` / `teams.*` verbs route through lb's MCP
//! dispatcher — so a native extension can mint a scoped grant over the
//! callback the same way production does (`care-authz-scope.md` §"Era 2").
//! `seed_reach_grant` / `revoke_reach_grant` here go through the
//! `SidecarClient::call_tool("grants.assign" | "grants.revoke")`
//! callback the live chokepoint uses; no in-process fallback, no force-
//! green — see `docs/debugging/authz/grants-verbs-not-on-mcp-callback-surface.md`
//! for the closed entry.
//!
//! What this asserts:
//!   - **grant → reach:** a seeded scoped grant makes a guardian's
//!     `assert_reach` / `reachable_children` (era 2, over HTTP) allow exactly
//!     the granted child.
//!   - **cross-family deny:** a guardian with no grant for a child is denied
//!     by the platform (never a leak).
//!   - **revoke → grant actually gone:** after the scoped grant is revoked,
//!     `assert_reach` denies AND `scope_filter` returns no ids — the grant is
//!     physically gone, not merely that a read denied (a grant surviving the
//!     edge is the existential leak).
//!   - **workspace isolation:** a ws-B guardian sees none of ws-A's grants.

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
const WS_B: &str = "ws-b";

/// The reach cap the chokepoint's scoped grants key on (orchestrator-owned;
/// mirrors `care::authz::caps::REACH_CAP`).
const REACH_CAP: &str = "mcp:care.reach.child:call";

/// Mint a Member token the gateway verifies (signed with the node key),
/// carrying `caps` for `sub` in `ws`. Wire sub form is `user:sub` (the
/// chokepoint's grants key on the bare user).
fn token(key: &SigningKey, ws: &str, sub: &str, caps: &[&str]) -> String {
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

/// Boot a real node + real gateway on a real ephemeral port.
async fn serve() -> (Arc<Node>, SigningKey, String) {
    let node = Arc::new(Node::boot_as(NodeRole::Hub).await.expect("node boots"));
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

/// A `SidecarClient` authenticated as the CARE EXTENSION (not a guardian),
/// carrying the reach-read caps PLUS the delegation cap — the exact grant set
/// a real spawned care sidecar's callback token holds (native-caller-identity
/// scope, node-v0.4.0). The chokepoint names the guardian as `subject` on the
/// reach verbs; the host resolves THAT subject's grants behind the delegation
/// cap. One client serves every guardian (the subject flows from the principal
/// at the `assert_reach` call site, not from this token).
fn reach_client(key: &SigningKey, base: &str, ws: &str) -> ReachClient {
    let cfg = lb_ext_native::Config::new(
        base,
        token(
            key,
            ws,
            "care-ext",
            &[
                "mcp:authz.check_scoped:call",
                "mcp:authz.scope_filter:call",
                "mcp:authz.delegate_reach:call",
            ],
        ),
        ws,
        "care",
    );
    ReachClient::new(SidecarClient::with_config(cfg))
}

/// A `SidecarClient` carrying the grant-write caps an extension needs to
/// mint scoped reach grants through the host-callback (the live `grants.*`
/// path the care sidecar's `authz::grant::derive_reach` uses). The
/// `mcp:grants.assign:call` cap aliases `mcp:grants.revoke:call` (assign/
/// revoke share the gate) per lb's cap-alias idiom — both verbs resolve
/// through the same alias at the wall; the reach cap is added so the
/// anti-widen check passes for `derive_reach`'s scope.
fn grants_client(key: &SigningKey, base: &str, ws: &str) -> SidecarClient {
    let cfg = lb_ext_native::Config::new(
        base,
        token(
            key,
            ws,
            "care-ext",
            &[
                "mcp:grants.assign:call",
                "mcp:authz.scope_filter:call",
                REACH_CAP,
            ],
        ),
        ws,
        "care",
    );
    SidecarClient::with_config(cfg)
}

/// An era-2 chokepoint that resolves reach through the platform. The reach
/// client is the CARE EXTENSION's identity (holds the delegation cap); WHOSE
/// reach a call resolves is decided by the `principal` passed to
/// `assert_reach`/`reachable_children` (the chokepoint names `principal.sub()`
/// as the `subject`), matching production (native-caller-identity scope). The
/// store is the node's real store (era-1 fallback), unused on the live path.
fn era2_cp(node: &Arc<Node>, key: &SigningKey, base: &str, ws: &str) -> Chokepoint {
    Chokepoint::with_host_callback(
        Arc::new(node.store.clone()),
        ws,
        reach_client(key, base, ws),
    )
}

/// Seed a scoped reach grant over the host-callback the way production does
/// (`care.guardianship.link` → `authz::grant::derive_reach` →
/// `SidecarClient::call_tool("grants.assign", …)`). With `node-v0.3.2+`,
/// lb routes `grants.*` through the MCP dispatcher; before that tag, this
/// returned `Denied` and the test seed used the in-process `lb_host::grants_assign`
/// fallback (closed).
async fn seed_reach_grant(
    key: &SigningKey,
    base: &str,
    ws: &str,
    guardian_sub: &str,
    child_id: &str,
) {
    let client = grants_client(key, base, ws);
    let out = client
        .call_tool(
            "grants.assign",
            json!({
                "subject": guardian_sub,
                "cap": REACH_CAP,
                "scope": { "kind": "ids", "table": "child", "ids": [child_id] },
            }),
        )
        .await
        .expect("grants.assign over the callback (the live derivation path)");
    assert_eq!(out["ok"], true);
}

/// Revoke the seeded scoped reach grant over the host-callback the way
/// production does (`care.guardianship.unlink` →
/// `authz::grant::remove_reach` → `SidecarClient::call_tool("grants.revoke", …)`).
async fn revoke_reach_grant(
    key: &SigningKey,
    base: &str,
    ws: &str,
    guardian_sub: &str,
    child_id: &str,
) {
    let client = grants_client(key, base, ws);
    let out = client
        .call_tool(
            "grants.revoke",
            json!({
                "subject": guardian_sub,
                "cap": REACH_CAP,
                "scope": { "kind": "ids", "table": "child", "ids": [child_id] },
            }),
        )
        .await
        .expect("grants.revoke over the callback (the live derivation path)");
    assert_eq!(out["ok"], true);
}

/// Verify a `Member` guardian principal (the chokepoint reads its role).
fn guardian(key: &SigningKey, sub: &str, ws: &str) -> Principal {
    verify(key, &token(key, ws, sub, &["mcp:care.child.get:call"]), NOW).expect("verifies")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn era2_grant_then_reach_and_revoke_removes_it() {
    let (node, key, base) = serve().await;

    // Seed the scoped reach grant for sam → leo (the derivation the care
    // sidecar performs on care.guardianship.link, via the live
    // grants.assign callback).
    seed_reach_grant(&key, &base, WS, "user:sam", "child:leo").await;

    // The era-2 chokepoint resolves reach through the PLATFORM over HTTP; the
    // subject flows from the principal (`sam`) at each call site.
    let sam_cp = era2_cp(&node, &key, &base, WS);
    let sam = guardian(&key, "sam", WS);

    assert_reach(&sam_cp, &sam, "child:leo")
        .await
        .expect("era-2 allow: sam reaches leo via the platform grant (over HTTP)");

    let reached = reachable_children(&sam_cp, &sam).await;
    assert_eq!(
        reached,
        vec!["child:leo".to_string()],
        "era-2 scope_filter set"
    );

    // Cross-family deny: sam has no grant for mia → the platform denies.
    assert_reach(&sam_cp, &sam, "child:mia")
        .await
        .expect_err("era-2 deny: sam does not reach mia");

    // Revoke (the derivation care.guardianship.unlink performs).
    revoke_reach_grant(&key, &base, WS, "user:sam", "child:leo").await;

    // Reach now denies AND the grant is PHYSICALLY GONE — scope_filter returns
    // no ids (not merely that a read denied). A grant surviving unlink is the
    // existential cross-family leak; this asserts it cannot happen.
    assert_reach(&sam_cp, &sam, "child:leo")
        .await
        .expect_err("era-2 deny after revoke");

    let sam_reach = reach_client(&key, &base, WS);
    match sam_reach
        .reachable("user:sam")
        .await
        .expect("scope_filter ok")
    {
        ReachFilter::Ids(ids) => assert!(
            ids.is_empty(),
            "grant physically gone: scope_filter returns no ids after revoke, got {ids:?}"
        ),
        ReachFilter::All => panic!("a guardian must never resolve to All"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn era2_workspace_isolation() {
    let (node, key, base) = serve().await;

    // Grant sam → leo in ws-a only.
    seed_reach_grant(&key, &base, WS, "user:sam", "child:leo").await;

    // A guardian in ws-b (same sub name, different workspace) sees NONE of
    // ws-a's grants — the workspace is the token's, un-spoofable (the wall).
    let sam_b_cp = era2_cp(&node, &key, &base, WS_B);
    let sam_b = guardian(&key, "sam", WS_B);
    assert_reach(&sam_b_cp, &sam_b, "child:leo")
        .await
        .expect_err("ws-b sam does not reach ws-a's leo");
    assert!(
        reachable_children(&sam_b_cp, &sam_b).await.is_empty(),
        "ws isolation: no cross-workspace reach"
    );
}
