#!/usr/bin/env bash
# check-i18n-parity.sh — the catalog completeness gate (i18n-scope §Enforcement).
#
# CLAUDE.md rule 8 (sacred): every user-facing surface ships 100% in en
# AND es from day one. The CI catalog completeness check is the
# enforcement: every key present in `i18n/en.json` must exist in
# `i18n/es.json` and vice versa. A key added in one catalog without the
# other fails the build (the missing-translation guard). Hardcoded-
# string lint on user-facing code is the second guard (the no-literal
# guard); see scripts/check-hardcoded-strings.sh.
#
# Run from the repo root. Compares the leaf keys (the dot-paths the
# catalog is read through) of every JSON file in `i18n/`.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
I18N_DIR="$ROOT/i18n"

if [ ! -d "$I18N_DIR" ]; then
  echo "::error::i18n directory missing: $I18N_DIR"
  exit 1
fi

# Discover every locale catalog in the directory. Each file MUST be a
# valid JSON object whose keys are dot-path prefixes (no extension
# filtering — a typo in the locale extension fails the validation).
mapfile -t catalogs < <(find "$I18N_DIR" -maxdepth 1 -type f -name "*.json" | sort)

if [ "${#catalogs[@]}" -lt 2 ]; then
  echo "::error::i18n has ${#catalogs[@]} catalog(s); need at least 2 (en + es)"
  exit 1
fi

fail=0

# Walk a JSON tree, printing every leaf path. Uses python3 (universally
# available in CI; faster + more correct than a hand-rolled jq-free
# walker).
leaves() {
  python3 -c '
import json, sys
data = json.load(sys.stdin)
def walk(node, path):
    if isinstance(node, dict):
        for k, v in node.items():
            walk(v, path + [k])
    else:
        print(".".join(path))
walk(data, [])
' < "$1" | sort -u
}

# For each catalog pair: every leaf in either must be in both.
# We compute the UNION across all catalogs and assert each catalog
# contains the full union. That catches both "added in en, missing in
# es" AND "added in es, missing in en" with one pass.
union="$(mktemp)"
trap "rm -f $union" EXIT
> "$union"
for c in "${catalogs[@]}"; do
  leaves "$c" >> "$union"
done
sort -u "$union" -o "$union"

for c in "${catalogs[@]}"; do
  catalog_leaves="$(mktemp)"
  leaves "$c" > "$catalog_leaves"
  missing="$(comm -23 "$union" "$catalog_leaves")"
  if [ -n "$missing" ]; then
    echo "::error::$(basename "$c") is missing keys present in another locale:"
    echo "$missing" | sed 's/^/  /'
    fail=1
  fi
  rm -f "$catalog_leaves"
done

# Also check that the locale codes listed in `_meta.locale` match the
# file names (a typo like `i18n/enn.json` would slip the leaf check).
for c in "${catalogs[@]}"; do
  meta_locale="$(python3 -c "import json,sys; print(json.load(open('$c')).get('_meta',{}).get('locale','?'))")"
  file_stem="$(basename "$c" .json)"
  if [ "$meta_locale" != "$file_stem" ]; then
    echo "::error::$c declares _meta.locale='$meta_locale' but filename is '$file_stem.json' (must match)"
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  echo "i18n parity check FAILED — every locale catalog must contain every leaf key, and filename must match _meta.locale"
  exit 1
fi

n_keys="$(wc -l < "$union")"
echo "i18n: ${#catalogs[@]} catalogs, ${n_keys} leaf keys, parity OK"