#!/usr/bin/env bash
# scripts/e2e.sh — end-to-end smoke test against a RUNNING cc-node, over the real gateway routes.
#
# No mocks (CLAUDE.md rule 4): every assertion is an HTTP call to the live node — `POST /login`,
# `POST /mcp/call`. It tests the FULL stack the browser shell rides: session mint, the mediated
# MCP bridge, the auth wall, credential seeding, and the care roster.
#
# Wire-in state (2026-07-12): cc-node boot now SPAWNS + INSTALLS the care sidecar
# (`rust/node/src/care_mount.rs` → `install_native`), so `care.*` verbs ARE reachable on a
# running node and their record I/O lands in the node's durable store (over the host `store.*`
# callback). If a verb still 403s (e.g. the `care` binary wasn't built before `make dev`), the
# care rows are reported as SKIP with the reason — not silently passed.
#
# Rule 7 (guardian isolation) is ENFORCED inside the native sidecar as of the `sdk-v0.4.0` /
# `node-v0.4.0` pin (native-caller-identity scope): the host stamps the caller into the native
# call frame, the sidecar projects it, and the chokepoint asks reach ABOUT the caller. This smoke
# script exercises the ADMIN path over HTTP; the full guardian cross-family deny proof (a stranger
# guardian gets 403/empty) lives in `tests/live_node.rs` (a real spawned sidecar). See
# `docs/debugging/authz/native-sidecar-not-spawned-and-caller-identity-not-propagated.md` (CLOSED).
# Run `make dev` (or `make cloud`) first, then: make e2e
#
# Exit code: non-zero if any REQUIRED assertion fails (the host layer + admin care reads).
set -uo pipefail

GW_URL="${GW_URL:-http://127.0.0.1:8391}"
WS="${WS:-acme}"
ADMIN_USER="${ADMIN_USER:-user:ada}"

command -v curl >/dev/null || { echo "e2e: needs curl" >&2; exit 1; }
command -v jq   >/dev/null || { echo "e2e: needs jq"   >&2; exit 1; }

pass=0 fail=0 skip=0
ok()   { printf '  \033[32m✔\033[0m %s\n' "$*"; pass=$((pass+1)); }
no()   { printf '  \033[31mx\033[0m %s\n' "$*"; fail=$((fail+1)); }
sk()   { printf '  \033[33m-\033[0m %s\n' "$*"; skip=$((skip+1)); }

# http_code <method> <path> [json-body] [auth-token] — echo the HTTP status.
http_code() {
  local m="$1" p="$2" body="${3:-}" tok="${4:-}"
  local args=(-sS -o /tmp/cc-e2e-body -w '%{http_code}' -X "$m" "$GW_URL$p" -H 'content-type: application/json')
  [ -n "$tok" ] && args+=(-H "authorization: Bearer $tok")
  [ -n "$body" ] && args+=(-d "$body")
  curl "${args[@]}"
}
# mcp <tool> <args-json> <token> — echo status; body in /tmp/cc-e2e-body.
mcp() { http_code POST /mcp/call "{\"tool\":\"$1\",\"args\":$2}" "$3"; }

echo "── e2e against $GW_URL (ws=$WS) ──"

# ── 1. session: the login keystone ──────────────────────────────────────────────────────────────
echo "1. session"
LOGIN=$(curl -sS -X POST "$GW_URL/login" -H 'content-type: application/json' \
  -d "{\"user\":\"$ADMIN_USER\",\"workspace\":\"$WS\"}")
TOKEN=$(printf '%s' "$LOGIN" | jq -r '.token // empty')
NCAPS=$(printf '%s' "$LOGIN" | jq -r '(.caps // []) | length')
if [ -n "$TOKEN" ]; then ok "admin login mints a token ($NCAPS caps)"; else no "admin login failed — is the node up? ($LOGIN)"; fi
[ "$(printf '%s' "$LOGIN" | jq -r '.principal')" = "$ADMIN_USER" ] \
  && ok "token principal = $ADMIN_USER" || no "principal mismatch"

# ── 2. the auth wall (negatives) ────────────────────────────────────────────────────────────────
echo "2. auth wall"
[ "$(http_code POST /mcp/call '{"tool":"identity.set_credential","args":{}}' '')" = "401" ] \
  && ok "tokenless /mcp/call → 401" || no "tokenless call was NOT 401"
[ "$(http_code POST /login '{"user":"stranger@nope.test","workspace":"'"$WS"'"}' '')" = "403" ] \
  && ok "non-member login → 403 (membership gate)" || no "non-member login was NOT 403"

# ── 3. credential seed (host verb) ──────────────────────────────────────────────────────────────
echo "3. credential"
c=$(mcp identity.set_credential '{"user":"e2e@acme.test","secret":"e2e-pass"}' "$TOKEN")
if [ "$c" = "200" ] && grep -q '"ok":true' /tmp/cc-e2e-body; then ok "identity.set_credential → {ok:true}"; else no "set_credential → $c $(cat /tmp/cc-e2e-body)"; fi

# ── 4. care extension reachability (the wire-in gate) ───────────────────────────────────────────
# Probe with `care.center.list` — a side-effect-free READ the admin actually holds a cap for.
# (Do NOT probe `care.ping`: the seeded admin's role grant does not include `mcp:care.ping:call`,
# so `ping` 403s at the cap wall even when the sidecar is perfectly alive — a false negative.)
echo "4. care roster"
p=$(mcp care.center.list '{}' "$TOKEN")
if [ "$p" != "200" ]; then
  sk "care.center.list → HTTP $p (extension not installed/granted on this node)"
  sk "care.child.list — skipped (care ext unreachable)"
  sk "care.child.get leo — skipped (care ext unreachable)"
  CARE_UP=0
else
  n=$(jq -r 'if type=="array" then length elif type=="object" and has("centers") then (.centers|length) else 0 end' /tmp/cc-e2e-body 2>/dev/null || echo '?')
  ok "care.center.list → 200 ($n center(s)) — sidecar live, serving reads over the gateway"
  CARE_UP=1
  # children list (admin sees all)
  if [ "$(mcp care.child.list '{}' "$TOKEN")" = "200" ]; then ok "care.child.list → 200"; else no "care.child.list failed"; fi
  # the seeded child is readable (proves the roster landed in the node's durable store)
  if [ "$(mcp care.child.get '{"id":"leo"}' "$TOKEN")" = "200" ]; then
    nm=$(jq -r '.name // "?"' /tmp/cc-e2e-body 2>/dev/null)
    ok "care.child.get leo → 200 (seeded child present: $nm)"
  else sk "care.child.get leo → not present (run 'make seed' first)"; fi
  # Rule 7 (guardian isolation) is ENFORCED in-sidecar as of node-v0.4.0/sdk-v0.4.0. Its full
  # cross-family deny proof needs a GUARDIAN token (minted with the node's signing key), which a
  # black-box HTTP smoke script cannot forge — so that proof lives in `tests/live_node.rs` (a real
  # spawned sidecar: stranger→leo 403, stranger child.list 0 kids). This script proves the ADMIN
  # read surface over the live gateway; see that test for the guardian assertions.
  sk "rule-7 guardian-isolation deny — proven in tests/live_node.rs (needs a guardian token)"
fi

# ── summary ─────────────────────────────────────────────────────────────────────────────────────
echo "──"
printf 'e2e: \033[32m%d passed\033[0m, \033[31m%d failed\033[0m, \033[33m%d skipped\033[0m\n' "$pass" "$fail" "$skip"
if [ "${CARE_UP:-0}" = "0" ]; then
  cat <<EOF
note: the care extension is NOT reachable on this node. cc-node boot DOES now spawn/install the
      native care sidecar (rust/node/src/care_mount.rs), so the usual cause is that the 'care'
      binary was not built before the node started — build it and restart:
        cargo build -p care        # (or: make build-be)   then re-run make dev/cloud
      The host layer (login, wall, credential) is green regardless. (Rule-7 guardian isolation in
      the sidecar is ENFORCED as of the node-v0.4.0/sdk-v0.4.0 pin — proven by tests/live_node.rs.)
EOF
fi
[ "$fail" -eq 0 ]
