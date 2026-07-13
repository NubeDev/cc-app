//! `care.child.update` — admin edits a child profile. Cap:
//! `mcp:care.child.update:call`. Admin-only (staff `care.child.update` → 403
//! is the canonical cap-deny test, `03-enrollment.md` exit gate).
//!
//! Partial update: only the fields present in the input overwrite; a DOB in
//! the input is re-validated (a safety field). The `archived` flag is NOT
//! editable here — `care.child.archive` owns it (archive, never delete).

use lb_auth::Principal;

use crate::authz::{assert_reach, Chokepoint};
use crate::center::Locale;
use crate::child::{validate_dob, ChildError};
use crate::i18n::t;

#[derive(Debug, serde::Deserialize)]
pub struct UpdateInput {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub dob: Option<String>,
    #[serde(default)]
    pub room_id: Option<String>,
    #[serde(default)]
    pub allergies: Option<Vec<String>>,
    #[serde(default)]
    pub medical_notes: Option<String>,
    #[serde(default)]
    pub immunizations: Option<Vec<String>>,
    #[serde(default)]
    pub photo_consent: Option<bool>,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct UpdateReply {
    pub id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: UpdateInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.child.update input: {e}"))?;
    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    // Reach check first (admin audited; the wall already denied staff/guardian).
    assert_reach(cp, principal, &parsed.id)
        .await
        .map_err(|e| format!("{e}"))?;

    let mut row = cp
        .records()
        .read("child", &parsed.id)
        .await
        .map_err(|_| format!("{}", ChildError::StoreDenied("update read".into())))?
        .ok_or_else(|| format!("{}", ChildError::NotFound(parsed.id.clone())))?;

    if let Some(name) = &parsed.name {
        if name.trim().is_empty() {
            return Err(format!("{}", ChildError::MissingField("name")));
        }
        row["name"] = serde_json::Value::String(name.clone());
    }
    if let Some(dob) = &parsed.dob {
        validate_dob(dob).map_err(|e| format!("{e}"))?;
        row["dob"] = serde_json::Value::String(dob.clone());
    }
    if let Some(room) = &parsed.room_id {
        row["room_id"] = serde_json::Value::String(room.clone());
    }
    if let Some(allergies) = &parsed.allergies {
        row["allergies"] = serde_json::to_value(allergies).unwrap_or(row["allergies"].clone());
    }
    if let Some(notes) = &parsed.medical_notes {
        row["medical_notes"] = serde_json::Value::String(notes.clone());
    }
    if let Some(imm) = &parsed.immunizations {
        row["immunizations"] = serde_json::to_value(imm).unwrap_or(row["immunizations"].clone());
    }
    if let Some(pc) = parsed.photo_consent {
        row["photo_consent"] = serde_json::Value::Bool(pc);
    }

    cp.records()
        .write("child", &parsed.id, &row)
        .await
        .map_err(|e| format!("{}: {e}", ChildError::StoreDenied("update write".into())))?;

    let name = row.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let reply = UpdateReply {
        message: t(locale, "child.created", &[("name", name)]),
        id: parsed.id,
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
                "mcp:care.child.update:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn update_edits_allergies_and_room() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        child_create::run(&cp, &p, r#"{"id":"leo","name":"Leo","dob":"2021-03-15"}"#)
            .await
            .expect("create");

        run(
            &cp,
            &p,
            r#"{"id":"leo","allergies":["dairy","eggs"],"room_id":"possums"}"#,
        )
        .await
        .expect("update");

        let row = read(&store, "acme", "child", "leo").await.unwrap().unwrap();
        assert_eq!(row["allergies"][1], "eggs");
        assert_eq!(row["room_id"], "possums");
    }

    #[tokio::test]
    async fn update_rejects_a_bad_dob() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");
        child_create::run(&cp, &p, r#"{"id":"leo","name":"Leo","dob":"2021-03-15"}"#)
            .await
            .unwrap();
        let res = run(&cp, &p, r#"{"id":"leo","dob":"2021-13-99"}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("invalid dob"));
    }

    #[tokio::test]
    async fn update_of_a_missing_child_is_not_found() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(&cp, &p, r#"{"id":"ghost","name":"Nobody"}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not found"));
    }
}
