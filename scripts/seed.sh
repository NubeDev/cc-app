#!/usr/bin/env bash
# scripts/seed.sh — seed a RUNNING cc-node with a demo login + the minimal care roster.
#
# Idempotent, real-write-path only (CLAUDE.md rule 4): every write goes through a mediated
# gateway route — `POST /login` (session), `POST /mcp/call` (the one universal MCP contract).
# We NEVER touch the store directly; the node's cap wall gates every call exactly as it would
# a browser.
#
# What it seeds (the "1x of each" the demo needs to sign in and see real data):
#   1. a LOGIN CREDENTIAL  — email + password via `identity.set_credential` so the browser
#                            login form (email/password) actually authenticates against a
#                            node running in argon2 mode.
#   2. 1 CENTER            — `care.center.create`
#   3. 1 ROOM              — `care.room.create` (the teacher + child need a home room)
#   4. 1 TEACHER (staff)   — `care.invite.create_staff` (records-before-accounts: the staff
#                            record is minted on invite; this is the real staff primitive).
#   5. 1 FAMILY            — 1 guardian (`care.guardian.create`) + 1 child
#                            (`care.child.create`) + the guardianship edge
#                            (`care.guardianship.link`) so the guardian can actually reach
#                            a child (rule 7).
#
# Prereq: a node reachable at $GW_URL. The admin bootstrap login is password-less, which needs
# the node booted with LB_DEV_LOGIN set (the `make dev`/`cloud` targets do this). Run:
#   make dev            # (leave running)
#   make seed           # in another terminal
#
# Then log into the browser shell with the seeded EMAIL + PASSWORD printed at the end.
#
# Overridable knobs (env): GW_URL, WS, ADMIN_USER, SEED_EMAIL, SEED_PASSWORD.
set -euo pipefail

GW_URL="${GW_URL:-http://127.0.0.1:8391}"
WS="${WS:-acme}"
# The bootstrap admin — a workspace-admin member the node seeds at boot (CC_SEED_USER). Under the
# default PasswordHash node (CC_PASSWORD_LOGIN=1) the node ALSO seeds this admin an argon2 credential
# (CC_SEED_PASSWORD), so the login below authenticates with that password. Under a password-less
# DevTrustAny node the password is ignored (any secret works). Either way the token every seed write
# rides is minted here.
ADMIN_USER="${ADMIN_USER:-user:ada}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-cc-admin-1234}"

# The DEMO guardian the email-login path uses. `care.guardian.create` already seeds Ana's record
# (below); the real email+password login is: mint an invite for her → open /invite/<token> → accept
# (sets membership + HER password) → thereafter email+password /login works. The seed prints the
# accept URL + token at the end. (SEED_EMAIL/SEED_PASSWORD kept for the legacy admin-credential seed.)
SEED_EMAIL="${SEED_EMAIL:-admin@acme.test}"
SEED_PASSWORD="${SEED_PASSWORD:-cc-demo-1234}"
GUARDIAN_EMAIL="ana@familia.test"

command -v curl >/dev/null || { echo "seed: needs curl" >&2; exit 1; }
command -v jq   >/dev/null || { echo "seed: needs jq"   >&2; exit 1; }

say() { printf '  %s\n' "$*"; }

# --- session -------------------------------------------------------------------------------------
# Password-less admin login (dev). The token carries the admin caps every write below is gated on.
echo "→ login $GW_URL as $ADMIN_USER / $WS (bootstrap admin)"
TOKEN="$(curl -fsS -X POST "$GW_URL/login" -H 'content-type: application/json' \
  -d "{\"user\":\"$ADMIN_USER\",\"workspace\":\"$WS\",\"secret\":\"$ADMIN_PASSWORD\"}" | jq -r .token)"
[ -n "$TOKEN" ] && [ "$TOKEN" != "null" ] || { echo "seed: admin login failed. Under PasswordHash mode the node seeds \$CC_SEED_PASSWORD for $ADMIN_USER — is the node up ('make dev') and is ADMIN_PASSWORD ($ADMIN_PASSWORD) the one it booted with?" >&2; exit 1; }

# mcp <tool> <json-args> — one bridged MCP call. Prints the tool's JSON reply. A `403` (cap deny)
# or `409`-style conflict surfaces as a non-2xx; we tolerate "already exists" so re-runs are no-ops.
mcp() {
  local tool="$1" args="$2" code body
  body="$(curl -sS -o /tmp/cc-seed-resp -w '%{http_code}' -X POST "$GW_URL/mcp/call" \
    -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
    -d "{\"tool\":\"$tool\",\"args\":$args}")" || true
  code="$body"
  if [ "$code" = "200" ]; then
    cat /tmp/cc-seed-resp
    return 0
  fi
  # Idempotency: a duplicate id is a success for a seed (the record is already there).
  if grep -qi 'already exists\|conflict' /tmp/cc-seed-resp; then
    say "(exists) $tool — skipped"
    return 0
  fi
  echo "seed: $tool FAILED (HTTP $code): $(cat /tmp/cc-seed-resp)" >&2
  return 1
}

# mcp_show <tool> <json-args> <jq-format> — like `mcp` but pretty-prints one field of the reply
# (best-effort). A hard failure inside `mcp` still aborts (set -e); the jq render never masks it.
mcp_show() {
  local out; out="$(mcp "$1" "$2")" || return 1
  printf '%s' "$out" | jq -r "$3" 2>/dev/null | sed 's/^/  /' || true
}

# Preflight: is the care extension installed + reachable on this node? Its verbs are 403 until it
# is published (which grants `mcp:care.*:call` to workspace-admin AND spawns the sidecar that
# services /mcp/call). We probe `care.center.list` — a side-effect-free READ the admin actually
# holds a cap for, and the exact surface the roster seed rides — and stop early with a clear
# pointer rather than emitting a stream of opaque 403s and a false "complete".
#
# NOTE: do NOT probe `care.ping` here. `ping` is a liveness verb the seeded admin's role grant
# does NOT include (the admin holds `mcp:care.<noun>.<verb>:call` for the real CRUD verbs, plus
# the `mcp:*.{create,get,list,update,…}:call` role wildcards — none of which match the bare
# `ping`). Probing `ping` 403s at the cap WALL even when the sidecar is perfectly alive, giving a
# false "care unreachable". `center.list` returns 200 `[]` on a live-but-empty node.
echo "→ preflight: care extension reachable?"
ping_code="$(curl -sS -o /dev/null -w '%{http_code}' -X POST "$GW_URL/mcp/call" \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"tool":"care.center.list","args":{}}')"

# --- 1. demo login credential --------------------------------------------------------------------
echo "→ set login credential for $SEED_EMAIL"
mcp identity.set_credential "{\"user\":\"$SEED_EMAIL\",\"secret\":\"$SEED_PASSWORD\"}" >/dev/null
# Make the demo login a workspace member too, so it can actually log in (an empty ws bootstraps it
# as admin on first login; a seeded ws needs an explicit member). We do that by logging it in once
# password-less as the admin does — but membership is minted by the login-resolve path, so a first
# login with the credential is enough. We defer that to the browser (the operator's first sign-in).
say "credential set (login as $SEED_EMAIL / <password> in the shell)"

# Gate the care roster on the extension being live. The credential above stands alone (it is a
# host verb), so it is always seeded; the domain records need the ext.
if [ "$ping_code" != "200" ]; then
  cat >&2 <<EOF

⚠ care extension not reachable (care.center.list → HTTP $ping_code) — the login credential IS seeded,
  but the center/room/teacher/family need the care sidecar running. cc-node installs the native
  care sidecar at BOOT (rust/node/src/care_mount.rs → install_native); if it 403s, the 'care'
  binary was almost certainly not built before the node started. Build it, restart the node,
  then re-run this seed (idempotent):

    cargo build -p care     # (or: make build-be)   — produces target/debug/care
    make dev                # restart the node so it spawns the freshly-built sidecar
    make seed               # re-run — the credential step is idempotent

EOF
  exit 2
fi

# --- 2. center -----------------------------------------------------------------------------------
echo "→ create center 'sunshine'"
mcp_show care.center.create \
  '{"id":"sunshine","name":"Sunshine Childcare","default_locale":"en","address":"1 Playground Way","phone":"555-0100","email":"hello@sunshine.test"}' \
  '"center: \(.name) (\(.id))"'

# --- 3. room -------------------------------------------------------------------------------------
echo "→ create room 'possums' in 'sunshine'"
mcp_show care.room.create '{"id":"possums","name":"Possums","center_id":"sunshine"}' \
  '"room: \(.name) (\(.id))"'

# --- 4. teacher (staff) --------------------------------------------------------------------------
# Records-before-accounts: the staff record is minted on invite. This rides the host callback
# (invite.create over the era-2 wire); if the ext isn't wired for era-2 it degrades — tolerate it.
echo "→ invite teacher 'Ms. Rivera' to room 'possums'"
if mcp care.invite.create_staff \
  '{"slot_id":"possums-lead","email":"rivera@sunshine.test","room_id":"possums","name":"Ms. Rivera","locale":"en"}' \
  >/dev/null 2>&1; then
  say "teacher invited: rivera@sunshine.test → possums"
else
  say "teacher invite skipped (needs the era-2 host callback; center+room+family still seeded)"
fi

# --- 5. family: guardian + child + edge ----------------------------------------------------------
echo "→ create guardian 'Ana García'"
mcp_show care.guardian.create \
  '{"id":"ana","name":"Ana García","email":"ana@familia.test","phone":"555-0111","locale":"es"}' \
  '"\(.message)"'

echo "→ create child 'Leo' in room 'possums'"
mcp_show care.child.create \
  '{"id":"leo","name":"Leo García","dob":"2021-03-15","room_id":"possums","allergies":["peanuts"],"photo_consent":true,"locale":"es"}' \
  '"\(.message)"'

echo "→ link guardian ↔ child (mother, pickup + daily feed)"
mcp_show care.guardianship.link \
  '{"guardian_sub":"ana","child_id":"leo","relationship":"mother","can_pickup":true,"receives_daily_feed":true,"emergency_contact":true,"locale":"es"}' \
  '"\(.message)"'

# --- 6. guardian invite (the email-login golden path) --------------------------------------------
# Mint a real lb invite for Ana. The verb returns only the invite id (records-before-accounts — the
# raw token ships in the email effect, never the reply). For a self-contained dev demo we recover the
# raw token from the OUTBOX (the durable send_invite effect), so the seed can print a ready-to-open
# accept URL. In production the token only ever reaches the invitee's inbox.
echo "→ invite guardian Ana (email-login golden path)"
mcp care.invite.create_guardian '{"guardian_id":"ana","locale":"es"}' >/dev/null 2>&1 \
  || say "guardian invite skipped (needs the era-2 host callback)"

# Recover the raw token from the outbox effect for THIS guardian's email (best-effort — dev only).
ACCEPT_TOKEN="$(curl -sS "$GW_URL/store/tables/outbox/rows" -H "authorization: Bearer $TOKEN" 2>/dev/null \
  | jq -r --arg email "$GUARDIAN_EMAIL" '
      [ .rows[]?.data.data
        | select(.action=="send_invite")
        | (.payload | fromjson)
        | select(.email==$email) ]
      | last // empty
      | .token' 2>/dev/null)"

# --- done ----------------------------------------------------------------------------------------
cat <<EOF

✔ seed complete — workspace '$WS' at $GW_URL

  ── Sign in as the ADMIN (workspace-admin) ──────────────────────────────────────
    URL:      http://127.0.0.1:5391
    email:    ${ADMIN_USER#user:}
    password: $ADMIN_PASSWORD
    (the node runs in real PasswordHash mode; this credential was seeded at boot —
     a WRONG password is rejected. For the old password-less mode: PASSWORD_LOGIN= make dev)

  ── Real EMAIL + PASSWORD login (the guardian golden path) ──────────────────────
    A guardian sets her OWN password by accepting an invite, then logs in with it:
      1. open the accept link below → set a password
      2. thereafter sign in at http://127.0.0.1:5391 with:
           email:    $GUARDIAN_EMAIL
           password: (the one she set on the accept page)
EOF
if [ -n "${ACCEPT_TOKEN:-}" ] && [ "$ACCEPT_TOKEN" != "null" ]; then
cat <<EOF
    Accept link (dev — token recovered from the outbox effect):
      http://127.0.0.1:5391/invite/$ACCEPT_TOKEN
EOF
else
cat <<EOF
    (accept token not recovered — the guardian invite may need the era-2 host callback;
     re-run 'make seed' once the care sidecar is reachable)
EOF
fi
cat <<EOF

  Seeded roster:
    center:   Sunshine Childcare (sunshine)
    room:     Possums (possums)
    teacher:  Ms. Rivera  (rivera@sunshine.test — staff invite)
    family:   Ana García (guardian, $GUARDIAN_EMAIL) → Leo García (child), live edge

  Re-running is safe (idempotent — existing records are skipped).
EOF
