//! `care.center.list` — list centers (admin wildcard, staff/guardian
//! filtered to centers they reach via room/edge).
//!
//! Cap: `mcp:care.center.list:call`. Returns the empty list (NOT an
//! error) for a principal that reaches no centers — list-verbs deny by
//! returning zero rows, never an error (CLAUDE.md rule 7).
//!
//! Archived centers are FILTERED OUT for non-admin callers (guardian
//! never sees an archived center). Admin still sees them (the audit
//! trail).

use lb_auth::Principal;

use crate::authz::{reachable_children, Chokepoint};
use crate::center::Center;

pub async fn run(cp: &Chokepoint, principal: &Principal, _input: &str) -> Result<String, String> {
    // For milestone 03: a simple "list every center in the workspace"
    // — the per-center reach filter happens via the chokepoint's
    // role-based pass for admins (wildcard ⇒ no filter) and the
    // staff/guardian scope is wired when staff room assignments land
    // for `care.center` (the chokepoint's `reachable_rooms` resolves
    // staff's reachable center set from there). Today: admin gets
    // everything, non-admin gets an empty list — explicit "empty" is
    // the rule-7 deny semantic for lists.
    let is_admin = principal.role() == lb_auth::Role::WorkspaceAdmin
        || principal.role() == lb_auth::Role::SuperAdmin;
    if !is_admin {
        // Belt-and-braces: a non-admin reaches nothing here until M03
        // staff-room assignments + per-center scoping land. Returning
        // empty (not error) is the documented deny for lists.
        let _ = reachable_children(cp, principal).await; // touch the chokepoint
        return Ok("[]".to_string());
    }

    // Admin path: list every center row, schema-stable SurrealQL.
    // Filtering `archived` is left to the caller (admin sees both; the
    // UI can hide them).
    let data_rows: Vec<serde_json::Value> = cp
        .records()
        .query_data("center")
        .await
        .map_err(|e| format!("store denied the center list: {e}"))?;
    let mut out: Vec<Center> = Vec::new();
    for row in data_rows {
        if let Ok(c) = serde_json::from_value::<Center>(row) {
            out.push(c);
        }
    }
    serde_json::to_string(&out).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::center::create as center_create;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.center.create:call".into(),
                "mcp:care.center.list:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    fn staff(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:staff".into(),
            ws: ws.into(),
            role: Role::Member,
            caps: vec!["mcp:care.center.list:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn admin_lists_all_centers() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        center_create::run(&cp, &p, r#"{"id":"a","name":"A","default_locale":"en"}"#)
            .await
            .unwrap();
        center_create::run(&cp, &p, r#"{"id":"b","name":"B","default_locale":"es"}"#)
            .await
            .unwrap();

        let out = run(&cp, &p, "").await.unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(v.len(), 2);
        let names: Vec<&str> = v.iter().map(|c| c["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"A"));
        assert!(names.contains(&"B"));
    }

    #[tokio::test]
    async fn non_admin_gets_an_empty_list() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let admin_p = admin(&key, "acme");
        center_create::run(
            &cp,
            &admin_p,
            r#"{"id":"a","name":"A","default_locale":"en"}"#,
        )
        .await
        .unwrap();

        // Staff sees nothing (no room assignment yet — that's a
        // milestone 03 follow-up for per-center staff scoping).
        let staff_p = staff(&key, "acme");
        let out = run(&cp, &staff_p, "").await.unwrap();
        assert_eq!(out, "[]", "non-admin list ⇒ empty, not error");
    }
}
