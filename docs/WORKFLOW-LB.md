# Working across lb, the SDKs, and cc-app

You are standing in **`cc-app`** (a **product host**). It doesn't contain the core — it
**embeds** it. When a task needs a change in the core or an SDK, that change lands in a
**different repo**, gets **released** (a git tag), and then cc-app **bumps a pin** to pick
it up. This is the same family workflow as `NubeIO/rubix-ai` — read that repo's
`docs/WORKFLOW-LB.md` for the long-form version with the full release ritual; this file is
the cc-app-specific map.

> **Read `lb`'s `MIGRATION.md` too.** It is the authoritative "what moved out of lb and
> where it lives now" map.

## 1. The repos and who owns what

| Repo | Owns (authoritative) | cc-app consumes it as |
|---|---|---|
| **`NubeDev/lb`** | The **core platform** (identity, caps wall, SurrealDB store, Zenoh bus, ext runtime, channels, gateway), embedded via the `lb-node` seam (`BootConfig` + `boot_full`/`RunningNode`). | Rust git-dep `lb-node = { git, tag = "node-vX.Y.Z" }` |
| **`NubeDev/lb-ext-sdk`** | The **Rust extension contract**: `lb-sdk` (WIT world) + `lb-ext-native` (native child wire). | by the in-repo extensions, git-tag `sdk-vX.Y.Z` |
| **`NubeDev/lb-ext-ui-sdk`** | The **UI extension contract**: `@nube/ext-ui-sdk` — `defineRemote`, mount contracts, Vite + Tailwind presets. | by the ext UIs + the thin shell, git-tag `ui-vX.Y.Z` |
| **`cc-app`** (this repo) | The product: host binary (`rust/node/`), the **thin mobile-first shell** (`ui/`), and the **product extensions** (`rust/extensions/<id>/` — all domain logic + 100% of the product UI). | — |

**Difference from rubix-ai:** the extensions live **in-repo** under `rust/extensions/`, not in a
sibling repo. The product *is* its extensions (the host is a boot shim), so splitting them
out would separate the product from itself. They still build **against the published SDKs
only** — an extension importing anything from an lb checkout is a violation. Rejected
alternative: a sibling `cc-app-extensions` repo (rubix-ai's shape) — revisit only if a
third party ever needs to ship extensions without product-repo access.

**Dependency arrows point one way:** `cc-app → lb`, `extensions → SDKs`, `lb → SDKs`.
Nothing upstream may name or special-case this product (lb rule 10).

## 2. "Where does this change go?" — decide first

| The change is… | It belongs in… |
|---|---|
| A new/changed **host verb, boot option, gateway route, reactor, capability grammar** | **`lb`** → then bump the `lb-node` tag here |
| The **extension ABI** (WIT world, native wire) | **`lb-ext-sdk`** |
| The **UI mount contract** (`defineRemote`, presets, CSS isolation) | **`lb-ext-ui-sdk`** |
| Childcare **domain logic, tools, pages, widgets** | in-repo **`rust/extensions/<id>/`** — no core change |
| Boot wiring, config, packaging, the thin shell | **this repo** (`rust/node/`, `ui/`, `docker/`) |

**The rule-10 gut check:** if you're editing a core/SDK file *only* to make cc-app work,
stop — the fix belongs behind a generic seam (an additive `BootConfig` field, a new generic
MCP verb, an additive SDK export), never a branch on a name. The lb gaps this product has
already identified (entity-scoped grants, invites, push, media — see
`scope/care/care-scope.md` §"lb gaps") are exactly such seams: they are **generic lb
features** that happen to be motivated here.

## 3. Local development across repos

Sibling checkouts under `~/code/rust/`. Override the lb git-dep with a local path in the
**git-ignored** `.cargo/config.toml` (which is also where the zigcc linker wiring lives —
no system `cc` on this box):

```toml
# cc-app/.cargo/config.toml (git-ignored; machine-local)
[patch."https://github.com/NubeDev/lb"]
lb-node = { path = "../lb/rust/node" }
```

Same pattern for `lb-ext-sdk` inside an extension's `Cargo.toml` patch, and `pnpm add
../lb-ext-ui-sdk` for UI SDK work (remember its `dist/` is committed — rebuild it).
**Never commit a `path`/`[patch]` pointing at a sibling checkout.**

On-disk state (store, installed ext bundles) is repo-anchored under **`.cc-app/`**
(threaded to lb as `BootConfig` fields, same posture as rubix-ai's `.rubix-ai/`). Log in as
a real seeded user, never `dev`; re-login after publishing an extension to pick up caps.

## 4. Releasing (PR → tag → bump)

Bottom-up, each consumer builds against a released producer: **SDK tag → lb PR+tag →
cc-app bumps `lb-node`** → in-repo extensions bump SDK tags and republish to the running
node. Prove the whole feature top-down locally with patches *first*, then release, then drop
the patches (a clean `cargo build` with no local paths is the "am I on releases?" check).

## 5. Gotchas inherited from the family

- zigcc linker via git-ignored `.cargo/config.toml` — a fresh checkout won't link until it exists.
- `lb-ext-ui-sdk` ships a committed `dist/` — edit source → `pnpm build` → commit dist.
- Native ext re-publish hits `Text file busy` — `DELETE /extensions/<id>` first (until lb fixes it).
- Re-login after publish/install to pick up new caps.
- No Rust hot-reload — restart the node after any Rust change.
- Ext UI bundles are served **flat** (no `ui/` subdir) for native-tier artifacts.
