//! TOML spec file parser and descriptor types.
//!
//! Parses `*.sensor.toml` files into `SensorSpec` structs and produces
//! `SensorTableDescriptor` values for downstream consumption by prism-query.
//!
//! # Architecture Compliance
//! - Does NOT import DataFusion or Arrow.
//! - `SensorTableDescriptor` uses `prism_core::ColumnType` only.
//! - Table name conflicts are detected at load time (BC-2.16.001 postcondition).

use prism_core::{ColumnOptions, ColumnType, PrismError, SpecError, SpecErrorCode, TableType};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Spec data model
// ---------------------------------------------------------------------------

/// Authentication type declared in a sensor spec.
///
/// Determines how prism-spec-engine resolves credentials from the credential
/// store at query time (BC-2.16.001 Auth Type Resolution).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    /// OAuth2 client-credentials flow; token fetched at query time.
    Oauth2ClientCredentials,
    /// Static bearer token resolved from credential store.
    BearerStatic,
    /// Cookie-based auth requiring a round-trip login step.
    CookieRoundtrip,
    /// API key injected as header or query parameter.
    ApiKey,
}

/// Pagination configuration for a fetch step (BC-2.16.002).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PaginationConfig {
    /// No pagination; single request returns all records.
    None,
    /// Cursor-token pagination; `cursor_response_path` must be a valid JSONPath.
    CursorToken { cursor_response_path: String },
    /// Offset/limit pagination; `page_size` must be > 0.
    OffsetLimit { page_size: u32 },
}

/// Rate limit hints from the sensor spec (BC-2.16.002 postcondition).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimitHints {
    /// Maximum requests per second. inter-request delay = 1 / requests_per_second.
    pub requests_per_second: Option<f64>,
    /// Burst allowance in requests.
    pub burst_size: Option<u32>,
}

/// A single step in a multi-step fetch pipeline (BC-2.16.002).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchStep {
    /// Step name, used as variable scope prefix (e.g., `${step_name.field}`).
    pub name: String,
    /// HTTP method: "GET" or "POST".
    pub method: String,
    /// Path template with `${step_name.field}` variable interpolation.
    pub path_template: String,
    /// Optional body template for POST requests.
    pub body_template: Option<String>,
    /// JSONPath expression into the JSON response pointing to the results array.
    pub response_path: String,
    /// Optional JSONPath expression for cursor-based pagination.
    pub pagination_cursor_path: Option<String>,
    /// Variable names produced by this step for downstream interpolation.
    pub variables_produced: Vec<String>,
    /// Batch size for fan-out when a variable resolves to an array. Default 100.
    pub fan_out_batch_size: Option<u32>,
    /// Pagination configuration for this step.
    pub pagination: Option<PaginationConfig>,
}

/// A single column definition in a sensor table (BC-2.16.001 postconditions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnSpec {
    /// Column name. Must be unique within the table.
    pub name: String,
    /// Data type of this column.
    pub column_type: ColumnType,
    /// OCSF field path this column maps to (e.g., `"device.ip"`). None = raw_extensions.
    pub ocsf_field: Option<String>,
    /// Column options controlling query engine behavior.
    #[serde(default)]
    pub options: Vec<ColumnOptions>,
}

/// A table within a sensor spec (BC-2.16.001).
///
/// S-2.08 adds `table_type`, `poll_interval_secs`, and `retention_secs` fields.
/// Both `poll_interval_secs` and `retention_secs` are only valid when
/// `table_type == TableType::EventStream`; `SpecParser::validate_table_spec`
/// enforces this constraint (AC-7, EC-002).
///
/// `#[non_exhaustive]` prevents external crates from constructing `TableSpec`
/// via struct literal, allowing future fields to be added without a semver
/// major bump (cargo-semver-checks `constructible_struct_adds_field` lint).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableSpec {
    /// Table name. Combined with sensor_id as `{sensor_id}.{table_name}` in DataFusion.
    pub table_name: String,
    /// OCSF event class for records in this table (e.g., `"security_finding"`).
    pub ocsf_class: String,
    /// Column definitions.
    pub columns: Vec<ColumnSpec>,
    /// Fetch pipeline steps, executed sequentially.
    pub steps: Vec<FetchStep>,
    /// Data-delivery model for this table (default: `PointInTime`).
    ///
    /// S-2.08: added to support event-stream local buffering.
    #[serde(default)]
    pub table_type: TableType,
    /// How often (in seconds) the background `EventPoller` calls the sensor API
    /// to ingest new events. Only valid when `table_type == EventStream`.
    ///
    /// Minimum: 10 seconds (AC-7, EC-002). Default: `None` (PointInTime tables).
    /// Stored as raw seconds to avoid pulling a `Duration`-aware serde dep here;
    /// callers convert to `std::time::Duration` as needed.
    #[serde(default)]
    pub poll_interval_secs: Option<u64>,
    /// Retention period in seconds for buffered events. Only valid when
    /// `table_type == EventStream`.
    ///
    /// Maximum: 604800 seconds (7 days). Default: 86400 seconds (24 hours).
    /// `None` means use the default retention (86400s).
    #[serde(default)]
    pub retention_secs: Option<u64>,
}

impl TableSpec {
    /// Constructs a `TableSpec` for a `PointInTime` table (the common case).
    ///
    /// Sets `table_type = TableType::PointInTime`, `poll_interval_secs = None`,
    /// and `retention_secs = None`. Use this constructor when the S-2.08
    /// event-stream fields are not needed — it remains forward-compatible with
    /// any future `#[non_exhaustive]` fields.
    ///
    /// # Usage in tests
    /// Prefer this over struct literal construction so test code remains
    /// forward-compatible with future field additions.
    pub fn new_point_in_time(
        table_name: impl Into<String>,
        ocsf_class: impl Into<String>,
        columns: Vec<ColumnSpec>,
        steps: Vec<FetchStep>,
    ) -> Self {
        Self {
            table_name: table_name.into(),
            ocsf_class: ocsf_class.into(),
            columns,
            steps,
            table_type: TableType::PointInTime,
            poll_interval_secs: None,
            retention_secs: None,
        }
    }

    /// Constructs a `TableSpec` with all S-2.08 fields explicitly provided.
    ///
    /// Use this constructor when `table_type`, `poll_interval_secs`, or
    /// `retention_secs` need to be set explicitly (e.g., in event-stream
    /// validation tests). This constructor is forward-compatible with any
    /// future `#[non_exhaustive]` additions.
    pub fn new(
        table_name: impl Into<String>,
        ocsf_class: impl Into<String>,
        columns: Vec<ColumnSpec>,
        steps: Vec<FetchStep>,
        table_type: TableType,
        poll_interval_secs: Option<u64>,
        retention_secs: Option<u64>,
    ) -> Self {
        Self {
            table_name: table_name.into(),
            ocsf_class: ocsf_class.into(),
            columns,
            steps,
            table_type,
            poll_interval_secs,
            retention_secs,
        }
    }
}

/// The top-level sensor spec parsed from a `*.sensor.toml` file (BC-2.16.001).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorSpec {
    /// Unique sensor identifier. Must match `^[a-z][a-z0-9_-]*$`.
    pub sensor_id: String,
    /// Human-readable sensor name.
    pub name: String,
    /// Authentication type required by this sensor's API.
    pub auth_type: AuthType,
    /// Base URL for the sensor's API.
    pub base_url: String,
    /// Tables exposed by this sensor.
    pub tables: Vec<TableSpec>,
    /// Rate limit hints for requests to this sensor's API.
    pub rate_limit_hints: Option<RateLimitHints>,
    /// Spec version string (semver).
    pub version: String,
}

/// Descriptor exported from a loaded spec for downstream consumption.
///
/// prism-query (S-3.02) uses these descriptors to register DataFusion TableProviders.
/// prism-spec-engine MUST NOT import DataFusion — it exports descriptors only (AD-015).
#[derive(Debug, Clone, PartialEq)]
pub struct SensorTableDescriptor {
    /// Fully-qualified DataFusion table name: `{sensor_id}.{table_name}`.
    pub table_name: String,
    /// Column schemas derived from the spec's ColumnSpec entries.
    pub columns: Vec<ColumnSpec>,
    /// The sensor_id that owns this table.
    pub sensor_id: String,
    /// Whether the sensor has credentials registered for any client.
    /// False = tables queryable but return `status: no_credentials` (DEC-036).
    pub has_credentials: bool,
}

// ---------------------------------------------------------------------------
// SpecLoader — implementation (BC-2.16.001)
// ---------------------------------------------------------------------------

/// Loads sensor specs from a directory of `*.sensor.toml` files (BC-2.16.001).
///
/// Scans `sensor_specs_dir` (flat, non-recursive), parses each file, validates it,
/// and returns the set of `SensorTableDescriptor`s for DataFusion registration.
/// Invalid specs are skipped with errors; valid specs load independently (DI-030).
pub struct SpecLoader {
    sensor_specs_dir: String,
}

impl SpecLoader {
    /// Create a new SpecLoader for the given directory.
    pub fn new(sensor_specs_dir: impl Into<String>) -> Self {
        SpecLoader {
            sensor_specs_dir: sensor_specs_dir.into(),
        }
    }

    /// Validates `table_type`-specific constraints for a `TableSpec` (AC-7, EC-002).
    ///
    /// Rules:
    /// - `poll_interval_secs` and `retention_secs` are only valid for `EventStream`.
    /// - `poll_interval_secs` minimum: 10 seconds.
    /// - `retention_secs` maximum: 604800 seconds (7 days).
    ///
    /// Returns `Ok(())` on valid input; `Err(PrismError::Spec)` with a descriptive
    /// message on invalid input.
    ///
    /// # AC-7, EC-002
    /// Called by `parse()` for each table in the spec; validation failures prevent
    /// the spec from loading.
    pub fn validate_table_spec(sensor_id: &str, table: &TableSpec) -> Result<(), PrismError> {
        const MIN_POLL_INTERVAL_SECS: u64 = 10;
        const MAX_RETENTION_SECS: u64 = 604_800; // 7 days

        // PointInTime tables must NOT have poll_interval_secs or retention_secs
        if table.table_type == TableType::PointInTime {
            if let Some(poll_interval) = table.poll_interval_secs {
                return Err(PrismError::Spec(SpecError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "sensor '{}' table '{}': poll_interval_secs={} is only valid for \
                         EventStream tables, not PointInTime (AC-7)",
                        sensor_id, table.table_name, poll_interval
                    ),
                    toml_path: Some(format!(
                        "sensor.tables[{}].poll_interval_secs",
                        table.table_name
                    )),
                    file_path: None,
                    line_number: None,
                }));
            }
            if let Some(retention) = table.retention_secs {
                return Err(PrismError::Spec(SpecError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "sensor '{}' table '{}': retention_secs={} is only valid for \
                         EventStream tables, not PointInTime (AC-7)",
                        sensor_id, table.table_name, retention
                    ),
                    toml_path: Some(format!(
                        "sensor.tables[{}].retention_secs",
                        table.table_name
                    )),
                    file_path: None,
                    line_number: None,
                }));
            }
            return Ok(());
        }

        // EventStream: validate poll_interval_secs minimum
        if let Some(poll_interval) = table.poll_interval_secs {
            if poll_interval < MIN_POLL_INTERVAL_SECS {
                return Err(PrismError::Spec(SpecError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "sensor '{}' table '{}': poll_interval_secs={} is below the minimum \
                         of {}s (AC-7, EC-002). Increase poll_interval to at least {}s.",
                        sensor_id,
                        table.table_name,
                        poll_interval,
                        MIN_POLL_INTERVAL_SECS,
                        MIN_POLL_INTERVAL_SECS
                    ),
                    toml_path: Some(format!(
                        "sensor.tables[{}].poll_interval_secs",
                        table.table_name
                    )),
                    file_path: None,
                    line_number: None,
                }));
            }
        }

        // EventStream: validate retention_secs maximum
        if let Some(retention) = table.retention_secs {
            if retention > MAX_RETENTION_SECS {
                return Err(PrismError::Spec(SpecError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "sensor '{}' table '{}': retention_secs={} exceeds the maximum of \
                         {}s (7 days) (AC-7). Reduce retention to at most {} seconds.",
                        sensor_id,
                        table.table_name,
                        retention,
                        MAX_RETENTION_SECS,
                        MAX_RETENTION_SECS
                    ),
                    toml_path: Some(format!(
                        "sensor.tables[{}].retention_secs",
                        table.table_name
                    )),
                    file_path: None,
                    line_number: None,
                }));
            }
        }

        Ok(())
    }

    /// Parse a single TOML string into a `SensorSpec`.
    ///
    /// Returns `Ok(SensorSpec)` or `Err(PrismError)` — never panics (VP-023).
    pub fn parse(toml_input: &str) -> Result<SensorSpec, PrismError> {
        toml::from_str::<SensorSpec>(toml_input).map_err(|e| {
            let line_number = e.span().map(|span| {
                // Count newlines before the error span start
                let before = &toml_input[..span.start.min(toml_input.len())];
                (before.chars().filter(|&c| c == '\n').count() + 1) as u32
            });
            PrismError::Spec(SpecError {
                code: SpecErrorCode::ESpec001,
                message: format!("TOML parse error: {e}"),
                toml_path: None,
                file_path: None,
                line_number,
            })
        })
    }

    /// Load all `*.sensor.toml` files from `sensor_specs_dir`.
    ///
    /// Returns (descriptors, errors): valid specs produce descriptors; invalid files
    /// produce errors but do not block valid specs from loading (DI-030).
    pub fn load_all(&self) -> (Vec<SensorTableDescriptor>, Vec<PrismError>) {
        let mut descriptors = Vec::new();
        let mut errors = Vec::new();

        // Read the directory; if it doesn't exist or is empty, return empty results.
        let read_dir = match std::fs::read_dir(&self.sensor_specs_dir) {
            Ok(rd) => rd,
            Err(e) => {
                // Non-existent directory = no specs, no errors (DI-030).
                if e.kind() == std::io::ErrorKind::NotFound {
                    return (descriptors, errors);
                }
                errors.push(PrismError::Spec(SpecError {
                    code: SpecErrorCode::ESpec001,
                    message: format!("cannot read sensor specs directory: {e}"),
                    toml_path: None,
                    file_path: Some(self.sensor_specs_dir.clone()),
                    line_number: None,
                }));
                return (descriptors, errors);
            }
        };

        let mut named_specs: Vec<(String, SensorSpec)> = Vec::new();

        for entry in read_dir.flatten() {
            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Only process `*.sensor.toml` files (flat, non-recursive).
            if !file_name.ends_with(".sensor.toml") {
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(PrismError::Spec(SpecError {
                        code: SpecErrorCode::ESpec001,
                        message: format!("cannot read spec file: {e}"),
                        toml_path: None,
                        file_path: Some(file_name.clone()),
                        line_number: None,
                    }));
                    continue;
                }
            };

            match Self::parse(&content) {
                Ok(spec) => {
                    named_specs.push((file_name, spec));
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        // Detect sensor_id conflicts — second occurrence is rejected (BC-2.16.001).
        let id_conflicts = Self::detect_sensor_id_conflicts(&named_specs);
        let rejected_ids: std::collections::HashSet<String> = id_conflicts
            .iter()
            .filter_map(|e| {
                if let PrismError::Spec(se) = e {
                    se.message
                        .split("sensor_id '")
                        .nth(1)
                        .and_then(|s| s.split('\'').next())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();
        errors.extend(id_conflicts);

        // For each valid spec (not rejected), detect intra-spec table name conflicts
        // and produce descriptors.
        let mut seen_sensor_ids: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for (_file_name, spec) in named_specs {
            if rejected_ids.contains(&spec.sensor_id) {
                // Already counted as error; skip
                if seen_sensor_ids.contains(&spec.sensor_id) {
                    continue;
                }
            }
            seen_sensor_ids.insert(spec.sensor_id.clone());

            // Detect intra-spec table name conflicts.
            let table_conflicts = Self::detect_table_name_conflicts(std::slice::from_ref(&spec));
            if !table_conflicts.is_empty() {
                errors.extend(table_conflicts);
                continue;
            }

            // Produce descriptors for each table.
            for table in &spec.tables {
                descriptors.push(SensorTableDescriptor {
                    table_name: format!("{}.{}", spec.sensor_id, table.table_name),
                    columns: table.columns.clone(),
                    sensor_id: spec.sensor_id.clone(),
                    has_credentials: false, // credentials unknown at load time
                });
            }
        }

        (descriptors, errors)
    }

    /// Detect duplicate table names across multiple specs.
    ///
    /// Returns error codes for any second-occurrence table names (BC-2.16.001).
    pub fn detect_table_name_conflicts(specs: &[SensorSpec]) -> Vec<PrismError> {
        let mut errors = Vec::new();
        let mut seen: std::collections::HashMap<String, &str> = std::collections::HashMap::new();

        for spec in specs {
            let mut intra_seen: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for table in &spec.tables {
                let qualified = format!("{}.{}", spec.sensor_id, table.table_name);
                if intra_seen.contains(&table.table_name) {
                    errors.push(PrismError::Spec(SpecError {
                        code: SpecErrorCode::ESpec004,
                        message: format!(
                            "duplicate table_name '{}' within sensor '{}' (BC-2.16.001)",
                            table.table_name, spec.sensor_id
                        ),
                        toml_path: Some(format!("sensor.tables[{}]", table.table_name)),
                        file_path: None,
                        line_number: None,
                    }));
                } else {
                    intra_seen.insert(table.table_name.clone());
                }

                // Also check cross-spec conflicts
                if let Some(prev_sensor) = seen.get(&qualified) {
                    errors.push(PrismError::Spec(SpecError {
                        code: SpecErrorCode::ESpec004,
                        message: format!(
                            "duplicate table_name '{}' (also in sensor '{}')",
                            qualified, prev_sensor
                        ),
                        toml_path: None,
                        file_path: None,
                        line_number: None,
                    }));
                } else {
                    seen.insert(qualified, &spec.sensor_id);
                }
            }
        }

        errors
    }

    /// Detect duplicate sensor_ids across spec files.
    ///
    /// Returns E-SPEC-009 for each second-occurrence sensor_id (BC-2.16.001).
    pub fn detect_sensor_id_conflicts(specs: &[(String, SensorSpec)]) -> Vec<PrismError> {
        let mut errors = Vec::new();
        let mut seen: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();

        for (file_name, spec) in specs {
            if let Some(prev_file) = seen.get(spec.sensor_id.as_str()) {
                errors.push(PrismError::Spec(SpecError {
                    code: SpecErrorCode::ESpec009,
                    message: format!(
                        "duplicate sensor_id '{}' in '{}' (first seen in '{}')",
                        spec.sensor_id, file_name, prev_file
                    ),
                    toml_path: Some("sensor.sensor_id".to_string()),
                    file_path: Some(file_name.clone()),
                    line_number: None,
                }));
            } else {
                seen.insert(&spec.sensor_id, file_name);
            }
        }

        errors
    }
}
