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

echo "# ========================================"
echo "# S-0.01 Red Gate Summary"
echo "# Total:   ${TOTAL}"
echo "# Passed:  ${PASS}"
echo "# Failed:  ${FAIL}"
echo "# Skipped: ${SKIP}"
echo "# ========================================"

if [ "$FAIL" -gt 0 ]; then
  echo "# RED GATE ACTIVE: ${FAIL} test(s) failing — stubs are hollow, implementation required."
  exit 1
else
  echo "# WARNING: All tests passed — Red Gate breached. Investigate; tests may be vacuously true."
  exit 0
fi
