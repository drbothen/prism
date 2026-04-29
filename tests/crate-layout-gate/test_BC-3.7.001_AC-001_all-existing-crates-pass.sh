#!/usr/bin/env bash
# test_BC-3.7.001_AC-001_all-existing-crates-pass.sh
#
# AC-001 / VP-134: running scripts/check-crate-layout.sh against the real
# 22-crate workspace exits 0 with no per-crate violation lines.
#
# BC-3.7.001 postcondition 1 + TV-1.
#
# MUST FAIL at Red Gate: the stub script unconditionally exits 1.

set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
# shellcheck source=tap_lib.sh
source "${SCRIPT_DIR}/tap_lib.sh"

SCRIPT="${WORKTREE}/scripts/check-crate-layout.sh"

# ── Test 1: script exits 0 for the real workspace ────────────────────────────
output=$(bash "${SCRIPT}" 2>&1)
exit_code=$?

if [ "${exit_code}" -eq 0 ]; then
  tap_pass "AC-001/VP-134: check-crate-layout.sh exits 0 for 22-crate workspace"
else
  tap_fail "AC-001/VP-134: check-crate-layout.sh exits 0 for 22-crate workspace" \
    "Got exit code ${exit_code}. Stub is active — real validation not yet implemented."
fi

# ── Test 2: no violation lines produced ──────────────────────────────────────
violation_lines=$(echo "${output}" | grep '^crates/' || true)

if [ -z "${violation_lines}" ]; then
  tap_pass "AC-001/VP-134: no violation lines emitted for conformant workspace"
else
  tap_fail "AC-001/VP-134: no violation lines emitted for conformant workspace" \
    "Unexpected violations: ${violation_lines}"
fi

# ── Test 3: at least 22 crates exist in the workspace ────────────────────────
crate_count=$(find "${WORKTREE}/crates" -maxdepth 2 -name "Cargo.toml" | wc -l | tr -d ' ')

if [ "${crate_count}" -ge 22 ]; then
  tap_pass "AC-001: workspace contains at least 22 crates (found ${crate_count})"
else
  tap_fail "AC-001: workspace contains at least 22 crates" \
    "Found only ${crate_count} — workspace may be incomplete"
fi

tap_done
