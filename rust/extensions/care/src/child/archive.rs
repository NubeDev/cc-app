//! `care.child.archive` — admin soft-deletes a child profile (archive, NEVER
//! hard-delete — retention). Cap: `mcp:care.child.archive:call`. Admin-only.
//!
//! Sets `archived = true`. An archived child is INVISIBLE to guardians (the
//! `list` verb filters archived rows from non-admin reads) and RECOVERABLE by
//! admin (`restore: true` flips it back). The record is never removed from
//! the store (`03-enrollment.md` exit gate: "invisible to guardians,
//! recoverable by admin").

use lb_auth::Principal;

use crate::authz::{assert_reach, Chokepoint};
use crate::center::Locale;
use crate::child::ChildError;
use crate::i18n::t;

#[derive(Debug, serde::Deserialize)]
pub struct ArchiveInput {
    pub id: String,
    /// `true` ⇒ un-archive (admin recovery). Defaults to archiving.
    #[serde(default)]
    pub restore: bool,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ArchiveReply {
    pub id: String,
    pub archived: bool,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: ArchiveInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.child.archive input: {e}"))?;
    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    assert_reach(cp, principal, &parsed.id)
        .await
        .map_err(|e| format!("{e}"))?;

    let mut row = cp
        .records()
        .read("child", &parsed.id)
        .await
        .map_err(|_| format!("{}", ChildError::StoreDenied("archive read".into())))?
        .ok_or_else(|| format!("{}", ChildError::NotFound(parsed.id.clone())))?;

    let archived = !parsed.restore;

    // Archive → stop-posts (milestone 10): BEFORE flipping the flag, capture the
    // child channel's current members so we can revoke their posting/reading —
    // once `archived = true` the derivation returns an empty set (the channel is
    // frozen), so we must read the live members first. Best-effort + surfaced (a
    // surviving grant on an archived child is a stale post surface); no host
    // client (era-1/test) ⇒ the list is empty and this is a no-op. Only on the
    // archive transition; `restore` re-derives via a normal reconcile.
    let members_to_freeze = if archived {
        crate::authz::channel_members(cp, &crate::authz::ChannelTarget::Child(&parsed.id)).await
    } else {
        Vec::new()
    };

    row["archived"] = serde_json::Value::Bool(archived);
    cp.records()
        .write("child", &parsed.id, &row)
        .await
        .map_err(|e| format!("{}: {e}", ChildError::StoreDenied("archive write".into())))?;

    if archived {
        if let Some(client) = cp.host_client() {
            let channel = crate::messaging::child_channel(&parsed.id);
            for m in &members_to_freeze {
                crate::messaging::reconcile::revoke_membership(Some(client), &channel, &m.subject)
                    .await
                    .map_err(|e| {
                        ["archive stop-posts revoke failed (retry): ", &e.to_string()].concat()
                    })?;
            }
        }
    }

    let name = row.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let key = if archived {
        "child.archived"
    } else {
        "child.created"
    };
    let reply = ArchiveReply {
        id: parsed.id,
        archived,
        message: t(locale, key, &[("name", name)]),
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::child::create as child_create;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{read, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.child.create:call".into(),
                "mcp:care.child.archive:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn archive_then_restore_round_trips() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        child_create::run(&cp, &p, r#"{"id":"leo","name":"Leo","dob":"2021-03-15"}"#)
            .await
            .unwrap();

        run(&cp, &p, r#"{"id":"leo"}"#).await.expect("archive");
        let row = read(&store, "acme", "child", "leo").await.unwrap().unwrap();
        assert_eq!(row["archived"], true, "record retained but archived");

        run(&cp, &p, r#"{"id":"leo","restore":true}"#)
            .await
            .expect("restore");
        let row = read(&store, "acme", "child", "leo").await.unwrap().unwrap();
        assert_eq!(row["archived"], false, "admin recovered it");
    }

    /// Archive → stop-posts (milestone 10): an ARCHIVED child's channel derives
    /// NO members, so a healing sweep grants nobody and no post/read surface
    /// survives. A linked messaging guardian is a member BEFORE archive and gone
    /// AFTER — the freeze. (The live grant-revoke round-trip is a live-node concern;
    /// the derivation is the leak-critical invariant and is unit-tested here.)
    #[tokio::test]
    async fn an_archived_childs_channel_derives_no_members() {
        use crate::authz::{channel_members, ChannelTarget};
        use crate::guardianship::link as guardianship_link;

        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        child_create::run(&cp, &p, r#"{"id":"leo","name":"Leo","dob":"2021-03-15"}"#)
            .await
            .unwrap();
        // A live, messaging-enabled guardian ⇒ a channel member while active.
        guardianship_link::run(
            &cp,
            &p,
            r#"{"guardian_sub":"user:ana","child_id":"leo","relationship":"mother","receives_messaging":true}"#,
        )
        .await
        .expect("link");

        let before = channel_members(&cp, &ChannelTarget::Child("leo")).await;
        assert!(
            before.iter().any(|m| m.subject == "user:ana"),
            "an active child's messaging guardian is a channel member: {before:?}"
        );

        run(&cp, &p, r#"{"id":"leo"}"#).await.expect("archive");

        let after = channel_members(&cp, &ChannelTarget::Child("leo")).await;
        assert!(
            after.is_empty(),
            "an ARCHIVED child's channel must derive NO members (frozen): {after:?}"
        );
    }

    #[tokio::test]
    async fn archive_of_a_missing_child_is_not_found() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(&cp, &p, r#"{"id":"ghost"}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not found"));
    }
}
