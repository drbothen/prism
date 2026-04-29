#!/usr/bin/env bash
# test_BC-3.7.001_EC-007_loose-rs-not-buildrs.sh
#
# EC-007: a .rs file at crate root with a name OTHER than build.rs (e.g.,
# helpers.rs) must trigger a "loose .rs file at crate root" violation.
#
# BC-3.7.001 edge case EC-007.
#
# MUST FAIL at Red Gate: stub exits 1 but with wrong output format.

set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
# shellcheck source=tap_lib.sh
source "${SCRIPT_DIR}/tap_lib.sh"

SCRIPT="${WORKTREE}/scripts/check-crate-layout.sh"

TMP_WORKSPACE="$(mktemp -d)"
trap 'rm -rf "${TMP_WORKSPACE}"' EXIT

mkdir -p "${TMP_WORKSPACE}/crates/test-ec007-loose-helpers/src"
cat > "${TMP_WORKSPACE}/crates/test-ec007-loose-helpers/Cargo.toml" <<'EOF'
[package]
name = "test-ec007-loose-helpers"
version = "0.1.0"
edition = "2021"
EOF
echo "// ok src entry" > "${TMP_WORKSPACE}/crates/test-ec007-loose-helpers/src/lib.rs"
# Loose helpers.rs at crate root — EC-007 violation
echo "// bad: loose .rs at root" > "${TMP_WORKSPACE}/crates/test-ec007-loose-helpers/helpers.rs"

output=$(WORKSPACE_ROOT="${TMP_WORKSPACE}" bash "${SCRIPT}" 2>&1)
exit_code=$?

# ── Test 1: script exits non-zero ────────────────────────────────────────────
if [ "${exit_code}" -ne 0 ]; then
  tap_pass "EC-007: script exits non-zero for loose helpers.rs at crate root"
else
  tap_fail "EC-007: script exits non-zero for loose helpers.rs at crate root" \
    "Got exit 0 — loose .rs file was not detected"
fi

# ── Test 2: crate name or file name appears in output ────────────────────────
if echo "${output}" | grep -qE "helpers\.rs|test-ec007-loose-helpers"; then
  tap_pass "EC-007: violation output references 'helpers.rs' or the crate name"
else
  tap_fail "EC-007: violation output references 'helpers.rs' or the crate name" \
    "Output: ${output}"
fi

# ── Test 3: build.rs is NOT flagged alongside helpers.rs ─────────────────────
# Add a build.rs to the same crate — it must NOT add a second violation.
echo "fn main() {}" > "${TMP_WORKSPACE}/crates/test-ec007-loose-helpers/build.rs"

output2=$(WORKSPACE_ROOT="${TMP_WORKSPACE}" bash "${SCRIPT}" 2>&1)

# build.rs itself should not appear as a separate violation
build_rs_violations=$(echo "${output2}" | grep "build\.rs" || true)
if [ -z "${build_rs_violations}" ]; then
  tap_pass "EC-007/AC-007: build.rs is not mentioned in violation output"
else
  tap_fail "EC-007/AC-007: build.rs must not be flagged as a violation" \
    "Unexpected build.rs mention: ${build_rs_violations}"
fi

tap_done
