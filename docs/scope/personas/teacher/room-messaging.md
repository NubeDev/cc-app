# Teacher use case — answer guardians, post to the room

**Goal:** every guardian question lands where the room team sees it, and every exchange
stays on the center's record.

## Journey

1. Ana asks "did he nap?" in Leo's channel → the room team sees it in the staff view →
   the room leader answers; Sam (the other guardian) sees the thread too.
2. Room broadcast: "wear gumboots tomorrow" posted to the Possums room channel — all the
   room's guardians read it.
3. Staff moved to another room → the reconciler swaps their channel memberships with the
   assignment; no hand-managed lists.

## Verbs & screens

- lb channel read/post (granted, generic) on child + room channels; membership derived
  from staff room assignments by the messaging reconciler.
- Screen: staff → Messages (child threads for the room, room channel).

## Deny / edge cases

- Staff reach only their assigned rooms' channels; posting to announcements follows the
  post-policy (admin/staff only).
- No DMs v1 — a guardian↔teacher exchange always lives in the child channel (deliberate:
  the center's record).
- An unlinked ex-guardian is out of the child channel before the teacher's next reply
  (unlink = immediate removal — same severity as a feed leak).

## Source scopes

[`../../care/messaging-scope.md`](../../care/messaging-scope.md) ·
lb `channels/channels-scope.md` ·
[`../../care/care-authz-scope.md`](../../care/care-authz-scope.md).
