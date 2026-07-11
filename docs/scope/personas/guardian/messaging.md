# Guardian use case — message the room

**Goal:** ask the teacher a question and get an answer, in the one place the center keeps
its record — the child's channel.

## Journey

1. Ana opens Messages → Leo's channel (its members: Leo's guardians + Possums staff) →
   "did he nap?" → the room leader answers; Sam sees the thread too.
2. Room channel: gumboots reminder from the teacher; announcements from the center —
   both read-only surfaces for the guardian to differing degrees (room: members post;
   announcements: read-only).
3. Sam's Messages list shows channels for both his children; Ana's shows Leo's only.

## Verbs & screens

- lb channel read/post (granted) on member channels; membership derived from
  guardianship edges by the care reconciler.
- Screen: guardian → Messages tab (channel list per edge, thread view).

## Deny / edge cases

- No path to any channel of a child without an edge — read *or* post (cross-family
  matrix messaging rows: Ana never sees Mia's channel exists).
- Posting to announcements → denied by post-policy.
- Unlink → removed from the child's channel immediately; her authored history stays on
  the center's record (retention posture — confirm privacy stance, messaging scope open
  question).
- No DMs v1 — deliberate (everything on the record).

## Source scopes

[`../../care/messaging-scope.md`](../../care/messaging-scope.md) ·
[`../../care/care-authz-scope.md`](../../care/care-authz-scope.md) ·
lb `channels/channels-scope.md`.
