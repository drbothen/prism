#!/usr/bin/env bash
# tests/crate-layout-gate/run.sh
#
# Aggregator: runs all crate-layout-gate test files and emits TAP output.
# Exits non-zero if any test fails.
#
# Red Gate discipline: ALL tests MUST fail until the implementer replaces the
# check-crate-layout.sh stub body with real validation logic (S-3.5.01).
#
# Traces to: BC-3.7.001 (VP-134, VP-135, VP-136)
# Story: S-3.5.01
#
# Usage:
#   bash tests/crate-layout-gate/run.sh
#   just test-crate-layout-gate   (once Justfile target is added by implementer)

set -uo pipefail
GATE_DIR="$(cd "$(dirname "$0")" && pwd)"

PASS=0
FAIL=0
SKIP=0
TOTAL=0

echo "TAP version 13"
echo "# S-3.5.01 Crate Layout Gate — Red Gate validation suite"
echo "# Traces to BC-3.7.001 (VP-134, VP-135, VP-136)"
echo "# Running from: ${GATE_DIR}"
echo ""

for test_file in "${GATE_DIR}"/test_*.sh; do
  [ -f "$test_file" ] || continue
  test_name="$(basename "$test_file")"
  echo "# --- ${test_name} ---"

  # Run the test file; capture output and exit code.
  output=$(bash "$test_file" 2>&1)
  exit_code=$?

  # Print output lines.
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
echo "# S-3.5.01 Crate Layout Gate Summary"
echo "# Total:   ${TOTAL}"
echo "# Passed:  ${PASS}"
echo "# Failed:  ${FAIL}"
echo "# Skipped: ${SKIP}"
echo "# ========================================"

if [ "$FAIL" -gt 0 ]; then
  echo "# RED GATE ACTIVE: ${FAIL} test(s) failing — check-crate-layout.sh stub not yet implemented."
  exit 1
else
  echo "# WARNING: All tests passed — Red Gate lifted. Implementation complete."
  exit 0
fi
