//! Tests for S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B MCP mapping — BC-2.11.019.
//!
//! Finding N1-B: `map_prism_error(PrismError::EnrichUdfNotFound(...))` must return
//! `-32602` (INVALID_PARAMS), NOT `-32000` (INTERNAL_ERROR catch-all).
//!
//! `PrismError::EnrichUdfNotFound` is implemented (`prism-core/src/error.rs`,
//! `EnrichUdfNotFoundDetails`). The explicit E-QUERY-039 arm in `map_prism_error` maps
//! the variant to `-32602` (INVALID_PARAMS).
//!
//! Tests verify:
//! 1. `PrismError::UnknownSourceTable` (E-QUERY-036) still maps to `-32602` — regression
//!    guard that the existing arm was not accidentally broken.
//! 2. `PrismError::EnrichUdfNotFound` (E-QUERY-039) maps to `-32602` INVALID_PARAMS,
//!    NOT `-32000` INTERNAL_ERROR.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_bc_2_11_019_n1b_mcp_maps_to_32602 | AC-N1B | BC-2.11.019 |

use prism_core::{
    error::{EnrichUdfNotFoundDetails, PrismError},
    UnknownSourceTableDetails,
};
use prism_mcp::error_mapping::{codes, map_prism_error, prism_error_to_structured_call_result};

/// BC-2.11.019 AC-N1B — `map_prism_error` for E-QUERY-039.
///
/// Asserts that `PrismError::EnrichUdfNotFound` maps to `-32602` INVALID_PARAMS:
/// - Regression guard: `PrismError::UnknownSourceTable` (E-QUERY-036) maps to -32602.
/// - Direct assertion: `PrismError::EnrichUdfNotFound` (E-QUERY-039) maps to -32602,
///   NOT to -32000 (INTERNAL_ERROR catch-all).
#[test]
fn test_bc_2_11_019_n1b_mcp_maps_to_32602() {
    // ── Regression guard: existing E-QUERY-036 arm must map to -32602 ──
    // This assertion PASSES before the fix — it is a guard that the existing arm
    // is not accidentally broken when the E-QUERY-039 arm is added.
    let e036_err = PrismError::UnknownSourceTable(Box::new(UnknownSourceTableDetails::new(
        "ghost_sensor.table",
        vec!["crowdstrike".to_string()],
        Some("crowdstrike".to_string()),
    )));
    let (code_036, _) = map_prism_error(e036_err);
    assert_eq!(
        code_036,
        codes::INVALID_PARAMS,
        "BC-2.11.019 AC-N1B regression guard: E-QUERY-036 (UnknownSourceTable) must still \
         map to -32602 after the E-QUERY-039 arm is added. Got: {code_036}"
    );

    // ── Direct E-QUERY-039 assertion: EnrichUdfNotFound must map to -32602 ──
    let e039_err = PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails::new(
        "threat_intel",
        vec![
            "threat_score".to_string(),
            "threat_is_known_malicious".to_string(),
            "threat_sources".to_string(),
        ],
        None,
    )));
    let (code_039, message_039) = map_prism_error(e039_err);

    assert_eq!(
        code_039,
        codes::INVALID_PARAMS,
        "BC-2.11.019 AC-N1B: E-QUERY-039 (EnrichUdfNotFound) must map to \
         -32602 INVALID_PARAMS, NOT -32000 INTERNAL_ERROR. Got: {code_039}"
    );

    assert_ne!(
        code_039,
        codes::INTERNAL_ERROR,
        "BC-2.11.019 AC-N1B: E-QUERY-039 errors must NOT map to -32000 INTERNAL_ERROR. \
         Got message: {message_039}"
    );
}

// ─── MED-5: suggestion string byte-for-byte conformance to BC-2.11.019 §MCP surface ─────────

/// MED-5 BC-2.11.019 §MCP surface — non-empty suggestion form.
///
/// BC specifies the suggestion text (NO brackets, comma-joined list):
///   "Use one of the registered enrichment functions: {available_infusions}. Call
///    prism_describe('<client_id>') to see pql_hints including available enrichment functions."
///
/// The prior implementation wrapped the list in brackets: "Use one of the registered
/// enrichment functions: [threat_score, ...]. Call ..." — violating the BC §MCP surface.
/// MED-5 removes the brackets. This test pins the exact byte sequence.
///
/// RED GATE: if brackets are re-introduced ("Use one of the registered enrichment
/// functions: [{list}]. Call ..."), this test fails.
#[test]
fn test_med5_enrich_udf_not_found_suggestion_non_empty_no_brackets() {
    let err = PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails::new(
        "threat_intel",
        vec![
            "threat_score".to_string(),
            "threat_is_known_malicious".to_string(),
        ],
        None,
    )));

    let result = prism_error_to_structured_call_result(err);
    let sc = result
        .structured_content
        .expect("MED-5: structuredContent must be present");
    let suggestion = sc
        .get("error")
        .and_then(|e| e.get("suggestion"))
        .and_then(|v| v.as_str())
        .unwrap_or("<missing>");

    // BC-2.11.019 §MCP surface canonical text (no brackets around list):
    let expected = "Use one of the registered enrichment functions: \
                    threat_score, threat_is_known_malicious. \
                    Call prism_describe('<client_id>') to see pql_hints including available \
                    enrichment functions.";
    assert_eq!(
        suggestion, expected,
        "MED-5 BC-2.11.019 §MCP surface: suggestion must not wrap the list in brackets \
         and must follow the BC canonical template byte-for-byte.\n\
         Expected: {expected:?}\n\
         Got:      {suggestion:?}"
    );
}

/// MED-5 BC-2.11.019 §MCP surface — empty infusions form.
///
/// BC specifies:
///   "No enrichment functions are registered. Enrichment is not available in this deployment."
#[test]
fn test_med5_enrich_udf_not_found_suggestion_empty_infusions() {
    let err = PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails::new(
        "ghost_udf",
        vec![],
        None,
    )));

    let result = prism_error_to_structured_call_result(err);
    let sc = result
        .structured_content
        .expect("MED-5 empty: structuredContent must be present");
    let suggestion = sc
        .get("error")
        .and_then(|e| e.get("suggestion"))
        .and_then(|v| v.as_str())
        .unwrap_or("<missing>");

    let expected = "No enrichment functions are registered. \
                    Enrichment is not available in this deployment.";
    assert_eq!(
        suggestion, expected,
        "MED-5 BC-2.11.019 §MCP surface: empty-infusions suggestion must match BC text.\n\
         Expected: {expected:?}\n\
         Got:      {suggestion:?}"
    );
}
