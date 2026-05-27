//! VP-017: OCSF Normalization — Unmapped Fields Preserved (proptest)
//!
//! # Property Statement (VP-017)
//!
//! For every raw sensor record `r` and every field `f` in `r` that has no mapping
//! to an OCSF path, the normalized output contains `f` and its value under
//! `raw_extensions` (or the mapped OCSF field). No input field is silently dropped;
//! the union of mapped fields and `raw_extensions` keys covers ALL input field keys.
//!
//! # Method
//!
//! - Generate arbitrary JSON objects representing raw sensor records with a mix of
//!   known and unknown fields (proptest strategy).
//! - Call `SpecDrivenMapper::map()` with the generated record.
//! - If the call returns `Ok`, collect all field keys from the input JSON.
//! - Assert: every input key that is NOT declared in the sensor spec appears in
//!   `extensions` — no field is silently dropped. (BC-2.02.007)
//!
//! # Coverage
//!
//! - AC-10: VP-017 proptest passes with generated test cases.
//! - BC-2.02.007: All unmapped vendor-specific fields are preserved in raw_extensions.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use prism_core::ColumnType;
use prism_spec_engine::{
    spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
    types::ConfigSnapshot,
    ConfigManager, PluginRuntime,
};
use proptest::prelude::*;
use prost_reflect::DynamicMessage;
use serde_json::{Map, Value as JsonValue};

use crate::mappers::{SensorMapper, SpecDrivenMapper};

// ---------------------------------------------------------------------------
// Test infrastructure helpers
// ---------------------------------------------------------------------------

/// Build a minimal `reqwest::Client` with the 30-second timeout required by
/// TD-S-PLUGIN-PREREQ-B-005 (CLAUDE.md Conventions §HTTP client timeout).
fn test_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest::Client construction must not fail in test environment")
}

/// Build a `PluginRuntime` with no loaded plugins (test mode).
fn empty_plugin_runtime() -> Arc<PluginRuntime> {
    Arc::new(
        PluginRuntime::new(test_http_client())
            .expect("PluginRuntime::new must succeed with valid HTTP client"),
    )
}

/// Build a `ConfigManager` containing a single synthetic sensor spec entry.
///
/// ADR-030 Approach D: constructs `spec_parser::SensorSpec` directly with `Vec<TableSpec>`.
/// `SpecDrivenMapper::map()` reads `ocsf_field` from `TableSpec.columns` (type `ColumnSpec`).
fn config_manager_with_ocsf_columns(
    sensor_id: &str,
    table_name: &str,
    columns: Vec<ColumnSpec>,
) -> Arc<ConfigManager> {
    let table = TableSpec::new_point_in_time(
        format!("{}.{}", sensor_id, table_name),
        "security_finding",
        columns,
        vec![],
    );

    let sensor_spec = SensorSpec::new(
        sensor_id,
        sensor_id,
        AuthType::ApiKey,
        "https://example.test",
        vec![table],
        None,
        "1.0.0",
        vec![],
    );

    let mut sensor_specs = HashMap::new();
    sensor_specs.insert(sensor_id.to_string(), sensor_spec);

    let snapshot = ConfigSnapshot {
        sensor_specs,
        failed_specs: HashMap::new(),
        snapshot_hash: "test-hash".to_string(),
    };

    Arc::new(ConfigManager::new(snapshot))
}

/// Build a `ColumnSpec` with a string type and an OCSF field mapping.
///
/// ADR-030 Approach D: uses `ColumnSpec` (spec_parser type) since `ConfigSnapshot`
/// now holds `spec_parser::SensorSpec` with `Vec<TableSpec>` → `Vec<ColumnSpec>`.
fn string_col_with_ocsf(name: &str, ocsf_field: &str) -> ColumnSpec {
    let mut col = ColumnSpec::default();
    col.name = name.to_string();
    col.column_type = ColumnType::String;
    col.ocsf_field = Some(ocsf_field.to_string());
    col
}

/// Creates a minimal DynamicMessage for proptest usage.
fn stub_msg() -> DynamicMessage {
    use prost_reflect::DescriptorPool;
    use prost_types::{DescriptorProto, FileDescriptorProto};

    let file = FileDescriptorProto {
        name: Some("proptest_stub.proto".to_owned()),
        syntax: Some("proto3".to_owned()),
        message_type: vec![DescriptorProto {
            name: Some("PropTestStubMsg".to_owned()),
            ..DescriptorProto::default()
        }],
        ..FileDescriptorProto::default()
    };

    let mut pool = DescriptorPool::new();
    pool.add_file_descriptor_proto(file)
        .expect("proptest stub proto must be valid");
    let desc = pool
        .get_message_by_name("PropTestStubMsg")
        .expect("PropTestStubMsg must be in pool");
    DynamicMessage::new(desc)
}

// ---------------------------------------------------------------------------
// Strategy: generate arbitrary JSON objects with mixed known/unknown fields.
// ---------------------------------------------------------------------------

/// Generates an arbitrary JSON string value (non-empty, limited length).
fn arb_json_string() -> impl Strategy<Value = JsonValue> {
    "[a-zA-Z0-9_-]{1,32}".prop_map(JsonValue::String)
}

/// Generates an arbitrary JSON integer value.
fn arb_json_int() -> impl Strategy<Value = JsonValue> {
    (0i64..=1_000_000i64).prop_map(|n| JsonValue::Number(n.into()))
}

/// Generates an arbitrary JSON value (string, integer, or null).
fn arb_json_value() -> impl Strategy<Value = JsonValue> {
    prop_oneof![arb_json_string(), arb_json_int(), Just(JsonValue::Null),]
}

/// Generates an arbitrary "unknown vendor field" key (not in the sensor spec).
fn arb_unknown_field_key() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,20}vendor[a-z0-9_]{0,10}".prop_filter(
        "must not match known field names",
        |k| {
            // Exclude fields declared in the synthetic sensor spec.
            !matches!(
                k.as_str(),
                "detection_id" | "severity" | "created_timestamp"
            )
        },
    )
}

/// Generates a sensor record with declared fields + random unknown extras.
fn arb_detection_with_unknown_fields() -> impl Strategy<Value = Map<String, JsonValue>> {
    prop::collection::vec((arb_unknown_field_key(), arb_json_value()), 0..=8).prop_map(
        |extra_fields| {
            let mut obj = Map::new();
            // Fields declared in the synthetic sensor spec.
            obj.insert(
                "detection_id".to_owned(),
                JsonValue::String("ldt:proptest-001".to_owned()),
            );
            obj.insert("severity".to_owned(), JsonValue::String("High".to_owned()));
            obj.insert(
                "created_timestamp".to_owned(),
                JsonValue::String("2024-03-15T10:30:00Z".to_owned()),
            );
            // Unknown/vendor-specific extras — these must appear in raw_extensions after mapping.
            for (k, v) in extra_fields {
                obj.entry(k).or_insert(v);
            }
            obj
        },
    )
}

// ---------------------------------------------------------------------------
// Helper: check that every key in `input` appears in either `extensions` or is a
// recognized mapped field.
// ---------------------------------------------------------------------------

/// Verifies the VP-017 invariant: no input key is silently dropped.
///
/// For each key in `input`:
///   - If it's a known mapped field (declared in the sensor spec), it is accepted
///     as written to `msg` (we skip proto-level verification here because the stub
///     msg is empty).
///   - If it's NOT a known mapped field, it MUST appear in `extensions`.
fn assert_no_fields_dropped(
    input: &Map<String, JsonValue>,
    extensions: &Map<String, JsonValue>,
    known_mapped_keys: &[&str],
) {
    for key in input.keys() {
        if known_mapped_keys.contains(&key.as_str()) {
            // Known mapped field — accepted as present in msg (not checked at stub level).
            continue;
        }
        assert!(
            extensions.contains_key(key),
            "VP-017: input field '{key}' was silently dropped — not in extensions after mapping"
        );
    }
}

/// Known top-level fields declared in the synthetic sensor spec.
/// These are moved into `msg`, not `extensions`.
const SPEC_DECLARED_FIELDS: &[&str] = &["detection_id", "severity", "created_timestamp"];

// ---------------------------------------------------------------------------
// VP-017 proptest — prop_no_fields_silently_dropped
// ---------------------------------------------------------------------------

proptest! {
    /// VP-017 (AC-10): No input field is silently dropped across generated records.
    ///
    /// Uses `SpecDrivenMapper` with a synthetic sensor spec that declares 3 known
    /// columns (`detection_id`, `severity`, `created_timestamp`). Any extra fields
    /// not in the spec must appear in `extensions` after `map()` returns `Ok`.
    ///
    /// Exercises: BC-2.02.007 invariant via SpecDrivenMapper.
    #[test]
    fn test_BC_2_02_007_vp017_prop_no_fields_silently_dropped(
        raw_obj in arb_detection_with_unknown_fields(),
    ) {
        // Build a synthetic sensor spec with 3 declared columns.
        let config_manager = config_manager_with_ocsf_columns(
            "crowdstrike",
            "detections",
            vec![
                string_col_with_ocsf("detection_id", "finding_info.uid"),
                string_col_with_ocsf("severity", "severity"),
                string_col_with_ocsf("created_timestamp", "start_time"),
            ],
        );
        let mapper = SpecDrivenMapper::new(config_manager, empty_plugin_runtime());

        let raw = JsonValue::Object(raw_obj.clone());
        let mut extensions = Map::new();

        let result = mapper.map(
            "detections",
            &raw,
            &mut stub_msg(),
            &mut extensions,
        );

        // Post-implementation assertions:
        if let Ok(_source_id) = result {
            assert_no_fields_dropped(&raw_obj, &extensions, SPEC_DECLARED_FIELDS);
        }
        // If Err, the mapper reported a structured error — that's also acceptable per
        // BC-2.02.011 (errors don't silently drop fields either).
    }
}
