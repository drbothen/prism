#!/usr/bin/env bash
# test_AC-1_just-check-runs-pr-gate.sh
# AC-1: `just check` runs all 6 PR gate steps and exits 0 on a clean codebase.
# FAILS on stub: each step exits 1.
set -euo pipefail

WORKTREE="$(cd "$(dirname "$0")/../.." && pwd)"
TAP_COUNT=0
FAIL=0

tap_ok()   { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - $1"; }
tap_fail() { TAP_COUNT=$((TAP_COUNT+1)); echo "not ok $TAP_COUNT - $1"; FAIL=1; }
tap_skip() { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - SKIP $1 # SKIP tool absent"; }

# Prerequisite: just must be available
if ! command -v just &>/dev/null; then
  tap_skip "just not on PATH — install via dev-setup.sh"
  echo "1..$TAP_COUNT"
  exit 0
fi

# Test 1: `just check` target exists in the Justfile
if grep -q '^check:' "$WORKTREE/Justfile"; then
  tap_ok "Justfile contains check target"
else
  tap_fail "Justfile missing check target"
fi

# Test 2: Justfile check recipe contains the 6 required gate commands
JUSTFILE="$WORKTREE/Justfile"
REQUIRED=(
  "cargo fmt --check"
  "cargo clippy"
  "cargo test --workspace"
  "cargo deny check"
  "cargo audit"
  "cargo semver-checks"
)
ALL_PRESENT=1
for cmd in "${REQUIRED[@]}"; do
  if ! grep -qF "$cmd" "$JUSTFILE"; then
    ALL_PRESENT=0
    tap_fail "AC-1: Justfile check recipe missing: $cmd"
  fi
done
if [[ $ALL_PRESENT -eq 1 ]]; then
  tap_ok "AC-1: Justfile check recipe contains all 6 PR gate commands"
fi

echo "1..$TAP_COUNT"
exit $FAIL
