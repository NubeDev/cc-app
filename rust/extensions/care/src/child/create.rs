//! `care.child.create` — admin creates a child profile (the safety record).
//! Cap: `mcp:care.child.create:call`. Admin-only.
//!
//! Validate id → validate DOB (a safety field, fail hard) → validate the
//! allergy list is present-or-empty (never garbage) → first-write. All
//! validation before the store call so a malformed profile never lands.

use lb_auth::Principal;
use lb_store::{create as store_create, StoreError};

use crate::authz::{assert_reach, Chokepoint};
use crate::center::Locale;
use crate::child::{validate_dob, Child, ChildError, EmergencyContact, PickupPerson};
use crate::i18n::t;

#[derive(Debug, serde::Deserialize)]
pub struct CreateInput {
    pub id: String,
    pub name: String,
    pub dob: String,
    #[serde(default)]
    pub room_id: Option<String>,
    #[serde(default)]
    pub allergies: Vec<String>,
    #[serde(default)]
    pub medical_notes: Option<String>,
    #[serde(default)]
    pub immunizations: Vec<String>,
    #[serde(default)]
    pub emergency_contacts: Vec<EmergencyContact>,
    #[serde(default)]
    pub authorized_pickups: Vec<PickupPerson>,
    #[serde(default)]
    pub photo_consent: bool,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CreateReply {
    pub id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: CreateInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.child.create input: {e}"))?;

    if parsed.id.is_empty() || parsed.id.len() > 64 {
        return Err(format!("{}", ChildError::InvalidId(parsed.id.clone())));
    }
    if parsed.name.trim().is_empty() {
        return Err(format!("{}", ChildError::MissingField("name")));
    }
    validate_dob(&parsed.dob).map_err(|e| format!("{e}"))?;
    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    let child = Child {
        name: parsed.name.clone(),
        dob: parsed.dob,
        room_id: parsed.room_id,
        allergies: parsed.allergies,
        medical_notes: parsed.medical_notes,
        immunizations: parsed.immunizations,
        emergency_contacts: parsed.emergency_contacts,
        authorized_pickups: parsed.authorized_pickups,
        photo_consent: parsed.photo_consent,
        archived: false,
    };
    let value = serde_json::to_value(&child).map_err(|e| format!("serialize child: {e}"))?;

    store_create(&cp.store, &cp.ws, "child", &parsed.id, &value)
        .await
        .map_err(|e| match e {
            StoreError::Conflict => format!("{}", ChildError::AlreadyExists(parsed.id.clone())),
            other => format!("{}: {other}", ChildError::StoreDenied("create".into())),
        })?;

    // Admin audit through the chokepoint (one audit point).
    let _ = assert_reach(cp, principal, &parsed.id).await;

    let reply = CreateReply {
        message: t(locale, "child.created", &[("name", &child.name)]),
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
            caps: vec!["mcp:care.child.create:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn create_writes_a_child_with_safety_data() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        let out = run(
            &cp,
            &p,
            r#"{"id":"leo","name":"Leo","dob":"2021-03-15","allergies":["peanuts"],"photo_consent":true}"#,
        )
        .await
        .expect("create");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["id"], "leo");

        let row = read(&store, "acme", "child", "leo").await.unwrap().unwrap();
        assert_eq!(row["name"], "Leo");
        assert_eq!(row["allergies"][0], "peanuts");
        assert_eq!(row["archived"], false);
        assert_eq!(row["photo_consent"], true);
    }

    #[tokio::test]
    async fn create_rejects_a_bad_dob() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(&cp, &p, r#"{"id":"leo","name":"Leo","dob":"not-a-date"}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("invalid dob"));
    }

    #[tokio::test]
    async fn create_is_first_write() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let input = r#"{"id":"leo","name":"Leo","dob":"2021-03-15"}"#;
        run(&cp, &p, input).await.expect("first");
        assert!(run(&cp, &p, input).await.is_err(), "dup conflicts");
    }
}
