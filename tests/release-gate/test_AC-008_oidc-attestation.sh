#!/usr/bin/env bash
# S-REL-001 AC-008: OIDC attestation preserved with correct pin v4.1.1.
#
# AC: id-token: write permission is present; attest-build-provenance step uses v4.1.1
# (NOT v4.1.0). Pinned to a resolved commit SHA with a version comment.
#
# Red Gate state (mixed):
#   - 'id-token: write' present → PASS at Red
#   - 'attest-build-provenance' step present → PASS at Red
#   - '# v4.1.1' comment → FAIL at Red (current comment says '# v4.1.0')
#   - '# v4.1.0' absent guard → FAIL at Red (forbidden stale version is present)
#
# Traces to: delta-analysis.md §2.1 "OIDC attestation"; research U5
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-008"

# 1. id-token: write permission must be present (required for OIDC).
# PASS at Red: already present on line 18.
assert_contains "$REL_YML" "id-token: write" "AC-008"

# 2. attest-build-provenance action must be referenced.
# PASS at Red: step present on line 92.
assert_contains "$REL_YML" "attest-build-provenance" "AC-008"

# 3. Version pin comment must say v4.1.1 (NOT v4.1.0 which is stale per research U5).
# SID-2: assert the FULL composed pin comment '# v4.1.1', not just 'v4.1' or 'attest'.
# FAIL at Red: current comment says '# v4.1.0'.
assert_contains "$REL_YML" "# v4.1.1" "AC-008"

# 4. Stale v4.1.0 must not be referenced (makes the v4.1.1 assertion meaningful).
# FAIL at Red: '# v4.1.0' is present in the current file.
assert_not_contains "$REL_YML" "# v4.1.0" "AC-008"

# 5. SHA pin must be present (action pinned to immutable commit SHA per repo convention).
# The format is '@<40-hex-chars>'. Use regex to detect.
if grep -qE 'attest-build-provenance@[0-9a-f]{40}' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-008: attest-build-provenance is SHA-pinned (immutable commit SHA present)"
else
  tap_fail "AC-008: attest-build-provenance is NOT SHA-pinned" \
    "AC-008 FAIL: expected 'attest-build-provenance@<40-char-SHA>' in release.yml"
fi

tap_done
