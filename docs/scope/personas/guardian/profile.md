# Guardian use case — own profile, notifications, devices

**Goal:** the guardian controls what is theirs — contact details, notification
preferences, registered devices — and nothing that is the center's.

## Journey

1. Profile tab: update own phone/email/address (`care.guardian.*` self-scoped update —
   emergency-contact accuracy is why this is self-serve).
2. Notification prefs: per-type push toggles (meal/nap = feed-only vs push), quiet hours
   (push-target prefs axis). Incidents are always-push regardless.
3. Devices: this phone registered on PWA install (`device.register`); list and remove
   own devices (`device.list/remove` — self-only).
4. Sees their children's profiles read-only (allergies, room, schedule) — edits go
   through the center (admin owns the safety data).

## Verbs & screens

- Self-scoped guardian contact update; lb `device.register/list/remove`; push prefs
  (`push_muted` axis + per-type policy).
- Screen: guardian → Profile tab.

## Deny / edge cases

- Guardian cannot edit child records, edges, or their own edge flags (`can_pickup` is
  the center's + custody reality's call, not self-serve) → 403.
- Device management is self-only — one guardian can't list another's devices.
- Contact-detail changes should be auditable (emergency contact history) — cheap now,
  brutal to retrofit (master scope §Risks, regulatory posture).

## Source scopes

[`../../care/enrollment-invites-scope.md`](../../care/enrollment-invites-scope.md) ·
lb `inbox-outbox/push-target-scope.md` (devices, prefs) ·
[`../../care/care-authz-scope.md`](../../care/care-authz-scope.md).
