# `rust/extensions/care/`

The **single backend extension** owning the whole childcare domain and **100% of
the product UI**. Tools namespaced `care.*`; all guardian scoping flows through
the one `authz/` module (CLAUDE.md rule 7 — sacred).

## Layout

```
care/
├── src/                  Rust crate: tools, storage, outbox targets
│   ├── authz/            THE chokepoint — every read/write verb passes through it
│   ├── center/ room/     care.center.*, care.room.*          (admin)
│   ├── child/            care.child.*          (admin writes; staff/guardian read)
│   ├── guardian/ guardianship/
│   │                     care.guardian.* + care.guardianship.link/unlink/update
│   ├── invite/           care.invite.*         (admin creates; pre-auth accept)
│   ├── enrollment/       care.enrollment.*     (waitlist + CSV import job)
│   ├── attendance/       care.attendance.*     (staff/kiosk)
│   ├── log/              care.log.*            (staff write; guardian read)
│   ├── feed/             care.feed.watch       (SSE live feed)
│   └── menu/             care.menu.*           (plans + per-child substitutions)
└── ui/                   Extension UI: pages + widgets via defineRemote (SDK-only)
```

Folder-of-verbs layout mirrors the `care.*` MCP surface in
[`../../../docs/scope/care/care-scope.md`](../../../docs/scope/care/care-scope.md) §Data / §MCP.

## Owners

- Skeleton + authz chokepoint: [`02-care-skeleton-authz.md`](../../../docs/build/02-care-skeleton-authz.md).
- Per-feature verbs: milestones 03 → 09 → [`10-hardening-launch.md`](../../../docs/build/10-hardening-launch.md).

## Rules

- [`../../../docs/FILE-LAYOUT.md`](../../../docs/FILE-LAYOUT.md), [`../../../docs/HOW-TO-CODE.md`](../../../docs/HOW-TO-CODE.md).
- CLAUDE.md rules 7 (guardian isolation) and 8 (en+es from day one) are enforced
  per milestone exit gate.
- UI: never hand-write `remoteEntry` (CLAUDE.md rule 6) — single `defineRemote(...)`
  from `@nube/ext-ui-sdk`. See [`./ui/README.md`](./ui/README.md).