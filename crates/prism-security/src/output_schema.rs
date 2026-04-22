//! OutputSchema — JSON Schema types for structured LLM reasoning (BC-2.09.007).
//!
//! BC-2.09.007 requires every MCP tool to declare an `outputSchema` that includes:
//! - `_meta` envelope fields (tool, data_source, query_time, trust_level,
//!   safety_flags, total_results, page, has_more, next_cursor)
//! - `results` array item types with field names, types, descriptions
//! - `_meta.safety_flags` declared as `type: "array"` with structured items
//! - NO per-field parallel `safety_flag` fields

use serde_json::{json, Value};

/// JSON Schema for the `_meta` envelope (BC-2.09.007, BC-2.09.008).
pub struct MetaEnvelopeSchema;

impl MetaEnvelopeSchema {
    /// Returns the JSON Schema object for the `_meta` field.
    ///
    /// Includes all required envelope fields per BC-2.09.007 postcondition 2.
    pub fn schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "tool": { "type": "string", "description": "Name of the MCP tool that produced this response." },
                "data_source": {
                    "description": "Sensor identifier(s). String for single-sensor; array for cross-client queries.",
                    "oneOf": [
                        { "type": "string" },
                        { "type": "array", "items": { "type": "string" } }
                    ]
                },
                "query_time": { "type": "string", "format": "date-time", "description": "ISO8601 timestamp of query execution." },
                "trust_level": {
                    "type": "string",
                    "enum": ["untrusted_external", "internal"],
                    "description": "Trust classification: untrusted_external for sensor data, internal for Prism-generated data."
                },
                "safety_flags": {
                    "type": "array",
                    "description": "Centralized injection detection flags. Empty array when no patterns detected.",
                    "items": Self::safety_flag_item_schema()
                },
                "total_results": { "type": "integer", "description": "Total number of results in this response." },
                "page": { "type": "integer", "description": "Current page number (1-based)." },
                "has_more": { "type": "boolean", "description": "True when additional pages are available." },
                "next_cursor": {
                    "description": "Cursor for the next page; null when has_more is false.",
                    "oneOf": [
                        { "type": "string" },
                        { "type": "null" }
                    ]
                }
            },
            "required": ["tool", "data_source", "query_time", "trust_level", "safety_flags",
                         "total_results", "page", "has_more", "next_cursor"]
        })
    }

    /// Returns the JSON Schema for a single `safety_flags` array item.
    ///
    /// BC-2.09.007: `{field: string, index: integer, pattern: string, category: string}`
    /// No per-field parallel `safety_flag` fields.
    pub fn safety_flag_item_schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "field": { "type": "string", "description": "Sensor record field that triggered detection." },
                "index": { "type": "integer", "description": "Zero-based index of the item in the results array." },
                "pattern": { "type": "string", "description": "Human-readable description of the matched pattern." },
                "category": {
                    "type": "string",
                    "description": "Detection category.",
                    "enum": ["prompt_injection", "role_impersonation", "xml_context_escape",
                             "code_fence_escape", "base64_encoded", "truncated_scan"]
                }
            },
            "required": ["field", "index", "pattern", "category"]
        })
    }
}

/// JSON Schema for one item in the `results` array (BC-2.09.007).
pub struct ResultsItemSchema;

impl ResultsItemSchema {
    /// Returns `true` if the provided JSON Schema value correctly declares
    /// `_meta.safety_flags` as a typed array with no per-field parallel fields.
    ///
    /// BC-2.09.007 postconditions 4-5.
    pub fn has_valid_safety_flags_declaration(schema: &Value) -> bool {
        // Navigate to _meta.properties.safety_flags
        let safety_flags = schema
            .pointer("/properties/_meta/properties/safety_flags")
            .or_else(|| schema.pointer("/properties/safety_flags"));

        match safety_flags {
            Some(sf) => {
                sf.get("type").and_then(Value::as_str) == Some("array") && sf.get("items").is_some()
            }
            None => false,
        }
    }

    /// Returns `true` if the schema contains any per-field parallel
    /// `{field}_safety_flag` keys (which are forbidden by BC-2.09.004 and BC-2.09.007).
    pub fn has_forbidden_per_field_safety_flags(schema: &Value) -> bool {
        schema_has_per_field_safety_flag(schema)
    }
}

/// Recursively check a JSON Schema value for any key ending in `_safety_flag`.
fn schema_has_per_field_safety_flag(value: &Value) -> bool {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                if key.ends_with("_safety_flag") {
                    return true;
                }
                if schema_has_per_field_safety_flag(val) {
                    return true;
                }
            }
            false
        }
        Value::Array(arr) => arr.iter().any(schema_has_per_field_safety_flag),
        _ => false,
    }
}

/// Complete `outputSchema` for a sensor query tool (BC-2.09.007).
pub struct OutputSchema;

impl OutputSchema {
    /// Builds the full `outputSchema` JSON Schema for a sensor query tool.
    ///
    /// Includes `_meta` envelope and `results` array with the given item schema.
    /// Conforms to BC-2.09.007 postconditions 1-5.
    pub fn for_sensor_tool(tool_name: &str, _sensor_name: &str, results_item: Value) -> Value {
        json!({
            "type": "object",
            "description": format!("Response envelope for {tool_name}"),
            "properties": {
                "_meta": MetaEnvelopeSchema::schema(),
                "results": {
                    "type": "array",
                    "items": results_item
                }
            },
            "required": ["_meta", "results"]
        })
    }
}
