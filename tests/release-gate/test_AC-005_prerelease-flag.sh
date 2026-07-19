#!/usr/bin/env bash
# S-REL-001 AC-005: Prerelease flag applied via bash array pattern.
#
# AC: The --prerelease flag is set via a bash array (args+=(--prerelease)) or equivalent
# parameter-expansion form (${PRERELEASE_FLAG:+--prerelease}), NOT via a quoted-empty
# variable that would send an empty positional arg. For tags NOT containing '-',
# --prerelease is absent. gh does NOT auto-detect prerelease from the tag (research U3).
#
# Red Gate:
#   - Array/expansion prerelease handling is completely absent → FAILS (primary gate)
#   - Tag-contains-dash detection ('*-*') is absent → FAILS
#   - Full composed array form 'PRERELEASE_ARGS+=(--prerelease)' absent → FAILS
#
# Quality gate (passes at Red since forbidden pattern is also absent):
#   - '"$PRERELEASE_FLAG"' as a quoted-empty-variable must not appear → PASS at Red
#
# Traces to: delta-analysis.md §2.1; research U3
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-005"

# 1. Prerelease detection logic must be present: array push or parameter-expansion form.
# After implementation: '+=(--prerelease)' or ':+--prerelease' in the release step.
# At Red: no prerelease handling at all in the file → FAILS.
if grep -qF '+=(--prerelease)' "$REL_YML" 2>/dev/null || \
   grep -qF ':+--prerelease' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-005: prerelease flag uses safe array or \${VAR:+} form"
else
  tap_fail "AC-005: prerelease flag handling absent" \
    "AC-005 FAIL: expected '+=(--prerelease)' or ':+--prerelease' in release.yml — gh does NOT auto-detect prerelease"
fi

# 2. Tag-contains-dash detection pattern must be present.
# After implementation: [[ "\$TAG" == *-* ]] or equivalent '*-*' glob check.
# At Red: absent → FAILS.
if grep -qF '*-*' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-005: tag-contains-dash detection pattern '*-*' present"
else
  tap_fail "AC-005: tag-contains-dash detection pattern absent" \
    "AC-005 FAIL: expected '*-*' glob check for prerelease tag detection in release.yml"
fi

# 3. SID-2: full composed array form 'PRERELEASE_ARGS+=(--prerelease)'.
# Asserts the COMPLETE array push idiom, not just a fragment.
# At Red: absent → FAILS.
if grep -qF 'PRERELEASE_ARGS+=(--prerelease)' "$REL_YML" 2>/dev/null || \
   grep -qF 'args+=(--prerelease)' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-005: full composed prerelease array form present (SID-2)"
else
  tap_fail "AC-005: full composed prerelease array form absent (SID-2)" \
    "AC-005 FAIL: expected 'PRERELEASE_ARGS+=(--prerelease)' or 'args+=(--prerelease)' in release.yml"
fi

# 4. Quality gate: forbidden quoted-empty-variable form must NOT be used.
# '$PRERELEASE_FLAG' passed as a quoted var would send an empty positional arg when unset.
# At Red: also absent (passes at Red). Guards against wrong implementation pattern.
assert_not_contains "$REL_YML" '"$PRERELEASE_FLAG"' "AC-005"

# 5. Idempotent re-run guard: 'gh release view "$TAG"' existence check must be present.
# F-REL001-P10-001 (MED): removing the guard would pass all prior assertions; this is
# the load-bearing assertion that catches deletion of the idempotent guard entirely.
# At Green: present in the Create GitHub Release step.
if grep -qF 'gh release view "$TAG"' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-005: idempotent guard 'gh release view \"\$TAG\"' present (F-REL001-P10-001)"
else
  tap_fail "AC-005: idempotent guard check absent (F-REL001-P10-001)" \
    "AC-005 FAIL: expected 'gh release view \"\$TAG\"' in release.yml — guard deleted means re-runs unconditionally try to create, failing with 'release already exists'"
fi

# 6. Idempotent re-run guard: 'gh release upload "$TAG" --clobber' upload path must be present.
# F-REL001-P10-001: the upload-to-existing path is the whole point of the guard; without it the
# guard check is useless.  Asserts the clobber arm is reachable, not just the view check.
# At Green: present on the then-branch of the guard.
if grep -qF 'gh release upload "$TAG" --clobber' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-005: idempotent guard upload path 'gh release upload \"\$TAG\" --clobber' present (F-REL001-P10-001)"
else
  tap_fail "AC-005: idempotent guard upload path absent (F-REL001-P10-001)" \
    "AC-005 FAIL: expected 'gh release upload \"\$TAG\" --clobber' in release.yml — upload-to-existing arm missing"
fi

# 7. SID-2 composed-output assertion: the create path must splice '"${PRERELEASE_ARGS[@]}"'.
# Assertions 2-4 above verify the array is correctly POPULATED; this assertion verifies it is
# actually PASSED to 'gh release create'.  An implementation that populates the array but
# forgets to splice it silently drops --prerelease on new releases.
# F-REL001-P10-001.
if grep -qF '"${PRERELEASE_ARGS[@]}"' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-005: create path splices '\"\${PRERELEASE_ARGS[@]}\"' into gh release create (SID-2 / F-REL001-P10-001)"
else
  tap_fail "AC-005: create path missing '\"\${PRERELEASE_ARGS[@]}\"' splice (SID-2 / F-REL001-P10-001)" \
    "AC-005 FAIL: expected '\"\${PRERELEASE_ARGS[@]}\"' in release.yml — array populated but never passed to gh release create, --prerelease silently dropped"
fi

tap_done
