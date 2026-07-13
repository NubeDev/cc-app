//! `care.room.create` — admin creates a room record inside a center.
//!
//! Same shape as `center.create`: validate id → upsert FIRST-WRITE → audit.
//! The `center_id` is recorded verbatim; cross-workspace isolation is the
//! store's responsibility (the workspace is selected from `cp.ws`).

use lb_auth::Principal;

use crate::authz::{assert_reach, Chokepoint, RecordError};
use crate::room::{Room, RoomError};

#[derive(Debug, serde::Deserialize)]
pub struct CreateInput {
    pub id: String,
    pub name: String,
    pub center_id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct CreateReply {
    pub id: String,
    pub name: String,
    pub center_id: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: CreateInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.room.create input: {e}"))?;

    if parsed.id.is_empty() || parsed.id.len() > 64 {
        return Err(format!(
            "{}: id must be 1..=64 chars",
            RoomError::InvalidId(parsed.id.clone())
        ));
    }

    let room = Room {
        name: parsed.name.clone(),
        center_id: parsed.center_id.clone(),
        archived: false,
    };
    let value = serde_json::to_value(&room).map_err(|e| format!("serialize: {e}"))?;

    cp.records()
        .create("room", &parsed.id, &value)
        .await
        .map_err(|e| match e {
            RecordError::Conflict => {
                format!("{}", RoomError::AlreadyExists(parsed.id.clone()))
            }
            RecordError::Store(s) => format!("{}: {s}", RoomError::StoreDenied("create".into())),
        })?;

    let _ = assert_reach(cp, principal, &parsed.id).await;

    let reply = CreateReply {
        id: parsed.id,
        name: room.name,
        center_id: room.center_id,
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
            caps: vec!["mcp:care.room.create:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn create_writes_a_room_record() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let out = run(
            &cp,
            &p,
            r#"{"id":"possums","name":"Possums","center_id":"main"}"#,
        )
        .await
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["id"], "possums");
        assert_eq!(v["center_id"], "main");
    }

    #[tokio::test]
    async fn create_is_first_write_a_duplicate_id_conflicts() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let input = r#"{"id":"possums","name":"P","center_id":"main"}"#;
        run(&cp, &p, input).await.unwrap();
        let res = run(&cp, &p, input).await;
        assert!(res.is_err());
        // "room already exists: possums"
        assert!(res.unwrap_err().contains("already exists"));
    }
}
