#!/usr/bin/env bash
# AC-6: Release workflow builds 5 platform binaries, computes SHA-256, creates GitHub Release.
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-6"

# Trigger must be v* tags only (not branches).
if grep -qF "tags:" "$REL_YML" && grep -qF "'v*'" "$REL_YML" 2>/dev/null; then
  tap_pass "AC-6: release.yml triggers on 'v*' tags"
else
  tap_fail "AC-6: release.yml missing tag trigger 'v*'" \
    "AC-6 FAIL: release workflow must trigger on tags: ['v*'] only, not on PR branches"
fi

# Must NOT trigger on pull_request (architecture compliance rule).
if grep -qF "pull_request" "$REL_YML" 2>/dev/null; then
  tap_fail "AC-6: release.yml incorrectly triggers on pull_request" \
    "AC-6 FAIL: release workflow must NOT run on PR branches — tag trigger only"
else
  tap_pass "AC-6: release.yml does not trigger on pull_request"
fi

# 5 platform targets must be in matrix.
TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "x86_64-unknown-linux-musl"
  "x86_64-pc-windows-msvc"
)
for target in "${TARGETS[@]}"; do
  if grep -qF "$target" "$REL_YML" 2>/dev/null; then
    tap_pass "AC-6: release matrix target '${target}' present"
  else
    tap_fail "AC-6: release matrix target '${target}' missing" \
      "AC-6 FAIL: all 5 platform targets required in release.yml matrix"
  fi
done

# cargo build --release --locked must be a real run step.
if grep -qE '^\s+run:\s+cargo build --release --locked' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-6: 'cargo build --release --locked' is a real run step"
else
  tap_fail "AC-6: 'cargo build --release --locked' missing as real run step" \
    "AC-6 FAIL: expected 'run: cargo build --release --locked --target ...' — found only TODO echo"
fi

# SHA-256 checksum step must be real (sha256sum or shasum invocation).
if grep -qE '^\s+run:.*sha256sum|shasum' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-6: SHA-256 checksum step is a real run step"
else
  tap_fail "AC-6: SHA-256 checksum step missing or still a TODO echo" \
    "AC-6 FAIL: expected real sha256sum/shasum run step producing checksums.txt"
fi

# checksums.txt must be referenced.
if grep -qF "checksums.txt" "$REL_YML" 2>/dev/null; then
  tap_pass "AC-6: checksums.txt referenced in release.yml"
else
  tap_fail "AC-6: checksums.txt not referenced in release.yml" \
    "AC-6 FAIL: checksums.txt must be generated and attached to GitHub Release"
fi

# gh release create must be a real step.
if grep -qE '^\s+run:.*gh release create' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-6: 'gh release create' is a real run step"
else
  tap_fail "AC-6: 'gh release create' missing as real run step" \
    "AC-6 FAIL: expected 'run: gh release create ...' step — found only TODO echo"
fi

tap_done
