# Guardian persona — the parent's phone

The **guardian** (the "user"): the narrowest cap set in the product — read own children
(via live guardianship edges), read their feed/menus, ack attendance, message in their
children's channels, manage own contact info. Phone-only posture, feed-first home, PWA on
the lock screen.

**Everything a guardian sees is derived from live guardianship edges** — CLAUDE.md rule 7,
the product's defining invariant. Blended families fall out of the model: Sam sees Leo
*and* Mia (two edges, child switcher in the header); Ana sees only Leo; nobody ever sees a
child they hold no edge to — not in a list, a photo URL, a channel, or a push.

## Use cases

1. [join-by-invite.md](join-by-invite.md) — email → account → child's feed in one flow.
2. [daily-feed.md](daily-feed.md) — the home screen: live day stream, incident
   acknowledgement, push.
3. [menus.md](menus.md) — the week's food, own child's substitutions inline.
4. [messaging.md](messaging.md) — message the room via the child's channel.
5. [profile.md](profile.md) — own contact details, notification preferences, devices.

Deferred: paying invoices (billing phase — **last**), guardian-authored feed entries
(guardians read + ack; messaging is the reply channel), native app-store app (PWA v1).

## Open questions (guardian-specific)

- Multi-workspace guardian (children at two unrelated orgs): shell workspace-switch UX —
  lb's normal multi-workspace login, but confirm the mobile flow is one tap.
- Per-edge `receives_daily_feed: false` (e.g. a nanny with pickup rights but no feed):
  what does that guardian's home screen show instead of the feed?
