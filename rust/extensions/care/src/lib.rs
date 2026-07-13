//! care — the single backend extension owning the childcare domain.
//!
//! See `rust/extensions/care/README.md` for layout / scope / rules.
//! All read/write verbs must pass through `authz` (CLAUDE.md rule 7).
//!
//! ## Layout (one verb per file, FILE-LAYOUT)
//!
//! - `authz/` — the chokepoint (every verb calls `assert_reach` /
//!   `reachable_children` / `reachable_rooms`). Sacred — CLAUDE.md rule 7.
//! - `call.rs` — the `Tools` dispatcher (the child wire impl; dispatches on
//!   bare tool name, the host strips the `care.` prefix).
//! - `ping.rs` — `care.ping`, the first verb (the loop proof).
//!
//! ## Tier
//!
//! Native (Tier-2 sidecar), same shape as `NubeIO/rubix-ai`'s
//! `host-metrics` extension (the canonical "why native" reference). A real
//! host-platform binary the host spawns + supervises over stdio via the
//! published `lb-ext-native` SDK — no lb-repo access (CLAUDE.md rule 10).

pub mod authz;

// The extension's user-facing string catalog (CLAUDE.md rule 8). Every
// verb that emits a user-visible string resolves it via `i18n::t` — no
// raw English literals (enforced by scripts/check-hardcoded-strings.sh).
pub mod i18n;

// The `call` and `ping` modules are `pub` to integration tests under
// `tests/` so the matrix harness can exercise the dispatcher (the
// harness asserts the `Tools::tools()` handshake + the cap-deny half
// of the gate). They stay out of the public extension API — consumers
// see `serve_stdio(Care::new(ws))`, not the module internals.
pub mod call;
mod ping;

// The verb folders. Each follows the same shape (FILE-LAYOUT §2):
// `mod.rs` is a barrel, `records.rs` is the orchestrator-owned schema,
// each verb is its own file (`create.rs`, `get.rs`, `list.rs`, …).
// The cross-family matrix harness (tests/) exercises the chokepoint
// the same way for every noun. All milestone-03 nouns now ship.
pub mod attendance;
pub mod center;
pub mod child;
pub mod enrollment;
pub mod feed;
pub mod guardian;
pub mod guardianship;
pub mod invite;
pub mod log;
pub mod menu;
pub mod push;
pub mod room;

// re-exported by the binary; left out of the lib so the only platform
// dependency callers see is the child-wire SDK name.

use std::sync::Arc;

use lb_auth::Principal;
use lb_ext_native::Caller;

use crate::authz::{Chokepoint, ReachClient};

/// The per-sidecar runtime — one `Care` instance lives for the whole
/// supervisor-managed lifetime. Holds the workspace handle the host
/// stamped at spawn (`LB_EXT_WS`), the per-call principal resolver, and
/// the [`Chokepoint`] that every verb body reaches reach decisions
/// through (era-2 read delegation when the platform routes `authz.*`
/// over `/mcp/call`; era-1 store-resolution fallback otherwise).
///
/// Verb bodies never reach into env directly — they ask the impl for
/// the chokepoint (CLAUDE.md rule 5: env is a binary concern, not a
/// verb concern). The principal the verb sees comes from the native call
/// FRAME: since `sdk-v0.4.0` the host stamps the authorized caller onto
/// every `call` (`CallParams.caller`, native-caller-identity scope), and
/// [`Care::call_with_caller`] projects it into a [`Principal`] per
/// dispatch (see [`principal_from_caller`]). When the frame carries no
/// caller — the era-1 in-process test path (`Care::new`, driven without a
/// host frame) — it degrades to a synthetic WorkspaceAdmin with full caps.
#[derive(Clone)]
pub struct Care {
    /// The workspace the host spawned this sidecar into. Stamped once at
    /// start from `LB_EXT_WS`; every store call is workspace-scoped.
    pub ws: String,
    /// The authz chokepoint — the one surface every verb reaches reach
    /// decisions through (CLAUDE.md rule 7, sacred). The `store` half is
    /// required for era 1; the `reach` half is `Some` whenever the host
    /// can route `authz.*` over the callback (i.e. always, since
    /// `node-v0.3.0`). Constructed once at `Care::boot` and shared across
    /// every call.
    chokepoint: Chokepoint,
}

impl Care {
    /// Build a `Care` impl for the workspace the host spawned us in.
    ///
    /// The minimal constructor — chokepoint + principal default to the
    /// era-1 store-only path and a synthetic WorkspaceAdmin so a unit
    /// test (or a hand-driven integration test) can drive the
    /// dispatcher without spinning the host callback. Real production
    /// binaries use [`Care::boot`] (built from env at sidecar start).
    pub fn new(ws: String) -> Self {
        // A standalone store-less chokepoint is only useful for tests; we
        // surface a `None` reach (era-1 fallback only). The boot path
        // constructs the real store + reach.
        let cp = Chokepoint::new(Arc::new(test_store_fallback()), ws.clone());
        Self { ws, chokepoint: cp }
    }

    /// The boot constructor — what a real sidecar uses. Reads the
    /// supervisor-injected identity (`LB_EXT_TOKEN` / `LB_GATEWAY_URL` /
    /// `LB_EXT_WS` / `LB_EXT_ID`) from the env.
    ///
    /// Constructs the era-2 chokepoint (the LIVE production path) when
    /// BOTH `LB_EXT_TOKEN` AND `LB_GATEWAY_URL` are present (the host-callback
    /// gate is wired). When either is absent — the boot-time integration test
    /// runs without the host, and the documented era-1 fallback path covers
    /// the "lb's authz verbs aren't reachable" case — falls back to the
    /// era-1 store-resolved chokepoint so the same binary boots in either
    /// posture (the `care-authz-scope.md` §"Era 2" fallback contract).
    ///
    /// `LB_EXT_WS` is required in BOTH postures (every store call is
    /// workspace-scoped; the wall is the workspace). `LB_EXT_ID` is a
    /// friendly tag for the host's boot logs; absent is fine.
    pub async fn boot(env: &std::collections::HashMap<String, String>) -> Result<Self, String> {
        let ws = env
            .get("LB_EXT_WS")
            .cloned()
            .ok_or_else(|| "missing LB_EXT_WS in env".to_string())?;

        // Era-2 when the host-callback gate is wired (token + gateway
        // both present); era-1 store-only fallback otherwise.
        let era2 = env.get("LB_EXT_TOKEN").is_some() && env.get("LB_GATEWAY_URL").is_some();

        let store = open_store_from_env(env).await?;
        let cp = if era2 {
            let token = env.get("LB_EXT_TOKEN").cloned().unwrap();
            let gateway = env.get("LB_GATEWAY_URL").cloned().unwrap();
            let ext_id = env.get("LB_EXT_ID").cloned().unwrap_or_default();
            let cfg = lb_ext_native::Config::new(gateway, token, ws.clone(), ext_id);
            let client = lb_ext_native::SidecarClient::with_config(cfg);
            let reach = ReachClient::new(client);
            Chokepoint::with_host_callback(store, ws.clone(), reach)
        } else {
            Chokepoint::new(store, ws.clone())
        };
        Ok(Self { ws, chokepoint: cp })
    }

    /// The chokepoint every verb body calls — one surface, two eras
    /// (era-1 fallback when `reach` is `None`; era-2 read delegation when
    /// it is `Some`). Verb bodies must NEVER construct their own.
    pub fn chokepoint(&self) -> &Chokepoint {
        &self.chokepoint
    }

    /// The principal for the current dispatch, projected from the native
    /// call frame's [`Caller`] (`sdk-v0.4.0`, native-caller-identity scope).
    /// The host authorized this caller at the `mcp:<tool>:call` wall BEFORE
    /// the sidecar saw the call, workspace-first; this projection is a read
    /// of that already-authorized principal, so the chokepoint's role check
    /// and its `subject`-parameterized reach are ABOUT the real caller.
    ///
    /// `None` — an old-host frame or a call with no resolvable caller — is
    /// the era-1 in-process test path only (`Care::new`, driven without a
    /// host); it degrades to the synthetic WorkspaceAdmin so those tests keep
    /// booting. A real spawned sidecar on `node-v0.4.0`+ always carries a
    /// caller, so a guardian dispatch never falls into the admin branch.
    pub fn principal_for_caller(&self, caller: Option<Caller>) -> Principal {
        match caller {
            Some(c) => principal_from_caller(&c),
            None => synthetic_admin(),
        }
    }
}

/// Project a frame [`Caller`] into a typed [`lb_auth::Principal`] for the
/// chokepoint. The caller is NOT a replayable token (the frame projection
/// carries no gateway-accepted signature — native-caller-identity scope's
/// non-goal): we mint+verify locally with a throwaway key purely to obtain
/// a well-formed `Principal` carrying the caller's `sub` / `ws` and its
/// AUTHORITY. The host wall is the authoritative gate; this projection only
/// lets the chokepoint attribute its row-filter decision to the caller and
/// name `caller.sub` as the delegated-reach `subject`.
///
/// **Authority comes from `caller.admin`, NOT `caller.role`.** lb's role
/// enum is cosmetic — the gateway mints EVERY session as `member`, so a real
/// admin and a guardian are indistinguishable by role. The host derives the
/// authoritative `admin` marker from the caller's caps (native-caller-identity
/// scope, `sdk-v0.4.1`/`node-v0.4.1`); we map `admin ⇒ WorkspaceAdmin` so the
/// chokepoint's audited admin-pass fires for real admins and NEVER for a
/// guardian. `caps` stay empty — the chokepoint reads role + sub, and the
/// subject's reach is resolved server-side behind the delegation cap.
fn principal_from_caller(caller: &Caller) -> Principal {
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    let key = SigningKey::generate();
    // Fail-closed: default to the least-privileged Member; only the host-derived
    // `admin` marker (caps-based) grants the admin role that unlocks the
    // chokepoint's bypass. The cosmetic `caller.role` is intentionally ignored.
    let role = if caller.admin {
        Role::WorkspaceAdmin
    } else {
        Role::Member
    };
    let claims = Claims {
        sub: caller.sub.clone(),
        ws: caller.ws.clone(),
        role,
        caps: Vec::new(),
        iat: 0,
        exp: u64::MAX,
        constraint: None,
        run_id: None,
    };
    verify(&key, &mint(&key, &claims), 1).expect("frame-caller projection verifies")
}

/// Open the store the chokepoint reads era-1 records from. Today: only
/// `mem://` is wired (the boot-time integration test + the matrix
/// harness); a real `file://` path would go through lb-store's
/// persistent backend (`Store::open(path)`) — which lands alongside the
/// durable on-disk deployment in a later slice (milestone 05+).
async fn open_store_from_env(
    env: &std::collections::HashMap<String, String>,
) -> Result<Arc<lb_store::Store>, String> {
    let url = env
        .get("LB_EXT_STORE_URL")
        .map(|s| s.as_str())
        .unwrap_or("mem://");
    // `mem://` is the integration-test / boot-test path (in-process
    // SurrealDB, dropped on handle drop). A real path goes through
    // `Store::open`, which opens a durable SurrealKV backend. We
    // branch on the prefix so a misconfigured url never silently
    // becomes an in-memory store.
    let store = if url == "mem://" || url.is_empty() {
        lb_store::Store::memory()
            .await
            .map_err(|e| format!("could not open mem store: {e}"))?
    } else {
        lb_store::Store::open(url)
            .await
            .map_err(|e| format!("could not open store {url}: {e}"))?
    };
    Ok(Arc::new(store))
}

/// A placeholder store used by `Care::new` (the no-callback constructor
/// used by unit tests). A real sidecar is built via `Care::boot`, which
/// resolves the store from `LB_EXT_STORE_URL`.
fn test_store_fallback() -> lb_store::Store {
    // `lb_store::Store` is the type; we construct a memory store for
    // tests. Synchronously unwrap — mem:// is infallible.
    futures::executor::block_on(lb_store::Store::memory()).expect("mem:// store must open")
}

/// A synthetic WorkspaceAdmin — the fallback principal when the host
/// has not stamped a per-call one (the integration test boot path /
/// `Care::new` unit tests). Wall caps are listed so a deny-test that
/// relies on missing caps sees the expected deny (the cap wall is
/// authoritative; this is a defence-in-depth fake).
fn synthetic_admin() -> Principal {
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    let key = SigningKey::generate();
    let claims = Claims {
        sub: "user:care-ext".into(),
        ws: String::new(),
        role: Role::WorkspaceAdmin,
        caps: vec![
            "mcp:care.center.create:call".into(),
            "mcp:care.center.get:call".into(),
            "mcp:care.center.list:call".into(),
            "mcp:care.room.create:call".into(),
            "mcp:care.room.get:call".into(),
            "mcp:care.room.list:call".into(),
            "mcp:care.child.create:call".into(),
            "mcp:care.child.get:call".into(),
            "mcp:care.child.list:call".into(),
            "mcp:care.child.update:call".into(),
            "mcp:care.child.archive:call".into(),
            "mcp:care.guardian.create:call".into(),
            "mcp:care.guardian.get:call".into(),
            "mcp:care.guardian.list:call".into(),
            "mcp:care.guardianship.link:call".into(),
            "mcp:care.guardianship.unlink:call".into(),
            "mcp:care.guardianship.update:call".into(),
            "mcp:care.enrollment.create:call".into(),
            "mcp:care.enrollment.list:call".into(),
            "mcp:care.enrollment.update:call".into(),
            "mcp:care.invite.create_guardian:call".into(),
            "mcp:care.invite.create_staff:call".into(),
            "mcp:care.invite.list:call".into(),
            "mcp:care.invite.resend:call".into(),
            "mcp:care.invite.revoke:call".into(),
        ],
        iat: 0,
        exp: u64::MAX,
        constraint: None,
        run_id: None,
    };
    verify(&key, &mint(&key, &claims), 1).expect("synthetic admin verifies")
}
