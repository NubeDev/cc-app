//! The era-1 / era-2 scope resolution. The CHOKEPOINT's call sites call
//! into this module; the module decides which era implements the look-up.
//!
//! ## Era 1 (now)
//!
//! `resolve_era1_guardian` reads the `guardianship` edge record directly
//! from the store (workspace-scoped) and asserts the edge is live.
//! `resolve_era1_guardian_set` lists all live edges for the principal.
//! `resolve_era1_staff_rooms` lists all live `staff_assignment` records.
//!
//! All three resolve per call, per-request cache only. No cross-request
//! cache (staleness = leak, see `mod.rs` doc).
//!
//! ## Era 2 (LIVE — `super::host_callback`)
//!
//! Era 2 is wired and live as of milestone 03 (node-v0.3.0 shipped the
//! native host-callback client). The chokepoint delegates to lb's
//! entity-scoped grants (`authz.check_scoped` / `authz.scope_filter`)
//! through [`super::host_callback::ReachClient`] whenever a
//! [`super::Chokepoint`] carries one (`with_host_callback`). The functions
//! in THIS module are the era-1 FALLBACK — used when no host-callback client
//! is present (store-only unit tests, or when lb's verbs aren't reachable).
//! The call sites in `mod.rs` are identical across both eras (the whole
//! point of the chokepoint), so which era runs is a construction choice, not
//! a call-site change.

use lb_auth::Principal;
use lb_store::{list, read};

use super::AuthzError;
use super::Chokepoint;

/// Resolve a single guardian→child reach decision (era 1).
///
/// Reads the `guardianship:<edge_id>` record from the store (workspace-
/// scoped) and asserts the edge is live. The edge id is derived from
/// `(guardian_sub, child_id)` so the read is O(1) and the matrix harness
/// can seed it directly.
///
/// TODO(era-2): replace this body with a call to lb's
/// `authz.check_scoped` once the native child has a host-callback
/// client. The call site in [`super::assert_reach`] does NOT change.
pub async fn resolve_era1_guardian(
    cp: &Chokepoint,
    principal: &Principal,
    child_id: &str,
) -> Result<(), AuthzError> {
    let edge_id = edge_id(principal.sub(), child_id);
    let row = match read(&cp.store, &cp.ws, "guardianship", &edge_id).await {
        Ok(Some(row)) => row,
        // No edge ⇒ deny. (Empty for get/update/watch paths — a list
        // would have translated this to an empty reply in
        // `reachable_children`.)
        Ok(None) => {
            return Err(AuthzError::Denied {
                reason: "no live guardianship edge",
            });
        }
        Err(_) => {
            // Store errors deny — the chokepoint never bubbles them. A
            // future milestone routes the read failure to the platform
            // audit reactor.
            return Err(AuthzError::Denied {
                reason: "guardianship read failed",
            });
        }
    };

    let live = row.get("live").and_then(|v| v.as_bool()).unwrap_or(false);
    if !live {
        return Err(AuthzError::Denied {
            reason: "guardianship edge is archived",
        });
    }
    Ok(())
}

/// Resolve the set of children a guardian reaches (era 1).
///
/// Lists all live `guardianship` rows where `data.guardian_sub = sub`.
/// Empty when none (list-verbs return an empty reply).
///
/// TODO(era-2): replace this body with a call to lb's `authz.scope_filter`
/// — the verb body interprets an empty reply identically.
pub async fn resolve_era1_guardian_set(cp: &Chokepoint, principal: &Principal) -> Vec<String> {
    let rows = match list(
        &cp.store,
        &cp.ws,
        "guardianship",
        "guardian_sub",
        principal.sub(),
    )
    .await
    {
        Ok(rs) => rs,
        Err(_) => return Vec::new(),
    };
    rows.into_iter()
        .filter_map(|row| {
            let live = row.get("live").and_then(|v| v.as_bool()).unwrap_or(false);
            if !live {
                return None;
            }
            row.get("child_id")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect()
}

/// Resolve the guardians who receive the daily feed for a child (era 1) — the
/// push + emit recipient set for `care.log.add` (daily-feed-scope §Push:
/// "`notify.send` to the guardians holding `receives_daily_feed` edges").
///
/// Lives HERE (inside `authz/`) because it reads the `guardianship` table — the
/// grep fence (`check-authz-fence.sh`) forbids that word outside this module,
/// and "which guardians reach this child, with which per-edge flag" IS a reach
/// question. Returns only guardians whose edge is LIVE and carries
/// `receives_daily_feed == true`: a `false` edge gets NEITHER feed NOR push
/// (asserted in daily-feed tests), and an archived (unlinked) edge is dropped
/// so a former guardian never receives a new entry. Each returned tuple is
/// `(guardian_sub, receives_daily_feed)` — always `true` here, but the flag is
/// carried explicitly so the caller reads intent, not a bare list.
///
/// Empty when no live feed-edge holder exists (a private child, or every edge
/// opted out) — never an error (fail-closed: a store fault yields no
/// recipients rather than a broadcast).
pub async fn resolve_era1_feed_recipients(cp: &Chokepoint, child_id: &str) -> Vec<String> {
    let rows = match list(&cp.store, &cp.ws, "guardianship", "child_id", child_id).await {
        Ok(rs) => rs,
        Err(_) => return Vec::new(),
    };
    rows.into_iter()
        .filter_map(|row| {
            let live = row.get("live").and_then(|v| v.as_bool()).unwrap_or(false);
            let receives = row
                .get("receives_daily_feed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !live || !receives {
                return None;
            }
            row.get("guardian_sub")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect()
}

/// Resolve the rooms a staff member reaches (era 1).
///
/// Lists all `staff_assignment` rows where `data.staff_sub = sub`. Empty
/// when none. Same era-2 swap as the guardian set.
pub async fn resolve_era1_staff_rooms(cp: &Chokepoint, principal: &Principal) -> Vec<String> {
    let rows = match list(
        &cp.store,
        &cp.ws,
        "staff_assignment",
        "staff_sub",
        principal.sub(),
    )
    .await
    {
        Ok(rs) => rs,
        Err(_) => return Vec::new(),
    };
    rows.into_iter()
        .filter_map(|row| {
            row.get("room_id")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect()
}

/// Canonicalise a guardian subject to the **auth-subject** form the reach path
/// keys on. A guardian's token `sub` is `user:<x>` (the gateway mints it that
/// way; the era-1 edge read uses `principal.sub()`, and the era-2 grant subject
/// must parse as `Subject::User` — a bare id is rejected by lb's `Subject::parse`
/// and the `grants.assign` denies). A caller (the seed, the admin UI) may pass a
/// bare guardian id (`ana`) OR the already-prefixed form (`user:ana`); both
/// normalise to `user:ana`, so the edge id, the era-1 lookup, and the era-2
/// grant subject all address the SAME identity. Idempotent on an already-prefixed
/// subject. One owner of this rule (a drift = a lockout or a leak).
pub fn canonical_subject(guardian_sub: &str) -> String {
    if guardian_sub.starts_with("user:")
        || guardian_sub.starts_with("team:")
        || guardian_sub.starts_with("key:")
    {
        guardian_sub.to_string()
    } else {
        format!("user:{guardian_sub}")
    }
}

/// The deterministic edge id (so the matrix harness can seed it directly).
pub fn edge_id(guardian_sub: &str, child_id: &str) -> String {
    // Same shape as the durable `guardianship` edge id (the link verb in
    // milestone 03 derives it this way too). Sorting not needed — the
    // edge has a natural direction (guardian → child).
    format!("{}::{}", guardian_sub, child_id)
}

/// A derived channel member — the subject + the role (Full = post+read;
/// ReadOnly = read only). Milestone 09: membership is DERIVED here (behind the
/// fence — it reads `guardianship`), never hand-managed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelMember {
    pub subject: String,
    pub full: bool,
}

/// Derive the membership of a CHILD channel (`care-child-<child_id>`): the
/// child's messaging-enabled guardians (live edge + `receives_messaging`) + the
/// staff assigned to the child's room — all FULL members (post + read). This is
/// the leak vector: a guardian appears iff a LIVE edge with the messaging flag
/// exists, so an unlinked or non-messaging guardian is absent (their next read
/// 403s at lb's gate). Fail-closed: a store fault yields NO members rather than
/// a broadcast. Behind the fence (reads `guardianship`).
pub async fn resolve_child_channel_members(cp: &Chokepoint, child_id: &str) -> Vec<ChannelMember> {
    let mut members = Vec::new();

    // Messaging-enabled guardians of the child.
    if let Ok(rows) = list(&cp.store, &cp.ws, "guardianship", "child_id", child_id).await {
        for row in rows {
            let live = row.get("live").and_then(|v| v.as_bool()).unwrap_or(false);
            let messaging = row
                .get("receives_messaging")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !live || !messaging {
                continue;
            }
            if let Some(sub) = row.get("guardian_sub").and_then(|v| v.as_str()) {
                members.push(ChannelMember { subject: sub.to_string(), full: true });
            }
        }
    }

    // Staff of the child's room (read child → room → its assignments).
    if let Some(room_id) = child_room(cp, child_id).await {
        for sub in room_staff(cp, &room_id).await {
            members.push(ChannelMember { subject: sub, full: true });
        }
    }

    dedupe_members(members)
}

/// Derive the STAFF membership of a ROOM channel (`care-room-<room_id>`) — the
/// room's assigned staff, all FULL members. Same fail-closed + fence posture as
/// the child channel.
///
/// ## Why staff-only here (guardians are event-driven, not derived)
///
/// The generic store `list` returns a row's DATA, never its key, and the `child`
/// record carries no `id` field — so a room cannot enumerate "its children" (and
/// thus their guardians) by derivation. That is fine: guardian room-membership is
/// reconciled at the PLACEMENT event, where the child id IS known (the enrollment/
/// room-move handler calls `grant_membership` for the child's messaging guardians
/// on the room channel). The derivation's job is the STAFF set (the stable
/// broadcast authors); the per-placement handler owns the guardian folding. A
/// healing sweep over a room reconciles staff; a per-child sweep reconciles that
/// child's guardians onto both channels.
pub async fn resolve_room_channel_members(cp: &Chokepoint, room_id: &str) -> Vec<ChannelMember> {
    dedupe_members(
        room_staff(cp, room_id)
            .await
            .into_iter()
            .map(|subject| ChannelMember { subject, full: true })
            .collect(),
    )
}

/// The `child.room_id` for a child (None if unenrolled or missing). Behind the
/// fence's module but reads `child` (not `guardianship`), so no fence concern.
async fn child_room(cp: &Chokepoint, child_id: &str) -> Option<String> {
    let value = read(&cp.store, &cp.ws, "child", child_id).await.ok()??;
    value
        .get("room_id")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// The staff subjects assigned to a room (lists `staff_assignment` by room).
async fn room_staff(cp: &Chokepoint, room_id: &str) -> Vec<String> {
    let Ok(rows) = list(&cp.store, &cp.ws, "staff_assignment", "room_id", room_id).await else {
        return Vec::new();
    };
    rows.into_iter()
        .filter_map(|row| {
            row.get("staff_sub")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect()
}

/// Dedupe members by subject, keeping the STRONGEST role (Full wins over
/// ReadOnly) — a person who is both a room's staff and a guardian is one Full
/// member, never a duplicate grant.
fn dedupe_members(members: Vec<ChannelMember>) -> Vec<ChannelMember> {
    let mut out: Vec<ChannelMember> = Vec::new();
    for m in members {
        if let Some(existing) = out.iter_mut().find(|e| e.subject == m.subject) {
            existing.full = existing.full || m.full;
        } else {
            out.push(m);
        }
    }
    out
}

/// The `Scope` of a single verb — the cap + the chokepoint call shape.
/// Used by the matrix harness to enumerate verbs and assert the allow /
/// deny / empty table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scope {
    /// The `mcp:care.<verb>:call` cap the verb requires (the host's
    /// wall check).
    pub cap: &'static str,
    /// The chokepoint call the verb body makes.
    pub kind: ScopeKind,
}

/// The two scope shapes the chokepoint enforces. The matrix harness
/// runs each verb against the right shape — `Single` for get/update/
/// watch, `Set` for list (empty-on-miss), `Rooms` for staff-scoped
/// lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// `assert_reach(principal, child_id)` — get/update/watch.
    Single,
    /// `reachable_children(principal)` — list of child ids (empty on
    /// miss).
    Set,
    /// `reachable_rooms(principal)` — list of room ids (empty on miss).
    Rooms,
}
