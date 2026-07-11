# Care scope — the daily feed (logs, photos, live updates, push)

Status: scope (the ask). Promotes to `doc-site/content/public/care/` once shipped.
Owning repo: **this repo** (`rust/extensions/care/`), consuming **lb**'s
`files/media-scope.md` (photos — upstream, must ship first) and
`inbox-outbox/push-target-scope.md` (notifications — upstream).

The guardian home screen and the teacher's highest-frequency surface: a per-child stream of
typed entries — meals, naps, diapers, activities, photos, notes, incidents, medications —
written in two taps from a phone in a busy room, arriving live on a parent's lock screen.

## Goals

- **`daily_log`** entries: type (`meal|nap|diaper|activity|photo|note|incident|medication`),
  child id(s) (one tap can log lunch for the whole room — fan-out to per-child entries),
  author, ts, type-specific payload (nap start/end, meal portions, `media_id`s from the lb
  media path), room. Incidents carry required fields (what/where/action) and a
  guardian-acknowledgement flag; medications record dose + witness.
- **Verbs:** `care.log.add` (staff; multi-child), `care.log.list` (cursor-paged per child,
  guardian via authz; staff per room), `care.log.correct` (compensating, audited — like
  attendance), `care.feed.watch` (the live feed: SSE over a per-child bus subject).
- **Photos** ride lb media: staff upload via the resumable path → `media_id` on the entry;
  the feed renders `?variant=thumb`; media reads are reach-checked (same authz).
- **Push:** entry types map to notification policy (incident = always push; meal = feed
  only, configurable per guardian via edge flag + prefs) → `notify.send` to the guardians
  holding `receives_daily_feed` edges.
- **Day summary read:** `care.log.day(child, date)` — the "Leo's day" rollup the UI and the
  AI ("summarize Leo's week") both consume.

## Non-goals

- Learning/milestone tagging on entries (phase 3 rides the same records — the `type` enum
  is extensible, additive).
- Video v1 (media scope stores it; the feed shows photos first).
- Guardian-authored entries (guardians read + acknowledge; messaging is the reply channel).

## Intent / approach

Entries are state (SurrealDB); "a new entry exists" is motion (one bus subject per child,
filtered at emit per `care-authz-scope.md`; SSE via the gateway's stream). Push is a
must-deliver outbox effect — a missed incident notification is not acceptable, so it is
never raw pub/sub. Rejected: modeling the feed as lb channel messages — a feed entry is
structured domain data (queried, corrected, summarized, regulated), not chat; channels stay
the conversation surface (`messaging-scope.md`).

## How it fits

- **Capabilities + reach:** `care.log.add` staff-only (deny-tested); every read through the
  authz chokepoint; media bytes reach-checked at the serve route.
- **API shape:** append + cursor list + one derived read + **watch** (the live-feed shape);
  multi-child add is a small bounded fan-out (≤ room size), synchronous by design.
- **State vs motion / durability:** exactly the split above; push via outbox.

## Example flow

1. 11:30, staff: one "lunch" entry for 8 tapped children (Leo's shows the peanut-free
   substitution from `menus-scope.md`) + a photo (media ticket → chunks → commit).
2. Fan-out: 8 `daily_log` rows; bus events per child; Sam's open PWA appends live; Ana
   (locked phone) gets the configured push.
3. 15:10 incident (scraped knee) → required fields → **always-push** to both guardians →
   Ana acknowledges; the acknowledgement is on the record for the center's file.
4. Mia's mum sees none of it — emit-side filter + authz on list/get/media.

## Testing plan

Cross-family matrix rows for `list/get/day/watch` **and the media URL** (the classic leak:
record filtered, photo URL guessable — must 403). Cap-deny (guardian `log.add` → 403),
workspace isolation, `receives_daily_feed: false` edge gets neither feed nor push,
incident push always fires (outbox retry asserted via the recording provider fake),
multi-child fan-out atomicity, correction events, cursor stability under concurrent adds.

## Risks & hard problems

- **The photo-URL leak** — media reach-checks must be the same authz decision, tested per
  variant.
- **Room-tablet offline gaps** — entries composed offline must queue and land with true
  timestamps (lb sync posture; flagged in master scope open questions).
- **Photo consent** (a guardian forbids photos): a child-level flag the media attach path
  enforces at write, not render — decide in open questions, do not defer past first ship.

## Open questions

- Consent model: per-child flag with per-edge override? (Recommend: child-level
  `photos_allowed` v1, set from enrollment paperwork.)
- Does `care.log.day` also emit a weekly digest push (quiet-hours-friendly) v1?
- Incident guardian-ack: required within N hours with re-push, or best-effort v1?

## Related

`care-scope.md` · `care-authz-scope.md` · `menus-scope.md` · `attendance-scope.md` ·
lb `files/media-scope.md` · lb `inbox-outbox/push-target-scope.md` · lb unified event
stream (SSE).
