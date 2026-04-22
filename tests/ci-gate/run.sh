#!/usr/bin/env bash
# Aggregator: runs all test_AC-*.sh files and produces TAP output.
# Exits 1 if any test fails.
# Compatible with bash 3.2+ (macOS default).
# Usage: bash tests/ci-gate/run.sh
#        just test-ci-gate   (once Justfile target is added by implementer)

set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

PASS=0
FAIL=0
SKIP=0
TOTAL=0

echo "TAP version 13"
echo "# S-0.01 CI Gate — Red Gate validation suite"
echo "# Running from: ${SCRIPT_DIR}"
echo ""

for test_file in "${SCRIPT_DIR}"/test_AC-*.sh; do
  [ -f "$test_file" ] || continue
  test_name="$(basename "$test_file")"
  echo "# --- ${test_name} ---"

  # Run the test file; capture output and exit code.
  output=$(bash "$test_file" 2>&1)
  exit_code=$?

  # Print output with test-file context prefix.
  while IFS= read -r line; do
    echo "$line"
  done <<< "$output"

  # Tally from TAP lines emitted.
  file_pass=$(echo "$output" | grep -c '^ok ' || true)
  file_fail=$(echo "$output" | grep -c '^not ok ' || true)
  file_skip=$(echo "$output" | grep -c '# SKIP' || true)

  PASS=$((PASS + file_pass))
  FAIL=$((FAIL + file_fail))
  SKIP=$((SKIP + file_skip))
  TOTAL=$((TOTAL + file_pass + file_fail))

  echo ""
done

echo "# ========================================"
echo "# S-0.01 Red Gate Summary"
echo "# Total:  ${TOTAL}"
echo "# Passed: ${PASS}"
echo "# Failed: ${FAIL}"
echo "# Skipped (tool not found): ${SKIP}"
echo "# ========================================"

if [ "$FAIL" -gt 0 ]; then
  echo "# RED GATE ACTIVE: ${FAIL} test(s) failing — stubs are hollow, implementation required."
  exit 1
else
  echo "# WARNING: All tests passed — Red Gate breached. Investigate; tests may be vacuously true."
  exit 0
fi
