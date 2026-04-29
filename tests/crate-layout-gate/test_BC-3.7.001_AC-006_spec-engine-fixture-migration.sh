#!/usr/bin/env bash
# test_BC-3.7.001_AC-006_spec-engine-fixture-migration.sh
#
# AC-006 / BC-3.7.001 postcondition 6: after implementation, the
# prism-spec-engine crate must have fixtures/ at the crate root and must NOT
# have tests/fixtures/.
#
# Also exercises TV-3: tests/fixtures/ triggers a violation in the real script.
#
# MUST FAIL at Red Gate: tests/fixtures/ still exists in the workspace.

set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
# shellcheck source=tap_lib.sh
source "${SCRIPT_DIR}/tap_lib.sh"

SPEC_ENGINE="${WORKTREE}/crates/prism-spec-engine"
OLD_PATH="${SPEC_ENGINE}/tests/fixtures"
NEW_PATH="${SPEC_ENGINE}/fixtures"

# ── Test 1: tests/fixtures/ must NOT exist ───────────────────────────────────
if [ ! -d "${OLD_PATH}" ]; then
  tap_pass "AC-006: crates/prism-spec-engine/tests/fixtures/ does not exist (migration complete)"
else
  tap_fail "AC-006: crates/prism-spec-engine/tests/fixtures/ does not exist (migration complete)" \
    "Found ${OLD_PATH} — implementer must run: mv crates/prism-spec-engine/tests/fixtures/ crates/prism-spec-engine/fixtures/"
fi

# ── Test 2: fixtures/ must exist at crate root ───────────────────────────────
if [ -d "${NEW_PATH}" ]; then
  tap_pass "AC-006: crates/prism-spec-engine/fixtures/ exists (migration complete)"
else
  tap_fail "AC-006: crates/prism-spec-engine/fixtures/ exists (migration complete)" \
    "Not found: ${NEW_PATH} — implementer must create this directory"
fi

# ── Test 3: TV-3 — a synthetic crate with tests/fixtures/ triggers violation ─
TMP_WORKSPACE="$(mktemp -d)"
trap 'rm -rf "${TMP_WORKSPACE}"' EXIT

mkdir -p "${TMP_WORKSPACE}/crates/test-tv3-tests-fixtures/src"
mkdir -p "${TMP_WORKSPACE}/crates/test-tv3-tests-fixtures/tests/fixtures"
cat > "${TMP_WORKSPACE}/crates/test-tv3-tests-fixtures/Cargo.toml" <<'EOF'
[package]
name = "test-tv3-tests-fixtures"
version = "0.1.0"
edition = "2021"
EOF
echo "// ok src" > "${TMP_WORKSPACE}/crates/test-tv3-tests-fixtures/src/lib.rs"
echo '{"data": "fixture"}' > "${TMP_WORKSPACE}/crates/test-tv3-tests-fixtures/tests/fixtures/data.json"

SCRIPT="${WORKTREE}/scripts/check-crate-layout.sh"
tv3_output=$(WORKSPACE_ROOT="${TMP_WORKSPACE}" bash "${SCRIPT}" 2>&1)
tv3_exit=$?

if [ "${tv3_exit}" -ne 0 ]; then
  tap_pass "TV-3/EC-002: crate with tests/fixtures/ causes non-zero exit"
else
  tap_fail "TV-3/EC-002: crate with tests/fixtures/ causes non-zero exit" \
    "Expected non-zero; got 0. Output: ${tv3_output}"
fi

# ── Test 4: TV-3 violation message references the fixture placement rule ──────
if echo "${tv3_output}" | grep -qiE "tests/fixtures|fixtures/"; then
  tap_pass "TV-3/EC-002: violation message references fixture placement rule"
else
  tap_fail "TV-3/EC-002: violation message references fixture placement rule" \
    "Output did not mention 'tests/fixtures' or 'fixtures/'. Got: ${tv3_output}"
fi

tap_done
