#!/usr/bin/env bash
# S-REL-001 AC-006: 4-platform matrix preserved and correctly spelled (ADR-065 D1).
#
# AC: All four targets are present and correctly spelled. Notably musl is
# 'x86_64-unknown-linux-musl' NOT 'x86_x64-unknown-linux-musl'.
#
# Red Gate state: EXPECTED TO PASS — all 4 targets are correctly present in the
# release.yml. The musl typo (U1) was already corrected. These
# assertions confirm the implementation does not accidentally break the matrix.
#
# Traces to: delta-analysis.md §2.1; U1 typo fix x86_x64->x86_64
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-006"

# All 4 required targets must be present with correct spelling (ADR-065 D1).
assert_contains "$REL_YML" "aarch64-apple-darwin" "AC-006"
assert_contains "$REL_YML" "x86_64-unknown-linux-gnu" "AC-006"
assert_contains "$REL_YML" "x86_64-unknown-linux-musl" "AC-006"
assert_contains "$REL_YML" "x86_64-pc-windows-msvc" "AC-006"

# Musl typo guard: 'x86_x64-unknown-linux-musl' must NOT be present.
# This is the defect spelling from U1 — the correct form is x86_64 (with '64').
assert_not_contains "$REL_YML" "x86_x64-unknown-linux-musl" "AC-006"

# Exactly 4 matrix target: entries (ADR-065 D1). Counts lines matching '^[[:space:]]*target:' which are
# the matrix include entries. Guards against accidentally adding or removing a leg.
target_count=$(grep -c '^[[:space:]]*target:' "$REL_YML" 2>/dev/null || echo 0)
if [ "$target_count" -eq 4 ]; then
  tap_pass "AC-006: exactly 4 matrix 'target:' entries found (count=${target_count})"
else
  tap_fail "AC-006: expected 4 matrix target entries, found ${target_count}" \
    "AC-006 FAIL: release.yml matrix must have exactly 4 target: entries (ADR-065 D1)"
fi

# fail-fast: false must be set so one platform failure does not cancel others.
assert_contains "$REL_YML" "fail-fast: false" "AC-006"

tap_done
