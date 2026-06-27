//! Red Gate test for S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B MCP mapping — BC-2.11.019 v1.2.
//!
//! Finding N1-B: `map_prism_error(PrismError::EnrichUdfNotFound(...))` must return
//! `-32602` (INVALID_PARAMS), NOT `-32000` (INTERNAL_ERROR catch-all).
//!
//! Since `PrismError::EnrichUdfNotFound` does not exist yet (zero workspace matches
//! verified 2026-06-26), this test cannot directly construct the variant. Instead it
//! validates the MCP mapping through the observable `map_prism_error` behavior by
//! asserting that:
//!
//! 1. The `PrismError::TableNotAvailable` arm (E-QUERY-037, confirmed present) maps
//!    to `-32602` — regression guard that the existing arm is not accidentally broken
//!    when the E-QUERY-039 arm is added.
//! 2. After `EnrichUdfNotFound` is implemented, `map_prism_error` must return `-32602`.
//!
//! # Pre-implementation RED strategy
//!
//! A `compile_check_enrich_udf_not_found_variant_exists` test uses a Rust compile-time
//! assertion pattern: it calls `prism_core::error::PrismError::from_str` (which doesn't
//! exist) to force the test to FAIL at compile time once the variant exists and the
//! compile-check is updated. For the RED gate before implementation, we assert the
//! currently-absent variant WOULD map correctly by checking that:
//! - The catch-all `_` arm in `map_prism_error` returns `-32000`.
//! - A known E-QUERY error maps to `-32602`.
//! - A specific "gate exists" marker assertion fails (no "E-QUERY-039" arm present).
//!
//! The cleanest RED strategy: assert that `map_prism_error` for a known
//! `UnknownSourceTable` error (E-QUERY-036) maps to `-32602` (regression guard),
//! AND separately assert that the catch-all `-32000` path is NOT the only path
//! (by checking the known variants don't fall through). The E-QUERY-039 arm itself
//! cannot be asserted until the variant exists — the RED gate for THAT assertion is
//! in the companion engine tests (`test_bc_2_11_019_n1b_infusion_id_as_udf_name`).
//!
//! However, we CAN write a test that will FAIL RED by asserting a property that the
//! current code does NOT satisfy: if we call `map_prism_error` with a variant that
//! currently falls through to `-32000` and assert it returns `-32602`, that will fail.
//!
//! Strategy: use `PrismError::InfusionError(...)` which currently maps to the catch-all
//! `-32000` arm, and assert it should return `-32602` — this FAILS RED (correct).
//! The implementer's job: replace this test once `EnrichUdfNotFound` exists by adding
//! the explicit E-QUERY-039 arm and updating this test to use the real variant.
//!
//! # NOTE TO IMPLEMENTER
//!
//! Once `PrismError::EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>)` is added to
//! `prism-core/src/error.rs`, REPLACE this test with the canonical form:
//! ```rust
//! let err = PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails {
//!     infusion: "threat_intel".to_string(),
//!     available_infusions: vec!["threat_score".to_string()],
//!     did_you_mean: None,
//! }));
//! let (code, message) = map_prism_error(err);
//! assert_eq!(code, codes::INVALID_PARAMS, "EnrichUdfNotFound must map to -32602");
//! assert!(!message.contains("Internal error"), "must not fall through to catch-all");
//! ```
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_bc_2_11_019_n1b_mcp_maps_to_32602 | AC-N1B | BC-2.11.019 v1.2 |

use prism_core::{
    error::{EnrichUdfNotFoundDetails, PrismError},
    UnknownSourceTableDetails,
};
use prism_mcp::error_mapping::{codes, map_prism_error};

/// BC-2.11.019 v1.2 AC-N1B — `map_prism_error` for E-QUERY-039 Red Gate test.
///
/// This test asserts that `PrismError::EnrichUdfNotFound` maps to `-32602` INVALID_PARAMS.
///
/// # Red Gate approach (pre-implementation)
///
/// Since `PrismError::EnrichUdfNotFound` doesn't exist yet, we assert it indirectly:
/// - Regression guard: `PrismError::UnknownSourceTable` (E-QUERY-036) maps to -32602 (PASSES).
/// - RED assertion: `PrismError::InfusionError(InfusionError::NotFound {...})` currently
///   falls through to the catch-all `-32000` arm. We assert it must return `-32602`.
///   This FAILS RED because no explicit arm exists for `InfusionError`.
///   The implementer adds the `EnrichUdfNotFound` variant + arm in a single fix.
///   Once the `EnrichUdfNotFound` arm is present, the test is updated to use the real
///   variant (see NOTE TO IMPLEMENTER above). This ensures the RED gate is load-bearing.
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
    // Per NOTE TO IMPLEMENTER: replace proxy InfusionError assertion with direct
    // EnrichUdfNotFound variant now that the variant exists.
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
