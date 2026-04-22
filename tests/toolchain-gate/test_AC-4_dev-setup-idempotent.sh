#!/usr/bin/env bash
# test_AC-4_dev-setup-idempotent.sh
# AC-4: dev-setup.sh is idempotent — running it a second time exits 0 without errors.
# FAILS on stub: stub exits 1 on first invocation, so idempotency is also broken.
set -euo pipefail

WORKTREE="$(cd "$(dirname "$0")/../.." && pwd)"
SETUP="$WORKTREE/scripts/dev-setup.sh"
TAP_COUNT=0
FAIL=0

tap_ok()   { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - $1"; }
tap_fail() { TAP_COUNT=$((TAP_COUNT+1)); echo "not ok $TAP_COUNT - $1"; FAIL=1; }

# Test 1: script uses existence checks before reinstalling (cargo install --locked pattern or similar)
# This verifies idempotency intent is coded, not just that it runs once.
if grep -qE 'command -v|which |cargo install.*--locked|if .*installed' "$SETUP"; then
  tap_ok "AC-4: dev-setup.sh contains existence/idempotency checks"
else
  tap_fail "AC-4: dev-setup.sh lacks existence checks for installed tools (idempotency not implemented)"
fi

# Test 2: stub exits 1 on first run — confirms idempotency not yet working
if bash "$SETUP" &>/dev/null; then
  tap_fail "AC-4: stub exited 0 on first run — must FAIL before implementation (Red Gate)"
else
  tap_ok "AC-4: stub fails on first run as expected (Red Gate)"
fi

# Test 3: if stub exits 1 on first run, idempotency is trivially broken too
# Real test: run twice, both must exit 0 (cannot verify until implementation)
# We assert here that the stub does NOT satisfy this behavior.
FIRST_RUN_EXIT=0
bash "$SETUP" &>/dev/null && FIRST_RUN_EXIT=0 || FIRST_RUN_EXIT=$?
SECOND_RUN_EXIT=0
bash "$SETUP" &>/dev/null && SECOND_RUN_EXIT=0 || SECOND_RUN_EXIT=$?

if [[ $FIRST_RUN_EXIT -eq 0 && $SECOND_RUN_EXIT -eq 0 ]]; then
  tap_fail "AC-4: both runs exited 0 on stub — must FAIL before implementation (Red Gate)"
else
  tap_ok "AC-4: stub is not idempotent-passing (Red Gate verified)"
fi

echo "1..$TAP_COUNT"
exit $FAIL
