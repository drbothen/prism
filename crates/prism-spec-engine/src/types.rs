// S-1.12: Core types for prism-spec-engine hot reload and runtime management.
// These types are the S-1.11 surface on which S-1.12 builds.
// Origin: S-1.11 established SensorSpec, SensorTableDescriptor, and validation
// infrastructure — referenced here as local stubs per the story dependency model.

use serde::{Deserialize, Serialize};

/// Column type for a sensor table column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColumnType {
    String,
    Int64,
    Float64,
    Boolean,
    Timestamp,
    Json,
}

/// A single column definition within a sensor table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub column_type: ColumnType,
    /// OCSF field path mapping (e.g., "event.src_endpoint.ip")
    pub ocsf_field: Option<String>,
    pub nullable: bool,
}

/// Pagination type for a fetch pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaginationType {
    Cursor,
    Offset,
    None,
}

/// Exported table descriptor for downstream prism-query consumption.
/// Origin: S-1.11 — SensorTableDescriptor is the public export from spec loading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorTableDescriptor {
    /// Fully-qualified table name: "{sensor_id}.{table_name}"
    pub table_name: String,
    pub columns: Vec<ColumnDef>,
    pub steps_count: usize,
    pub pagination_type: PaginationType,
}

/// Parsed representation of a .sensor.toml file.
/// Origin: S-1.11 — SensorSpec is the parsed representation established there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorSpec {
    pub sensor_id: String,
    pub name: String,
    pub version: String,
    pub auth_type: String,
    pub base_url: String,
    pub tables: Vec<SensorTableDescriptor>,
    /// SHA-256 hash of the source file content (for change detection)
    pub file_hash: String,
    /// Source file path
    pub source_path: String,
}

/// Per-spec availability status for list_sensor_specs (BC-2.16.010).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecStatus {
    Loaded,
    FailedValidation,
    PendingReload,
    NoCredentials,
    ValidationWarnings { warnings: Vec<String> },
}

/// Per-client credential status in list_sensor_specs response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientStatus {
    Configured,
    NotConfigured,
}

/// Entry in the list_sensor_specs response (BC-2.16.010).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorSpecEntry {
    pub sensor_id: String,
    pub name: String,
    pub version: String,
    pub auth_type: String,
    pub base_url: String,
    pub tables: Vec<SensorTableDescriptor>,
    pub status: SpecStatus,
    pub client_status: Option<ClientStatus>,
}

/// Immutable configuration snapshot. Arc-swapped atomically on reload.
/// Origin: AD-018 — ArcSwap<ConfigSnapshot> is the mandated config access pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// All successfully loaded sensor specs, keyed by sensor_id
    pub sensor_specs: std::collections::HashMap<String, SensorSpec>,
    /// Specs that failed validation (tracked for list_sensor_specs status)
    pub failed_specs: std::collections::HashMap<String, ValidationError>,
    /// SHA-256 hash of all config files combined (for change detection)
    pub snapshot_hash: String,
}

impl ConfigSnapshot {
    pub fn empty() -> Self {
        Self {
            sensor_specs: std::collections::HashMap::new(),
            failed_specs: std::collections::HashMap::new(),
            snapshot_hash: String::new(),
        }
    }
}

/// Structured validation error for a spec file (BC-2.16.009).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub sensor_id: Option<String>,
    pub source_path: String,
    pub errors: Vec<String>,
}

/// Result of a reload operation (BC-2.16.005 postconditions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadResult {
    pub status: ReloadStatus,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<ModifiedSpec>,
    pub unchanged: Vec<String>,
    pub validation_errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReloadStatus {
    Ok,
    Unchanged,
    ValidationFailed,
    PartialReload,
    DryRun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedSpec {
    pub sensor_id: String,
    pub table_names: Vec<String>,
    pub schema_changed: bool,
}

/// Arguments to reload_config (BC-2.16.005).
#[derive(Debug, Clone, Default)]
pub struct ReloadConfigArgs {
    pub dry_run: bool,
}

/// Arguments to add_sensor_spec (BC-2.16.008).
#[derive(Debug, Clone)]
pub struct AddSensorSpecArgs {
    pub spec_toml: String,
    pub file_name: Option<String>,
    pub dry_run: bool,
}

/// Result of add_sensor_spec (BC-2.16.008).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddSensorSpecResult {
    Added {
        sensor_id: String,
        tables: Vec<SensorTableDescriptor>,
    },
    DryRun {
        sensor_id: String,
        tables: Vec<SensorTableDescriptor>,
        validation_errors: Vec<String>,
    },
    ConfirmationRequired {
        sensor_id: String,
        confirmation_token: String,
    },
    ValidationFailed {
        errors: Vec<ValidationError>,
    },
    WriteError {
        path: String,
        os_error: String,
    },
}

/// Arguments to list_sensor_specs (BC-2.16.010).
#[derive(Debug, Clone, Default)]
pub struct ListSensorSpecsArgs {
    pub client_id: Option<String>,
    pub sensor_id: Option<String>,
}

/// Result of list_sensor_specs (BC-2.16.010).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSensorSpecsResult {
    pub specs: Vec<SensorSpecEntry>,
    pub total_specs: usize,
    pub total_tables: usize,
}
