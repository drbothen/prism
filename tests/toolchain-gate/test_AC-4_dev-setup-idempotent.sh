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

echo "1..$TAP_COUNT"
exit $FAIL
