#!/usr/bin/env sh
# Build the `care` NATIVE Tier-2 extension — a host-target binary the host
# spawns + supervises over stdio. The native sidecar's Tools impl serves
# the `care.*` verbs (today: just `care.ping`).
#
# Emits:
#   target/release/care       (the native child binary the host spawns)
#   ui/dist/remoteEntry.js    (the federated UI bundle, shipped flat into
#                              the node's ext-ui-dir by the host's install
#                              flow — `lb-ext` for milestone 04+)
#
# Mirrors rubix-ai's `host-metrics/build.sh` exactly (the canonical
# "why native" reference). NO wasm build (care is native-tier — see
# `extension.toml` `[runtime].tier`).
set -e
cd "$(dirname "$0")"

echo "==> building native host-target binary"
cargo build --release
echo "built: $(pwd)/target/release/care"

# The UI bundle is built by milestone 04 (the `ui/` folder is a stub
# today, awaiting lb's `minimal-shell` scope landing). When it ships,
# this block is the shape:
#
#   if [ -d ui ]; then
#     echo "==> building federated UI bundle"
#     cd ui
#     pnpm install --frozen-lockfile || pnpm install || true
#     ./node_modules/.bin/vite build
#     cd ..
#     echo "built: $(pwd)/ui/dist/remoteEntry.js"
#   fi
#
# (Mirrors host-metrics's exact pattern; we keep it commented so the
# build script works today even with the UI stub.)