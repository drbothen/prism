//! TOML spec file parser and descriptor types.
//!
//! Parses `*.sensor.toml` files into `SensorSpec` structs and produces
//! `SensorTableDescriptor` values for downstream consumption by prism-query.
//!
//! # Architecture Compliance
//! - Does NOT import DataFusion or Arrow.
//! - `SensorTableDescriptor` uses `prism_core::ColumnType` only.
//! - Table name conflicts are detected at load time (BC-2.16.001 postcondition).

use prism_core::{ColumnOptions, ColumnType, PrismError};
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
    pub options: Vec<ColumnOptions>,
}

/// A table within a sensor spec (BC-2.16.001).
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
// SpecLoader — stub (all methods unimplemented!)
// ---------------------------------------------------------------------------

/// Loads sensor specs from a directory of `*.sensor.toml` files (BC-2.16.001).
///
/// Scans `sensor_specs_dir` (flat, non-recursive), parses each file, validates it,
/// and returns the set of `SensorTableDescriptor`s for DataFusion registration.
/// Invalid specs are skipped with errors; valid specs load independently (DI-030).
pub struct SpecLoader {
    _sensor_specs_dir: String,
}

impl SpecLoader {
    /// Create a new SpecLoader for the given directory.
    pub fn new(sensor_specs_dir: impl Into<String>) -> Self {
        SpecLoader {
            _sensor_specs_dir: sensor_specs_dir.into(),
        }
    }

    /// Parse a single TOML string into a `SensorSpec`.
    ///
    /// Returns `Ok(SensorSpec)` or `Err(PrismError)` — never panics (VP-023).
    pub fn parse(toml_input: &str) -> Result<SensorSpec, PrismError> {
        unimplemented!("SpecLoader::parse — implement in S-1.11 (BC-2.16.001)")
    }

    /// Load all `*.sensor.toml` files from `sensor_specs_dir`.
    ///
    /// Returns (descriptors, errors): valid specs produce descriptors; invalid files
    /// produce errors but do not block valid specs from loading (DI-030).
    pub fn load_all(&self) -> (Vec<SensorTableDescriptor>, Vec<PrismError>) {
        unimplemented!("SpecLoader::load_all — implement in S-1.11 (BC-2.16.001)")
    }

    /// Detect duplicate table names across multiple specs.
    ///
    /// Returns error codes for any second-occurrence table names (BC-2.16.001).
    pub fn detect_table_name_conflicts(specs: &[SensorSpec]) -> Vec<PrismError> {
        unimplemented!(
            "SpecLoader::detect_table_name_conflicts — implement in S-1.11 (BC-2.16.001)"
        )
    }

    /// Detect duplicate sensor_ids across spec files.
    ///
    /// Returns E-SPEC-009 for each second-occurrence sensor_id (BC-2.16.001).
    pub fn detect_sensor_id_conflicts(specs: &[(String, SensorSpec)]) -> Vec<PrismError> {
        unimplemented!("SpecLoader::detect_sensor_id_conflicts — implement in S-1.11 (BC-2.16.001)")
    }
}
