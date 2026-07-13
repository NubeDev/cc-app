//! `care.invite.*` — admin mints an invite; pre-auth accept on the lb side;
//! the extension binds the new `sub` on `invite.accepted` and (era 2) derives
//! the guardian's scoped reach grants from their existing edges.
//!
//! Scope: [`../../../../../docs/scope/care/enrollment-invites-scope.md`](../../../../../docs/scope/care/enrollment-invites-scope.md).
//! Build: [`../../../../../docs/build/05-invites-golden-path.md`](../../../../../docs/build/05-invites-golden-path.md).
//!
//! ## Orchestrator-owned shapes
//!
//! - `Invite` — the durable mirror row the extension keeps per invite it has
//!   minted (admin list / revoke / resend / park queue read from here; the lb
//!   invite itself is the source of truth for `accepted_at`, lb owns the email
//!   delivery).
//! - `InviteRole` — the role the invite carries (`guardian-member` /
//!   `staff-member`); lb's `invite.create` accepts the role string verbatim.
//! - `InviteStatus` — the lifecycle (`pending` → `sent` → `accepted` |
//!   `revoked` | `expired` | `parked`).
//! - `InviteError` — the typed surface the verb layer maps to the MCP error
//!   shape; the words live in the Display impls (the audit-key analog, the
//!   hardcoded-string lint excludes them per the script's comment).
//!
//! ## Wire contract (lb `invite.create` / `invite.revoke` / `invite.list`)
//!
//! - `invite.create { email, role, team, payload, locale }` — `team`
//!   (`guardians` | `staff`) and `payload` (the `guardian_id` / staff slot)
//!   are opaque to lb (rule 10); `locale` drives the email template.
//! - `invite.revoke { invite_id }` — admin-side kill switch; the extension's
//!   `care.invite.revoke` mirrors the same row.
//! - `invite.accepted` — host-side event the extension consumes via a hook
//!   (milestone 05); NOT a `care.invite.*` verb (the accept page is pre-auth
//!   and lives outside the child wire).
//!
//! ## Scaffolded state (this session)
//!
//! Per `docs/debugging/authz/grants-verbs-not-on-mcp-callback-surface.md`,
//! the upstream `grants.*` / `roles.*` / `teams.*` routing fix has NOT yet
//! shipped in lb (newest published tag is still `node-v0.3.1`). Milestone 05's
//! accept-side grant derivation (bind `sub → guardian`, then derive the
//! guardian's scoped grants from existing edges) is therefore a TODO — the
//! verbs are scaffolded to the SHAPE the milestone 05 implementation will
//! fill in. Each verb's `run` validates input + persists the local mirror
//! row, then returns `InviteError::NotImplemented` for the SidecarClient
//! `invite.create` / `invite.revoke` call. The next session (after lb ships
//! the routing fix + the invite relay is verified delivering per the build-05
//! entry gate) flips the body to the real call.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A `care.invite.*` mirror row (workspace-scoped). lb's invite store is the
/// authoritative source for `lb_invite_id`, `accepted_at`, and email-delivery
/// status; the extension mirrors what it needs to drive the admin UI
/// (pending list, re-send, revoke, park queue) without round-tripping to lb
/// on every admin interaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Invite {
    /// The extension's id for the invite (a deterministic slug the admin can
    /// see in the UI — e.g. `inv-<guardian_id>` or `inv-staff-<room>-<n>`).
    pub id: String,
    /// The guardian record this invite binds to (`guardian-member` invites
    /// only; `None` for staff invites).
    #[serde(default)]
    pub guardian_id: Option<String>,
    /// The invite email — the lb side emails this address.
    pub email: String,
    /// The role the invite carries (the `role` parameter on lb's
    /// `invite.create`).
    pub role: InviteRole,
    /// The staff-room assignment (`staff-member` invites only). Drives the
    /// downstream `staff_assignment` row.
    #[serde(default)]
    pub room_id: Option<String>,
    /// The locale the invite email renders in — the admin passes it from the
    /// guardian record's `locale` (records-before-accounts), so a Spanish-
    /// speaking Ana gets a Spanish email.
    pub locale: Locale,
    /// The lifecycle status (admin sees this in the list).
    pub status: InviteStatus,
    /// The lb `invite.create` id once minted (set when the body flips from
    /// scaffolded to live — see module doc).
    #[serde(default)]
    pub lb_invite_id: Option<String>,
    /// `created_at` (unix ms).
    pub created_at_ms: u64,
    /// `sent_at` (unix ms; `None` while still `pending`).
    #[serde(default)]
    pub sent_at_ms: Option<u64>,
    /// `accepted_at` (unix ms; mirror of lb's host-side event).
    #[serde(default)]
    pub accepted_at_ms: Option<u64>,
    /// `revoked_at` (unix ms; admin kill switch).
    #[serde(default)]
    pub revoked_at_ms: Option<u64>,
    /// `expired_at` (unix ms; token TTL).
    #[serde(default)]
    pub expired_at_ms: Option<u64>,
    /// Why this invite is parked for admin review (the accept email did NOT
    /// match the guardian record's email — `enrollment-invites-scope.md`
    /// §"Risks: wrong-person binding"). `None` when not parked.
    #[serde(default)]
    pub parked_reason: Option<String>,
}

/// The role an invite carries — the value lb's `invite.create` accepts on its
/// `role` parameter. Adding a role is a here-and-lb catalog add.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InviteRole {
    /// The invitee is a guardian (their accept binds `sub → guardian_id`).
    GuardianMember,
    /// The invitee is staff (their accept binds `sub` + a `staff_assignment`
    /// row for the room).
    StaffMember,
}

impl InviteRole {
    /// The wire string for lb's `invite.create { role }`.
    pub fn as_str(&self) -> &'static str {
        match self {
            InviteRole::GuardianMember => "guardian-member",
            InviteRole::StaffMember => "staff-member",
        }
    }
    /// The `team` parameter on lb's `invite.create` (the lb-side team the
    /// invite is filed under — drives the default cap set).
    pub fn team(&self) -> &'static str {
        match self {
            InviteRole::GuardianMember => "guardians",
            InviteRole::StaffMember => "staff",
        }
    }
}

/// The lifecycle status. The extension mirrors lb's authoritative state for
/// `accepted` (driven by the `invite.accepted` host-side event in milestone
/// 05); the extension owns `revoked` (admin kill switch), `expired` (token
/// TTL, also surface from lb), and `parked` (mismatch-at-accept — the
/// extension is the only one that knows why).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InviteStatus {
    /// Created in the extension but `invite.create` not yet sent (the scaffold
    /// state — flips to `Sent` the moment the SidecarClient call lands).
    Pending,
    /// `invite.create` delivered to lb, the email is in flight.
    Sent,
    /// The invitee accepted — the lb-side event triggered the bind.
    Accepted,
    /// Admin killed the invite (care.invite.revoke) — the email is invalidated
    /// and the token can no longer be exchanged.
    Revoked,
    /// The token TTL elapsed without an accept — surfaced from lb.
    Expired,
    /// The accept email did NOT match the guardian record's email
    /// (`enrollment-invites-scope.md` §"Risks: wrong-person binding") — the
    /// invite is parked for admin review instead of bound.
    Parked,
}

impl InviteStatus {
    /// The wire form (`"pending"` / `"sent"` / …) — same as
    /// `serde_json::to_value(self)` but `&'static str` so the list filter
    /// can compare without allocating.
    pub fn as_str(&self) -> &'static str {
        match self {
            InviteStatus::Pending => "pending",
            InviteStatus::Sent => "sent",
            InviteStatus::Accepted => "accepted",
            InviteStatus::Revoked => "revoked",
            InviteStatus::Expired => "expired",
            InviteStatus::Parked => "parked",
        }
    }
}

/// The locale the invite renders in. Re-use the `center::Locale` so the
/// guardian-record locale and the invite-email locale are the same type
/// (records-before-accounts binds locale to the record, then the invite email
/// renders in it — `enrollment-invites-scope.md` §"Localized onboarding").
pub use crate::center::Locale;

/// Typed errors the verb layer maps to the MCP `ToolError` shape. Words live
/// in the Display impls (the audit-key analog; the hardcoded-string lint
/// excludes these per the script's `ERROR_CONTEXT` carve-out).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InviteError {
    /// The verb body is scaffolded — the SidecarClient `invants.create` /
    /// `invite.revoke` call is a TODO (milestone 05, blocked on lb's
    /// `grants.*` routing fix shipping + the invite relay verified delivering).
    /// The string is developer-facing (audit), not user chrome.
    NotImplemented(&'static str),
    /// The invite email was empty or shape-invalid (a typo parks the invite
    /// later — reject the obvious garbage now).
    InvalidEmail(String),
    /// A required id was empty.
    MissingField(&'static str),
    /// The guardian record this invite would bind to does not exist.
    GuardianNotFound(String),
    /// The role string was outside the enum (paranoia — the wire layer
    /// already restricts it).
    InvalidRole(String),
    /// The invite row was not found (revoke / resend / get on a missing id).
    NotFound(String),
    /// The store denied the read or write.
    StoreDenied(String),
}

impl fmt::Display for InviteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InviteError::NotImplemented(what) => write!(
                f,
                "not implemented (milestone 05): {what} — see docs/debugging/authz/grants-verbs-not-on-mcp-callback-surface.md"
            ),
            InviteError::InvalidEmail(e) => write!(f, "invalid email: {e:?}"),
            InviteError::MissingField(s) => write!(f, "missing required field: {s}"),
            InviteError::GuardianNotFound(g) => write!(f, "guardian not found: {g}"),
            InviteError::InvalidRole(r) => write!(f, "invalid role: {r:?}"),
            InviteError::NotFound(id) => write!(f, "invite not found: {id}"),
            InviteError::StoreDenied(what) => write!(f, "store denied: {what}"),
        }
    }
}

impl std::error::Error for InviteError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invite_role_wire_shape_matches_lb() {
        assert_eq!(InviteRole::GuardianMember.as_str(), "guardian-member");
        assert_eq!(InviteRole::GuardianMember.team(), "guardians");
        assert_eq!(InviteRole::StaffMember.as_str(), "staff-member");
        assert_eq!(InviteRole::StaffMember.team(), "staff");
    }

    #[test]
    fn invite_status_serializes_lowercase() {
        let v = serde_json::to_value(InviteStatus::Pending).unwrap();
        assert_eq!(v, serde_json::json!("pending"));
        let v = serde_json::to_value(InviteStatus::Parked).unwrap();
        assert_eq!(v, serde_json::json!("parked"));
    }

    #[test]
    fn invite_error_display_carries_the_debug_entry_pointer() {
        let s = InviteError::NotImplemented("invite.create").to_string();
        assert!(s.contains("milestone 05"));
        assert!(s.contains("grants-verbs-not-on-mcp-callback-surface"));
    }
}
