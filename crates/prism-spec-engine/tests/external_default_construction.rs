// Integration test: positive-case for non_exhaustive forward-compat construction.
//
// These tests verify that external callers CAN construct TOML-deserialized types
// via the provided constructors even after `#[non_exhaustive]` is applied.
// This is the complement to the compile-fail test in tests/external/non-exhaustive-violation/.
//
// AC-5 + HIGH-004 (S-PLUGIN-PREREQ-C): These tests MUST PASS after the Default
// impls and constructors are added.  They ensure the forward-compat construction
// pattern is functional for every type in the sibling-sweep.

use prism_core::{ColumnOptions, ColumnType};
use prism_spec_engine::spec_parser::{
    AuthType, ColumnSpec, CredentialRef, FetchStep, PaginationConfig, RateLimitHints, SensorSpec,
    SensorTableDescriptor,
};

/// Verify that `SensorSpec::new()` constructor works for external callers.
#[test]
fn test_sensor_spec_constructor_succeeds() {
    let spec = SensorSpec::new(
        "my-sensor",
        "My Sensor",
        AuthType::ApiKey,
        "https://api.example.com",
        vec![],
        None,
        "1.0.0",
        vec![],
    );
    assert_eq!(spec.sensor_id, "my-sensor");
    assert_eq!(spec.tables.len(), 0);
}

/// Verify that `FetchStep::new()` constructor works for external callers.
#[test]
fn test_fetch_step_constructor_succeeds() {
    let step = FetchStep::new(
        "fetch",
        "GET",
        "/v1/devices",
        None,
        "$.items",
        None,
        vec![],
        None,
        Some(PaginationConfig::CursorToken {
            cursor_response_path: "$.next_cursor".to_string(),
            page_size: Some(50),
        }),
    );
    assert_eq!(step.name, "fetch");
    assert!(step.pagination.is_some());
}

/// Verify that `ColumnSpec::new()` constructor works for external callers.
#[test]
fn test_column_spec_constructor_succeeds() {
    let col = ColumnSpec::new("device_id", ColumnType::String, None, vec![]);
    assert_eq!(col.name, "device_id");
    assert!(col.ocsf_field.is_none());
    assert!(col.options.is_empty());
}

/// Verify that `CredentialRef::new()` constructor works for external callers.
#[test]
fn test_credential_ref_constructor_succeeds() {
    let cref = CredentialRef::new("api_key");
    assert_eq!(cref.name, "api_key");
}

/// Verify that `SensorTableDescriptor::new()` constructor works for external callers.
#[test]
fn test_sensor_table_descriptor_constructor_succeeds() {
    let desc = SensorTableDescriptor::new("crowdstrike.devices", vec![], "crowdstrike", false);
    assert_eq!(desc.table_name, "crowdstrike.devices");
    assert!(!desc.has_credentials);
}

/// Verify that `RateLimitHints::new()` constructor works for external callers.
#[test]
fn test_rate_limit_hints_constructor_succeeds() {
    let hints = RateLimitHints::new(Some(10.0), None);
    assert_eq!(hints.requests_per_second, Some(10.0));
    assert!(hints.burst_size.is_none());
}

/// Verify ColumnType and ColumnOptions can be matched with wildcard arms (forward compat).
#[test]
fn test_column_type_match_with_wildcard() {
    let col_type = ColumnType::String;
    let result = match col_type {
        ColumnType::String => "string",
        ColumnType::Integer => "integer",
        ColumnType::Float => "float",
        ColumnType::Boolean => "boolean",
        ColumnType::Datetime => "datetime",
        ColumnType::Json => "json",
        _ => "unknown", // required wildcard for #[non_exhaustive]
    };
    assert_eq!(result, "string");
}

/// Verify ColumnOptions can be matched with wildcard arms.
#[test]
fn test_column_options_match_with_wildcard() {
    let col_opt = ColumnOptions::Required;
    let result = match col_opt {
        ColumnOptions::Required => "required",
        ColumnOptions::Index => "index",
        ColumnOptions::Additional => "additional",
        ColumnOptions::Hidden => "hidden",
        ColumnOptions::Optimized => "optimized",
        _ => "unknown", // required wildcard for #[non_exhaustive]
    };
    assert_eq!(result, "required");
}
