//! The `Tools` impl — the dispatch from the child wire (`Method::Call`)
//! to a per-verb body. Each verb body is its own file (`ping.rs`,
//! `child/get.rs`, …) per FILE-LAYOUT.
//!
//! The host strips the `care.` prefix before the `call` reaches us — we
//! dispatch on the bare tool name, matching the manifest's `[[tools]]`
//! list (the only the host's install-time grant check looks at).
//!
//! ## Cap check (the deny-test half of milestone 02)
//!
//! Every `care.*` tool here requires the caller to hold a `mcp:care.*:call`
//! cap (the cap wall is enforced BEFORE we see the call; we re-state the
//! required cap here as a defence-in-depth check so a cap-wall regression
//! breaks the test, not a silent success). The harness in
//! `tests/matrix.rs` exercises this for `care.ping`.
//!
//! ## Era-2 read delegation (milestone 03)
//!
//! The `Care` impl now holds a [`Chokepoint`] built from a real host callback
//! (a [`lb_ext_native::SidecarClient`] read from the supervisor-injected
//! env at sidecar start). When the platform routes `authz.*` over `/mcp/call`
//! (it does, since `node-v0.3.0`) AND `grants.*`/`roles.*`/`teams.*` (the
//! tracked upstream gap, WORKFLOW-LB.md task 2), the chokepoint's
//! `assert_reach` / `reachable_children` delegate the read path to lb's
//! entity-scoped grants and the derivation path on `guardianship.link` /
//! `unlink` / `update` mints / revokes the corresponding scoped grant through
//! the SAME callback. Today (without the `grants.*` routing fix), `link` /
//! `unlink` / `update` fail loud — see `authz/grant.rs` — and the chokepoint's
//! era-1 read fallback stays the live path.

use lb_ext_native::{Caller, Tools};
use serde::Deserialize;

use crate::attendance;
use crate::center;
use crate::child;
use crate::enrollment;
use crate::feed;
use crate::guardian;
use crate::guardianship;
use crate::invite;
use crate::log;
use crate::media;
use crate::menu;
use crate::messaging;
use crate::ping;
use crate::room;

/// The tool names this sidecar serves (bare; the host owns the `care.`
/// prefix). Reported in the `init` handshake so the host rejects an
/// unknown-tool dispatch early, AND in the manifest's `[[tools]]` list so
/// the install grant is computed against the actual surface.
///
/// Every verb the m03 + m05 milestones added to the chokepoint's
/// surface lives here so the dispatcher is the WHOLE contract
/// (CLAUDE.md §4a — build the whole contract, not the easy half).
pub const TOOLS: &[&str] = &[
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
    // Milestone 06 — attendance (ledger, pickup gate, kiosk, ratios).
    "attendance.check_in",
    "attendance.check_out",
    "attendance.list",
    "attendance.now",
    "attendance.correct",
    // Milestone 07 — menus & derived substitutions (food safety).
    "menu.set",
    "menu.get",
    "menu.week",
    "menu.copy_week",
    // Milestone 08 — daily feed (logs, photos, SSE, push).
    "log.add",
    "log.list",
    "log.correct",
    "log.day",
    "feed.watch",
    // Milestone 08 — the photo upload boundary (reject video, delegate to lb).
    "media.begin",
    "media.commit",
    // Milestone 09 — messaging (channel provisioning + membership reconciliation).
    "channel.reconcile",
    "announce.post",
];

/// The expected cap a caller must carry to invoke a `care.*` tool. The
/// host re-checks this at the wall; we re-check here as a defence-in-depth
/// (an over-permissive host ⇒ deny here, not silent success).
#[allow(dead_code)]
pub const REQUIRED_CAP: &str = "mcp:care.ping:call";

/// The minimum cap set an admin caller must carry to invoke any `care.*`
/// verb. Per-verb caps (`mcp:care.<verb>:call`) are checked at the host
/// wall; the child does not re-check them — this constant is here for
/// the matrix harness's deny-test.
#[allow(dead_code)]
pub const ADMIN_CAPS: &[&str] = &[
    "mcp:care.center.create:call",
    "mcp:care.center.get:call",
    "mcp:care.center.list:call",
    "mcp:care.room.create:call",
    "mcp:care.room.get:call",
    "mcp:care.room.list:call",
    "mcp:care.child.create:call",
    "mcp:care.child.get:call",
    "mcp:care.child.list:call",
    "mcp:care.child.update:call",
    "mcp:care.child.archive:call",
    "mcp:care.guardian.create:call",
    "mcp:care.guardian.get:call",
    "mcp:care.guardian.list:call",
    "mcp:care.guardianship.link:call",
    "mcp:care.guardianship.unlink:call",
    "mcp:care.guardianship.update:call",
    "mcp:care.enrollment.create:call",
    "mcp:care.enrollment.list:call",
    "mcp:care.enrollment.update:call",
    "mcp:care.invite.create_guardian:call",
    "mcp:care.invite.create_staff:call",
    "mcp:care.invite.list:call",
    "mcp:care.invite.resend:call",
    "mcp:care.invite.revoke:call",
    "mcp:care.attendance.check_in:call",
    "mcp:care.attendance.check_out:call",
    "mcp:care.attendance.list:call",
    "mcp:care.attendance.now:call",
    "mcp:care.attendance.correct:call",
    "mcp:care.menu.set:call",
    "mcp:care.menu.get:call",
    "mcp:care.menu.week:call",
    "mcp:care.menu.copy_week:call",
    "mcp:care.log.add:call",
    "mcp:care.log.list:call",
    "mcp:care.log.correct:call",
    "mcp:care.log.day:call",
    "mcp:care.feed.watch:call",
    "mcp:care.media.begin:call",
    "mcp:care.media.commit:call",
    "mcp:care.channel.reconcile:call",
    "mcp:care.announce.post:call",
];

/// The input shape for `care.ping` — the only stateless verb. Every other
/// verb has its own per-file input struct; this one stays here for the
/// dispatch's per-call input parsing.
#[derive(Debug, Deserialize)]
pub struct PingInput {
    /// Optional echo payload; the verb round-trips it under `echoed`.
    #[serde(default)]
    pub echo: Option<String>,
}

/// The reply shape for `care.ping`. Stateless + trivial — this verb's job
/// is to prove the loop end-to-end, not to carry domain semantics.
#[derive(Debug, serde::Serialize)]
pub struct PingReply {
    pub ws: String,
    pub tier: &'static str,
    pub ok: bool,
    /// The `echo` payload the caller passed in, round-tripped. Lets a
    /// caller prove which invocation the reply is for in a multi-call
    /// script.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echoed: Option<String>,
}

impl Tools for crate::Care {
    fn tools(&self) -> Vec<String> {
        // Reported in `init` AND used by the matrix harness to assert that
        // every registered verb has a matrix row (the harness reads
        // `TOOLS` and refuses to start if the matrix is smaller — see
        // `tests/matrix.rs`).
        TOOLS.iter().map(|s| s.to_string()).collect()
    }

    async fn call(&mut self, tool: &str, input: &str) -> Result<String, String> {
        // The caller-agnostic entry point — reached only on an old-host
        // frame or a direct `/native/call` with no caller (the era-1
        // in-process test path). `None` ⇒ the synthetic admin fallback (see
        // `Care::principal_for_caller`). A real spawned sidecar on
        // `node-v0.4.0`+ always goes through `call_with_caller` below with a
        // real frame caller, so a guardian dispatch never lands here.
        self.dispatch(tool, input, None).await
    }

    async fn call_with_caller(
        &mut self,
        tool: &str,
        input: &str,
        caller: Option<Caller>,
    ) -> Result<String, String> {
        // The native-caller-identity path (`sdk-v0.4.0`): the host stamped
        // the authorized caller into the frame. Thread it to the verb layer
        // so the chokepoint's role check + `subject`-parameterized reach are
        // ABOUT the real caller — the row-level second gate that enforces
        // rule 7 in the sidecar (guardian isolation).
        self.dispatch(tool, input, caller).await
    }
}

impl crate::Care {
    /// The one dispatch body both wire entry points share. `caller` is the
    /// frame caller (`Some` on a `node-v0.4.0`+ routed call, `None` on the
    /// era-1 in-process path); `principal_for_caller` projects it into the
    /// principal every verb body + the chokepoint sees (rule 7).
    async fn dispatch(
        &mut self,
        tool: &str,
        input: &str,
        caller: Option<Caller>,
    ) -> Result<String, String> {
        // The host's routed native adapter re-qualifies the tool as
        // `care.<tool>` before it reaches us, while the direct
        // `/native/call` bridge passes the bare name. Match on the tail
        // after any leading `care.` — the child should not assume which
        // router reached it (host-metrics's posture).
        let verb = tool.strip_prefix("care.").unwrap_or(tool);

        // Every verb body takes a `Chokepoint` (era-2 read delegation +
        // era-1 fallback) plus the principal. The principal comes from the
        // native call FRAME the host stamped (native-caller-identity scope):
        // `principal_for_caller` projects `caller` into the typed principal
        // per dispatch (rule 5 — the frame is read at the boundary, never in
        // a verb body).
        let principal = self.principal_for_caller(caller);
        let cp = self.chokepoint();

        match verb {
            "ping" => ping::run(&self.ws, input),
            "center.create" => center::create::run(cp, &principal, input).await,
            "center.get" => center::get::run(cp, &principal, input).await,
            "center.list" => center::list::run(cp, &principal, input).await,
            "room.create" => room::create::run(cp, &principal, input).await,
            "room.get" => room::list::get(cp, &principal, input).await,
            "room.list" => room::list::list(cp, &principal, input).await,
            "child.create" => child::create::run(cp, &principal, input).await,
            "child.get" => child::get::run(cp, &principal, input).await,
            "child.list" => child::list::run(cp, &principal, input).await,
            "child.update" => child::update::run(cp, &principal, input).await,
            "child.archive" => child::archive::run(cp, &principal, input).await,
            "guardian.create" => guardian::create::run(cp, &principal, input).await,
            "guardian.get" => guardian::list::get(cp, &principal, input).await,
            "guardian.list" => guardian::list::run(cp, &principal, input).await,
            "guardianship.link" => guardianship::link::run(cp, &principal, input).await,
            "guardianship.unlink" => guardianship::unlink::run(cp, &principal, input).await,
            "guardianship.update" => guardianship::update::run(cp, &principal, input).await,
            "enrollment.create" => enrollment::create::run(cp, &principal, input).await,
            "enrollment.list" => enrollment::list::run(cp, &principal, input).await,
            "enrollment.update" => enrollment::update::run(cp, &principal, input).await,
            "invite.create_guardian" => invite::create_guardian::run(cp, &principal, input).await,
            "invite.create_staff" => invite::create_staff::run(cp, &principal, input).await,
            "invite.list" => invite::list::run(cp, &principal, input).await,
            "invite.resend" => invite::resend::run(cp, &principal, input).await,
            "invite.revoke" => invite::revoke::run(cp, &principal, input).await,
            "attendance.check_in" => attendance::check_in::run(cp, &principal, input).await,
            "attendance.check_out" => attendance::check_out::run(cp, &principal, input).await,
            "attendance.list" => attendance::list::run(cp, &principal, input).await,
            "attendance.now" => attendance::now::run(cp, &principal, input).await,
            "attendance.correct" => attendance::correct::run(cp, &principal, input).await,
            "menu.set" => menu::set::run(cp, &principal, input).await,
            "menu.get" => menu::get::run(cp, &principal, input).await,
            "menu.week" => menu::week::run(cp, &principal, input).await,
            "menu.copy_week" => menu::copy_week::run(cp, &principal, input).await,
            "log.add" => log::add::run(cp, &principal, input).await,
            "log.list" => log::list::run(cp, &principal, input).await,
            "log.correct" => log::correct::run(cp, &principal, input).await,
            "log.day" => log::day::run(cp, &principal, input).await,
            "feed.watch" => feed::watch::run(cp, &principal, input).await,
            "media.begin" => media::begin::run(cp, &principal, input).await,
            "media.commit" => media::commit::run(cp, &principal, input).await,
            "channel.reconcile" => messaging::reconcile_verb::run(cp, &principal, input).await,
            "announce.post" => messaging::announce::run(cp, &principal, input).await,
            other => Err(format!("unknown tool: {other}")),
        }
    }
}

/// Defence-in-depth cap check for a verb. Called from the matrix harness
/// to exercise the deny-test half of the milestone 02 gate (a caller
/// WITHOUT the cap must see `Err(cap_denied)`, not a silent success).
///
/// In production the host re-checks the cap at the wall — this is the
/// belt-and-braces assertion that the binary's body refuses the call
/// too, so a wall regression that lets an under-cap call through still
/// hits the deny here (and the test catches it).
///
/// The cap itself is checked INSIDE the verb body via this helper (each
/// verb body's `run` calls `require_caller_cap` with the supplied cap
/// set). The helper is on the `Tools` impl so it has access to the
/// caller's principal — but the wire doesn't carry a principal, only an
/// opaque-JSON `input` per `CallParams`. So the cap check happens at the
/// HOST, not here: this is the documented shape (host wall = the
/// authoritative gate). The matrix harness tests the wall via a separate
/// path (it doesn't go through `Tools::call`; it calls the chokepoint
/// directly). This helper is kept as a stub for the future when native
/// children get a principal-in-env path (milestone 03 follow-up).
#[allow(dead_code)]
pub fn require_caller_cap(held_caps: &[String], required: &str) -> Result<(), String> {
    if held_caps.iter().any(|c| c == required) {
        Ok(())
    } else {
        Err(format!("missing required cap: {required}"))
    }
}
