# Debugging — the working history

Append-only. Every non-trivial issue and how it became working, one row per
entry: `debugging/<area>/<symptom>.md`. On resolution, fill in root cause +
fix + a regression test (`docs/ABOUT-DOCS.md` §"debugging").

| Date | Area | Symptom | Status | Entry |
|---|---|---|---|---|
| 2026-07-12 | authz | Native host-callback `grants.assign`/`grants.revoke` → `Denied`: `grants.*` verbs are not on the lb `/mcp/call` dispatcher surface, so a native extension cannot derive scoped grants over the callback. | OPEN (upstream lb fix) | [grants-verbs-not-on-mcp-callback-surface.md](authz/grants-verbs-not-on-mcp-callback-surface.md) |
