//! `care.enrollment.update` — admin edits an enrollment (status change,
//! schedule, start date). Cap: `mcp:care.enrollment.update:call`. Admin-only.
//!
//! Partial update: only fields present in the input overwrite. A status change
//! to `withdrawn` KEEPS the existing `waitlist_seq` — withdrawing a mid-list
//! child never renumbers the room's waitlist (`enrollment-invites-scope.md`
//! "waitlist ordering stable").

use lb_auth::Principal;

use crate::authz::{assert_reach, Chokepoint};
use crate::center::Locale;
use crate::enrollment::{EnrollmentError, EnrollmentStatus, Weekday};
use crate::i18n::t;

#[derive(Debug, serde::Deserialize)]
pub struct UpdateInput {
    pub child_id: String,
    pub room_id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub schedule: Option<Vec<String>>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct UpdateReply {
    pub id: String,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: UpdateInput = serde_json::from_str(input)
        .map_err(|e| format!("invalid care.enrollment.update input: {e}"))?;
    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);

    if parsed.child_id.trim().is_empty() {
        return Err(format!("{}", EnrollmentError::MissingField("child_id")));
    }
    if parsed.room_id.trim().is_empty() {
        return Err(format!("{}", EnrollmentError::MissingField("room_id")));
    }

    let id = format!("{}::{}", parsed.child_id, parsed.room_id);

    // Reach check first (admin audited; the wall already denied staff/guardian).
    assert_reach(cp, principal, &parsed.child_id)
        .await
        .map_err(|e| format!("{e}"))?;

    let mut row = cp
        .records()
        .read("enrollment", &id)
        .await
        .map_err(|_| format!("{}", EnrollmentError::StoreDenied("update read".into())))?
        .ok_or_else(|| format!("{}", EnrollmentError::NotFound(id.clone())))?;

    // Track a status transition so the reply can announce a withdrawal.
    let mut new_status: Option<EnrollmentStatus> = None;
    if let Some(status) = &parsed.status {
        let parsed_status = EnrollmentStatus::parse(status).map_err(|e| format!("{e}"))?;
        // A status change to withdrawn keeps waitlist_seq as-is (stable order).
        row["status"] = serde_json::Value::String(parsed_status.as_str().to_string());
        new_status = Some(parsed_status);
    }
    if let Some(schedule) = &parsed.schedule {
        let mut days: Vec<Weekday> = Vec::with_capacity(schedule.len());
        for day in schedule {
            days.push(Weekday::parse(day).map_err(|e| format!("{e}"))?);
        }
        row["schedule"] = serde_json::to_value(&days).unwrap_or(row["schedule"].clone());
    }
    if let Some(start_date) = &parsed.start_date {
        row["start_date"] = serde_json::Value::String(start_date.clone());
    }

    cp.records()
        .write("enrollment", &id, &row)
        .await
        .map_err(|e| {
            format!(
                "{}: {e}",
                EnrollmentError::StoreDenied("update write".into())
            )
        })?;

    let message = if new_status == Some(EnrollmentStatus::Withdrawn) {
        t(
            locale,
            "enrollment.withdrawn",
            &[("child", &parsed.child_id), ("room", &parsed.room_id)],
        )
    } else {
        t(locale, "enrollment.updated", &[("child", &parsed.child_id)])
    };

    let reply = UpdateReply { id, message };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enrollment::create as enrollment_create;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{read, Store};
    use std::sync::Arc;

    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.enrollment.create:call".into(),
                "mcp:care.enrollment.update:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    #[tokio::test]
    async fn update_changes_status_and_keeps_waitlist_seq() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let p = admin(&key, "acme");

        enrollment_create::run(
            &cp,
            &p,
            r#"{"child_id":"leo","room_id":"possums","status":"waitlist"}"#,
        )
        .await
        .expect("create");

        let out = run(
            &cp,
            &p,
            r#"{"child_id":"leo","room_id":"possums","status":"withdrawn"}"#,
        )
        .await
        .expect("update");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["id"], "leo::possums");
        // The withdrawal message key is used on a status→withdrawn change.
        assert!(v["message"].as_str().unwrap().contains("withdrawn"));

        let row = read(&store, "acme", "enrollment", "leo::possums")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row["status"], "withdrawn");
        // Stable ordering: the seq is retained, not zeroed, on withdrawal.
        assert_eq!(row["waitlist_seq"], 1);
    }

    #[tokio::test]
    async fn update_of_a_missing_enrollment_is_not_found() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store, "acme");
        let p = admin(&key, "acme");
        let res = run(&cp, &p, r#"{"child_id":"ghost","room_id":"nowhere"}"#).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not found"));
    }
}
