#!/usr/bin/env bash
# AC-1: PR gate fails on cargo fmt --check.
# Asserts ci.yml contains a real `cargo fmt --check` run step (not a TODO echo).
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=tap_lib.sh
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CI_YML="${WORKTREE}/.github/workflows/ci.yml"

assert_file_exists "$CI_YML" "AC-1"

# Must have a real `cargo fmt --check` run step, not a TODO echo.
if grep -qE '^\s+run:\s+cargo fmt --check' "$CI_YML" 2>/dev/null; then
  tap_pass "AC-1: ci.yml has real 'run: cargo fmt --check' step"
else
  tap_fail "AC-1: ci.yml missing real 'cargo fmt --check' step" \
    "AC-1 FAIL: expected .github/workflows/ci.yml to contain 'run: cargo fmt --check'; got only TODO echo placeholder"
fi

# Step must NOT be wrapped in an echo (confirms it's real, not a stub comment).
if grep -qE 'echo.*cargo fmt' "$CI_YML" 2>/dev/null; then
  tap_fail "AC-1: cargo fmt step is still an echo stub" \
    "AC-1 FAIL: 'cargo fmt --check' is inside an echo — this is the TODO stub, not a real step"
else
  tap_pass "AC-1: cargo fmt step is not an echo stub"
fi

tap_done
