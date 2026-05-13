// S-1.12: Core types for prism-spec-engine hot reload and runtime management.
// These types are the S-1.11 surface on which S-1.12 builds.
// Origin: S-1.11 established SensorSpec, SensorTableDescriptor, and validation
// infrastructure — referenced here as local stubs per the story dependency model.

use serde::{Deserialize, Serialize};

// ADR-024: retire the shadow ColumnType stub; use the canonical enum from prism-core.
// prism_core::column::ColumnType has variants String | Integer | Float | Boolean | Datetime | Json
// with #[non_exhaustive] and serde rename_all = "snake_case".
pub use prism_core::column::ColumnType;

/// A single column definition within a sensor table.
///
/// `#[non_exhaustive]`: forward-compat for schema evolution — fields may expand
/// (e.g., default value, index hints, description) without a breaking semver change.
/// Use `..Default::default()` for forward-compatible external construction.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub column_type: ColumnType,
    /// OCSF field path mapping (e.g., "event.src_endpoint.ip")
    pub ocsf_field: Option<String>,
    pub nullable: bool,
}

impl Default for ColumnDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            column_type: ColumnType::String,
            ocsf_field: None,
            nullable: true,
        }
    }
}

/// Pagination type for a fetch pipeline.
///
/// `#[non_exhaustive]`: forward-compat for schema evolution — new pagination strategies
/// (e.g., page-based with link header, keyset pagination) may be added without a breaking
/// semver change. External match arms must include a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaginationType {
    Cursor,
    Offset,
    None,
}

/// Exported table descriptor for downstream prism-query consumption.
/// Origin: S-1.11 — SensorTableDescriptor is the public export from spec loading.
///
/// `#[non_exhaustive]`: forward-compat for hot-reload config schema evolution —
/// table metadata may gain new config fields. External construction must use
/// `..Default::default()` pattern.
///
/// Note: This is the hot-reload infrastructure type (distinct from
/// `spec_parser::SensorTableDescriptor` which is the spec-parser output type).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorTableDescriptor {
    /// Fully-qualified table name: "{sensor_id}.{table_name}"
    pub table_name: String,
    pub columns: Vec<ColumnDef>,
    pub steps_count: usize,
    pub pagination_type: PaginationType,
}

impl Default for SensorTableDescriptor {
    fn default() -> Self {
        Self {
            table_name: String::new(),
            columns: vec![],
            steps_count: 0,
            pagination_type: PaginationType::None,
        }
    }
}

impl SensorTableDescriptor {
    /// Construct a `SensorTableDescriptor`.
    ///
    /// Internal construction shortcut for forward-compatible external construction.
    pub fn new(
        table_name: impl Into<String>,
        columns: Vec<ColumnDef>,
        steps_count: usize,
        pagination_type: PaginationType,
    ) -> Self {
        Self {
            table_name: table_name.into(),
            columns,
            steps_count,
            pagination_type,
        }
    }
}

/// A credential reference declared in a sensor spec.
///
/// References the credential by sensor name and logical key name within that
/// sensor's namespace.  The reference is resolved at boot time (step 5) via the
/// keyring backend — no secret value is ever loaded into memory (AD-017).
///
/// `#[non_exhaustive]`: forward-compat for hot-reload config schema evolution —
/// fields may expand as new auth types are added. External construction must use
/// `..Default::default()` pattern.
///
/// # F-PASS2-HIGH-3 (S-WAVE5-PREP-01)
///
/// BC-2.03.013 §Postconditions (Happy path bullet 2):
/// "All credential references declared in sensor specs are validated as resolvable."
/// `SensorSpec.credential_refs` is the data model field that carries those refs.
///
/// TOML format in `*.types.toml` specs (optional section):
/// ```toml
/// [[credential_refs]]
/// name = "api_key"
///
/// [[credential_refs]]
/// name = "client_secret"
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CredentialRef {
    /// Logical credential name within this sensor's keyring namespace.
    ///
    /// Must be non-empty and match `[a-zA-Z0-9_-]+` (validated at parse time).
    /// Example: `"api_key"`, `"client_secret"`.
    pub name: String,
}

impl CredentialRef {
    /// Construct a `CredentialRef` with the given name.
    ///
    /// Internal construction shortcut. External callers should use struct-literal +
    /// `..Default::default()` for forward compatibility when new fields are added.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Parsed representation of a .sensor.toml file.
/// Origin: S-1.11 — SensorSpec is the parsed representation established there.
///
/// `#[non_exhaustive]`: forward-compat for hot-reload config schema evolution —
/// root spec type; fields will expand with ADR-023 grammar. External construction
/// must use `..Default::default()` pattern.
///
/// Note: This is the hot-reload infrastructure type (distinct from
/// `spec_parser::SensorSpec` which is the TOML spec-parser output type).
#[non_exhaustive]
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
    /// Credential references declared by this sensor spec.
    ///
    /// Each ref names a credential in the sensor's keyring namespace that must be
    /// resolvable at boot time (BC-2.03.013 §Postconditions bullet 2).
    /// Empty when the sensor declares no credentials (EC-03-013-001: zero refs
    /// validated is not an error — boot continues normally).
    ///
    /// F-PASS2-HIGH-3 (S-WAVE5-PREP-01): added to support credential-ref
    /// iteration in step5_init_credential_store. Future sensor TOML specs that
    /// declare `[[credential_refs]]` sections will have their refs validated here.
    #[serde(default)]
    pub credential_refs: Vec<CredentialRef>,
}

impl Default for SensorSpec {
    fn default() -> Self {
        Self {
            sensor_id: String::new(),
            name: String::new(),
            version: "1.0.0".to_string(),
            auth_type: "api_key".to_string(),
            base_url: String::new(),
            tables: vec![],
            file_hash: String::new(),
            source_path: String::new(),
            mode: DtuMode::default(),
            credential_refs: vec![],
        }
    }
}

impl SensorSpec {
    /// Construct a `SensorSpec` for the hot-reload config manager.
    ///
    /// Internal construction shortcut for forward-compatible external construction.
    /// Sets `mode = DtuMode::Shared` (default) and `credential_refs = []`.
    #[allow(clippy::too_many_arguments)]
    pub fn new_hot_reload(
        sensor_id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        auth_type: impl Into<String>,
        base_url: impl Into<String>,
        tables: Vec<SensorTableDescriptor>,
        file_hash: impl Into<String>,
        source_path: impl Into<String>,
    ) -> Self {
        Self {
            sensor_id: sensor_id.into(),
            name: name.into(),
            version: version.into(),
            auth_type: auth_type.into(),
            base_url: base_url.into(),
            tables,
            file_hash: file_hash.into(),
            source_path: source_path.into(),
            mode: DtuMode::default(),
            credential_refs: vec![],
        }
    }
}

/// Per-spec availability status for list_sensor_specs (BC-2.16.010).
///
/// `#[non_exhaustive]`: forward-compat for spec lifecycle evolution — new status
/// values (e.g., `Deprecated`, `Upgrading`) may be added without a breaking semver change.
/// External match arms must include a wildcard arm.
#[non_exhaustive]
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
///
/// `#[non_exhaustive]`: forward-compat for credential status evolution — new states
/// (e.g., `Expired`, `PartiallyConfigured`) may be added without a breaking semver change.
/// External match arms must include a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientStatus {
    Configured,
    NotConfigured,
}

/// Entry in the list_sensor_specs response (BC-2.16.010).
///
/// # AC-5 scope exclusion
///
/// This type is intentionally NOT marked `#[non_exhaustive]`. It is an MCP
/// protocol wire type (request DTO / result / event / status), and its stability
/// is governed by the MCP protocol specification rather than by the Rust
/// forward-compatibility property. External consumers (Claude Code / MCP clients)
/// exhaustively match against the protocol's enumerated variants; adding a new
/// variant requires an MCP protocol version bump, not a Rust source-level
/// non-exhaustive annotation. Documented per S-PLUGIN-PREREQ-C F-LP3-MED-002
/// adjudication.
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
///
/// # AC-5 scope exclusion
///
/// This type is intentionally NOT marked `#[non_exhaustive]`. It is an MCP
/// protocol wire type (request DTO / result / event / status), and its stability
/// is governed by the MCP protocol specification rather than by the Rust
/// forward-compatibility property. External consumers (Claude Code / MCP clients)
/// exhaustively match against the protocol's enumerated variants; adding a new
/// variant requires an MCP protocol version bump, not a Rust source-level
/// non-exhaustive annotation. Documented per S-PLUGIN-PREREQ-C F-LP3-MED-002
/// adjudication.
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
///
/// # AC-5 scope exclusion
///
/// This type is intentionally NOT marked `#[non_exhaustive]`. It is an MCP
/// protocol wire type (request DTO / result / event / status), and its stability
/// is governed by the MCP protocol specification rather than by the Rust
/// forward-compatibility property. External consumers (Claude Code / MCP clients)
/// exhaustively match against the protocol's enumerated variants; adding a new
/// variant requires an MCP protocol version bump, not a Rust source-level
/// non-exhaustive annotation. Documented per S-PLUGIN-PREREQ-C F-LP3-MED-002
/// adjudication.
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
///
/// # AC-5 scope exclusion
///
/// This type is intentionally NOT marked `#[non_exhaustive]`. It is an MCP
/// protocol wire type (request DTO / result / event / status), and its stability
/// is governed by the MCP protocol specification rather than by the Rust
/// forward-compatibility property. External consumers (Claude Code / MCP clients)
/// exhaustively match against the protocol's enumerated variants; adding a new
/// variant requires an MCP protocol version bump, not a Rust source-level
/// non-exhaustive annotation. Documented per S-PLUGIN-PREREQ-C F-LP3-MED-002
/// adjudication.
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
///
/// # AC-5 scope exclusion
///
/// This type is intentionally NOT marked `#[non_exhaustive]`. It is an MCP
/// protocol wire type (request DTO / result / event / status), and its stability
/// is governed by the MCP protocol specification rather than by the Rust
/// forward-compatibility property. External consumers (Claude Code / MCP clients)
/// exhaustively match against the protocol's enumerated variants; adding a new
/// variant requires an MCP protocol version bump, not a Rust source-level
/// non-exhaustive annotation. Documented per S-PLUGIN-PREREQ-C F-LP3-MED-002
/// adjudication.
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

/// Status of a reload operation (BC-2.16.005).
///
/// # AC-5 scope exclusion
///
/// This type is intentionally NOT marked `#[non_exhaustive]`. It is an MCP
/// protocol wire type (request DTO / result / event / status), and its stability
/// is governed by the MCP protocol specification rather than by the Rust
/// forward-compatibility property. External consumers (Claude Code / MCP clients)
/// exhaustively match against the protocol's enumerated variants; adding a new
/// variant requires an MCP protocol version bump, not a Rust source-level
/// non-exhaustive annotation. Documented per S-PLUGIN-PREREQ-C F-LP3-MED-002
/// adjudication.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReloadStatus {
    Ok,
    Unchanged,
    ValidationFailed,
    PartialReload,
    DryRun,
}

/// A sensor spec that was modified during a reload operation (BC-2.16.005).
///
/// # AC-5 scope exclusion
///
/// This type is intentionally NOT marked `#[non_exhaustive]`. It is an MCP
/// protocol wire type (request DTO / result / event / status), and its stability
/// is governed by the MCP protocol specification rather than by the Rust
/// forward-compatibility property. External consumers (Claude Code / MCP clients)
/// exhaustively match against the protocol's enumerated variants; adding a new
/// variant requires an MCP protocol version bump, not a Rust source-level
/// non-exhaustive annotation. Documented per S-PLUGIN-PREREQ-C F-LP3-MED-002
/// adjudication.
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
///
/// # AC-5 scope exclusion
///
/// This type is intentionally NOT marked `#[non_exhaustive]`. It is an MCP
/// protocol wire type (request DTO / result / event / status), and its stability
/// is governed by the MCP protocol specification rather than by the Rust
/// forward-compatibility property. External consumers (Claude Code / MCP clients)
/// exhaustively match against the protocol's enumerated variants; adding a new
/// variant requires an MCP protocol version bump, not a Rust source-level
/// non-exhaustive annotation. Documented per S-PLUGIN-PREREQ-C F-LP3-MED-002
/// adjudication.
#[derive(Debug, Clone)]
pub struct AddSensorSpecArgs {
    pub spec_toml: String,
    pub file_name: Option<String>,
    pub dry_run: bool,
}

/// Result of add_sensor_spec (BC-2.16.008).
///
/// # AC-5 scope exclusion
///
/// This type is intentionally NOT marked `#[non_exhaustive]`. It is an MCP
/// protocol wire type (request DTO / result / event / status), and its stability
/// is governed by the MCP protocol specification rather than by the Rust
/// forward-compatibility property. External consumers (Claude Code / MCP clients)
/// exhaustively match against the protocol's enumerated variants; adding a new
/// variant requires an MCP protocol version bump, not a Rust source-level
/// non-exhaustive annotation. Documented per S-PLUGIN-PREREQ-C F-LP3-MED-002
/// adjudication.
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
///
/// # AC-5 scope exclusion
///
/// This type is intentionally NOT marked `#[non_exhaustive]`. It is an MCP
/// protocol wire type (request DTO / result / event / status), and its stability
/// is governed by the MCP protocol specification rather than by the Rust
/// forward-compatibility property. External consumers (Claude Code / MCP clients)
/// exhaustively match against the protocol's enumerated variants; adding a new
/// variant requires an MCP protocol version bump, not a Rust source-level
/// non-exhaustive annotation. Documented per S-PLUGIN-PREREQ-C F-LP3-MED-002
/// adjudication.
#[derive(Debug, Clone, Default)]
pub struct ListSensorSpecsArgs {
    pub client_id: Option<String>,
    pub sensor_id: Option<String>,
}

/// Result of list_sensor_specs (BC-2.16.010).
///
/// # AC-5 scope exclusion
///
/// This type is intentionally NOT marked `#[non_exhaustive]`. It is an MCP
/// protocol wire type (request DTO / result / event / status), and its stability
/// is governed by the MCP protocol specification rather than by the Rust
/// forward-compatibility property. External consumers (Claude Code / MCP clients)
/// exhaustively match against the protocol's enumerated variants; adding a new
/// variant requires an MCP protocol version bump, not a Rust source-level
/// non-exhaustive annotation. Documented per S-PLUGIN-PREREQ-C F-LP3-MED-002
/// adjudication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSensorSpecsResult {
    pub specs: Vec<SensorSpecEntry>,
    pub total_specs: usize,
    pub total_tables: usize,
}
