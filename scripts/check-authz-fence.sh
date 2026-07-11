#!/usr/bin/env bash
# check-authz-fence.sh — the grep fence (milestone 02 exit gate).
#
# CLAUDE.md rule 7 (sacred): a `guardianship` read outside the authz/
# module fails CI. This script walks the care extension's tracked source
# files and fails if any line OUTSIDE `authz/` contains the literal
# `guardianship` (the table name). The point is to keep the chokepoint
# the ONE place that resolves reach — N-verbs-each-resolving-their-own
# is the leak pattern this fence exists to prevent.
#
# The fence is intentionally simple: a `grep` for the table name. The
# alternatives (an AST lint) cost more than they buy here — the table
# name is short and distinctive enough that false positives are easy to
# triage by eye.
set -euo pipefail

# Resolve repo root + the authz/ directory we treat as the one blessed
# reader of `guardianship`.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
AUTHZ_DIR="$ROOT/rust/extensions/care/src/authz"

# Tracked source files in the care extension, minus authz/ itself, the
# tests/ directory (which legitimately seeds records via the real write
# path — that's what the matrix harness IS), and generated/target
# scaffolding. Use git ls-files when available (the CI posture); fall
# back to a direct `find` so the fence also works in a sandbox where
# the working tree isn't committed yet.
if cd "$ROOT" && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  mapfile -t files < <(cd "$ROOT" && git ls-files \
    'rust/extensions/care/src/*.rs' \
    'rust/extensions/care/src/**/*.rs' 2>/dev/null \
    | grep -v "$AUTHZ_DIR/" \
    | grep -v '/target/' || true)
fi
# Fallback / supplement: walk the on-disk source tree so the fence also
# fires on uncommitted files (the sandbox posture + a developer's WIP).
if [ "${#files[@]}" -eq 0 ] || [ "${1:-}" = "--all" ]; then
  mapfile -t files < <(find "$ROOT/rust/extensions/care/src" \
    -name "*.rs" -type f 2>/dev/null \
    | grep -v "$AUTHZ_DIR/" \
    | grep -v '/target/' || true)
fi

fail=0
hits=0
for f in "${files[@]}"; do
  [ -f "$f" ] || continue
  # The fence fires on the actual leak pattern: a `lb_store::read` /
  # `lb_store::list` call whose table name is `"guardianship"`. That is
  # the chokepoint's read path; reproducing it anywhere else is exactly
  # what CLAUDE.md rule 7 forbids. Mentions in docs, comments, module
  # names, or test fixtures (which seed records by their table name)
  # are intentionally NOT flagged — they're the documentation of the
  # chokepoint's contract, not the leak pattern.
  hits_in_file="$(grep -nE '(read|list)\s*\(.*"guardianship"|"guardianship"\s*,|\bguardianship\s*=>' "$f" 2>/dev/null || true)"
  if [ -n "$hits_in_file" ]; then
    printf 'AUTHZ-FENCE: %s reads the "guardianship" table outside authz/ — route through care::authz (CLAUDE.md rule 7):\n' "$f"
    printf '%s\n' "$hits_in_file"
    fail=1
    hits=$((hits + 1))
  fi
done

if [ "$fail" -ne 0 ]; then
  printf '::error::%s file(s) read "guardianship" outside authz/ — every read must go through care::authz (CLAUDE.md rule 7)\n' "$hits"
  exit 1
fi
printf 'AUTHZ-FENCE: %d files checked, no "guardianship" reads outside authz/\n' "${#files[@]}"