//! Tests for BC-2.09.007: OutputSchema for Type-Safe LLM Reasoning
//!
//! Verifies: every tool has an outputSchema; `_meta.safety_flags` is a typed array;
//! no per-field parallel safety_flag fields; `_meta` envelope fields all present.
//!
//! All tests pass (implementation complete).

use prism_security::output_schema::{MetaEnvelopeSchema, OutputSchema, ResultsItemSchema};
use serde_json::{json, Value};

// ─── BC-2.09.007 Postcondition 1 — outputSchema present ─────────────────────

/// BC-2.09.007 postcondition 1: outputSchema exists for sensor tool.
/// AC-8 canonical vector: CrowdStrike sensor tool.
#[test]
fn test_BC_2_09_007_output_schema_exists_for_sensor_tool() {
    let schema = OutputSchema::for_sensor_tool(
        "crowdstrike_detections",
        "crowdstrike",
        json!({"type": "object", "properties": {"hostname": {"type": "string"}}}),
    );
    assert!(schema.is_object(), "outputSchema must be a JSON object");
}

// ─── BC-2.09.007 Postcondition 2 — _meta envelope fields ────────────────────

/// BC-2.09.007 postcondition 2: outputSchema includes all _meta envelope fields.
#[test]
fn test_BC_2_09_007_meta_schema_includes_all_required_envelope_fields() {
    let meta_schema = MetaEnvelopeSchema::schema();
    let required_fields = [
        "tool",
        "data_source",
        "query_time",
        "trust_level",
        "safety_flags",
        "total_results",
        "page",
        "has_more",
        "next_cursor",
    ];
    let properties = meta_schema
        .get("properties")
        .expect("_meta schema must have 'properties'");
    for field in &required_fields {
        assert!(
            properties.get(field).is_some(),
            "_meta schema must include '{field}' property"
        );
    }
}

// ─── BC-2.09.007 Postcondition 4 — safety_flags typed array ─────────────────

/// BC-2.09.007 postcondition 4: `_meta.safety_flags` declared as typed array.
/// AC-8: schema includes `_meta.safety_flags` as `type: "array"` with structured items.
#[test]
fn test_BC_2_09_007_safety_flags_declared_as_typed_array_in_meta_schema() {
    let meta_schema = MetaEnvelopeSchema::schema();
    let safety_flags_schema = meta_schema["properties"]["safety_flags"].clone();
    assert_eq!(
        safety_flags_schema["type"].as_str().unwrap_or(""),
        "array",
        "safety_flags must be declared as type 'array'"
    );
    // Must have an 'items' schema
    assert!(
        safety_flags_schema.get("items").is_some(),
        "safety_flags must have 'items' schema for structured items"
    );
}

/// BC-2.09.007 postcondition 4: safety_flag item schema has required fields.
#[test]
fn test_BC_2_09_007_safety_flag_item_schema_has_required_fields() {
    let item_schema = MetaEnvelopeSchema::safety_flag_item_schema();
    let required_fields = ["field", "index", "pattern", "category"];
    let properties = item_schema
        .get("properties")
        .expect("safety_flag item schema must have 'properties'");
    for field in &required_fields {
        assert!(
            properties.get(field).is_some(),
            "safety_flag item must have '{field}' property in schema"
        );
    }
}

// ─── BC-2.09.007 Postcondition 5 — no per-field parallel fields ─────────────

/// BC-2.09.007 postcondition 5: schema must NOT contain per-field `{field}_safety_flag` keys.
/// AC-8: no per-field parallel safety_flag fields in schema.
#[test]
fn test_BC_2_09_007_schema_has_no_per_field_safety_flag_keys() {
    let full_schema = OutputSchema::for_sensor_tool(
        "crowdstrike_detections",
        "crowdstrike",
        json!({
            "type": "object",
            "properties": {
                "hostname": {"type": "string"},
                "severity": {"type": "string"}
            }
        }),
    );
    assert!(
        !ResultsItemSchema::has_forbidden_per_field_safety_flags(&full_schema),
        "schema must not contain per-field parallel safety_flag keys"
    );
}

/// ResultsItemSchema::has_valid_safety_flags_declaration returns true for correct schema.
#[test]
fn test_BC_2_09_007_valid_safety_flags_declaration_returns_true() {
    let schema = json!({
        "properties": {
            "_meta": {
                "properties": {
                    "safety_flags": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "field": {"type": "string"},
                                "index": {"type": "integer"},
                                "pattern": {"type": "string"},
                                "category": {"type": "string"}
                            }
                        }
                    }
                }
            }
        }
    });
    assert!(
        ResultsItemSchema::has_valid_safety_flags_declaration(&schema),
        "valid safety_flags declaration must be recognized"
    );
}

/// ResultsItemSchema::has_forbidden_per_field_safety_flags detects violations.
#[test]
fn test_BC_2_09_007_detects_forbidden_per_field_safety_flag_key() {
    let bad_schema = json!({
        "properties": {
            "hostname": {"type": "string"},
            "hostname_safety_flag": {"type": "string"}
        }
    });
    assert!(
        ResultsItemSchema::has_forbidden_per_field_safety_flags(&bad_schema),
        "schema with 'hostname_safety_flag' must be detected as forbidden"
    );
}

// ─── BC-2.09.007 — trust_level typed enum in schema ─────────────────────────

/// BC-2.09.007: trust_level in _meta schema must be a string enum with valid values.
#[test]
fn test_BC_2_09_007_trust_level_in_meta_schema_is_string_enum() {
    let meta_schema = MetaEnvelopeSchema::schema();
    let trust_level_schema = &meta_schema["properties"]["trust_level"];
    assert_eq!(
        trust_level_schema["type"].as_str().unwrap_or(""),
        "string",
        "trust_level must be declared as type 'string'"
    );
    // Must enumerate valid values
    let enum_values = trust_level_schema
        .get("enum")
        .and_then(|e| e.as_array())
        .expect("trust_level must have 'enum' with valid values");
    assert!(
        enum_values
            .iter()
            .any(|v| v.as_str() == Some("untrusted_external")),
        "enum must include 'untrusted_external'"
    );
    assert!(
        enum_values.iter().any(|v| v.as_str() == Some("internal")),
        "enum must include 'internal'"
    );
}
