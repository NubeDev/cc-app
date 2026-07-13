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

- [x] Matrix rows for `list/get/day/watch` **and the media URL** (`matrix_daily_feed`:
      guardian sees only her child's rows, stranger empty, day 403 with no leak, the
      media-id never surfaces across families).
- [ ] Unlink mid-stream → the **open SSE stream terminates** — **DEFERRED to m10** (lb
      `bus.watch` is workspace-wide + has no mid-stream revoke; `feed.watch` v1 is
      reach-check-at-subscribe; lb work first —
      `docs/debugging/authz/bus-watch-unscoped-and-no-midstream-revoke.md`).
- [x] `receives_daily_feed: false` edge → no feed, no push (`authz::feed_recipients`);
      incident/medication push always (`push::decide` must-deliver). Outbox-retry +
      quiet-hours assertions are lb-outbox/prefs surfaces → live-node/m10.
- [x] Multi-child fan-out atomic (validate-all-before-any-write); corrections
      (compensating); cursor `(at,row_id)` stability.
- [ ] E2E: staff two-tap (with photo) → guardian PWA appends live → locked phone push —
      **DEFERRED to a live-node E2E pass (m10)**. Photo-consent child never gets media:
      **[x]** proven in-lib (`add.rs::photo_attach_blocked_for_non_consenting_child`).
- [x] **Incident push asserted in both languages** (`matrix_daily_feed`: en + es titles
      and bodies differ, child interpolated in both, not the raw key). The es feed UI E2E
      run is **DEFERRED to m10** with the motion E2E.
- [x] Open questions resolved: child-level `photo_consent` v1 (enforced at write);
      weekly digest deferred; incident-ack best-effort v1 (recorded).
- [x] STATUS.md moved.

## Subagent notes

Fix the `daily_log` schema + bus subject shape first (orchestrator). Fan out: verb agents,
media-path agent, push-policy agent, two UI agents. Reviewer briefs: (1) reach a photo
byte without an edge, (2) keep an SSE stream alive after unlink.

## Sources

`../scope/care/daily-feed-scope.md` · lb media + push-target scopes ·
`../scope/care/care-authz-scope.md` · guardian/teacher persona docs ·
`../scope/personas/guardian/profile.md` (prefs/devices — build its Profile-tab slice here
or as a 09 rider; don't drop it).
