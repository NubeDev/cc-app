//! `care.center.create` — admin creates a center record.
//!
//! Cap: `mcp:care.center.create:call`. Admin-only (staff/guardian 403).
//! Path: validate id → validate locale → upsert FIRST-WRITE (a duplicate
//! id ⇒ `AlreadyExists`, mirroring `lb_store::create`'s first-settle
//! semantic — CLAUDE.md rule 4 / testing-scope §"first-settle").
//!
//! All validation happens BEFORE the store call so the verb never writes
//! garbage; a malformed id / unknown locale fails fast on the first
//! line of input.

use lb_auth::Principal;

use crate::authz::{assert_reach, Chokepoint, RecordError};
use crate::center::{Center, CenterError, Locale};

/// The verb body's input. Opaque JSON the host marshals — the dispatcher
/// hands it as a JSON string.
#[derive(Debug, serde::Deserialize)]
pub struct CreateInput {
    /// The center's stable id (a slug the admin picks; recorded verbatim).
    /// Mirrors the durable record shape.
    pub id: String,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    /// Wire form (`"en"` | `"es"`); parsed via [`Locale::parse`].
    pub default_locale: String,
}

/// The verb body's reply. The minimal JSON shape the host hands back.
#[derive(Debug, serde::Serialize)]
pub struct CreateReply {
    pub id: String,
    pub name: String,
    pub default_locale: &'static str,
}

/// Run `care.center.create`. Returns opaque-JSON for the wire.
pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    // Parse input.
    let parsed: CreateInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.center.create input: {e}"))?;

    // Validate id (durability guard).
    if parsed.id.is_empty() || parsed.id.len() > 64 {
        return Err(format!(
            "{}: id must be 1..=64 chars (got {})",
            CenterError::InvalidId(parsed.id.clone()),
            parsed.id.len()
        ));
    }

    // Validate locale.
    let locale = Locale::parse(&parsed.default_locale).map_err(|e| format!("{e}"))?;

    // Build the durable record (orchestrator-owned shape — see records.rs).
    let center = Center {
        name: parsed.name,
        address: parsed.address,
        phone: parsed.phone,
        email: parsed.email,
        default_locale: locale,
        archived: false,
    };
    let value = serde_json::to_value(&center).map_err(|e| format!("serialize center: {e}"))?;

    // First-write (Conflict ⇒ AlreadyExists; the verb body's typed
    // surface — the host maps this to the MCP error shape).
    cp.records()
        .create("center", &parsed.id, &value)
        .await
        .map_err(|e| match e {
            RecordError::Conflict => {
                format!("{}", CenterError::AlreadyExists(parsed.id.clone()))
            }
            RecordError::Store(s) => {
                format!("{}: {s}", CenterError::StoreDenied("create".into()))
            }
        })?;

    // Audit the create (one audit point — admin is authorized by role
    // inside the chokepoint; this is the per-call trail). The admin
    // pass lives in `authz::assert_reach`'s role check; we re-touch it
    // here so every admin verb body has a single-line audit even when
    // the verb doesn't otherwise call the chokepoint.
    let _ = assert_reach(cp, principal, &parsed.id).await;

    let reply = CreateReply {
        id: parsed.id,
        name: center.name,
        default_locale: center.default_locale.as_str(),
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::Store;
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.center.create:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn create_writes_a_center_record() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        let out = run(
            &cp,
            &p,
            r#"{"id":"main","name":"Main Center","default_locale":"en"}"#,
        )
        .await
        .expect("ok");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["id"], "main");
        assert_eq!(v["name"], "Main Center");
        assert_eq!(v["default_locale"], "en");

        // The record landed in the store under the workspace namespace.
        let read = lb_store::read(&store, "acme", "center", "main")
            .await
            .unwrap()
            .expect("present");
        assert_eq!(read["name"], "Main Center");
    }

    #[tokio::test]
    async fn create_is_first_write_a_duplicate_id_conflicts() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let input = r#"{"id":"main","name":"A","default_locale":"en"}"#;
        run(&cp, &p, input).await.expect("first write");

        let res = run(&cp, &p, input).await;
        assert!(res.is_err(), "second create of the same id must fail");
        // The Conflict path formats as "center already exists: main".
        assert!(res.unwrap_err().contains("already exists"));
    }

    #[tokio::test]
    async fn create_rejects_unknown_locale() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let res = run(&cp, &p, r#"{"id":"main","name":"M","default_locale":"fr"}"#).await;
        assert!(res.is_err(), "fr is not in the launch set");
        // The locale error formats as "invalid locale: \"fr\"".
        assert!(res.unwrap_err().contains("invalid locale"));
    }
}
