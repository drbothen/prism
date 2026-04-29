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
  if [ -n "${2:-}" ]; then
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
