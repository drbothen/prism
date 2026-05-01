//! RED tests for WGS-W2-001 (HIGH, CWE-943): AQL injection mitigation.
//!
//! Validates the new `validate_aql()` function and `build_aql()` pre-wire rejection
//! per ADR-005 (accepted).  Also covers the new `SensorError::ConfigValidation` variant
//! and TV-BC-2.01.008-006 (pre-wire `ConfigValidation` rejection).
//!
//! # RED gate
//!
//! All tests below FAIL on current code because:
//! - `validate_aql()` does not exist yet
//! - `SensorError::ConfigValidation` does not exist yet
//! - `build_aql()` forwards AQL verbatim without validation
//!
//! After implementation these tests become GREEN.
//!
//! Security fix: WGS-W2-001 | ADR: ADR-005 | BC: BC-2.01.008 (TV-006)

#![allow(clippy::expect_used, clippy::unwrap_used)]

use secrecy::SecretString;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use prism_sensors::adapter::{QueryParams, SensorError, SensorSpec};
use prism_sensors::auth::armis::{validate_aql, AqlValidationError, ArmisAdapter, ArmisAuth};
use prism_sensors::auth::SensorAuth;
use prism_sensors::{OrgId, SensorAdapter};

/// Returns a stable test `OrgId` for adapter constructor migration (AC-006).
///
/// Same value as `DEFAULT_ORG_ID_BYTES` in lib.rs; duplicated here because
/// `#[cfg(test)]` items in the library are not accessible from external
/// integration test crates.
fn test_org_id() -> OrgId {
    OrgId::from_uuid(uuid::Uuid::from_bytes([
        0x01, 0x8e, 0x3f, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_auth(instance_url: &str) -> ArmisAuth {
    ArmisAuth {
        instance_url: instance_url.to_string(),
        secret_key: SecretString::new("test-armis-secret-key".into()),
    }
}

fn make_spec_with_aql(table: &str, aql_query: &str) -> SensorSpec {
    #[allow(deprecated)]
    SensorSpec {
        org_id: test_org_id(), // Must match adapter's OrgId (BC-3.2.001 precondition 4)
        source_table: table.into(),
        client_id: "acme".into(),
        sensor_config: serde_json::json!({ "aql_query": aql_query }),
    }
}

#[allow(dead_code)]
fn make_spec_no_aql(table: &str) -> SensorSpec {
    #[allow(deprecated)]
    SensorSpec {
        org_id: test_org_id(), // Must match adapter's OrgId (BC-3.2.001 precondition 4)
        source_table: table.into(),
        client_id: "acme".into(),
        sensor_config: serde_json::json!({}),
    }
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.008-006 (Happy path): valid AQL passes validation
// ---------------------------------------------------------------------------

/// validate_aql() happy path: "in:devices" (default template form) → Ok(())
#[test]
fn test_WGS_W2_001_validate_aql_simple_in_devices_passes() {
    let result = validate_aql("in:devices");
    assert!(
        result.is_ok(),
        "WGS-W2-001: 'in:devices' must pass validation; got: {result:?}"
    );
}

/// validate_aql() happy path: "in:devices id:(1,2,3)" → Ok(())
#[test]
fn test_WGS_W2_001_validate_aql_in_with_id_list_passes() {
    let result = validate_aql("in:devices id:(1,2,3)");
    assert!(
        result.is_ok(),
        "WGS-W2-001: 'in:devices id:(1,2,3)' must pass validation; got: {result:?}"
    );
}

/// validate_aql() happy path: simple field predicate with equality
#[test]
fn test_WGS_W2_001_validate_aql_field_equals_predicate_passes() {
    let result = validate_aql("in:devices name:\"acme-corp\"");
    assert!(
        result.is_ok(),
        "WGS-W2-001: simple quoted string predicate must pass; got: {result:?}"
    );
}

/// validate_aql() happy path: integer filter value
#[test]
fn test_WGS_W2_001_validate_aql_integer_filter_passes() {
    let result = validate_aql("in:devices riskLevel:7");
    assert!(
        result.is_ok(),
        "WGS-W2-001: integer value filter must pass; got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Rejection: SQL injection patterns
// ---------------------------------------------------------------------------

/// validate_aql() rejection: SQL DROP TABLE is not AQL
#[test]
fn test_WGS_W2_001_validate_aql_sql_drop_table_rejected() {
    let result = validate_aql("DROP TABLE users");
    assert!(
        result.is_err(),
        "WGS-W2-001: 'DROP TABLE users' must be rejected by AQL validator"
    );
}

/// validate_aql() rejection: SQL comment injection via double-dash
#[test]
fn test_WGS_W2_001_validate_aql_comment_injection_double_dash_rejected() {
    let result = validate_aql("in:devices id:1 -- exfil comment");
    assert!(
        result.is_err(),
        "WGS-W2-001: double-dash comment injection must be rejected; got: {result:?}"
    );
}

/// validate_aql() rejection: block comment injection /* */
#[test]
fn test_WGS_W2_001_validate_aql_comment_injection_block_comment_rejected() {
    let result = validate_aql("in:devices /* inject */ id:1");
    assert!(
        result.is_err(),
        "WGS-W2-001: block comment /* */ injection must be rejected; got: {result:?}"
    );
}

/// validate_aql() rejection: stacked query via semicolon
#[test]
fn test_WGS_W2_001_validate_aql_stacked_query_semicolon_rejected() {
    let result = validate_aql("in:devices id:1; DROP TABLE users");
    assert!(
        result.is_err(),
        "WGS-W2-001: semicolon stacked query must be rejected; got: {result:?}"
    );
}

/// validate_aql() rejection: quote injection attempt
#[test]
fn test_WGS_W2_001_validate_aql_quote_injection_rejected() {
    let result = validate_aql("in:devices id:1\" OR \"a\"=\"a");
    assert!(
        result.is_err(),
        "WGS-W2-001: unbalanced quote injection must be rejected; got: {result:?}"
    );
}

/// validate_aql() rejection: nested in: sub-query injection
#[test]
fn test_WGS_W2_001_validate_aql_nested_in_subquery_rejected() {
    let result = validate_aql("in:devices id:(in:devices riskLevel:10)");
    assert!(
        result.is_err(),
        "WGS-W2-001: nested in: sub-query must be rejected; got: {result:?}"
    );
}

/// validate_aql() rejection: select sub-query marker
#[test]
fn test_WGS_W2_001_validate_aql_select_subquery_rejected() {
    let result = validate_aql("in:devices id:(select id from devices)");
    assert!(
        result.is_err(),
        "WGS-W2-001: select sub-query must be rejected; got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Rejection: empty / whitespace-only
// ---------------------------------------------------------------------------

/// validate_aql() rejection: empty string → Err
#[test]
fn test_WGS_W2_001_validate_aql_empty_string_rejected() {
    let result = validate_aql("");
    assert!(
        result.is_err(),
        "WGS-W2-001: empty string must be rejected by AQL validator"
    );
}

/// validate_aql() rejection: whitespace-only string → Err
#[test]
fn test_WGS_W2_001_validate_aql_whitespace_only_rejected() {
    let result = validate_aql("   ");
    assert!(
        result.is_err(),
        "WGS-W2-001: whitespace-only AQL must be rejected"
    );
}

// ---------------------------------------------------------------------------
// Rejection: excessive length
// ---------------------------------------------------------------------------

/// validate_aql() rejection: query exceeding 512-byte limit
#[test]
fn test_WGS_W2_001_validate_aql_exceeds_max_length_rejected() {
    let long_query = format!("in:devices name:\"{}\"", "x".repeat(500));
    let result = validate_aql(&long_query);
    assert!(
        result.is_err(),
        "WGS-W2-001: AQL query > 512 bytes must be rejected; length={}",
        long_query.len()
    );
}

// ---------------------------------------------------------------------------
// TV-BC-2.01.008-006: build_aql() rejects malicious AQL, no HTTP call issued
// ---------------------------------------------------------------------------

/// TV-BC-2.01.008-006: when sensor_config supplies a malicious AQL,
/// build_aql() returns Err(SensorError::ConfigValidation) and NO HTTP call is
/// attempted (mock server gets zero requests).
#[tokio::test]
async fn test_WGS_W2_001_build_aql_malicious_aql_returns_config_validation_no_http_call() {
    let server = MockServer::start().await;

    // Register a handler that MUST NOT be called
    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "results": [], "total": 0 }
        })))
        .expect(0) // zero calls expected
        .named("should_not_be_called")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ArmisAdapter::new(test_org_id(), &auth, SecretString::new("tok".into()));
    let spec = make_spec_with_aql("devices", "in:devices id:1; DROP TABLE users--");
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;

    assert!(
        result.is_err(),
        "TV-BC-2.01.008-006: malicious AQL must return Err"
    );
    match result {
        Err(SensorError::ConfigValidation { sensor, detail }) => {
            assert_eq!(
                sensor, "armis",
                "TV-BC-2.01.008-006: ConfigValidation must name sensor 'armis'"
            );
            assert!(
                !detail.is_empty(),
                "TV-BC-2.01.008-006: ConfigValidation detail must not be empty"
            );
        }
        Err(e) => panic!("TV-BC-2.01.008-006: Expected SensorError::ConfigValidation, got: {e:?}"),
        Ok(_) => panic!("TV-BC-2.01.008-006: Expected Err, got Ok"),
    }
    // wiremock verifies 0 calls to /api/v1/search on server drop
}

/// TV-BC-2.01.008-006 (passing AQL): valid spec-supplied AQL passes validation
/// and the HTTP call IS made (mock expects 1 call).
#[tokio::test]
async fn test_WGS_W2_001_build_aql_valid_aql_proceeds_to_http_call() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "results": [], "total": 0 }
        })))
        .expect(1) // exactly 1 call expected
        .named("valid_aql_call")
        .mount(&server)
        .await;

    let auth = make_auth(&server.uri());
    let adapter = ArmisAdapter::new(test_org_id(), &auth, SecretString::new("tok".into()));
    let spec = make_spec_with_aql("devices", "in:devices riskLevel:7");
    let params = QueryParams::default();

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;
    assert!(
        result.is_ok(),
        "TV-BC-2.01.008-006: valid AQL must proceed to HTTP call and succeed; got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// AqlValidationError struct is accessible
// ---------------------------------------------------------------------------

/// validate_aql() error type check: returned Err carries a non-empty reason.
#[test]
fn test_WGS_W2_001_aql_validation_error_carries_reason() {
    let err: AqlValidationError = validate_aql("").unwrap_err();
    assert!(
        !err.reason.is_empty(),
        "WGS-W2-001: AqlValidationError.reason must not be empty"
    );
}

/// validate_aql() error implements std::error::Error (via thiserror).
#[test]
fn test_WGS_W2_001_aql_validation_error_implements_std_error() {
    let err: AqlValidationError = validate_aql("DROP TABLE x").unwrap_err();
    // If this compiles, the trait bound is satisfied.
    let _: &dyn std::error::Error = &err;
}

// ---------------------------------------------------------------------------
// Pass 7 HIGH-002: select keyword — all-occurrences bypass fix
// ---------------------------------------------------------------------------

/// Pass 7 HIGH-002: "selected:y" satisfies the first-occurrence heuristic but
/// a subsequent "select:x" is the real keyword.  The validator MUST scan ALL
/// occurrences and reject because the second one is a standalone keyword.
///
/// Bug: `find("select")` stops at the first match inside "selected", which fails
/// the word-boundary next_ok check (`e` follows), so the validator fell through
/// to Ok(()).  Fix: `match_indices("select")` checks every occurrence.
#[test]
fn test_pass7_high_002_select_after_selected_word_bypass() {
    let result = validate_aql("in:devices selected:y or select:x");
    assert!(
        result.is_err(),
        "Pass7-HIGH-002: 'in:devices selected:y or select:x' must be rejected \
         (select keyword appears after selected field name); got: {result:?}"
    );
}

/// Pass 7 HIGH-002: standalone select keyword in third predicate position.
/// Ensures the fix covers cases where the keyword is not in position 2.
#[test]
fn test_pass7_high_002_select_in_third_position() {
    let result = validate_aql("in:devices a:1 b:2 select:x");
    assert!(
        result.is_err(),
        "Pass7-HIGH-002: 'in:devices a:1 b:2 select:x' must be rejected \
         (select keyword in third predicate position); got: {result:?}"
    );
}

/// Pass 7 HIGH-002: two standalone select keywords — both should trigger rejection.
/// Also regression-checks that the fix correctly identifies real keywords when
/// there is no misleading "selected"-style false positive first.
#[test]
fn test_pass7_high_002_double_select_both_keywords() {
    let result = validate_aql("in:devices select:a or select:b");
    assert!(
        result.is_err(),
        "Pass7-HIGH-002: 'in:devices select:a or select:b' must be rejected \
         (both select occurrences are keywords); got: {result:?}"
    );
}

/// Pass 7 HIGH-002 (negative case): "selected:y" alone (no second select) is
/// NOT a select keyword match — but it IS an unknown field not in the Armis
/// field allowlist.  This test documents the failure mode is field rejection,
/// NOT the select-keyword bypass.  The validator rejects it, just for a
/// different structural reason (does not start with a recognised operator/field).
///
/// NOTE: current validator does not implement a per-field allowlist beyond the
/// structural rules, so "selected:y" passes structural validation (in:, no SQL
/// keywords, balanced quotes, etc.).  This test therefore verifies the POSITIVE
/// case — "selected:y" alone passes all current structural gates because
/// "selected" ≠ standalone "select".
#[test]
fn test_pass7_high_002_word_with_select_substring_no_keyword() {
    // "selected" is NOT the keyword "select" — it must not trigger the select
    // keyword rejection.  The structural validator allows it.
    let result = validate_aql("in:devices selected:y");
    assert!(
        result.is_ok(),
        "Pass7-HIGH-002: 'in:devices selected:y' must PASS — 'selected' is not the \
         'select' keyword; got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Pass 7 HIGH-002: single-quote injection bypass fix
// ---------------------------------------------------------------------------

/// Pass 7 HIGH-002: single-quote OR-injection bypass.
/// `id:1' OR 'a'='a` uses single-quote breakout — the validator must reject
/// single-quote characters that appear in injection patterns.
#[test]
fn test_pass7_high_002_single_quote_breakout() {
    let result = validate_aql("in:devices id:1' OR 'a'='a");
    assert!(
        result.is_err(),
        "Pass7-HIGH-002: single-quote OR breakout must be rejected; got: {result:?}"
    );
}

/// Pass 7 HIGH-002: single-quote equality injection pattern.
/// `id:'='` is a quote-comparison injection analogous to `id:"="`.
#[test]
fn test_pass7_high_002_single_quote_equality() {
    let result = validate_aql("in:devices id:'='");
    assert!(
        result.is_err(),
        "Pass7-HIGH-002: single-quote equality injection ('=') must be rejected; got: {result:?}"
    );
}
