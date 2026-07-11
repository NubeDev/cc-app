# Milestone 10 — hardening & phase-1 launch

Everything is built; this milestone proves it holds together, on a phone, under the rules,
and promotes the docs. Nothing new ships here except fixes.

## Entry gate

- [ ] Milestones 00–09 all closed (every exit-gate checkbox ticked in this directory).

## Work items

- [ ] **Full matrix sweep:** every registered `care.*` verb has its cross-family matrix
      row (the 02 harness enumerates and fails on gaps); the three mandatory suites green
      in one run — capability-deny per persona + kiosk, workspace isolation (two orgs),
      cross-family (Sam/Ana/Mia's-mum over every verb, SSE, channels, media URLs).
- [ ] **Persona acceptance pass:** walk all 15 use-case docs in
      [`../scope/personas/`](../scope/personas/README.md) on a real node, on a real phone
      (360px), as the real persona — **and again on a laptop viewport (~1280px), and in
      both dark and light themes** (CLAUDE.md rule 9). Each doc's deny/edge cases
      observed. This is the product sign-off checklist.
- [ ] **The full Spanish pass (the day-one MUST,
      [`../scope/ui/i18n-scope.md`](../scope/ui/i18n-scope.md)):** the same 15-use-case
      walk performed entirely as `es` users — every screen, the invite email, the
      incident push, all deny messages, dates/numbers. Catalog CI gate green with zero
      missing keys; hardcoded-string lint clean; the `es` catalog reviewed by a human
      Spanish speaker (resolve the scope's reviewer open question — launch-blocking).
- [ ] **Edge-change drill:** one scripted E2E — unlink Ana↔Leo mid-session → feed SSE
      terminates, channel access gone, media 403s, push stops, grants revoked. The
      existential-bug drill, end to end, one test.
- [ ] Performance pass on a mid-range phone: photo-heavy feed (thumbs, virtualization),
      cold PWA start, flaky-network behavior (skeleton + retry, no blank screens).
- [ ] Ops pass: clean boot from fresh checkout per README; `.cc-app/` state documented;
      backup/restore of the store exercised once; CI green (fmt, clippy, tests,
      file-size ≤400).
- [ ] Release hygiene: all pins on tags (no `[patch]` anywhere committed); versions
      recorded in STATUS.md.
- [ ] **Promote docs:** shipped scopes → `doc-site/content/public/care/` + `public/ui/`
      per each scope's "Promotes to" header; session docs complete; STATUS.md flipped to
      "phase-1 shipped".
- [ ] Sweep every scope's remaining open questions: resolved, or explicitly carried as
      phase-2 items in STATUS.md (master-scope items: naming/branding, PWA-vs-RN, offline
      posture for staff tablets, billing provider — carry, don't block).

## Exit gate

- [ ] A real center could onboard tomorrow: admin sets up, enrolls, invites; teachers run
      a room day; guardians live on the feed — demonstrated end-to-end on tagged releases
      with the full test suite green (output pasted in the session doc).

## What comes after (not this milestone)

Phase 2 begins with **billing, last as decided**
([`../scope/billing/billing-scope.md`](../scope/billing/billing-scope.md)) — re-scope it
properly first (its own open questions), as its own extension `care-billing`. Also queued:
kiosk PIN self-check-in (1.5), video (needs lb media Range), admissions forms, reports.

## Sources

Every scope in `../scope/` · `../HOW-TO-CODE.md` §4 · CLAUDE.md.
