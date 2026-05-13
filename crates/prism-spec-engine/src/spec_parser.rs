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
///
/// `#[non_exhaustive]`: forward-compat for plugin TOML schema evolution — new auth
/// variants will be added (ADR-023 §C2 WASM auth). Fields may expand without a semver bump.
/// External crates matching on this enum MUST include a wildcard `_ => {}` arm.
#[non_exhaustive]
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
///
/// `#[non_exhaustive]`: forward-compat for plugin TOML schema evolution — AC-1 adds
/// `page_size` field to `CursorToken`; future variants possible (e.g., keyset pagination).
/// Fields may expand without a semver bump; use the `Default` impl or builder pattern.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PaginationConfig {
    /// No pagination; single request returns all records.
    None,
    /// Cursor-token pagination; `cursor_response_path` must be a valid JSONPath.
    ///
    /// `page_size` — when `Some(n)`, the `page_size` query parameter is appended to
    /// BOTH the first-call URL (no cursor yet) and all cursor-continuation URLs.
    /// When `None`, no `page_size` parameter is appended.
    CursorToken {
        cursor_response_path: String,
        /// Page size to append to every cursor-pagination request (first-call and continuations).
        ///
        /// `None` = omit the parameter entirely (default; backward-compatible with older TOML
        /// specs that do not declare a `page_size` field).
        ///
        /// `Some(0)` is accepted and forwarded verbatim to the sensor API. The pipeline does
        /// NOT validate whether the API accepts zero as a page size — callers MUST avoid
        /// `Some(0)` if their sensor API rejects `page_size=0` requests.
        #[serde(default)]
        page_size: Option<u32>,
    },
    /// Offset/limit pagination; `page_size` must be > 0.
    OffsetLimit { page_size: u32 },
}

/// Rate limit hints from the sensor spec (BC-2.16.002 postcondition).
///
/// `#[non_exhaustive]`: forward-compat for plugin TOML schema evolution — request
/// bucket policy, jitter, and retry configuration are planned additions.
/// Fields may expand without a semver bump.
///
/// # Forward-compatible construction
/// External callers should use `..Default::default()` to avoid breakage when new fields are added:
/// ```ignore
/// let hints = RateLimitHints { requests_per_second: Some(10.0), ..Default::default() };
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RateLimitHints {
    /// Maximum requests per second. inter-request delay = 1 / requests_per_second.
    pub requests_per_second: Option<f64>,
    /// Burst allowance in requests.
    pub burst_size: Option<u32>,
}

impl RateLimitHints {
    /// Construct a `RateLimitHints` with the specified values.
    ///
    /// Internal construction shortcut for forward-compatible external construction.
    pub fn new(requests_per_second: Option<f64>, burst_size: Option<u32>) -> Self {
        Self {
            requests_per_second,
            burst_size,
        }
    }
}

/// A single step in a multi-step fetch pipeline (BC-2.16.002).
///
/// `#[non_exhaustive]`: forward-compat for plugin TOML schema evolution — `retry`,
/// `batch`, `cache_ttl` are planned additions. Fields may expand without a semver bump;
/// use the `Default` impl or builder pattern for external construction.
#[non_exhaustive]
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

impl Default for FetchStep {
    /// Default `FetchStep` — all optional fields are `None`/empty; required fields use empty strings.
    ///
    /// External callers should use struct-literal + `..Default::default()` for forward-compatible
    /// construction — adding a field to `FetchStep` will not break callers that use this pattern:
    /// ```ignore
    /// let step = FetchStep { name: "fetch".to_string(), method: "GET".to_string(), ..Default::default() };
    /// ```
    fn default() -> Self {
        Self {
            name: String::new(),
            method: "GET".to_string(),
            path_template: String::new(),
            body_template: None,
            response_path: "$.items".to_string(),
            pagination_cursor_path: None,
            variables_produced: vec![],
            fan_out_batch_size: None,
            pagination: None,
        }
    }
}

impl FetchStep {
    /// Construct a `FetchStep` with all fields.
    ///
    /// Internal construction shortcut. External callers should use struct-literal +
    /// `..Default::default()` for forward compatibility when new fields are added.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        method: impl Into<String>,
        path_template: impl Into<String>,
        body_template: Option<String>,
        response_path: impl Into<String>,
        pagination_cursor_path: Option<String>,
        variables_produced: Vec<String>,
        fan_out_batch_size: Option<u32>,
        pagination: Option<PaginationConfig>,
    ) -> Self {
        Self {
            name: name.into(),
            method: method.into(),
            path_template: path_template.into(),
            body_template,
            response_path: response_path.into(),
            pagination_cursor_path,
            variables_produced,
            fan_out_batch_size,
            pagination,
        }
    }
}

/// A single column definition in a sensor table (BC-2.16.001 postconditions).
///
/// `#[non_exhaustive]`: forward-compat for plugin TOML schema evolution — `ocsf_field`
/// grammar expansions expected. Fields may expand without a semver bump.
#[non_exhaustive]
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

impl Default for ColumnSpec {
    /// Default `ColumnSpec` — empty name, `ColumnType::String`, no OCSF field, no options.
    ///
    /// External callers should use struct-literal + `..Default::default()` for forward-compatible
    /// construction:
    /// ```ignore
    /// let col = ColumnSpec { name: "host".to_string(), column_type: ColumnType::String, ..Default::default() };
    /// ```
    fn default() -> Self {
        Self {
            name: String::new(),
            column_type: ColumnType::String,
            ocsf_field: None,
            options: vec![],
        }
    }
}

impl ColumnSpec {
    /// Construct a `ColumnSpec`.
    ///
    /// Internal construction shortcut. External callers should use struct-literal +
    /// `..Default::default()` for forward compatibility when new fields are added.
    pub fn new(
        name: impl Into<String>,
        column_type: ColumnType,
        ocsf_field: Option<String>,
        options: Vec<ColumnOptions>,
    ) -> Self {
        Self {
            name: name.into(),
            column_type,
            ocsf_field,
            options,
        }
    }
}

/// A table within a sensor spec (BC-2.16.001).
///
/// S-2.08 adds `table_type`, `poll_interval_secs`, and `retention_secs` fields.
/// Both `poll_interval_secs` and `retention_secs` are only valid when
/// `table_type == TableType::EventStream`; `SpecParser::validate_table_spec`
/// enforces this constraint (AC-7, EC-002).
///
/// `#[non_exhaustive]`: forward-compat for plugin TOML schema evolution — new declarative
/// features planned. Fields may expand without a semver bump; use `TableSpec::new_point_in_time`
/// or `TableSpec::new` constructors for forward-compatible construction.
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

// `CredentialRef` canonical definition lives in `crate::types` — re-export here
// so `spec_parser::CredentialRef` import paths remain stable for callers.
// Consolidation closes TD-S-PLUGIN-PREREQ-C-001-A: the two byte-identical
// declarations were a Rule 3 violation (Canonical Principle, CLAUDE.md).
pub use crate::types::CredentialRef;

/// The top-level sensor spec parsed from a `*.sensor.toml` file (BC-2.16.001).
///
/// `#[non_exhaustive]`: forward-compat for plugin TOML schema evolution — root spec
/// type; fields will expand with ADR-023 grammar. Fields may expand without a semver bump;
/// use the `Default` impl or builder pattern for external construction.
#[non_exhaustive]
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
    /// Credential references declared by this sensor (BC-2.03.013).
    ///
    /// Each ref names a credential in the sensor's keyring namespace that must be
    /// resolvable at boot time (step 5). Empty = no credentials declared.
    /// `#[serde(default)]` ensures backward-compatible parsing of TOML files
    /// that predate this field.
    #[serde(default)]
    pub credential_refs: Vec<CredentialRef>,
}

impl Default for SensorSpec {
    /// Default `SensorSpec` — empty strings, `AuthType::ApiKey`, no tables.
    ///
    /// External callers should use struct-literal + `..Default::default()` for forward-compatible
    /// construction:
    /// ```ignore
    /// let spec = SensorSpec {
    ///     sensor_id: "my-sensor".to_string(),
    ///     name: "My Sensor".to_string(),
    ///     auth_type: AuthType::ApiKey,
    ///     base_url: "https://api.example.com".to_string(),
    ///     ..Default::default()
    /// };
    /// ```
    fn default() -> Self {
        Self {
            sensor_id: String::new(),
            name: String::new(),
            auth_type: AuthType::ApiKey,
            base_url: String::new(),
            tables: vec![],
            rate_limit_hints: None,
            version: "1.0.0".to_string(),
            credential_refs: vec![],
        }
    }
}

impl SensorSpec {
    /// Construct a `SensorSpec` with all fields.
    ///
    /// Internal construction shortcut. External callers should use struct-literal +
    /// `..Default::default()` for forward compatibility when new fields are added.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sensor_id: impl Into<String>,
        name: impl Into<String>,
        auth_type: AuthType,
        base_url: impl Into<String>,
        tables: Vec<TableSpec>,
        rate_limit_hints: Option<RateLimitHints>,
        version: impl Into<String>,
        credential_refs: Vec<CredentialRef>,
    ) -> Self {
        Self {
            sensor_id: sensor_id.into(),
            name: name.into(),
            auth_type,
            base_url: base_url.into(),
            tables,
            rate_limit_hints,
            version: version.into(),
            credential_refs,
        }
    }
}

/// Descriptor exported from a loaded spec for downstream consumption.
///
/// prism-query (S-3.02) uses these descriptors to register DataFusion TableProviders.
/// prism-spec-engine MUST NOT import DataFusion — it exports descriptors only (AD-015).
///
/// `#[non_exhaustive]`: forward-compat for plugin TOML schema evolution — table metadata
/// fields (columns, steps) may gain new config fields. Fields may expand without a semver bump.
#[non_exhaustive]
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

impl Default for SensorTableDescriptor {
    /// Default `SensorTableDescriptor` — empty table name, no columns, empty sensor_id, no credentials.
    ///
    /// External callers should use struct-literal + `..Default::default()` for forward-compatible
    /// construction:
    /// ```ignore
    /// let desc = SensorTableDescriptor {
    ///     table_name: "crowdstrike.devices".to_string(),
    ///     sensor_id: "crowdstrike".to_string(),
    ///     ..Default::default()
    /// };
    /// ```
    fn default() -> Self {
        Self {
            table_name: String::new(),
            columns: vec![],
            sensor_id: String::new(),
            has_credentials: false,
        }
    }
}

impl SensorTableDescriptor {
    /// Construct a `SensorTableDescriptor`.
    ///
    /// Internal construction shortcut. External callers should use struct-literal +
    /// `..Default::default()` for forward compatibility when new fields are added.
    pub fn new(
        table_name: impl Into<String>,
        columns: Vec<ColumnSpec>,
        sensor_id: impl Into<String>,
        has_credentials: bool,
    ) -> Self {
        Self {
            table_name: table_name.into(),
            columns,
            sensor_id: sensor_id.into(),
            has_credentials,
        }
    }
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
        if let Some(poll_interval) = table.poll_interval_secs
            && poll_interval < MIN_POLL_INTERVAL_SECS
        {
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

        // EventStream: validate retention_secs maximum
        if let Some(retention) = table.retention_secs
            && retention > MAX_RETENTION_SECS
        {
            return Err(PrismError::Spec(SpecError {
                code: SpecErrorCode::ESpec001,
                message: format!(
                    "sensor '{}' table '{}': retention_secs={} exceeds the maximum of \
                     {}s (7 days) (AC-7). Reduce retention to at most {} seconds.",
                    sensor_id, table.table_name, retention, MAX_RETENTION_SECS, MAX_RETENTION_SECS
                ),
                toml_path: Some(format!(
                    "sensor.tables[{}].retention_secs",
                    table.table_name
                )),
                file_path: None,
                line_number: None,
            }));
        }

        Ok(())
    }

    /// Parse a single TOML string into a `SensorSpec`.
    ///
    /// Returns `Ok(SensorSpec)` or `Err(PrismError)` — never panics (VP-023).
    pub fn parse(toml_input: &str) -> Result<SensorSpec, PrismError> {
        toml::from_str::<SensorSpec>(toml_input).map_err(|e| {
            let line_number = e.span().map(|span| {
                // Count newlines before the error span start.
                // F-LP10-MED-001 (defensive): `span.start` is a byte offset from the toml crate.
                // TOML structural tokens are always ASCII so span.start is always a char boundary
                // in practice; however, we use char_indices to count safely regardless.
                let safe_start = span.start.min(toml_input.len());
                let newline_count = toml_input
                    .char_indices()
                    .take_while(|(byte_idx, _)| *byte_idx < safe_start)
                    .filter(|(_, c)| *c == '\n')
                    .count();
                (newline_count + 1) as u32
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
