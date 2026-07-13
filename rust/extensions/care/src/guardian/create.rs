//! `care.guardian.create` — admin creates a guardian record (records-before-
//! accounts: the record exists before the person has an account; an invite
//! binds a `sub` later — milestone 05). Cap: `mcp:care.guardian.create:call`.
//! Admin-only.
//!
//! Validate id → validate name non-empty → validate email shape (an invite
//! target — a typo parks the invite) → parse locale (default `en`) → first-
//! write. All validation before the store call so a malformed record never
//! lands; a duplicate id ⇒ `AlreadyExists` (create is first-write, not upsert).

use lb_auth::Principal;

use crate::authz::{assert_reach, Chokepoint, RecordError};
use crate::center::Locale;
use crate::guardian::{validate_email, Guardian, GuardianError};
use crate::i18n::t;

#[derive(Debug, serde::Deserialize)]
pub struct CreateInput {
    /// The guardian's stable id (a slug the admin picks; recorded verbatim).
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub phone: Option<String>,
    pub email: String,
    /// Wire form (`"en"` | `"es"`); parsed via [`Locale::parse`], defaulting
    /// to `en` when absent.
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CreateReply {
    pub id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: CreateInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.guardian.create input: {e}"))?;

    // Validate id (durability guard).
    if parsed.id.is_empty() || parsed.id.len() > 64 {
        return Err(format!("{}", GuardianError::InvalidId(parsed.id.clone())));
    }
    // Name is required (never translated — the admin-entered display name).
    if parsed.name.trim().is_empty() {
        return Err(format!("{}", GuardianError::MissingField("name")));
    }
    // Email is the invite address — reject the obvious garbage now.
    validate_email(&parsed.email).map_err(|e| format!("{e}"))?;

    // Locale defaults to `en` when absent (records-before-accounts: the invite
    // renders in this locale before the account exists — CLAUDE.md rule 8).
    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // Build the durable record (orchestrator-owned shape — see records.rs).
    // `sub: None` — the account is pending until the invite is accepted.
    let guardian = Guardian {
        name: parsed.name.clone(),
        email: parsed.email,
        phone: parsed.phone,
        sub: None,
        locale,
    };
    let value = serde_json::to_value(&guardian).map_err(|e| format!("serialize guardian: {e}"))?;

    // First-write (Conflict ⇒ AlreadyExists — create is first-write, not
    // upsert; the host maps this typed surface to the MCP error shape).
    cp.records()
        .create("guardian", &parsed.id, &value)
        .await
        .map_err(|e| match e {
            RecordError::Conflict => {
                format!("{}", GuardianError::AlreadyExists(parsed.id.clone()))
            }
            RecordError::Store(s) => {
                format!("{}: {s}", GuardianError::StoreDenied("create".into()))
            }
        })?;

    // Admin audit through the chokepoint (one audit point).
    let _ = assert_reach(cp, principal, &parsed.id).await;

    let reply = CreateReply {
        message: t(locale, "guardian.created", &[("name", &guardian.name)]),
        id: parsed.id,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{read, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec!["mcp:care.guardian.create:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn create_writes_a_guardian_record() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        let out = run(
            &cp,
            &p,
            r#"{"id":"sam","name":"Sam","email":"sam@example.com","locale":"es"}"#,
        )
        .await
        .expect("create");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["id"], "sam");
        // Message resolves through the es catalog (rule 8 — es from day one).
        assert_eq!(v["message"], "Tutor Sam agregado.");

        // The record landed under the workspace namespace, `sub` pending.
        let row = read(&store, "acme", "guardian", "sam")
            .await
            .unwrap()
            .expect("present");
        assert_eq!(row["name"], "Sam");
        assert_eq!(row["email"], "sam@example.com");
        assert_eq!(row["sub"], serde_json::Value::Null);
        assert_eq!(row["locale"], "es");
    }

    #[tokio::test]
    async fn create_rejects_a_bad_email() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let res = run(
            &cp,
            &p,
            r#"{"id":"sam","name":"Sam","email":"not-an-email"}"#,
        )
        .await;
        assert!(res.is_err(), "a malformed email must fail");
        assert!(res.unwrap_err().contains("invalid email"));
    }

    #[tokio::test]
    async fn create_rejects_an_empty_name() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let res = run(
            &cp,
            &p,
            r#"{"id":"sam","name":"  ","email":"sam@example.com"}"#,
        )
        .await;
        assert!(res.is_err(), "an empty name must fail");
        assert!(res.unwrap_err().contains("missing required field"));
    }

    #[tokio::test]
    async fn create_is_first_write_a_duplicate_id_conflicts() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");

        let input = r#"{"id":"sam","name":"Sam","email":"sam@example.com"}"#;
        run(&cp, &p, input).await.expect("first write");

        let res = run(&cp, &p, input).await;
        assert!(res.is_err(), "second create of the same id must fail");
        assert!(res.unwrap_err().contains("already exists"));
    }
}
