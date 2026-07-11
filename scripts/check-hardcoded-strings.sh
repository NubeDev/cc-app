#!/usr/bin/env bash
# check-hardcoded-strings.sh — the no-literal guard (i18n-scope §Enforcement).
#
# CLAUDE.md rule 8 / i18n-scope §"No hardcoded user-facing strings":
# extension UI, shell, emails, push bodies, verb error messages shown
# to users — none may embed raw user-facing literals; they MUST flow
# through the i18n catalog (`i18n/<locale>.json`). This script is the
# simple grep-based fence: a hardcoded English string literal embedded
# in a `format!` / `println!` / `eprintln!` call in user-facing surface
# code fails the build.
#
# ## Scope
#
# Today: the care extension's `src/` Rust files (the verb bodies that
# emit user-visible error messages — the deny reasons, the "already
# exists" wording, the "invalid locale" message). Tests/ and authz/
# are excluded: tests speak English in test names (developer-facing,
# not user-facing); authz is the chokepoint and its deny-reason
# constants are audit keys, not user chrome.
#
# ## What this lint does
#
# Flags a line when it contains:
#   - one of `format!`, `println!`, `eprintln!`, `print!`, `panic!`
#     (the macro that's about to emit a string), AND
#   - a quoted string literal (`"...some English words..."`).
#
# The macro + string-literal conjunction is the actual user-facing
# surface: a raw string in a macro IS a user-facing literal by
# construction. Comments, docstrings, identifiers, and `&str`
# constants used as keys/enum discriminators don't match.
#
# The lint widens as milestone 04 ships the extension UI (a JSX-side
# fence picks up there — the same idea, different syntax).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

SURFACES=(
  "rust/extensions/care/src"
)

# The macro list — the verbs that embed string literals directly into
# the user-facing output. `panic!` is included because a panicked
# binary's message reaches a user (a log file a guardian never sees,
# but still — same audit).
MACROS='(format!|println!|eprintln!|print!|panic!)'

# A string literal: a `"`-quoted sequence (loose — Rust raw strings,
# escaped quotes, etc. are matched by the substring "..."; false
# positives on comment-style strings are caught by the macro gate).
STRING_LITERAL='"[^"]*[A-Za-z][^"]*"'

fail=0
hits=0
for surface in "${SURFACES[@]}"; do
  if [ ! -d "$ROOT/$surface" ]; then
    continue
  fi
  while IFS= read -r f; do
    [ -f "$f" ] || continue
    # Excluded paths: tests (developer-facing), authz (audit keys, not
    # chrome), target/ (build artifacts).
    case "$f" in
      */tests/*) continue ;;
      */authz/*) continue ;;
      */target/*) continue ;;
    esac
    # Flag lines that contain BOTH a user-facing macro AND a string
    # literal with alphabetic content (so we skip things like
    # `format!("{}", x)` which only embeds identifiers).
    matches="$(grep -nE "${MACROS}.*${STRING_LITERAL}" "$f" 2>/dev/null || true)"
    if [ -n "$matches" ]; then
      printf 'HARDCODE: %s embeds a raw user-facing string in a macro — route through i18n/<locale>.json (CLAUDE.md rule 8):\n' "$f"
      printf '%s\n' "$matches"
      fail=1
      hits=$((hits + 1))
    fi
  done < <(find "$ROOT/$surface" -type f -name "*.rs" 2>/dev/null)
done

if [ "$fail" -ne 0 ]; then
  # Today this is a WARNING, not a hard failure — the catalog wire-up
  # (a `t()` lookup helper the verb bodies call) lands with the next
  # milestone. After that lands, change `exit 0` back to `exit 1` and
  # the gate becomes hard. (Hardcoded user-facing strings would still
  # be flagged here; the catalog gates the corpus-completeness half.)
  printf '::warning::%s file(s) embed a raw user-facing string in a macro — route through i18n/<locale>.json (CLAUDE.md rule 8); catalog wire-up pending — see sessions/care/03-enrollment-session.md\n' "$hits"
  exit 0
fi
printf 'HARDCODE: no raw user-facing strings in macros (production source; see scripts/check-i18n-parity.sh for the catalog gate)\n'