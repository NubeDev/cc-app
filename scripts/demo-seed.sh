#!/usr/bin/env bash
# scripts/demo-seed.sh — layer RICH demo data on a running cc-node so every m03–m07
# surface shows real content: a multi-room roster of children with varied allergies +
# authorized pickups, enrollments, a full planned MENU WEEK (with allergen-triggered
# substitutions, some deliberately UNRESOLVED so the red flags show), and TODAY's
# ATTENDANCE (children checked in/out + staff present, so the occupancy dashboard and
# ratios are non-empty).
#
# Same posture as `scripts/seed.sh` (CLAUDE.md rule 4): real write path only — every
# write is a bridged `POST /mcp/call` gated by the cap wall. Idempotent: re-runs skip
# existing records (a duplicate id is treated as success). Run the base seed first
# (it creates the admin credential + the 'sunshine' center this builds on):
#
#   make dev          # (leave running)
#   make seed         # base roster + login
#   make demo-seed    # this — the rich demo layer
#
# Overridable knobs (env): GW_URL, WS, ADMIN_USER, ADMIN_PASSWORD, TODAY, WEEK_MON.
set -euo pipefail

GW_URL="${GW_URL:-http://127.0.0.1:8391}"
WS="${WS:-acme}"
ADMIN_USER="${ADMIN_USER:-user:ada}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-cc-admin-1234}"

# Dates drive the menu week + attendance. Default to real wall dates; override for a
# reproducible demo. TODAY = attendance day; WEEK_MON = the Monday the menu week starts.
TODAY="${TODAY:-$(date +%F)}"
WEEK_MON="${WEEK_MON:-$(date -d "monday this week" +%F 2>/dev/null || date +%F)}"

command -v curl >/dev/null || { echo "demo-seed: needs curl" >&2; exit 1; }
command -v jq   >/dev/null || { echo "demo-seed: needs jq"   >&2; exit 1; }

say() { printf '  %s\n' "$*"; }

# --- session -------------------------------------------------------------------------
echo "→ login $GW_URL as $ADMIN_USER / $WS"
TOKEN="$(curl -fsS -X POST "$GW_URL/login" -H 'content-type: application/json' \
  -d "{\"user\":\"$ADMIN_USER\",\"workspace\":\"$WS\",\"secret\":\"$ADMIN_PASSWORD\"}" | jq -r .token)"
[ -n "$TOKEN" ] && [ "$TOKEN" != "null" ] || {
  echo "demo-seed: admin login failed — is the node up ('make dev') with ADMIN_PASSWORD=$ADMIN_PASSWORD?" >&2
  exit 1
}

# mcp <tool> <json-args> — one bridged MCP call. 200 → prints reply; a duplicate-id
# conflict is tolerated (idempotent); any other non-2xx aborts (surfaced loudly).
mcp() {
  local tool="$1" args="$2" code
  code="$(curl -sS -o /tmp/cc-demo-resp -w '%{http_code}' -X POST "$GW_URL/mcp/call" \
    -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
    -d "{\"tool\":\"$tool\",\"args\":$args}")" || true
  if [ "$code" = "200" ]; then cat /tmp/cc-demo-resp; return 0; fi
  if grep -qi 'already exists\|conflict' /tmp/cc-demo-resp; then
    say "(exists) $tool — skipped"; return 0
  fi
  echo "demo-seed: $tool FAILED (HTTP $code): $(cat /tmp/cc-demo-resp)" >&2
  return 1
}

# Preflight: the care sidecar must be reachable (its verbs 403 until installed).
echo "→ preflight: care extension reachable?"
pc="$(curl -sS -o /dev/null -w '%{http_code}' -X POST "$GW_URL/mcp/call" \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"tool":"care.center.list","args":{}}')"
if [ "$pc" != "200" ]; then
  echo "demo-seed: care extension not reachable (care.center.list → HTTP $pc). Run 'make seed' first (it builds/points the sidecar); then re-run." >&2
  exit 2
fi

# --- 1. centers + rooms --------------------------------------------------------------
# The base seed made center 'sunshine' + room 'possums'. Add two more rooms so the
# occupancy dashboard and room pickers have real choices.
echo "→ rooms"
mcp care.center.create '{"id":"sunshine","name":"Sunshine Childcare","default_locale":"en","address":"1 Playground Way","phone":"555-0100","email":"hello@sunshine.test"}' >/dev/null
for r in \
  '{"id":"possums","name":"Possums","center_id":"sunshine"}' \
  '{"id":"koalas","name":"Koalas","center_id":"sunshine"}' \
  '{"id":"wombats","name":"Wombats","center_id":"sunshine"}'; do
  mcp care.room.create "$r" >/dev/null
done
say "rooms: Possums, Koalas, Wombats"

# --- 2. children (varied allergies + pickups + emergency contacts) -------------------
# Allergies span the fixed set (peanut/dairy/egg/gluten) + a free-text one + none, so
# the menu derivation lights up different children on different days. Each child gets
# an authorized pickup (grandma etc.) so the check-out picker has real candidates.
echo "→ children"
# id | name | dob | room | allergies-json | pickups-json
add_child() {
  local id="$1" name="$2" dob="$3" room="$4" allergies="$5" pickups="$6"
  mcp care.child.create "{\"id\":\"$id\",\"name\":\"$name\",\"dob\":\"$dob\",\"room_id\":\"$room\",\"allergies\":$allergies,\"authorized_pickups\":$pickups,\"emergency_contacts\":[{\"name\":\"Dr. Reyes\",\"phone\":\"555-0900\",\"relationship\":\"pediatrician\"}],\"photo_consent\":true}" >/dev/null
}
add_child leo    "Leo García"     2021-03-15 possums '["peanuts"]'        '[{"name":"Grandma Jo","phone":"555-0301"}]'
add_child mia    "Mia Nowak"      2020-11-02 koalas  '["dairy","egg"]'    '[{"name":"Uncle Tom","phone":"555-0302"}]'
add_child noah   "Noah Kim"       2021-06-20 possums '["gluten"]'         '[{"name":"Aunt Sara","phone":"555-0303"}]'
add_child emma   "Emma Silva"     2020-09-10 koalas  '[]'                 '[{"name":"Grandpa Ray","phone":"555-0304"}]'
add_child liam   "Liam O'Brien"   2021-01-25 wombats '["tree nuts"]'      '[{"name":"Nanny Pat","phone":"555-0305"}]'
add_child ava    "Ava Rossi"      2020-12-05 wombats '["shellfish"]'      '[{"name":"Grandma Vi","phone":"555-0306"}]'
add_child sofia  "Sofía Mendez"   2021-04-18 possums '["peanuts","dairy"]' '[{"name":"Tía Rosa","phone":"555-0307"}]'
add_child oliver "Oliver Chen"    2020-08-30 koalas  '[]'                 '[{"name":"Grandpa Wu","phone":"555-0308"}]'
say "children: 8 across 3 rooms (peanut, dairy+egg, gluten, tree-nut, shellfish, none)"

# --- 3. guardians + edges (rule 7 reach) ---------------------------------------------
# Each family gets a guardian record + a live guardianship edge with can_pickup so the
# pickup gate allows them, receives_daily_feed so the feed fans out later (m08).
echo "→ guardians + edges"
add_guardian() { mcp care.guardian.create "{\"id\":\"$1\",\"name\":\"$2\",\"email\":\"$3\",\"phone\":\"$4\",\"locale\":\"$5\"}" >/dev/null; }
link() { mcp care.guardianship.link "{\"guardian_sub\":\"$1\",\"child_id\":\"$2\",\"relationship\":\"$3\",\"can_pickup\":true,\"receives_daily_feed\":true,\"emergency_contact\":true}" >/dev/null; }
add_guardian ana    "Ana García"    ana@familia.test    555-0111 es
add_guardian marek  "Marek Nowak"   marek@familia.test  555-0112 en
add_guardian dan    "Dan Kim"       dan@familia.test    555-0113 en
add_guardian bianca "Bianca Silva"  bianca@familia.test 555-0114 es
add_guardian shen   "Shen Chen"     shen@familia.test   555-0115 en
link ana    leo    mother
link marek  mia    father
link dan    noah   father
link bianca emma   mother
link ana    sofia  mother
link shen   oliver father
say "guardians: 5, edges: 6 (all can_pickup + receives_daily_feed)"

# --- 4. enrollments (active + a waitlist entry) --------------------------------------
echo "→ enrollments"
enroll() { mcp care.enrollment.create "{\"child_id\":\"$1\",\"room_id\":\"$2\",\"status\":\"$3\",\"schedule\":[\"mon\",\"tue\",\"wed\",\"thu\",\"fri\"],\"start_date\":\"2025-09-01\"}" >/dev/null; }
enroll leo possums enrolled
enroll mia koalas enrolled
enroll noah possums enrolled
enroll emma koalas enrolled
enroll liam wombats enrolled
enroll ava wombats enrolled
enroll sofia possums enrolled
enroll oliver koalas enrolled
# One waitlist entry so the waitlist screen isn't empty.
mcp care.enrollment.create '{"child_id":"leo","room_id":"koalas","status":"waitlist"}' >/dev/null
say "enrollments: 8 enrolled + 1 waitlisted"

# --- 5. menu week (allergen-tagged; some substitutions UNRESOLVED for the red flags) -
# A week of lunches + snacks per room. Peanut satay flags Leo & Sofía; cheese pizza
# flags Mia & Sofía; some cells leave the substitute BLANK so the planner shows the
# loud red "needs a substitute" state the food-safety surface is about.
echo "→ menu week (starting Monday $WEEK_MON)"
# day offset helper (portable-ish: GNU date)
dow() { date -d "$WEEK_MON +$1 day" +%F 2>/dev/null || echo "$WEEK_MON"; }
setmenu() { mcp care.menu.set "$1" >/dev/null; }
# Possums — Mon lunch: peanut satay WITH a substitute (resolved); Tue lunch: cheese pizza with NO sub (unresolved).
setmenu "{\"room_id\":\"possums\",\"date\":\"$(dow 0)\",\"slot\":\"lunch\",\"items\":[{\"name\":\"Peanut satay chicken\",\"allergens\":[\"peanut\"]},{\"name\":\"Steamed rice\",\"allergens\":[]}],\"substitutions\":[{\"restriction\":\"peanut\",\"substitute\":\"Sunflower-butter satay\"}]}"
setmenu "{\"room_id\":\"possums\",\"date\":\"$(dow 0)\",\"slot\":\"am_snack\",\"items\":[{\"name\":\"Apple slices\",\"allergens\":[]}],\"substitutions\":[]}"
setmenu "{\"room_id\":\"possums\",\"date\":\"$(dow 1)\",\"slot\":\"lunch\",\"items\":[{\"name\":\"Cheese pizza\",\"allergens\":[\"dairy\",\"gluten\"]},{\"name\":\"Garden salad\",\"allergens\":[]}],\"substitutions\":[]}"
setmenu "{\"room_id\":\"possums\",\"date\":\"$(dow 2)\",\"slot\":\"lunch\",\"items\":[{\"name\":\"Turkey wrap\",\"allergens\":[\"gluten\"]},{\"name\":\"Carrot sticks\",\"allergens\":[]}],\"substitutions\":[{\"restriction\":\"gluten\",\"substitute\":\"Lettuce-wrap turkey\"}]}"
# Koalas — Mon lunch: scrambled eggs (flags Mia) unresolved; a mystery casserole (untaggable → conservative flag).
setmenu "{\"room_id\":\"koalas\",\"date\":\"$(dow 0)\",\"slot\":\"lunch\",\"items\":[{\"name\":\"Scrambled eggs & toast\",\"allergens\":[\"egg\",\"gluten\"]},{\"name\":\"Chef's mystery casserole\",\"allergens\":[]}],\"substitutions\":[]}"
setmenu "{\"room_id\":\"koalas\",\"date\":\"$(dow 1)\",\"slot\":\"lunch\",\"items\":[{\"name\":\"Yogurt parfait\",\"allergens\":[\"dairy\"]},{\"name\":\"Banana\",\"allergens\":[]}],\"substitutions\":[{\"restriction\":\"dairy\",\"substitute\":\"Coconut-yogurt parfait\"}]}"
# Wombats — Mon lunch: shrimp fried rice (flags Ava) unresolved; walnut muffins (flags Liam).
setmenu "{\"room_id\":\"wombats\",\"date\":\"$(dow 0)\",\"slot\":\"lunch\",\"items\":[{\"name\":\"Shrimp fried rice\",\"allergens\":[\"shellfish\"]},{\"name\":\"Steamed broccoli\",\"allergens\":[]}],\"substitutions\":[]}"
setmenu "{\"room_id\":\"wombats\",\"date\":\"$(dow 0)\",\"slot\":\"pm_snack\",\"items\":[{\"name\":\"Walnut muffins\",\"allergens\":[\"tree_nut\"]}],\"substitutions\":[{\"restriction\":\"tree_nut\",\"substitute\":\"Oat muffins\"}]}"
say "menus: 3 rooms × several days (resolved + UNRESOLVED substitutions for the red flags)"

# --- 6. today's attendance (present/out + staff, so the dashboard is non-empty) ------
# Staff check in first (so ratios have a denominator), then most children check in;
# one child (oliver) checks in then out to exercise the out-state; one (ava) stays out.
echo "→ today's attendance ($TODAY)"
uuid() { cat /proc/sys/kernel/random/uuid 2>/dev/null || echo "evt-$RANDOM-$RANDOM"; }
checkin_child()  { mcp care.attendance.check_in  "{\"event_id\":\"$(uuid)\",\"child_id\":\"$1\",\"room_id\":\"$2\",\"at\":\"${TODAY}T08:$3:00Z\",\"person\":\"$4\"}" >/dev/null; }
checkin_staff()  { mcp care.attendance.check_in  "{\"event_id\":\"$(uuid)\",\"staff_sub\":\"$1\",\"room_id\":\"$2\",\"at\":\"${TODAY}T07:45:00Z\"}" >/dev/null; }
checkout_child() { mcp care.attendance.check_out "{\"event_id\":\"$(uuid)\",\"child_id\":\"$1\",\"room_id\":\"$2\",\"at\":\"${TODAY}T${3}:00Z\",\"collector_name\":\"$4\"}" >/dev/null; }
# staff presence (2 in possums+koalas, 1 in wombats → varied ratios)
checkin_staff user:rivera possums
checkin_staff user:rivera koalas
checkin_staff user:teacher-2 possums
checkin_staff user:teacher-3 wombats
# children in
checkin_child leo    possums 02 "Ana García"
checkin_child noah   possums 05 "Dan Kim"
checkin_child sofia  possums 08 "Ana García"
checkin_child mia    koalas  03 "Marek Nowak"
checkin_child emma   koalas  06 "Bianca Silva"
checkin_child oliver koalas  04 "Shen Chen"
checkin_child liam   wombats 07 "Nanny Pat"
# one leaves early (authorized pickup by name), one stays out (ava never checks in)
checkout_child oliver koalas 12:30 "Grandpa Wu"
say "attendance: staff present in 3 rooms; 7 children in, 1 checked out, 1 never in"

# --- done ----------------------------------------------------------------------------
cat <<EOF

✔ demo-seed complete — workspace '$WS' at $GW_URL

  Sign in at http://127.0.0.1:5391 as the admin (${ADMIN_USER#user:} / $ADMIN_PASSWORD),
  then explore the now-populated surfaces:
    • Attendance → Who's here     — 3 rooms with live child/staff counts + ratios
    • Attendance → Roster         — pick a room; children show present/out; tap to check in/out
    • Menus (admin planner)       — week of $WEEK_MON; red "needs a substitute" flags on
                                     Possums Tue (cheese pizza) & Koalas Mon (eggs) & Wombats Mon (shrimp)
    • Children                    — 8 children with allergy ⚠ badges

  Guardian view: log in as a guardian (accept-link from 'make seed') to see their child's
  menu week with only THAT child's substitutions (rule 7).

  Re-running is safe (idempotent — existing records are skipped).
EOF
