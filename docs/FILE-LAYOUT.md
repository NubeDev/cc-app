# FILE LAYOUT — one responsibility per file

Inherited verbatim in spirit from the upstream **lb** platform (see `docs/FILE-LAYOUT.md`
there for the long-form reference with worked examples). This is the local, condensed copy;
it applies to `.rs`, `.ts`, and `.tsx` alike, in this repo, without exception.

Small, well-named files are not a style preference. They are how an AI (or a human reading
cold) finds the right code without burning context on irrelevant lines.

## 1. The hard limits

| Limit | Value | Hard? |
|---|---|---|
| Lines per file | **400** | Hard. PR blocked above this. |
| Lines per file (warning) | 300 | Soft. Plan the split. |
| Lines per function | 50 | Soft. Extract a sub-function. |
| Nesting depth | 4 | Soft. Early return, extract. |

400 lines is the **ceiling**, not the target. Most files should sit between 30 and 150 lines.

## 2. The verb-per-file pattern

Group code by **the verb the caller performs**, not the noun it operates on. One file = one
verb (or one phase of one verb). `mod.rs` / `index.ts` are **barrels** — re-exports only,
never bodies.

```
user/
  mod.rs      ← re-exports + module doc, ≤30 lines
  get.rs      ← one verb
  create.rs   ← one verb: validate, persist, emit
  delete.rs   ← one verb: cascade rules
```

A new engineer reading `user/` learns the API by reading the filenames before opening a file.

## 3. When the verb itself is too big

Split by **phase of the verb**, not by reusable helper noun (`validate.rs`, `persist.rs`,
`emit.rs`). Never `helpers.rs` / `utils.rs` / `internal.rs`.

## 4. Frontend (React / TypeScript)

One component per `.tsx`, `PascalCase` matching the component. One hook per file
(`use<Concept>.ts`). Separate data from markup. `index.ts` is a barrel. One store per file
(`session.store.ts`, never `store.ts`). One API call per export, grouped by resource.

## 5. File-naming rules

| Never | Always |
|---|---|
| `utils.rs` / `utils.ts` | Name the concept: `retry.rs`, `token_cache.rs` |
| `helpers.rs` / `helpers.ts` | Name the concept |
| `common.rs` / `misc.rs` / `support.rs` | Don't create. Trash drawers grow forever. |
| `mod.rs` with logic in it | `mod.rs` is a barrel. Body lives elsewhere. |
| `types.rs` / `models.rs` | Name by what they model: `address.rs`, `principal.rs` |

If you cannot describe the file's job in one sentence without "and" — it's two files.

## 6. The split heuristic

1. **One-sentence test.** Job describable in one sentence, no "and"?
2. **Filename test.** Would someone searching by filename find what they expect?
3. **Edit-locality test.** Do two PRs touching this file touch different concerns? → split.

If you're about to write more than **~150 lines** in a new file, pause and split first.

## 7. When NOT to split

Don't fragment for its own sake. A `Display` impl belongs with its type; a small struct +
its `Default` + `new()` belong together; a handler and its single private helper may live
together until a second caller appears. Split when there are **two distinct caller-visible
responsibilities**.

## 8. Enforcement

- Generated code (OpenAPI/protobuf) is exempt; put it under `src/generated/`.
- CI runs a file-size check (`scripts/check-file-size.sh`) failing any tracked human
  `*.rs`/`*.ts`/`*.tsx` over 400 lines.

## 9. One-line summary

**One verb per file. Folder-of-verbs over file-of-nouns. ≤400 lines hard, ~100 typical.
Names are concepts, never shapes (`utils`, `helpers`, `common`).**
