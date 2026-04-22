#!/usr/bin/env bash
# AC-7: Release workflow opens a PR against the Homebrew tap with updated url and sha256.
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-7"

# homebrew-update job must exist and need build-release.
if grep -qF "homebrew-update:" "$REL_YML" 2>/dev/null; then
  tap_pass "AC-7: homebrew-update job defined in release.yml"
else
  tap_fail "AC-7: homebrew-update job missing from release.yml" \
    "AC-7 FAIL: expected 'homebrew-update:' job in release.yml"
fi

# Must checkout tap repo 1898co/homebrew-tap.
if grep -qF "1898co/homebrew-tap" "$REL_YML" 2>/dev/null; then
  tap_pass "AC-7: tap repo '1898co/homebrew-tap' referenced"
else
  tap_fail "AC-7: tap repo '1898co/homebrew-tap' not referenced" \
    "AC-7 FAIL: expected checkout of '1898co/homebrew-tap' in homebrew-update job"
fi

# Formula/prism.rb must be referenced for the url+sha256 update.
if grep -qF "Formula/prism.rb" "$REL_YML" 2>/dev/null; then
  tap_pass "AC-7: 'Formula/prism.rb' referenced in release.yml"
else
  tap_fail "AC-7: 'Formula/prism.rb' not referenced" \
    "AC-7 FAIL: homebrew-update job must update Formula/prism.rb url and sha256"
fi

# gh pr create must be invoked against the tap.
if grep -qE '^\s+run:.*gh pr create' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-7: 'gh pr create' is a real run step in homebrew-update job"
else
  tap_fail "AC-7: 'gh pr create' missing as real run step" \
    "AC-7 FAIL: expected 'run: gh pr create' step in homebrew-update job — found only TODO echo"
fi

# HOMEBREW_TAP_TOKEN secret must be referenced (not hardcoded).
if grep -qF "HOMEBREW_TAP_TOKEN" "$REL_YML" 2>/dev/null; then
  tap_pass "AC-7: HOMEBREW_TAP_TOKEN is referenced in release.yml"
else
  tap_fail "AC-7: HOMEBREW_TAP_TOKEN not referenced" \
    "AC-7 FAIL: homebrew-update job must use secrets.HOMEBREW_TAP_TOKEN to authenticate with tap repo"
fi

tap_done
