#!/usr/bin/env bash
# tests/release-gate/run.sh
#
# Aggregator: runs all test_AC-*.sh files and produces TAP output.
# Exits 1 if any test fails (Red Gate active).
#
# Story: S-REL-001 — devops: release.yml repair
# Wave: F-A  Cycle: v1.0.0-release-engineering
#
# Red Gate discipline: AC-001..AC-005, AC-008 (v4.1.1), AC-009, AC-010, AC-011
# MUST fail before implementation. AC-006, AC-007, and partial AC-008 already
# pass on the unimplemented release.yml (see red-gate-log for details).
#
# Traces to: S-REL-001 AC-001 through AC-011
#
# Usage:
#   bash tests/release-gate/run.sh
#   just test-release-gate   (Justfile target added by implementer)
#
# Compatible with bash 3.2+ (macOS default).

set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

PASS=0
FAIL=0
SKIP=0
TOTAL=0

echo "TAP version 13"
echo "# S-REL-001 Release Gate — Red Gate validation suite"
echo "# release.yml repair (AC-001..AC-011)"
echo "# Running from: ${SCRIPT_DIR}"
echo ""

for test_file in "${SCRIPT_DIR}"/test_AC-*.sh; do
  [ -f "$test_file" ] || continue
  test_name="$(basename "$test_file")"
  echo "# --- ${test_name} ---"

  # Run the test file; capture output and exit code.
  output=$(bash "$test_file" 2>&1)
  exit_code=$?

  # Print output with context prefix.
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
echo "# S-REL-001 Release Gate Summary"
echo "# Total:  ${TOTAL}"
echo "# Passed: ${PASS}"
echo "# Failed: ${FAIL}"
echo "# Skipped (tool not found): ${SKIP}"
echo "# ========================================"

if [ "$FAIL" -gt 0 ]; then
  echo "# RED GATE ACTIVE: ${FAIL} test(s) failing — release.yml repair required."
  exit 1
else
  echo "# WARNING: All tests passed — Red Gate breached. Verify implementation is complete."
  exit 0
fi
