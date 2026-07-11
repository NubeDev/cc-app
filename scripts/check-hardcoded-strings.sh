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
#
# HARD as of milestone 03: the `crate::i18n::t` catalog helper landed and the
# shipped verbs' user-facing strings flow through it, so this gate is now
# `exit 1` (was a warning during the bootstrap). Developer-facing error
# context and the i18n engine are exempt (ERROR_CONTEXT + the i18n/ exclusion)
# so the gate is precise enough to block the build on a real violation only.
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

# The distinction this lint enforces (CLAUDE.md rule 8): a USER-FACING
# string — a sentence that reaches a guardian's screen — must flow through
# the i18n catalog (`crate::i18n::t`). A DEVELOPER-FACING string — the
# `Result<_, String>` error CONTEXT a verb returns, which the host maps to
# an OPAQUE `ToolError` and NEVER shows a user verbatim (see
# `child/get.rs`: "the host maps to ToolError::Denied") — is exempt, the
# same way `authz/` deny-reason audit keys are exempt. The two are told
# apart by CONTEXT, not by the words: an error-context `format!` lives
# inside `map_err` / `ok_or_else` / `Err(format!(…))` / `.expect(` or is a
# JSON (de)serialization diagnostic. Those are the plumbing; a raw sentence
# ANYWHERE ELSE (e.g. assigned to a reply `message`, or in a bare
# `format!(...)` produced as output) is the violation this gate blocks.
# Error-context / plumbing markers (developer-facing, host-opaque) and the
# i18n machinery itself. A line matching ANY of these is exempt:
#   - `map_err`/`ok_or_else`/`.expect(`/`Err(format!` — Result error context.
#   - serialize/deserialize/`input:` — JSON (de)serialization diagnostics.
#   - the typed error variants (`StoreDenied`/`MissingField`/…/`Failed(`) —
#     their Display impls live in `records.rs`, the audit-key analog.
#   - `t(locale` / `format!("{{...` / dotted-key `format!("[a-z]+\.` — this
#     is i18n LOOKUP (building a catalog KEY) or the interpolation engine,
#     the OPPOSITE of a hardcoded user string.
#   - raw-string JSON literals (`r#"{`) — wire payloads / test fixtures.
ERROR_CONTEXT='(map_err|ok_or_else|\.expect\(|Err\(format!|serialize|deserialize|input:|StoreDenied|MissingField|InvalidId|Failed\(|t\(locale|format!\("\{\{|format!\("[a-z_]+\.|r#"\{)'

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
      # i18n/ IS the catalog engine (the `t()` resolver + its `{{var}}`
      # interpolation machinery) — the one place `format!` legitimately
      # manipulates catalog strings. Exempt like authz/'s audit keys.
      */i18n/*) continue ;;
      */target/*) continue ;;
    esac
    # Flag lines that contain a user-facing macro AND a string literal with
    # alphabetic content, EXCLUDING developer-facing error-context /
    # (de)serialization plumbing (see ERROR_CONTEXT above). A genuine
    # user-facing sentence lives outside that context and must go through
    # `crate::i18n::t`. `grep -v` strips the exempt plumbing lines before we
    # judge — so the gate is precise enough to run hard.
    matches="$(grep -nE "${MACROS}.*${STRING_LITERAL}" "$f" 2>/dev/null \
      | grep -vE "${ERROR_CONTEXT}" || true)"
    if [ -n "$matches" ]; then
      printf 'HARDCODE: %s embeds a raw user-facing string in a macro — route through i18n/<locale>.json (CLAUDE.md rule 8):\n' "$f"
      printf '%s\n' "$matches"
      fail=1
      hits=$((hits + 1))
    fi
  done < <(find "$ROOT/$surface" -type f -name "*.rs" 2>/dev/null)
done

if [ "$fail" -ne 0 ]; then
  # HARD gate as of milestone 03: the catalog wire-up landed (`crate::i18n::t`
  # resolves every user-facing string from `i18n/{en,es}.json`), so a raw
  # user-facing literal is now a build failure, not a warning. The lint is
  # scoped to genuine chrome (developer-facing error context + the i18n engine
  # are exempt — see ERROR_CONTEXT and the i18n/ path exclusion above), so a
  # hit is a real rule-8 violation. Route the string through `t()`.
  printf '::error::%s file(s) embed a raw user-facing string in a macro — route through crate::i18n::t / i18n/<locale>.json (CLAUDE.md rule 8)\n' "$hits"
  exit 1
fi
printf 'HARDCODE: no raw user-facing strings in macros (production source; see scripts/check-i18n-parity.sh for the catalog gate)\n'