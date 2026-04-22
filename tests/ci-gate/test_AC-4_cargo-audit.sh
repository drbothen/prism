#!/usr/bin/env bash
# AC-4: cargo audit and cargo deny check must be real steps in ci.yml.
# Also checks step ORDER matches tooling-selection.md: fmt->clippy->test->deny->audit->semver-checks.
# requires: bash 3.2+

set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CI_YML="${WORKTREE}/.github/workflows/ci.yml"

assert_file_exists "$CI_YML" "AC-4"

# cargo audit must be a real run step (not an echo).
if grep -qE '^\s+run:\s+cargo audit' "$CI_YML" 2>/dev/null; then
  tap_pass "AC-4: ci.yml has real 'run: cargo audit' step"
else
  tap_fail "AC-4: ci.yml missing real 'cargo audit' step" \
    "AC-4 FAIL: expected 'run: cargo audit' step in ci.yml; RustSec scan must be a gate, not a TODO echo"
fi

# cargo deny check must also be a real run step.
if grep -qE '^\s+run:\s+cargo deny check' "$CI_YML" 2>/dev/null; then
  tap_pass "AC-4: ci.yml has real 'run: cargo deny check' step"
else
  tap_fail "AC-4: ci.yml missing real 'run: cargo deny check' step" \
    "AC-4 FAIL: expected 'run: cargo deny check' step in ci.yml (license + advisory gate)"
fi

# Verify step ORDER: deny must appear before audit in the file.
# We extract line numbers for the first occurrence of each real run step.
deny_line=$(grep -nE '^\s+run:\s+cargo deny' "$CI_YML" 2>/dev/null | head -1 | cut -d: -f1)
audit_line=$(grep -nE '^\s+run:\s+cargo audit' "$CI_YML" 2>/dev/null | head -1 | cut -d: -f1)
fmt_line=$(grep -nE '^\s+run:\s+cargo fmt' "$CI_YML" 2>/dev/null | head -1 | cut -d: -f1)
clippy_line=$(grep -nE '^\s+run:\s+cargo clippy' "$CI_YML" 2>/dev/null | head -1 | cut -d: -f1)
test_line=$(grep -nE '^\s+run:\s+cargo test' "$CI_YML" 2>/dev/null | head -1 | cut -d: -f1)
semver_line=$(grep -nE '^\s+run:\s+cargo semver' "$CI_YML" 2>/dev/null | head -1 | cut -d: -f1)

# Order check helper: pass if a < b (both non-empty numbers).
check_order() {
  local a="$1" b="$2" label_a="$3" label_b="$4" ac="$5"
  if [ -z "$a" ] || [ -z "$b" ]; then
    tap_fail "${ac}: cannot verify ${label_a} before ${label_b} — one or both steps missing" \
      "${ac} FAIL: step(s) not found as real run steps in ci.yml"
    return
  fi
  if [ "$a" -lt "$b" ]; then
    tap_pass "${ac}: step order correct — ${label_a} (line ${a}) before ${label_b} (line ${b})"
  else
    tap_fail "${ac}: step order WRONG — ${label_a} (line ${a}) must come before ${label_b} (line ${b})" \
      "${ac} FAIL: tooling-selection.md requires order: fmt->clippy->test->deny->audit->semver-checks"
  fi
}

check_order "$fmt_line"    "$clippy_line" "fmt"    "clippy"        "AC-4-order"
check_order "$clippy_line" "$test_line"   "clippy" "test"          "AC-4-order"
check_order "$test_line"   "$deny_line"   "test"   "deny"          "AC-4-order"
check_order "$deny_line"   "$audit_line"  "deny"   "audit"         "AC-4-order"
check_order "$audit_line"  "$semver_line" "audit"  "semver-checks" "AC-4-order"

tap_done
