#!/usr/bin/env bash
# check-file-size.sh — FILE-LAYOUT enforcement (docs/FILE-LAYOUT.md §8).
# Fails CI on any tracked human *.rs/*.ts/*.tsx over the 400-line hard limit.
# Mirrors lb's rust/scripts/check-file-size.sh verbatim, scoped to this repo.
#
# With `--all`, also walks the on-disk source tree (handy in a sandbox
# where nothing's committed yet, and a developer's WIP).
set -euo pipefail

LIMIT=400
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Tracked source files, minus generated trees.
mapfile -t files < <(git ls-files '*.rs' '*.ts' '*.tsx' \
  | grep -v '/generated/' \
  | grep -v '/target/' \
  | grep -v '/node_modules/' \
  | grep -v '/dist/' || true)

# `--all` also walks the on-disk source tree so the check fires on
# uncommitted files (sandbox + WIP). The dedup is intentional — a file
# in BOTH sets counts once.
if [ "${1:-}" = "--all" ]; then
  while IFS= read -r f; do
    case " ${files[*]:-} " in
      *" $f "*) ;;
      *) files+=("$f") ;;
    esac
  done < <(find . -type f \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' \) \
    | grep -v '/generated/' \
    | grep -v '/target/' \
    | grep -v '/node_modules/' \
    | grep -v '/dist/' \
    | sed 's|^\./||' || true)
fi

fail=0
for f in "${files[@]}"; do
  [ -f "$f" ] || continue
  n=$(wc -l < "$f")
  if [ "$n" -gt "$LIMIT" ]; then
    echo "FILE-LAYOUT: $f is $n lines (limit $LIMIT)"
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  echo "::error::file(s) exceed the ${LIMIT}-line FILE-LAYOUT limit"
  exit 1
fi
echo "FILE-LAYOUT: all source files within ${LIMIT} lines (${#files[@]} checked)"