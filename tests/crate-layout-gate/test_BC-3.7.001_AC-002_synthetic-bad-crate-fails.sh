#!/usr/bin/env bash
# test_BC-3.7.001_AC-002_synthetic-bad-crate-fails.sh
#
# AC-002 / VP-135 / TV-2: a synthetic crate with lib.rs at root (no src/)
# causes check-crate-layout.sh to exit non-zero with a violation line naming
# the crate and the violated rule.
#
# BC-3.7.001 postconditions 2 and 3 + EC-001.
#
# MUST FAIL at Red Gate: stub exits 1 but does not emit the required violation
# line format — the output-format assertions will fail.

set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
# shellcheck source=tap_lib.sh
source "${SCRIPT_DIR}/tap_lib.sh"

SCRIPT="${WORKTREE}/scripts/check-crate-layout.sh"

# Create a temporary workspace tree with one non-conformant crate.
TMP_WORKSPACE="$(mktemp -d)"
trap 'rm -rf "${TMP_WORKSPACE}"' EXIT

mkdir -p "${TMP_WORKSPACE}/crates/test-bad-no-src"
cat > "${TMP_WORKSPACE}/crates/test-bad-no-src/Cargo.toml" <<'EOF'
[package]
name = "test-bad-no-src"
version = "0.1.0"
edition = "2021"
EOF
echo "// bad: loose lib.rs at crate root" > "${TMP_WORKSPACE}/crates/test-bad-no-src/lib.rs"
# NO src/ directory — deliberate violation.

output=$(WORKSPACE_ROOT="${TMP_WORKSPACE}" bash "${SCRIPT}" 2>&1)
exit_code=$?

# ── Test 1: script exits non-zero ────────────────────────────────────────────
if [ "${exit_code}" -ne 0 ]; then
  tap_pass "AC-002/VP-135/TV-2: script exits non-zero for crate with lib.rs at root"
else
  tap_fail "AC-002/VP-135/TV-2: script exits non-zero for crate with lib.rs at root" \
    "Expected non-zero exit; got 0. Stub may have been replaced without real logic."
fi

# ── Test 2: crate name appears in output (AC-003 partial) ────────────────────
if echo "${output}" | grep -q "test-bad-no-src"; then
  tap_pass "AC-002/AC-003: violation output names the offending crate 'test-bad-no-src'"
else
  tap_fail "AC-002/AC-003: violation output names the offending crate 'test-bad-no-src'" \
    "Output: ${output}"
fi

# ── Test 3: output describes the missing src/lib.rs rule ─────────────────────
if echo "${output}" | grep -qE "src/lib\.rs|src/main\.rs|no src/"; then
  tap_pass "AC-002/AC-003: violation output describes the src/lib.rs rule"
else
  tap_fail "AC-002/AC-003: violation output describes the src/lib.rs rule" \
    "Output did not contain 'src/lib.rs', 'src/main.rs', or 'no src/'. Got: ${output}"
fi

# ── Test 4: violation line matches "crates/<name>: <rule>" format (AC-003) ───
if echo "${output}" | grep -qE '^crates/[^:]+: '; then
  tap_pass "AC-003: violation line follows 'crates/<name>: <rule description>' format"
else
  tap_fail "AC-003: violation line follows 'crates/<name>: <rule description>' format" \
    "No line starting with 'crates/' found. Output: ${output}"
fi

tap_done
