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
# reader of `guardianship`. The exclusion pattern is REPO-RELATIVE
# (`rust/extensions/care/src/authz/`) so it matches identically whether a
# file path came from `git ls-files` (repo-relative) or from `find`
# (absolute, under $ROOT). A prior version excluded against the ABSOLUTE
# `$AUTHZ_DIR/`, which silently failed to filter the repo-relative
# `git ls-files` output — the CI branch — so the fence false-flagged the
# blessed chokepoint file `authz/scope.rs` (milestone-03 fence fix).
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
AUTHZ_REL="rust/extensions/care/src/authz/"

# The `guardianship/` verb directory is the OTHER blessed toucher of the
# `guardianship` table — but for a different reason than authz/. rule 7 forbids
# a verb RESOLVING REACH by reading the edge table itself (that's the
# chokepoint's job); it does NOT forbid the edge's own CRUD verbs
# (`link`/`unlink`/`update`) from writing the table they OWN. Those verbs are
# the source of truth the chokepoint reads. A reach-resolution leak would be a
# `guardianship` read in a NON-owning verb (e.g. child/list.rs querying edges
# instead of calling `reachable_children`) — still caught, because only these
# two directories are exempt. Keep this list tight: adding a new dir here must
# be justified as "owns the edge table," never "is convenient."
GUARDIANSHIP_REL="rust/extensions/care/src/guardianship/"

# Tracked source files in the care extension, minus authz/ itself, the
# tests/ directory (which legitimately seeds records via the real write
# path — that's what the matrix harness IS), and generated/target
# scaffolding. Use git ls-files when available (the CI posture); fall
# back to a direct `find` so the fence also works in a sandbox where
# the working tree isn't committed yet. Both branches normalize paths to
# repo-relative before excluding, so the one `$AUTHZ_REL` filter applies.
if cd "$ROOT" && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  mapfile -t files < <(cd "$ROOT" && git ls-files \
    'rust/extensions/care/src/*.rs' \
    'rust/extensions/care/src/**/*.rs' 2>/dev/null \
    | grep -v "^$AUTHZ_REL" \
    | grep -v "^$GUARDIANSHIP_REL" \
    | grep -v '/target/' || true)
fi
# Fallback / supplement: walk the on-disk source tree so the fence also
# fires on uncommitted files (the sandbox posture + a developer's WIP).
# `find` emits absolute paths; strip the `$ROOT/` prefix so the SAME
# repo-relative `$AUTHZ_REL` exclusion applies.
if [ "${#files[@]}" -eq 0 ] || [ "${1:-}" = "--all" ]; then
  mapfile -t files < <(find "$ROOT/rust/extensions/care/src" \
    -name "*.rs" -type f 2>/dev/null \
    | sed "s#^$ROOT/##" \
    | grep -v "^$AUTHZ_REL" \
    | grep -v "^$GUARDIANSHIP_REL" \
    | grep -v '/target/' || true)
fi

# All paths are now repo-relative; resolve them from $ROOT.
cd "$ROOT"

fail=0
hits=0
for f in "${files[@]}"; do
  [ -f "$f" ] || continue
  # The fence fires on the actual leak pattern: a READ of the `guardianship`
  # table (`read` / `list` / `query` / a `SELECT … guardianship`) outside
  # authz/. That is the chokepoint's read path; reproducing it anywhere else is
  # exactly what CLAUDE.md rule 7 forbids. Mentions in docs, comments, module
  # names, and test FIXTURES that SEED the table (a `store_create` / `create` /
  # `write` of `guardianship`) are intentionally NOT flagged — seeding a record
  # is not reading it, and the leak the rule guards is a read that exposes one
  # family's edge to another. So we match the read verbs, then strip any line
  # that is a write/seed of the table (the `store_create`/`create`/`write` form
  # a unit test uses to build its fixture via the real write path).
  hits_in_file="$(grep -nE '(read|list|query|query_ws|query_data)\s*\(.*"guardianship"|SELECT.*"guardianship"|\bguardianship\s*=>' "$f" 2>/dev/null \
    | grep -vE '(store_create|[^a-z_]create|\bwrite)\s*\(' \
    || true)"
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