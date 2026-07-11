# Personas — the per-user view of the product scope

One folder per persona, one doc per **use case** (a journey the persona completes). This
layer answers *"what does this person actually do in the app, end to end?"* — the feature
scopes under [`../care/`](../care/) stay the source of truth for the domain model, verbs,
and data; a use-case doc **links into them and never restates them**. If a use case needs
something no feature scope covers, that is a gap: extend the feature scope (or write a new
one), then link it — don't grow the domain here.

Terminology mapping (folder name → scope term → who it is):

| Folder | Scope term | Who |
|---|---|---|
| [`admin/`](admin/) | owner/admin | The center owner or director — possibly multi-center. Full `care.*` reach. |
| [`teacher/`](teacher/) | staff | A room leader/educator — room-scoped verbs, phone/tablet in a busy room. |
| [`guardian/`](guardian/) | guardian | The parent/carer ("the user") — edge-scoped reads, phone-only, feed-first. |

Kiosk (the lobby tablet) is a *device principal*, not a persona — it lives in
[`../care/attendance-scope.md`](../care/attendance-scope.md).

## Rules for this layer

- **Caps decide the persona, never the UI** — a persona is what the folded capability set
  reaches (master scope §Personas). No use case may imply a UI-side role switch.
- **Every read a use case makes goes through the authz chokepoint**
  ([`../care/care-authz-scope.md`](../care/care-authz-scope.md)); every use case names its
  deny cases, and those become cross-family matrix rows.
- Each use-case doc keeps the shape: **Goal · Journey · Verbs & screens · Deny / edge
  cases · Source scopes**. One use case per file (FILE-LAYOUT rules apply to docs too).

## Index

- **Admin:** [setup-center](admin/setup-center.md) ·
  [enroll-family](admin/enroll-family.md) ·
  [invites-onboarding](admin/invites-onboarding.md) ·
  [operations-oversight](admin/operations-oversight.md) ·
  [menu-planning](admin/menu-planning.md) ·
  [announcements](admin/announcements.md)
- **Teacher:** [check-in-out](teacher/check-in-out.md) ·
  [daily-logging](teacher/daily-logging.md) ·
  [serving-meals](teacher/serving-meals.md) ·
  [room-messaging](teacher/room-messaging.md)
- **Guardian:** [join-by-invite](guardian/join-by-invite.md) ·
  [daily-feed](guardian/daily-feed.md) ·
  [menus](guardian/menus.md) ·
  [messaging](guardian/messaging.md) ·
  [profile](guardian/profile.md)

Billing use cases (admin invoicing, guardian pay) are **deliberately absent** — billing is
the last phase ([`../billing/billing-scope.md`](../billing/billing-scope.md), deferred).
