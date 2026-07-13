//! `care.guardianship.update` — admin edits an existing edge's relationship
//! and/or the five flags. Cap: `mcp:care.guardianship.update:call`. Admin-only.
//!
//! Update does NOT change reach by itself (reach is `live`, governed by
//! `link`/`unlink`) — it edits the relationship + flags. BUT if an update
//! re-affirms a previously-unlinked edge (`live` false → true), it re-derives
//! the scoped grant (era 2), transactionally, exactly like `link`; and if it
//! sets `live` true → false it removes the grant, exactly like `unlink`. The
//! flags alone (`can_pickup`, …) touch no grant. This keeps the edge and its
//! grant in lockstep no matter which verb changes liveness.

use lb_auth::Principal;

use crate::authz::{grant, Chokepoint};
use crate::center::Locale;
use crate::guardianship::{edge_id, GuardianshipError, Relationship};
use crate::i18n::t;

#[derive(Debug, serde::Deserialize)]
pub struct UpdateInput {
    pub guardian_sub: String,
    pub child_id: String,
    /// New relationship key (optional — omitted leaves it unchanged).
    #[serde(default)]
    pub relationship: Option<String>,
    /// Liveness (optional). Setting this re-derives / removes the grant.
    #[serde(default)]
    pub live: Option<bool>,
    #[serde(default)]
    pub can_pickup: Option<bool>,
    #[serde(default)]
    pub receives_daily_feed: Option<bool>,
    #[serde(default)]
    pub receives_billing: Option<bool>,
    #[serde(default)]
    pub emergency_contact: Option<bool>,
    #[serde(default)]
    pub custody_notes: Option<String>,
    /// Milestone 09 — messaging access (a flip re-derives channel membership).
    #[serde(default)]
    pub receives_messaging: Option<bool>,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct UpdateReply {
    pub edge_id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let _ = principal;
    let parsed: UpdateInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.guardianship.update input: {e}"))?;
    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // Canonicalise the subject to the auth-subject form so the edge id + the
    // era-2 grant subject match what `link` derived (see `authz::canonical_subject`).
    let guardian_sub = crate::authz::canonical_subject(&parsed.guardian_sub);
    let id = edge_id(&guardian_sub, &parsed.child_id);
    let mut row = cp
        .records()
        .read("guardianship", &id)
        .await
        .map_err(|_| format!("{}", GuardianshipError::StoreDenied("update read".into())))?
        .ok_or_else(|| format!("{}", GuardianshipError::NotFound(id.clone())))?;

    let was_live = row.get("live").and_then(|v| v.as_bool()).unwrap_or(false);
    let was_messaging = row
        .get("receives_messaging")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let was_daily_feed = row
        .get("receives_daily_feed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Validate + apply the relationship change (if any).
    if let Some(r) = &parsed.relationship {
        let rel = Relationship::parse(r).map_err(|e| format!("{e}"))?;
        row["relationship"] = serde_json::Value::String(rel.as_key().to_string());
    }
    // Apply flag changes (each optional — only overwrite what was sent).
    for (key, val) in [
        ("can_pickup", parsed.can_pickup),
        ("receives_daily_feed", parsed.receives_daily_feed),
        ("receives_billing", parsed.receives_billing),
        ("emergency_contact", parsed.emergency_contact),
        ("receives_messaging", parsed.receives_messaging),
    ] {
        if let Some(b) = val {
            row[key] = serde_json::Value::Bool(b);
        }
    }
    if let Some(notes) = &parsed.custody_notes {
        row["custody_notes"] = serde_json::Value::String(notes.clone());
    }
    let now_live = parsed.live.unwrap_or(was_live);
    row["live"] = serde_json::Value::Bool(now_live);

    // Persist the edited edge.
    cp.records()
        .write("guardianship", &id, &row)
        .await
        .map_err(|e| {
            format!(
                "{}: {e}",
                GuardianshipError::StoreDenied("update write".into())
            )
        })?;

    // Keep the scoped grant in lockstep with a liveness transition (era 2).
    if let Some(reach) = cp.reach() {
        let res = match (was_live, now_live) {
            (false, true) => {
                grant::derive_reach(reach.client(), &guardian_sub, &parsed.child_id).await
            }
            (true, false) => {
                grant::remove_reach(reach.client(), &guardian_sub, &parsed.child_id).await
            }
            _ => Ok(()), // no liveness change ⇒ no grant change
        };
        if let Err(e) = res {
            // Roll the liveness flag back so edge + grant never diverge. Surface
            // a failed rollback loudly via the typed `Diverged` error (same
            // lockout/leak divergence as link/unlink; words in records.rs).
            row["live"] = serde_json::Value::Bool(was_live);
            let err = if cp.records().write("guardianship", &id, &row).await.is_ok() {
                GuardianshipError::GrantDerivationFailed(e.to_string())
            } else {
                GuardianshipError::GrantDerivationDiverged {
                    edge: id.clone(),
                    cause: e.to_string(),
                }
            };
            return Err(err.to_string());
        }
    }

    // Keep channel membership in lockstep with the messaging-access transition
    // (milestone 09). A guardian is a channel member iff `live && messaging`, so
    // a flip of EITHER flag can add/remove the seat. Best-effort-but-surfaced (a
    // missing grant is a lockout, a surviving grant a leak — both worth a retry);
    // no host client ⇒ no-op. It does not roll back the edge (the durable flags
    // are the source of truth; the healing sweep repairs a grant fault).
    let now_messaging = parsed.receives_messaging.unwrap_or(was_messaging);
    let was_member = was_live && was_messaging;
    let now_member = now_live && now_messaging;
    if was_member != now_member {
        if let Some(client) = cp.host_client() {
            let child_ch = crate::messaging::child_channel(&parsed.child_id);
            let res = if now_member {
                crate::messaging::reconcile::grant_membership(
                    Some(client),
                    &child_ch,
                    &guardian_sub,
                    crate::messaging::ChannelRole::Full,
                )
                .await
            } else {
                crate::messaging::reconcile::revoke_membership(
                    Some(client),
                    &child_ch,
                    &guardian_sub,
                )
                .await
            };
            res.map_err(|e| {
                [
                    "channel membership update failed (retry via reconcile): ",
                    &e.to_string(),
                ]
                .concat()
            })?;
        }
    }

    // Keep the feed-watch cap in lockstep with the daily-feed-access transition
    // (milestone 10). Feed access = `live && receives_daily_feed`; a flip of
    // EITHER flag adds/removes the per-child `bus:care.feed.<child>:watch` grant
    // that gates the guardian's live SSE subscribe (lb#49 / node-v0.4.3). A revoke
    // here ALSO terminates an open stream within a 3s tick. Same
    // best-effort-but-surfaced discipline as the channel + reach grants; no host
    // client ⇒ no-op.
    let now_daily_feed = parsed.receives_daily_feed.unwrap_or(was_daily_feed);
    let was_feed = was_live && was_daily_feed;
    let now_feed = now_live && now_daily_feed;
    if was_feed != now_feed {
        if let Some(client) = cp.host_client() {
            let res = if now_feed {
                crate::feed::grant_feed_watch(Some(client), &parsed.child_id, &guardian_sub).await
            } else {
                crate::feed::revoke_feed_watch(Some(client), &parsed.child_id, &guardian_sub).await
            };
            res.map_err(|e| {
                [
                    "feed-watch update failed (retry via link): ",
                    &e.to_string(),
                ]
                .concat()
            })?;
        }
    }

    let reply = UpdateReply {
        edge_id: id,
        message: t(
            locale,
            "guardian.updated",
            &[("guardian", &guardian_sub), ("child", &parsed.child_id)],
        ),
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guardianship::link;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{read, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.guardianship.link:call".into(),
                "mcp:care.guardianship.update:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn update_edits_flags_without_touching_liveness() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");

        link::run(
            &cp,
            &a,
            r#"{"guardian_sub":"user:sam","child_id":"child:leo","relationship":"father","can_pickup":false}"#,
        )
        .await
        .expect("link");

        run(
            &cp,
            &a,
            r#"{"guardian_sub":"user:sam","child_id":"child:leo","can_pickup":true,"receives_billing":true}"#,
        )
        .await
        .expect("update");

        let row = read(&store, "acme", "guardianship", "user:sam::child:leo")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row["can_pickup"], true);
        assert_eq!(row["receives_billing"], true);
        assert_eq!(
            row["live"], true,
            "liveness untouched by a flag-only update"
        );
    }

    #[tokio::test]
    async fn update_of_a_missing_edge_is_not_found() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let a = admin(&key, "acme");
        let res = run(
            &cp,
            &a,
            r#"{"guardian_sub":"user:x","child_id":"child:y","can_pickup":true}"#,
        )
        .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not found"));
    }
}
