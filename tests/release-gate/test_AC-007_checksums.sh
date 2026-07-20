#!/usr/bin/env bash
# S-REL-001 AC-007: SHA-256 checksums step preserved.
#
# AC: At least one step that generates SHA-256 checksums and uploads
# checksums.txt is present.
#
# Red Gate state: EXPECTED TO PASS — the checksum step exists in the current
# unimplemented release.yml (sha256sum + shasum -a 256 conditional). These
# assertions confirm the implementation preserves the checksum infrastructure.
#
# Traces to: delta-analysis.md §2.1 "keep checksums"
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-007"

# sha256sum (Linux) or shasum -a 256 (macOS) must be present.
if grep -qF 'sha256sum' "$REL_YML" 2>/dev/null || \
   grep -qF 'shasum -a 256' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-007: SHA-256 computation command present (sha256sum or shasum -a 256)"
else
  tap_fail "AC-007: SHA-256 computation command absent" \
    "AC-007 FAIL: expected 'sha256sum' or 'shasum -a 256' step in release.yml"
fi

# checksums.txt must be referenced as an output file.
assert_contains "$REL_YML" "checksums.txt" "AC-007"

# The checksum merge step (cat artifacts/release-*/checksums.txt) must be present
# in the publish-release job. This ensures multi-platform checksums are combined.
assert_contains "$REL_YML" "artifacts/release-*/checksums.txt" "AC-007"

tap_done
