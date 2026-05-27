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
use prost_reflect::{DynamicMessage, Value as ProtoValue};

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
            .unwrap_or("");

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
        // The table_name is "{sensor_id}.{table_suffix}" format.
        // We match against record_type using the following priority order:
        //   1. Exact match: "{sensor_id}.{record_type}"
        //   2. Plural match: "{sensor_id}.{record_type}s"
        //   3. Prefix match: table suffix starts with record_type
        // This handles the common case where TOML specs use plural table names
        // (e.g., "crowdstrike.detections") but the normalizer uses singular record_type
        // ("detection") derived from EventClassSelector.
        let exact_name = format!("{}.{}", self.sensor_id, record_type);
        let plural_name = format!("{}.{}s", self.sensor_id, record_type);
        let table_descriptor = sensor_spec
            .tables
            .iter()
            .find(|t| {
                t.table_name == exact_name
                    || t.table_name == plural_name
                    || t.table_name
                        .strip_prefix(&format!("{}.", self.sensor_id))
                        .map(|suffix| suffix.starts_with(record_type))
                        .unwrap_or(false)
            })
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

                // HIGH-005: if the column is declared in spec but absent in raw, place
                // Null in extensions and emit a debug trace so operators can detect
                // schema drift between spec and actual data source.
                if raw_value.is_none() {
                    extensions.insert(col.name.clone(), serde_json::Value::Null);
                    tracing::debug!(
                        sensor_id = %self.sensor_id,
                        column = %col.name,
                        ocsf_field = %ocsf_target,
                        event_type = "ocsf.spec_column_absent_in_raw",
                        "spec-declared column absent in raw record; inserted Null into extensions"
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
                                    // `set_field_by_name` is a no-op when the field does not
                                    // exist in the descriptor (stub or partial schema) — safe.
                                    msg.set_field_by_name(ocsf_target, ProtoValue::I64(epoch_ms));
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
                        if let Some(v) = raw_value {
                            let (mapped_str, proto_val) = if let Some(n) = v.as_i64() {
                                (n.to_string(), ProtoValue::I64(n))
                            } else if let Some(n) = v.as_u64() {
                                (n.to_string(), ProtoValue::U64(n))
                            } else {
                                (v.to_string(), ProtoValue::String(v.to_string()))
                            };
                            // Write integer value into OCSF DynamicMessage (CRIT-001).
                            msg.set_field_by_name(ocsf_target, proto_val);
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
                            // `set_field_by_name` is a no-op when the field doesn't exist in
                            // the descriptor (stub descriptor or partial OCSF schema) — safe.
                            msg.set_field_by_name(ocsf_target, proto_val);
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
