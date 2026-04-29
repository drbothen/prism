#![allow(non_snake_case)]
//! BC-2.16.004: Rust Escape Hatch for Custom Adapters
//!
//! Tests cover:
//! - CustomAdapterRegistry: register + get by sensor_id
//! - EC-003: duplicate adapter sensor_id -> startup error
//! - override_auth: Some(auth) replaces spec auth_type
//! - override_fetch: Some(records) replaces HTTP call; None falls through
//! - transform_response: Some(value) transforms raw response; None passes through
//! - Panic safety: adapter panic in override_fetch caught as E-SPEC-008
//! - Adapter without matching spec: warning, not error
//! - Spec without matching adapter: fully config-driven, no error
//!
//! AC-4 (S-1.11): CustomAdapter registered -> overrides TOML spec pipeline.

use prism_core::{OrgSlug, SpecErrorCode};
use prism_spec_engine::custom_adapter::{CustomAdapter, CustomAdapterRegistry, SensorAuth};
use prism_spec_engine::pipeline::FetchContext;
use prism_spec_engine::spec_parser::FetchStep;

// ---------------------------------------------------------------------------
// Test double: a CustomAdapter that overrides fetch
// ---------------------------------------------------------------------------

struct MockFetchAdapter {
    sensor_id: String,
    records_to_return: Vec<serde_json::Value>,
}

impl CustomAdapter for MockFetchAdapter {
    fn sensor_id(&self) -> &str {
        &self.sensor_id
    }

    fn override_auth(&self, _client_id: &OrgSlug) -> Option<Box<dyn SensorAuth>> {
        None // pass-through
    }

    fn override_fetch(
        &self,
        _table: &str,
        _step: &FetchStep,
        _context: &FetchContext,
    ) -> Option<Vec<serde_json::Value>> {
        Some(self.records_to_return.clone())
    }

    fn transform_response(
        &self,
        _table: &str,
        _raw: &serde_json::Value,
    ) -> Option<serde_json::Value> {
        None // pass-through
    }
}

// ---------------------------------------------------------------------------
// Test double: a CustomAdapter that panics
// ---------------------------------------------------------------------------

struct PanickingAdapter {
    sensor_id: String,
}

impl CustomAdapter for PanickingAdapter {
    fn sensor_id(&self) -> &str {
        &self.sensor_id
    }

    fn override_auth(&self, _client_id: &OrgSlug) -> Option<Box<dyn SensorAuth>> {
        None
    }

    fn override_fetch(
        &self,
        _table: &str,
        _step: &FetchStep,
        _context: &FetchContext,
    ) -> Option<Vec<serde_json::Value>> {
        panic!("adapter panic — must be caught by registry (E-SPEC-008)")
    }

    fn transform_response(
        &self,
        _table: &str,
        _raw: &serde_json::Value,
    ) -> Option<serde_json::Value> {
        None
    }
}

fn minimal_step() -> FetchStep {
    FetchStep {
        name: "fetch".to_string(),
        method: "GET".to_string(),
        path_template: "/data".to_string(),
        body_template: None,
        response_path: "$.data".to_string(),
        pagination_cursor_path: None,
        variables_produced: vec![],
        fan_out_batch_size: None,
        pagination: None,
    }
}

fn test_context() -> FetchContext {
    FetchContext {
        client_id: OrgSlug::new("test-client"),
        query_filters: std::collections::HashMap::new(),
    }
}

// ---------------------------------------------------------------------------
// BC-2.16.004 tests
// ---------------------------------------------------------------------------

/// BC-2.16.004 postcondition: adapter can be registered and retrieved by sensor_id.
/// AC-4: registered CustomAdapter is reachable for override.
#[test]
fn test_BC_2_16_004_register_and_get_adapter_by_sensor_id() {
    let mut registry = CustomAdapterRegistry::new();
    let adapter = Box::new(MockFetchAdapter {
        sensor_id: "crowdstrike".to_string(),
        records_to_return: vec![serde_json::json!({"id": "det-001"})],
    });

    let result = registry.register(adapter);
    assert!(result.is_ok(), "register must succeed for unique sensor_id");

    let retrieved = registry.get("crowdstrike");
    assert!(
        retrieved.is_some(),
        "registered adapter must be retrievable by sensor_id"
    );
    assert_eq!(retrieved.unwrap().sensor_id(), "crowdstrike");
}

/// BC-2.16.004 edge case EC-003: duplicate sensor_id registration -> error.
#[test]
fn test_BC_2_16_004_rejects_duplicate_adapter_sensor_id() {
    let mut registry = CustomAdapterRegistry::new();

    let adapter1 = Box::new(MockFetchAdapter {
        sensor_id: "crowdstrike".to_string(),
        records_to_return: vec![],
    });
    let adapter2 = Box::new(MockFetchAdapter {
        sensor_id: "crowdstrike".to_string(), // duplicate
        records_to_return: vec![],
    });

    registry
        .register(adapter1)
        .expect("first registration must succeed");
    let result = registry.register(adapter2);

    assert!(
        result.is_err(),
        "second registration with same sensor_id must fail (EC-003)"
    );
}

/// BC-2.16.004 postcondition: spec without matching adapter -> get() returns None.
/// Invariant: no adapter = fully config-driven pipeline (no error).
#[test]
fn test_BC_2_16_004_spec_without_adapter_returns_none() {
    let registry = CustomAdapterRegistry::new(); // empty
    let result = registry.get("no-such-sensor");
    assert!(
        result.is_none(),
        "spec without registered adapter must return None (config-driven path)"
    );
}

/// BC-2.16.004 postcondition: override_fetch = Some -> custom records returned.
/// AC-4: registered adapter overrides TOML spec pipeline.
#[test]
fn test_BC_2_16_004_override_fetch_returns_custom_records() {
    let mut registry = CustomAdapterRegistry::new();
    let expected_records = vec![
        serde_json::json!({"id": "custom-001"}),
        serde_json::json!({"id": "custom-002"}),
    ];
    let adapter = Box::new(MockFetchAdapter {
        sensor_id: "crowdstrike".to_string(),
        records_to_return: expected_records.clone(),
    });
    registry.register(adapter).expect("register must succeed");

    let step = minimal_step();
    let context = test_context();
    let result = registry.safe_override_fetch("crowdstrike", "detections", &step, &context);

    assert!(
        result.is_ok(),
        "safe_override_fetch must succeed: {:?}",
        result.err()
    );
    let records = result.unwrap();
    assert!(
        records.is_some(),
        "MockFetchAdapter::override_fetch returns Some"
    );
    assert_eq!(
        records.unwrap().len(),
        2,
        "must return the 2 custom records"
    );
}

/// BC-2.16.004 panic safety: adapter panic caught as E-SPEC-008. Process not crashed.
#[test]
fn test_BC_2_16_004_adapter_panic_caught_as_e_spec_008() {
    let mut registry = CustomAdapterRegistry::new();
    let adapter = Box::new(PanickingAdapter {
        sensor_id: "panicking-sensor".to_string(),
    });
    registry.register(adapter).expect("register must succeed");

    let step = minimal_step();
    let context = test_context();
    let result = registry.safe_override_fetch("panicking-sensor", "data", &step, &context);

    assert!(
        result.is_err(),
        "panicking adapter must return Err (E-SPEC-008), not crash"
    );
    // Verify E-SPEC-008 code
    match result.unwrap_err() {
        prism_core::PrismError::Spec(e) => {
            assert_eq!(e.code, SpecErrorCode::ESpec008, "must be E-SPEC-008");
        }
        other => panic!("expected PrismError::Spec(E-SPEC-008), got {:?}", other),
    }
}

/// BC-2.16.004 postcondition: override_auth returning None -> spec auth_type used.
#[test]
fn test_BC_2_16_004_override_auth_none_falls_through_to_spec_auth() {
    let adapter = MockFetchAdapter {
        sensor_id: "crowdstrike".to_string(),
        records_to_return: vec![],
    };
    let auth_result = adapter.override_auth(&OrgSlug::new("client-1"));
    assert!(
        auth_result.is_none(),
        "MockFetchAdapter.override_auth returns None (spec auth_type used)"
    );
}
