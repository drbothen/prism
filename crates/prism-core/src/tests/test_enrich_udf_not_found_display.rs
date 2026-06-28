//! Regression tests for `EnrichUdfNotFoundDetails` Display output.
//!
//! HIGH-002/004 closure (S-DEMO-FIDELITY-REMEDIATION-001):
//! Asserts that the Display impl byte-matches the PO-reconciled canonical template:
//!
//!   E-QUERY-039: enrichment infusion '{infusion}' is not registered; available: [{available}]{did_you_mean}
//!
//! where:
//! - `{available}` = `available_infusions.join(", ")` wrapped in `[ ]` brackets (empty Vec → `[]`)
//! - `{did_you_mean}` = ` Did you mean: '{x}'?` (leading space) when Some, omitted when None
//!
//! TD-VSDD-059: load-bearing tests, not paper-fix. Each test drives the
//! `EnrichUdfNotFoundDetails::fmt` code path and asserts exact byte output.

use crate::error::{EnrichUdfNotFoundDetails, PrismError};

/// BC-2.11.019 v1.4 / HIGH-002/004 — canonical Display with available list and no
/// did_you_mean suffix.
///
/// Input: infusion="threat_intel", available=["threat_score", "threat_is_known_malicious"],
///        did_you_mean=None.
/// Expected: "E-QUERY-039: enrichment infusion 'threat_intel' is not registered;
///            available: [threat_score, threat_is_known_malicious]"
/// No trailing space, no "Did you mean" suffix.
#[test]
fn test_enrich_udf_not_found_display_no_did_you_mean() {
    let details = EnrichUdfNotFoundDetails::new(
        "threat_intel",
        vec![
            "threat_score".to_string(),
            "threat_is_known_malicious".to_string(),
        ],
        None,
    );
    let err = PrismError::EnrichUdfNotFound(Box::new(details));
    let display = format!("{err}");

    // Byte-exact match against PO canonical template.
    assert_eq!(
        display,
        "E-QUERY-039: enrichment infusion 'threat_intel' is not registered; \
         available: [threat_score, threat_is_known_malicious]",
        "HIGH-002/004: Display must byte-match canonical E-QUERY-039 template (no did_you_mean)"
    );
}

/// BC-2.11.019 v1.4 / HIGH-002/004 — canonical Display WITH did_you_mean suffix.
///
/// Input: infusion="thret_score", available=["threat_score"], did_you_mean=Some("threat_score").
/// Expected: "E-QUERY-039: enrichment infusion 'thret_score' is not registered;
///            available: [threat_score] Did you mean: 'threat_score'?"
/// Leading space before "Did you mean", no double space.
#[test]
fn test_enrich_udf_not_found_display_with_did_you_mean() {
    let details = EnrichUdfNotFoundDetails::new(
        "thret_score",
        vec!["threat_score".to_string()],
        Some("threat_score".to_string()),
    );
    let err = PrismError::EnrichUdfNotFound(Box::new(details));
    let display = format!("{err}");

    assert_eq!(
        display,
        "E-QUERY-039: enrichment infusion 'thret_score' is not registered; \
         available: [threat_score] Did you mean: 'threat_score'?",
        "HIGH-002/004: Display must byte-match canonical E-QUERY-039 template (with did_you_mean)"
    );
}

/// BC-2.11.019 v1.4 / HIGH-002/004 — empty available_infusions produces `[]` brackets.
///
/// Edge case: no infusions registered at all.
/// Expected: "E-QUERY-039: enrichment infusion 'anything' is not registered; available: []"
#[test]
fn test_enrich_udf_not_found_display_empty_available() {
    let details = EnrichUdfNotFoundDetails::new("anything", vec![], None);
    let err = PrismError::EnrichUdfNotFound(Box::new(details));
    let display = format!("{err}");

    assert_eq!(
        display, "E-QUERY-039: enrichment infusion 'anything' is not registered; available: []",
        "HIGH-002/004: Empty available_infusions must produce '[]' brackets"
    );
}

/// Structural assertion: Display starts with E-QUERY-039 code (prefix stability).
#[test]
fn test_enrich_udf_not_found_display_starts_with_error_code() {
    let details = EnrichUdfNotFoundDetails::new("x", vec!["y".to_string()], None);
    let err = PrismError::EnrichUdfNotFound(Box::new(details));
    let display = format!("{err}");

    assert!(
        display.starts_with("E-QUERY-039:"),
        "HIGH-002/004: Display must start with 'E-QUERY-039:' for structured log parsing. \
         Got: {display}"
    );
}
