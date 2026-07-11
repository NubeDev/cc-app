# Admin use case — center announcements

**Goal:** one-to-everyone broadcast ("closed Friday", "photo day") that every guardian and
staff member sees, without opening a reply free-for-all.

## Journey

1. Admin posts to the center announcements channel (provisioned automatically per
   center by the messaging scope).
2. All workspace members are read members; admin/staff may post, guardians read-only.
3. Urgent announcements ride the push policy (push-target scope) so a locked phone
   still hears about the closure.

## Verbs & screens

- lb channel post verbs (granted, generic) into the announcements channel; membership
  derived by the messaging reconciler, never hand-managed.
- Screen: admin → Announcements (compose + history); guardians see it in Messages.

## Deny / edge cases

- Guardian posting to announcements → denied by the channel post-policy. **Blocking
  dependency:** lb channels must expose a generic read-only/post-restricted membership —
  verify, and if absent it is the *small additive lb ask* the messaging scope flags. This
  use case does not ship on a care-side hack.
- Multi-center org: one announcements channel per center (messaging scope open
  question — resolve before build).

## Source scopes

[`../../care/messaging-scope.md`](../../care/messaging-scope.md) (posting policy, the lb
ask) · lb `channels/channels-scope.md` · lb `inbox-outbox/push-target-scope.md`.
