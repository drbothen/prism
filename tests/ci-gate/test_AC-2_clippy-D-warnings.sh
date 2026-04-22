#!/usr/bin/env bash
# AC-2: clippy must use -D warnings (AD-008 compliance).
# Asserts ci.yml contains a real `cargo clippy -- -D warnings` step (not a TODO echo).
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CI_YML="${WORKTREE}/.github/workflows/ci.yml"

assert_file_exists "$CI_YML" "AC-2"

# Must have real `cargo clippy -- -D warnings` step.
if grep -qE '^\s+run:\s+cargo clippy' "$CI_YML" 2>/dev/null && \
   grep -qF -- '-D warnings' "$CI_YML" 2>/dev/null; then
  tap_pass "AC-2: ci.yml has real 'cargo clippy -- -D warnings' step"
else
  tap_fail "AC-2: ci.yml missing real 'cargo clippy -- -D warnings' step" \
    "AC-2 FAIL: expected .github/workflows/ci.yml to contain 'run: cargo clippy -- -D warnings'; AD-008 requires -D warnings to block merge"
fi

# Confirm it's not just an echo placeholder.
if grep -qE 'echo.*cargo clippy' "$CI_YML" 2>/dev/null; then
  tap_fail "AC-2: clippy step is still an echo stub (AD-008 violation)" \
    "AC-2 FAIL: 'cargo clippy' is inside an echo — stub does not enforce AD-008"
else
  tap_pass "AC-2: clippy step is not an echo stub"
fi

tap_done
