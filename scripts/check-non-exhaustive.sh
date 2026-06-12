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
# S-DEMO-DTU-LIVE-SCENARIO-001-A: bumped 49→50 for ScenarioEntityCatalog (AC-014, ADR-036 §2.2).
# S-DEMO-DTU-LIVE-SCENARIO-001-B: bumped 50→52 for IncidentTimeline + IncidentStage (AC-014 pattern, BPRL-P3-01 sibling sweep).

EXPECTED=52
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(dirname "${SCRIPT_DIR}")"

# Per-run temp path (mktemp) so concurrent `just check` runs in parallel
# worktrees do not clobber each other's evidence log. OUTPUT_LOG env override
# is supported for debugging / failure injection; an override path is
# caller-owned and is NOT cleaned up by this script.
if [ -n "${OUTPUT_LOG:-}" ]; then
    JSON_LOG="${OUTPUT_LOG}"
else
    JSON_LOG="$(mktemp "${TMPDIR:-/tmp}/non-exhaustive-json.XXXXXX")" || {
        echo "FAIL: mktemp could not create a temp log file — failing closed."
        exit 1
    }
    trap 'rm -f "${JSON_LOG}"' EXIT
fi

echo "Verifying #[non_exhaustive] forward-compat enforcement (expected: ${EXPECTED} violations)..."

# cargo check exits non-zero (crate intentionally fails to compile). Capture json output.
cargo check \
    --message-format=json \
    --manifest-path "${WORKSPACE_ROOT}/tests/external/non-exhaustive-violation/Cargo.toml" \
    > "${JSON_LOG}" 2>/dev/null \
    && CARGO_RC=0 || CARGO_RC=$?

if [ "${CARGO_RC}" -eq 0 ]; then
    echo "FAIL: non-exhaustive-violation compiled successfully — at least one"
    echo "  #[non_exhaustive] annotation was removed. All types must reject external"
    echo "  struct-literal or exhaustive-match construction (BC-2.01.013 AC-5)."
    exit 1
fi

# Fail CLOSED: a gate whose evidence log is missing or empty must FAIL, not
# pass with an empty count (TD-VSDD-059 false-pass class).
if [ ! -s "${JSON_LOG}" ]; then
    echo "FAIL: cargo json evidence log missing or empty at ${JSON_LOG}."
    echo "  Cannot verify #[non_exhaustive] enforcement without evidence — failing closed."
    exit 1
fi

# Count E0639 and E0004 errors from JSON output (all violations, uncapped by rustc limit).
if ! TOTAL="$(python3 "${SCRIPT_DIR}/count-non-exhaustive-errors.py" "${JSON_LOG}")"; then
    echo "FAIL: count-non-exhaustive-errors.py exited non-zero — failing closed."
    exit 1
fi

# Validate the count is a non-empty integer BEFORE the -lt comparison; a bash
# integer-expression error in [ ] evaluates false and would fall through to PASS.
case "${TOTAL}" in
    ''|*[!0-9]*)
        echo "FAIL: error counter produced non-integer output '${TOTAL}' — failing closed."
        exit 1
        ;;
esac

if [ "${TOTAL}" -lt "${EXPECTED}" ]; then
    echo "FAIL: Expected at least ${EXPECTED} E0639/E0004 errors, got ${TOTAL}."
    echo "  Some #[non_exhaustive] annotations may have been removed from:"
    echo "  tests/external/non-exhaustive-violation/src/struct_violations.rs (E0639)"
    echo "  tests/external/non-exhaustive-violation/src/enum_violations.rs (E0004)"
    exit 1
fi

echo "PASS: ${TOTAL} types correctly reject external construction (expected: ${EXPECTED})"
