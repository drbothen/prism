//! Column-to-OCSF field mapping at query time (BC-2.16.003).
//!
//! After the pipeline returns raw records, columns with `ocsf_field` mappings
//! are translated to the corresponding OCSF protobuf field using the standard
//! four-tier field resolution (BC-2.02.008). Columns without mappings go to
//! `raw_extensions`. Type coercion is applied with non-fatal fallback.

use prism_core::{PrismError, column::ColumnType};
use serde_json::Value;

use crate::spec_parser::{ColumnSpec, TableSpec};

/// Result of mapping a single raw record to OCSF fields.
#[derive(Debug, Clone)]
pub struct MappingResult {
    /// Fields successfully mapped to OCSF protobuf paths.
    pub mapped_fields: std::collections::HashMap<String, Value>,
    /// Fields placed in raw_extensions (no ocsf_field, or coercion failure).
    pub raw_extensions: std::collections::HashMap<String, Value>,
    /// Coercion warnings for this record.
    pub coercion_warnings: Vec<CoercionWarning>,
}

/// A non-fatal coercion warning: field placed in raw_extensions due to type mismatch.
#[derive(Debug, Clone)]
pub struct CoercionWarning {
    pub column_name: String,
    pub expected_ocsf_type: String,
    pub actual_value: String,
}

/// Maps raw records from a pipeline execution to OCSF fields (BC-2.16.003).
pub struct ColumnMapper;

/// A single column mapping specification.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnMapping {
    /// Source column name from the raw response.
    pub source_column: String,
    /// OCSF field path target (e.g., `"device.ip"`).
    pub ocsf_field_path: String,
}

impl ColumnMapper {
    /// Map a raw JSON record to `MappingResult` using the table's ColumnSpec entries.
    ///
    /// - Columns with `ocsf_field` -> OCSF protobuf field
    /// - Columns without `ocsf_field` -> raw_extensions
    /// - Type coercion failures -> raw_extensions + CoercionWarning (non-fatal)
    /// - Records are NEVER dropped (invariant BC-2.16.003)
    ///
    /// ENRICH-1: when `col.source_path` is `Some(path)`, extraction uses
    /// `extract_at_path(raw, path)` instead of the flat `raw.get(&col.name)` lookup.
    /// Wildcard paths (`[*]`) that yield a `Value::Array` are serialized to a compact
    /// JSON-list string (e.g., `["v1","v2"]`). Empty array → `"[]"` (not null).
    /// On `extract_at_path` error, the column is skipped with a `tracing::warn!` emission.
    pub fn map_record(raw: &Value, table: &TableSpec) -> Result<MappingResult, PrismError> {
        let mut mapped_fields = std::collections::HashMap::new();
        let mut raw_extensions = std::collections::HashMap::new();
        let mut coercion_warnings = Vec::new();

        for col in &table.columns {
            // ENRICH-1: dispatch on source_path vs flat key lookup.
            let raw_value = if let Some(ref path) = col.source_path {
                // source_path extraction via extract_at_path (ENRICH-1 §Design Decision 1).
                match crate::pipeline::extract_at_path(raw, path) {
                    Ok(Value::Array(arr)) => {
                        // Wildcard result: serialize to compact JSON-list string.
                        // Empty array → "[]" (distinguishable from absent field).
                        // Design Decision 2: JSON-list string in string column.
                        let strings: Vec<String> = arr
                            .iter()
                            .map(|v| match v {
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            })
                            .collect();
                        let json_list =
                            serde_json::to_string(&strings).unwrap_or_else(|_| "[]".to_string());
                        Value::String(json_list)
                    }
                    Ok(v) => v,
                    Err(e) => {
                        // Extraction failed (path absent or type mismatch) — skip column.
                        // Emit structured warning per SAP-1 (ENRICH-1 §Design Decision 1).
                        tracing::warn!(
                            column = %col.name,
                            source_path = %path,
                            error = %e,
                            event_type = "column_source_path_extraction_failed",
                            "ENRICH-1: source_path extraction failed; column skipped"
                        );
                        continue;
                    }
                }
            } else {
                // Fast-path: flat key lookup (pre-ENRICH-1 behavior, fully backward compat).
                match raw.get(&col.name) {
                    Some(v) => v.clone(),
                    None => {
                        // Column not present in record — skip (no error, no raw_extension)
                        continue;
                    }
                }
            };

            match &col.ocsf_field {
                Some(ocsf_path) => {
                    // Attempt type coercion for the OCSF field
                    match Self::coerce_value(&raw_value, col, ocsf_path) {
                        Ok(coerced) => {
                            mapped_fields.insert(ocsf_path.clone(), coerced);
                        }
                        Err(warning) => {
                            // Coercion failed: emit structured audit warn, then divert to
                            // raw_extensions.  AC-004 / BC-2.16.003 §Coercion Warning
                            // Observability: structured fields MUST be column, column_type,
                            // actual_json_kind (consistent across all 3 emission sites).
                            let actual_json_kind = if raw_value.is_array() {
                                "array"
                            } else if raw_value.is_object() {
                                "object"
                            } else if raw_value.is_string() {
                                "string"
                            } else {
                                // N4 defensive fallback: BC-2.16.002 row 95 declares
                                // `actual_json_kind` as one of "array"/"object"/"string".
                                // All current Err-producing code paths reach array/object/string,
                                // so this arm is unreachable today.  If a new Err variant is added
                                // in a future story, the emitted "unknown" will be outside the
                                // declared domain — update this chain and BC-2.16.002 row 95 at
                                // that time.
                                "unknown"
                            };
                            tracing::warn!(
                                column = %col.name,
                                column_type = %column_type_toml_name(&col.column_type),
                                actual_json_kind = %actual_json_kind,
                                event_type = "column_coercion_failure",
                                "column coercion failed; value demoted to raw_extensions \
                                 (BC-2.16.003)"
                            );
                            raw_extensions.insert(col.name.clone(), raw_value);
                            coercion_warnings.push(warning);
                        }
                    }
                }
                None => {
                    // No ocsf_field mapping -> raw_extensions
                    raw_extensions.insert(col.name.clone(), raw_value);
                }
            }
        }

        Ok(MappingResult {
            mapped_fields,
            raw_extensions,
            coercion_warnings,
        })
    }

    /// Apply type coercion for a single column value.
    ///
    /// Returns `Ok(coerced_value)` on success, `Err(CoercionWarning)` on failure.
    /// The caller places failed values in raw_extensions and continues (never drops record).
    ///
    /// ## String-type-first rule (LIVE-DRIFT-003)
    ///
    /// When `column.column_type == ColumnType::String`, any scalar JSON value is normalized
    /// to a JSON string BEFORE the `is_numeric_ocsf_field` heuristic fires. This handles:
    /// 1. API returns integer IDs (e.g. `"id": 132` on alerts) but spec declares `column_type =
    ///    "string"` for polymorphic ID normalization (EC-016-013-004).
    /// 2. String usernames mapped to `actor.user.uid` (suffix `uid` is in the numeric list);
    ///    without this rule, a string username triggers a CoercionWarning and goes to
    ///    raw_extensions instead of the OCSF mapped field.
    pub fn coerce_value(
        value: &Value,
        column: &ColumnSpec,
        ocsf_field_path: &str,
    ) -> Result<Value, CoercionWarning> {
        // String-type-first: when the spec declares the column as string, normalize any
        // scalar to a JSON string value before the numeric-suffix heuristic is checked.
        if column.column_type == ColumnType::String {
            return match value {
                Value::String(_) => Ok(value.clone()),
                Value::Number(n) => Ok(Value::String(n.to_string())),
                Value::Bool(b) => Ok(Value::String(b.to_string())),
                // Null passes through and lands in mapped_fields as Value::Null
                // (only an *absent* key is skipped by map_record, not an explicit null).
                Value::Null => Ok(value.clone()),
                // Array and Object are structured types that cannot be safely coerced to a
                // String column — stringifying them produces opaque JSON blobs that corrupt
                // downstream typed Arrow columns.  Return CoercionWarning to divert the value
                // to raw_extensions (AC-001 / AC-002 / EC-016-013-007 / EC-016-013-008).
                Value::Array(_) => Err(CoercionWarning {
                    column_name: column.name.clone(),
                    expected_ocsf_type: "string".to_string(),
                    actual_value: "array".to_string(),
                }),
                Value::Object(_) => Err(CoercionWarning {
                    column_name: column.name.clone(),
                    expected_ocsf_type: "string".to_string(),
                    actual_value: "object".to_string(),
                }),
            };
        }

        // Integer-column Object demotion (AC-008 / EC-016-013-030):
        // Object values cannot be coerced to an Integer column — they must be diverted to
        // raw_extensions via CoercionWarning, symmetric with the String-branch Object handling
        // above (AC-002).  map_record emits the column_coercion_failure warn at demotion time.
        if column.column_type == ColumnType::Integer && matches!(value, Value::Object(_)) {
            return Err(CoercionWarning {
                column_name: column.name.clone(),
                expected_ocsf_type: "integer".to_string(),
                actual_value: "object".to_string(),
            });
        }

        // Integer-column String coercion (AC-003 / EC-016-013-009):
        // When column_type is Integer and the input is a JSON string, attempt s.parse::<i64>()
        // regardless of OCSF path suffix.  The column_type declaration is authoritative and
        // extends Rule 2 (numeric-suffix heuristic) to ALL Integer-declared columns.
        if column.column_type == ColumnType::Integer
            && let Value::String(s) = value
        {
            if let Ok(n) = s.parse::<i64>() {
                return Ok(Value::Number(serde_json::Number::from(n)));
            }
            return Err(CoercionWarning {
                column_name: column.name.clone(),
                expected_ocsf_type: "integer".to_string(),
                actual_value: s.clone(),
            });
        }

        // Rule 2 (numeric-suffix heuristic): for non-Integer-declared columns whose OCSF path
        // ends in a known numeric suffix, also attempt String→integer coercion.
        // (Integer-declared columns with String input are already handled above.)
        let target_is_numeric = is_numeric_ocsf_field(ocsf_field_path);

        if target_is_numeric {
            // Try to coerce string to integer
            if let Value::String(s) = value {
                if let Ok(n) = s.parse::<i64>() {
                    return Ok(Value::Number(serde_json::Number::from(n)));
                }
                // Failed coercion — produce warning
                return Err(CoercionWarning {
                    column_name: column.name.clone(),
                    expected_ocsf_type: "integer".to_string(),
                    actual_value: s.clone(),
                });
            }
        }

        // Default: return value as-is (no coercion needed)
        Ok(value.clone())
    }
}

/// Heuristic: OCSF fields whose last path segment is a known numeric field name
/// are treated as integer targets for coercion purposes.
/// This is a simplified model — the full implementation uses the embedded OCSF schema.
fn is_numeric_ocsf_field(path: &str) -> bool {
    // NOTE: `uid` is in this list — any OCSF field ending in `.uid` would try string→int
    // coercion without the String-type-first guard in `coerce_value` above.
    let numeric_suffixes = [
        "event_code",
        "class_uid",
        "activity_id",
        "type_uid",
        "severity_id",
        "status_id",
        "action_id",
        "count",
        "duration",
        "port",
        "pid",
        "uid",
        "code",
    ];
    let last_segment = path.split('.').next_back().unwrap_or(path);
    numeric_suffixes.contains(&last_segment)
}

/// Return the TOML canonical name for a `ColumnType` (lowercase, matching serde rename_all).
///
/// Used to populate the `column_type` field of the `column_coercion_failure` structured warn
/// across all emission sites.  Consistent with the TOML `type =` declaration in sensor specs.
fn column_type_toml_name(col_type: &ColumnType) -> &'static str {
    match col_type {
        ColumnType::String => "string",
        ColumnType::Integer => "integer",
        ColumnType::Float => "float",
        ColumnType::Boolean => "boolean",
        ColumnType::Datetime => "datetime",
        ColumnType::Json => "json",
        // N4 defensive fallback: BC-2.16.002 row 95 declares `column_type` as one of the six
        // TOML type names above.  The wildcard arm is required because `ColumnType` is
        // `#[non_exhaustive]` — adding a new variant here without extending the match would
        // silently emit "unknown" (outside the declared domain) rather than failing at compile
        // time.  If a new `ColumnType` variant is added, extend this match and update
        // BC-2.16.002 row 95's declared domain at the same time.
        _ => "unknown",
    }
}

// ---------------------------------------------------------------------------
// Tests — column_mapping coerce_value behavior
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use prism_core::column::ColumnType;
    use serde_json::{Value, json};

    use crate::spec_parser::{ColumnSpec, FetchStep, TableSpec};

    use super::ColumnMapper;

    /// Wire-shape assertion: when `column_type = "string"` and the API returns a JSON integer
    /// (e.g. Claroty alerts `"id": 132`), `coerce_value` must produce `Value::String("132")`.
    ///
    /// Before the String-type-first fix, the integer fell through to `Ok(value.clone())`
    /// returning a `Value::Number` — the downstream DataFusion schema expected a string column
    /// but received an integer, causing silent type mismatches.
    ///
    /// This test asserts the wire-level output: `Value::String("132")` NOT `Value::Number(132)`.
    #[test]
    fn test_coerce_value_string_type_normalizes_integer_to_string() {
        // Simulates claroty alerts `id` column: column_type="string", ocsf_field="finding.uid".
        // `uid` is in the is_numeric_ocsf_field suffix list — without the String-type-first fix
        // the numeric heuristic would fail to convert the Number (not a String input) and
        // fall through to Ok(value.clone()), returning a Value::Number.
        let col = ColumnSpec {
            name: "id".to_string(),
            column_type: ColumnType::String,
            ocsf_field: Some("finding.uid".to_string()),
            ..Default::default()
        };

        let integer_value = json!(132u32);
        let result = ColumnMapper::coerce_value(&integer_value, &col, "finding.uid")
            .expect("coerce_value must succeed for string column with integer input");

        assert_eq!(
            result,
            Value::String("132".to_string()),
            "string column receiving integer JSON value must be normalized to JSON string at wire; \
             got: {result:?} (LIVE-DRIFT-003, EC-016-013-004)"
        );
    }

    /// Wire-shape assertion: `column_type = "string"` with `ocsf_field = "actor.user.uid"`
    /// (the `username` audit_log column) must preserve string values without CoercionWarning.
    ///
    /// Before the String-type-first fix, a string username like "jdoe" would trigger the
    /// `is_numeric_ocsf_field("actor.user.uid")` → true path, attempt `s.parse::<i64>()`,
    /// fail, and return a CoercionWarning — pushing the value to raw_extensions instead of
    /// the mapped OCSF field.
    #[test]
    fn test_coerce_value_string_type_preserves_string_username_against_uid_heuristic() {
        let col = ColumnSpec {
            name: "username".to_string(),
            column_type: ColumnType::String,
            ocsf_field: Some("actor.user.uid".to_string()),
            ..Default::default()
        };

        let string_value = json!("analyst");
        let result = ColumnMapper::coerce_value(&string_value, &col, "actor.user.uid").expect(
            "coerce_value must NOT return CoercionWarning for string username (LIVE-DRIFT-003)",
        );

        assert_eq!(
            result,
            Value::String("analyst".to_string()),
            "string column with uid-path ocsf_field must preserve the string value; \
             got: {result:?}"
        );
    }

    /// RG-005 / AC-004 / BC-2.16.003 §Coercion Warning Observability DEFECT:
    ///
    /// `map_record` MUST:
    ///   (a) divert a String+Object value to `raw_extensions` (not `mapped_fields`), AND
    ///   (b) emit `tracing::warn!(event_type = "column_coercion_failure", ...)` at the
    ///       demotion point.
    ///
    /// **Placement:** in-crate (`prism_spec_engine::column_mapping::tests`). The default
    /// `tracing-test` env-filter is `<test_crate>=trace`. For an in-crate test the crate is
    /// `prism_spec_engine`, so the filter captures `column_mapping` warn events.
    /// An integration test in `tests/bc_2_16_003_test.rs` would use filter
    /// `bc_2_16_003_test=trace`, which EXCLUDES `prism_spec_engine` events — hence the
    /// in-crate placement (mirrors pipeline.rs `#[tracing_test::traced_test]` precedent).
    ///
    /// **Red gate:** Before AC-001 fix, `coerce_value` returns `Ok(Object)` for String+Object
    /// → `map_record` places it in `mapped_fields`, not `raw_extensions`. Assertion (a) fails.
    /// Before AC-004 fix, no `column_coercion_failure` warn is emitted. Assertion (b) fails.
    ///
    /// SAP-3 reachability note (defense-in-depth): `map_record` is on Path B (zero live
    /// production callers per ADR-058 §K5) — this test is intentionally defense-in-depth /
    /// forward-compat per SAP-3 rule 2/3. LIVE behavior is covered by RG-006/RG-009.
    #[test]
    #[tracing_test::traced_test]
    fn test_map_record_string_object_input_demotes_to_raw_extensions_and_emits_warning() {
        let table = TableSpec::new_point_in_time(
            "alerts",
            "security_finding",
            vec![ColumnSpec::new(
                "metadata",
                ColumnType::String,
                Some("finding.metadata".to_string()),
                vec![],
            )],
            vec![FetchStep::new(
                "fetch",
                "GET",
                "/data",
                None,
                "$.data",
                None,
                vec![],
                None,
                None,
            )],
        );
        let raw = json!({ "metadata": {"nested": "object"} });

        let result = ColumnMapper::map_record(&raw, &table).expect("map_record must not error");

        // (a) Object value must be diverted to raw_extensions, NOT mapped_fields.
        assert!(
            result.raw_extensions.contains_key("metadata"),
            "AC-004 / EC-016-013-008: String column + Object input must be demoted to \
             raw_extensions (coerce_value must return Err, not Ok(Object) pass-through)"
        );
        assert!(
            !result.mapped_fields.contains_key("finding.metadata"),
            "AC-004: String column + Object input must NOT appear in mapped_fields"
        );
        // (b) column_coercion_failure warn must be emitted at the demotion point.
        assert!(
            logs_contain("column_coercion_failure"),
            "AC-004 / BC-2.16.003 §Coercion Warning Observability DEFECT: \
             map_record must emit tracing::warn!(event_type = 'column_coercion_failure') \
             when demoting a String+Object value to raw_extensions. \
             Currently no warn is emitted (BC-2.02.011 violation)."
        );
        // (b-SID-2) Verify structured fields — SID-2 requires asserting the FULL composed
        // emitted string, not only the event_type key.  These three assertions together verify
        // the complete structured warn schema (column, column_type, actual_json_kind).
        assert!(
            logs_contain("column_type=string"),
            "SID-2 / AC-004: column_coercion_failure warn must include structured field \
             column_type=string (BC-2.16.003 §Coercion Warning Observability)"
        );
        assert!(
            logs_contain("actual_json_kind=object"),
            "SID-2 / AC-004: column_coercion_failure warn must include structured field \
             actual_json_kind=object (BC-2.16.003 §Coercion Warning Observability)"
        );
        assert!(
            logs_contain("column=metadata"),
            "SID-2 / AC-004: column_coercion_failure warn must include structured field \
             column=metadata identifying the demoted column (BC-2.16.003 §Coercion Warning Observability)"
        );
    }

    /// RG-011 / AC-008 / BC-2.16.003 §Path-B Integer+Object coercion gap (EC-016-013-030)
    ///
    /// `ColumnMapper::coerce_value` with `column_type = ColumnType::Integer` and a
    /// `Value::Object` input MUST return `Err(CoercionWarning)` — symmetric with the
    /// String-branch `Value::Object` handling added in AC-002 (`coerce_value` with
    /// `column_type = ColumnType::String` + `Value::Object`).
    ///
    /// **Red gate:** Current code has no explicit `Value::Object` arm in the Integer
    /// branch.  The Integer+String block (`if column.column_type == ColumnType::Integer &&
    /// let Value::String(s) = value`) does not match `Value::Object`, and the subsequent
    /// `is_numeric_ocsf_field` block only acts on `Value::String`.  `Value::Object` falls
    /// through to the final `Ok(value.clone())` pass-through at the bottom of
    /// `coerce_value`.  The test asserts `Err(CoercionWarning)` but receives `Ok(Object)`.
    ///
    /// This is a pure return-value assertion (no tracing capture needed for RG-011 itself
    /// — Path B emits the warn via `map_record` at demotion time, not inside
    /// `coerce_value`).
    ///
    /// Covers AC-008 Path B.
    ///
    /// SAP-3 reachability note (defense-in-depth): `coerce_value` is on Path B
    /// (`ColumnMapper::coerce_value` in `column_mapping.rs`), which has zero live
    /// production callers per ADR-058 §K5.  This test is intentionally defense-in-depth /
    /// forward-compat per SAP-3 rule 2/3.  The equivalent LIVE coverage on Path A is
    /// RG-010 (`build_column_array` Integer+Object → null+warn) in `spec_driven_adapter.rs`.
    #[test]
    fn test_coerce_value_integer_type_object_input_returns_err_coercion_warning() {
        let col = ColumnSpec {
            name: "severity_id".to_string(),
            column_type: ColumnType::Integer,
            ocsf_field: Some("finding.severity_id".to_string()),
            ..Default::default()
        };

        let object_value = json!({"nested": "object"});
        let result = ColumnMapper::coerce_value(&object_value, &col, "finding.severity_id");

        assert!(
            result.is_err(),
            "AC-008 / EC-016-013-030: coerce_value with Integer column + Object input MUST \
             return Err(CoercionWarning); current code falls through to Ok(value.clone()) \
             pass-through (no explicit Value::Object arm in the Integer branch). \
             Got: Ok({:?})",
            result.ok()
        );
        let warning = result.unwrap_err();
        assert_eq!(
            warning.column_name, "severity_id",
            "AC-008: CoercionWarning must carry the column name"
        );
        assert_eq!(
            warning.expected_ocsf_type, "integer",
            "AC-008: CoercionWarning must carry expected_ocsf_type = \"integer\""
        );
        assert_eq!(
            warning.actual_value, "object",
            "AC-008: CoercionWarning must carry actual_value = \"object\" \
             (matching the actual_json_kind field schema from BC-2.16.002 catalog row 95)"
        );
    }
}
