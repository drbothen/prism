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
# S-DEMO-MULTI-TENANT-DTU-001: bumped 52→60 for MultiInstanceConfig, InstanceEntry, DemoBindError,
#   MultiInstanceBindError (U-006), MultiInstanceServers (D-1075-API-GAP-001), MultiInstanceHarness,
#   HarnessEntry, BindError.
# S-5.02: bumped 60→61 for prism_mcp::error_mapping::StructuredErrorFields (BC-2.10.007 v1.5 9-field struct).
# S-5.02 follow-up fix-burst: bumped 61→64 for prism_mcp::{CapabilityEntry, ResolutionStep} (struct literals)
#   and prism_mcp::CapabilityStatus (enum match) — CRIT-1/HIGH-1 non-exhaustive gate sibling-sweep.
# S-3.13 (LOW-1 + CR-002): bumped 64→66 for TableNotAvailableDetails (prism-core::error) + TableRegistry (prism-query).
# S-1.14-REDO burst-2 (MED-1-RESIDUAL): bumped 66→67 for Tier3CacheEntry (prism-spec-engine::infusion::cache).
# S-1.14-REDO fix-burst (architect-ruled FIX-IN-SCOPE): bumped 67→69 for InfusionUdfDescriptor + EnrichStageDescriptor.
# S-1.14-REDO adversarial OBS-1 FIX-IN-SCOPE: bumped 69→70 for InfusionError (prism-core::error enum).
# S-5.03 (F-007 process-gap): bumped 70→76 for 6 prism-mcp resources types (ClientInventoryEntry,
#   SensorConfigEntry, SensorHealthResult, RateLimitInfo, ResourcePressure, SensorHealthStructuredContent).
# S-DEMO-ENRICHMENT-PIVOT-002 v1.3: bumped 76→79 for 3 http_lookup infusion types
#   (HttpLookupCredentialConfig E0639 v77, HttpLookupConfig E0639 v78, HttpLookupAuthType E0004 v79).
# S-DEMO-PRISMQL-ONBOARDING-001-A: bumped 79→82 for 3 prism_describe response types
#   (PrismDescribeResponse E0639 v80, TableDescriptor E0639 v81, ColumnDescriptor E0639 v82).
# S-DEMO-PRISMQL-ONBOARDING-001-B: bumped 82→83 for ColumnNotFoundDetails (prism-core::error E0639 v83).
# S-5.04 F-S504-P5-002: bumped 83→84 for HealthSummary (prism_mcp::resources E0639 v84).
# S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001: bumped 84→87 for ExampleKind (E0004), SqlPipeQuery (E0639),
#   UnknownSourceTableDetails (E0639) — Groups 1+3.
# S-DEMO-FIDELITY-REMEDIATION-001: bumped 87→88 for EnrichUdfNotFoundDetails (prism-core::error E0639 AC-N1B).
# S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: bumped 88→89 for TemporalLiteralPosition (prism-core::error E0004 v86).

EXPECTED=89
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
