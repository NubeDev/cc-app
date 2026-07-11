//! `care.guardian.list` + `care.guardian.get` — admin reads over guardian
//! records. Caps: `mcp:care.guardian.list:call` / `mcp:care.guardian.get:call`.
//!
//! Guardian reads are ADMIN-ONLY in this milestone: guardians and staff do
//! NOT list or fetch other guardians (there is no cross-guardian reach in the
//! product). A non-admin `list` returns the empty list (rule-7 deny semantic
//! for lists — never an error); a non-admin `get` is a denial after touching
//! the chokepoint.
//!
//! Both read paths live in this one file (the barrel declares only `create`
//! and `list`): `run` is the list-many, `get` is the fetch-one.

use lb_auth::Principal;

use crate::authz::{reachable_children, Chokepoint};
use crate::guardian::Guardian;

/// True for the workspace/super admin roles that may read guardian records.
fn is_admin(principal: &Principal) -> bool {
    principal.role() == lb_auth::Role::WorkspaceAdmin
        || principal.role() == lb_auth::Role::SuperAdmin
}

/// `care.guardian.list` — list every guardian in the workspace (admin only).
/// A non-admin reaches no guardians ⇒ empty list (rule-7 deny for lists).
pub async fn run(cp: &Chokepoint, principal: &Principal, _input: &str) -> Result<String, String> {
    if !is_admin(principal) {
        // Non-admin reaches no guardian records — return empty (not an error).
        let _ = reachable_children(cp, principal).await; // touch the chokepoint
        return Ok("[]".to_string());
    }

    // Admin path: list every guardian row, schema-stable SurrealQL.
    let mut resp = cp
        .store
        .query_ws(&cp.ws, "SELECT * FROM guardian", vec![])
        .await
        .map_err(|e| format!("store denied the guardian list: {e}"))?;

    // `SELECT *` returns each row as `{"data": <our record>, "id": <thing>}`;
    // take by field name ("data") so the row deserializer knows the shape.
    let data_rows: Vec<serde_json::Value> = resp
        .take::<Vec<serde_json::Value>>((0, "data"))
        .unwrap_or_default();
    let mut out: Vec<Guardian> = Vec::new();
    for row in data_rows {
        if let Ok(g) = serde_json::from_value::<Guardian>(row) {
            out.push(g);
        }
    }
    serde_json::to_string(&out).map_err(|e| format!("serialize reply: {e}"))
}

#[derive(Debug, serde::Deserialize)]
pub struct GetInput {
    pub id: String,
}

/// `care.guardian.get` — fetch a single guardian record by id (admin only).
/// A non-admin is denied after touching the chokepoint (never a phantom
/// object — a leak attempt is a denial, not a 200-`{}`; CLAUDE.md rule 7).
pub async fn get(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: GetInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.guardian.get input: {e}"))?;

    if !is_admin(principal) {
        // Non-admin has no reach to guardian records — touch the chokepoint,
        // then deny (never leak whether the id exists).
        let _ = reachable_children(cp, principal).await;
        return Err(format!(
            "{}",
            crate::guardian::GuardianError::NotFound(parsed.id)
        ));
    }

    let row = lb_store::read(&cp.store, &cp.ws, "guardian", &parsed.id)
        .await
        .map_err(|_| "store denied the guardian read".to_string())?;
    let value = row.ok_or_else(|| {
        format!(
            "{}",
            crate::guardian::GuardianError::NotFound(parsed.id.clone())
        )
    })?;

    let guardian: Guardian =
        serde_json::from_value(value).map_err(|e| format!("deserialize guardian: {e}"))?;
    serde_json::to_string(&guardian).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guardian::create as guardian_create;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.guardian.create:call".into(),
                "mcp:care.guardian.list:call".into(),
                "mcp:care.guardian.get:call".into(),
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
            caps: vec![
                "mcp:care.guardian.list:call".into(),
                "mcp:care.guardian.get:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    async fn seed(cp: &Chokepoint, p: &Principal) {
        guardian_create::run(
            cp,
            p,
            r#"{"id":"sam","name":"Sam","email":"sam@example.com","locale":"en"}"#,
        )
        .await
        .unwrap();
        guardian_create::run(
            cp,
            p,
            r#"{"id":"ana","name":"Ana","email":"ana@example.com","locale":"es"}"#,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn admin_lists_all_guardians() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        seed(&cp, &p).await;

        let out = run(&cp, &p, "").await.unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(v.len(), 2);
        let names: Vec<&str> = v.iter().map(|g| g["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"Sam"));
        assert!(names.contains(&"Ana"));
    }

    #[tokio::test]
    async fn non_admin_gets_an_empty_list() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let admin_p = admin(&key, "acme");
        seed(&cp, &admin_p).await;

        // Guardian reads are admin-only — staff sees nothing (empty, not error).
        let staff_p = staff(&key, "acme");
        let out = run(&cp, &staff_p, "").await.unwrap();
        assert_eq!(out, "[]", "non-admin list ⇒ empty, not error");
    }

    #[tokio::test]
    async fn get_round_trips_for_admin() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        seed(&cp, &p).await;

        let out = get(&cp, &p, r#"{"id":"sam"}"#).await.expect("get");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["name"], "Sam");
        assert_eq!(v["email"], "sam@example.com");
        assert_eq!(v["locale"], "en");
    }

    #[tokio::test]
    async fn get_returns_not_found_for_missing_id() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let res = get(&cp, &p, r#"{"id":"missing"}"#).await;
        assert!(res.is_err(), "missing id ⇒ error");
        assert!(res.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn non_admin_get_is_denied() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let admin_p = admin(&key, "acme");
        seed(&cp, &admin_p).await;

        // A staff principal must NOT fetch a guardian even though it exists —
        // guardian data is admin-only this milestone (rule 7 isolation).
        let staff_p = staff(&key, "acme");
        let res = get(&cp, &staff_p, r#"{"id":"sam"}"#).await;
        assert!(res.is_err(), "non-admin get must be denied");
    }
}
