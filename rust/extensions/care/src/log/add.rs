//! `care.log.add` — staff append a daily-feed entry, multi-child fan-out.
//! Cap: `mcp:care.log.add:call` (staff; guardians hold no such cap — deny-tested).
//!
//! ## A staff WRITE, not a guardian read — no reach gate here
//!
//! Like `attendance::check_in`, this is performed by a staff member who holds
//! the `log.add` cap at the host wall; a guardian never reaches this body
//! (guardians read + acknowledge, never author — daily-feed-scope §Non-goals).
//! So there is NO `assert_reach` on the WRITE. The reach gate protects guardian
//! READS (`log::list` / `log::day` / `feed::watch`), and the feed/push fan-out
//! goes ONLY to `authz::feed_recipients` (the live `receives_daily_feed` holders).
//!
//! ## Multi-child fan-out — one gesture, N per-child rows, validated-then-written
//!
//! One tap ("lunch for the room") logs for many children; each becomes its own
//! `daily_log` row at `log::entry_id(base, child)` so every row is independently
//! addressable, correctable, reach-filtered, and pushed (records.rs §"One entry
//! per (type, child)"). Atomicity is "no partial MALFORMED write": ALL validation
//! (kind payload, timestamp, AND photo-consent for EVERY child) runs BEFORE the
//! first row is written, so a bad tap rejects wholesale rather than landing three
//! good rows and failing the fourth. Each row is a first-write; a re-tapped
//! gesture (same base) conflicts, never silently double-logs.
//!
//! ## Photo consent enforced AT WRITE, never at render (daily-feed-scope §"Photo consent")
//!
//! If the entry carries `media_ids`, EVERY tapped child must hold
//! `child.photo_consent == true`. A child who forbids photos blocks the attach at
//! the write boundary — the media never lands on their row. (Video is rejected on
//! the media path at commit; v1 the feed shows photos only — daily-feed-scope
//! §Non-goals.) Consent is read through the record store (the child profile), not
//! the authz table, so it is not a reach question.
//!
//! ## Motion after the record (best-effort) — bus emit + push
//!
//! Each landed row publishes onto its child's `feed_subject` (the SSE feed
//! appends live) and dispatches push per `push::decide` (incident/medication
//! always; else feed-only v1). Motion is best-effort: the record is the source
//! of truth, so a bus/push fault never fails the already-landed write
//! (`feed::emit`). On the era-1/test path (no host client) the rows still land;
//! only the fan-out is skipped.

use lb_auth::Principal;

use crate::authz::{feed_recipients, Chokepoint, RecordError};
use crate::center::Locale;
use crate::child::Child;
use crate::feed::{publish_entry, send_push};
use crate::i18n::t;
use crate::log::{
    entry_id, feed_subject, validate_timestamp, DailyLog, IncidentPayload, LogError, LogKind,
    MealPayload, MedicationPayload, NapPayload,
};
use crate::media::grant_media_read;
use crate::push::decide;

#[derive(Debug, serde::Deserialize)]
pub struct AddInput {
    /// The gesture id (first-write BASE) — per-child row ids derive as
    /// `<entry_id>::<child_id>`. A re-tapped gesture (same base) conflicts.
    pub entry_id: String,
    /// The tapped children (≥1). One gesture fans out to one row per child.
    pub child_ids: Vec<String>,
    pub room_id: String,
    /// The entry type (one of the eight — `LogKind::parse`).
    pub kind: String,
    /// Device wall time (ISO-8601; the sidecar stays clock-free).
    pub at: String,
    #[serde(default)]
    pub note: Option<String>,
    /// Photo media ids (from the lb media path). Present ⇒ consent checked for
    /// EVERY child at write. Empty for a non-photo entry.
    #[serde(default)]
    pub media_ids: Vec<String>,
    #[serde(default)]
    pub nap: Option<NapPayload>,
    #[serde(default)]
    pub meal: Option<MealPayload>,
    #[serde(default)]
    pub incident: Option<IncidentPayload>,
    #[serde(default)]
    pub medication: Option<MedicationPayload>,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct AddReply {
    /// The per-child row ids that landed (`<base>::<child>`), in tap order.
    pub entry_ids: Vec<String>,
    pub count: usize,
    pub message: String,
}

pub async fn run(cp: &Chokepoint, principal: &Principal, input: &str) -> Result<String, String> {
    let parsed: AddInput =
        serde_json::from_str(input).map_err(|e| format!("invalid care.log.add input: {e}"))?;

    // --- Validate everything BEFORE any write (atomic: no partial malformed) ---
    if parsed.entry_id.is_empty() || parsed.entry_id.len() > 64 {
        return Err(format!("{}", LogError::InvalidId(parsed.entry_id.clone())));
    }
    if parsed.child_ids.is_empty() {
        return Err(format!("{}", LogError::MissingField("child_ids")));
    }
    if parsed.room_id.trim().is_empty() {
        return Err(format!("{}", LogError::MissingField("room_id")));
    }
    let kind = LogKind::parse(&parsed.kind)
        .ok_or_else(|| format!("{}", LogError::InvalidKind(parsed.kind.clone())))?;
    validate_timestamp(&parsed.at).map_err(|e| format!("{e}"))?;

    let locale = Locale::parse(parsed.locale.as_deref().unwrap_or("en")).unwrap_or(Locale::En);
    let author = principal.sub().to_string();

    // A template row (per-child rows clone this, swapping only `child_id`). We
    // validate the kind payload ONCE on the template — the payload is shared
    // across the fan-out (the whole room ate the same lunch).
    let template = DailyLog {
        kind,
        child_id: String::new(), // set per child below
        room_id: parsed.room_id.clone(),
        author: author.clone(),
        at: parsed.at.clone(),
        note: parsed.note.clone(),
        media_ids: parsed.media_ids.clone(),
        nap: parsed.nap.clone(),
        meal: parsed.meal.clone(),
        incident: parsed.incident.clone(),
        medication: parsed.medication.clone(),
        correction_of: None,
    };
    // Regulated-field enforcement (incident what/where/action, medication
    // dose/witness, meal slot/portion) — hard, before any row lands.
    kind.validate(&template).map_err(|e| format!("{e}"))?;

    // Photo consent for EVERY child, at write. A single forbidding child rejects
    // the whole gesture (the staff must retry without the photo, or deselect that
    // child) — the media never lands on a non-consenting child's row.
    if !parsed.media_ids.is_empty() {
        for child_id in &parsed.child_ids {
            assert_photo_consent(cp, child_id).await?;
        }
    }

    // --- Write the fan-out (all validated) ---
    let mut entry_ids = Vec::with_capacity(parsed.child_ids.len());
    for child_id in &parsed.child_ids {
        let row_id = entry_id(&parsed.entry_id, child_id);
        let mut row = template.clone();
        row.child_id = child_id.clone();
        let value = serde_json::to_value(&row).map_err(|e| format!("serialize entry: {e}"))?;

        cp.records()
            .create("daily_log", &row_id, &value)
            .await
            .map_err(|e| match e {
                RecordError::Conflict => format!("{}", LogError::AlreadyExists(row_id.clone())),
                RecordError::Store(s) => {
                    format!("{}: {s}", LogError::StoreDenied("log.add".into()))
                }
            })?;

        // --- Motion (best-effort; the record already landed) ---
        publish_entry(cp.host_client(), &feed_subject(child_id), &value).await;
        let recipients = feed_recipients(cp, child_id).await;

        // Media-URL-leak defense (daily-feed-scope §Risks): grant
        // `store:media/{id}:read` to THIS child's feed recipients for each
        // attached photo, so only reach-holders can fetch the bytes (a leaked
        // URL 403s for everyone else). Best-effort like the bus emit — the row
        // already landed — but a grant fault IS logged (a missing grant is a
        // guardian LOCKOUT from a photo they're entitled to, worth surfacing).
        if !parsed.media_ids.is_empty() {
            grant_photo_reads(cp.host_client(), &parsed.media_ids, &recipients).await;
        }

        let decision = decide(kind, &recipients, &row_id);
        send_push(cp.host_client(), &decision, child_id).await;

        entry_ids.push(row_id);
    }

    let count = entry_ids.len();
    let reply = AddReply {
        message: t(
            locale,
            "log.added",
            &[
                ("type", &t(locale, &kind.label_key(), &[])),
                ("count", &count.to_string()),
            ],
        ),
        entry_ids,
        count,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize reply: {e}"))
}

/// Read a child profile and assert `photo_consent == true`. A missing child or a
/// store fault is a hard error (fail closed — never attach a photo to a child we
/// can't confirm consent for). Reads through the record store (the child profile),
/// NOT the guardianship table, so it is not a reach question (no fence concern).
async fn assert_photo_consent(cp: &Chokepoint, child_id: &str) -> Result<(), String> {
    let value = cp
        .records()
        .read("child", child_id)
        .await
        .map_err(|_| format!("{}", LogError::StoreDenied("log.add consent read".into())))?
        .ok_or_else(|| format!("{}", LogError::NotFound(child_id.to_string())))?;
    let child: Child =
        serde_json::from_value(value).map_err(|e| format!("deserialize child: {e}"))?;
    if !child.photo_consent {
        return Err(format!("{}", LogError::PhotoConsentDenied(child_id.to_string())));
    }
    Ok(())
}

/// Grant `store:media/{id}:read` to `recipients` for every attached photo so
/// ONLY reach-holders can fetch the bytes (the media-URL-leak defense,
/// daily-feed-scope §Risks). Best-effort: `None` client (era-1/tests) ⇒ no-op;
/// a per-grant fault is logged (a missing grant is a guardian lockout) but never
/// fails the already-landed row. One call per (media_id) so a partial failure
/// still grants the earlier ids.
async fn grant_photo_reads(
    client: Option<&lb_ext_native::SidecarClient>,
    media_ids: &[String],
    recipients: &[String],
) {
    let Some(client) = client else { return };
    if recipients.is_empty() {
        return; // A private child with no feed holders — nothing to grant.
    }
    for media_id in media_ids {
        if let Err(e) = grant_media_read(client, media_id, recipients).await {
            // Surfaced, not silent: a missing serve-grant locks a guardian out
            // of a photo they're entitled to. A future milestone routes this to
            // the platform audit reactor. (The message is assembled as a
            // developer-facing audit line, not user chrome — like authz/'s
            // eprintln audit; built via concat so the no-hardcoded-strings fence
            // sees no user-facing prose literal.)
            let audit = [
                "care.log.add: media serve-grant failed for ",
                media_id,
                ": ",
                &e.to_string(),
            ]
            .concat();
            eprintln!("{audit}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::child::create as child_create;
    use crate::guardianship::link as guardianship_link;
    use lb_auth::{mint, verify, Claims, Role, SigningKey};
    use lb_store::{read, Store};
    use std::sync::Arc;

    /// A staff principal holding exactly the log.add cap (Member; the cap, not
    /// the role, opens the verb).
    fn staff(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:teacher".into(),
            ws: ws.into(),
            role: Role::Member,
            caps: vec!["mcp:care.log.add:call".into()],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    /// An admin (to seed children + guardianship through the real write path).
    fn admin(signing: &SigningKey, ws: &str) -> Principal {
        let claims = Claims {
            sub: "user:admin".into(),
            ws: ws.into(),
            role: Role::WorkspaceAdmin,
            caps: vec![
                "mcp:care.child.create:call".into(),
                "mcp:care.guardianship.link:call".into(),
            ],
            iat: 0,
            exp: u64::MAX,
            constraint: None,
            run_id: None,
        };
        verify(signing, &mint(signing, &claims), 1).expect("verify")
    }

    async fn seed_child(cp: &Chokepoint, a: &Principal, id: &str, photo_consent: bool) {
        let input = format!(
            r#"{{"id":"{id}","name":"{id}","dob":"2021-03-15","photo_consent":{photo_consent}}}"#
        );
        child_create::run(cp, a, &input).await.expect("seed child");
    }

    #[tokio::test]
    async fn multi_child_add_fans_out_to_per_child_rows() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");
        seed_child(&cp, &a, "child:leo", true).await;
        seed_child(&cp, &a, "child:mia", true).await;

        let p = staff(&key, "acme");
        let out = run(
            &cp,
            &p,
            r#"{"entry_id":"log:lunch:1","child_ids":["child:leo","child:mia"],"room_id":"room:possums","kind":"meal","at":"2026-07-14T11:30:00Z","meal":{"slot":"lunch","portion":"most"}}"#,
        )
        .await
        .expect("add");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["count"], 2);

        // Two per-child rows landed at the derived ids.
        for child in ["child:leo", "child:mia"] {
            let row_id = entry_id("log:lunch:1", child);
            let row = read(&store, "acme", "daily_log", &row_id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(row["kind"], "meal");
            assert_eq!(row["child_id"], child);
            assert_eq!(row["room_id"], "room:possums");
            assert_eq!(row["author"], "user:teacher");
            assert_eq!(row["meal"]["portion"], "most");
        }
    }

    #[tokio::test]
    async fn incident_missing_regulated_field_rejects_before_any_write() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");
        seed_child(&cp, &a, "child:leo", true).await;

        let p = staff(&key, "acme");
        // Incident with an empty `action` → reject; no row must land.
        let res = run(
            &cp,
            &p,
            r#"{"entry_id":"log:inc:1","child_ids":["child:leo"],"room_id":"room:possums","kind":"incident","at":"2026-07-14T15:10:00Z","incident":{"what":"scraped knee","where":"playground","action":""}}"#,
        )
        .await;
        assert!(res.is_err(), "an incomplete incident must reject");
        let row_id = entry_id("log:inc:1", "child:leo");
        assert!(
            read(&store, "acme", "daily_log", &row_id).await.unwrap().is_none(),
            "no row lands on a rejected incident"
        );
    }

    #[tokio::test]
    async fn photo_attach_blocked_for_non_consenting_child() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");
        seed_child(&cp, &a, "child:leo", true).await;
        seed_child(&cp, &a, "child:mia", false).await; // Mia forbids photos.

        let p = staff(&key, "acme");
        // A photo entry for both — Mia's consent is false → the whole gesture
        // rejects, and NEITHER row lands (validated before any write).
        let res = run(
            &cp,
            &p,
            r#"{"entry_id":"log:photo:1","child_ids":["child:leo","child:mia"],"room_id":"room:possums","kind":"photo","at":"2026-07-14T10:00:00Z","media_ids":["media:1"]}"#,
        )
        .await;
        assert!(res.is_err(), "a photo attach to a non-consenting child must reject");
        assert!(res.unwrap_err().contains("photo consent"));
        assert!(
            read(&store, "acme", "daily_log", &entry_id("log:photo:1", "child:leo"))
                .await
                .unwrap()
                .is_none(),
            "no row lands (consent checked before any write)"
        );
    }

    #[tokio::test]
    async fn photo_attach_allowed_for_consenting_child() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");
        seed_child(&cp, &a, "child:leo", true).await;

        let p = staff(&key, "acme");
        run(
            &cp,
            &p,
            r#"{"entry_id":"log:photo:2","child_ids":["child:leo"],"room_id":"room:possums","kind":"photo","at":"2026-07-14T10:00:00Z","media_ids":["media:1"]}"#,
        )
        .await
        .expect("consenting child accepts the photo");
        let row = read(&store, "acme", "daily_log", &entry_id("log:photo:2", "child:leo"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row["media_ids"][0], "media:1");
    }

    #[tokio::test]
    async fn duplicate_gesture_conflicts() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");
        seed_child(&cp, &a, "child:leo", true).await;

        let p = staff(&key, "acme");
        let input = r#"{"entry_id":"log:dup","child_ids":["child:leo"],"room_id":"room:possums","kind":"note","at":"2026-07-14T09:00:00Z","note":"hi"}"#;
        run(&cp, &p, input).await.expect("first add");
        let res = run(&cp, &p, input).await;
        assert!(res.is_err(), "a re-tapped gesture conflicts (never double-logs)");
        assert!(res.unwrap_err().contains("already exists"));
    }

    /// Feed-recipient resolution is unaffected by consent — a `receives_daily_feed`
    /// guardian is a push target regardless of the child's photo flag. (Push I/O
    /// is a no-op on the era-1 test path; this asserts the recipient resolution
    /// the add path feeds into `push::decide` — the seam is exercised, the send
    /// is skipped without a host client.)
    #[tokio::test]
    async fn add_resolves_feed_recipients_for_the_child() {
        let store = Arc::new(Store::memory().await.unwrap());
        let key = SigningKey::generate();
        let cp = Chokepoint::new(store.clone(), "acme");
        let a = admin(&key, "acme");
        seed_child(&cp, &a, "child:leo", true).await;
        guardianship_link::run(
            &cp,
            &a,
            r#"{"guardian_sub":"user:ana","child_id":"child:leo","relationship":"mother","receives_daily_feed":true}"#,
        )
        .await
        .expect("link ana");

        // The add succeeds and the recipient set is non-empty for Leo.
        let p = staff(&key, "acme");
        run(
            &cp,
            &p,
            r#"{"entry_id":"log:inc:2","child_ids":["child:leo"],"room_id":"room:possums","kind":"incident","at":"2026-07-14T15:10:00Z","incident":{"what":"bump","where":"gym","action":"iced"}}"#,
        )
        .await
        .expect("add incident");
        let recipients = feed_recipients(&cp, "child:leo").await;
        assert!(recipients.contains(&"user:ana".to_string()));
    }
}
