#!/usr/bin/env bash
# AC-5: Post-merge kani-proofs job with correct timeout, memory limit, and artifact upload.
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PM_YML="${WORKTREE}/.github/workflows/post-merge.yml"

assert_file_exists "$PM_YML" "AC-5"

# Trigger must be push to main only (not all branches).
if grep -qF "branches:" "$PM_YML" 2>/dev/null && grep -qF "- main" "$PM_YML" 2>/dev/null; then
  tap_pass "AC-5: post-merge.yml is scoped to main branch"
else
  tap_fail "AC-5: post-merge.yml not scoped to main branch" \
    "AC-5 FAIL: post-merge workflow must trigger only on push to main"
fi

# kani-proofs job must exist with real cargo kani invocation (not an echo).
if grep -qE '^\s+run:\s+cargo kani' "$PM_YML" 2>/dev/null; then
  tap_pass "AC-5: kani-proofs job has real 'run: cargo kani' step"
else
  tap_fail "AC-5: kani-proofs job missing real 'run: cargo kani' step" \
    "AC-5 FAIL: expected 'run: cargo kani --workspace --timeout 300 --mem-limit 8192' — found only TODO echo"
fi

# --timeout 300 must appear (300 s per proof).
if grep -qF 'timeout 300' "$PM_YML" 2>/dev/null; then
  tap_pass "AC-5: kani invocation includes --timeout 300"
else
  tap_fail "AC-5: --timeout 300 missing from kani invocation" \
    "AC-5 FAIL: cargo kani must include '--timeout 300' per tooling-selection.md"
fi

# --mem-limit 8192 must appear.
if grep -qF 'mem-limit 8192' "$PM_YML" 2>/dev/null; then
  tap_pass "AC-5: kani invocation includes --mem-limit 8192"
else
  tap_fail "AC-5: --mem-limit 8192 missing from kani invocation" \
    "AC-5 FAIL: cargo kani must include '--mem-limit 8192' per tooling-selection.md"
fi

# timeout-minutes: 120 must be set on the kani-proofs job.
if grep -qF "timeout-minutes: 120" "$PM_YML" 2>/dev/null; then
  tap_pass "AC-5: kani-proofs job has timeout-minutes: 120"
else
  tap_fail "AC-5: timeout-minutes: 120 missing on kani-proofs job" \
    "AC-5 FAIL: kani-proofs job must have timeout-minutes: 120 to bound worst-case runtime"
fi

# Artifact upload step must reference kani-report (real uses: action, not echo).
if grep -qE 'uses:\s+actions/upload-artifact' "$PM_YML" 2>/dev/null && \
   grep -qF "kani-report" "$PM_YML" 2>/dev/null; then
  tap_pass "AC-5: kani-report artifact upload step present (uses: actions/upload-artifact)"
else
  tap_fail "AC-5: kani-report artifact upload missing or still a TODO echo" \
    "AC-5 FAIL: expected 'uses: actions/upload-artifact' with name 'kani-report-\${{ github.sha }}'"
fi

# Fuzz corpus job must have all 6 named targets.
FUZZ_TARGETS=(
  "fuzz_prismql_parser"
  "fuzz_alias_expansion"
  "fuzz_normalize"
  "fuzz_spec_parser"
  "fuzz_template_interpolation"
  "fuzz_injection_scanner"
)
for ft in "${FUZZ_TARGETS[@]}"; do
  if grep -qF "$ft" "$PM_YML" 2>/dev/null; then
    # Must appear as a real argument, not only inside an echo.
    if grep -qE "echo.*${ft}" "$PM_YML" 2>/dev/null && \
       ! grep -qE "run:.*cargo fuzz.*${ft}" "$PM_YML" 2>/dev/null; then
      tap_fail "AC-5: fuzz target '${ft}' is still an echo stub" \
        "AC-5 FAIL: fuzz target '${ft}' must be a real cargo-fuzz invocation, not an echo"
    else
      tap_pass "AC-5: fuzz target '${ft}' invoked"
    fi
  else
    tap_fail "AC-5: fuzz target '${ft}' missing from post-merge.yml" \
      "AC-5 FAIL: all 6 fuzz targets required per story task 2"
  fi
done

tap_done
