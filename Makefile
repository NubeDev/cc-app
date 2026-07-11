# cc-app — the `cc-node` binary (Rust/cargo) + the care extension + the mobile shell (Vite/pnpm)
# developer tasks.
#
# Mirrors the lb Makefile's shape and conventions so the family shares one working
# method. Differences vs lb, on purpose:
#   - NO wasm guests (the care extension is an in-process Rust crate, not a wasi
#     component — see CLAUDE.md §What this is).
#   - NO federation / CE / devkit sidecars (out of scope for cc-app's phase 1).
#   - NO Docker / Fly deploys yet (no deploy milestone in the build runbook).
#   - On-disk state lives under `.cc-app/` (CLAUDE.md §Build / test commands).
#
# New here? The browser-against-the-cloud-node path is the full live-node demo:
#   make setup       one-time: pnpm install in ui/ and rust/extensions/care/ui/
#   make dev         the cloud node (gateway) + the mobile shell together
#   make kill        free the dev ports if a previous run was left running
#
# Targets (the full set — comment block at the bottom of each mirrors this order):
#   make setup       pnpm install in ui/ and rust/extensions/care/ui/
#   make build       build the host workspace + the care ext + the mobile shell + the care UI
#   make build-be    build the host workspace (the `cc-node` binary + the `care` extension crate)
#   make build-ui    type-check + production-build the mobile shell (`ui/`)
#   make build-ext   build the care extension UI bundle (pnpm build under extensions/care/ui)
#
#   make dev         cloud node + mobile shell together (Ctrl-C stops both) — the demo loop
#   make cloud       run JUST the node with the SSE gateway (CC_GATEWAY_ADDR) — cloud
#   make ui          run JUST the mobile shell dev server (browser build, points at the gateway)
#
#   make pack        build + sign the care extension into .cc-app/extensions (lb-pack)
#   make publish-ext pack + upload it to a RUNNING node (make cloud first) → installed + loaded live
#   make trusted-pubkey  print the dev publisher's CC_TRUSTED_PUBKEYS value (key auto-generated)
#
#   make test        cargo test (host) + vitest (mobile shell + care ext UI)
#   make test-be     cargo test --workspace
#   make test-ui     pnpm test (vitest, runs in both ui/ and extensions/care/ui/)
#   make lint        cargo clippy + UI type-check (both UI trees)
#   make fmt         cargo fmt + UI formatters where wired
#   make fmt-check   cargo fmt --check (what CI runs)
#   make size        the FILE-LAYOUT ≤400-line check (the S0 CI gate)
#   make i18n-check  en ⇔ es key parity gate (CLAUDE.md rule 8)
#   make clean       remove build artifacts (cargo target + ui/dist + ext UI dist)
#   make kill        free the dev ports + reap any orphaned node/cargo/vite
#
# See CLAUDE.md for the architecture and docs/build/README.md for the staged plan.

BE_DIR  := rust
UI_DIR  := ui
EXT_DIR := rust/extensions/care/ui
# `cargo run -p cc-node` runs that package's one binary. The compiled binary is still `cc-node`.
NODE_BIN := cc-node

# Dev ports — kept in sync with the code. The node mounts the SSE/HTTP gateway on GW_ADDR
# when CC_GATEWAY_ADDR is set; the mobile shell's browser build points VITE_GATEWAY_URL at it.
# Vite dev server listens on UI_PORT (strictPort in vite.config.ts — change it there too if
# you override this). We only track the ports `make kill` must free.
GW_HOST ?= 127.0.0.1
GW_PORT ?= 8080
GW_ADDR := $(GW_HOST):$(GW_PORT)
GW_URL  := http://$(GW_HOST):$(GW_PORT)
UI_PORT ?= 5173

# The workspace the node serves. One workspace is enough for the demo (= tenant).
WS ?= acme

# The dev identity the node seeds as a `workspace-admin` member of $(WS) at boot. The login
# gate requires membership, so the node boot-straps this identity into the workspace
# (provisioning + joining — NOT a login bypass). Override with the handle you log in as;
# clear it (CC_SEED_USER=) to skip seeding entirely.
SEED_USER ?= user:ada

# All persistent local dev state lives under ONE root: `.cc-app/`. The node store, the dev
# publisher key, and packaged artifacts are subdirs of it, so a single `rm -rf .cc-app`
# resets a dev box and one `.gitignore` line covers everything.
#   .cc-app/data/dev-store      the SurrealKV node store (CC_STORE_PATH)
#   .cc-app/keys/dev-publisher  the dev publisher Ed25519 seed (lb-pack reads/creates it)
#   .cc-app/extensions          packaged signed artifacts (lb-pack --out)
# Override the root with: make dev CC_DIR=/path/to/state
CC_DIR     ?= $(CURDIR)/.cc-app
STORE_DIR  ?= $(CC_DIR)/data
STORE_PATH ?= $(STORE_DIR)/dev-store
KEY_DIR    ?= $(CC_DIR)/keys
KEY_FILE   ?= $(KEY_DIR)/dev-publisher.key
ART_DIR    ?= $(CC_DIR)/extensions

# The dev publisher key id paired with KEY_FILE (must match what lb-pack stamps into the artifact).
PUBLISHER_ID ?= dev-publisher

# The extension the dev pack/publish targets operate on. Today there is only `care`.
EXT          ?= care
EXT_MANIFEST := $(BE_DIR)/extensions/$(EXT)/Cargo.toml
EXT_ARTIFACT := $(ART_DIR)/$(EXT).artifact.json
# The extension's built federated UI bundle and the dir the running node serves it from. The
# signed artifact carries ONLY the wasm + manifest; the node serves the page from CC_EXT_UI_DIR.
EXT_UI_DIST  := $(BE_DIR)/extensions/$(EXT)/ui/dist
EXT_UI_SERVE := $(BE_DIR)/extensions-ui/$(EXT)

.PHONY: setup build build-be build-ui build-ext \
        dev cloud ui \
        pack publish-ext trusted-pubkey \
        test test-be test-ui lint fmt fmt-check size i18n-check clean kill purge-store

# One-time setup: install the UI deps in both the mobile shell and the care ext UI.
# Idempotent — re-run after pulling deps changes.
setup:
	cd $(UI_DIR) && pnpm install
	cd $(EXT_DIR) && pnpm install
	$(MAKE) trusted-pubkey
	@echo "setup done — now: make dev"

build: build-be build-ui

# Build the host workspace (the `cc-node` binary + the `care` extension crate). Pure cargo.
build-be:
	cd $(BE_DIR) && cargo build --workspace

# Build the care extension UI bundle (pnpm build under extensions/care/ui). The mobile shell's
# federation runtime needs this bundle to actually load the extension — see `ui-preview` in lb.
build-ext:
	cd $(EXT_DIR) && pnpm build

# Type-check + production-build the mobile shell (ui/). Includes `tsc --noEmit` (the lint path).
build-ui:
	cd $(UI_DIR) && pnpm build

# The demo loop: the cloud node (gateway mounted) + the mobile shell pointed at it, in ONE
# foreground process group so Ctrl-C (or `make kill`) stops both. The trap reaps the children
# on exit so no orphan keeps a port held.
dev: trusted-pubkey
	@mkdir -p $(STORE_DIR)
	@echo "node gateway → $(GW_URL)   ui → http://127.0.0.1:$(UI_PORT)   (ws=$(WS), store=$(STORE_PATH))"
	@trap 'kill 0' EXIT INT TERM; \
	TRUSTED=$$($(BE_DIR)/target/debug/lb-pack pubkey $(KEY_FILE) --key-id $(PUBLISHER_ID)); \
	( cd $(BE_DIR) && CC_GATEWAY_ADDR=$(GW_ADDR) CC_GATEWAY_URL=$(GW_URL) CC_WORKSPACE=$(WS) CC_STORE_PATH=$(STORE_PATH) CC_SEED_USER=$(SEED_USER) CC_TRUSTED_PUBKEYS=$$TRUSTED cargo run -p $(NODE_BIN) ) & \
	( cd $(UI_DIR) && VITE_GATEWAY_URL=$(GW_URL) pnpm run dev ) & \
	wait

# CLOUD posture: the SAME binary with the SSE/HTTP gateway mounted (CC_GATEWAY_ADDR). A browser
# can now reach it. Run `make ui` (or `make dev`) against this.
cloud: trusted-pubkey
	@mkdir -p $(STORE_DIR)
	@echo "cloud: node + gateway → $(GW_URL)   (ws=$(WS), store=$(STORE_PATH))"
	TRUSTED=$$($(BE_DIR)/target/debug/lb-pack pubkey $(KEY_FILE) --key-id $(PUBLISHER_ID)); \
	cd $(BE_DIR) && CC_GATEWAY_ADDR=$(GW_ADDR) CC_GATEWAY_URL=$(GW_URL) CC_WORKSPACE=$(WS) CC_STORE_PATH=$(STORE_PATH) CC_SEED_USER=$(SEED_USER) CC_TRUSTED_PUBKEYS=$$TRUSTED cargo run -p $(NODE_BIN)

# Just the mobile shell dev server, browser build, pointed at the gateway. Pair with `make
# cloud` in another terminal.
ui:
	cd $(UI_DIR) && VITE_GATEWAY_URL=$(GW_URL) pnpm run dev

# ---------------------------------------------------------------------------------------------------
# Extension dev flow: build → pack (sign) → publish (upload, which installs + loads on the server).
# `lb-pack` is the bridge build.sh never had: it turns a built extension crate + manifest into the
# SIGNED Artifact JSON the gateway's `POST /extensions` and the UI's UploadArtifact accept. The dev
# publisher key lives at $(KEY_FILE) (generated on first use); its public half is trusted by the node
# via CC_TRUSTED_PUBKEYS (the `dev`/`cloud` targets wire it from `lb-pack pubkey`).

# Build the lb-pack tool (the dev packager). Cheap once built; the run targets depend on it.
$(BE_DIR)/target/debug/lb-pack:
	cd $(BE_DIR) && cargo build -p lb-pack

# Print the dev publisher's `key_id=hexpubkey` (generating the key on first run). This IS the value
# the node wants in CC_TRUSTED_PUBKEYS; the `dev`/`cloud` targets capture it automatically.
trusted-pubkey: $(BE_DIR)/target/debug/lb-pack
	@$(BE_DIR)/target/debug/lb-pack pubkey $(KEY_FILE) --key-id $(PUBLISHER_ID)

# Build the care extension and package it into a signed artifact at $(EXT_ARTIFACT). Pure local:
# produces the file the UI can upload OR `make publish-ext` can POST.
pack: $(BE_DIR)/target/debug/lb-pack
	@echo "→ building care extension"
	cd $(BE_DIR)/extensions/$(EXT) && cargo build --release
	@mkdir -p $(ART_DIR)
	$(BE_DIR)/target/debug/lb-pack $(EXT_MANIFEST) $(KEY_FILE) \
		--key-id $(PUBLISHER_ID) --out $(EXT_ARTIFACT)

# Publish $(EXT) to a RUNNING node ($(GW_URL)): pack it, log in for a session token (the dev-login
# grants ext.publish), then POST the artifact. `204` ⇒ verified, installed, and LOADED live — the
# extension is reachable immediately (no restart). Needs `make cloud`/`make dev` running first, plus
# curl + jq. The node must trust this publisher key (the run targets set CC_TRUSTED_PUBKEYS for you).
publish-ext: pack
	@command -v jq >/dev/null || { echo "publish-ext needs jq"; exit 1; }
	@echo "→ login $(GW_URL) as dev/$(WS)"
	@TOKEN=$$(curl -fsS -X POST $(GW_URL)/login -H 'content-type: application/json' \
		-d '{"user":"dev","workspace":"$(WS)"}' | jq -r .token); \
	echo "→ POST $(GW_URL)/extensions ($(EXT))"; \
	code=$$(curl -sS -o /tmp/cc-publish-resp -w '%{http_code}' -X POST $(GW_URL)/extensions \
		-H "authorization: Bearer $$TOKEN" -H 'content-type: application/json' \
		--data-binary @$(EXT_ARTIFACT)); \
	echo "← HTTP $$code"; \
	if [ "$$code" = "204" ]; then echo "published + installed + loaded: $(EXT)"; \
	else echo "FAILED ($$code): $$(cat /tmp/cc-publish-resp)"; exit 1; fi
	@if [ -d "$(EXT_UI_DIST)" ]; then \
		echo "-> deploy UI bundle -> $(EXT_UI_SERVE)"; \
		rm -rf "$(EXT_UI_SERVE)"; mkdir -p "$(EXT_UI_SERVE)"; \
		cp -r "$(EXT_UI_DIST)"/* "$(EXT_UI_SERVE)"/; \
		echo "  UI deployed ($(GW_URL)/extensions/$(EXT)/ui/assets/remoteEntry.js)"; \
		echo "  NOTE: extension pages load only in the BUILT shell — use 'make ui-preview', not 'make ui'."; \
	else echo "-> no ui/dist for $(EXT) -- skipping UI deploy"; fi

# ---------------------------------------------------------------------------------------------------
# Tests + gates. Real infra, seeded via the real write path — no mocks (CLAUDE.md rule 4).
test: test-be test-ui

test-be:
	cd $(BE_DIR) && cargo test --workspace

test-ui:
	cd $(UI_DIR) && pnpm test
	cd $(EXT_DIR) && pnpm test

lint:
	cd $(BE_DIR) && cargo clippy --all-targets -- -D warnings
	cd $(UI_DIR) && pnpm exec tsc --noEmit
	cd $(EXT_DIR) && pnpm exec tsc --noEmit

fmt:
	cd $(BE_DIR) && cargo fmt

fmt-check:
	cd $(BE_DIR) && cargo fmt --all --check

# The FILE-LAYOUT ≤400-line gate (the S0 CI check). One responsibility per file.
size:
	bash $(BE_DIR)/scripts/check-file-size.sh

# en ⇔ es key parity gate for both UI trees (CLAUDE.md rule 8 — phase-1 MUST).
i18n-check:
	cd $(UI_DIR) && node scripts/i18n-check.mjs
	cd $(EXT_DIR) && node scripts/i18n-check.mjs

# Remove build artifacts — the cargo target and the UI build outputs. Leaves
# node_modules alone (re-run `make setup` / `pnpm install` to refresh those).
clean:
	cd $(BE_DIR) && cargo clean
	rm -rf $(UI_DIR)/dist $(EXT_DIR)/dist
	@echo "cleaned cargo target + ui/dist + ext UI dist (node_modules kept)"

# Wipe ONLY the dev node store — immediate relief for a dev box whose store bloated. Stop
# the node first (`make kill`) so nothing is holding the store open.
purge-store:
	rm -rf $(STORE_PATH)
	@echo "purged dev store at $(STORE_PATH) — next \`make dev\` boots a fresh, idle store"

# Free the dev ports AND reap any orphaned node/cargo/vite left by a crashed run. Mirrors lb's
# kill target verbatim — same `[c]argo` / `[v]ite` bracket pattern, same SIGTERM-then-SIGKILL
# escalation, same ~5s grace window. The bracket class keeps the pattern STRING out of the
# recipe shell's argv so pkill doesn't SIGKILL itself.
kill:
	-@fuser -TERM -k $(GW_PORT)/tcp 2>/dev/null || true
	-@fuser -TERM -k $(UI_PORT)/tcp 2>/dev/null || true
	-@pkill -TERM -f '[c]argo run' 2>/dev/null || true
	-@pkill -TERM -f 'target/[d]ebug/cc-node' 2>/dev/null || true
	-@pkill -TERM -f '[v]ite' 2>/dev/null || true
	@i=0; \
	while pgrep -f 'target/[d]ebug/cc-node' >/dev/null 2>&1 \
	   || pgrep -f '[c]argo run' >/dev/null 2>&1 \
	   || pgrep -f '[v]ite' >/dev/null 2>&1; do \
		i=$$((i+1)); \
		if [ $$i -ge 50 ]; then \
			pkill -KILL -f '[c]argo run' 2>/dev/null || true; \
			pkill -KILL -f 'target/[d]ebug/cc-node' 2>/dev/null || true; \
			pkill -KILL -f '[v]ite' 2>/dev/null || true; \
			break; \
		fi; \
		sleep 0.1; \
	done
	@echo "freed ports $(GW_PORT)/$(UI_PORT) and killed any orphaned cc-node/cargo/vite"