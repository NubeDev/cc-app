# Scope writing — the setup playbook

Hand this file to an agent with a feature idea; it turns the idea into a complete scope
setup: the scope doc, the session/public stubs, the testing plan, and the index updates.
Adapted from the upstream **lb** platform (`docs/SCOPE-WRITTING.md`).

> **Read first:** `ABOUT-DOCS.md` (three-stage flow + session template),
> `FILE-LAYOUT.md` (one responsibility per file), and **`WORKFLOW-LB.md`** — a scope written
> here often describes a change that actually lands **upstream** (in `lb` or an SDK). Name the
> owning repo in the scope so the build lands it in the right place.

## 1. What you give the agent

A feature idea, at any fidelity. If too vague to scope, the agent asks 2–3 sharp questions
first, then proceeds.

## 2. What the agent produces

For topic `<topic>` (kebab-case) and ask `<name>`:

| Artifact | Path | State after setup |
|---|---|---|
| **Scope doc** | `scope/<topic>/<name>-scope.md` | Fully written (the deliverable). |
| **Session doc** | `sessions/<topic>/<name>-session.md` | Created when work starts. |
| **Public stub** | `doc-site/content/public/<topic>/<topic>.md` | TODO placeholder, filled on ship. |
| **Debug area** | `debugging/<topic>/` | Created lazily on first bug. |

## 3. Naming — one topic, used everywhere

Pick one kebab-case `<topic>` and reuse it across every folder so a feature reads
top-to-bottom: `scope/<topic>/` → `sessions/<topic>/` → `doc-site/content/public/<topic>/`.

## 4. Procedure

1. **Clarify if needed** (2–3 questions), else proceed and state assumptions.
1a. **Name the owning repo** (`WORKFLOW-LB.md` §2). Most of the platform surface — host verbs,
   boot options, the extension ABI, the UI mount contract — is owned **upstream** (`lb`,
   `lb-ext-sdk`, `lb-ext-ui-sdk`), and extensions live in the in-repo `extensions/`. State in the
   scope which repo builds it and, if upstream, which released tag cc-app then bumps to.
2. **Choose the topic** — reuse before coining.
3. **Read the neighbours** — existing `scope/<topic>/` and the README sections it touches.
4. **Write the scope doc** from the template in §5.
5. **Create the public stub** if missing.
6. **Wire the indexes and cross-references.**
7. **Run the self-check** (§7).

## 5. Scope doc template

```markdown
# <Topic> scope — <short title>

Status: scope (the ask). Promotes to `doc-site/content/public/<topic>/` once shipped.

One-paragraph statement of what we want and why. No implementation detail — the brief.

## Goals
## Non-goals
## Intent / approach
The shape at architecture altitude; the alternative rejected and why.

## How it fits
Address each that applies: isolation/tenancy, capabilities & the deny path, placement,
the API/MCP surface (CRUD / get-list / live-feed / batch), data, motion, secrets. For repos
in the lb family, keep rule 10 (core knows no extension) and rule 9 (no mocks) in view.

## Example flow
A concrete numbered walkthrough of the main path.

## Testing plan
Which mandatory categories apply (capability-deny, workspace-isolation, offline/sync,
hot-reload), plus the key unit/integration/E2E cases. Real infra, seeded data.

## Risks & hard problems
## Open questions
## Related
Links back to the upstream lb scope doc(s) this derives from, sibling scopes, README §s.
```

## 6. Platform checklist (work through while scoping)

Workspace is the hard wall · capability-first (name the deny) · symmetric nodes (no
`if cloud`) · one datastore · **no mocks / no fake backend** · state vs motion · stateless
extensions · MCP is the contract · the right API shape · durability via the outbox · one
responsibility per file · SDK/WIT impact flagged loudly · **an extension UI's `remoteEntry`
is generated (`defineRemote`), never hand-written** — if a scope touches ext UI, say so and
keep the mount/CSS contract in the SDK, not per-ext.

## 7. Definition of done

- [ ] Scope doc exists and is fully written (not a stub).
- [ ] The **owning repo** is named (cc-app vs upstream lb/SDK vs in-repo `rust/extensions/<id>/`) — WORKFLOW-LB.md §2.
- [ ] The platform checklist is addressed or explicitly N/A.
- [ ] A testing plan names the mandatory categories that apply.
- [ ] Open questions are specific and answerable.
- [ ] The public stub exists; indexes/cross-references updated.
- [ ] Voice is practical and decisive — recommends rather than surveys.
