# Billing scope — invoicing & payments (phase 2)

Status: scope (early — phase 2; recorded now so phase-1 data decisions don't preclude it).
Owning repo: **this repo**, as its **own extension** `rust/extensions/care-billing/`
(split from `care` deliberately: external provider, own secrets, own blast radius).

Tuition invoicing and payment collection — the brightwheel-class differentiator and the
center's cash flow: recurring tuition per enrollment, one-off charges, households as the
billing unit, a payment provider (Stripe assumed) behind one trait, and the
`receives_billing` guardianship flag deciding who sees and pays what.

## Goals (phase-2 fidelity — refine before build)

- **`household`** (billing-only grouping — never data access; the master scope's family
  model holds): payer guardian(s), children covered, split-billing percentages for the
  blended-family case (Sam pays 50% of Leo, 100% of Mia).
- **`invoice`/`payment`** records: recurring generation from enrollment schedules (a
  reactor/rule tick), line items, status lifecycle, receipts; append-only ledger
  discipline like attendance.
- **Provider behind one trait** (`PaymentProvider`, one named file — the sanctioned
  external fake in tests): checkout/autopay via provider-hosted flows — **no card data
  ever touches the node** (SAQ-A posture).
- **Verbs:** `care-billing.invoice.generate/list/get`, `.payment.record` (manual/cash),
  provider webhook ingest via lb's generic webhook surface; guardian sees exactly their
  household's invoices (authz reach extended to households).
- Subsidy/agency payments recorded as payment sources (tracking, not agency integration).

## Non-goals

- Payroll, expenses, accounting exports (CSV later; never a GL).
- Agency/government billing integrations.
- Phase 1 builds **nothing** here — but must: keep `receives_billing` on edges, keep
  enrollment schedules queryable, and never merge guardians into implicit families.

## How it fits (checkpoints for the future build)

- Separate extension = separate caps (`mcp:care-billing.*`), so a compromised or buggy
  billing surface can't read feeds; secrets via lb `secrets/` mediation; webhooks via lb's
  keyed inbound-HTTP scope; money effects are must-deliver (outbox) and idempotent
  (provider event ids); cross-family matrix extends to invoices (Ana never sees Sam's
  Mia-related charges even though they share Leo's channel).

## Open questions (to resolve at phase-2 scoping)

- Stripe confirmed? Regional alternatives needed (AU: direct debit)?
- Autopay mandates + failed-payment dunning policy.
- Does the household editor live in admin UI v1 or derive from `receives_billing` +
  percentages on edges?

## Related

`../care/care-scope.md` (§family model, §phases) · `../care/enrollment-invites-scope.md`
(schedules, `receives_billing`) · lb `secrets/` · lb `ingest/webhooks-scope.md` · lb
`inbox-outbox/outbox-scope.md`.
