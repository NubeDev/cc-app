# Product

## Register

product

## Users

Three personas, defined in `docs/scope/personas/` (the authoritative source):

- **Guardian (the parent)** — the primary and largest persona. On a phone, one-handed,
  often distracted (holding a child, on a commute); checks the daily feed, menus,
  messages. Spanish- or English-speaking (both are first-class, CLAUDE.md rule 8).
- **Teacher (staff)** — phone or shared tablet in a busy room full of toddlers; the
  two-tap logging flow and check-in/out are the whole job. Speed and legibility beat
  density.
- **Admin (owner/director)** — desktop-friendly surfaces (enrollment, dashboards, menu
  planner) but still responsive; may run multiple centers.

## Product Purpose

A brightwheel/lillio-class childcare management platform (attendance, daily feed with
photos, messaging, menus, enrollment, invites) built as extensions on the lb core.
Success: a real center onboards and parents live in the feed daily. The 2026 market bar
is *simplicity* — the winners are the apps a distracted parent and a busy room-leader can
use one-handed (`docs/scope/care/care-scope.md` §Market reference).

## Brand Personality

**Calm, trustworthy, effortless.** The product handles people's children — it must feel
as considered and reliable as a first-party Apple app: modern iOS design language
(generous whitespace, large-title hierarchy, continuous rounded corners, soft depth,
fluid motion), warmth carried by photos of the children — the content is the color.

## Anti-references

- Desktop admin dashboards pretending to be mobile apps (the vendored-lb-shell look this
  product explicitly rejected — sidebars, docks, dense tables on phones).
- SaaS-generic UI: identical card grids, hero metrics, gradient text, cream/beige body
  backgrounds, uppercase tracked eyebrows.
- Toy-like "kids app" aesthetics (primary-color confetti, cartoon mascots) — the users
  are adults; the tone is trusted-professional, not playful-juvenile.
- Skeuomorphic fake-native chrome: it's a PWA — iOS-*inspired* discipline, never a
  counterfeit iPhone frame.

## Design Principles

1. **One-handed, two taps.** Every guardian/staff core path reachable one-handed on a
   360px phone; the staff logging flow is two taps. Reach and speed beat density.
2. **The feed is the product; chrome disappears.** Photos and the child's day are the
   visual interest; UI recedes (neutral surfaces, restrained accent).
3. **Calm under pressure.** Incidents, allergy flags, and pickup denies are the moments
   that matter — loud where safety demands it (red allergen flags, unmissable denies),
   quiet everywhere else.
4. **Looks designed at every width.** Mobile-first, but admin on a laptop gets real
   multi-column layouts, not a stretched phone UI (CLAUDE.md rule 9).
5. **Both languages, both themes, always.** en/es and dark/light are correctness
   criteria, not variants (CLAUDE.md rules 8–9).

## Accessibility & Inclusion

WCAG 2.1 AA baseline: shadcn/ui's Radix primitives carry keyboard/focus/ARIA — keep
them, never rebuild them. Body text ≥4.5:1 in both themes; touch targets ≥44px on
guardian/staff surfaces; `prefers-reduced-motion` honored on every animation; full
en + es catalogs (rule 8). Assumed default — tighten if a market requires more.
