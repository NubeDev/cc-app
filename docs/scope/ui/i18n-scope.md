# i18n scope — English + Spanish from day one (a phase-1 MUST)

Status: scope (the ask). **Binding for phase 1: every user-facing surface ships in English
and Spanish (`en`, `es`) from the first release. Not deferred, not partial.** A screen,
email, or push that cannot render in both languages fails its milestone's exit gate.

Why a must: guardians are the product's largest persona and many are Spanish-speaking;
brightwheel-class competitors treat Spanish as table stakes. A guardian who can't read an
**incident notification** is a safety failure, not a cosmetic one.

## The rule

- **Two locales at launch:** `en` (default/fallback) and `es` — full catalogs, both 100%.
  The mechanism must make locale N+1 a catalog drop, no code change.
- **lb's multi-lang seam is the mechanism** (lb has multi-lang — consume it, rule 10:
  generic seams only). **Day-one verification task:** confirm lb's i18n covers all four
  surfaces below; any hole is an **lb gap** — fix upstream generically (locale on the
  member/session record, localized outbox templates, SDK string catalogs), never a
  care-side translation hack.
- **No hardcoded user-facing strings anywhere** — extension UI, shell, emails, push
  bodies, verb error messages shown to users. CI-gated (see §Enforcement).

## The four surfaces (all of them, both languages)

1. **Extension UI + shell** (`rust/extensions/care/ui/`, `ui/`): every label, empty
   state, error, deny message. Locale = the user's preference (member record), falling
   back to device/browser language, falling back to `en`.
2. **Invite emails** (lb outbox EmailTarget): the guardian's **first touchpoint** —
   admin picks the invitee's language at `care.invite_guardian` (a `locale` field on the
   guardian record, default from the workspace); the email template renders in it.
   Pre-auth accept page localizes the same way (token carries/looks up the locale).
3. **Push notifications** (lb outbox PushTarget): title/body rendered server-side in the
   recipient's locale at send time — **incident pushes especially**. Deep-linked UI then
   renders in the same locale.
4. **Domain-generated text:** daily-log type labels, menu slot names, allergen names,
   attendance deny reasons, incident templates — anything a verb emits that a guardian
   reads. Enum keys in the store (never translated strings in records); rendered per
   locale at the edge (UI or template).

**Never machine-translate at runtime; never store translated text in domain records.**
Records hold keys/enums; catalogs hold the words.

## Data & defaults

- `locale` on the member/user preference (lb seam) **and** on the `guardian` record
  pre-account (so invites are localized before the person exists as a member; accept
  copies it to the member preference).
- Workspace default locale (a center in El Paso defaults `es`); per-user overrides.
- Admin/staff surfaces: same mechanism, both languages (staff are bilingual users too).
- Dates, times, numbers: locale-formatted at render (the feed's timestamps, menu dates).
- Free-text *user content* (messages, log notes) is never translated — only chrome,
  templates, and enum renderings are.

## Enforcement (what makes "100%" true)

- **CI gate:** catalog completeness check — every key present in both `en` and `es`;
  a key added in one catalog without the other fails the build. Hardcoded-string lint on
  `ui/` + ext UI (no raw user-facing literals in JSX).
- **Per-milestone exit gates** (wired into `docs/build/`): every screen/E2E of that
  milestone also passes in `es` — at minimum the milestone's Playwright run executes once
  with an `es`-locale user and asserts localized chrome (spot keys, not full snapshots).
- **Milestone 10:** the full persona acceptance walk (all 15 use-case docs) is performed
  **in Spanish** as well as English; invite email + incident push fixtures asserted in
  both.
- Translation quality: `es` catalog reviewed by a human Spanish speaker before launch
  (open question: who — track as a launch task, not a code gate).

## Non-goals

- Locale N+3 (fr, zh, …) — the mechanism supports it; catalogs are post-phase-1.
- Translating user-authored content (messages, notes) — never.
- RTL layout work (neither launch language needs it; don't preclude it).

## Open questions

- What exactly does lb's multi-lang cover today (UI catalogs? outbox templates? member
  locale field?) — **the day-one verification; answer decides the lb asks.** Owner:
  build milestone 01/02 session.
- `es` variant: neutral Latin-American Spanish v1 (recommended) vs region variants.
- Who reviews the `es` catalog before launch?

## Related

`mobile-shell-scope.md` · every care sub-scope (their UI/emails/pushes inherit this) ·
lb multi-lang (verify + name the exact lb docs when found) · lb invites/push-target scopes
(template rendering) · `../../build/README.md` (gate wiring).
