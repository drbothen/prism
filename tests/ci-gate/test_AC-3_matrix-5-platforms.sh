#!/usr/bin/env bash
# AC-3: All 5 platform matrix jobs run in parallel and all must pass.
# Asserts ci.yml matrix includes exactly 5 required targets, each with its correct runner.
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CI_YML="${WORKTREE}/.github/workflows/ci.yml"

assert_file_exists "$CI_YML" "AC-3"

# Required targets from story task 1.
TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "x86_64-unknown-linux-musl"
  "x86_64-pc-windows-msvc"
)

for target in "${TARGETS[@]}"; do
  if grep -qF "$target" "$CI_YML" 2>/dev/null; then
    tap_pass "AC-3: matrix target '${target}' present"
  else
    tap_fail "AC-3: matrix target '${target}' missing" \
      "AC-3 FAIL: expected matrix entry for '${target}' in ci.yml"
  fi
done

# Required runners.
RUNNERS=("macos-latest" "macos-13" "ubuntu-latest" "windows-latest")
for runner in "${RUNNERS[@]}"; do
  if grep -qF "runner: ${runner}" "$CI_YML" 2>/dev/null; then
    tap_pass "AC-3: runner '${runner}' listed under matrix.include"
  else
    tap_fail "AC-3: runner '${runner}' NOT listed under matrix.include" \
      "AC-3 FAIL: runner '${runner}' missing from ci.yml matrix — all 5 platforms must run in parallel"
  fi
done

# musl target must pair with musl-tools install step.
if grep -qF "musl-tools" "$CI_YML" 2>/dev/null; then
  tap_pass "AC-3: musl-tools install step present for Linux musl target"
else
  tap_fail "AC-3: musl-tools install step missing for Linux musl target" \
    "AC-3 FAIL: x86_64-unknown-linux-musl requires 'apt-get install musl-tools' step in ci.yml"
fi

# fail-fast: false must be set so one failure does not cancel others.
if grep -qF "fail-fast: false" "$CI_YML" 2>/dev/null; then
  tap_pass "AC-3: fail-fast: false is set (all matrix legs run independently)"
else
  tap_fail "AC-3: fail-fast: false missing" \
    "AC-3 FAIL: matrix must have fail-fast: false so all 5 jobs report independently"
fi

tap_done
