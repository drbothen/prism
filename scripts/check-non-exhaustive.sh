#!/usr/bin/env bash
# check-non-exhaustive.sh — verify #[non_exhaustive] forward-compat enforcement.
#
# Single source of truth for the non-exhaustive gate: ci.yml calls this script
# directly (bash scripts/check-non-exhaustive.sh) rather than duplicating logic
# inline. Local `just check` also calls this script, so CI and local runs are
# identical. (F-CSD-P29-OBS-001 — DEFECT-CSDEVICES-EMPTY-PIPELINE-001)
#
# Two-layer gate:
#   Layer 1 (count): total E0639+E0004 errors == EXPECTED (catches removed annotations and unregistered additions)
#   Layer 2 (per-symbol): every distinct expected symbol appears in error output
#     (catches net-zero regressions: one type loses annotation, another gains it)
#
# Violations are split across src/enum_violations.rs and src/struct_violations.rs so
# that rustc's per-file error budget does not suppress later violations.
# Uses --message-format=json to count ALL violations (not capped by per-file rustc limit).
#
# Update EXPECTED_SYMBOLS in check-non-exhaustive-per-symbol.py when adding/removing
# violations from enum_violations.rs or struct_violations.rs.
# EXPECTED count in this script is derived from that manifest automatically — do not edit EXPECTED here.
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
# DEFECT-CSDEVICES-EMPTY-PIPELINE-001 F-CSD-P28-OBS-001: bumped 89→90 for VirtualField (prism-core::virtual_fields E0004 v90).
# DEFECT-CSDEVICES-EMPTY-PIPELINE-001 F-CSD-P31-OBS-002: bumped 90→91 for VirtualField (prism_query::ast E0004 v91).
# DEFECT-PQL-FNCALL-LHS-001 F-PQLFN-PR11-OBS-002 (BC-2.11.019 §OBS-005): bumped 91→92 for ParseError (prism_query::error E0639 v92).

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(dirname "${SCRIPT_DIR}")"

# Derive EXPECTED count from the Python manifest (single source of truth).
# check-non-exhaustive-per-symbol.py --count prints len(EXPECTED_SYMBOLS).
# Fail closed if the manifest is missing, unexecutable, or returns non-integer output.
if ! EXPECTED="$(python3 "${SCRIPT_DIR}/check-non-exhaustive-per-symbol.py" --count)"; then
    echo "FAIL: could not read EXPECTED count from check-non-exhaustive-per-symbol.py — failing closed."
    exit 1
fi
case "${EXPECTED}" in
    ''|*[!0-9]*)
        echo "FAIL: EXPECTED count produced non-integer output '${EXPECTED}' — failing closed."
        exit 1
        ;;
esac

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

if [ "${TOTAL}" -ne "${EXPECTED}" ]; then
    echo "FAIL: Expected exactly ${EXPECTED} E0639/E0004 errors, got ${TOTAL}."
    if [ "${TOTAL}" -gt "${EXPECTED}" ]; then
        echo "  TOTAL > EXPECTED: an unregistered #[non_exhaustive] type was added to"
        echo "  tests/external/non-exhaustive-violation/. Append the new symbol to"
        echo "  EXPECTED_SYMBOLS in scripts/check-non-exhaustive-per-symbol.py."
    else
        echo "  TOTAL < EXPECTED: a #[non_exhaustive] annotation may have been removed."
        echo "  Layer 2 (per-symbol check) will identify the missing symbol(s)."
        echo "  tests/external/non-exhaustive-violation/src/struct_violations.rs (E0639)"
        echo "  tests/external/non-exhaustive-violation/src/enum_violations.rs (E0004)"
    fi
    exit 1
fi

echo "Layer 1 (count): PASS — ${TOTAL}/${EXPECTED} violations present."

# ── Layer 2: per-symbol check ──────────────────────────────────────────────
# Verifies each distinct expected type produces at least one E0639 or E0004
# error. Catches net-zero regressions (one type loses annotation, another
# gains it) that the count-only Layer 1 cannot detect.
# Expected symbol list is maintained in check-non-exhaustive-per-symbol.py
# (single source of truth for the symbol manifest).
echo "Layer 2 (per-symbol): verifying each expected type appears in error output..."
if ! python3 "${SCRIPT_DIR}/check-non-exhaustive-per-symbol.py" "${JSON_LOG}"; then
    echo "FAIL: per-symbol check failed — see above for missing symbols."
    exit 1
fi

echo "PASS: ${TOTAL}/${EXPECTED} violations present; all distinct expected symbols verified."
