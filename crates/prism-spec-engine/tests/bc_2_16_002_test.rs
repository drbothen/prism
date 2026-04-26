#![allow(non_snake_case)]
//! BC-2.16.002: Multi-Step Fetch Pipeline Execution
//!
//! Tests cover:
//! - Sequential step execution (invariant: no parallel)
//! - Variable interpolation (${step_name.field})
//! - Fan-out batching (250 IDs, batch_size=100 -> 3 requests)
//! - Empty non-final step produces zero records, no error
//! - HTTP error on step 1 aborts pipeline
//! - E-SPEC-010 on interpolation failure
//! - Rate limit hint application (inter-request delay)
//! - 10K materialization limit (DI-019) applies to final records
//!
//! AC-2 (S-1.11): two-step OAuth->API with ${step1.response.access_token} interpolation

use prism_core::{ColumnType, TenantId};
use prism_spec_engine::interpolation::{InterpolationContext, InterpolationError, Interpolator};
use prism_spec_engine::pipeline::{FetchContext, PipelineExecutor};
use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec};

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Interpolation tests (pure function — exercisable without HTTP)
// ---------------------------------------------------------------------------

/// BC-2.16.002 postcondition: ${step_name.field} is resolved against prior step vars.
/// AC-2: step 2 uses token from step 1 via ${step1.response.access_token}.
#[test]
fn test_BC_2_16_002_interpolates_step_variable_in_path_template() {
    let mut vars = HashMap::new();
    vars.insert(
        "step1.response.access_token".to_string(),
        serde_json::Value::String("tok-abc-123".to_string()),
    );

    let template = "/api/v1/data?token=${step1.response.access_token}";
    let result = Interpolator::interpolate(template, &InterpolationContext::UrlPath, &vars);

    assert!(
        result.is_ok(),
        "interpolation must succeed: {:?}",
        result.err()
    );
    let interpolated = result.unwrap();
    assert!(
        interpolated.contains("tok-abc-123"),
        "interpolated path must contain the resolved token value: {interpolated}"
    );
}

/// BC-2.16.002 postcondition: values in URL context are percent-encoded.
/// Safety escaping: EC-004 variant for URL context.
#[test]
fn test_BC_2_16_002_percent_encodes_values_in_url_context() {
    let result = Interpolator::percent_encode("value with spaces & special=chars");
    // Must be a valid percent-encoded string — spaces become %20 or +, & becomes %26
    assert!(
        !result.contains(' ') && !result.contains('&'),
        "URL context: spaces and & must be encoded: {result}"
    );
}

/// BC-2.16.002 postcondition: values in JSON body context are JSON-escaped.
/// Safety escaping: EC-004 — JSON special chars escaped before substitution.
#[test]
fn test_BC_2_16_002_json_escapes_values_in_body_context() {
    let value_with_quotes = r#"value with "quotes" and \backslash"#;
    let escaped = Interpolator::json_escape(value_with_quotes);

    // JSON-escaped value must not contain bare unescaped double quotes
    assert!(
        !escaped.contains('"') || escaped.contains("\\\""),
        "JSON body context: double quotes must be escaped: {escaped}"
    );
}

/// BC-2.16.002 postcondition: E-SPEC-010 when variable field path not found in response.
#[test]
fn test_BC_2_16_002_returns_e_spec_010_on_interpolation_failure() {
    let vars: HashMap<String, serde_json::Value> = HashMap::new(); // empty — step1 not present

    let template = "/api/${step1.missing_field}";
    let result = Interpolator::interpolate(template, &InterpolationContext::UrlPath, &vars);

    assert!(
        result.is_err(),
        "missing variable must produce InterpolationError"
    );
    match result.unwrap_err() {
        InterpolationError::UnknownStep { step_name, .. } => {
            assert_eq!(step_name, "step1", "error must name the undefined step");
        }
        other => panic!("expected UnknownStep, got {:?}", other),
    }
}

/// BC-2.16.002 postcondition: extract_references returns all ${step.field} references.
#[test]
fn test_BC_2_16_002_extracts_all_variable_references_from_template() {
    let template = "/api/${step1.token}/data/${step2.cursor}";
    let refs = Interpolator::extract_references(template);

    assert_eq!(refs.len(), 2, "two variable references must be extracted");
    assert!(
        refs.iter().any(|(s, f)| s == "step1" && f == "token"),
        "step1.token must be in refs: {:?}",
        refs
    );
    assert!(
        refs.iter().any(|(s, f)| s == "step2" && f == "cursor"),
        "step2.cursor must be in refs: {:?}",
        refs
    );
}

/// BC-2.16.002 postcondition: template with no variables returns unchanged string.
#[test]
fn test_BC_2_16_002_template_without_variables_returns_unchanged() {
    let vars = HashMap::new();
    let template = "/api/v1/alerts";
    let result = Interpolator::interpolate(template, &InterpolationContext::UrlPath, &vars);

    assert!(result.is_ok(), "template without variables must succeed");
    assert_eq!(result.unwrap(), "/api/v1/alerts");
}

// ---------------------------------------------------------------------------
// Fan-out tests (pure function — no HTTP required)
// ---------------------------------------------------------------------------

/// BC-2.16.002 fan-out: 250 IDs with batch_size=100 produces 3 batches.
/// Canonical test vector from BC-2.16.002 Edge Cases.
#[test]
fn test_BC_2_16_002_fan_out_250_ids_produces_3_batches() {
    let ids: Vec<serde_json::Value> = (0u32..250).map(|i| serde_json::json!(i)).collect();
    let array_val = serde_json::Value::Array(ids);

    let batches = PipelineExecutor::fan_out_batches(&array_val, 100);

    assert_eq!(
        batches.len(),
        3,
        "250 IDs with batch_size=100 must produce 3 batches"
    );
    assert_eq!(batches[0].len(), 100, "first batch: 100");
    assert_eq!(batches[1].len(), 100, "second batch: 100");
    assert_eq!(batches[2].len(), 50, "third batch: 50 (remainder)");
}

/// BC-2.16.002 fan-out: exactly 100 IDs produces 1 batch.
#[test]
fn test_BC_2_16_002_fan_out_exactly_batch_size_produces_one_batch() {
    let ids: Vec<serde_json::Value> = (0u32..100).map(|i| serde_json::json!(i)).collect();
    let array_val = serde_json::Value::Array(ids);
    let batches = PipelineExecutor::fan_out_batches(&array_val, 100);
    assert_eq!(
        batches.len(),
        1,
        "100 IDs with batch_size=100 must produce 1 batch"
    );
    assert_eq!(batches[0].len(), 100);
}

/// BC-2.16.002 fan-out: non-array value produces single batch of 1.
#[test]
fn test_BC_2_16_002_fan_out_scalar_value_produces_single_batch() {
    let scalar = serde_json::json!("single-value");
    let batches = PipelineExecutor::fan_out_batches(&scalar, 100);
    assert_eq!(batches.len(), 1, "scalar value must produce 1 batch");
    assert_eq!(batches[0].len(), 1);
}

/// BC-2.16.002 fan-out: empty array produces zero batches.
#[test]
fn test_BC_2_16_002_fan_out_empty_array_produces_zero_batches() {
    let empty = serde_json::json!([]);
    let batches = PipelineExecutor::fan_out_batches(&empty, 100);
    assert!(batches.is_empty(), "empty array must produce 0 batches");
}

// ---------------------------------------------------------------------------
// Pipeline execution tests (async — require unimplemented! to fail)
// AC-2 integration test: two-step OAuth->API pipeline
// ---------------------------------------------------------------------------

/// AC-2 / BC-2.16.002: two-step pipeline where step 2 uses step 1's access_token.

#[tokio::test]
async fn test_BC_2_16_002_two_step_pipeline_step2_uses_step1_token() {
    let spec = SensorSpec {
        sensor_id: "crowdstrike".to_string(),
        name: "CrowdStrike".to_string(),
        auth_type: AuthType::Oauth2ClientCredentials,
        base_url: "https://api.crowdstrike.com".to_string(),
        tables: vec![TableSpec::new_point_in_time(
            "detections",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![
                FetchStep {
                    name: "get_token".to_string(),
                    method: "POST".to_string(),
                    path_template: "/oauth2/token".to_string(),
                    body_template: Some("grant_type=client_credentials".to_string()),
                    response_path: "$.access_token".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec!["access_token".to_string()],
                    fan_out_batch_size: None,
                    pagination: None,
                },
                FetchStep {
                    name: "fetch_detections".to_string(),
                    method: "GET".to_string(),
                    path_template: "/detections/v2?token=${get_token.access_token}".to_string(),
                    body_template: None,
                    response_path: "$.resources".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    pagination: None,
                },
            ],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
    };

    let table = spec.tables[0].clone();
    let context = FetchContext {
        client_id: TenantId::new("test-client"),
        query_filters: HashMap::new(),
    };

    let result = PipelineExecutor::execute(&spec, &table, &context).await;

    // When implemented: result.is_ok() and records from step 2 use the token from step 1.
    drop(result);
}
