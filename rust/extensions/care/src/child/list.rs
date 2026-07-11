//! `care.child.list` — list children, REACH-FILTERED (CLAUDE.md rule 7). Cap:
//! `mcp:care.child.list:call`.
//!
//! The list-verb deny semantic is **empty, never error** (`care-authz-scope.md`):
//! a guardian who reaches no children gets `[]`, not a 403. The reach set comes
//! from the chokepoint's `reachable_children` (era 2 → `authz.scope_filter`;
//! era 1 → the live edges). Admin gets the wildcard (`["*"]`) ⇒ all rows.
//!
//! Archived children are FILTERED OUT for non-admin callers (a guardian never
//! sees an archived child); admin sees them (the audit trail).

use lb_auth::Principal;
use lb_store::read;

use crate::authz::{reachable_children, Chokepoint};
use crate::child::Child;

pub async fn run(cp: &Chokepoint, principal: &Principal, _input: &str) -> Result<String, String> {
    let reach = reachable_children(cp, principal).await;
    let is_admin = reach.iter().any(|r| r == "*");

    let out: Vec<Child> = if is_admin {
        // Admin: every child in the workspace (archived included — the audit
        // trail). One scan.
        all_children(cp).await?
    } else {
        // Non-admin (rule 7): fetch ONLY the reached ids — one indexed read
        // per reached child, never the whole table (the scope's "push the ids
        // into the query" intent). The reach id equals the record id (the
        // guardianship edge's `child_id`), so a direct `read` addresses the
        // exact row; a miss just contributes nothing. Empty reach ⇒ empty
        // reply (never an error, never a leak).
        let mut acc = Vec::new();
        for id in &reach {
            if let Some(child) = read_child(cp, id).await? {
                // Archived children are invisible to non-admins.
                if !child.archived {
                    acc.push(child);
                }
            }
        }
        acc
    };
    serde_json::to_string(&out).map_err(|e| format!("serialize reply: {e}"))
}

/// Read one child record by its reach/record id, deserialized. `None` if
/// absent (a reached id whose record was archived-out or never created).
async fn read_child(cp: &Chokepoint, id: &str) -> Result<Option<Child>, String> {
    let value = read(&cp.store, &cp.ws, "child", id)
        .await
        .map_err(|e| format!("store denied the child read: {e}"))?;
    match value {
        Some(v) => serde_json::from_value::<Child>(v)
            .map(Some)
            .map_err(|e| format!("deserialize child: {e}")),
        None => Ok(None),
    }
}

/// Every child in the workspace (admin path). `SELECT data` deserializes each
/// row's record envelope — the same shape the store's `list` helper uses.
async fn all_children(cp: &Chokepoint) -> Result<Vec<Child>, String> {
    let mut resp = cp
        .store
        .query_ws(&cp.ws, "SELECT data FROM child", vec![])
        .await
        .map_err(|e| format!("store denied the child list: {e}"))?;
    let data_rows: Vec<serde_json::Value> =
        resp.take::<Vec<serde_json::Value>>((0, "data")).unwrap_or_default();
    let mut out = Vec::new();
    for row in data_rows {
        if let Ok(c) = serde_json::from_value::<Child>(row) {
            out.push(c);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::child::{archive as child_archive, create as child_create};
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.child.create:call".into(),
                "mcp:care.child.list:call".into(),
                "mcp:care.child.archive:call".into(),
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
            caps: vec!["mcp:care.child.list:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn admin_lists_all_children_incl_archived() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let a = admin(&key, "acme");
        child_create::run(&cp, &a, r#"{"id":"leo","name":"Leo","dob":"2021-03-15"}"#)
            .await
            .unwrap();
        child_create::run(&cp, &a, r#"{"id":"mia","name":"Mia","dob":"2020-06-01"}"#)
            .await
            .unwrap();
        child_archive::run(&cp, &a, r#"{"id":"mia"}"#).await.unwrap();

        let out = run(&cp, &a, "").await.unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(v.len(), 2, "admin sees archived too");
    }

    #[tokio::test]
    async fn guardian_with_no_reach_gets_empty_not_error() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let a = admin(&key, "acme");
        child_create::run(&cp, &a, r#"{"id":"leo","name":"Leo","dob":"2021-03-15"}"#)
            .await
            .unwrap();

        // A guardian with no edge reaches nothing ⇒ empty list (not error).
        let m = member(&key, "user:stranger", "acme");
        let out = run(&cp, &m, "").await.expect("empty, not error");
        assert_eq!(out, "[]");
    }
}
