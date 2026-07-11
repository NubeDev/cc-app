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
//! ## Scope of what this proves (and the tracked lb gap it does not)
//!
//! The **reach-resolution** half of era 2 (the read: `check_scoped` /
//! `scope_filter`) is LIVE and proven here. The scoped grants are seeded via
//! lb's REAL in-process grant write path (`lb_host::grants_assign` — the same
//! function `grants.assign` invokes), because the **grant-DERIVATION** half
//! (a native extension calling `grants.assign` / `grants.revoke` BACK over the
//! host-callback) is blocked by a platform gap: lb's `/mcp/call` dispatcher
//! routes only `authz.*` to the authz verbs, NOT `grants.*` — so a native
//! sidecar cannot mint a grant over the callback yet. That is an upstream lb
//! fix (route `grants.*`/`roles.*`/`teams.*` through the MCP dispatcher), NOT
//! a care-side workaround (rule 10). Tracked:
//! `docs/debugging/authz/grants-verbs-not-on-mcp-callback-surface.md`. The
//! care derivation code (`authz::grant`) is wired and correct against the
//! verb contract — it goes live the moment lb routes those verbs.
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
use lb_host::{grants_assign, grants_revoke, Node, Role as NodeRole, Scope, Subject};
use lb_role_gateway::{router, Gateway};

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

/// A `SidecarClient` authenticated as `sub` with the two reach-read caps,
/// pointed at `base` — the exact grant set a guardian's token carries.
fn reach_client(key: &SigningKey, base: &str, ws: &str, sub: &str) -> ReachClient {
    let cfg = lb_ext_native::Config::new(
        base,
        token(
            key,
            ws,
            sub,
            &["mcp:authz.check_scoped:call", "mcp:authz.scope_filter:call"],
        ),
        ws,
        "care",
    );
    ReachClient::new(SidecarClient::with_config(cfg))
}

/// An era-2 chokepoint whose reach resolves as `guardian_sub` through the
/// platform (the guardian's own token drives the callback). The store is the
/// node's real store (fallback), but the reach client makes era 2 the live path.
fn era2_cp(node: &Arc<Node>, key: &SigningKey, base: &str, ws: &str, guardian_sub: &str) -> Chokepoint {
    Chokepoint::with_host_callback(
        Arc::new(node.store.clone()),
        ws,
        reach_client(key, base, ws, guardian_sub),
    )
}

/// Seed a scoped reach grant via lb's REAL in-process grant write path (the
/// same `grants_assign` the `grants.assign` verb calls) — a genuine grant row
/// in the real store, not a mock. Stands in for the care sidecar's derivation
/// call, which is blocked by the tracked lb `grants.*`-callback gap.
async fn seed_reach_grant(node: &Node, key: &SigningKey, ws: &str, guardian_sub: &str, child_id: &str) {
    // A workspace-admin principal holding the reach cap (so anti-widen passes).
    let admin = admin_principal(key, ws);
    grants_assign(
        &node.store,
        &admin,
        ws,
        &Subject::User(guardian_sub.into()),
        REACH_CAP,
        &Scope::Ids {
            table: "child".into(),
            ids: vec![child_id.into()],
        },
    )
    .await
    .expect("seed scoped reach grant (real write path)");
}

/// Revoke the seeded scoped reach grant via the real in-process path (stands
/// in for the sidecar's `grants.revoke` derivation, same lb gap).
async fn revoke_reach_grant(node: &Node, key: &SigningKey, ws: &str, guardian_sub: &str, child_id: &str) {
    let admin = admin_principal(key, ws);
    grants_revoke(
        &node.store,
        &admin,
        ws,
        &Subject::User(guardian_sub.into()),
        REACH_CAP,
        &Scope::Ids {
            table: "child".into(),
            ids: vec![child_id.into()],
        },
    )
    .await
    .expect("revoke scoped reach grant (real write path)");
}

/// A workspace-admin principal that holds `grants.assign`/`revoke` AND the
/// reach cap (anti-widen: cannot grant a cap it lacks).
fn admin_principal(key: &SigningKey, ws: &str) -> Principal {
    let claims = Claims {
        sub: "user:care-ext".into(),
        ws: ws.into(),
        role: Role::WorkspaceAdmin,
        caps: vec![
            "mcp:grants.assign:call".into(),
            "mcp:grants.revoke:call".into(),
            REACH_CAP.into(),
        ],
        iat: NOW - 1,
        exp: NOW + 100_000,
        constraint: None,
        run_id: None,
    };
    verify(key, &mint(key, &claims), NOW).expect("admin verifies")
}

/// Verify a `Member` guardian principal (the chokepoint reads its role).
fn guardian(key: &SigningKey, sub: &str, ws: &str) -> Principal {
    verify(key, &token(key, ws, sub, &["mcp:care.child.get:call"]), NOW).expect("verifies")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn era2_grant_then_reach_and_revoke_removes_it() {
    let (node, key, base) = serve().await;

    // Seed the scoped reach grant for sam → leo (the derivation the care
    // sidecar performs on care.guardianship.link, via the real write path).
    seed_reach_grant(&node, &key, WS, "sam", "child:leo").await;

    // Sam's era-2 chokepoint resolves reach through the PLATFORM over HTTP.
    let sam_cp = era2_cp(&node, &key, &base, WS, "sam");
    let sam = guardian(&key, "sam", WS);

    assert_reach(&sam_cp, &sam, "child:leo")
        .await
        .expect("era-2 allow: sam reaches leo via the platform grant (over HTTP)");

    let reached = reachable_children(&sam_cp, &sam).await;
    assert_eq!(reached, vec!["child:leo".to_string()], "era-2 scope_filter set");

    // Cross-family deny: sam has no grant for mia → the platform denies.
    assert_reach(&sam_cp, &sam, "child:mia")
        .await
        .expect_err("era-2 deny: sam does not reach mia");

    // Revoke (the derivation care.guardianship.unlink performs).
    revoke_reach_grant(&node, &key, WS, "sam", "child:leo").await;

    // Reach now denies AND the grant is PHYSICALLY GONE — scope_filter returns
    // no ids (not merely that a read denied). A grant surviving unlink is the
    // existential cross-family leak; this asserts it cannot happen.
    assert_reach(&sam_cp, &sam, "child:leo")
        .await
        .expect_err("era-2 deny after revoke");

    let sam_reach = reach_client(&key, &base, WS, "sam");
    match sam_reach.reachable().await.expect("scope_filter ok") {
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
    seed_reach_grant(&node, &key, WS, "sam", "child:leo").await;

    // A guardian in ws-b (same sub name, different workspace) sees NONE of
    // ws-a's grants — the workspace is the token's, un-spoofable (the wall).
    let sam_b_cp = era2_cp(&node, &key, &base, WS_B, "sam");
    let sam_b = guardian(&key, "sam", WS_B);
    assert_reach(&sam_b_cp, &sam_b, "child:leo")
        .await
        .expect_err("ws-b sam does not reach ws-a's leo");
    assert!(
        reachable_children(&sam_b_cp, &sam_b).await.is_empty(),
        "ws isolation: no cross-workspace reach"
    );
}
