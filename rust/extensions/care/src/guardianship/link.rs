//! `care.guardianship.link` — admin links a guardian to a child (the edge
//! that grants reach). Cap: `mcp:care.guardianship.link:call`. Admin-only.
//!
//! ## Era-2 grant derivation (Step C — the load-bearing transaction)
//!
//! The edge is the authz source of truth, and in era 2 the guardian's reach
//! is a SCOPED GRANT derived from the edge (`care-authz-scope.md` §"Era 2").
//! This verb writes the edge AND derives the grant — **edge AND grant, or
//! neither**: if the grant derivation fails, the edge write is rolled back
//! (a live edge with no grant is a lockout; the inverse — a grant with no
//! edge — is the leak `unlink` prevents). Ordering: write the edge first,
//! derive the grant second; on grant failure, archive the edge back out so
//! the store never carries an ungranted live edge.
//!
//! When no host-callback client is present (era-1 fallback), the edge write
//! alone is the reach source — the chokepoint resolves from it directly.

use lb_auth::Principal;

use crate::authz::{assert_reach, grant, Chokepoint, RecordError};
use crate::center::Locale;
use crate::guardianship::{edge_id, EdgeFlags, Guardianship, GuardianshipError, Relationship};
use crate::i18n::t;

/// The verb input — the pair, the relationship, and the five flags.
#[derive(Debug, serde::Deserialize)]
pub struct LinkInput {
    pub guardian_sub: String,
    pub child_id: String,
    /// The relationship key (`"mother"`, …) — parsed to the enum.
    pub relationship: String,
    #[serde(default)]
    pub can_pickup: bool,
    #[serde(default)]
    pub receives_daily_feed: bool,
    #[serde(default)]
    pub receives_billing: bool,
    #[serde(default)]
    pub emergency_contact: bool,
    #[serde(default)]
    pub custody_notes: Option<String>,
    /// Milestone 09 — messaging access to the child's channel (a DISTINCT flag
    /// from `receives_daily_feed`; the reconciler derives channel membership
    /// from it).
    #[serde(default)]
    pub receives_messaging: bool,
    /// The locale to render the confirmation string in (the admin's, or the
    /// workspace default). Defaults to `en`.
    #[serde(default)]
    pub locale: Option<String>,
}

/// The verb reply — the edge id + a localized confirmation.
#[derive(Debug, serde::Serialize)]
pub struct LinkReply {
    pub edge_id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: LinkInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.guardianship.link input: {e}"))?;

    if parsed.guardian_sub.is_empty() {
        return Err(format!(
            "{}",
            GuardianshipError::MissingField("guardian_sub")
        ));
    }
    if parsed.child_id.is_empty() {
        return Err(format!("{}", GuardianshipError::MissingField("child_id")));
    }
    let relationship = Relationship::parse(&parsed.relationship).map_err(|e| format!("{e}"))?;
    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // Canonicalise the guardian subject to the auth-subject form (`user:<x>`)
    // the reach path keys on — so the edge id, the era-1 lookup
    // (`principal.sub()`), and the era-2 grant subject all address the SAME
    // identity (a bare id would parse-reject in `grants.assign` AND miss the
    // era-1 edge read). See `authz::canonical_subject`.
    let guardian_sub = crate::authz::canonical_subject(&parsed.guardian_sub);

    let id = edge_id(&guardian_sub, &parsed.child_id);
    let edge = Guardianship {
        guardian_sub: guardian_sub.clone(),
        child_id: parsed.child_id.clone(),
        relationship,
        live: true,
        flags: EdgeFlags {
            can_pickup: parsed.can_pickup,
            receives_daily_feed: parsed.receives_daily_feed,
            receives_billing: parsed.receives_billing,
            emergency_contact: parsed.emergency_contact,
            custody_notes: parsed.custody_notes,
            receives_messaging: parsed.receives_messaging,
        },
    };
    let value = serde_json::to_value(&edge).map_err(|e| format!("serialize edge: {e}"))?;

    // 1) Write the edge FIRST (first-write; a duplicate pair conflicts).
    cp.records()
        .create("guardianship", &id, &value)
        .await
        .map_err(|e| match e {
            RecordError::Conflict => format!("{}", GuardianshipError::AlreadyExists(id.clone())),
            RecordError::Store(s) => {
                format!("{}: {s}", GuardianshipError::StoreDenied("link".into()))
            }
        })?;

    // 2) Derive the scoped reach grant (era 2). If it fails, ROLL BACK the
    //    edge — the store must never hold a live edge without its grant
    //    (a lockout). Era-1 fallback (no client): the edge alone is reach,
    //    nothing to derive.
    if let Some(reach) = cp.reach() {
        if let Err(e) = grant::derive_reach(reach.client(), &guardian_sub, &parsed.child_id).await {
            // Roll the edge back so edge-and-grant stays all-or-nothing. If the
            // rollback ITSELF fails, surface the divergence LOUDLY (a live edge
            // with no grant = a lockout the admin must know about) via the typed
            // `Diverged` error rather than swallow it. The words live in the
            // error's Display (records.rs); only the dynamic id + cause pass here.
            let err = if cp.records().delete("guardianship", &id).await.is_ok() {
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

    // 3) Grant channel membership (milestone 09) iff the edge opts into
    //    messaging. A missing channel grant is a LOCKOUT (the guardian can't read
    //    a channel they're entitled to), not a leak — so unlike the reach grant it
    //    does NOT roll back the edge; it is surfaced (returned) for retry via the
    //    healing sweep. Full member (post + read) on the child channel. Best-effort
    //    no-op without a host client (era-1/test path).
    if parsed.receives_messaging {
        if let Some(client) = cp.host_client() {
            let child_ch = crate::messaging::child_channel(&parsed.child_id);
            crate::messaging::reconcile::grant_membership(
                Some(client),
                &child_ch,
                &guardian_sub,
                crate::messaging::ChannelRole::Full,
            )
            .await
            .map_err(|e| format!("channel membership grant failed (retry via reconcile): {e}"))?;
        }
    }

    // Admin audit through the chokepoint (one audit point — link is
    // admin-gated at the wall; this records the per-call trail on the child).
    let _ = assert_reach(cp, principal, &parsed.child_id).await;

    let reply = LinkReply {
        edge_id: id,
        message: t(
            locale,
            "guardian.linked",
            &[
                ("guardian", &guardian_sub),
                ("child", &parsed.child_id),
                (
                    "relationship",
                    t(
                        locale,
                        &format!("relationship.{}", relationship.as_key()),
                        &[],
                    )
                    .as_str(),
                ),
            ],
        ),
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authz::assert_reach;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{read, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.guardianship.link:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn link_writes_a_live_edge_with_flags() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme"); // era-1 (no client)
        let p = admin(&key, "acme");

        let out = run(
            &cp,
            &p,
            r#"{"guardian_sub":"user:sam","child_id":"child:leo","relationship":"father","can_pickup":true,"receives_daily_feed":true}"#,
        )
        .await
        .expect("link ok");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["edge_id"], "user:sam::child:leo");

        // The edge landed live with the flags.
        let row = read(&store, "acme", "guardianship", "user:sam::child:leo")
            .await
            .unwrap()
            .expect("edge present");
        assert_eq!(row["live"], true);
        assert_eq!(row["can_pickup"], true);
        assert_eq!(row["receives_daily_feed"], true);
        assert_eq!(row["relationship"], "father");

        // Era-1 reach resolves through the edge immediately.
        let sam = super::tests::member(&key, "user:sam", "acme");
        assert_reach(&cp, &sam, "child:leo").await.expect("reach");
    }

    #[tokio::test]
    async fn link_is_first_write_a_duplicate_pair_conflicts() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let input = r#"{"guardian_sub":"user:sam","child_id":"child:leo","relationship":"father"}"#;
        run(&cp, &p, input).await.expect("first");
        assert!(run(&cp, &p, input).await.is_err(), "duplicate conflicts");
    }

    #[tokio::test]
    async fn link_rejects_unknown_relationship() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(
            &cp,
            &p,
            r#"{"guardian_sub":"user:sam","child_id":"child:leo","relationship":"cousin"}"#,
        )
        .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("invalid relationship"));
    }

    /// A `Member` principal for the reach assertion (shared with sibling
    /// verb tests via `super::tests::member`).
    pub(crate) fn member(signing: &SigningKey, sub: &str, ws: &str) -> Principal {
        let claims = Claims {
            sub: sub.into(),
            ws: ws.into(),
            role: Role::Member,
            caps: vec!["mcp:care.ping:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }
}
