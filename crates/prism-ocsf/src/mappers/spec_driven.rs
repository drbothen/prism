//! Spec-driven OCSF field mapper — reads `ocsf_field` column annotations from the
//! spec-catalog and dispatches to WASM plugins for complex transforms.
//!
//! # Architecture (PLUGIN-MIGRATION-001-C)
//!
//! `SpecDrivenMapper` replaces the four hardcoded sensor mapper modules
//! (crowdstrike, cyberint, claroty, armis) with a single config-driven implementation
//! that reads `ocsf_field` column annotations from `TableSpec` / `ColumnSpec` at
//! runtime and dispatches complex transforms to WASM plugins via `PluginRuntime`.
//!
//! # Behavioral Contracts
//!
//! - VP-PLUGIN-006: spec-driven field mapping correctness

use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use prism_core::{ColumnType, PrismError};
use prism_spec_engine::{ConfigManager, PluginRuntime};
use prost_reflect::{DynamicMessage, Kind, ReflectMessage, Value as ProtoValue};

/// Spec-driven OCSF field mapper.
///
/// Holds an `Arc<ConfigManager>` for live sensor-spec lookup (supports hot reload)
/// and an `Arc<PluginRuntime>` for WASM plugin dispatch on complex transforms.
///
/// The mapper reads `ocsf_field` column annotations from the spec-catalog
/// `TableSpec` / `ColumnSpec` at `map()` call time, so spec reloads are
/// reflected on the next call without restart.
///
/// Each `SpecDrivenMapper` instance is bound to a single sensor ID — the sensor ID
/// is derived at construction time from the first (and typically only) sensor in the
/// config snapshot. The boot path constructs one `SpecDrivenMapper` per sensor.
///
/// # Memory Note on `Box::leak` usage
///
/// `sensor_id` is derived from the config snapshot at construction time and stored as
/// a `&'static str` to satisfy the `SensorMapper` trait signature. `Box::leak` is used
/// to produce the `'static` lifetime.
///
/// SAFETY: bounded leak — one allocation per sensor at boot time. In production this is
/// at most one allocation per registered sensor (typically 4–10). In test runs, each
/// `SpecDrivenMapper::new()` call leaks a string (~8–32 bytes). A 1000-test run leaks at
/// most ~32KB — well within the OS reclaim-at-process-exit guarantee. A static
/// interning cache would avoid this but adds complexity not justified by the memory
/// profile at this usage frequency.
#[non_exhaustive]
pub struct SpecDrivenMapper {
    /// Live spec catalog — supports ArcSwap hot reload per ADR-007 / AD-018.
    config_manager: Arc<ConfigManager>,
    /// WASM plugin runtime for complex field transforms.
    plugin_runtime: Arc<PluginRuntime>,
    /// Sensor identifier for this mapper instance, derived from the config snapshot
    /// at construction time. Leaked to `&'static str` to satisfy the `SensorMapper`
    /// trait signature.
    sensor_id: &'static str,
}

impl SpecDrivenMapper {
    /// Construct a `SpecDrivenMapper` with the given config manager and plugin runtime.
    ///
    /// Derives the sensor ID from the first sensor in the current config snapshot.
    /// Returns a mapper bound to that sensor ID.
    ///
    /// # Parameters
    ///
    /// - `config_manager`: live spec catalog (Arc<ConfigManager> per ADR-022 Arc-DI wiring).
    /// - `plugin_runtime`: WASM plugin runtime for dispatching complex transforms.
    pub fn new(config_manager: Arc<ConfigManager>, plugin_runtime: Arc<PluginRuntime>) -> Self {
        // Derive sensor_id from the config snapshot at construction time.
        // The boot path constructs one SpecDrivenMapper per sensor, so the snapshot
        // typically contains exactly one sensor. We take the first sensor_id found.
        // Box::leak is used to produce a &'static str to satisfy the SensorMapper trait.
        let snapshot = config_manager.load();
        let sensor_id: &'static str = snapshot
            .sensor_specs
            .keys()
            .next()
            .map(|s| Box::leak(s.clone().into_boxed_str()) as &'static str)
            .unwrap_or_else(|| {
                // F-LP2-MED-001: warn when config has no sensors — empty sensor_id will cause
                // every subsequent map() call to fail with OcsfNormalizationFailed (sensor not
                // found). The warning makes this misconfiguration observable in operator logs.
                tracing::warn!(
                    event_type = "ocsf.spec_driven_mapper_empty_sensor_id",
                    "SpecDrivenMapper constructed with empty config snapshot (no sensors); \
                     all map() calls will return OcsfNormalizationFailed until config is populated"
                );
                ""
            });

        Self {
            config_manager,
            plugin_runtime,
            sensor_id,
        }
    }
}

impl super::SensorMapper for SpecDrivenMapper {
    /// Returns the sensor identifier for this mapper instance.
    ///
    /// Derived at construction time from the config snapshot's first sensor.
    fn sensor_id(&self) -> &'static str {
        self.sensor_id
    }

    /// Returns the record types this mapper handles.
    ///
    /// The spec-driven mapper handles all record types — the specific table is looked
    /// up by `record_type` in the sensor spec at `map()` time. Returns a static
    /// wildcard slice that matches all incoming record types.
    fn record_types(&self) -> &'static [&'static str] {
        // The spec-driven mapper accepts all record types; the normalizer dispatches
        // by sensor_id only, not by record_type, so this slice is informational.
        // We return a well-known sentinel that documents this intent.
        &["*"]
    }

    /// Maps vendor-specific fields from `raw` into `msg` (known OCSF paths) and
    /// `extensions` (unknown fields) using `ocsf_field` annotations from the spec-catalog.
    ///
    /// For each column in the matching sensor table spec:
    /// - `ColumnType::Json` columns → WASM dispatch via `PluginRuntime` (when plugin
    ///   present, raw value placed in `extensions` with a deferred-dispatch warning;
    ///   when plugin absent, returns `OcsfNormalizationFailed`)
    /// - `ColumnType::Datetime` columns → RFC3339 parse → epoch-millis written to `msg`;
    ///   on parse failure the raw value is placed in `extensions` and a warning emitted
    /// - `ColumnType::Integer` columns with `ocsf_field` → int-to-string cast written to `msg`
    /// - All other typed columns with `ocsf_field` → direct string representation written to `msg`
    /// - Columns without `ocsf_field` → placed into `extensions` (BC-2.02.007)
    /// - Columns present in spec but absent in raw → placed as Null in `extensions` (HIGH-005)
    ///
    /// The first column with `ocsf_field` mapped is returned as the `source_id`.
    ///
    /// # Errors
    ///
    /// - `PrismError::OcsfNormalizationFailed` — required field missing or type mismatch,
    ///   or no ocsf_transform plugin registered for a Json column.
    /// - `PrismError::OcsfTimestampParseError` — timestamp could not be parsed.
    fn map(
        &self,
        record_type: &str,
        raw: &serde_json::Value,
        msg: &mut DynamicMessage,
        extensions: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<String, PrismError> {
        let snapshot = self.config_manager.load();

        // Look up the sensor spec for this mapper's sensor_id.
        let sensor_spec = snapshot.sensor_specs.get(self.sensor_id).ok_or_else(|| {
            PrismError::OcsfNormalizationFailed {
                source_id: format!("<{}>", self.sensor_id),
                reason: format!("no sensor spec found for sensor '{}'", self.sensor_id),
            }
        })?;

        // Find the table descriptor matching record_type.
        // The table_name in spec_parser::SensorSpec is UNQUALIFIED (e.g., "events", not
        // "crowdstrike.events"). TOML loading stores the raw table name; qualification as
        // "{sensor_id}.{table_name}" happens at DataFusion registration time in
        // sensor_table_descriptor_from_table_spec. We compare record_type directly
        // against t.table_name (unqualified), using two strategies only
        // (F-LP2-MED-002: the overly-broad prefix-fallback has been removed):
        //   1. Exact match: record_type == t.table_name (e.g., "detections" == "detections")
        //   2. Plural match: record_type + "s" == t.table_name (e.g., "detection" → "detections")
        // This handles the common case where TOML specs use plural table names
        // (e.g., "detections") but the normalizer uses singular record_type
        // ("detection") derived from EventClassSelector.
        // The prefix fallback (`suffix.starts_with(record_type)`) was removed because it
        // could match wrong tables (e.g., "detection" matching "detections_v2").
        let plural_record_type = format!("{}s", record_type);
        let table_descriptor = sensor_spec
            .tables
            .iter()
            .find(|t| t.table_name == record_type || t.table_name == plural_record_type)
            .ok_or_else(|| PrismError::OcsfNormalizationFailed {
                source_id: format!("<{}>", self.sensor_id),
                reason: format!(
                    "no table spec found for record_type '{}' in sensor '{}'",
                    record_type, self.sensor_id
                ),
            })?;

        let raw_obj = raw
            .as_object()
            .ok_or_else(|| PrismError::OcsfNormalizationFailed {
                source_id: format!("<{}>", self.sensor_id),
                reason: "raw record is not a JSON object".to_owned(),
            })?;

        // Track the first mapped ocsf_field value as source_id (BC-2.02.002 postcondition).
        let mut source_id: Option<String> = None;

        for col in &table_descriptor.columns {
            let raw_value = raw_obj.get(&col.name);

            if let Some(ocsf_target) = &col.ocsf_field {
                // This column has an OCSF field mapping.

                // F-LP2-HIGH-001 + HIGH-005: if the column is declared in spec but absent
                // in raw (None) OR explicitly null in raw (Some(Value::Null)), treat both
                // identically — place Null in extensions and emit a debug trace.
                // `Some(Value::Null).to_string()` would produce the string `"null"` which
                // is incorrect; we must handle the null case explicitly here.
                if raw_value.is_none() || raw_value == Some(&serde_json::Value::Null) {
                    extensions.insert(col.name.clone(), serde_json::Value::Null);
                    tracing::debug!(
                        sensor_id = %self.sensor_id,
                        column = %col.name,
                        ocsf_field = %ocsf_target,
                        event_type = "ocsf.spec_column_absent_in_raw",
                        "spec-declared column absent or null in raw record; inserted Null into extensions"
                    );
                    continue;
                }

                match col.column_type {
                    ColumnType::Json => {
                        // Complex transform path — dispatch to WASM plugin (AC-002/AC-003).
                        // Construct the plugin_id from convention: "{sensor_id}-ocsf-transform".
                        let plugin_id = format!("{}-ocsf-transform", self.sensor_id);
                        let plugin_result = self.plugin_runtime.get_plugin(&plugin_id);

                        match plugin_result {
                            Ok(_plugin) => {
                                // CRIT-002: Plugin is present but the WIT call_ocsf_transform
                                // interface does not yet exist on PluginRuntime. Per BC-2.02.007,
                                // preserve the raw value in extensions rather than silently
                                // dropping it (production-grade graceful degradation).
                                // The ocsf.wasm_dispatch_deferred warning makes this observable
                                // to operators (EC-009 edge case).
                                if let Some(v) = raw_value {
                                    extensions.insert(col.name.clone(), v.clone());
                                    if source_id.is_none() {
                                        source_id = Some(v.to_string());
                                    }
                                }
                                tracing::warn!(
                                    sensor_id = %self.sensor_id,
                                    column = %col.name,
                                    ocsf_field = %ocsf_target,
                                    plugin_id = %plugin_id,
                                    event_type = "ocsf.wasm_dispatch_deferred",
                                    "WASM plugin found but call_ocsf_transform interface not yet \
                                     wired; raw column value preserved in extensions (BC-2.02.007)"
                                );
                            }
                            Err(_) => {
                                // No plugin registered — return structured error (AC-003).
                                return Err(PrismError::OcsfNormalizationFailed {
                                    source_id: source_id
                                        .clone()
                                        .unwrap_or_else(|| format!("<{}>", self.sensor_id)),
                                    reason: format!(
                                        "no ocsf_transform plugin registered for sensor '{}' \
                                         table '{}' — column '{}' requires WASM transform to \
                                         map to OCSF field '{}'",
                                        self.sensor_id, record_type, col.name, ocsf_target
                                    ),
                                });
                            }
                        }
                    }
                    ColumnType::Datetime => {
                        // RFC3339 timestamp → epoch-millis (AC-001 part 2).
                        // CRIT-003: Replace `?` with graceful degradation on parse failure.
                        // On error, place raw value in extensions and emit a warning rather
                        // than aborting the entire record (EC-004, BC-2.02.002).
                        if let Some(v) = raw_value {
                            match parse_timestamp_to_epoch_ms(&col.name, v) {
                                Ok(epoch_ms) => {
                                    let mapped_str = epoch_ms.to_string();
                                    // Write epoch-millis as I64 into the OCSF DynamicMessage.
                                    // set_nested_field handles dotted paths (e.g. "time") and
                                    // is a no-op when the field does not exist in the descriptor
                                    // (stub or partial schema) — safe.
                                    set_nested_field(msg, ocsf_target, ProtoValue::I64(epoch_ms));
                                    if source_id.is_none() {
                                        source_id = Some(mapped_str);
                                    }
                                }
                                Err(e) => {
                                    // EC-004: timestamp parse failure degrades gracefully.
                                    // Preserve raw value in extensions so no data is lost.
                                    extensions.insert(col.name.clone(), v.clone());
                                    tracing::warn!(
                                        sensor_id = %self.sensor_id,
                                        field = %col.name,
                                        ocsf_field = %ocsf_target,
                                        error = %e,
                                        event_type = "ocsf.timestamp_parse_failed",
                                        "timestamp parse failed; raw value preserved in extensions"
                                    );
                                }
                            }
                        }
                    }
                    ColumnType::Integer => {
                        // Integer → string cast for OCSF string fields (AC-001 part 3).
                        // AC-001 specifies "integer-to-string cast" — the OCSF target field
                        // is typed as string, so we always write ProtoValue::String (F-LP2-MED-003).
                        if let Some(v) = raw_value {
                            let mapped_str = if let Some(n) = v.as_i64() {
                                n.to_string()
                            } else if let Some(n) = v.as_u64() {
                                n.to_string()
                            } else {
                                v.to_string()
                            };
                            // Write integer-as-string into OCSF DynamicMessage via nested path.
                            set_nested_field(
                                msg,
                                ocsf_target,
                                ProtoValue::String(mapped_str.clone()),
                            );
                            if source_id.is_none() {
                                source_id = Some(mapped_str);
                            }
                        }
                        // Integer column with ocsf_field — do NOT add to extensions.
                    }
                    _ => {
                        // String, Float, Boolean, or other — direct mapping.
                        // Extract string representation and write to OCSF DynamicMessage (CRIT-001).
                        if let Some(v) = raw_value {
                            let (mapped_str, proto_val) = match v {
                                serde_json::Value::String(s) => {
                                    (s.clone(), ProtoValue::String(s.clone()))
                                }
                                serde_json::Value::Bool(b) => (b.to_string(), ProtoValue::Bool(*b)),
                                serde_json::Value::Number(n) => {
                                    if let Some(i) = n.as_i64() {
                                        (n.to_string(), ProtoValue::I64(i))
                                    } else if let Some(f) = n.as_f64() {
                                        (n.to_string(), ProtoValue::F64(f))
                                    } else {
                                        (n.to_string(), ProtoValue::String(n.to_string()))
                                    }
                                }
                                other => (other.to_string(), ProtoValue::String(other.to_string())),
                            };
                            // set_nested_field handles both flat names and dotted paths
                            // (e.g. "finding_info.uid"). No-op when field doesn't exist in
                            // the descriptor (stub descriptor or partial OCSF schema) — safe.
                            set_nested_field(msg, ocsf_target, proto_val);
                            if source_id.is_none() {
                                source_id = Some(mapped_str);
                            }
                        }
                        // Column with ocsf_field — do NOT add to extensions.
                    }
                }
            } else {
                // No ocsf_field — preserve in extensions (BC-2.02.007).
                if let Some(v) = raw_value {
                    extensions.insert(col.name.clone(), v.clone());
                }
                // If absent in raw, nothing to do — undeclared-ocsf columns with no
                // raw value are simply not present (no Null insertion needed here).
            }
        }

        // BC-2.02.007: Preserve any raw fields NOT declared in the spec into extensions.
        // The spec's column list is the declared schema; fields beyond it are vendor-specific
        // extras that must not be silently dropped.
        let declared_column_names: std::collections::HashSet<&str> = table_descriptor
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        for (key, value) in raw_obj {
            if !declared_column_names.contains(key.as_str()) {
                // Field not declared in the spec — preserve in extensions.
                extensions
                    .entry(key.clone())
                    .or_insert_with(|| value.clone());
            }
        }

        // BC-2.02.002 postcondition: return source record ID.
        // Fall back to a synthesis from raw data if no ID was captured.
        let final_source_id = source_id.unwrap_or_else(|| {
            // Use the first string value in the raw object as a fallback identifier.
            raw_obj
                .values()
                .find_map(|v| v.as_str())
                .map(|s| s.to_owned())
                .unwrap_or_else(|| format!("<{}>", self.sensor_id))
        });

        Ok(final_source_id)
    }
}

/// Set a field in a `DynamicMessage` using a dotted-path OCSF field name.
///
/// `ocsf_field` is either:
/// - A flat name like `"severity"` → calls `set_field_by_name` directly.
/// - A dotted path like `"finding_info.uid"` → navigates the intermediate sub-message
///   chain, creating or updating each intermediate `Value::Message`, and sets the leaf field.
///
/// ## Stub-Descriptor Safety
///
/// When the OCSF descriptor pool is a stub (zero bytes at build time), the message has no
/// fields. `get_field_by_name` returns `None`, and `set_nested_field` silently no-ops —
/// identical to the existing `set_field_by_name` behavior for unknown fields. Tests that
/// use the stub descriptor cannot assert on DynamicMessage field values; the real test
/// for field correctness uses the actual OCSF descriptor pool (F-LP2-HIGH-002 compliance).
///
/// ## Multi-Segment Path Algorithm
///
/// For a path `"a.b.c"`:
/// 1. Look up field `"a"` on `msg.descriptor()`. If not found → no-op (stub).
/// 2. Verify field `"a"` has Kind::Message (a sub-message field). If not → no-op (schema mismatch).
/// 3. Retrieve the current `Value::Message(sub_msg)` via `msg.get_field(&field_a_desc)` → clone.
/// 4. Recurse with the sub-message and path `"b.c"`.
/// 5. After recursion, write the mutated sub-message back via `msg.set_field(&field_a_desc, Value::Message(sub_msg))`.
fn set_nested_field(msg: &mut DynamicMessage, ocsf_field: &str, value: ProtoValue) {
    let dot_pos = ocsf_field.find('.');
    match dot_pos {
        None => {
            // Flat field: delegate directly to set_field_by_name (no-op on unknown field).
            msg.set_field_by_name(ocsf_field, value);
        }
        Some(dot) => {
            // Dotted path: navigate the intermediate sub-message.
            let head = &ocsf_field[..dot];
            let tail = &ocsf_field[dot + 1..];

            // Look up the intermediate field descriptor on the current message.
            let field_desc = match msg.descriptor().get_field_by_name(head) {
                Some(fd) => fd,
                None => return, // Unknown field — stub descriptor or schema gap; no-op.
            };

            // Verify this is a sub-message field (not a scalar).
            match field_desc.kind() {
                Kind::Message(_) => {}
                _ => return, // Schema mismatch — not a message field; no-op.
            }

            // Retrieve the current sub-message value (or the default empty sub-message).
            // get_field returns Cow — clone into owned DynamicMessage.
            let mut sub_msg = match msg.get_field(&field_desc).into_owned() {
                ProtoValue::Message(m) => m,
                _ => return, // Unexpected value type for a message field — no-op.
            };

            // Recursively set the tail path on the sub-message.
            set_nested_field(&mut sub_msg, tail, value);

            // Write the mutated sub-message back into the parent.
            msg.set_field(&field_desc, ProtoValue::Message(sub_msg));
        }
    }
}

/// Parse a timestamp value (RFC3339 string or Unix integer) to epoch milliseconds.
///
/// Supported formats:
/// 1. RFC3339 / ISO 8601 with timezone: `"2024-01-15T10:30:00Z"`
/// 2. ISO 8601 without timezone (assumes UTC): `"2024-01-15T10:30:00"`
/// 3. Unix timestamp seconds (integer): `1705311000`
///
/// # Errors
///
/// Returns `PrismError::OcsfTimestampParseError` if none of the formats match.
fn parse_timestamp_to_epoch_ms(
    field_name: &str,
    value: &serde_json::Value,
) -> Result<i64, PrismError> {
    // Attempt 1: RFC3339 string
    if let Some(s) = value.as_str() {
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Ok(dt.with_timezone(&Utc).timestamp_millis());
        }
        // Attempt 2: ISO 8601 without timezone (assume UTC)
        if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&naive).timestamp_millis());
        }
        return Err(PrismError::OcsfTimestampParseError {
            field: field_name.to_owned(),
            raw: s.to_owned(),
        });
    }

    // Attempt 3: Unix timestamp integer (seconds → milliseconds)
    if let Some(unix_secs) = value.as_i64() {
        return Utc
            .timestamp_opt(unix_secs, 0)
            .single()
            .map(|dt| dt.timestamp_millis())
            .ok_or_else(|| PrismError::OcsfTimestampParseError {
                field: field_name.to_owned(),
                raw: unix_secs.to_string(),
            });
    }

    Err(PrismError::OcsfTimestampParseError {
        field: field_name.to_owned(),
        raw: value.to_string(),
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// S-ADR058-OCSF-ROUTING-001 Red Gate Test — RG-013
//
// Defense-in-depth test for `set_nested_field` under EntityManagement (3004).
// This test is likely GREEN-BY-DESIGN: `set_nested_field` is already implemented,
// and EntityManagement v1.7.0 inherits `comment` from BaseEvent.
//
// SAP-3 annotation: `set_nested_field` is private; this test covers the
// private implementation path. The public reachability path is via
// `SpecDrivenMapper::map()` → exercised by RG-016/019 end-to-end tests.
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use prost_reflect::Value as ProtoValue;

    use super::set_nested_field;
    use crate::pool::OcsfDescriptors;

    /// RG-013 / AC-005 / BC-2.16.003 KF-03/04 — EntityManagement `comment` field preserved
    ///
    /// `set_nested_field(&mut entity_management_msg, "comment", ProtoValue::String("reviewed"))`
    /// MUST write the field — EntityManagement (3004) inherits `comment` from BaseEvent
    /// in OCSF v1.7.0.
    ///
    /// This guards against the historical class mismatch where Claroty `audit_log` was routed
    /// to AccountChange (3001) instead of EntityManagement (3004). AccountChange was believed
    /// to LACK `comment`; if `set_nested_field` silently no-ops for AccountChange, every
    /// `note → comment` mapping drops the value.
    ///
    /// **SAP-3 annotation (defense-in-depth):** `set_nested_field` is a private function.
    /// This test exercises it directly and is classified DEFENSE-IN-DEPTH under SAP-3.
    /// The load-bearing public-surface reachability path is via `SpecDrivenMapper::map()`
    /// → `pipeline_result_to_record_batch` exercised by RG-016..019/022.
    ///
    /// **GREEN-BY-DESIGN note:** `set_nested_field` has a real implementation; EntityManagement
    /// inherits `comment` from BaseEvent. This test documents the invariant and passes
    /// immediately on an environment with compiled-in real OCSF descriptors. When the pool
    /// is a stub (ocsf-proto-gen not run), the test gracefully skips via the Err-arm.
    #[test]
    fn test_claroty_note_comment_not_silently_dropped_under_entity_management() {
        let pool = OcsfDescriptors::get();

        // Get the EntityManagement (3004) descriptor.
        let em_desc = match pool.get_message_by_name("ocsf.v1_7_0.events.iam.EntityManagement") {
            Some(d) => d,
            None => {
                // Stub pool (ocsf-proto-gen not yet run) — graceful skip.
                // The load-bearing assertion runs on the CI build which has real descriptors.
                return;
            }
        };

        let mut msg = prost_reflect::DynamicMessage::new(em_desc.clone());

        // Call the private `set_nested_field` with the `note → comment` mapping.
        // "comment" is a single-segment path (flat, not dotted) — uses set_field_by_name.
        set_nested_field(
            &mut msg,
            "comment",
            ProtoValue::String("reviewed".to_owned()),
        );

        // Assert "comment" field was written.
        let comment_desc = em_desc.get_field_by_name("comment").expect(
            "AC-005 (RG-013): EntityManagement (3004) descriptor MUST have a 'comment' field \
             (inherited from BaseEvent in OCSF v1.7.0); missing field means wrong descriptor \
             or wrong class_uid (AccountChange 3001 does not carry 'comment' in OCSF v1.7.0 schema)"
        );
        let written_val = msg.get_field(&comment_desc);
        assert_eq!(
            written_val.as_ref(),
            &ProtoValue::String("reviewed".to_owned()),
            "AC-005 (RG-013): set_nested_field('comment', 'reviewed') under EntityManagement \
             MUST write the value. If it returns the default empty string, 'comment' is \
             absent from the EntityManagement descriptor (wrong class_uid or schema mismatch). \
             Every Claroty audit_log 'note' value would be silently dropped."
        );
    }
}
