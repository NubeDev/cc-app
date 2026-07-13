//! The PICKUP GATE — the child-safety control at check-out. ORCHESTRATOR-OWNED
//! (attendance-scope §"Pickup authorization"; the reviewer's brief is literally
//! "bypass the gate"). A pure decision function: given the collector and the
//! child's authorization roster, decide ALLOW or DENY-with-reason. Only an
//! admin `pickup_override` gets past a DENY, and that override is audited (the
//! event stores the reason it bypassed).
//!
//! ## Why the roster is passed in (rule 7 + the authz fence)
//!
//! The `can_pickup` flag and `custody_notes` live on the `guardianship` edge —
//! and the authz fence (`check-authz-fence.sh`) forbids reading `guardianship`
//! anywhere but `authz/`. So the check-out verb resolves the roster through the
//! ONE chokepoint helper (`authz::pickup_roster`), then hands the resolved
//! [`PickupRoster`] to this pure gate. The safety LOGIC lives here, in one
//! tested place; the guardianship READ lives in authz/, behind the fence. No
//! verb re-implements either half.
//!
//! ## The decision (fail-closed, conservative)
//!
//! A collector is allowed iff they are EITHER a `can_pickup` guardian for this
//! child OR a named authorized-pickup entry on the child record — AND the child
//! is not under a custody hold. Anything else is a hard DENY carrying a
//! localizable [`PickupDeny`] reason. Ambiguity denies (child-safety default).

use super::records::PickupDeny;

/// The resolved authorization roster for ONE child — everything the gate needs,
/// pre-fetched by the verb through `authz::pickup_roster` (guardianship edges)
/// + the child record (authorized-pickup names). Names are compared
/// case-insensitively, trimmed; guardian ids are the auth subjects.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PickupRoster {
    /// Auth subjects of guardians whose edge has `can_pickup == true`.
    pub can_pickup_guardians: Vec<String>,
    /// Display names of guardians whose edge has `can_pickup == true` (so a
    /// staff-selected NAME, not a sub, still authorizes — the UI shows names).
    pub can_pickup_names: Vec<String>,
    /// Names on the child record's `authorized_pickups` list (grandma, etc.).
    pub authorized_pickup_names: Vec<String>,
    /// `true` if the child's edges carry any `custody_notes` — a hold that
    /// requires the note be read (staff sees it) and release needs the admin
    /// override. Conservative: a note present ⇒ gate holds.
    pub custody_hold: bool,
}

/// The collector presented at check-out — either a guardian (by sub) or a
/// person selected by name (an authorized-pickup entry, or a guardian picked
/// from the name list). The verb builds this from its input.
#[derive(Debug, Clone)]
pub struct Collector {
    /// The collector's auth subject, if they authenticated as a guardian.
    pub sub: Option<String>,
    /// The collector's display name (always present — the staff picks a name).
    pub name: String,
}

/// Decide whether `collector` may take the child, given the resolved `roster`.
/// `Ok(())` ⇒ allowed. `Err(PickupDeny)` ⇒ refused with a localizable reason;
/// only an admin override (checked by the verb, not here) may proceed past it.
///
/// Order matters: a custody hold denies FIRST (the strongest control), even if
/// the collector is otherwise authorized — the note must be read and the
/// release explicitly overridden.
pub fn decide(collector: &Collector, roster: &PickupRoster) -> Result<(), PickupDeny> {
    // Custody hold is the strongest gate — it denies even an authorized
    // guardian so the note is read and the release is an audited override.
    if roster.custody_hold {
        return Err(PickupDeny::CustodyHold);
    }

    let name = collector.name.trim().to_ascii_lowercase();

    // A guardian authenticated by sub with a live can_pickup edge → allowed.
    if let Some(sub) = &collector.sub {
        if roster.can_pickup_guardians.iter().any(|g| g == sub) {
            return Ok(());
        }
    }

    // A person selected by NAME: a can_pickup guardian's name OR an
    // authorized-pickup entry on the child record → allowed.
    let name_authorized = roster
        .can_pickup_names
        .iter()
        .chain(roster.authorized_pickup_names.iter())
        .any(|n| n.trim().to_ascii_lowercase() == name && !name.is_empty());
    if name_authorized {
        return Ok(());
    }

    // The collector matched no guardian sub and no name anywhere on the
    // roster → they are unknown to this child. If the roster HAS authorized
    // people but this collector isn't one, it's `NotAuthorized`; if the child
    // has no roster at all, the collector is simply unknown.
    let roster_empty = roster.can_pickup_guardians.is_empty()
        && roster.can_pickup_names.is_empty()
        && roster.authorized_pickup_names.is_empty();
    if roster_empty {
        Err(PickupDeny::UnknownPerson)
    } else {
        Err(PickupDeny::NotAuthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roster() -> PickupRoster {
        PickupRoster {
            can_pickup_guardians: vec!["user:sam".into()],
            can_pickup_names: vec!["Sam".into()],
            authorized_pickup_names: vec!["Grandma Jo".into()],
            custody_hold: false,
        }
    }

    #[test]
    fn a_can_pickup_guardian_by_sub_is_allowed() {
        let c = Collector {
            sub: Some("user:sam".into()),
            name: "Sam".into(),
        };
        assert!(decide(&c, &roster()).is_ok());
    }

    #[test]
    fn an_authorized_pickup_by_name_is_allowed() {
        let c = Collector {
            sub: None,
            name: "grandma jo".into(), // case-insensitive
        };
        assert!(decide(&c, &roster()).is_ok());
    }

    #[test]
    fn a_stranger_is_denied_not_authorized() {
        let c = Collector {
            sub: Some("user:mallory".into()),
            name: "Mallory".into(),
        };
        assert_eq!(decide(&c, &roster()), Err(PickupDeny::NotAuthorized));
    }

    #[test]
    fn a_non_can_pickup_guardian_is_still_denied() {
        // Ana holds a guardianship edge but WITHOUT can_pickup — she is not on
        // any roster list, so she is denied (the flag is the gate, not the edge).
        let c = Collector {
            sub: Some("user:ana".into()),
            name: "Ana".into(),
        };
        assert_eq!(decide(&c, &roster()), Err(PickupDeny::NotAuthorized));
    }

    #[test]
    fn a_custody_hold_denies_even_an_authorized_guardian() {
        let mut r = roster();
        r.custody_hold = true;
        let c = Collector {
            sub: Some("user:sam".into()),
            name: "Sam".into(),
        };
        assert_eq!(decide(&c, &r), Err(PickupDeny::CustodyHold));
    }

    #[test]
    fn an_empty_roster_denies_as_unknown() {
        let c = Collector {
            sub: None,
            name: "Anyone".into(),
        };
        assert_eq!(
            decide(&c, &PickupRoster::default()),
            Err(PickupDeny::UnknownPerson)
        );
    }

    #[test]
    fn an_empty_collector_name_never_slips_through() {
        // A blank name must not match a blank slot — the guard `!name.is_empty()`.
        let mut r = PickupRoster::default();
        r.authorized_pickup_names = vec!["".into()];
        let c = Collector {
            sub: None,
            name: "".into(),
        };
        assert!(decide(&c, &r).is_err());
    }
}
