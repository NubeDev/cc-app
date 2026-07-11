//! `care.center.get` — get a single center record.
//!
//! Cap: `mcp:care.center.get:call`. The authz chokepoint narrows by reach
//! (admin wildcard; staff/guardian via their room's center). Returns
//! `null` for the wire (the host maps to `ToolError::Denied` after the
//! chokepoint denies) — NEVER a phantom empty object, so a leak attempt
//! is a 403 not a 200-`{}` (CLAUDE.md rule 7).
//!
//! Archived centers are returned as-is (admin-only audit). The list
//! verb is the one that filters `archived` from guardian reads — the
//! get verb stays a thin pass-through to the store.

use lb_auth::Principal;
use lb_store::read;

use crate::authz::{assert_reach, Chokepoint};
use crate::center::Center;

#[derive(Debug, serde::Deserialize)]
pub struct GetInput {
    pub id: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: GetInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.center.get input: {e}"))?;

    // Reach check first (403 on miss — never a phantom empty object).
    // For an admin this is the audited role pass; for staff/guardian
    // it's the room/edge filter (the chokepoint's era-1 path resolves
    // from the records).
    assert_reach(cp, principal, &parsed.id)
        .await
        .map_err(|e| format!("{e}"))?;

    let row = read(&cp.store, &cp.ws, "center", &parsed.id)
        .await
        .map_err(|_| "store denied the center read".to_string())?;
    let value = row.ok_or_else(|| "center not found".to_string())?;

    // The durable record is wrapped under `data` by `lb_store::create`
    // — unwrap it for the wire shape (so the UI sees the Center fields
    // at the top level, not the `data` envelope).
    let center: Center =
        serde_json::from_value(value).map_err(|e| format!("deserialize center: {e}"))?;

    serde_json::to_string(&center).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authz::Chokepoint;
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
                "mcp:care.center.get:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn get_round_trips_after_create() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        center_create::run(
            &cp,
            &p,
            r#"{"id":"main","name":"Main","default_locale":"en"}"#,
        )
        .await
        .expect("create");

        let out = run(&cp, &p, r#"{"id":"main"}"#).await.expect("get");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["name"], "Main");
        assert_eq!(v["default_locale"], "en");
    }

    #[tokio::test]
    async fn get_returns_not_found_for_missing_id() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let res = run(&cp, &p, r#"{"id":"missing"}"#).await;
        assert!(res.is_err(), "missing id ⇒ error");
        assert!(res.unwrap_err().contains("not found"));
    }
}
