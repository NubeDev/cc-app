//! care.invite.* — admin mints an invite (guardian / staff); the lb side
//! emails it; the pre-auth accept page handles the bind (milestone 05).
//!
//! Wired — `node-v0.3.3` ships the upstream `grants.*`/`roles.*`/`teams.*`
//! routing AND the live invite verbs; each verb mints/revokes/resends via
//! `SidecarClient::call_tool("invite.create"|"invite.revoke"|"invite.resend")`
//! using the SAME host-callback client the era-2 chokepoint reads from.
//!
//! Per FILE-LAYOUT §2: `mod.rs` is the barrel; one verb per file
//! (`create_guardian` / `create_staff` / `list` / `revoke` / `resend`).

pub mod create_guardian;
pub mod create_staff;
pub mod list;
pub mod resend;
pub mod revoke;

mod records;
mod token_hash;

pub use records::{Invite, InviteError, InviteRole, InviteStatus, Locale};
pub use token_hash::hash_invite_token;
