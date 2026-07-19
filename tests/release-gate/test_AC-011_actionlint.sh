#!/usr/bin/env bash
# S-REL-001 AC-011: Workflow YAML parses without errors (actionlint).
#
# AC: actionlint .github/workflows/release.yml exits 0. Zero errors reported.
#
# Red Gate: EXPECTED TO FAIL — actionlint 1.7.12 reports SC2086 shellcheck
# findings on the current release.yml (double-quote prevention for globbing/
# word splitting in the 'Check for binary crate' and 'Create archive' run: blocks).
# exitcode=1 at Red.
#
# Note: actionlint must be installed via 'brew install actionlint' or the official
# download-actionlint.bash script. 'cargo install actionlint' DOES NOT WORK
# (actionlint is a Go tool; research U4). If actionlint is not installed, this
# test is SKIPPED (not failed).
#
# Traces to: delta-analysis.md §8 "manual test tag push gate"; research U4
# requires: bash 3.2+, actionlint (brew install actionlint)

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-011"

if ! command -v actionlint >/dev/null 2>&1; then
  tap_skip "AC-011: actionlint exits 0 on release.yml" \
    "actionlint not installed — install via 'brew install actionlint' (NOT cargo install, which does not work)"
  tap_done
  exit 0
fi

# Run actionlint; capture output and exit code without triggering set -e.
set +e
actionlint_output=$(actionlint "$REL_YML" 2>&1)
actionlint_exit=$?
set -e

if [ "$actionlint_exit" -eq 0 ]; then
  tap_pass "AC-011: actionlint .github/workflows/release.yml exits 0 (zero errors)"
else
  tap_fail "AC-011: actionlint .github/workflows/release.yml exits ${actionlint_exit} (errors present)" \
    "$(echo "$actionlint_output" | head -3 | tr '\n' '|')"
fi

tap_done
