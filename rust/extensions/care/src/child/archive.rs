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
    row["archived"] = serde_json::Value::Bool(archived);
    cp.records()
        .write("child", &parsed.id, &row)
        .await
        .map_err(|e| format!("{}: {e}", ChildError::StoreDenied("archive write".into())))?;

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
