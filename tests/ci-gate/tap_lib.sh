#!/usr/bin/env bash
# TAP helper library — sourced by each test file.
# Compatible with bash 3.2+ (macOS default).

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
  if [ -n "$2" ]; then
    echo "#   FAIL detail: $2"
  fi
}

tap_skip() {
  _TAP_COUNT=$((_TAP_COUNT + 1))
  echo "ok ${_TAP_COUNT} - $1 # SKIP $2"
}

tap_done() {
  echo "1..${_TAP_COUNT}"
  return ${_TAP_FAILURES}
}

# Require a command is available; skip with message if not.
require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "# SKIP: required tool '${cmd}' not found in PATH"
    return 1
  fi
  return 0
}

# Assert that a file contains a literal string.
# Usage: assert_contains FILE NEEDLE AC_ID
assert_contains() {
  local file="$1"
  local needle="$2"
  local ac_id="$3"
  if grep -qF "$needle" "$file" 2>/dev/null; then
    tap_pass "${ac_id}: '${needle}' found in ${file##*/}"
  else
    tap_fail "${ac_id}: '${needle}' NOT found in ${file##*/}" \
      "${ac_id} FAIL: expected '${needle}' in ${file} — found only TODO placeholder"
  fi
}

# Assert a string is NOT present in a file (secret-hardcoding guard).
assert_not_contains() {
  local file="$1"
  local needle="$2"
  local ac_id="$3"
  if grep -qF "$needle" "$file" 2>/dev/null; then
    tap_fail "${ac_id}: hardcoded value '${needle}' found in ${file##*/}" \
      "${ac_id} FAIL: '${needle}' must not be hardcoded — use secrets.VARNAME instead"
  else
    tap_pass "${ac_id}: '${needle}' correctly absent (not hardcoded) in ${file##*/}"
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
