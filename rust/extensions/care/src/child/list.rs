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

use crate::authz::{reachable_children, Chokepoint};
use crate::child::Child;

pub async fn run(cp: &Chokepoint, principal: &Principal, _input: &str) -> Result<String, String> {
    let reach = reachable_children(cp, principal).await;
    let is_admin = reach.iter().any(|r| r == "*");

    // Each row carries its store `id` (`{ id, ...child }`) so the UI can
    // address a child by id (route, select, enroll) — the record body has none.
    let out: Vec<serde_json::Value> = if is_admin {
        // Admin: every child in the workspace (archived included — the audit
        // trail). One scan, id-carrying.
        let rows = cp
            .records()
            .query_rows("child")
            .await
            .map_err(|e| format!("store denied the child list: {e}"))?;
        rows.into_iter()
            .filter_map(|(id, v)| serde_json::from_value::<Child>(v.clone()).ok().map(|_| with_id(&id, v)))
            .collect()
    } else {
        // Non-admin (rule 7): fetch ONLY the reached ids — one indexed read
        // per reached child, never the whole table. The reach id equals the
        // record id, so a direct `read` addresses the exact row; a miss
        // contributes nothing. Empty reach ⇒ empty reply (never a leak).
        let mut acc = Vec::new();
        for id in &reach {
            if let Some((child, value)) = read_child(cp, id).await? {
                // Archived children are invisible to non-admins.
                if !child.archived {
                    acc.push(with_id(id, value));
                }
            }
        }
        acc
    };
    serde_json::to_string(&out).map_err(|e| format!("serialize reply: {e}"))
}

/// Merge the store `id` into a child record's `data` object → `{ id, ...child }`.
fn with_id(id: &str, mut data: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = data.as_object_mut() {
        obj.insert("id".to_string(), serde_json::Value::String(id.to_string()));
    }
    data
}

/// Read one child record by its reach/record id, returning BOTH the typed
/// child (for the archived check) and the raw value (to merge the id into).
/// `None` if absent (a reached id whose record was archived-out or never made).
async fn read_child(cp: &Chokepoint, id: &str) -> Result<Option<(Child, serde_json::Value)>, String> {
    let value = cp
        .records()
        .read("child", id)
        .await
        .map_err(|e| format!("store denied the child read: {e}"))?;
    match value {
        Some(v) => serde_json::from_value::<Child>(v.clone())
            .map(|c| Some((c, v)))
            .map_err(|e| format!("deserialize child: {e}")),
        None => Ok(None),
    }
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
        child_archive::run(&cp, &a, r#"{"id":"mia"}"#)
            .await
            .unwrap();

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
