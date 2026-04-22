#!/usr/bin/env bash
# tests/toolchain-gate/run.sh
# Aggregator: runs all toolchain-gate tests and emits TAP output.
# Exits non-zero if any test file exits non-zero.
# Usage: bash tests/toolchain-gate/run.sh
set -euo pipefail

GATE_DIR="$(cd "$(dirname "$0")" && pwd)"
PASS=0
FAIL=0
SKIP=0
TEST_NUM=0

echo "TAP version 13"

for test_file in "$GATE_DIR"/test_*.sh; do
  TEST_NUM=$((TEST_NUM+1))
  test_name="$(basename "$test_file")"

  # Run each test script; capture output and exit code
  exit_code=0
  output=$(bash "$test_file" 2>&1) || exit_code=$?

  # Print the sub-test output indented
  while IFS= read -r line; do
    echo "    $line"
  done <<< "$output"

  if [[ $exit_code -eq 0 ]]; then
    echo "ok $TEST_NUM - $test_name"
    PASS=$((PASS+1))
  else
    echo "not ok $TEST_NUM - $test_name"
    FAIL=$((FAIL+1))
  fi
done

TOTAL=$((PASS+FAIL))
echo "1..$TOTAL"
echo ""
echo "# Results: $PASS passed, $FAIL failed out of $TOTAL test files"

if [[ $FAIL -gt 0 ]]; then
  echo "# Red Gate: FAIL — $FAIL test file(s) non-zero (expected before implementation)"
  exit 1
else
  echo "# Red Gate: PASS — all tests passed (unexpected on stubs — review test logic)"
  exit 0
fi
