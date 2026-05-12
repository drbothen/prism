#!/usr/bin/env bash
# check-non-exhaustive.sh — verify #[non_exhaustive] forward-compat enforcement.
#
# Mirrors the CI `non-exhaustive-violation-compile-fail` job for local pre-push parity.
# Violations are split across src/enum_violations.rs and src/struct_violations.rs so
# that rustc's per-file error budget does not suppress later violations.
# Uses --message-format=json to count ALL violations (not capped by per-file rustc limit).
#
# Update EXPECTED when adding/removing violations from enum_violations.rs or struct_violations.rs.
# (BC-2.01.013 AC-5 / F-LP2-OBS-001 S-PLUGIN-PREREQ-C)

EXPECTED=29
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(dirname "${SCRIPT_DIR}")"

echo "Verifying #[non_exhaustive] forward-compat enforcement (expected: ${EXPECTED} violations)..."

# cargo check exits non-zero (crate intentionally fails to compile). Capture json output.
cargo check \
    --message-format=json \
    --manifest-path "${WORKSPACE_ROOT}/tests/external/non-exhaustive-violation/Cargo.toml" \
    > /tmp/non-exhaustive-json.log 2>/dev/null \
    && CARGO_RC=0 || CARGO_RC=$?

if [ "${CARGO_RC}" -eq 0 ]; then
    echo "FAIL: non-exhaustive-violation compiled successfully — at least one"
    echo "  #[non_exhaustive] annotation was removed. All types must reject external"
    echo "  struct-literal or exhaustive-match construction (BC-2.01.013 AC-5)."
    rm -f /tmp/non-exhaustive-json.log
    exit 1
fi

# Count E0639 and E0004 errors from JSON output (all violations, uncapped by rustc limit).
TOTAL=$(python3 "${SCRIPT_DIR}/count-non-exhaustive-errors.py" /tmp/non-exhaustive-json.log)
rm -f /tmp/non-exhaustive-json.log

if [ "${TOTAL}" -lt "${EXPECTED}" ]; then
    echo "FAIL: Expected at least ${EXPECTED} E0639/E0004 errors, got ${TOTAL}."
    echo "  Some #[non_exhaustive] annotations may have been removed from:"
    echo "  tests/external/non-exhaustive-violation/src/struct_violations.rs (E0639)"
    echo "  tests/external/non-exhaustive-violation/src/enum_violations.rs (E0004)"
    exit 1
fi

echo "PASS: ${TOTAL} types correctly reject external construction (expected: ${EXPECTED})"
