# Design

Seed DESIGN.md (pre-implementation — no code yet). Direction: **modern iOS, on
shadcn/ui**. Re-run `/impeccable document` once real tokens exist to capture actuals.
Binding constraints from CLAUDE.md rule 9 and `docs/scope/ui/mobile-shell-scope.md`
§"UI stack": shadcn/ui only, semantic tokens only in extension UI, host owns
`:root{}`/`.dark{}`, mobile-first + laptop-good, dark + light from day one.

## Theme

iOS-modern discipline expressed through shadcn's token system — not a native
counterfeit. Content-forward: photo-rich feed on calm neutral surfaces. Light and dark
are peers; default follows the system, user toggle persisted.

## Colors

OKLCH. **Restrained strategy:** true neutrals (chroma ≈ 0, no warm-tint default) + one
accent at ≤10% of the surface, iOS-blue-adjacent until branding is decided (then a
variable swap, not a rework). Reserved semantic colors, used only for what they mean:
red = allergy flags / incidents / pickup denies; green = present/checked-in; amber =
ratio warnings. Dark theme: elevated surfaces step lighter (iOS dark elevation), never
pure black cards on pure black.

## Typography

System stack (`-apple-system, system-ui, …`) — SF on Apple devices, free, fast, and the
iOS feel is largely the type. iOS-style hierarchy: large-title screen headers (~28–34px,
bold, tight-but-≥-0.02em) collapsing to a compact bar on scroll; body 16–17px, ≥4.5:1
contrast in both themes; footnote/caption for timestamps. One family, weights carry
hierarchy. `text-wrap: balance` on headings.

## Shape & Depth

Continuous-feel rounded corners, one radius scale (`--radius` ≈ 12px; cards/sheets
16–20px; controls 10–12px). Depth via soft, large-blur, low-alpha shadows + hairline
separators (0.5–1px, low-alpha ink) — never hard borders everywhere. Translucent
bar chrome (backdrop-blur on the bottom tab bar / collapsed header) is the one
sanctioned blur — purposeful iOS material, not decorative glassmorphism.

## Layout

Mobile-first 360px; 4px spacing grid with generous verticals (screen padding 16–20px).
Bottom tab bar (guardian/staff) in the thumb zone; secondary tasks open as **bottom
sheets** on phones (shadcn Drawer/Sheet), centered dialogs on desktop. Grouped
inset-list pattern (rounded card sections with hairline row separators) for settings/
profile/detail screens. Laptop (~1280px): content max-widths, real multi-column (admin
dashboard, menu-planner week grid) — never a stretched phone column.

## Components

shadcn/ui exclusively (Radix a11y intact): Tabs, Sheet/Drawer, Dialog, Form, Table
(admin), Toast, Avatar, Badge, Skeleton. Feed entries are typed rows/cards with the
photo dominant. Skeletons + retry on every loading state — blank screens are banned
(mobile-shell scope).

## Motion

iOS-feel: quick, fluid, exponential ease-out (200–300ms); spring-like sheet
presentation; live feed items enter with a subtle fade/slide; layout jumps avoided
(reserve image space). `prefers-reduced-motion` → crossfade/instant, always.

## Anti-patterns (see PRODUCT.md)

No sidebars/docks on phone surfaces, no cream body bg, no gradient text, no side-stripe
borders, no identical card grids, no eyebrow kickers, no fake native chrome.
