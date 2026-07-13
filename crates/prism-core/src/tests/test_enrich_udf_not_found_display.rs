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

/// BC-2.11.019 / HIGH-002/004 — canonical Display with available list and no
/// did_you_mean suffix.
///
/// Input: infusion="threat_intel", available=["threat_score", "threat_is_known_malicious"],
///        did_you_mean=None.
/// Expected output (sorted): "E-QUERY-039: enrichment infusion 'threat_intel' is not registered;
///            available: [threat_is_known_malicious, threat_score]"
/// No trailing space, no "Did you mean" suffix.
///
/// Note: "threat_is_known_malicious" < "threat_score" lexicographically, so sorted order is
/// [threat_is_known_malicious, threat_score]. Display MUST sort per BC-2.11.019 §PrismError-variant.
/// (F-PBL1-LOW-002 fix: updated expected output to reflect self-sorting in Display.)
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

    // Byte-exact match against PO canonical template (sorted lexicographically).
    // "threat_is_known_malicious" < "threat_score" alphabetically.
    assert_eq!(
        display,
        "E-QUERY-039: enrichment infusion 'threat_intel' is not registered; \
         available: [threat_is_known_malicious, threat_score]",
        "HIGH-002/004: Display must byte-match canonical E-QUERY-039 template \
         (sorted: threat_is_known_malicious before threat_score)"
    );
}

/// BC-2.11.019 / HIGH-002/004 — canonical Display WITH did_you_mean suffix.
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

/// BC-2.11.019 / HIGH-002/004 — empty available_infusions produces `[]` brackets.
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

/// F-PBL1-LOW-002 — Display must self-sort `available_infusions` lexicographically
/// per BC-2.11.019 §PrismError-variant.
///
/// Previously the sort lived only in the gate caller (`check_enrich_udf_availability`).
/// But the Display contract says "the Display implementation MUST comma-join the
/// Vec<String> (sorted lexicographically)," which is a self-enforcement requirement.
///
/// This test passes an UNSORTED input `["z_infusion", "a_infusion"]` directly to
/// the Display impl and asserts the output shows them sorted `a_infusion, z_infusion`.
///
/// Before fix: Display joins verbatim → `[z_infusion, a_infusion]` → FAIL.
/// After fix: Display sorts a clone → `[a_infusion, z_infusion]` → PASS.
///
/// Load-bearing (F-PBL1-LOW-002): removing the sort from within Display causes
/// this test to fail (output would be `[z_infusion, a_infusion]`).
#[test]
fn test_f_pbl1_low002_display_self_sorts_available_infusions() {
    let details = EnrichUdfNotFoundDetails::new(
        "unknown",
        vec!["z_infusion".to_string(), "a_infusion".to_string()], // intentionally unsorted
        None,
    );
    let err = PrismError::EnrichUdfNotFound(Box::new(details));
    let display = format!("{err}");

    // The Display impl MUST sort the available_infusions before joining.
    // Unsorted input [z_infusion, a_infusion] must produce sorted [a_infusion, z_infusion].
    assert_eq!(
        display,
        "E-QUERY-039: enrichment infusion 'unknown' is not registered; \
         available: [a_infusion, z_infusion]",
        "F-PBL1-LOW-002: Display must sort available_infusions before joining. \
         Before fix, joins verbatim → [z_infusion, a_infusion]; \
         after fix, sorts first → [a_infusion, z_infusion]."
    );
}

/// F-PQLFN-P7-LOW-001 — SEC-001 (CWE-117): `EnrichUdfNotFoundDetails::new` must strip
/// Unicode Cc control characters (and U+2028/U+2029 line/paragraph separators) from the
/// `infusion` field at construction time.
///
/// The `infusion` field comes from analyst-provided query text (e.g., the UDF name in
/// `| enrich <name>(col)` or `WHERE <name>(col) = val`). An analyst could embed C1
/// control characters or line separators in a UDF name to inject log-splitting sequences
/// into agent-consumed structured logs (AD-017 extension, CWE-117).
///
/// This test mirrors the sanitize_for_log behaviour verified for `ColumnNotFoundDetails::new`
/// (see `ColumnNotFoundDetails` doc, SEC-001 parity).
///
/// Load-bearing (F-PQLFN-P7-LOW-001): removing `sanitize_for_log` from
/// `EnrichUdfNotFoundDetails::new` causes this test to fail — the stored field would
/// contain the raw control character instead of the stripped string.
///
/// Traces to: F-PQLFN-P7-LOW-001 (SEC-001 sibling parity); CWE-117; AD-017.
#[test]
fn test_f_pqlfn_p7_low_001_enrich_udf_infusion_cc_stripped_at_construction() {
    use crate::error::sanitize_for_log;

    // Build a UDF name embedding a C1 control char (U+0085 NEL) and a line separator
    // (U+2028). These are valid Unicode code points but log-injection vectors.
    let raw_name = "threat\u{0085}intel\u{2028}score";
    let expected_stored = sanitize_for_log(raw_name); // "threatintelscore"

    let details = EnrichUdfNotFoundDetails::new(raw_name, vec![], None);

    // Assert the stored field is the sanitized value — control chars stripped.
    assert_eq!(
        details.infusion, expected_stored,
        "F-PQLFN-P7-LOW-001: EnrichUdfNotFoundDetails::new must strip Cc/U+2028/U+2029 \
         from the infusion field at construction (SEC-001, CWE-117). \
         Raw input: {:?}. Expected stored: {:?}. Got: {:?}",
        raw_name, expected_stored, details.infusion
    );

    // Belt-and-suspenders: the stored value must NOT contain the raw control chars.
    assert!(
        !details.infusion.contains('\u{0085}'),
        "F-PQLFN-P7-LOW-001: stored infusion must not contain U+0085 (NEL). \
         Got: {:?}",
        details.infusion
    );
    assert!(
        !details.infusion.contains('\u{2028}'),
        "F-PQLFN-P7-LOW-001: stored infusion must not contain U+2028 (LINE SEPARATOR). \
         Got: {:?}",
        details.infusion
    );
}
