#!/usr/bin/env bash
# test_BC-3.7.001_AC-007_build-rs-permitted.sh
#
# AC-007 / TV-5 / EC-004: a crate with src/lib.rs AND build.rs at the crate
# root must NOT produce a "loose .rs file at crate root" violation.
# build.rs is the Cargo-mandated build script location (ADR-012 §7 OQ-1).
#
# Also validates the real prism-ocsf crate which has build.rs.
#
# BC-3.7.001 postcondition 7.
#
# MUST FAIL at Red Gate: stub exits 1.

set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
# shellcheck source=tap_lib.sh
source "${SCRIPT_DIR}/tap_lib.sh"

SCRIPT="${WORKTREE}/scripts/check-crate-layout.sh"

# ── Synthetic fixture: conformant crate with build.rs ────────────────────────
TMP_WORKSPACE="$(mktemp -d)"
trap 'rm -rf "${TMP_WORKSPACE}"' EXIT

mkdir -p "${TMP_WORKSPACE}/crates/mock-with-build-rs/src"
cat > "${TMP_WORKSPACE}/crates/mock-with-build-rs/Cargo.toml" <<'EOF'
[package]
name = "mock-with-build-rs"
version = "0.1.0"
edition = "2021"
build = "build.rs"
EOF
echo "// conformant src entry" > "${TMP_WORKSPACE}/crates/mock-with-build-rs/src/lib.rs"
echo "fn main() {}" > "${TMP_WORKSPACE}/crates/mock-with-build-rs/build.rs"

output=$(WORKSPACE_ROOT="${TMP_WORKSPACE}" bash "${SCRIPT}" 2>&1)
exit_code=$?

# ── Test 1: synthetic crate with build.rs passes ─────────────────────────────
if [ "${exit_code}" -eq 0 ]; then
  tap_pass "AC-007/TV-5/EC-004: crate with build.rs at root passes (synthetic fixture)"
else
  tap_fail "AC-007/TV-5/EC-004: crate with build.rs at root passes (synthetic fixture)" \
    "Exit code: ${exit_code}. Output: ${output}"
fi

# ── Test 2: no violation line for mock-with-build-rs ─────────────────────────
if echo "${output}" | grep -q "mock-with-build-rs"; then
  tap_fail "AC-007/EC-004: no violation line for 'mock-with-build-rs'" \
    "Unexpected: $(echo "${output}" | grep "mock-with-build-rs")"
else
  tap_pass "AC-007/EC-004: no violation line emitted for 'mock-with-build-rs'"
fi

# ── Test 3: real prism-ocsf has build.rs and is not flagged ──────────────────
if [ -f "${WORKTREE}/crates/prism-ocsf/build.rs" ]; then
  real_output=$(bash "${SCRIPT}" 2>&1)
  if echo "${real_output}" | grep -q "prism-ocsf"; then
    tap_fail "AC-007: real prism-ocsf (has build.rs) must not appear in violations" \
      "Violation: $(echo "${real_output}" | grep "prism-ocsf")"
  else
    tap_pass "AC-007: real prism-ocsf (has build.rs) does not appear in violations"
  fi
else
  tap_skip "AC-007: prism-ocsf does not have build.rs" \
    "prism-ocsf/build.rs not present — real-crate cross-check skipped"
fi

tap_done
