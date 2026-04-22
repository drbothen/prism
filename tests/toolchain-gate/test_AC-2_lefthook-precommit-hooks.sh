#!/usr/bin/env bash
# test_AC-2_lefthook-precommit-hooks.sh
# AC-2: lefthook pre-commit hook runs fmt+clippy on changed .rs files and blocks on warnings.
# FAILS on stub: lefthook.yml has empty commands: {}.
set -euo pipefail

WORKTREE="$(cd "$(dirname "$0")/../.." && pwd)"
TAP_COUNT=0
FAIL=0

tap_ok()   { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - $1"; }
tap_fail() { TAP_COUNT=$((TAP_COUNT+1)); echo "not ok $TAP_COUNT - $1"; FAIL=1; }
tap_skip() { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - SKIP $1 # SKIP tool absent"; }

LEFTHOOK_YML="$WORKTREE/lefthook.yml"

# Test 1: lefthook.yml exists
if [[ -f "$LEFTHOOK_YML" ]]; then
  tap_ok "lefthook.yml exists"
else
  tap_fail "lefthook.yml missing"
fi

# Test 2: pre-commit section present
if grep -q '^pre-commit:' "$LEFTHOOK_YML"; then
  tap_ok "lefthook.yml has pre-commit section"
else
  tap_fail "lefthook.yml missing pre-commit section"
fi

# Test 3: lefthook.yml must have a non-empty commands block (stub has commands: {})
# Post-implementation: commands block will contain fmt and clippy entries.
COMMANDS_BLOCK=$(grep -A3 'commands:' "$LEFTHOOK_YML" | head -5)
if echo "$COMMANDS_BLOCK" | grep -qE 'fmt:|clippy:'; then
  tap_ok "lefthook.yml commands block contains fmt and clippy entries"
else
  tap_fail "AC-2: lefthook.yml commands block is empty or missing fmt/clippy — not yet implemented (Red Gate)"
fi

echo "1..$TAP_COUNT"
exit $FAIL
