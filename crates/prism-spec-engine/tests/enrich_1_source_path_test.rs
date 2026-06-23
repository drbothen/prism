//! ENRICH-1 Red Gate tests — `source_path` field on `ColumnSpec`.
//!
//! These tests drive the ENRICH-1 story:
//! 1. `source_path` round-trips through TOML parse (BC-2.16.001 / design §1).
//! 2. Parse-time validation rejects non-`$.` paths (design §1 validation gate).
//! 3. `ColumnMapper::map_record` with `source_path = "$.a.b"` extracts nested field (design §2).
//! 4. `ColumnMapper::map_record` with `source_path = "$.iocs[*].value"` returns JSON-list string.
//! 5. `source_path = None` backward compat (flat key lookup unchanged).
//! 6. Wildcard on empty array → `"[]"` (NOT null) (design §2, zero-element case).
//! 7. `pivot_enrich`-path: when input is a JSON-list string, the UDF layer receives the
//!    whole list (tested via InfusionAsyncUdf unit path — the contract is at the mapper
//!    output level; UDF handling is in prism-query but the list format is defined here).
//!
//! All tests in this file MUST FAIL before implementation (Red Gate gate).
//! Tests MUST PASS after implementation with zero regressions.
//!
//! Naming convention: `test_ENRICH_1_<area>_<description>`.

use prism_core::{ColumnType, PrismError};
use prism_spec_engine::{
    column_mapping::ColumnMapper,
    spec_parser::{ColumnSpec, SpecLoader, TableSpec},
};
use serde_json::json;

// ---------------------------------------------------------------------------
// Helper: minimal valid sensor TOML base for source_path injection tests.
// ---------------------------------------------------------------------------

const SOURCE_PATH_TOML_BASE: &str = r#"
sensor_id = "test"
name = "Test Sensor"
auth_type = "bearer_static"
base_url = "https://example.com"
version = "1.0.0"

[[tables]]
table_name = "events"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "REPLACE_NAME"
  column_type = "string"
REPLACE_SOURCE_PATH
  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/api/v1/events"
  response_path = "$.data"
  variables_produced = []
  [tables.steps.pagination]
  type = "none"
"#;

// ---------------------------------------------------------------------------
// Test 1: source_path round-trips through TOML parse.
// FAILS RED: ColumnSpec has no `source_path` field before ENRICH-1 implementation.
// ---------------------------------------------------------------------------

/// ENRICH-1: `source_path` is parsed from TOML and set on `ColumnSpec`.
#[test]
fn test_ENRICH_1_toml_parse_source_path_round_trips() {
    let toml = SOURCE_PATH_TOML_BASE
        .replace("REPLACE_NAME", "ioc_value")
        .replace(
            "REPLACE_SOURCE_PATH",
            r#"  source_path = "$.iocs[*].value""#,
        );

    let spec = SpecLoader::parse(&toml).expect("TOML with valid source_path must parse");

    let col = spec
        .tables
        .first()
        .expect("table must be present")
        .columns
        .first()
        .expect("column must be present");

    assert_eq!(
        col.source_path.as_deref(),
        Some("$.iocs[*].value"),
        "ENRICH-1: source_path must round-trip through TOML parse; got {:?}",
        col.source_path
    );
}

// ---------------------------------------------------------------------------
// Test 2: source_path defaults to None when absent (backward compat).
// FAILS RED: ColumnSpec has no `source_path` field before ENRICH-1 implementation.
// ---------------------------------------------------------------------------

/// ENRICH-1: `source_path` defaults to `None` when not declared in TOML.
#[test]
fn test_ENRICH_1_toml_parse_source_path_absent_defaults_to_none() {
    let toml = SOURCE_PATH_TOML_BASE
        .replace("REPLACE_NAME", "alert_id")
        .replace("REPLACE_SOURCE_PATH", ""); // no source_path field

    let spec =
        SpecLoader::parse(&toml).expect("TOML without source_path must parse (backward compat)");

    let col = spec
        .tables
        .first()
        .expect("table must be present")
        .columns
        .first()
        .expect("column must be present");

    assert_eq!(
        col.source_path, None,
        "ENRICH-1: source_path must default to None when absent; got {:?}",
        col.source_path
    );
}

// ---------------------------------------------------------------------------
// Test 3: parse-time validation rejects source_path not starting with "$."
// FAILS RED: validation gate not present before ENRICH-1 implementation.
// ---------------------------------------------------------------------------

/// ENRICH-1: `source_path` without `$.` prefix is rejected at parse time with E-SPEC-001.
#[test]
fn test_ENRICH_1_validation_rejects_source_path_without_dollar_dot() {
    let toml = SOURCE_PATH_TOML_BASE
        .replace("REPLACE_NAME", "ioc_value")
        .replace("REPLACE_SOURCE_PATH", r#"  source_path = "iocs[*].value""#); // missing $.

    let err =
        SpecLoader::parse(&toml).expect_err("source_path without $. prefix must fail validation");

    match err {
        PrismError::Spec(se) => {
            let msg = &se.message;
            assert!(
                msg.contains("source_path"),
                "ENRICH-1: error must mention source_path; got: {msg}"
            );
            assert!(
                msg.contains("$."),
                "ENRICH-1: error must mention the required '$.' prefix; got: {msg}"
            );
        }
        other => panic!("ENRICH-1: expected PrismError::Spec, got: {other:?}"),
    }
}

/// ENRICH-1: `source_path = "$."` with no segment after it is also rejected.
#[test]
fn test_ENRICH_1_validation_rejects_source_path_bare_dollar_dot() {
    let toml = SOURCE_PATH_TOML_BASE
        .replace("REPLACE_NAME", "ioc_value")
        .replace("REPLACE_SOURCE_PATH", r#"  source_path = "$.""#); // empty key segment

    let err = SpecLoader::parse(&toml)
        .expect_err("source_path '$.' with no segment must fail validation");

    match err {
        PrismError::Spec(se) => {
            let msg = &se.message;
            assert!(
                msg.contains("source_path"),
                "ENRICH-1: error must mention source_path; got: {msg}"
            );
        }
        other => panic!("ENRICH-1: expected PrismError::Spec, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 4: ColumnSpec default() has source_path = None.
// FAILS RED: ColumnSpec has no `source_path` field before ENRICH-1 implementation.
// ---------------------------------------------------------------------------

/// ENRICH-1: `ColumnSpec::default()` produces `source_path = None`.
#[test]
fn test_ENRICH_1_column_spec_default_source_path_is_none() {
    let col = ColumnSpec::default();
    assert_eq!(
        col.source_path, None,
        "ENRICH-1: ColumnSpec::default() must have source_path = None"
    );
}

// ---------------------------------------------------------------------------
// Helper: build a minimal TableSpec for ColumnMapper tests.
// ---------------------------------------------------------------------------

fn make_table_with_cols(cols: Vec<ColumnSpec>) -> TableSpec {
    use prism_spec_engine::spec_parser::{FetchStep, PaginationConfig};
    // FetchStep is #[non_exhaustive] — use Default + field mutation pattern.
    let mut step = FetchStep::default();
    step.name = "fetch".to_string();
    step.method = "GET".to_string();
    step.path_template = "/api/v1/events".to_string();
    step.response_path = "$.data".to_string();
    step.pagination = Some(PaginationConfig::None);
    TableSpec::new_point_in_time("events", "security_finding", cols, vec![step])
}

// ---------------------------------------------------------------------------
// Test 5: ColumnMapper with source_path = "$.a.b" extracts nested field.
// FAILS RED: ColumnMapper does flat get(&col.name) only before ENRICH-1.
// ---------------------------------------------------------------------------

/// ENRICH-1: `ColumnMapper::map_record` dispatches on `source_path` for nested dot-notation.
#[test]
fn test_ENRICH_1_column_mapper_source_path_nested_extraction() {
    // ColumnSpec is #[non_exhaustive] — use Default + field mutation pattern.
    let mut col = ColumnSpec::default();
    col.name = "alert_data_ip".to_string();
    col.column_type = ColumnType::String;
    col.source_path = Some("$.alert_data.ip".to_string());
    let table = make_table_with_cols(vec![col]);

    let raw = json!({
        "alert_data": {
            "ip": "10.0.0.1",
            "domain": "example.com"
        }
    });

    let result = ColumnMapper::map_record(&raw, &table)
        .expect("ENRICH-1: ColumnMapper::map_record must succeed");

    let val = result
        .raw_extensions
        .get("alert_data_ip")
        .expect("ENRICH-1: column 'alert_data_ip' must be present in raw_extensions");

    assert_eq!(
        val.as_str().unwrap_or(""),
        "10.0.0.1",
        "ENRICH-1: nested source_path extraction must produce '10.0.0.1'; got {:?}",
        val
    );
}

// ---------------------------------------------------------------------------
// Test 6: ColumnMapper with source_path = "$.iocs[*].value" returns JSON-list string.
// FAILS RED: ColumnMapper does flat get(&col.name) only; wildcard path not handled.
// ---------------------------------------------------------------------------

/// ENRICH-1: wildcard `[*]` path returns compact JSON-list string `["h1","h2"]`.
#[test]
fn test_ENRICH_1_column_mapper_source_path_wildcard_returns_json_list_string() {
    let mut col = ColumnSpec::default();
    col.name = "iocs_value".to_string();
    col.column_type = ColumnType::String;
    col.source_path = Some("$.iocs[*].value".to_string());
    let table = make_table_with_cols(vec![col]);

    let raw = json!({
        "iocs": [
            { "type": "ip", "value": "hash1" },
            { "type": "domain", "value": "hash2" }
        ]
    });

    let result = ColumnMapper::map_record(&raw, &table)
        .expect("ENRICH-1: ColumnMapper::map_record must succeed");

    let val = result
        .raw_extensions
        .get("iocs_value")
        .expect("ENRICH-1: column 'iocs_value' must be present in raw_extensions");

    // The value must be a JSON-list string (compact, no spaces).
    let val_str = val.as_str().unwrap_or_else(|| {
        panic!(
            "ENRICH-1: wildcard result must be a string (JSON-list), got {:?}",
            val
        )
    });
    assert_eq!(
        val_str, r#"["hash1","hash2"]"#,
        "ENRICH-1: wildcard result must be '[\"hash1\",\"hash2\"]'; got: {val_str}"
    );
}

// ---------------------------------------------------------------------------
// Test 7: Wildcard on empty array → "[]" (not null, not empty string).
// FAILS RED: extract_at_path returns empty Array but column_mapper doesn't handle it.
// ---------------------------------------------------------------------------

/// ENRICH-1: wildcard `[*]` on empty array yields `"[]"` (not null).
#[test]
fn test_ENRICH_1_column_mapper_wildcard_empty_array_returns_empty_json_list() {
    let mut col = ColumnSpec::default();
    col.name = "iocs_value".to_string();
    col.column_type = ColumnType::String;
    col.source_path = Some("$.iocs[*].value".to_string());
    let table = make_table_with_cols(vec![col]);

    let raw = json!({
        "iocs": []   // empty array
    });

    let result = ColumnMapper::map_record(&raw, &table)
        .expect("ENRICH-1: ColumnMapper::map_record must succeed with empty iocs");

    let val = result
        .raw_extensions
        .get("iocs_value")
        .expect("ENRICH-1: column 'iocs_value' must be present (not absent) for empty array");

    let val_str = val.as_str().unwrap_or_else(|| {
        panic!(
            "ENRICH-1: empty wildcard result must be string '[]', got {:?}",
            val
        )
    });
    assert_eq!(
        val_str, "[]",
        "ENRICH-1: empty wildcard must produce '[]', not null or empty string; got: {val_str}"
    );
}

// ---------------------------------------------------------------------------
// Test 8: source_path = None uses flat key lookup (backward compat).
// FAILS RED only if the new code path breaks the old one; documents invariant.
// ---------------------------------------------------------------------------

/// ENRICH-1 backward-compat: `source_path = None` still uses flat `get(&col.name)`.
#[test]
fn test_ENRICH_1_column_mapper_backward_compat_flat_key_when_no_source_path() {
    let mut col = ColumnSpec::default();
    col.name = "alert_id".to_string();
    col.column_type = ColumnType::String;
    // source_path = None by default (explicit for clarity)
    let table = make_table_with_cols(vec![col]);

    let raw = json!({
        "alert_id": "ALERT-001",
        "status": "open"
    });

    let result = ColumnMapper::map_record(&raw, &table)
        .expect("ENRICH-1: backward-compat flat lookup must succeed");

    let val = result
        .raw_extensions
        .get("alert_id")
        .expect("ENRICH-1: 'alert_id' must be present in raw_extensions via flat key lookup");

    assert_eq!(
        val.as_str().unwrap_or(""),
        "ALERT-001",
        "ENRICH-1 backward-compat: flat key lookup must return 'ALERT-001'; got {:?}",
        val
    );
}

// ---------------------------------------------------------------------------
// Test 9: source_path on a field absent in the record → column absent from result.
// FAILS RED: mapper must handle extract_at_path Err gracefully (emit warn, skip).
// ---------------------------------------------------------------------------

/// ENRICH-1: when `source_path` extraction fails (path absent), column is skipped (no crash).
#[test]
fn test_ENRICH_1_column_mapper_source_path_absent_field_produces_no_entry() {
    let mut col = ColumnSpec::default();
    col.name = "alert_data_ip".to_string();
    col.column_type = ColumnType::String;
    col.source_path = Some("$.alert_data.ip".to_string());
    let table = make_table_with_cols(vec![col]);

    // raw record has NO alert_data key at all
    let raw = json!({
        "alert_id": "ALERT-001"
    });

    let result = ColumnMapper::map_record(&raw, &table)
        .expect("ENRICH-1: absent source_path field must not error; must skip column");

    // The column must NOT appear in raw_extensions (was not present in the record).
    assert!(
        !result.raw_extensions.contains_key("alert_data_ip"),
        "ENRICH-1: absent path column must not be in raw_extensions; got {:?}",
        result.raw_extensions
    );
}

// ---------------------------------------------------------------------------
// Test 10: singleton nested extraction (no wildcard, dot-path).
// FAILS RED: ColumnMapper only does flat get().
// ---------------------------------------------------------------------------

/// ENRICH-1: `source_path = "$.ioc.value"` extracts from a singleton nested object.
#[test]
fn test_ENRICH_1_column_mapper_singleton_nested_ioc_value() {
    let mut col = ColumnSpec::default();
    col.name = "ioc_value_singleton".to_string();
    col.column_type = ColumnType::String;
    col.source_path = Some("$.ioc.value".to_string());
    let table = make_table_with_cols(vec![col]);

    let raw = json!({
        "ioc": {
            "type": "ip",
            "value": "1.2.3.4"
        }
    });

    let result = ColumnMapper::map_record(&raw, &table)
        .expect("ENRICH-1: singleton nested extraction must succeed");

    let val = result
        .raw_extensions
        .get("ioc_value_singleton")
        .expect("ENRICH-1: 'ioc_value_singleton' must be present");

    assert_eq!(
        val.as_str().unwrap_or(""),
        "1.2.3.4",
        "ENRICH-1: singleton nested extraction via source_path; got {:?}",
        val
    );
}
