# About these docs

How this repo's `docs/` is organized and which folder a given note belongs in. This
discipline is inherited from the upstream **lb** platform (`docs/ABOUT-DOCS.md` there) so
every repo in the family shares one working method — read that repo's copy for the fuller
rationale; this file is the local adaptation.

The docs track a feature through three stages — **what we want → how we built it → what
shipped**:

```
docs/
├── scope/      ← "hey, we want this"   — the ask, before any work
├── sessions/   ← the AI-agent sessions — what was done, while doing it
├── vision/     ← the north star        — the big-picture direction (optional per repo)
└── debugging/  ← the working history   — every issue and how it became working
```

Shipped, reader-facing truth is authored under `doc-site/content/public/` (MDX) — the
durable source of truth a new person reads. `scope/` and `sessions/` are working material
and are not published.

Top-level guides tie it together: `STATUS.md` (the live "where are we" dashboard),
`SCOPE-WRITTING.md` (how to write a scope), `HOW-TO-CODE.md` (how to build one), and
`FILE-LAYOUT.md` (how to lay out the code).

## The three stages

### `scope/` — what we want

The **intent**, written *before* the work: the goal, rough requirements, constraints, open
questions. One file per feature/ask, under a topic subfolder
(`scope/<topic>/<name>-scope.md`). Don't write it from scratch — follow
[`SCOPE-WRITTING.md`](SCOPE-WRITTING.md).

### `sessions/` — the AI agent session

The **working log** of a Claude Code session — what was explored, decided, and changed
*while doing the work*. One file per session under the matching topic subfolder
(`sessions/<topic>/<name>-session.md`). Captures *how* it got done and *why*, not just that
it did.

### `public/` — what shipped (lives in `doc-site/`)

The **shipped, durable** documentation, trimmed of session noise, authored under
`doc-site/content/public/` as MDX. This is what a new person reads.

## `debugging/` — the working history

Append-only. Every issue and how it became working: `debugging/<area>/<symptom>.md` +
a row in `debugging/README.md`. On resolution, fill in root cause + fix and add a
regression test.

## `vision/` — the north star

The big-picture direction for this repo. Optional — a pure mirror/consumer repo may have
none; a product host repo should.

---

## Rules for AI sessions (required)

**Documentation is part of every task, not an afterthought.** A session that changed code
or decisions but wrote no docs is **incomplete**.

### At the start of a session
1. Read this repo's `README.md`, `CLAUDE.md`, and any `scope/<topic>/` doc covering the work.
2. Read `docs/FILE-LAYOUT.md` before writing code.
3. Create the session doc `sessions/<topic>/<name>-session.md` from the template below.

### While working
- Keep the session doc updated as you go — it is the working log, not a final report.
- Record the alternative you rejected and *why*.
- If the scope was wrong, update the `scope/` doc — don't silently diverge.

### At the end of a session
- Resolve or update open questions in the relevant `scope/` doc.
- Promote durable truth to `doc-site/content/public/` if something shipped.
- Test the change (see below) and paste the green output into the session doc.
- Cross-link scope ↔ session ↔ public for the same topic.

### Testing & debugging are part of the session
- **Test it** in the same session; show the green output. "Tests later" is not allowed.
- **Log the debugging**: open a `debugging/<area>/<symptom>.md` entry as you investigate;
  on resolution add root cause + fix + a regression test and update `debugging/README.md`.

### Definition of done for a session
Work complete **and** the session doc exists and is filled in **and** the change is tested
**and** any debugging is captured with a regression test **and** any shipped change is in
`doc-site/content/public/` **and** the scope doc's open questions are current.

### Session doc template

```markdown
# <Topic> — <short title> (session)

- Date: YYYY-MM-DD
- Scope: ../../scope/<topic>/<name>-scope.md
- Status: in-progress | done | blocked

## Goal
What this session set out to do.

## What changed
The concrete edits/decisions (link files as path:line where useful).

## Decisions & alternatives
- Chose X over Y because …

## Tests
What was tested and the **green command output pasted here**.

## Debugging
Links to any `debugging/<area>/<symptom>.md` entries, each with its regression test.

## Public / scope updates
What was promoted to `doc-site/content/public/`, which scope open questions resolved.

## Follow-ups
- Open questions pushed back to the scope doc; STATUS.md updated?
```
