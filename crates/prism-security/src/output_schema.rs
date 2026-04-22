//! OutputSchema — JSON Schema types for structured LLM reasoning (BC-2.09.007).
//!
//! Stub: `unimplemented!()` bodies. Red Gate — tests must fail.
//!
//! BC-2.09.007 requires every MCP tool to declare an `outputSchema` that includes:
//! - `_meta` envelope fields (tool, data_source, query_time, trust_level,
//!   safety_flags, total_results, page, has_more, next_cursor)
//! - `results` array item types with field names, types, descriptions
//! - `_meta.safety_flags` declared as `type: "array"` with structured items
//! - NO per-field parallel `safety_flag` fields

use serde_json::Value;

/// JSON Schema for the `_meta` envelope (BC-2.09.007, BC-2.09.008).
pub struct MetaEnvelopeSchema;

impl MetaEnvelopeSchema {
    /// Returns the JSON Schema object for the `_meta` field.
    ///
    /// Must include:
    /// - `tool`: string
    /// - `data_source`: string or array of strings
    /// - `query_time`: string (ISO8601)
    /// - `trust_level`: string enum (`untrusted_external` | `internal`)
    /// - `safety_flags`: array with structured items
    /// - `total_results`: integer
    /// - `page`: integer
    /// - `has_more`: boolean
    /// - `next_cursor`: string or null
    pub fn schema() -> Value {
        unimplemented!("MetaEnvelopeSchema::schema — stub (Red Gate)")
    }

    /// Returns the JSON Schema for a single `safety_flags` array item.
    ///
    /// BC-2.09.007: `{field: string, index: integer, pattern: string, category: string}`
    /// No per-field parallel `safety_flag` fields.
    pub fn safety_flag_item_schema() -> Value {
        unimplemented!("MetaEnvelopeSchema::safety_flag_item_schema — stub (Red Gate)")
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
        unimplemented!("ResultsItemSchema::has_valid_safety_flags_declaration — stub (Red Gate)")
    }

    /// Returns `true` if the schema contains any per-field parallel
    /// `{field}_safety_flag` keys (which are forbidden by BC-2.09.004 and BC-2.09.007).
    pub fn has_forbidden_per_field_safety_flags(schema: &Value) -> bool {
        unimplemented!("ResultsItemSchema::has_forbidden_per_field_safety_flags — stub (Red Gate)")
    }
}

/// Complete `outputSchema` for a sensor query tool (BC-2.09.007).
pub struct OutputSchema;

impl OutputSchema {
    /// Builds the full `outputSchema` JSON Schema for a sensor query tool.
    ///
    /// Includes `_meta` envelope and `results` array with the given item schema.
    pub fn for_sensor_tool(tool_name: &str, sensor_name: &str, results_item: Value) -> Value {
        unimplemented!("OutputSchema::for_sensor_tool — stub (Red Gate)")
    }
}
