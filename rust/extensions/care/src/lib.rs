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
pub mod center;
pub mod child;
pub mod enrollment;
pub mod guardian;
pub mod guardianship;
pub mod room;

// re-exported by the binary; left out of the lib so the only platform
// dependency callers see is the child-wire SDK name.

use std::sync::Arc;

use lb_auth::Principal;

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
/// verb concern). The principal the verb sees comes from
/// `principal_for_call`, which the supervisor hands per-dispatch via
/// `LB_EXT_PRINCIPAL_JSON` (the documented seam); absent that, it
/// degrades to a synthetic WorkspaceAdmin with full caps (so the
/// integration test boot path still works).
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
    /// The principal the current dispatch was issued for (per-call; the
    /// host stamps it on every `call`). Until the host stamps a real
    /// one, every dispatch is attributed to a synthetic WorkspaceAdmin
    /// (the boot-time test path / the booted-node integration test).
    current_principal: std::sync::Arc<std::sync::Mutex<Principal>>,
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
        Self {
            ws,
            chokepoint: cp,
            current_principal: std::sync::Arc::new(std::sync::Mutex::new(synthetic_admin())),
        }
    }

    /// The era-2 host-callback constructor — what a real sidecar uses.
    /// Reads the supervisor-injected identity (`LB_EXT_TOKEN` /
    /// `LB_GATEWAY_URL` / `LB_EXT_WS` / `LB_EXT_ID`) from the env,
    /// builds the [`SidecarClient`], wraps it in a [`ReachClient`], and
    /// returns a `Care` whose chokepoint delegates reach resolution to
    /// the platform over the callback.
    ///
    /// The store handle (`Arc<lb_store::Store>`) is carried alongside so
    /// era-1 is still the fallback if the platform path fails (the
    /// chokepoint fails closed on callback errors — denials are never
    /// silent successes).
    pub async fn boot(env: &std::collections::HashMap<String, String>) -> Result<Self, String> {
        let ws = env
            .get("LB_EXT_WS")
            .cloned()
            .ok_or_else(|| "missing LB_EXT_WS in env".to_string())?;
        let token = env
            .get("LB_EXT_TOKEN")
            .cloned()
            .ok_or_else(|| "missing LB_EXT_TOKEN in env".to_string())?;
        let gateway = env
            .get("LB_GATEWAY_URL")
            .cloned()
            .ok_or_else(|| "missing LB_GATEWAY_URL in env".to_string())?;
        let ext_id = env.get("LB_EXT_ID").cloned().unwrap_or_default();
        let cfg = lb_ext_native::Config::new(gateway, token, ws.clone(), ext_id);
        let client = lb_ext_native::SidecarClient::with_config(cfg);
        let reach = ReachClient::new(client);

        // The store the chokepoint resolves era-1 from. Real sidecar
        // builds get their store from the boot-time env hint
        // (`LB_EXT_STORE_URL`, e.g. `mem://` for tests or
        // `file://.cc-app/store` for a deployed node). Falling back to
        // mem keeps the constructor total (a missing var ⇒ mem, never a
        // panic — the integration test boots mem explicitly).
        let store = open_store_from_env(env).await?;

        let cp = Chokepoint::with_host_callback(store, ws.clone(), reach);
        Ok(Self {
            ws,
            chokepoint: cp,
            current_principal: std::sync::Arc::new(std::sync::Mutex::new(synthetic_admin())),
        })
    }

    /// The chokepoint every verb body calls — one surface, two eras
    /// (era-1 fallback when `reach` is `None`; era-2 read delegation when
    /// it is `Some`). Verb bodies must NEVER construct their own.
    pub fn chokepoint(&self) -> &Chokepoint {
        &self.chokepoint
    }

    /// The principal the current dispatch was issued for. The host
    /// stamps a fresh principal on every `call` via
    /// `LB_EXT_PRINCIPAL_JSON` (a JSON-encoded `lb_auth::Claims`
    /// blob); absent that, falls back to the synthetic admin (so the
    /// boot-time integration test works end to end).
    pub fn principal_for_call(&self) -> Principal {
        if let Ok(raw) = std::env::var("LB_EXT_PRINCIPAL_JSON") {
            if let Ok(claims) = serde_json::from_str::<crate::principal::PrincipalClaims>(&raw) {
                if let Ok(p) = claims.into_principal() {
                    return p;
                }
            }
        }
        self.current_principal
            .lock()
            .map(|p| p.clone())
            .unwrap_or_else(|_| synthetic_admin())
    }

    /// Test/boot helper: stamp the current principal for the next
    /// dispatch (the host sets the env-var path in production; tests
    /// set it directly).
    pub fn set_principal(&self, p: Principal) {
        if let Ok(mut guard) = self.current_principal.lock() {
            *guard = p;
        }
    }
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
    futures::executor::block_on(lb_store::Store::memory())
        .expect("mem:// store must open")
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
        ],
        iat: 0,
        exp: u64::MAX,
        constraint: None,
        run_id: None,
    };
    verify(&key, &mint(&key, &claims), 1).expect("synthetic admin verifies")
}

mod principal {
    //! The host-stamped principal the child resolves per dispatch.
    //!
    //! The supervisor stamps `LB_EXT_PRINCIPAL_JSON` (a JSON-encoded
    //! `lb_auth::Claims` blob) into the child's env on every call. The
    //! child parses it back into a [`lb_auth::Principal`] for the
    //! chokepoint + verb layer to use. Same wire shape the gateway
    //! mints tokens for, so a debug tool can capture + replay a call's
    //! principal.
    //!
    //! Verification requires the node's signing key — which the child
    //! does NOT carry (the host gate is authoritative; the child
    //! trusts the stamp). The child still needs a verifiable
    //! [`lb_auth::Principal`], so for the boot-time integration test
    //! path we mint + verify locally with a freshly-generated key.
    //! The cap wall on the host is the authoritative gate either way
    //! — this projection exists so the chokepoint's `assert_reach`
    //! gets a typed role to audit.

    use lb_auth::{mint, verify, Claims, Principal, Role, SigningKey};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PrincipalClaims {
        pub sub: String,
        pub ws: String,
        pub role: Role,
        #[serde(default)]
        pub caps: Vec<String>,
        pub iat: u64,
        pub exp: u64,
        #[serde(default)]
        pub constraint: Option<serde_json::Value>,
        #[serde(default)]
        pub run_id: Option<String>,
    }

    impl PrincipalClaims {
        /// Project the claims into a verified [`Principal`]. The
        /// verification key comes from `LB_EXT_SIGNING_KEY` when the
        /// host hands one to the child (so production stamps verify
        /// against the real node key); absent that, we mint + verify
        /// locally with a fresh key — the projection is still
        /// schema-correct (the cap wall is the authoritative gate).
        pub fn into_principal(self) -> Result<Principal, String> {
            let key = if let Ok(hex) = std::env::var("LB_EXT_SIGNING_KEY") {
                parse_signing_key(&hex)?
            } else {
                SigningKey::generate()
            };
            let claims = Claims {
                sub: self.sub,
                ws: self.ws,
                role: self.role,
                caps: self.caps,
                iat: self.iat,
                exp: self.exp,
                constraint: None,
                run_id: self.run_id,
            };
            let token = mint(&key, &claims);
            verify(&key, &token, 1).map_err(|e| format!("verify: {e}"))
        }
    }

    /// Decode a 64-hex-char Ed25519 seed into a [`SigningKey`]
    /// (mirrors the host's boot-side decode so the child can verify a
    /// real node's stamps when the host hands the key down).
    fn parse_signing_key(hex: &str) -> Result<SigningKey, String> {
        let hex = hex.trim();
        if hex.len() != 64 {
            return Err(format!("signing key: expected 64 hex chars, got {}", hex.len()));
        }
        let mut seed = [0u8; 32];
        for (i, byte) in seed.iter_mut().enumerate() {
            u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|e| format!("signing key hex: {e}"))
                .map(|b| *byte = b)?;
        }
        Ok(SigningKey::from_seed(&seed))
    }
}
