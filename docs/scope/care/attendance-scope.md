# Care scope — attendance (check-in/out, pickup authorization, kiosk, ratios)

Status: scope (the ask). Promotes to `doc-site/content/public/care/` once shipped.
Owning repo: **this repo** (`rust/extensions/care/`); kiosk devices ride **lb**'s
`auth-caps/api-keys-scope.md` (machine principals — verify shipped state before building).

The operational heartbeat: who is in the building right now, who brought them, who may take
them home. This is also the compliance surface — staff-to-child **ratios** are computed
from it, and regulators read it.

## Goals

- **`attendance_event`** (append-only): `check_in|check_out`, child, room, timestamp,
  performed-by (staff sub or kiosk key id), drop-off/pick-up **person** (a guardian or an
  authorized-pickup entry), optional note. Corrections are compensating events
  (`correction_of` ref), never edits — the record is a ledger.
- **Verbs:** `care.attendance.check_in/check_out` (staff + kiosk), `care.attendance.list`
  (admin all; staff room-scoped; guardian own-children via authz), `care.attendance.now`
  (current occupancy per room — the admin dashboard number), `care.attendance.correct`.
- **Pickup authorization at check-out:** the verb takes the collecting person; if they are
  neither a `can_pickup` guardian nor on the child's authorized-pickup list → **hard deny**
  with a staff-visible reason. Override = admin-capped verb, audited.
- **Kiosk mode:** a lobby tablet holds an lb **API key** granted exactly
  `mcp:care.attendance.check_in/check_out:call` + the minimal roster read. Guardian
  self-check-in via per-family PIN at the kiosk (phase 1.5).
- **Ratio read-out:** `care.attendance.now` returns per-room `{children, staff, ratio}`
  (staff presence from their own check-in events) — display + threshold warning only.

## Non-goals

- Staff *scheduling*/rostering (phase 3) — presence is recorded, shifts are not planned here.
- Regulatory export formats (data model must suffice; reports are a later slice).
- Geofencing/auto-checkin.

## Intent / approach

Append-only ledger + derived "now" view (state = SurrealDB events; the current-occupancy
read is a query, not a mutable counter — no drift). Feed fan-out (guardian sees the
check-in live) is `daily-feed-scope.md`'s bus event, emitted here. Rejected: mutable
`present: bool` on the child (loses history, races, unauditable).

## How it fits

- **Capabilities:** staff verbs deny-tested for guardians; kiosk key deny-tested for
  everything beyond its two verbs (the api-keys pattern proves itself here).
- **API shape:** event-append verbs + list (time/room filters) + one derived read; live
  updates ride the feed's bus subject; batch N/A.
- **State vs motion:** events are state; "Leo checked in" to a phone is motion (feed/push).

## Example flow

1. 08:02 staff taps Leo present (or Sam PINs at the kiosk) → event appended
   (performed-by recorded) → feed/push to Sam + Ana.
2. 17:20 grandma (authorized-pickup entry, not a guardian) collects Leo → staff selects
   her → allowed, named in the event. A stranger/unlisted parent → deny + reason;
   admin override possible, audited.
3. Admin dashboard: Possums 11 children / 2 staff — amber at the configured ratio threshold.

## Testing plan

Cap-deny (guardian can't `check_in`; kiosk key can't read logs/menus — the full deny sweep),
workspace isolation, cross-family matrix rows for `list`. Plus: unauthorized pickup denied +
audited override, correction events (net occupancy right after a wrong tap), `now` correct
across in/out/correct sequences, kiosk key revocation kills the tablet instantly (api-keys
instant-revoke asserted end-to-end).

## Risks & hard problems

- **The pickup deny is a child-safety control** — it must be unbypassable in the UI and
  loud, but staff must have the audited override for real-world edge cases (custody
  disputes land exactly here; `custody_notes` surfaces to staff at check-out).
- Kiosk = a shared unattended device — key scope tiny, PIN rate-limited, no child data
  browsable from it beyond the roster it needs.

## Open questions

- PIN per family or per guardian? (Recommend per guardian — events should name the person.)
- Does staff presence use the same event table (recommended) or a staff-specific one?

## Related

`care-scope.md` · `care-authz-scope.md` · `enrollment-invites-scope.md` (pickup list) ·
`daily-feed-scope.md` (fan-out) · lb `auth-caps/api-keys-scope.md`.
