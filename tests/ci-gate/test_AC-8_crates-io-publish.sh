#!/usr/bin/env bash
# AC-8: cargo publish invoked for all crates using CRATES_IO_TOKEN, gated on build success.
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

assert_file_exists "$REL_YML" "AC-8"

# crates-io-publish job must exist.
if grep -qF "crates-io-publish:" "$REL_YML" 2>/dev/null; then
  tap_pass "AC-8: crates-io-publish job defined in release.yml"
else
  tap_fail "AC-8: crates-io-publish job missing from release.yml" \
    "AC-8 FAIL: expected 'crates-io-publish:' job in release.yml"
fi

# Must have 'needs: build-release' (gate: only publish if builds passed).
if grep -qE 'needs:.*build-release' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-8: crates-io-publish has 'needs: build-release' gate"
else
  tap_fail "AC-8: crates-io-publish missing 'needs: build-release'" \
    "AC-8 FAIL: crates publish must be gated on all matrix builds passing (needs: build-release)"
fi

# cargo publish must be a real run step (not echo).
if grep -qE '^\s+run:\s+cargo publish' "$REL_YML" 2>/dev/null; then
  tap_pass "AC-8: 'run: cargo publish' is a real step"
else
  tap_fail "AC-8: 'run: cargo publish' missing as real step" \
    "AC-8 FAIL: expected 'run: cargo publish -p prism-core' — found only TODO echo"
fi

# CRATES_IO_TOKEN must be referenced via secrets (not hardcoded).
if grep -qF "CRATES_IO_TOKEN" "$REL_YML" 2>/dev/null; then
  tap_pass "AC-8: CRATES_IO_TOKEN referenced in release.yml"
else
  tap_fail "AC-8: CRATES_IO_TOKEN not referenced" \
    "AC-8 FAIL: crates-io-publish must use secrets.CRATES_IO_TOKEN"
fi

# prism-core must be the first published crate (dependency order).
if grep -qF "prism-core" "$REL_YML" 2>/dev/null; then
  tap_pass "AC-8: 'prism-core' referenced in publish steps"
else
  tap_fail "AC-8: 'prism-core' not referenced in publish steps" \
    "AC-8 FAIL: cargo publish must include prism-core in dependency-safe order"
fi

tap_done
