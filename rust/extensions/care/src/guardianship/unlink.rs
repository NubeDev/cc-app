//! `care.guardianship.unlink` — admin removes a guardian↔child link. Cap:
//! `mcp:care.guardianship.unlink:call`. Admin-only.
//!
//! ## Era-2 grant removal (Step C — the mirror of `link`)
//!
//! Unlink must leave **neither** a live edge **nor** a scoped grant: a grant
//! surviving unlink is a cross-family LEAK — the existential bug
//! (`care-authz-scope.md` §"Risks"). This verb sets the edge `live = false`
//! (soft — retained for audit) AND revokes the derived scoped grant in the
//! same handler. If the grant revoke fails, the edge is restored to `live`
//! so the two never diverge (a live-looking edge whose grant is gone is a
//! lockout; better to fail loudly and retry than to half-apply).
//!
//! Era-1 fallback (no client): archiving the edge is the whole operation —
//! the chokepoint denies on `live == false` on the very next call.

use lb_auth::Principal;

use crate::authz::{grant, Chokepoint};
use crate::center::Locale;
use crate::guardianship::{edge_id, GuardianshipError};
use crate::i18n::t;

#[derive(Debug, serde::Deserialize)]
pub struct UnlinkInput {
    pub guardian_sub: String,
    pub child_id: String,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct UnlinkReply {
    pub edge_id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let _ = principal; // admin-gated at the wall; role audited in the chokepoint
    let parsed: UnlinkInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.guardianship.unlink input: {e}"))?;
    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // Canonicalise the subject to the auth-subject form so the edge id + the
    // era-2 grant subject match what `link` derived (see `authz::canonical_subject`).
    let guardian_sub = crate::authz::canonical_subject(&parsed.guardian_sub);
    let id = edge_id(&guardian_sub, &parsed.child_id);

    // Load the edge (must exist to unlink).
    let mut row = cp
        .records()
        .read("guardianship", &id)
        .await
        .map_err(|_| format!("{}", GuardianshipError::StoreDenied("unlink read".into())))?
        .ok_or_else(|| format!("{}", GuardianshipError::NotFound(id.clone())))?;

    // 1) Archive the edge (live → false). Retained for audit.
    row["live"] = serde_json::Value::Bool(false);
    cp.records()
        .write("guardianship", &id, &row)
        .await
        .map_err(|e| {
            format!(
                "{}: {e}",
                GuardianshipError::StoreDenied("unlink write".into())
            )
        })?;

    // 2) Revoke the derived scoped grant (era 2). If revoke fails, RESTORE
    //    the edge to live — edge-and-grant stays all-or-nothing, and a
    //    surviving grant never outlives a live-looking edge (fail loud).
    if let Some(reach) = cp.reach() {
        if let Err(e) = grant::remove_reach(reach.client(), &guardian_sub, &parsed.child_id).await {
            // Restore the edge to live so edge + grant never diverge. If the
            // restore ITSELF fails, surface it LOUDLY via the typed `Diverged`
            // error: a `live=false` edge with a SURVIVING grant is the
            // existential cross-family LEAK — the admin must retry, never a
            // silent swallow. Words live in the error Display (records.rs).
            row["live"] = serde_json::Value::Bool(true);
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

    // 3) Revoke the guardian's channel membership (milestone 09) in the same
    //    breath — an ex-partner reading the child channel is the same severity as
    //    a reach leak (`messaging-scope.md` §"Unlink = immediate removal"). This
    //    revokes BOTH the child channel and (best-effort) the room channel the
    //    child sits in. Idempotent (revoking an absent grant succeeds). A revoke
    //    fault is surfaced (returned) — a surviving channel grant is a leak — but
    //    the edge is already archived (reach denies), so we do not restore it: the
    //    channel revoke is retried by the healing path, not by resurrecting reach.
    if let Some(client) = cp.host_client() {
        let child_ch = crate::messaging::child_channel(&parsed.child_id);
        crate::messaging::reconcile::revoke_membership(Some(client), &child_ch, &guardian_sub)
            .await
            .map_err(|e| format!("channel membership revoke failed (retry unlink): {e}"))?;
    }

    let reply = UnlinkReply {
        edge_id: id,
        message: t(
            locale,
            "guardian.unlinked",
            &[("guardian", &guardian_sub), ("child", &parsed.child_id)],
        ),
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authz::assert_reach;
    use crate::guardianship::link;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.guardianship.link:call".into(),
                "mcp:care.guardianship.unlink:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    fn member(signing: &SigningKey, sub: &str, ws: &str) -> Principal {
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

    #[tokio::test]
    async fn unlink_archives_the_edge_and_denies_immediately() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme"); // era-1
        let a = admin(&key, "acme");

        link::run(
            &cp,
            &a,
            r#"{"guardian_sub":"user:ana","child_id":"child:leo","relationship":"mother"}"#,
        )
        .await
        .expect("link");

        let ana = member(&key, "user:ana", "acme");
        assert_reach(&cp, &ana, "child:leo")
            .await
            .expect("reach pre-unlink");

        run(
            &cp,
            &a,
            r#"{"guardian_sub":"user:ana","child_id":"child:leo"}"#,
        )
        .await
        .expect("unlink");

        // Immediate deny — no caching, no grace.
        assert_reach(&cp, &ana, "child:leo")
            .await
            .expect_err("deny after unlink");

        // The edge is retained (audit) but live == false.
        let row = lb_store::read(&store, "acme", "guardianship", "user:ana::child:leo")
            .await
            .unwrap()
            .expect("edge retained");
        assert_eq!(row["live"], false);
    }

    #[tokio::test]
    async fn unlink_of_a_missing_edge_is_not_found() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let a = admin(&key, "acme");
        let res = run(
            &cp,
            &a,
            r#"{"guardian_sub":"user:ghost","child_id":"child:none"}"#,
        )
        .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not found"));
    }
}
