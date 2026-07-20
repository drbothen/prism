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

# Floor constants — exact expected counts (not minimums).
# When adding or removing a test file or assertion, update ALL THREE:
#   1. EXPECTED_TEST_FILES here
#   2. EXPECTED_ASSERTIONS here
#   3. README.md "Floor constants" section assertion count
# This floor exists so silent coverage shrink fails loud (F-REL001-P7-001).
# Precedent: scripts/check-non-exhaustive.sh EXPECTED=92 pattern.
EXPECTED_TEST_FILES=11
EXPECTED_ASSERTIONS=81

PASS=0
FAIL=0
SKIP=0
TOTAL=0
FILES_EXECUTED=0

echo "TAP version 13"
echo "# S-REL-001 Release Gate — Red Gate validation suite"
echo "# release.yml repair (AC-001..AC-011)"
echo "# Running from: ${SCRIPT_DIR}"
echo ""

for test_file in "${SCRIPT_DIR}"/test_AC-*.sh; do
  [ -f "$test_file" ] || continue
  FILES_EXECUTED=$((FILES_EXECUTED + 1))
  test_name="$(basename "$test_file")"
  echo "# --- ${test_name} ---"

  # Run the test file; capture output and exit code.
  output=$(bash "$test_file" 2>&1)
  exit_code=$?

  # Print output with context prefix.
  while IFS= read -r line; do
    echo "$line"
  done <<< "$output"

  # Tally from TAP result lines.
  # tap_skip emits "ok N - msg # SKIP reason" and must NOT inflate PASS count.
  # Use a case loop to distinguish pass / fail / skip with no grep edge cases.
  file_pass=0
  file_fail=0
  file_skip=0
  while IFS= read -r tap_line; do
    case "$tap_line" in
      'ok '*'# SKIP'*) file_skip=$((file_skip + 1)) ;;
      'ok '*)          file_pass=$((file_pass + 1)) ;;
      'not ok '*)      file_fail=$((file_fail + 1)) ;;
    esac
  done <<< "$output"

  # TAP plan reconciliation: parse 1..N and verify assertion count matches.
  # Missing or mismatched plan is treated as a hard failure (silent-shrink guard).
  plan_n=$(echo "$output" | grep '^1\.\.[0-9]' | head -1 | cut -c4-)
  if [ -z "$plan_n" ]; then
    echo "# HARNESS ERROR: ${test_name} produced no TAP plan line (1..N missing) — hard failure"
    FAIL=$((FAIL + 1))
  else
    tap_total=$((file_pass + file_fail + file_skip))
    if [ "$tap_total" -ne "$plan_n" ]; then
      echo "# HARNESS ERROR: ${test_name} plan mismatch — plan declares ${plan_n} tests, counted ${tap_total} result lines (pass:${file_pass} fail:${file_fail} skip:${file_skip})"
      FAIL=$((FAIL + 1))
    fi
  fi

  # Non-zero exit with no reported failures = crash or silent failure before tap_done.
  if [ "$exit_code" -ne 0 ] && [ "$file_fail" -eq 0 ]; then
    echo "# HARNESS ERROR: ${test_name} exited ${exit_code} with zero TAP failures — likely crashed before tap_done"
    FAIL=$((FAIL + 1))
  fi

  PASS=$((PASS + file_pass))
  FAIL=$((FAIL + file_fail))
  SKIP=$((SKIP + file_skip))
  TOTAL=$((TOTAL + file_pass + file_fail + file_skip))

  echo ""
done

# Floor guard — exact equality, not >= (F-REL001-P7-001 / EXPECTED=92 precedent).
# An unexpected increase also demands a conscious constant bump.
if [ "$FILES_EXECUTED" -ne "$EXPECTED_TEST_FILES" ]; then
  echo "HARNESS ERROR: expected ${EXPECTED_TEST_FILES} test files, executed ${FILES_EXECUTED} — add/remove a test_AC-*.sh file? Update EXPECTED_TEST_FILES + README."
  exit 1
fi
if [ "$TOTAL" -ne "$EXPECTED_ASSERTIONS" ]; then
  echo "HARNESS ERROR: expected ${EXPECTED_ASSERTIONS} assertions, counted ${TOTAL} — update EXPECTED_ASSERTIONS + README."
  exit 1
fi
[ "$TOTAL" -gt 0 ] || { echo "HARNESS ERROR: no tests executed"; exit 1; }

echo "# ========================================"
echo "# S-REL-001 Release Gate Summary"
echo "# Total:   ${TOTAL}"
echo "# Passed:  ${PASS}"
echo "# Failed:  ${FAIL}"
echo "# Skipped: ${SKIP}"
echo "# ========================================"

if [ "$FAIL" -gt 0 ]; then
  echo "# RED GATE ACTIVE: ${FAIL} test(s) failing — release.yml repair required."
  exit 1
else
  echo "# All release-gate tests passed."
  exit 0
fi
