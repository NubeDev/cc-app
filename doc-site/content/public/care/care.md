# Care — childcare management on lb

Care is a **brightwheel/lillio-class childcare-management platform** — enrollment,
attendance, a photo daily feed, menus with allergy-aware substitutions, and
guardian↔staff messaging — for owners/admins of one or more childcare centers, their
staff, and the guardians of enrolled children.

It is built **on lb**: the core provides identity, the capability wall, the store,
channels, and the extension runtime; **all childcare domain logic and 100% of the
product UI are lb extensions**. Mobile-first — a parent lives in this on a phone.

> **Status: phase 1 shipped.** The scope of record is
> [`docs/scope/care/care-scope.md`](../../../../docs/scope/care/care-scope.md); the
> build history is under `docs/sessions/care/`.

## Who it serves

- **Owner / admin** — sets up centers and rooms, enrolls children, creates guardian
  and staff records, sends invites, and sees the whole roster. May run more than one
  center.
- **Staff (teacher)** — runs a room day: check children in and out, post the daily
  feed (meals, naps, activities, photos, incident and medication logs), see their
  room's menu and allergy substitutions.
- **Guardian** — child-scoped and mobile-first: lives on the live daily feed for
  their own children, sees this week's menu and their child's substitutions, picks up
  (custody-aware pickup gate), and messages their child's room.

## What phase 1 does

- **Enrollment** — centers, rooms, child profiles (DOB, allergies, medical notes,
  photo consent), guardian records, and the guardianship edges that connect a
  guardian to a child. Per-room waitlist.
- **Invites** — a guardian or staff member is invited by email, accepts, and sets
  their own password; the accept page follows the person's language.
- **Attendance** — an append-only check-in / check-out ledger, staff-assisted or
  kiosk, a custody-aware pickup gate, live room occupancy and staff-ratio view.
- **Daily feed** — staff post entries (naps, meals, activities, photos, incident and
  medication logs) that fan out to each tapped child; guardians see their own
  children's feed live, with must-deliver push for incidents and medication.
- **Menus** — a per-room weekly plan with per-child allergy-aware substitutions (a
  food-safety control), and a guardian view of only their own child's week.
- **Messaging** — per-child and per-room channels plus center-wide announcements
  (guardians read announcements; staff and admins post).

Everything is reachable as `care.*` MCP tools, so an AI agent can drive the exact
same surface a person can.

## The one rule everything bends around — guardian isolation

**A guardian may only ever see records for children they have a live guardianship
edge to.** Every guardian-visible read goes through a single authorization
chokepoint, and severing a guardianship edge revokes *every* access surface in the
same breath — the durable reads, the live feed stream, the photo bytes, the messaging
channel, and push. A cross-family leak is the worst bug this product can have, and the
test suite includes a scripted "edge-change drill" that proves the collapse end to
end.

## Bilingual from day one

Every user-facing surface — the UI, invite emails, push notifications, and every
message the system emits — ships in **English and Spanish**, selected per user. There
are no hardcoded user-facing strings: records store keys and enums, and the language
catalogs hold the words, checked for completeness in CI.

## Design

Mobile-first (a phone at 360px is the design target) and good on a laptop, dark and
light following the system with a user toggle, built on shadcn/ui in a modern-iOS
visual language (large titles, bottom tabs and sheets, soft depth). A distracted
parent and a busy room leader can both use it one-handed.

## Not in phase 1

Billing and payments (phase 2, built last as its own `care-billing` extension), video
in the feed, kiosk PIN self-check-in, admissions forms, and reporting.
