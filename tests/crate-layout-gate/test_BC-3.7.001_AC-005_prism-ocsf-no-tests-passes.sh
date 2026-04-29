#!/usr/bin/env bash
# test_BC-3.7.001_AC-005_prism-ocsf-no-tests-passes.sh
#
# AC-005 / TV-4 / EC-003: a crate without a tests/ directory (modelled on
# prism-ocsf) must NOT produce a violation — tests/ is optional per ADR-012.
#
# Also directly validates the REAL prism-ocsf crate in the workspace.
#
# BC-3.7.001 postcondition 5.
#
# MUST FAIL at Red Gate: stub exits 1.

set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
# shellcheck source=tap_lib.sh
source "${SCRIPT_DIR}/tap_lib.sh"

SCRIPT="${WORKTREE}/scripts/check-crate-layout.sh"

# ── Synthetic fixture: crate with src/lib.rs, no tests/ ──────────────────────
TMP_WORKSPACE="$(mktemp -d)"
trap 'rm -rf "${TMP_WORKSPACE}"' EXIT

mkdir -p "${TMP_WORKSPACE}/crates/mock-prism-ocsf/src"
cat > "${TMP_WORKSPACE}/crates/mock-prism-ocsf/Cargo.toml" <<'EOF'
[package]
name = "mock-prism-ocsf"
version = "0.1.0"
edition = "2021"
EOF
echo "// no tests/ — conformant" > "${TMP_WORKSPACE}/crates/mock-prism-ocsf/src/lib.rs"
# Intentionally NO tests/ directory.

output=$(WORKSPACE_ROOT="${TMP_WORKSPACE}" bash "${SCRIPT}" 2>&1)
exit_code=$?

# ── Test 1: synthetic no-tests crate passes ──────────────────────────────────
if [ "${exit_code}" -eq 0 ]; then
  tap_pass "AC-005/TV-4/EC-003: crate without tests/ dir passes (synthetic fixture)"
else
  tap_fail "AC-005/TV-4/EC-003: crate without tests/ dir passes (synthetic fixture)" \
    "Exit code: ${exit_code}. Output: ${output}"
fi

# ── Test 2: no violation line for mock-prism-ocsf ────────────────────────────
if echo "${output}" | grep -q "mock-prism-ocsf"; then
  tap_fail "AC-005/EC-003: no violation line emitted for 'mock-prism-ocsf'" \
    "Unexpected violation: $(echo "${output}" | grep "mock-prism-ocsf")"
else
  tap_pass "AC-005/EC-003: no violation line emitted for 'mock-prism-ocsf'"
fi

# ── Test 3: real prism-ocsf in workspace has no tests/ directory ─────────────
OCSF_TESTS_DIR="${WORKTREE}/crates/prism-ocsf/tests"
if [ ! -d "${OCSF_TESTS_DIR}" ]; then
  tap_pass "AC-005: crates/prism-ocsf/tests/ does not exist (confirmed optional)"
else
  tap_skip "AC-005: crates/prism-ocsf/tests/ now exists" \
    "prism-ocsf gained a tests/ dir — no longer validates the optional-tests exception; test coverage OK"
fi

# ── Test 4: real prism-ocsf passes the script (workspace-level) ──────────────
real_output=$(bash "${SCRIPT}" 2>&1)
real_exit=$?
# We care specifically that prism-ocsf is NOT in violation output.
if echo "${real_output}" | grep -q "prism-ocsf"; then
  tap_fail "AC-005: real prism-ocsf must not appear in violation output" \
    "Violation found: $(echo "${real_output}" | grep "prism-ocsf")"
else
  tap_pass "AC-005: real prism-ocsf does not appear in violation output"
fi

tap_done
