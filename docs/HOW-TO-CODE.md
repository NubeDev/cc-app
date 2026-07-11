# How to code — the coding-session playbook

The execution counterpart to `SCOPE-WRITTING.md`. Hand this file to an agent along with a
**scope doc**; it turns that ask into shipped, tested, documented work. Adapted from the
upstream **lb** platform (`docs/HOW-TO-CODE.md`) — read that for the fuller version.

> **Read first:** `FILE-LAYOUT.md` (one responsibility per file — before writing any code),
> `ABOUT-DOCS.md` (session doc rules + template), this repo's `CLAUDE.md` (the rules), and —
> because cc-app is one repo in a family — **`WORKFLOW-LB.md`** (how a change lands across
> `lb` / the SDKs / the in-repo `extensions/`: where it goes, local `[patch]` dev, and the
> PR→tag→bump release flow). Many tasks here are not a cc-app edit at all — they belong
> upstream. Decide that FIRST.

## 1. What you give the agent

A **scope doc** (`scope/<topic>/<name>-scope.md`). If there is no scope doc yet, stop and
run `SCOPE-WRITTING.md` first. Code without a scope is how the contracts come out wrong.

## 2. What the agent produces (the deliverables)

| Deliverable | Path | Always? |
|---|---|---|
| **Code** | within FILE-LAYOUT limits | yes |
| **Tests** | beside the source / `tests/` — incl. mandatory categories | yes |
| **Green output** | pasted into the session doc | yes |
| **Session doc** | `sessions/<topic>/<name>-session.md` | yes |
| **Debug entries** | `debugging/<area>/<symptom>.md` + `debugging/README.md` row | if something broke |
| **Public promotion** | `doc-site/content/public/<topic>/` | if something shipped |
| **Scope updates** | open questions resolved or refreshed | yes |
| **STATUS.md** | mark the slice/stage state | yes |

## 3. Procedure

0. **Place the change in its repo (cross-repo first move — see `WORKFLOW-LB.md`).** Before
   coding, decide *where this belongs*: a host verb / boot option / gateway route / capability
   is an **`lb`** change; the extension ABI or UI mount contract is an **SDK** change
   (`lb-ext-sdk` / `lb-ext-ui-sdk`); an extension is an **in-repo `rust/extensions/<id>/`** change; only
   product logic, the thin mobile shell (`ui/`), boot wiring, and packaging live **here**. Rule-10 gut
   check: if you're editing a core/SDK file *only* to make cc-app work, it belongs behind a
   generic seam (an additive `BootConfig` field, a new MCP verb, an SDK export) — never a
   special case. If the change is upstream, follow that repo's playbook, use a local `[patch]`
   to prove it from here (`WORKFLOW-LB.md` §3), then release + bump.
1. **Locate yourself.** Read `STATUS.md`: which slice, what is the exit gate? Restate it.
2. **Read the scope.** Its "How it fits", "Testing plan", and "Open questions" *are* your
   task list and acceptance criteria.
3. **Open the session doc**, status `in-progress`; keep it updated as you work.
4. **Slice vertically** — build one capability through all its layers, not one file in
   isolation. Respect FILE-LAYOUT.
4a. **Build the whole contract, not the easy half.** Ship every verb the scope named, wired
   end to end, each with its own deny-test. A half-wired surface *looks* finished, then
   doesn't work. If a verb genuinely shouldn't exist yet, it is an explicit scope non-goal
   with a reason — never a silent gap.
4b. **Building an extension UI? NEVER hand-write `remoteEntry`.** The federation entry
   (`remoteEntry.tsx`) is a single `defineRemote({ id, styles, page, widgets })` call from
   `@nube/ext-ui-sdk` — the SDK owns the scoped mount + React root + widget dispatch. Do not
   write your own `mount`/`mountWidget`, `createRoot`, `document.head` style-injection, or a
   `mount.tsx`; do not hand-roll `mountScoped`. Pair it with `defineExtConfig({ entry:
   "src/remoteEntry.tsx" })`, `extTailwindPreset()`, and a `tokens.css` with no
   `@tailwind base`/`:root{}`/`.dark{}`. A hand-rolled entry reintroduces the CSS-into-host
   leak — it's a regression. `host-metrics` / `proof-panel` are the reference; see
   `docs/extensions/README.md` §3a.
5. **Test in the same session.** Real infra, seeded data — **never mock data, never a fake
   backend** (this family inherits lb rule 9). Paste the green output.
6. **Debug in the open.** Non-trivial break → a `debugging/` entry with root cause + fix +
   a regression test that fails-before/passes-after.
7. **Promote what shipped** to `doc-site/content/public/`.
8. **Close the scope.** Resolve/refresh open questions.
9. **Move STATUS.**
10. **Run the self-check** (§4).

## 4. Definition of done

- [ ] The change is in the **right repo** (WORKFLOW-LB.md §2); any upstream part is released
      (tagged) and cc-app's pin bumped — **no committed `path`/`[patch]` to a sibling checkout**.
- [ ] Work satisfies the scope and its exit gate.
- [ ] The full API surface the scope named is built end to end (no easy-subset, no silent gap).
- [ ] Code respects FILE-LAYOUT (one verb/file, ≤400 lines, named concepts).
- [ ] If an extension UI: `remoteEntry.tsx` is a single `defineRemote(...)` call — no hand-written
      `mount`/`mountWidget`/`createRoot`/`document.head` injection, no `mount.tsx`.
- [ ] No mock data / no fake backend — tests run on real infra, seeded via the real write path.
- [ ] Tests exist, mandatory categories included, and the **green output is pasted**.
- [ ] Every bug fixed has a regression test and a closed debug entry.
- [ ] `sessions/<topic>/<name>-session.md` is filled in (not a stub).
- [ ] Anything shipped is in `doc-site/content/public/`.
- [ ] The scope doc's open questions are current; `STATUS.md` reflects the new state.

If any box is unchecked, the session is **incomplete** — say so, don't claim done.
