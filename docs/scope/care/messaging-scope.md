# Care scope — messaging (provisioned channels, membership reconciliation)

Status: scope (the ask). Promotes to `doc-site/content/public/care/` once shipped.
Owning repo: **this repo** (`rust/extensions/care/`) — **transport is lb channels
(shipped core), untouched.** This scope is only provisioning + membership.

Guardian↔staff messaging without building a messaging system: the care extension
*provisions* lb channels and keeps their membership derived from domain records — a
channel per child (its guardians + its room's staff), a channel per room (broadcast), and
a center announcements channel. lb owns transport, history, SSE, and the channel UI
widgets; care owns *who is in which room*.

## Goals

- **Provisioning:** creating a child/room/center auto-creates its channel (idempotent,
  named by convention `care-child-<id>` etc. — an opaque string to the core); archiving
  archives it (history retained — regulatory).
- **Derived membership, one reconciler:** guardianship edges (`receives_daily_feed` or a
  dedicated `messaging` flag — open question) + staff room assignments are the *only*
  membership source. One `reconcile(channel)` function, called from the same handlers that
  touch edges/assignments and from an idempotent sweep (repair, not primary — event-driven
  first, cron as healing).
- **Unlink = immediate removal:** the edge-unlink handler removes channel membership in
  the same breath as grants (`care-authz-scope.md`) — an ex-partner reading the child
  channel is the same severity as a feed leak.
- **Posting policy:** child + room channels: members post. Announcements: admin/staff
  post, guardians read (whatever lb channels expose for post-restriction — if nothing
  generic exists, that's a *small additive lb ask*, flagged loudly, not a care hack).

## Non-goals

- No message transport/threading/receipt features — lb channels as-is.
- No DMs v1 (a guardian messages the room via the child channel — keeps every exchange on
  the center's record, which directors want).
- No moderation tooling v1.

## Intent / approach

The whole design is rule-10-shaped: care reaches channels only through the generic granted
channel verbs; ids are conventions, membership is derivation. Rejected: building
care-specific messaging records (re-implements a shipped core surface); rejected:
letting admins hand-manage channel membership (drifts from custody reality — the
derivation *is* the feature).

## How it fits

- **Capabilities:** channel access rides lb's channel membership + caps; the reconciler
  runs with the extension's granted verbs. Guardian cap set includes channel read/post for
  member channels only.
- **State vs motion:** membership derivation = state handlers; messages = lb's motion.
- **Rule 9:** tests against real channels on a real node — post as Ana, assert Mia's-mum's
  reader never sees it.

## Example flow

1. Leo is created → `care-child-leo` channel exists; Sam + Ana bound (edges), Possums
   staff added (assignment).
2. Ana asks "did he nap?" in Leo's channel; room leader answers from the staff view; Sam
   sees the thread too (his edge). Mia's channel is separate — Ana has no path to it.
3. Custody change: admin unlinks Ana↔Leo → membership removed in the same handler → her
   next read 403s; history stays on the center's record.
4. Staff moves rooms → reconciler swaps their room + child channels.

## Testing plan

Cross-family matrix on channel read/post (the messaging rows). Cap-deny (guardian posting
to announcements → 403/denied per the post-policy mechanism), workspace isolation,
unlink → immediate removal (deny on next read, asserted), reconciler idempotency (double
events safe), sweep repairs a hand-broken membership, archive retains history but stops
posts.

## Risks & hard problems

- **Membership drift** is the leak vector — event-driven + sweep + the matrix test on
  every path that touches edges/assignments.
- **Post-restricted channels** may need a small generic lb addition (see Goals) — decide
  early; it gates announcements.
- History retention vs a removed guardian's *authored* messages (they stay — center
  record; confirm the privacy posture in open questions).

## Open questions

- Membership flag: reuse `receives_daily_feed` or a distinct `messaging` edge flag?
  (Recommend distinct — custody arrangements differ on exactly this.)
- Do lb channels support read-only membership today? Verify; file the additive lb ask if not.
- Announcements: one center-wide + per-center-record channels for multi-center orgs?

## Related

`care-scope.md` · `care-authz-scope.md` · `enrollment-invites-scope.md` (edges/assignments)
· lb `channels/channels-scope.md` · lb unified event stream.
