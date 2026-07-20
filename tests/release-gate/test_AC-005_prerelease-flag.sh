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

# ====================================================================
# CWE-78 injection regression guard — F-REL001-P1-001 / F-REL001-P18-001
#
# F-REL001-P1-001 fix: ref/event-derived GitHub context values
# (github.ref_name, github.event.*, github.head_ref) and env: re-exposure
# values (env.ARCHIVE) are bound via an explicit env: map and accessed
# inside run: scripts as plain shell variables ($TAG, $ARCHIVE).  They
# must NEVER appear as ${{ }} textual substitutions inside run: script
# bodies.  The GitHub Actions runner performs ${{ }} substitution BEFORE
# passing the script text to bash; an attacker-controlled tag like
# v1.0.0$(id) would therefore be interpolated into the bash source code
# before bash sees it — the classic CWE-78 OS command injection path.
#
# F-REL001-P18-001 (MED): S-REL-003/004 will add more run: blocks;
# without this regression guard the forbidden pattern could be silently
# reintroduced.  This guard catches it mechanically on every suite run.
#
# Allowlist — correctly used inside run: bodies, not an injection risk:
#   ${{ matrix.target }}, ${{ matrix.archive_ext }} — workflow-controlled
#   matrix values defined in the jobs.*.strategy.matrix block, not
#   reachable by an external attacker.
#   ${{ secrets.* }} — repo-controlled, never set from user input.
#   ${{ runner.* }} — runner-controlled metadata.
#
# Design: POSIX awk state machine extracts the text of every run: block
# (both block-scalar "run: |" and inline "run: command" forms).  Four
# separate negative assertions then check the extracted text for each
# forbidden expression class.  A preflight assertion verifies the awk
# produced non-empty output so a broken awk cannot create a false pass
# on the negative checks.
# ====================================================================

# Extract all run: block bodies using a POSIX awk state machine.
# Algorithm:
#   - Detect "run:" key (followed by a space) at any indentation.
#   - Record the indent depth of the "run:" key as run_indent.
#   - For block-scalar forms ("run: |"), subsequent lines at strictly
#     greater indent are the body; blank lines are retained.
#   - For inline forms ("run: command"), the rest-of-line after "run: "
#     is the body (one line).
#   - Exit capture when a non-blank line at indent <= run_indent is seen.
# Verified against release.yml: produces exactly 63 lines covering all
# 9 run: blocks; all ${{ }} expressions in that output are matrix.* only.
run_blocks=$(awk '
  BEGIN { cap = 0; rind = 0 }
  {
    match($0, /^[[:space:]]*/); ci = RLENGTH
    s = substr($0, ci + 1)
    if (cap) {
      if (s != "" && ci <= rind) { cap = 0 }
      else { print; next }
    }
    if (!cap && s ~ /^run:[[:space:]]/) {
      cap = 1; rind = ci
      r = substr(s, 5)
      match(r, /^[[:space:]]*/); r = substr(r, RLENGTH + 1)
      if (r != "" && r !~ /^[|>]/) print r
    }
  }
' "$REL_YML")

# 8. Preflight: awk must produce non-empty output.
# An empty result would silently false-pass assertions 9-12 (negative
# checks against empty text always succeed).
if [ -n "$run_blocks" ]; then
  tap_pass "AC-005: run: block extraction non-empty (awk state-machine preflight)"
else
  tap_fail "AC-005: run: block extraction empty — awk state-machine broken" \
    "AC-005 FAIL: awk produced no output from release.yml run: blocks — assertions 9-12 would false-pass; fix awk"
fi

# 9. No ${{ github.ref* }} in run: blocks.
# Covers github.ref, github.ref_name, github.ref_type.
# Correct pattern: bind via env: map (env: TAG: ${{ github.ref_name }}) and
# use plain $TAG in run: bodies.  CWE-78 / F-REL001-P1-001 / F-REL001-P18-001.
if echo "$run_blocks" | grep -qF '${{ github.ref' 2>/dev/null; then
  tap_fail "AC-005: forbidden \${{ github.ref* }} in run: block (CWE-78 / F-REL001-P1-001 regression)" \
    "AC-005 FAIL: '\${{ github.ref' must not appear in run: script bodies — bind via env: map and use plain \$TAG (F-REL001-P1-001 / F-REL001-P18-001 / CWE-78)"
else
  tap_pass "AC-005: \${{ github.ref* }} absent from all run: blocks (F-REL001-P1-001 / F-REL001-P18-001)"
fi

# 10. No ${{ github.event* }} in run: blocks.
# github.event.* values (PR body, commit message, etc.) are fully
# attacker-controlled via PR creation or commit authorship.
# CWE-78 / F-REL001-P1-001 / F-REL001-P18-001.
if echo "$run_blocks" | grep -qF '${{ github.event' 2>/dev/null; then
  tap_fail "AC-005: forbidden \${{ github.event* }} in run: block (CWE-78 / F-REL001-P1-001 regression)" \
    "AC-005 FAIL: '\${{ github.event' must not appear in run: script bodies — event-derived values are attacker-controlled via PR/commit"
else
  tap_pass "AC-005: \${{ github.event* }} absent from all run: blocks (F-REL001-P1-001 / F-REL001-P18-001)"
fi

# 11. No ${{ github.head_ref }} in run: blocks.
# head_ref is the PR source branch name — attacker-controlled when a PR
# is opened from a fork with an arbitrary branch name.
# CWE-78 / F-REL001-P1-001 / F-REL001-P18-001.
if echo "$run_blocks" | grep -qF '${{ github.head_ref' 2>/dev/null; then
  tap_fail "AC-005: forbidden \${{ github.head_ref }} in run: block (CWE-78 / F-REL001-P1-001 regression)" \
    "AC-005 FAIL: '\${{ github.head_ref' must not appear in run: script bodies — PR source branch name is attacker-controlled"
else
  tap_pass "AC-005: \${{ github.head_ref }} absent from all run: blocks (F-REL001-P1-001 / F-REL001-P18-001)"
fi

# 12. No ${{ env.* }} in run: blocks (env re-exposure vector).
# If an env var (e.g. ARCHIVE) was transitively set from a ref-derived
# expression, using ${{ env.ARCHIVE }} inside run: re-opens CWE-78:
# the runner substitutes the env value textually into bash source before
# bash sees it.  Correct pattern: use the plain shell variable form
# ($ARCHIVE, $TAG) which bash receives as an already-resolved string.
# The ${{ env.ARCHIVE }} form is allowed in with:/env: keys (not run:).
# F-REL001-P1-001 / F-REL001-P18-001 / CWE-78.
if echo "$run_blocks" | grep -qF '${{ env.' 2>/dev/null; then
  tap_fail "AC-005: forbidden \${{ env.* }} in run: block (env re-exposure / F-REL001-P1-001 regression)" \
    "AC-005 FAIL: '\${{ env.' must not appear in run: script bodies — use plain shell var (\$ARCHIVE, \$TAG) not \${{ env.VAR }} (F-REL001-P1-001 / F-REL001-P18-001 / CWE-78)"
else
  tap_pass "AC-005: \${{ env.* }} absent from all run: blocks (F-REL001-P1-001 / F-REL001-P18-001 env-re-exposure vector)"
fi

tap_done
