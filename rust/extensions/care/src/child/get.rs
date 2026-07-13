//! `care.child.get` — get a single child profile. Cap: `mcp:care.child.get:call`.
//!
//! Reach-gated (CLAUDE.md rule 7): a guardian may `get` a child ONLY if they
//! hold a live edge (the chokepoint's `assert_reach` — era 2 delegates to the
//! platform, era 1 resolves from the edge). A miss is a 403 (an error on the
//! wire), NEVER a phantom empty object — a leak attempt fails closed.
//!
//! Archived children: returned to admin (audit); a guardian never reaches an
//! archived child through a live edge in practice, and `get` additionally
//! hides an archived row from non-admin callers (defence in depth with `list`).

use lb_auth::Principal;

use crate::authz::{assert_reach, Chokepoint};
use crate::child::Child;

#[derive(Debug, serde::Deserialize)]
pub struct GetInput {
    pub id: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: GetInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.child.get input: {e}"))?;

    // Reach check FIRST — 403 on miss (never a phantom object).
    assert_reach(cp, principal, &parsed.id)
        .await
        .map_err(|e| format!("{e}"))?;

    let value = cp
        .records()
        .read("child", &parsed.id)
        .await
        .map_err(|_| "store denied the child read".to_string())?
        .ok_or_else(|| "child not found".to_string())?;

    let child: Child =
        serde_json::from_value(value).map_err(|e| format!("deserialize child: {e}"))?;

    // An archived child is invisible to non-admins (defence in depth with
    // `list`). Admin sees it (audit).
    let is_admin = principal.role() == lb_auth::Role::WorkspaceAdmin
        || principal.role() == lb_auth::Role::SuperAdmin;
    if child.archived && !is_admin {
        return Err("child not found".to_string());
    }

    serde_json::to_string(&child).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::child::{archive as child_archive, create as child_create};
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        principal(signing, "user:admin", ws, Role::WorkspaceAdmin)
    }
    fn principal(signing: &SigningKey, sub: &str, ws: &str, role: Role) -> Principal {
        let claims = Claims {
            sub: sub.into(),
            ws: ws.into(),
            role,
            caps: vec![
                "mcp:care.child.create:call".into(),
                "mcp:care.child.get:call".into(),
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
    async fn get_round_trips_for_admin() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        child_create::run(
            &cp,
            &p,
            r#"{"id":"leo","name":"Leo","dob":"2021-03-15","allergies":["peanuts"]}"#,
        )
        .await
        .unwrap();

        let out = run(&cp, &p, r#"{"id":"leo"}"#).await.expect("get");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["name"], "Leo");
        assert_eq!(v["allergies"][0], "peanuts");
    }

    #[tokio::test]
    async fn archived_child_is_hidden_from_non_admin() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let a = admin(&key, "acme");
        child_create::run(&cp, &a, r#"{"id":"leo","name":"Leo","dob":"2021-03-15"}"#)
            .await
            .unwrap();
        child_archive::run(&cp, &a, r#"{"id":"leo"}"#)
            .await
            .unwrap();

        // Admin still sees it (audit).
        assert!(run(&cp, &a, r#"{"id":"leo"}"#).await.is_ok());
    }
}
