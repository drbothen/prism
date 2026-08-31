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
    /// Authentication delegated to a WASM plugin (PLUGIN-MIGRATION-001-E).
    ///
    /// The plugin implementing auth is named via `auth_plugin` field in the sensor spec.
    /// Plugin-based auth participates in Rule C enforcement: the probe shape must match
    /// `"custom_via_plugin"` if a ShapedProbe is used (BC-2.01.016 Rule C).
    CustomViaPlugin,
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
    /// Ordered list of timestamp format names to try when parsing this column.
    ///
    /// Only meaningful when `column_type == ColumnType::Datetime`. Empty vec (default)
    /// means the column is treated as a single well-known ISO 8601 string.
    ///
    /// Recognized format names (closed set per ADR-028 v1.10 §D8-C):
    /// - `"iso8601"` — ISO 8601 / RFC 3339 string (e.g., `"2024-01-15T10:30:00Z"`)
    /// - `"unix_epoch_seconds"` — integer Unix timestamp in seconds (e.g., `1705311000`)
    /// - `"unix_epoch_millis"` — integer Unix timestamp in milliseconds
    ///
    /// Unrecognized names → `E-SPEC-001` validation error at load time (BC-2.16.009).
    /// On all-formats parse failure → `E-SPEC-018` (`TimestampParseFailure`).
    #[serde(default)]
    pub timestamp_formats: Vec<String>,
    /// Ordered list of source field names to try when the primary column field is null/absent.
    ///
    /// Only meaningful when `column_type == ColumnType::Datetime`. Empty vec (default) means
    /// no fallback chain — a null primary field produces a null output column.
    ///
    /// The pipeline executor tries each field in order. If all chain fields are also null/absent,
    /// falls back to `DateTime::now()` (UTC) and emits
    /// `tracing::warn!(event_type = "timestamp.fallback_to_now", column = %col_name)`.
    ///
    /// BC-2.16.013 §O-001 Option A LOCKED; ADR-028 v1.10 §D8-B/C.
    #[serde(default)]
    pub timestamp_fallback_chain: Vec<String>,
    /// Optional JSONPath expression for extracting this column's value from the
    /// raw JSON record returned by the pipeline executor.
    ///
    /// ## Semantics
    ///
    /// When `None` (default), the column value is extracted by looking up `col.name`
    /// as a flat top-level key on the record — identical to the pre-ENRICH-1 behavior.
    /// This default preserves full backward compatibility for all existing flat columns.
    ///
    /// When `Some(path)`, the column value is extracted using `extract_at_path(record, path)`.
    /// Paths MUST use the `$.` prefix convention of the existing `extract_at_path` function:
    ///   - `$.field`          — top-level key (redundant but valid)
    ///   - `$.a.b`            — nested object traversal
    ///   - `$.arr[*].field`   — wildcard: yields all `field` values from array `arr`
    ///
    /// The `name` field is always the SQL column identifier — a clean identifier with
    /// no `.`, `[`, or `]` characters. `source_path` is the extraction instruction only.
    ///
    /// `#[serde(default)]` ensures backward compatibility: existing TOML files without
    /// this field parse as `None`.
    ///
    /// ENRICH-1 / design document §Design Decision 1.
    #[serde(default)]
    pub source_path: Option<String>,
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
            timestamp_formats: vec![],
            timestamp_fallback_chain: vec![],
            source_path: None,
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
            timestamp_formats: vec![],
            timestamp_fallback_chain: vec![],
            source_path: None,
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
/// This is the **canonical** sensor spec type for `prism-spec-engine` (ADR-030 Approach D).
/// `ConfigSnapshot::sensor_specs` stores this type directly; `types::SensorSpec` is retired.
///
/// The three post-parse metadata fields (`file_hash`, `source_path`, `mode`) are set by the
/// file-loading caller immediately after `SpecLoader::parse` returns — they are not TOML
/// grammar fields. `#[serde(default)]` ensures existing TOML files without these fields
/// continue to parse (they are infrastructure metadata, not spec grammar).
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
    ///
    /// `#[serde(default)]` allows sensor specs with no `[[tables]]` sections to parse
    /// (credential-only specs, boot test fixtures). An empty tables vec is valid at
    /// parse time; DataFusion registration simply registers zero tables.
    #[serde(default)]
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
    /// Optional plugin ID for authentication routing (PLUGIN-MIGRATION-001-E).
    ///
    /// When `Some(plugin_id)`, the spec engine routes authentication for this sensor
    /// through the named `.prx` WASM plugin rather than through the built-in Rust
    /// adapter. The plugin must be registered in `PluginRuntime.registry` at spec-load
    /// time (AC-007); an unregistered plugin_id emits `E-SPEC-012`.
    ///
    /// Example (crowdstrike.sensor.toml `[auth]` section):
    ///   `auth_plugin = "crowdstrike-oauth2"`
    ///
    /// `#[serde(default)]` ensures backward-compatible parsing: TOML files without
    /// this field parse as `None` (no plugin auth routing). No existing sensor TOML
    /// files break when this field is absent.
    ///
    /// Traces to: BC-2.01.016 §Plugin-Implementable Auth; BC-2.17.007 manifest gate.
    #[serde(default)]
    pub auth_plugin: Option<String>,

    // -------------------------------------------------------------------------
    // ADR-030 §D2 — Post-parse hot-reload metadata fields (not TOML grammar).
    //
    // These three fields are populated by the file-loading caller immediately after
    // `SpecLoader::parse` returns. They are NOT present in `.sensor.toml` files;
    // `#[serde(default)]` ensures backward-compatible deserialization.
    // -------------------------------------------------------------------------
    /// SHA-256 hash of the source file content (for hot-reload change detection).
    ///
    /// Set by the file-loading caller (config_manager, hot_reload) immediately
    /// after parse. Empty string indicates the spec was constructed without a
    /// file source (e.g., via AddSensorSpec MCP tool from in-memory TOML).
    #[serde(default)]
    pub file_hash: String,

    /// Source file path of the `.sensor.toml` file from which this spec was parsed.
    ///
    /// **File-origin metadata** — NOT a JSONPath extraction instruction.  Distinct from
    /// `ColumnSpec::source_path` (which is a JSONPath like `$.device.hostname` used to
    /// extract a column value from the API response).  This field records where on disk
    /// the sensor spec lives; used by hot-reload change detection.
    ///
    /// Set by the file-loading caller. Empty string for in-memory-constructed specs.
    #[serde(default)]
    pub source_path: String,

    /// DTU deployment mode — set at parse time from the `[sensor]` TOML table.
    ///
    /// Defaults to `DtuMode::Shared` for backward compatibility. Governs the
    /// DTU topology used for this sensor's data flow (BC-3.2.005).
    #[serde(default)]
    pub mode: crate::types::DtuMode,

    /// Health-probe table name (BC-2.08.001 probe_table / probe-table-field-design.md §1).
    ///
    /// When `Some(name)`, `name` MUST case-sensitively match the `table_name` of a declared
    /// `[[tables]]` block in this spec. Validated at parse time as Rule 8 (E-SPEC-026).
    ///
    /// When `None`, the connectivity probe falls back to the first declared table (if any),
    /// or to the legacy `{sensor_id}_devices` no-op if no tables are declared (Section 3
    /// of probe-table-field-design.md).
    ///
    /// `#[serde(default)]` ensures existing TOML files without this field parse without error.
    ///
    #[serde(default)]
    pub probe_table: Option<String>,

    /// When `true`, `pipeline_result_to_record_batch` uses `ocsf_field_to_arrow_name(col.ocsf_field)`
    /// as the Arrow schema field name for columns with an `ocsf_field` declaration, and aggregates
    /// columns with `ocsf_field == None` into a single `raw_extensions` JSON blob.
    ///
    /// When `false` (default), the existing `col.name` path is used (backward-compatible with
    /// CrowdStrike, Armis, Cyberint sensors). Only Claroty activates this flag per AC-005.
    ///
    /// `#[serde(default)]` uses `bool::default()` = `false`; all existing sensor TOMLs without
    /// this field deserialize as `ocsf_column_naming = false`. (AC-001, ADR-058 §D2)
    #[serde(default)]
    pub ocsf_column_naming: bool,
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
            auth_plugin: None,
            file_hash: String::new(),
            source_path: String::new(),
            mode: crate::types::DtuMode::default(),
            probe_table: None,
            ocsf_column_naming: false,
        }
    }
}

impl AuthType {
    /// Return the canonical snake_case string name for this auth type.
    ///
    /// Matches the serde `rename_all = "snake_case"` serialization and the
    /// `VALID_AUTH_TYPES` list in `SpecLoader::validate_cross_composition`.
    ///
    /// Used by `step5_init_credential_store_with_probe` to pass the auth type string
    /// to `validate_cross_composition` (F-LP-IMPL-P1-003 / BC-2.01.016 Rule A).
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthType::Oauth2ClientCredentials => "oauth2_client_credentials",
            AuthType::BearerStatic => "bearer_static",
            AuthType::CookieRoundtrip => "cookie_roundtrip",
            AuthType::ApiKey => "api_key",
            AuthType::CustomViaPlugin => "custom_via_plugin",
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
            auth_plugin: None,
            file_hash: String::new(),
            source_path: String::new(),
            mode: crate::types::DtuMode::default(),
            probe_table: None,
            ocsf_column_naming: false,
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
    /// After successful TOML deserialization, validates SensorAuth × DataSource
    /// cross-composition rules for sensors that declare credential_refs (BC-2.01.016
    /// Rule 2 / ADR-026 §D3; F-LP-IMPL-P1-003):
    ///
    /// - **Rule B (E-SPEC-013):** multiple `credential_refs` declared (cardinality must be ≤ 1
    ///   OR exactly 1 when a credential is declared). Sensors with zero credential_refs
    ///   (no auth configured) are not validated here — they are allowed to parse successfully.
    ///   Sensors with ≥ 2 credential_refs are rejected immediately.
    ///
    /// Rule A (E-SPEC-012) is implicitly enforced by serde deserialization of `AuthType`.
    /// Rule C (E-SPEC-014) is deferred to `step5_init_credential_store_with_probe` where
    /// credential introspection is available (AD-017 AI-opaque credential model).
    ///
    /// Returns `Ok(SensorSpec)` or `Err(PrismError)` — never panics (VP-023).
    pub fn parse(toml_input: &str) -> Result<SensorSpec, PrismError> {
        let spec = toml::from_str::<SensorSpec>(toml_input).map_err(|e| {
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
        })?;

        // Cross-composition Rules A+B check at parse time (F-LP-IMPL-P1-003):
        //
        // Rule A (E-SPEC-012): auth_type must be a scalar from the closed enumeration.
        // Rule B (E-SPEC-013): exactly 1 credential_ref per auth method.
        //
        // Only applies when credential_refs are declared (> 0). Sensors with 0 credential_refs
        // are valid (no auth credentials declared — auth will fail at runtime if needed).
        //
        // Rule C (E-SPEC-014) is enforced at step5 credential-introspection time via
        // CredentialRefProbe::probe() returning Some(actual_shape), NOT at parse time.
        // Parse time has no access to the resolved credential type — Rule C requires
        // the credential store to report the auth_type the credential was configured for.
        if spec.credential_refs.len() > 1
            && let Err(spec_err) = Self::validate_cross_composition(
                spec.sensor_id.as_str(),
                spec.auth_type.as_str(),
                spec.credential_refs.len(),
                spec.auth_type.as_str(), // expected_shape: same as auth_type — Rule A+B only
                spec.auth_type.as_str(), // actual_shape: same as auth_type — Rule C skipped (no credential access at parse time)
            )
        {
            return Err(PrismError::Internal {
                detail: format!(
                    "cross-composition validation failed for sensor '{}': {}",
                    spec.sensor_id, spec_err
                ),
            });
        }

        // BC-2.16.009 timestamp_formats validation gate (ADR-028 v1.10 §D8-C):
        // timestamp_formats is a closed set: only these names are recognized.
        // Unrecognized format names → E-SPEC-001 at load time.
        // Stage 1 (F-LP2-HIGH-005): timestamp_formats / timestamp_fallback_chain are only
        // valid on Datetime columns — reject any non-Datetime column that declares them.
        const RECOGNIZED_TIMESTAMP_FORMATS: &[&str] =
            &["iso8601", "unix_epoch_seconds", "unix_epoch_millis"];
        for table in &spec.tables {
            for col in &table.columns {
                // Stage 1: reject timestamp fields on non-Datetime columns.
                if col.column_type != ColumnType::Datetime
                    && (!col.timestamp_formats.is_empty()
                        || !col.timestamp_fallback_chain.is_empty())
                {
                    return Err(PrismError::Spec(SpecError {
                        code: SpecErrorCode::ESpec001,
                        message: format!(
                            "sensor '{}' table '{}' column '{}': timestamp_formats or \
                             timestamp_fallback_chain declared on a '{:?}' column; \
                             these fields are only valid on Datetime columns \
                             (BC-2.16.009; ADR-028 v1.10 §D8-C)",
                            spec.sensor_id, table.table_name, col.name, col.column_type,
                        ),
                        toml_path: Some(format!(
                            "sensor.tables[{}].columns[{}]",
                            table.table_name, col.name
                        )),
                        file_path: None,
                        line_number: None,
                    }));
                }

                // Stage 2: for Datetime columns, validate that each named format is in the
                // recognized closed set.
                if col.column_type == ColumnType::Datetime {
                    for fmt in &col.timestamp_formats {
                        if !RECOGNIZED_TIMESTAMP_FORMATS.contains(&fmt.as_str()) {
                            return Err(PrismError::Spec(SpecError {
                                code: SpecErrorCode::ESpec001,
                                message: format!(
                                    "sensor '{}' table '{}' column '{}': unrecognized \
                                     timestamp_formats entry '{}'. Recognized values: {}. \
                                     (BC-2.16.009; ADR-028 v1.10 §D8-C)",
                                    spec.sensor_id,
                                    table.table_name,
                                    col.name,
                                    fmt,
                                    RECOGNIZED_TIMESTAMP_FORMATS.join(", ")
                                ),
                                toml_path: Some(format!(
                                    "sensor.tables[{}].columns[{}].timestamp_formats",
                                    table.table_name, col.name
                                )),
                                file_path: None,
                                line_number: None,
                            }));
                        }
                    }
                }
            }
        }

        // Stage 3 (F-LP3-MEDIUM-001): timestamp_fallback_chain field-name resolution gate
        // (BC-2.16.009). Each name in timestamp_fallback_chain must resolve to an actual
        // column on the same table. Self-references (fb_name == col.name) are allowed —
        // the defensive skip guard in normalize_timestamp_fields handles them at runtime.
        // Unknown names → E-SPEC-001 at load time.
        for table in &spec.tables {
            let column_names: std::collections::HashSet<&str> =
                table.columns.iter().map(|c| c.name.as_str()).collect();
            for col in &table.columns {
                for fb_name in &col.timestamp_fallback_chain {
                    // Self-reference: allowed — skip-guard in normalize handles it.
                    if fb_name == &col.name {
                        continue;
                    }
                    if !column_names.contains(fb_name.as_str()) {
                        let mut known: Vec<&str> = column_names.iter().copied().collect();
                        known.sort_unstable();
                        return Err(PrismError::Spec(SpecError {
                            code: SpecErrorCode::ESpec001,
                            message: format!(
                                "sensor '{}' table '{}' column '{}': \
                                 timestamp_fallback_chain references unknown field '{}'. \
                                 Known columns on table '{}': [{}]. \
                                 (BC-2.16.009; ADR-028 v1.10 §D8-B)",
                                spec.sensor_id,
                                table.table_name,
                                col.name,
                                fb_name,
                                table.table_name,
                                known.join(", "),
                            ),
                            toml_path: Some(format!(
                                "sensor.tables[{}].columns[{}].timestamp_fallback_chain",
                                table.table_name, col.name,
                            )),
                            file_path: None,
                            line_number: None,
                        }));
                    }
                }
            }
        }

        // Stage 4 (ENRICH-1): source_path validation gate.
        //
        // When `source_path` is `Some(p)`:
        //   - `p` must begin with `$.` (the extract_at_path prefix convention).
        //   - `p` must have at least one key segment after `$.` (i.e., not just `"$."`)
        //     to avoid the empty-path error surface already guarded by extract_at_path.
        //
        // Rationale: early rejection at parse time gives a clear actionable error rather
        // than a runtime extraction failure. Runtime validation of wildcard syntax is
        // deferred per design §1 (handled by extract_at_path on first execution).
        for table in &spec.tables {
            for col in &table.columns {
                if let Some(ref p) = col.source_path {
                    if !p.starts_with("$.") {
                        return Err(PrismError::Spec(SpecError {
                            code: SpecErrorCode::ESpec001,
                            message: format!(
                                "sensor '{}' table '{}' column '{}': \
                                 source_path '{p}' must start with '$.'. \
                                 Use JSONPath expressions like '$.field', '$.a.b', \
                                 or '$.arr[*].field' (ENRICH-1 §Design Decision 1).",
                                spec.sensor_id, table.table_name, col.name
                            ),
                            toml_path: Some(format!(
                                "sensor.tables[{}].columns[{}].source_path",
                                table.table_name, col.name
                            )),
                            file_path: None,
                            line_number: None,
                        }));
                    }
                    // Reject bare "$." with no key segment after it.
                    let after_prefix = p.trim_start_matches("$.");
                    if after_prefix.is_empty() {
                        return Err(PrismError::Spec(SpecError {
                            code: SpecErrorCode::ESpec001,
                            message: format!(
                                "sensor '{}' table '{}' column '{}': \
                                 source_path '{p}' must contain at least one key segment \
                                 after '$.' (ENRICH-1 §Design Decision 1).",
                                spec.sensor_id, table.table_name, col.name
                            ),
                            toml_path: Some(format!(
                                "sensor.tables[{}].columns[{}].source_path",
                                table.table_name, col.name
                            )),
                            file_path: None,
                            line_number: None,
                        }));
                    }
                }
            }
        }

        // Rule 8 (E-SPEC-026 / AC-8): if probe_table is declared, it MUST match a table_name
        // in the spec's [[tables]] blocks. An empty tables list is also rejected — there is
        // nothing to probe against.
        //
        // Validation: collect declared table names, check membership, build sorted list for
        // the error message (BC-2.16.009 Rule 8 / probe-table-field-design.md §1).
        if let Some(ref pt) = spec.probe_table {
            let mut declared_names: Vec<String> =
                spec.tables.iter().map(|t| t.table_name.clone()).collect();
            declared_names.sort_unstable();

            if !declared_names.iter().any(|name| name == pt) {
                // F-S504-P2-003: empty tables renders as "[]" (the brackets come from the
                // format string below), not "(none declared)".  Do not inject placeholder text
                // between the brackets — the spec requires "Declared tables: []" for an empty list.
                let tables_list = declared_names.join(", ");
                return Err(PrismError::Spec(SpecError {
                    code: SpecErrorCode::ESpec026,
                    message: format!(
                        "sensor '{}' declares probe_table = '{}' but no [[tables]] block \
                         has table_name = '{}'. Declared tables: [{}]. Remove probe_table \
                         or add a matching [[tables]] block.",
                        spec.sensor_id, pt, pt, tables_list,
                    ),
                    toml_path: Some("sensor.probe_table".to_string()),
                    file_path: None,
                    line_number: None,
                }));
            }
        }

        Ok(spec)
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
                Ok(mut spec) => {
                    // BC-2.16.001 §Error Conditions E-SPEC-017:
                    // The filename stem must case-sensitively match the spec's sensor_id.
                    // E.g., `crowdstrike.sensor.toml` → stem = "crowdstrike" → must match
                    // sensor_id "crowdstrike". Generic check — no hardcoded sensor names
                    // (BC-2.16.012 INV-SPEC-PARSER-OPEN-001).
                    let stem = file_name
                        .strip_suffix(".sensor.toml")
                        .unwrap_or(&file_name)
                        .to_string();
                    if stem != spec.sensor_id {
                        errors.push(PrismError::Spec(SpecError {
                            code: SpecErrorCode::ESpec017,
                            // error-taxonomy.md §E-SPEC-017 canonical message template
                            // (POLICY 24: byte-for-byte match required):
                            // "Spec sensor_id '{sensor_id}' does not match filename stem
                            //  '{filename_stem}' in spec file '{file}'"
                            message: format!(
                                "Spec sensor_id '{}' does not match filename stem '{}' \
                                 in spec file '{}'",
                                spec.sensor_id, stem, file_name
                            ),
                            toml_path: None,
                            file_path: Some(file_name.clone()),
                            line_number: None,
                        }));
                        // DI-030: reject this spec, continue loading others
                        continue;
                    }

                    // BC-2.16.009 §Validation Rules 6 (AC-6) — env-var token resolution.
                    //
                    // F-PR1-MED-001 FIX: Rule 6 must run BEFORE Rule 7 in ALL spec-load paths,
                    // including load_all. The previous code skipped Rule 6 here, causing an env-
                    // resolved invalid method (e.g., method="${env.M}" with M=CONNECT) to be
                    // silently skipped by Rule 7's skip-guard instead of rejected with E-SPEC-025
                    // (EC-009-019 divergence from the parse_and_validate_spec_toml path).
                    //
                    // Rule 6 resolves `${env.VAR_NAME}` tokens in all String fields in-place.
                    // Unresolvable tokens (var absent or empty) → E-SPEC-024 (fail-closed per
                    // AD-017 no-value-leak; var NAME only in error, never the resolved VALUE).
                    // DI-030: env errors → reject this spec, continue loading others.
                    //
                    // AD-017: `resolve_env_var_tokens` emits only var NAME + toml_path, never
                    // the resolved value — no credential leak through this error path.
                    //
                    // BC-2.16.009 §VR6→VR7 ordering; S-SPEC-HTTP-METHOD-VALIDATION-001;
                    // error-taxonomy.md E-SPEC-024.
                    {
                        let env_errors =
                            crate::env_resolver::resolve_env_var_tokens(&mut spec, &file_name);
                        if !env_errors.is_empty() {
                            for env_err in env_errors {
                                // Route through pinned Display — env_resolver.rs #[error(...)]
                                // is the single source of truth for E-SPEC-024 messages.
                                errors.push(PrismError::Spec(SpecError {
                                    code: SpecErrorCode::ESpec024,
                                    message: env_err.to_string(),
                                    toml_path: None,
                                    file_path: Some(file_name.clone()),
                                    line_number: None,
                                }));
                            }
                            // DI-030: reject this spec, continue loading others.
                            continue;
                        }
                    }

                    // BC-2.16.009 §Validation Rules 7 (AC-7) — HTTP method whitelist validation.
                    //
                    // Runs AFTER Rule 6 env-var resolution (F-PR1-MED-001 fix). Now that all
                    // `${env.VAR}` tokens are resolved, the skip-guard in validate_step_methods
                    // fires only for the residual case: a step whose method token was not
                    // resolved because Rule 6 produced an E-SPEC-024 error. Since Rule 6 errors
                    // cause a `continue` above, this code path only sees specs with all tokens
                    // successfully resolved. The skip-guard is therefore moot for load_all after
                    // this fix, but is retained as belt-and-suspenders for future callers.
                    //
                    // Invalid method values → E-SPEC-025 via structured PrismError::Spec channel.
                    // S-SPEC-HTTP-METHOD-VALIDATION-001; BC-2.16.009 §VR7; error-taxonomy.md E-SPEC-025.
                    let method_errors = crate::validation::validate_step_methods(&spec);
                    if !method_errors.is_empty() {
                        for (ti, si, method_err) in method_errors {
                            // Convert SpecEngineError::InvalidHttpMethod → PrismError::Spec(ESpec025).
                            // Route through the pinned Display (error.rs #[error(...)]) rather than
                            // a duplicate format!() literal — error.rs is the single source of truth
                            // for the E-SPEC-025 message (test_BC_2_16_009_e_spec_025_display_…
                            // pins that Display byte-for-byte per POL-24).
                            //
                            // F-LOCAL-P4-MED-001 / F-LOCAL-P2-MED-001: toml_path uses the NUMERIC
                            // indices (ti, si) carried directly from validate_step_methods' enumerate
                            // loop — NO name reverse-lookup. Reverse-lookup by step_name is fragile
                            // when two steps in one table share the same name (step-name uniqueness
                            // is NOT enforced; see F-LOCAL-P4-MED-001 root-cause analysis).
                            match &method_err {
                                crate::error::SpecEngineError::InvalidHttpMethod { .. } => {
                                    let toml_path =
                                        Some(format!("sensor.tables[{ti}].steps[{si}].method"));
                                    errors.push(PrismError::Spec(SpecError {
                                        code: SpecErrorCode::ESpec025,
                                        message: method_err.to_string(),
                                        toml_path,
                                        file_path: Some(file_name.clone()),
                                        line_number: None,
                                    }));
                                }
                                // Unreachable: validate_step_methods only emits InvalidHttpMethod.
                                // If a future refactor adds new variants, this arm surfaces them
                                // as Internal errors rather than silently swallowing.
                                other => {
                                    errors.push(PrismError::Internal {
                                        detail: format!(
                                            "unexpected error from validate_step_methods in load_all: {other}"
                                        ),
                                    });
                                }
                            }
                        }
                        // DI-030: reject this spec, continue loading others.
                        continue;
                    }
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

    /// Validate SensorAuth × DataSource cross-composition rules at credential-validation pass.
    ///
    /// Enforces the three runtime rejection rules introduced when the `SensorAuth` sealed
    /// trait is removed (S-PLUGIN-PREREQ-E / BC-2.01.016 Rule 2 / ADR-023 Rule 2):
    ///
    /// - **Rule A / E-SPEC-012:** `auth_type` is multi-valued or outside the closed
    ///   enumeration `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key,
    ///   custom_via_plugin}`.
    /// - **Rule B / E-SPEC-013:** `credential_refs` cardinality must match the auth method's
    ///   schema: exactly 2 for `oauth2_client_credentials` (client_id + client_secret per
    ///   BC-2.06.003 / ADR-032), exactly 1 for all other auth types.
    /// - **Rule C / E-SPEC-014:** Structural mismatch between resolved credential shape and
    ///   declared `auth_type`.
    ///
    /// Returns `Ok(())` if all three rules pass, or the first `Err(SpecEngineError::Auth*)` on
    /// violation (fail-fast per ADR-026 D3).
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-3 / AC-3b / AC-3c / Task 6b | ADR-026 §D3 | ADR-023 Rule 2
    pub fn validate_cross_composition(
        sensor_id: &str,
        auth_type: &str,
        credential_refs_count: usize,
        expected_shape: &str,
        actual_shape: &str,
    ) -> Result<(), crate::error::SpecEngineError> {
        use crate::error::SpecEngineError;

        // Rule A (E-SPEC-012): auth_type must be a scalar from the closed enumeration.
        // {oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}
        const VALID_AUTH_TYPES: &[&str] = &[
            "oauth2_client_credentials",
            "bearer_static",
            "cookie_roundtrip",
            "api_key",
            "custom_via_plugin",
        ];
        if !VALID_AUTH_TYPES.contains(&auth_type) {
            return Err(SpecEngineError::AuthTypeCrossComposition {
                sensor_id: sensor_id.to_string(),
                provided_value: auth_type.to_string(),
            });
        }

        // Rule B (E-SPEC-013): credential_ref cardinality must match the auth method's schema.
        //
        // Per BC-2.06.003 / ADR-032 (per-client credential convention):
        //   - `oauth2_client_credentials` requires exactly 2 refs: client_id + client_secret
        //     (BC-2.06.003 §Per-Sensor credential_refs Declarations, CrowdStrike entry).
        //   - All other auth types require exactly 1 ref.
        //
        // Note: Rule B fires only when credential_refs.len() > 1 (call site in SpecLoader::parse).
        // A sensor with 0 credential_refs is valid at parse time (boot step 5 validates existence).
        let expected_ref_count = if auth_type == "oauth2_client_credentials" {
            2
        } else {
            1
        };
        if credential_refs_count != expected_ref_count {
            return Err(SpecEngineError::MultipleCredentialRefs {
                sensor_id: sensor_id.to_string(),
                credential_count: credential_refs_count,
            });
        }

        // Rule C (E-SPEC-014): resolved credential structural shape must match auth_type.
        if expected_shape != actual_shape {
            return Err(SpecEngineError::AuthTypeCredentialMismatch {
                sensor_id: sensor_id.to_string(),
                expected_shape: expected_shape.to_string(),
                actual_shape: actual_shape.to_string(),
            });
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// F-LP2-HIGH-005 — validator rejects timestamp_formats / timestamp_fallback_chain
// on non-Datetime columns.
// BC-2.16.009; ADR-028 v1.10 §D8-C.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod timestamp_column_type_validation_tests {
    use prism_core::PrismError;

    use super::SpecLoader;

    /// Minimal valid sensor TOML with a single column (no credential_refs, no timestamp fields).
    /// Sensors with 0 credential_refs are valid at parse time (Rule B only applies when ≥ 2).
    const MINIMAL_TOML_BASE: &str = r#"
sensor_id = "test"
name = "Test Sensor"
auth_type = "bearer_static"
base_url = "https://example.com"
version = "1.0.0"

[[tables]]
table_name = "events"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "REPLACE_COL"
  column_type = "REPLACE_TYPE"
REPLACE_TIMESTAMP_FIELDS
  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/api/v1/events"
  response_path = "$.data"
  variables_produced = []
  [tables.steps.pagination]
  type = "none"
"#;

    /// F-LP2-HIGH-005 negative: timestamp_formats on a String column → E-SPEC-001.
    #[test]
    fn test_validation_rejects_timestamp_formats_on_string_column() {
        let toml = MINIMAL_TOML_BASE
            .replace("REPLACE_COL", "alert_id")
            .replace("REPLACE_TYPE", "string")
            .replace(
                "REPLACE_TIMESTAMP_FIELDS",
                "  timestamp_formats = [\"iso8601\"]\n",
            );
        let err = SpecLoader::parse(&toml)
            .expect_err("String column with timestamp_formats must fail validation");
        match err {
            PrismError::Spec(se) => {
                let msg = &se.message;
                assert!(
                    msg.contains("alert_id"),
                    "must cite column name; got: {msg}"
                );
                assert!(
                    msg.contains("timestamp_formats"),
                    "must mention timestamp_formats; got: {msg}"
                );
                assert!(
                    msg.contains("Datetime"),
                    "must mention Datetime restriction; got: {msg}"
                );
            }
            other => panic!("expected PrismError::Spec, got: {other:?}"),
        }
    }

    /// F-LP2-HIGH-005 negative: timestamp_fallback_chain on an Integer column → E-SPEC-001.
    #[test]
    fn test_validation_rejects_timestamp_fallback_chain_on_integer_column() {
        let toml = MINIMAL_TOML_BASE
            .replace("REPLACE_COL", "count")
            .replace("REPLACE_TYPE", "integer")
            .replace(
                "REPLACE_TIMESTAMP_FIELDS",
                "  timestamp_fallback_chain = [\"other_field\"]\n",
            );
        let err = SpecLoader::parse(&toml)
            .expect_err("Integer column with timestamp_fallback_chain must fail validation");
        match err {
            PrismError::Spec(se) => {
                let msg = &se.message;
                assert!(msg.contains("count"), "must cite column name; got: {msg}");
                assert!(
                    msg.contains("timestamp_fallback_chain"),
                    "must mention timestamp_fallback_chain; got: {msg}"
                );
            }
            other => panic!("expected PrismError::Spec, got: {other:?}"),
        }
    }

    /// F-LP2-HIGH-005 positive: Datetime column with timestamp_formats → validation passes.
    #[test]
    fn test_validation_accepts_timestamp_formats_on_datetime_column() {
        let toml = MINIMAL_TOML_BASE
            .replace("REPLACE_COL", "created_at")
            .replace("REPLACE_TYPE", "datetime")
            .replace(
                "REPLACE_TIMESTAMP_FIELDS",
                "  timestamp_formats = [\"iso8601\", \"unix_epoch_seconds\"]\n",
            );
        assert!(
            SpecLoader::parse(&toml).is_ok(),
            "Datetime column with recognized timestamp_formats must pass validation"
        );
    }

    /// F-LP2-HIGH-005 backward-compat: String column with no timestamp fields → validation passes.
    #[test]
    fn test_validation_accepts_empty_timestamp_fields_on_string_column() {
        let toml = MINIMAL_TOML_BASE
            .replace("REPLACE_COL", "alert_id")
            .replace("REPLACE_TYPE", "string")
            .replace("REPLACE_TIMESTAMP_FIELDS", "");
        assert!(
            SpecLoader::parse(&toml).is_ok(),
            "String column with empty timestamp fields must pass validation (backward compat)"
        );
    }

    // ── S-ADR058-OCSF-ROUTING-001 Red Gate Tests ───────────────────────────────

    /// RG-001 / AC-001 / BC-2.16.003 §Column Routing
    ///
    /// `SensorSpec` deserialized from TOML without `ocsf_column_naming` key MUST default
    /// to `false` via `#[serde(default)]`. This exercises the serde default path.
    ///
    /// GREEN-BY-DESIGN: the field + `#[serde(default)]` + Default impl were added in the
    /// stub commit. This test is a LOAD-BEARING REGRESSION GUARD that prevents the default
    /// from silently flipping to `true` in a future refactor.
    #[test]
    fn test_sensor_spec_ocsf_column_naming_defaults_to_false() {
        let toml = r#"
sensor_id = "test"
name = "Test Sensor"
auth_type = "api_key"
base_url = "https://example.com"
version = "1.0.0"
"#;
        let spec =
            SpecLoader::parse(toml).expect("minimal TOML without ocsf_column_naming must parse");
        assert!(
            !spec.ocsf_column_naming,
            "AC-001 (RG-001): SensorSpec deserialized without ocsf_column_naming must default \
             to false via #[serde(default)]; got ocsf_column_naming = {}. \
             A true default would route every non-Claroty sensor through the OCSF naming branch, \
             breaking CrowdStrike/Armis/Cyberint col.name semantics (AC-004 regression).",
            spec.ocsf_column_naming
        );
    }

    /// RG-002 / AC-001 / BC-2.16.003 §Column Routing
    ///
    /// `SensorSpec` deserialized from TOML WITH `ocsf_column_naming = true` MUST parse
    /// the field as `true` (not silently ignored or rejected).
    ///
    /// GREEN-BY-DESIGN: the field + `#[serde(default)]` were added in the stub commit.
    /// This test is a LOAD-BEARING REGRESSION GUARD.
    #[test]
    fn test_sensor_spec_ocsf_column_naming_parses_true_from_toml() {
        let toml = r#"
sensor_id = "claroty"
name = "Claroty Test"
auth_type = "api_key"
base_url = "https://example.com"
version = "1.0.0"
ocsf_column_naming = true
"#;
        let spec = SpecLoader::parse(toml)
            .expect("TOML with ocsf_column_naming = true must parse successfully");
        assert!(
            spec.ocsf_column_naming,
            "AC-001 (RG-002): SensorSpec deserialized with ocsf_column_naming = true must \
             carry true; got ocsf_column_naming = {}. \
             A false result means the serde field name or attribute is wrong.",
            spec.ocsf_column_naming
        );
    }
}

// =============================================================================
// BC-2.16.016: Claroty xDome OT Activity Events Table — spec-parse Red Gate tests
// S-CLAROTY-OT-EVENTS-001 RG-001 + RG-002
//
// RED GATE: Both tests panic at `.expect("claroty_ot_activity_events table must exist")`
// because the [[tables]] block has not yet been added to claroty.sensor.toml.
// They compile cleanly but assert-fail before the TOML block is added.
//
// BC: BC-2.16.016
// Story: S-CLAROTY-OT-EVENTS-001
// Tasks: Task 1 (spec_parser.rs inline tests)
// =============================================================================

#[cfg(test)]
mod claroty_ot_activity_events_parse_tests {
    use super::SpecLoader;
    use prism_core::ColumnType;

    const CLAROTY_TOML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../prism-sensors/specs/claroty.sensor.toml"
    ));

    // ── RG-001 ────────────────────────────────────────────────────────────────
    /// BC-2.16.016 §Precondition P1 + AC-001:
    ///   The `[[tables]]` block with `table_name = "ot_activity_events"` must parse
    ///   without error and appear in the SensorSpec tables list.
    ///
    ///   Additional assertions (AC-001):
    ///   - `ocsf_column_naming = true` on the claroty sensor (ADR-058 §D2)
    ///   - `ocsf_class = "detection_finding"` on the table
    ///   - exactly 21 ColumnSpec entries declared
    ///   - `fetch_ot_activity_events` step exists and its `body_template` carries a
    ///     `"fields"` array containing all 21 declared column names (BC-2.16.016 §TOML
    ///     Contract §body_template)
    ///
    /// RED: panics at `.expect("claroty_ot_activity_events table must exist")` because
    ///      the [[tables]] block has not been added to claroty.sensor.toml yet.
    ///
    /// BC-2.16.016 AC-001; ADR-058 §D2; S-CLAROTY-OT-EVENTS-001 RG-001.
    #[test]
    fn test_BC_2_16_016_claroty_ot_activity_events_toml_block_parses() {
        let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

        assert!(
            spec.ocsf_column_naming,
            "RG-001: claroty sensor must carry ocsf_column_naming = true (ADR-058 §D2)"
        );

        let table = spec
            .tables
            .iter()
            .find(|t| t.table_name == "ot_activity_events")
            .expect("claroty_ot_activity_events table must exist");

        assert_eq!(
            table.table_name, "ot_activity_events",
            "RG-001: table_name must be 'ot_activity_events'"
        );

        assert_eq!(
            table.ocsf_class.as_str(),
            "detection_finding",
            "RG-001 (AC-001): ocsf_class must be 'detection_finding' \
             (BC-2.16.016 §Postconditions OCSF class_uid 2004; ADR-058 §C2)"
        );

        assert_eq!(
            table.columns.len(),
            21,
            "RG-001 (AC-001): ot_activity_events must declare exactly 21 ColumnSpec entries; \
             got {}: {:?}. BC-2.16.016 §Postconditions §1 (4 Tier-1 + 17 Tier-2).",
            table.columns.len(),
            table.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
        );

        // AC-001: body_template of fetch_ot_activity_events must contain all 21 field names.
        // The Claroty xDome POST API requires a 'fields' projection; a missing column
        // would silently omit data from the outgoing request (BC-2.16.016 §TOML Contract).
        let fetch_step = table
            .steps
            .iter()
            .find(|s| s.name == "fetch_ot_activity_events")
            .expect(
                "RG-001: fetch_ot_activity_events step must exist in ot_activity_events table; \
                 body_template 21-field projection cannot be verified without this step",
            );

        let body_template_str = fetch_step.body_template.as_deref().expect(
            "RG-001: fetch_ot_activity_events step must carry a body_template (AC-001); \
             Claroty API requires a fields projection (minItems: 1)",
        );

        let body_template_json: serde_json::Value = serde_json::from_str(body_template_str)
            .expect("RG-001: fetch_ot_activity_events body_template must be valid JSON (AC-001)");

        let fields_array = body_template_json
            .get("fields")
            .and_then(|v| v.as_array())
            .expect(
                "RG-001: fetch_ot_activity_events body_template must contain a top-level \
                 'fields' JSON array (AC-001); the Claroty xDome POST API requires it",
            );

        let actual_fields: std::collections::HashSet<&str> = fields_array
            .iter()
            .map(|v| {
                v.as_str().expect(
                    "RG-001: each element in body_template 'fields' array must be a JSON string",
                )
            })
            .collect();

        // Ground-truth 21-field projection: all 21 declared columns.
        // BC-2.16.016 §Postconditions §1 + §TOML Contract §body_template.
        let expected_fields: std::collections::HashSet<&str> = [
            "event_id",
            "detection_time",
            "event_type",
            "description",
            "source_ip",
            "dest_ip",
            "protocol",
            "dest_port",
            "source_port",
            "ip_protocol",
            "source_asset_id",
            "dest_asset_id",
            "source_device_name",
            "dest_device_name",
            "source_device_type",
            "dest_device_type",
            "source_site_name",
            "dest_site_name",
            "source_username",
            "related_alert_ids",
            "mode",
        ]
        .iter()
        .copied()
        .collect();

        assert_eq!(
            actual_fields,
            expected_fields,
            "RG-001 LOAD-BEARING: fetch_ot_activity_events body_template 'fields' array must be \
             EXACTLY the 21-field projection. Extra: {:?}, Missing: {:?}. \
             BC-2.16.016 §TOML Contract §body_template.",
            actual_fields
                .difference(&expected_fields)
                .collect::<Vec<_>>(),
            expected_fields
                .difference(&actual_fields)
                .collect::<Vec<_>>()
        );

        assert_eq!(
            fields_array.len(),
            21,
            "RG-001: fetch_ot_activity_events body_template 'fields' array must have exactly 21 \
             elements (no duplicates, no extras); got {}. BC-2.16.016 §TOML Contract.",
            fields_array.len()
        );

        // ── RG-001 v1.1 hardening: related_alert_ids column_type == ColumnType::Json ──
        // BC-2.16.016 §Postconditions §2 (EC-016-016-002 / AC-006):
        //   `related_alert_ids` is a JSON array column and MUST declare column_type = "json"
        //   in claroty.sensor.toml so the pipeline preserves it as a native JSON array
        //   inside raw_extensions (not stringified). F-COE1-P1-LOW-001 closure.
        let related_alert_ids_col = table
            .columns
            .iter()
            .find(|c| c.name == "related_alert_ids")
            .expect(
                "RG-001 v1.1: column 'related_alert_ids' must be present in ot_activity_events. \
                 BC-2.16.016 §Postconditions §2.",
            );
        assert_eq!(
            related_alert_ids_col.column_type,
            ColumnType::Json,
            "RG-001 v1.1 LOAD-BEARING: 'related_alert_ids' ColumnSpec.column_type MUST be \
             ColumnType::Json (declared as `column_type = \"json\"` in claroty.sensor.toml). \
             Got: {:?}. BC-2.16.016 AC-006 / EC-016-016-002; F-COE1-P1-LOW-001.",
            related_alert_ids_col.column_type
        );
    }

    // ── RG-002 ────────────────────────────────────────────────────────────────
    /// BC-2.16.016 §Postconditions AC-002: exactly 4 Tier-1 columns with `ocsf_field`:
    ///   - `event_id`       → `ocsf_field = "finding_info.uid"` (REQUIRED, Integer)
    ///   - `detection_time` → `ocsf_field = "time"`             (Datetime)
    ///   - `event_type`     → `ocsf_field = "activity_name"`    (String)
    ///   - `description`    → `ocsf_field = "message"`          (String)
    ///
    /// Also asserts exactly 17 Tier-2 columns (no ocsf_field) with the correct names.
    ///
    /// RED: panics at `.expect("claroty_ot_activity_events table must exist")` because
    ///      the [[tables]] block has not been added to claroty.sensor.toml yet.
    ///
    /// BC-2.16.016 AC-002; ADR-058 §C2; S-CLAROTY-OT-EVENTS-001 RG-002.
    #[test]
    fn test_BC_2_16_016_claroty_ot_activity_events_four_tier1_columns() {
        let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

        let table = spec
            .tables
            .iter()
            .find(|t| t.table_name == "ot_activity_events")
            .expect("claroty_ot_activity_events table must exist");

        let tier1: Vec<_> = table
            .columns
            .iter()
            .filter(|c| c.ocsf_field.is_some())
            .collect();

        assert_eq!(
            tier1.len(),
            4,
            "RG-002 (AC-002): expected exactly 4 Tier-1 columns with ocsf_field; \
             got {}: {:?}. BC-2.16.016 §Postconditions §1.",
            tier1.len(),
            tier1.iter().map(|c| &c.name).collect::<Vec<_>>()
        );

        // ── event_id → finding_info.uid ───────────────────────────────────────
        let event_id_col = tier1
            .iter()
            .find(|c| c.name == "event_id")
            .expect("RG-002: Tier-1 column 'event_id' must exist (BC-2.16.016 §Postconditions)");
        assert_eq!(
            event_id_col.ocsf_field.as_deref(),
            Some("finding_info.uid"),
            "RG-002: column 'event_id' must declare ocsf_field = \"finding_info.uid\" \
             (BC-2.16.016 AC-002; ADR-058 §C2 Option 4 dot-notation)"
        );

        // ── detection_time → time ─────────────────────────────────────────────
        let detection_time_col = tier1.iter().find(|c| c.name == "detection_time").expect(
            "RG-002: Tier-1 column 'detection_time' must exist (BC-2.16.016 §Postconditions)",
        );
        assert_eq!(
            detection_time_col.ocsf_field.as_deref(),
            Some("time"),
            "RG-002: column 'detection_time' must declare ocsf_field = \"time\" \
             (BC-2.16.016 AC-002; OCSF detection_finding time field)"
        );

        // ── event_type → activity_name ────────────────────────────────────────
        let event_type_col = tier1
            .iter()
            .find(|c| c.name == "event_type")
            .expect("RG-002: Tier-1 column 'event_type' must exist (BC-2.16.016 §Postconditions)");
        assert_eq!(
            event_type_col.ocsf_field.as_deref(),
            Some("activity_name"),
            "RG-002: column 'event_type' must declare ocsf_field = \"activity_name\" \
             (BC-2.16.016 AC-002; OCSF detection_finding activity_name field)"
        );

        // ── description → message ─────────────────────────────────────────────
        let description_col = tier1
            .iter()
            .find(|c| c.name == "description")
            .expect("RG-002: Tier-1 column 'description' must exist (BC-2.16.016 §Postconditions)");
        assert_eq!(
            description_col.ocsf_field.as_deref(),
            Some("message"),
            "RG-002: column 'description' must declare ocsf_field = \"message\" \
             (BC-2.16.016 AC-002; OCSF detection_finding message field)"
        );

        // ── Tier-2: exactly 17 columns ────────────────────────────────────────
        let tier2: Vec<_> = table
            .columns
            .iter()
            .filter(|c| c.ocsf_field.is_none())
            .collect();

        assert_eq!(
            tier2.len(),
            17,
            "RG-002 (AC-002): expected exactly 17 Tier-2 columns without ocsf_field; \
             got {}: {:?}. BC-2.16.016 §Postconditions §1.",
            tier2.len(),
            tier2.iter().map(|c| &c.name).collect::<Vec<_>>()
        );

        let tier2_names: std::collections::HashSet<&str> =
            tier2.iter().map(|c| c.name.as_str()).collect();
        let expected_tier2: std::collections::HashSet<&str> = [
            "source_ip",
            "dest_ip",
            "protocol",
            "dest_port",
            "source_port",
            "ip_protocol",
            "source_asset_id",
            "dest_asset_id",
            "source_device_name",
            "dest_device_name",
            "source_device_type",
            "dest_device_type",
            "source_site_name",
            "dest_site_name",
            "source_username",
            "related_alert_ids",
            "mode",
        ]
        .iter()
        .copied()
        .collect();

        assert_eq!(
            tier2_names,
            expected_tier2,
            "RG-002 LOAD-BEARING: Tier-2 column set must match exactly. \
             Extra: {:?}, Missing: {:?}. BC-2.16.016 §Postconditions §1.",
            tier2_names.difference(&expected_tier2).collect::<Vec<_>>(),
            expected_tier2.difference(&tier2_names).collect::<Vec<_>>()
        );
    }
}
