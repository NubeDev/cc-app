# Teacher persona — the room's day, one-handed

"Teacher" = the scope's **staff** persona: room-scoped operational verbs — attendance,
daily-log write, menu read, child read *for their assigned room(s)*, channel posting. No
enrollment, no member admin, no billing (`care.child.update` → 403 is the canonical deny).
Reach comes from the staff room assignment via the one authz chokepoint
(`reachable_rooms`).

The UI bar (mobile-shell scope): a phone or shared tablet, in a room full of toddlers —
every core flow must survive one hand and two taps.

## Use cases (a day in the room)

1. [check-in-out.md](check-in-out.md) — morning arrivals, afternoon handover, the pickup
   authorization check.
2. [daily-logging.md](daily-logging.md) — the two-tap logging flow: meals, naps, diapers,
   activities, photos, incidents, medications.
3. [serving-meals.md](serving-meals.md) — the serving view with red allergy flags and
   substitutions.
4. [room-messaging.md](room-messaging.md) — answering guardians in child channels,
   posting to the room channel.

Deferred: lesson plans/assessments (phase 3), shift scheduling (phase 3), offline
composition on room tablets (master-scope open question — the flows must degrade to
skeleton+retry, never a blank screen).

## Open questions (teacher-specific)

- Shared room tablet vs personal phone: is a fast user-switch (PIN?) needed on a shared
  device, or is the tablet a kiosk-adjacent posture? (Events must name the person.)
- Does the teacher see custody_notes proactively at check-out, or on tap? (Attendance
  scope surfaces it at check-out — confirm the UI treatment.)
