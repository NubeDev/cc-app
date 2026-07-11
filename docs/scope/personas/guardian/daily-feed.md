# Guardian use case — the live daily feed

**Goal:** the answer to "how's Leo's day going?" without calling the center: check-in,
meals, naps, photos, incidents — live on the phone, with the important ones pushed.

## Journey

1. 08:02: Leo checked in → Sam's lock screen (if he's opted into that push type) and his
   open PWA both show it live (`care.feed.watch` over SSE).
2. 11:30: lunch entry with the peanut-free substitution noted; fingerpainting photo
   renders as a thumb, taps to full size.
3. 15:10: incident (scraped knee) → **always pushed**; Sam taps the notification → deep
   link to the entry → acknowledges; the ack is on the center's record.
4. Child switcher: Sam flips to Mia's day (his other edge). "Leo's day" rollup
   (`care.log.day`) gives the end-of-day summary — the same read the AI uses for
   "summarize Leo's week".

## Verbs & screens

- `care.log.list` (cursor-paged), `care.log.day`, `care.feed.watch` (SSE), media serve
  `?variant=thumb`, push via `receives_daily_feed` edges + prefs.
- Screen: guardian home = Feed tab (mobile-shell scope), pull-to-refresh + live append.

## Deny / edge cases

- Every read edge-scoped through the authz chokepoint; the **photo URL** is reach-checked
  too (the classic leak: record filtered, media URL guessable — must 403).
- `receives_daily_feed: false` edge → no feed, no push.
- Unlink mid-day → the open SSE stream terminates, not just future subscribes.
- Quiet hours / per-type push prefs honored (incident overrides to always-push).
- Photo-consent: a no-photos child never has media attached in the first place.

## Source scopes

[`../../care/daily-feed-scope.md`](../../care/daily-feed-scope.md) ·
[`../../care/care-authz-scope.md`](../../care/care-authz-scope.md) ·
lb `files/media-scope.md` · lb `inbox-outbox/push-target-scope.md`.
