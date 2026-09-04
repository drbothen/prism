#!/usr/bin/env bash
# TAP helper library — sourced by each test file.
# Compatible with bash 3.2+ (macOS default).
# Extended from tests/ci-gate/tap_lib.sh with helpers for functional-line assertions.

_TAP_COUNT=0
_TAP_FAILURES=0

tap_pass() {
  _TAP_COUNT=$((_TAP_COUNT + 1))
  echo "ok ${_TAP_COUNT} - $1"
}

tap_fail() {
  _TAP_COUNT=$((_TAP_COUNT + 1))
  _TAP_FAILURES=$((_TAP_FAILURES + 1))
  echo "not ok ${_TAP_COUNT} - $1"
  if [ -n "${2:-}" ]; then
    echo "#   FAIL detail: $2"
  fi
}

tap_skip() {
  _TAP_COUNT=$((_TAP_COUNT + 1))
  echo "ok ${_TAP_COUNT} - $1 # SKIP ${2:-}"
}

tap_done() {
  echo "1..${_TAP_COUNT}"
  return ${_TAP_FAILURES}
}

# Assert that a file contains a literal string.
# Usage: assert_contains FILE NEEDLE AC_ID
assert_contains() {
  local file="$1"
  local needle="$2"
  local ac_id="$3"
  if grep -qF -- "$needle" "$file" 2>/dev/null; then
    tap_pass "${ac_id}: '${needle}' found in ${file##*/}"
  else
    tap_fail "${ac_id}: '${needle}' NOT found in ${file##*/}" \
      "${ac_id} FAIL: expected '${needle}' in ${file} — not present"
  fi
}

# Assert a string is NOT present in a file.
assert_not_contains() {
  local file="$1"
  local needle="$2"
  local ac_id="$3"
  if grep -qF -- "$needle" "$file" 2>/dev/null; then
    tap_fail "${ac_id}: forbidden '${needle}' found in ${file##*/}" \
      "${ac_id} FAIL: '${needle}' must not be present — found in ${file}"
  else
    tap_pass "${ac_id}: '${needle}' correctly absent from ${file##*/}"
  fi
}

# Assert file exists.
assert_file_exists() {
  local file="$1"
  local ac_id="$2"
  if [ -f "$file" ]; then
    tap_pass "${ac_id}: file exists: ${file##*/}"
  else
    tap_fail "${ac_id}: file missing: ${file}" \
      "${ac_id} FAIL: expected file ${file} to exist"
  fi
}

# Assert that a literal string does NOT appear in functional (non-comment) lines.
# Comment lines are those where the first non-whitespace character is '#'.
# Useful for "comment-only references acceptable" assertions in AC-002..AC-004.
# N5-hardened: fails explicitly when the target file is absent (fail-closed).
# Usage: assert_not_in_functional_lines FILE NEEDLE AC_ID
assert_not_in_functional_lines() {
  local file="$1"
  local needle="$2"
  local ac_id="$3"
  # N5: fail-closed — a missing file is an error, not a vacuous pass.
  if [ ! -f "$file" ]; then
    tap_fail "${ac_id}: cannot check functional lines — file absent: ${file##*/}" \
      "${ac_id} FAIL: ${file} must exist for assert_not_in_functional_lines to be non-vacuous"
    return
  fi
  local functional_content
  functional_content=$(grep -v '^[[:space:]]*#' "$file" 2>/dev/null) || true
  if echo "$functional_content" | grep -qF -- "$needle" 2>/dev/null; then
    tap_fail "${ac_id}: '${needle}' found in functional lines of ${file##*/}" \
      "${ac_id} FAIL: '${needle}' must not appear in functional (non-comment) lines of ${file}"
  else
    tap_pass "${ac_id}: '${needle}' correctly absent from functional lines of ${file##*/}"
  fi
}

# Assert that a regex pattern does NOT appear in functional (non-comment) lines.
# N5-hardened: fails explicitly when the target file is absent (fail-closed).
# Usage: assert_not_in_functional_lines_re FILE PATTERN AC_ID
assert_not_in_functional_lines_re() {
  local file="$1"
  local pattern="$2"
  local ac_id="$3"
  # N5: fail-closed — a missing file is an error, not a vacuous pass.
  if [ ! -f "$file" ]; then
    tap_fail "${ac_id}: cannot check functional lines — file absent: ${file##*/}" \
      "${ac_id} FAIL: ${file} must exist for assert_not_in_functional_lines_re to be non-vacuous"
    return
  fi
  local functional_content
  functional_content=$(grep -v '^[[:space:]]*#' "$file" 2>/dev/null) || true
  if echo "$functional_content" | grep -qE "$pattern" 2>/dev/null; then
    tap_fail "${ac_id}: pattern '${pattern}' found in functional lines of ${file##*/}" \
      "${ac_id} FAIL: pattern '${pattern}' must not appear in functional (non-comment) lines"
  else
    tap_pass "${ac_id}: pattern '${pattern}' correctly absent from functional lines of ${file##*/}"
  fi
}
