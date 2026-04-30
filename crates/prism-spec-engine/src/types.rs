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
    /// DTU deployment mode — set at TOML parse time, never changed at runtime.
    ///
    /// Defaults to `DtuMode::Shared` for backward compatibility with existing
    /// TOML files that do not specify a `mode` field (BC-3.2.005, D-161 lesson).
    #[serde(default)]
    pub mode: DtuMode,
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

/// DTU deployment mode — set once at startup, never mutated at runtime.
///
/// # BC-3.2.005 Invariant 1
/// `DtuMode` is `Copy` — it is a value type with no interior mutability.
/// The `mode` field in the sensor spec registration struct is set exactly once,
/// at startup parse time, and has no setter method.
///
/// `#[non_exhaustive]` prevents external match arms from exhausting the enum
/// without a wildcard, enabling future mode variants (e.g. `Isolated`) without
/// a breaking semver change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DtuMode {
    /// Shared-mode: single adapter instance serves all orgs.
    Shared,
    /// Client-mode: one adapter instance per customer org.
    Client,
}

impl Default for DtuMode {
    /// Default DTU deployment mode is `Shared` (MSSP Coordination pattern).
    ///
    /// Used as the `#[serde(default)]` value for `SensorSpec.mode` so that
    /// existing TOML files without an explicit `mode` field deserialise without
    /// error (forward-compat guard, D-161 lesson).
    fn default() -> Self {
        DtuMode::Shared
    }
}

/// A detected mode change that was suppressed during `reload_config`.
///
/// Produced when the incoming customer TOML changes the `mode` field of a
/// `[[dtu]]` block.  The change is NOT applied — the old mode is preserved —
/// and this struct is returned in `ReloadResult::mode_change_warnings` so the
/// caller can surface an actionable warning to the operator.
///
/// # BC-3.2.005 Invariant 4 + EC-006
/// "reload_config detects the mode change, emits a warning that mode changes
/// require restart, but does not apply the change; running mode is preserved."
///
/// Stub added by S-3.3.06 stub-architect phase; full wiring by implementer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeChange {
    /// Organisation slug for the affected `[[dtu]]` block.
    pub org_slug: String,
    /// DTU type string (e.g. `"claroty"`, `"armis"`).
    pub dtu_type: String,
    /// The mode currently active in the running process.
    pub old: DtuMode,
    /// The mode that was requested in the new config file (but not applied).
    pub new: DtuMode,
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
    /// Mode changes detected during reload that were NOT applied.
    ///
    /// Non-empty when at least one `[[dtu]]` block in the new config has a
    /// different `mode` value from the currently-active mode.  The old mode
    /// is always preserved (BC-3.2.005 invariant 4).
    ///
    /// Empty on `ReloadStatus::Unchanged` and when no DTU mode fields changed.
    pub mode_change_warnings: Vec<ModeChange>,
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
