# `rust/extensions/`

In-repo product extensions, built against the published SDKs (`lb-ext-sdk` /
`@nube/ext-ui-sdk`) only — no special-casing (CLAUDE.md rule 10).

## Layout

- `care/` — the single backend extension owning the whole childcare domain and
  100% of the product UI. See [`./care/README.md`](./care/README.md).

## Owner

Filled by build milestones starting at
[`../../docs/build/02-care-skeleton-authz.md`](../../docs/build/02-care-skeleton-authz.md).

## Rules

- One extension per capability area. `care` is one because splitting multiplies the
  authz module (each ext re-deriving guardian→child edges = N chances to leak,
  per `care-scope.md` §Intent).
- Build/publish loop reference: `NubeIO/rubix-ai` `docs/extensions/README.md`.