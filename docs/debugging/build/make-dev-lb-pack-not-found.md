# `make dev` fails: `cargo build -p lb-pack` — no such package

- **Area:** build / extension packaging (dev flow)
- **Date:** 2026-07-12 (found)
- **Date:** 2026-07-12 — **CLOSED.** `node-v0.3.3` shipped `lb-devkit` +
  `lb-pack` as PUBLISHED crates (`publish = false` dropped; the artifact
  packager + the Ed25519 sign / key / trust-line idiom are now
  git-tag-consumable). The cc-app follow-on lives in the `pack` target
  below — `cargo install --git https://github.com/NubeDev/lb --tag
  node-v0.3.3 lb-pack` is the new mechanism (Makefile §"Extension dev
  flow"). The `cargo build -p lb-pack` workaround in `Makefile` is
  gone; `make pack` now installs the tool at the pinned lb tag and
  produces the signed `Artifact` JSON.

> **Status:** CLOSED 2026-07-12.

## Symptom (history)

```
$ make dev
cd rust && cargo build -p lb-pack
error: package ID specification `lb-pack` did not match any packages
help: a package with a similar name exists: `lb-caps`
make: *** [Makefile:158: rust/target/debug/lb-pack] Error 101
```

The `Makefile` `dev`/`pack`/`publish` targets depended on
`$(BE_DIR)/target/debug/lb-pack`, built via `cd rust && cargo build -p
lb-pack` (Makefile §"Extension dev flow"). The cc-app workspace has only
two local packages — `cc-node` and `care` — so there was no `lb-pack`
to build.

## Root cause (found by reading lb, read-only)

`lb-pack` is the dev **artifact packager**: it turns a built `*.wasm` +
its `extension.toml` into the SIGNED `Artifact` JSON the gateway's
`POST /extensions` requires. It existed ONLY inside the lb repo, at
`rust/tools/pack/` (crate name `lb-pack`), and it — together with the
one lb library it needs, `lb-devkit` (the Ed25519 sign / key / trust-
line idiom) — was `publish = false`. So the packager was not consumable
by any embedder: cc-app could not build, install, or depend on it.

The Makefile was written anticipating a local `lb-pack` crate (`cargo
build -p lb-pack`), which only ever worked inside lb's own workspace.

## The fix (shipped as `node-v0.3.3`)

Publish the pack toolchain so an embedder consumes it by git tag / `cargo
install`, the same way cc-app already embeds the core via `lb-node`.
Scoped in lb at `docs/scope/extensions/pack-toolchain-publish-scope.md`
— drop `publish = false` on `lb-devkit` and `lb-pack`, make `lb-pack`
installable (`cargo install --git …lb --tag <tag> lb-pack`),
document it in the dev flow. Purely a packaging/exposure change: the
`Artifact` shape, Ed25519 signature, `verify_artifact`, `ext.publish`
cap, and `LB_TRUSTED_PUBKEYS` are all unchanged.

Rejected (rule 10): vendoring `rust/tools/pack/` into cc-app. It forks
the sign/verify idiom — the moment `lb-devkit`'s digest/trust-line
format evolves, a vendored copy silently produces artifacts the node
rejects.

## cc-app follow-on (now in the Makefile)

Edit the `Makefile`: replace the `$(BE_DIR)/target/debug/lb-pack:`
build target (`cargo build -p lb-pack`) with installing/pinning the
PUBLISHED `lb-pack` at the same lb tag `lb-node` uses:

```make
LB_TAG ?= node-v0.3.3

# Install lb-pack at the pinned lb tag. Idempotent; cheap; $(LB_TAG) makes
# the install a one-line re-target when the lb pin bumps.
LB_PACK_BIN := $(HOME)/.cargo/bin/lb-pack
$(LB_PACK_BIN):
	@command -v lb-pack >/dev/null 2>&1 || \
	  cargo install --git https://github.com/NubeDev/lb --tag $(LB_TAG) lb-pack --locked
```

The `trust` + `pack` + `publish` targets now depend on `$(LB_PACK_BIN)`
(instead of the now-deleted `$(BE_DIR)/target/debug/lb-pack`):

```make
trusted-pubkey: $(LB_PACK_BIN)
	@$(LB_PACK_BIN) pubkey $(KEY_FILE) --key-id $(PUBLISHER_ID)

pack: $(LB_PACK_BIN)
	$(LB_PACK_BIN) $(EXT_MANIFEST) $(KEY_FILE) --key-id $(PUBLISHER_ID) \
	    --out .cc-app/extensions/care-$(BUILD_TAG).artifact.json
```

`make pack` → produces the signed `Artifact` JSON (you don't need a
running node to verify the gate; the tool's exit code on a deterministic
fixture is the check). The rest of the build (`cargo build` / `cargo
test` of `cc-node` + `care`, the UI) is unaffected.

## Patch (upstream lb — shipped, archived for the record)

The single-line `publish = false` drop on `rust/tools/pack/Cargo.toml`
+ the matching line on `rust/libs/devkit/Cargo.toml` is the whole fix
on the lb side; the docs / skill / session-doc that go with the change
live in lb (`docs/scope/extensions/pack-toolchain-publish-scope.md`).
Tagged `node-v0.3.3` (commit `a02c353`).
