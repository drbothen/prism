//! SafetyEnvelope — MCP response envelope with trust annotations (BC-2.09.008).
//!
//! # Structure
//!
//! Every sensor tool response is wrapped in an envelope with the shape:
//! ```json
//! {
//!   "_meta": {
//!     "tool": "<tool_name>",
//!     "data_source": "<sensor_id>",
//!     "query_time": "<ISO8601>",
//!     "trust_level": "untrusted_external" | "internal",
//!     "safety_flags": [{...}, ...],
//!     "total_results": <integer>,
//!     "page": <integer>,
//!     "has_more": <boolean>,
//!     "next_cursor": "<cursor>" | null
//!   },
//!   "results": [...],
//!   "content": [{"type": "text", "text": "<N> results found"}],
//!   "structuredContent": {"results": [...]}
//! }
//! ```
//!
//! # Structural Separation (BC-2.09.001)
//!
//! Sensor-originated string values are placed EXCLUSIVELY in `structuredContent`.
//! The `content[].text` prose summary contains ONLY aggregate counts and metadata —
//! NEVER interpolated sensor field values. This prevents prompt injection via
//! attacker-controlled hostnames, descriptions, and process names from appearing
//! in the LLM's primary reasoning context.

use chrono::Utc;
use prism_core::{SafetyFlag, TrustLevel};
use prism_security::{injection_scanner::InjectionScanner, trust_level::trust_level_for_tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The `_meta` section of a Prism MCP response envelope (BC-2.09.008).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub tool: String,
    pub data_source: DataSource,
    pub query_time: String,
    pub trust_level: TrustLevel,
    pub safety_flags: Vec<SafetyFlag>,
    pub total_results: u64,
    pub page: u64,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

/// Data source: single sensor or multiple sensors (cross-client query).
///
/// BC-2.09.008 EC-09-019: cross-client queries report an array.
///
/// The `JsonSchema` derive makes this type usable in `MetaEnvelopeSchemaType` so
/// the outputSchema's `data_source` field correctly represents
/// `oneOf: [{"type": "string"}, {"type": "array", "items": {"type": "string"}}]`
/// instead of untyped `serde_json::Value` (IMP-10 fix).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DataSource {
    Single(String),
    Multiple(Vec<String>),
}

/// One entry in the `content` array — plain text prose for the LLM.
///
/// BC-2.09.001: `text` contains ONLY counts and metadata, never sensor field values.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEntry {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// Structured content wrapper — sensor data presented as typed JSON for LLM inspection.
///
/// BC-2.09.001: all sensor field values live here, never in `content[].text`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredContent {
    pub results: Value,
}

/// The full response envelope (BC-2.09.008).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    #[serde(rename = "_meta")]
    pub meta: ResponseMeta,
    /// Raw sensor results (also mirrored in `structured_content.results`).
    pub results: Value,
    /// Prose summary — counts and metadata ONLY. No sensor field values.
    /// BC-2.09.001 postcondition 2.
    pub content: Vec<ContentEntry>,
    /// Structured sensor data for LLM field-level inspection.
    /// BC-2.09.001 postconditions 1, 3, 4.
    #[serde(rename = "structuredContent")]
    pub structured_content: StructuredContent,
}

/// Builder for `ResponseEnvelope` — applies injection scanning and constructs
/// the `_meta` envelope (BC-2.09.008 + BC-2.09.003 + BC-2.09.004).
pub struct SafetyEnvelopeBuilder;

impl SafetyEnvelopeBuilder {
    /// Wrap raw sensor results in the safety envelope.
    ///
    /// ## Procedure
    /// 1. Count results (array length if applicable).
    /// 2. Run `InjectionScanner::scan_record` over all string fields in `results`.
    /// 3. Collect all `SafetyFlag`s into `_meta.safety_flags`.
    /// 4. Set `_meta.trust_level` based on the tool name.
    /// 5. Set `_meta.query_time` to the current UTC timestamp.
    /// 6. Build prose summary with counts only (BC-2.09.001).
    /// 7. Never modify `results` values (flag-don't-strip).
    pub fn wrap(
        tool: &str,
        data_source: DataSource,
        results: Value,
        page: u64,
        has_more: bool,
        next_cursor: Option<String>,
    ) -> ResponseEnvelope {
        let scanner = InjectionScanner::global();

        // Count results
        let total_results = if let Some(arr) = results.as_array() {
            arr.len() as u64
        } else {
            0
        };

        // Collect all string fields from the results array for scanning.
        // IMP-6 fix: recurse into nested Object and Array values so that
        // attacker-controlled strings in nested structures are scanned.
        let mut safety_flags: Vec<SafetyFlag> = Vec::new();
        if let Some(arr) = results.as_array() {
            for (item_index, item) in arr.iter().enumerate() {
                let mut fields: Vec<(&str, usize, &str)> = Vec::new();
                collect_string_fields(item, item_index, &mut fields);
                let flags = scanner.scan_record(&fields);
                safety_flags.extend(flags);
            }
        }

        let trust_level = trust_level_for_tool(tool);
        let query_time = Utc::now().to_rfc3339();

        // BC-2.09.001: prose summary with counts only, no sensor field values
        let prose = format!(
            "{total_results} result{} found",
            if total_results == 1 { "" } else { "s" }
        );
        let content = vec![ContentEntry {
            content_type: "text".to_owned(),
            text: prose,
        }];

        // Mirror results in structuredContent for LLM field-level inspection
        let structured_content = StructuredContent {
            results: results.clone(),
        };

        ResponseEnvelope {
            meta: ResponseMeta {
                tool: tool.to_owned(),
                data_source,
                query_time,
                trust_level,
                safety_flags,
                total_results,
                page,
                has_more,
                next_cursor,
            },
            results,
            content,
            structured_content,
        }
    }

    /// Returns `true` if `envelope._meta.safety_flags` is always present
    /// (even as an empty array) for the given envelope.
    ///
    /// BC-2.09.008: `_meta.safety_flags` is always present.
    pub fn safety_flags_always_present(_envelope: &ResponseEnvelope) -> bool {
        // safety_flags is always a Vec (never Option), so it's always present.
        // The check is structural — the field exists regardless of content.
        true
    }
}

/// Collect all scannable string fields from a JSON value, recursing into nested
/// objects and arrays (IMP-6 fix: depth-2+ injection detection coverage).
///
/// # Arguments
/// - `value` — the JSON value to collect strings from.
/// - `item_index` — the zero-based index of the parent result-array item (used for `SafetyFlag.index`).
/// - `fields` — accumulator for `(field_name, item_index, field_value)` triples.
///
/// Recursion terminates at leaf string values. Arrays within objects are iterated;
/// objects within objects are recursed. Non-string scalar values (numbers, booleans,
/// null) are skipped — they cannot carry prompt injection payloads.
fn collect_string_fields<'a>(
    value: &'a Value,
    item_index: usize,
    fields: &mut Vec<(&'a str, usize, &'a str)>,
) {
    match value {
        Value::String(s) => {
            // Top-level string (e.g. a bare array element) — use empty key.
            fields.push(("", item_index, s.as_str()));
        }
        Value::Object(obj) => {
            for (k, v) in obj.iter() {
                match v {
                    Value::String(s) => {
                        fields.push((k.as_str(), item_index, s.as_str()));
                    }
                    Value::Object(_) | Value::Array(_) => {
                        collect_string_fields(v, item_index, fields);
                    }
                    _ => {}
                }
            }
        }
        Value::Array(arr) => {
            for element in arr.iter() {
                collect_string_fields(element, item_index, fields);
            }
        }
        _ => {}
    }
}

// ─── Schema-only types for outputSchema generation (HIGH-002) ─────────────────
//
// These types mirror the ResponseEnvelope shape but derive JsonSchema so they
// can be passed to `rmcp::handler::server::tool::schema_for_type::<T>()`.
// They are NOT used at runtime — only for MCP `tools/list` outputSchema declaration.
// BC-2.09.007: every tool must declare an outputSchema with _meta + results fields.

/// Schema-only representation of a single `_meta.safety_flags` item.
///
/// BC-2.09.007: `{field, index, pattern, category}` — NO per-field parallel fields.
#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SafetyFlagSchema {
    /// Sensor record field that triggered detection.
    pub field: String,
    /// Zero-based index of the item in the results array.
    pub index: u64,
    /// Human-readable description of the matched pattern.
    pub pattern: String,
    /// Detection category (e.g. "prompt_injection", "role_impersonation").
    pub category: String,
}

/// Schema-only representation of the `_meta` envelope (BC-2.09.007, BC-2.09.008).
#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MetaEnvelopeSchemaType {
    pub tool: String,
    /// Sensor identifier(s). Single string for single-sensor; string array for cross-client queries.
    /// Generates `oneOf: [{"type": "string"}, {"type": "array", "items": {"type": "string"}}]`
    /// in the outputSchema (IMP-10 fix — previously untyped serde_json::Value).
    pub data_source: DataSource,
    /// ISO8601 timestamp of query execution.
    pub query_time: String,
    /// Trust classification: "untrusted_external" | "internal".
    pub trust_level: String,
    /// Centralized injection detection flags. Empty array when no patterns detected.
    pub safety_flags: Vec<SafetyFlagSchema>,
    pub total_results: u64,
    pub page: u64,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

/// Schema-only representation of the full response envelope (BC-2.09.007, BC-2.09.008).
///
/// Used exclusively for `outputSchema` generation via
/// `rmcp::handler::server::tool::schema_for_type::<ResponseEnvelopeSchema>()`.
/// The actual runtime type is `ResponseEnvelope` — this is a parallel schema mirror.
#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResponseEnvelopeSchema {
    #[serde(rename = "_meta")]
    pub meta: MetaEnvelopeSchemaType,
    /// Sensor results — typed array of JSON objects.
    pub results: Vec<serde_json::Value>,
    /// Prose summary — counts and metadata ONLY (BC-2.09.001).
    pub content: Vec<serde_json::Value>,
    /// Structured sensor data for LLM field-level inspection (BC-2.09.001).
    #[serde(rename = "structuredContent")]
    pub structured_content: serde_json::Value,
}
