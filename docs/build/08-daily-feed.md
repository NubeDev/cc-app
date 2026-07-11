# Milestone 08 — the daily feed (logs, photos, SSE, push)

The guardian home screen and the teacher's two-tap flow — the biggest milestone; budget
2–3 sessions (suggested seams: verbs+bus / media+push / UI).
Scope: [`../scope/care/daily-feed-scope.md`](../scope/care/daily-feed-scope.md).

## Entry gate

- [ ] Milestones 05 (guardians logged in), 06 (attendance events to fan out) and 07
      (substitutions shown on meal logs) closed.
- [ ] lb media + push-target on the pinned tag; **photos only** until lb media ships
      Range support (video is a scope non-goal v1 anyway — enforce it).

## Work items

- [ ] `daily_log` records (the 8 types; type-specific payloads — nap start/end, meal
      portions, incident required-fields + guardian-ack flag, medication dose + witness);
      multi-child add fans out to per-child rows atomically.
- [ ] Verbs: `care.log.add` (staff, multi-child), `care.log.list` (cursor-paged),
      `care.log.correct` (compensating), `care.log.day` (the rollup the UI **and the AI**
      consume), `care.feed.watch` (SSE over per-child bus subjects, **filtered at emit**
      per the 02 decision).
- [ ] Photos: lb media begin/chunks/commit → `media_id` on the entry; feed renders
      `?variant=thumb`; **media serve is reach-checked with the same authz decision**;
      photo-consent flag blocks attach **at write**.
- [ ] Push: type → policy mapping (incident = always; others per edge flag + prefs +
      quiet hours) → `notify.send` to `receives_daily_feed` edge-holders via the outbox
      (must-deliver); push deep-links to the entry. **Push title/body rendered
      server-side in each recipient's locale** — Sam gets English, Ana gets Spanish, for
      the same incident. Log-type labels are enum keys rendered per locale.
- [ ] Guardian UI: Feed tab — live SSE append, child switcher, day rollup, incident ack
      ([`../scope/personas/guardian/daily-feed.md`](../scope/personas/guardian/daily-feed.md)).
      Staff UI: the **two-tap logging flow** — children multi-select → type → done
      ([`../scope/personas/teacher/daily-logging.md`](../scope/personas/teacher/daily-logging.md)) —
      prototype on a real phone before polishing anything else (mobile-shell scope §Risks).
- [ ] Feed widgets exported (meal/nap/photo/incident renderers) via `defineRemote`.

## Exit gate

- [ ] Matrix rows for `list/get/day/watch` **and the media URL** (record filtered but
      photo URL guessable = the classic leak → must 403, tested per variant).
- [ ] Unlink mid-stream → the **open SSE stream terminates**, not just future subscribes.
- [ ] `receives_daily_feed: false` edge → no feed, no push; incident push always fires
      (outbox retry asserted via the recording PushProvider fake); quiet hours honored.
- [ ] Multi-child fan-out atomic; corrections; cursor stability under concurrent adds.
- [ ] E2E: staff two-tap log (with photo) → guardian's open PWA appends live → locked
      phone gets the push record. Photo-consent child never gets media attached.
- [ ] **The incident push asserted in both languages** (recording PushProvider fake:
      `en` recipient and `es` recipient, same incident, each localized); feed UI E2E run
      once as an `es` user; date/time formatting locale-correct.
- [ ] Open questions resolved: child-level `photos_allowed` v1 (recommended); weekly
      digest deferred unless trivial; incident-ack best-effort v1 (record the choice).
- [ ] STATUS.md moved.

## Subagent notes

Fix the `daily_log` schema + bus subject shape first (orchestrator). Fan out: verb agents,
media-path agent, push-policy agent, two UI agents. Reviewer briefs: (1) reach a photo
byte without an edge, (2) keep an SSE stream alive after unlink.

## Sources

`../scope/care/daily-feed-scope.md` · lb media + push-target scopes ·
`../scope/care/care-authz-scope.md` · guardian/teacher persona docs ·
`../scope/personas/guardian/profile.md` (prefs/devices — build its Profile-tab slice here
or as a 09 rider; don't drop it).
