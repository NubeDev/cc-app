# UI scope — the thin mobile shell + the care extension UI

Status: scope (the ask). Promotes to `doc-site/content/public/ui/` once shipped.
Owning repos: shell chrome = **lb** (`frontend/minimal-shell-scope.md` — upstream, the
preferred path); this repo owns `ui/` (the shell *instance* + config) and
`rust/extensions/care/ui/` (**100% of the product screens**).

The product ships no lb shell: `ui/` is a thin host (login, invite-accept, workspace pick,
full-screen mount, PWA) and every screen a user recognizes as "the app" is the care
extension's federated UI — mobile-first, Tailwind + shadcn tokens, `defineRemote` only.

## Goals

- **Shell (`ui/`):** consume lb's minimal-shell package the moment it ships; config only
  (gateway URL, home = the care ext page, branding blob). If care's build outpaces the lb
  package, `ui/` starts as the *smallest possible* interim host (login + mount + SSE,
  hard ~15-file budget) whose parts are contributed upstream into that package — the
  interim shell is a loan to lb, not a fork.
- **Care ext UI — three persona surfaces behind one `defineRemote` page** (persona = what
  the folded caps reach, never a UI-side role switch):
  - **Guardian (phone-first):** bottom tabs Feed / Menu / Messages / Profile; multi-child
    switcher (Sam sees Leo + Mia); live feed via SSE; push-tap deep-links (`deep_link`
    from the push scope).
  - **Staff (phone/tablet):** room roster with tap check-in/out; the **two-tap logging
    flow** (pick children → pick type → done) — the make-or-break interaction; serving
    view with red allergy flags; room channel.
  - **Admin (desktop-friendly, still responsive):** centers/rooms, enrollment + invites,
    menu planner, occupancy/ratio dashboard, announcements.
- **Widgets** exported for the feed entry types (meal/nap/photo/incident renderers) so the
  AI/channel surfaces can render them (the frames-in contract).
- **Mobile discipline:** one-handed reach for guardian+staff paths, offline-tolerant
  (skeleton + retry, no blank screens), image thumbs only in lists, installable PWA.

## Non-goals

- No native RN app v1 (PWA; the lb `app/` line is the later path — decision recorded in
  the master scope's open questions).
- No lb chrome (sidebar/dock/admin console) ever in this product's surface.
- No hand-rolled mount/CSS: `remoteEntry.tsx` = one `defineRemote(...)` (CLAUDE.md rule 6).

## Intent / approach

One extension page owning the viewport with in-ext routing (tabs), rather than N ext pages
in a host nav — keeps the shell contract minimal and the product feel native. Rejected:
vendoring the lb shell (rubix-ai's compromise) — desktop-admin-shaped, and subtraction is
more work than the thin host; the lb minimal-shell scope exists to end that pattern.

## How it fits

- **Capabilities:** the UI renders from folded caps + authz-scoped data; no cap probing, no
  hidden-but-mounted admin routes. Deny renders the same not-available state everywhere.
- **Rule 9:** UI tests run against a real node (Playwright + the gateway harness);
  screenshots of real seeded families, no fixture JSON.
- **Rule 10 both directions:** the shell treats the care ext id as config data; the ext
  uses only SDK contracts (`extTailwindPreset`, scoped tokens, no `:root`/preflight).

## Example flow

Sam taps the incident push → PWA opens → deep-link to Leo's feed entry → acknowledges →
switches to Mia (tab header) → checks her room's Friday menu → messages Mia's room leader.
Three domains, zero lb chrome visible.

## Testing plan

Playwright against a real node: login → guardian feed renders seeded entries live (SSE
append asserted); staff two-tap log lands and appears on the guardian screen; cross-family
UI check (Ana's session shows no trace of Mia — nav, lists, switcher); deny state for a
capless page; CSS isolation (host styles byte-identical after ext mount — the SDK contract
test); PWA manifest/installability; small-viewport (360px) layout for every guardian/staff
screen.

## Risks & hard problems

- **The two-tap staff flow** decides adoption; prototype it first, on a real phone, before
  the admin surfaces.
- Interim-shell scope creep if the lb package lags — the file budget + "contribute
  upstream" rule is the fence.
- Photo-heavy feed performance on mid-range phones — thumbs, virtualization, and the media
  variant discipline from day one.

## Open questions

- Persona tabs: one page with cap-driven tab sets (recommended) or three ext pages?
- Do admin surfaces get a desktop-width layout pass v1 or ship responsive-only?
- Which shadcn theme tokens become the product brand (blocked on naming/branding)?

## Related

`../care/care-scope.md` · every care sub-scope (the screens they imply) · lb
`frontend/minimal-shell-scope.md` · lb `auth-caps/invites-scope.md` (accept screen) ·
`@nube/ext-ui-sdk` (`defineRemote`, presets) · lb push-target (deep links).
