//! `care.room.get` + `care.room.list` — read paths (admin wildcard; staff
//! filtered to rooms they reach via `staff_assignment`).

use lb_auth::Principal;

use crate::authz::{assert_reach, reachable_rooms, Chokepoint};
use crate::room::Room;

// ----- get ---------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
pub struct GetInput {
    pub id: String,
}

pub async fn get(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: GetInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.room.get input: {e}"))?;
    assert_reach(cp, principal, &parsed.id)
        .await
        .map_err(|e| format!("{e}"))?;

    let row = cp
        .records()
        .read("room", &parsed.id)
        .await
        .map_err(|_| "store denied the room read".to_string())?;
    let value = row.ok_or_else(|| "room not found".to_string())?;
    let room: Room = serde_json::from_value(value).map_err(|e| format!("deserialize: {e}"))?;
    serde_json::to_string(&room).map_err(|e| format!("serialize: {e}"))
}

// ----- list --------------------------------------------------------------

pub async fn list(cp: &Chokepoint, principal: &Principal, _input: &str) -> Result<String, String> {
    let is_admin = principal.role() == lb_auth::Role::WorkspaceAdmin
        || principal.role() == lb_auth::Role::SuperAdmin;
    if !is_admin {
        // Staff: filter to the rooms the chokepoint resolves.
        let rooms = reachable_rooms(cp, principal).await;
        if rooms.is_empty() || rooms == vec!["*".to_string()] {
            return Ok("[]".to_string());
        }
        // Per-room read for the resolved set. Each row carries its store `id`
        // (the reach id IS the record id) so the UI can select/route by it.
        let mut out: Vec<serde_json::Value> = Vec::new();
        for id in rooms {
            if let Ok(Some(v)) = cp.records().read("room", &id).await {
                out.push(with_id(&id, v));
            }
        }
        return serde_json::to_string(&out).map_err(|e| format!("serialize: {e}"));
    }

    // Admin path: list every room, each carrying its store `id`.
    let rows: Vec<(String, serde_json::Value)> = cp
        .records()
        .query_rows("room")
        .await
        .map_err(|e| format!("store denied: {e}"))?;
    let out: Vec<serde_json::Value> = rows.into_iter().map(|(id, v)| with_id(&id, v)).collect();
    serde_json::to_string(&out).map_err(|e| format!("serialize: {e}"))
}

/// Merge the store `id` into a record's `data` object so a list row is
/// `{ id, ...record }` — the shape the UI addresses rooms by (the record body
/// itself carries no id). A non-object value is returned unchanged (defensive).
pub(crate) fn with_id(id: &str, mut data: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = data.as_object_mut() {
        obj.insert("id".to_string(), serde_json::Value::String(id.to_string()));
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::create as room_create;
    use crate::room::records::Room;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{create as store_create, write as store_write, Store};
    use serde_json::json;
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.room.create:call".into(),
                "mcp:care.room.list:call".into(),
                "mcp:care.room.get:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn admin_list_returns_all_rooms() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        room_create::run(&cp, &p, r#"{"id":"possums","name":"P","center_id":"main"}"#)
            .await
            .unwrap();
        room_create::run(&cp, &p, r#"{"id":"koalas","name":"K","center_id":"main"}"#)
            .await
            .unwrap();
        let out = list(&cp, &p, "").await.unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(v.len(), 2);
        // Each row MUST carry its store id (the UI selects/routes by it — the
        // record body has none; a missing id left the room picker unusable).
        let ids: std::collections::HashSet<&str> =
            v.iter().filter_map(|r| r["id"].as_str()).collect();
        assert!(
            ids.contains("possums") && ids.contains("koalas"),
            "rows carry their id: {v:?}"
        );
    }

    #[tokio::test]
    async fn staff_list_filters_to_assigned_rooms() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let ap = admin(&key, "acme");
        room_create::run(
            &cp,
            &ap,
            r#"{"id":"possums","name":"P","center_id":"main"}"#,
        )
        .await
        .unwrap();
        room_create::run(&cp, &ap, r#"{"id":"koalas","name":"K","center_id":"main"}"#)
            .await
            .unwrap();

        // Sam is assigned to Possums only.
        store_create(
            &store,
            "acme",
            "staff_assignment",
            "user:sam::possums",
            &json!({"staff_sub":"user:sam","room_id":"possums"}),
        )
        .await
        .unwrap();
        store_write(
            &store,
            "acme",
            "room",
            "possums",
            &serde_json::to_value(Room {
                name: "P".into(),
                center_id: "main".into(),
                archived: false,
            })
            .unwrap(),
        )
        .await
        .unwrap();

        let claims = Claims {
            sub: "user:sam".into(),
            ws: "acme".into(),
            role: Role::Member,
            caps: vec!["mcp:care.room.list:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        let sam = verify(&key, &mint(&key, &claims), 1).unwrap();
        let out = list(&cp, &sam, "").await.unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(v.len(), 1, "sam sees only Possums");
        assert_eq!(v[0]["name"], "P");
    }
}
