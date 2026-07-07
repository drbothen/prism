//! F-CRIT-001 (LOCAL pass-5): byte-for-byte Display assertions for
//! `PrismError::QueryTypeMismatch`.
//!
//! ## The defect
//!
//! The `#[error]` template in `crates/prism-core/src/error.rs` currently reads:
//!
//! ```text
//! "E-QUERY-002: type mismatch — column '{column}' in table '{table}' has type \
//!  '{actual_type:?}' which does not support operator '{operator}{}",
//! SuggestedSuffix(suggested_column)
//! ```
//!
//! The template ends `'{operator}{}` — opening quote `'` before `{operator}`,
//! then immediately `{}` (the positional arg for `SuggestedSuffix`), with NO
//! closing `'` between `{operator}` and `{}`.
//!
//! Correct template (after fix): `'{operator}'{}`
//!
//! ## Red Gate behaviour at HEAD b2e3892c
//!
//! | Test | Actual (buggy) | Expected (correct) | Failure site |
//! |------|----------------|---------------------|--------------|
//! | with_suggestion_exact | `operator 'IEQ; for label...` | `operator 'IEQ'; for label...` | missing `'` before `;` |
//! | without_suggestion_exact | `operator 'IEQ` | `operator 'IEQ'` | missing closing `'` |
//!
//! Both tests FAIL at HEAD b2e3892c because `assert_eq!` detects the missing quote.
//! Both tests PASS after the template is corrected to `'{operator}'{}`.
//!
//! ## Traces
//!
//! - BC-2.11.024 §AC-022 — structured E-QUERY-002 message with suggestion suffix
//! - error-taxonomy v2.19 §E-QUERY-002 — normative Display form
//! - POL-24 — error taxonomy compliance enforced by full-string equality, not `.contains()`
//! - S-PRISMQL-CASE-INSENSITIVE-001 LOCAL adversary pass-5 finding F-CRIT-001

use prism_core::{column::ColumnType, PrismError};

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_query_type_mismatch_display_with_suggestion_exact
// ─────────────────────────────────────────────────────────────────────────────

/// F-CRIT-001: `QueryTypeMismatch` Display with `suggested_column: Some("severity")`
/// must be byte-for-byte identical to the error-taxonomy v2.19 §E-QUERY-002
/// normative form.
///
/// ## Correct expected output
///
/// ```text
/// E-QUERY-002: type mismatch — column 'severity_id' in table 'crowdstrike_detections'
/// has type 'Integer' which does not support operator 'IEQ'; for label comparison,
/// use the string column 'severity' with IEQ/IIN/INE instead
/// ```
///
/// The critical byte is `'` immediately after `IEQ`, before `;`.
///
/// ## Red Gate reason (HEAD b2e3892c)
///
/// The `#[error]` template is `'{operator}{}` — the closing `'` after `{operator}`
/// is absent.  `SuggestedSuffix(Some("severity"))` renders as
/// `"; for label comparison..."` so the full output starts with `'IEQ;` (no quote
/// before `;`), not `'IEQ';` (quote present).  The `assert_eq!` fails because
/// the actual string and the expected string diverge at that position.
///
/// ## Traces
///
/// BC-2.11.024 §AC-022; error-taxonomy v2.19 §E-QUERY-002; POL-24.
#[test]
fn test_BC_2_11_024_query_type_mismatch_display_with_suggestion_exact() {
    let err = PrismError::QueryTypeMismatch {
        column: "severity_id".to_string(),
        table: "crowdstrike_detections".to_string(),
        actual_type: ColumnType::Integer,
        operator: "IEQ".to_string(),
        suggested_column: Some("severity".to_string()),
    };
    assert_eq!(
        err.to_string(),
        "E-QUERY-002: type mismatch \u{2014} column 'severity_id' in table \
         'crowdstrike_detections' has type 'Integer' which does not support operator \
         'IEQ'; for label comparison, use the string column 'severity' with \
         IEQ/IIN/INE instead",
        "F-CRIT-001: QueryTypeMismatch Display with Some(suggested_column) must be \
         byte-for-byte identical to error-taxonomy v2.19 \u{00A7}E-QUERY-002 normative form. \
         The closing single-quote after the operator name is required before the suffix. \
         Template 'operator {{operator}}{{}}' is missing that quote \
         (HEAD b2e3892c error.rs line ~886)."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// test_BC_2_11_024_query_type_mismatch_display_without_suggestion_exact
// ─────────────────────────────────────────────────────────────────────────────

/// F-CRIT-001: `QueryTypeMismatch` Display with `suggested_column: None` must be
/// byte-for-byte identical to the error-taxonomy v2.19 §E-QUERY-002 no-suffix form.
///
/// ## Correct expected output
///
/// ```text
/// E-QUERY-002: type mismatch — column 'severity_id' in table 'crowdstrike_detections'
/// has type 'Integer' which does not support operator 'IEQ'
/// ```
///
/// The closing `'` after `IEQ` is required even when there is no suffix.
///
/// ## Red Gate reason (HEAD b2e3892c)
///
/// The `#[error]` template is `'{operator}{}` — with `SuggestedSuffix(None)`
/// rendering as `""`, the full output ends with `'IEQ` (no closing quote).
/// The `assert_eq!` fails because the actual string ends at `'IEQ` but the
/// expected string ends at `'IEQ'`.
///
/// ## Traces
///
/// BC-2.11.024 §AC-022; error-taxonomy v2.19 §E-QUERY-002; POL-24.
#[test]
fn test_BC_2_11_024_query_type_mismatch_display_without_suggestion_exact() {
    let err = PrismError::QueryTypeMismatch {
        column: "severity_id".to_string(),
        table: "crowdstrike_detections".to_string(),
        actual_type: ColumnType::Integer,
        operator: "IEQ".to_string(),
        suggested_column: None,
    };
    assert_eq!(
        err.to_string(),
        "E-QUERY-002: type mismatch \u{2014} column 'severity_id' in table \
         'crowdstrike_detections' has type 'Integer' which does not support operator \
         'IEQ'",
        "F-CRIT-001: QueryTypeMismatch Display with None suggested_column must be \
         byte-for-byte identical to error-taxonomy v2.19 \u{00A7}E-QUERY-002 no-suffix form. \
         The closing single-quote after the operator name is required even with no suffix. \
         Template 'operator {{operator}}{{}}' with SuggestedSuffix(None)='' renders as \
         '...operator \'IEQ' \u{2014} missing the closing quote \
         (HEAD b2e3892c error.rs line ~886)."
    );
}
