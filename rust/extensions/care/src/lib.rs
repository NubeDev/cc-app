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
//! - (per-verb folders `center/`, `room/`, `child/`, `guardian/`, … come in
//!   later milestones — folders exist as one-line stubs today.)
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

/// A workspace handle is stamped onto every child by the host at spawn
/// (`LB_EXT_WS` env). Today every tool body is workspace-scoped through the
/// args; we carry it on the impl so it's visible to every verb body without
/// plumbing it through params.
#[derive(Clone)]
pub struct Care {
    pub ws: String,
}

impl Care {
    /// Build a `Care` impl for the workspace the host spawned us in.
    pub fn new(ws: String) -> Self {
        Self { ws }
    }
}
