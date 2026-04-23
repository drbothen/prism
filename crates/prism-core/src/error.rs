//! PrismError — canonical error taxonomy for the entire Prism platform.
//!
//! Every variant's Display output MUST begin with its structured error code token,
//! e.g. `"E-STORE-001: ..."`. Callers rely on the prefix for structured logging
//! and metric tagging.

use thiserror::Error;

/// Canonical error type for the Prism platform.
///
/// Covers all 90+ error codes across every subsystem category. Group variants
/// by category prefix; each category maps to a subsystem.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum PrismError {
    // -------------------------------------------------------------------------
    // E-AUTH — Authentication / tenant identity
    // -------------------------------------------------------------------------
    /// E-AUTH-001: Tenant identifier failed validation.
    #[error("E-AUTH-001: invalid tenant ID: {reason}")]
    InvalidTenantId { reason: String },

    /// E-AUTH-002: Analyst identifier failed validation.
    #[error("E-AUTH-002: invalid analyst ID: {reason}")]
    InvalidAnalystId { reason: String },

    /// E-AUTH-003: Client identifier failed validation.
    #[error("E-AUTH-003: invalid client ID: {reason}")]
    InvalidClientId { reason: String },

    /// E-AUTH-010: Auth token expired.
    #[error("E-AUTH-010: auth token expired")]
    AuthTokenExpired,

    /// E-AUTH-011: Auth token invalid.
    #[error("E-AUTH-011: auth token invalid: {reason}")]
    AuthTokenInvalid { reason: String },

    /// E-AUTH-020: Unauthorized — caller lacks required permission.
    #[error("E-AUTH-020: unauthorized: {action}")]
    Unauthorized { action: String },

    // -------------------------------------------------------------------------
    // E-SENSOR — Sensor adapter errors
    // -------------------------------------------------------------------------
    /// E-SENSOR-001: Sensor adapter returned an unexpected HTTP status.
    #[error("E-SENSOR-001: sensor {sensor} returned HTTP {status}: {body}")]
    SensorHttpError {
        sensor: String,
        status: u16,
        body: String,
    },

    /// E-SENSOR-002: Sensor adapter timed out.
    #[error("E-SENSOR-002: sensor {sensor} timed out after {elapsed_ms}ms")]
    SensorTimeout { sensor: String, elapsed_ms: u64 },

    /// E-SENSOR-003: Sensor adapter returned malformed response.
    #[error("E-SENSOR-003: sensor {sensor} response parse error: {detail}")]
    SensorResponseParse { sensor: String, detail: String },

    /// E-SENSOR-010: Unknown sensor type.
    #[error("E-SENSOR-010: unknown sensor type: {name}")]
    UnknownSensorType { name: String },

    /// E-SENSOR-020: Sensor rate limited.
    #[error("E-SENSOR-020: sensor {sensor} rate limited; retry after {retry_after_ms}ms")]
    SensorRateLimited { sensor: String, retry_after_ms: u64 },

    // -------------------------------------------------------------------------
    // E-OCSF — OCSF normalization errors
    // -------------------------------------------------------------------------
    /// E-OCSF-001: Required OCSF field missing from source event.
    #[error("E-OCSF-001: required OCSF field missing: {field}")]
    OcsfFieldMissing { field: String },

    /// E-OCSF-002: OCSF field type mismatch.
    #[error("E-OCSF-002: OCSF field type mismatch on {field}: expected {expected}, got {got}")]
    OcsfFieldTypeMismatch {
        field: String,
        expected: String,
        got: String,
    },

    /// E-OCSF-003: Unknown OCSF class UID.
    #[error("E-OCSF-003: unknown OCSF class_uid: {class_uid}")]
    OcsfUnknownClassUid { class_uid: u32 },

    /// E-OCSF-010: OCSF protobuf encode failure.
    #[error("E-OCSF-010: protobuf encode error: {detail}")]
    OcsfProtobufEncode { detail: String },

    /// E-OCSF-011: OCSF protobuf decode failure.
    #[error("E-OCSF-011: protobuf decode error: {detail}")]
    OcsfProtobufDecode { detail: String },

    /// E-OCSF-020: No OCSF event class mapping for the given sensor + record_type pair.
    ///
    /// Emitted by `EventClassSelector::select()` when the sensor/record_type combination
    /// is not found in the compile-time mapping table. (BC-2.02.012, AC-8)
    #[error(
        "E-OCSF-020: no OCSF event class mapping for sensor={sensor}, record_type={record_type}"
    )]
    OcsfUnknownEventClass { sensor: String, record_type: String },

    /// E-OCSF-021: OCSF normalization failed — `normalize()` could not produce a valid
    /// `DynamicMessage` from the provided raw input.
    ///
    /// This is the catch-all error for BC-2.02.002 / VP-022: normalize() must return
    /// this error rather than panicking on malformed input.
    #[error("E-OCSF-021: OCSF normalization failed for source {source_id}: {reason}")]
    OcsfNormalizationFailed { source_id: String, reason: String },

    /// E-OCSF-022: The OCSF protobuf descriptor pool does not contain a descriptor for
    /// the requested `class_uid`.
    ///
    /// Returned by `OcsfNormalizer::normalize()` when `EventClassSelector::select()`
    /// resolves to a class_uid that is absent from the compiled DescriptorPool.
    /// (BC-2.02.001, AC-2)
    #[error("E-OCSF-022: OCSF descriptor not found for class_uid={class_uid}")]
    OcsfDescriptorNotFound { class_uid: u32 },

    // -------------------------------------------------------------------------
    // E-CRED — Credential management errors
    // -------------------------------------------------------------------------
    /// E-CRED-001: Credential name failed validation (S-1.02 + S-1.06).
    #[error("E-CRED-001: invalid credential name '{name}': {reason}")]
    InvalidCredentialName { name: String, reason: String },

    /// E-CRED-002: Credential not found.
    #[error("E-CRED-002: credential not found: {name}")]
    CredentialNotFound { name: String },

    /// E-CRED-003: Credential access denied (AI-opaque boundary enforced).
    #[error("E-CRED-003: credential access denied for {name} — credential values never transit AI context")]
    CredentialAccessDenied { name: String },

    /// E-CRED-004: Backend-level credential store failure (S-1.06).
    #[error("E-CRED-004: credential store error (backend={backend}): {reason}")]
    CredentialStoreError { backend: String, reason: String },

    /// E-CRED-005: Credential encryption or decryption failure (S-1.06).
    #[error("E-CRED-005: credential encryption error: {reason}")]
    CredentialEncryptionError { reason: String },

    /// E-CRED-006: Encryption passphrase not configured (S-1.06).
    #[error("E-CRED-006: encryption key not configured: {reason}")]
    EncryptionKeyMissing { reason: String },

    /// E-CRED-010: Keyring backend error.
    #[error("E-CRED-010: keyring error: {detail}")]
    KeyringError { detail: String },

    // -------------------------------------------------------------------------
    // E-IO — I/O errors
    // -------------------------------------------------------------------------
    /// E-IO-001: I/O error (S-1.06). String-ified so PrismError remains PartialEq+Eq.
    #[error("E-IO-001: I/O error: {0}")]
    Io(String),

    // -------------------------------------------------------------------------
    // E-FLAG — Feature flag / capability errors (BC-2.04.015, E-FLAG-001)
    // -------------------------------------------------------------------------
    /// E-FLAG-001 (CAPABILITY_DENIED): Write capability is denied — structured
    /// error for BC-2.04.015.  The `resolution_trace` is a BTreeMap-derived
    /// ordered list of path→effect pairs showing how the denial was reached.
    #[error(
        "CAPABILITY_DENIED: capability '{capability}' denied for client '{client_id}': {reason}"
    )]
    CapabilityDenied {
        /// The capability path that was checked (e.g., `sensor.crowdstrike.containment`).
        capability: String,
        /// The client whose effective capabilities were consulted.
        client_id: String,
        /// Human-readable denial reason.
        reason: String,
        /// Actionable guidance (exact TOML path + restart instruction or rebuild note).
        suggestion: String,
        /// Ordered list of `"path=effect"` pairs showing the resolution walk.
        /// Minimum one entry (the winning tier).
        resolution_trace: Vec<String>,
    },

    /// E-FLAG-006: Cross-client write without client_id.
    #[error(
        "E-FLAG-006: write operation requires client_id — cross-client writes are not supported"
    )]
    WriteRequiresClientId,

    /// E-FLAG-002: Feature flag disabled — write operation blocked.
    #[error("E-FLAG-002: feature flag {flag} is disabled; write operations are locked")]
    FeatureFlagDisabled { flag: String },

    /// E-FLAG-010: Feature flag evaluation error.
    #[error("E-FLAG-010: feature flag evaluation error for {flag}: {detail}")]
    FeatureFlagEvalError { flag: String, detail: String },

    // -------------------------------------------------------------------------
    // E-STORE — Storage backend errors
    // -------------------------------------------------------------------------
    /// E-STORE-001: RocksDB open failed.
    #[error("E-STORE-001: RocksDB open failed: {detail}")]
    StorageOpenFailed { detail: String },

    /// E-STORE-002: RocksDB write failed.
    #[error("E-STORE-002: RocksDB write failed on domain {domain}: {detail}")]
    StorageWriteFailed { domain: String, detail: String },

    /// E-STORE-003: RocksDB read failed.
    #[error("E-STORE-003: RocksDB read failed on domain {domain}: {detail}")]
    StorageReadFailed { domain: String, detail: String },

    /// E-STORE-004: Storage domain not found / column family missing.
    #[error("E-STORE-004: storage domain not found: {domain}")]
    StorageDomainNotFound { domain: String },

    /// E-STORE-005: Storage key not found.
    #[error("E-STORE-005: key not found in domain {domain}")]
    StorageKeyNotFound { domain: String },

    /// E-STORE-010: Storage batch write failed.
    #[error("E-STORE-010: storage batch write failed: {detail}")]
    StorageBatchFailed { detail: String },

    /// E-STORE-020: Cursor cap exceeded (S-1.02).
    /// Unit variant: CursorRegistry enforces the cap at the type boundary.
    #[error("E-STORE-020: cursor cap exceeded: cannot allocate more than 200 active cursors")]
    CursorCapExceeded,

    // -------------------------------------------------------------------------
    // E-CFG — Configuration errors
    // -------------------------------------------------------------------------
    /// E-CFG-001: Config file not found.
    #[error("E-CFG-001: config file not found: {path}")]
    ConfigNotFound { path: String },

    /// E-CFG-002: Config parse error.
    #[error("E-CFG-002: config parse error: {detail}")]
    ConfigParseFailed { detail: String },

    /// E-CFG-003: Config validation error.
    #[error("E-CFG-003: config validation failed: {detail}")]
    ConfigValidationFailed { detail: String },

    /// E-CFG-010: Config snapshot stale.
    #[error("E-CFG-010: config snapshot stale: version {current} < required {required}")]
    ConfigSnapshotStale { current: u64, required: u64 },

    /// E-CFG-020: Capability path validation failed.
    ///
    /// Returned by `CapabilityPath::new()` when the input string violates any
    /// of the format rules: empty string, empty segment, invalid characters,
    /// more than 8 segments, or total length > 256 characters.
    #[error("E-CFG-020: invalid capability path: {reason}")]
    InvalidCapabilityPath {
        /// Human-readable description of the validation failure.
        reason: String,
    },

    // -------------------------------------------------------------------------
    // E-MCP — MCP protocol errors
    // -------------------------------------------------------------------------
    /// E-MCP-001: MCP tool not found.
    #[error("E-MCP-001: MCP tool not found: {tool}")]
    McpToolNotFound { tool: String },

    /// E-MCP-002: MCP parameter validation failed.
    #[error("E-MCP-002: MCP parameter validation failed for tool {tool}: {detail}")]
    McpParameterInvalid { tool: String, detail: String },

    /// E-MCP-003: MCP response serialization error.
    #[error("E-MCP-003: MCP response serialization error: {detail}")]
    McpSerializationError { detail: String },

    /// E-MCP-010: Prompt injection detected (safety boundary).
    #[error("E-MCP-010: prompt injection detected in tool {tool}")]
    McpPromptInjectionDetected { tool: String },

    // -------------------------------------------------------------------------
    // E-SAFETY — Safety boundary violations
    // -------------------------------------------------------------------------
    /// E-SAFETY-001: AI context contamination attempt blocked.
    #[error("E-SAFETY-001: AI context contamination attempt blocked: {detail}")]
    SafetyContextContamination { detail: String },

    /// E-SAFETY-002: Sensitive data exfiltration blocked.
    #[error("E-SAFETY-002: sensitive data exfiltration blocked: {field}")]
    SafetyDataExfiltration { field: String },

    // -------------------------------------------------------------------------
    // E-QUERY — Query engine errors
    // -------------------------------------------------------------------------
    /// E-QUERY-001: Query parse error.
    #[error("E-QUERY-001: query parse error at offset {offset}: {detail}")]
    QueryParseFailed { offset: usize, detail: String },

    /// E-QUERY-002: Query planning failed.
    #[error("E-QUERY-002: query planning failed: {detail}")]
    QueryPlanFailed { detail: String },

    /// E-QUERY-003: Query execution error.
    #[error("E-QUERY-003: query execution error: {detail}")]
    QueryExecutionFailed { detail: String },

    /// E-QUERY-004: Memory budget exceeded.
    #[error("E-QUERY-004: query memory budget exceeded: limit {limit_mb}MB, used {used_mb}MB")]
    QueryMemoryBudgetExceeded { limit_mb: u64, used_mb: u64 },

    /// E-QUERY-005: Query timeout.
    #[error("E-QUERY-005: query timed out after {elapsed_ms}ms")]
    QueryTimeout { elapsed_ms: u64 },

    /// E-QUERY-010: Virtual field resolution failed.
    #[error("E-QUERY-010: virtual field resolution failed for {field}: {detail}")]
    QueryVirtualFieldFailed { field: String, detail: String },

    // -------------------------------------------------------------------------
    // E-SCHED — Scheduler errors
    // -------------------------------------------------------------------------
    /// E-SCHED-001: Schedule not found.
    #[error("E-SCHED-001: schedule not found: {id}")]
    ScheduleNotFound { id: String },

    /// E-SCHED-002: Schedule conflict — overlapping execution window.
    #[error("E-SCHED-002: schedule conflict for {id}: overlapping window with {conflicting_id}")]
    ScheduleConflict { id: String, conflicting_id: String },

    /// E-SCHED-010: Cron expression parse error.
    #[error("E-SCHED-010: invalid cron expression '{expr}': {detail}")]
    ScheduleCronInvalid { expr: String, detail: String },

    // -------------------------------------------------------------------------
    // E-DET — Detection rule errors
    // -------------------------------------------------------------------------
    /// E-DET-001: Detection rule parse error.
    #[error("E-DET-001: detection rule parse error in {rule_id}: {detail}")]
    DetectionRuleParseFailed { rule_id: String, detail: String },

    /// E-DET-002: Detection rule not found.
    #[error("E-DET-002: detection rule not found: {rule_id}")]
    DetectionRuleNotFound { rule_id: String },

    /// E-DET-010: Detection state corruption.
    #[error("E-DET-010: detection state corrupt for rule {rule_id}: {detail}")]
    DetectionStateCorrupt { rule_id: String, detail: String },

    // -------------------------------------------------------------------------
    // E-CASE — Case management errors
    // -------------------------------------------------------------------------
    /// E-CASE-001: Case not found.
    #[error("E-CASE-001: case not found: {case_id}")]
    CaseNotFound { case_id: String },

    /// E-CASE-002: Case state transition invalid.
    #[error("E-CASE-002: invalid case state transition for {case_id}: {from} -> {to}")]
    CaseStateTransitionInvalid {
        case_id: String,
        from: String,
        to: String,
    },

    // -------------------------------------------------------------------------
    // E-WATCH — Watchdog errors
    // -------------------------------------------------------------------------
    /// E-WATCH-001: Watchdog heartbeat missed.
    #[error("E-WATCH-001: watchdog heartbeat missed for {component} after {elapsed_ms}ms")]
    WatchdogHeartbeatMissed { component: String, elapsed_ms: u64 },

    /// E-WATCH-002: Watchdog restart limit exceeded.
    #[error("E-WATCH-002: watchdog restart limit exceeded for {component}: {count} restarts")]
    WatchdogRestartLimitExceeded { component: String, count: u32 },

    // -------------------------------------------------------------------------
    // E-SPEC — Spec engine errors
    // -------------------------------------------------------------------------
    /// E-SPEC structured error (BC-2.16.001, BC-2.16.002, BC-2.16.009).
    /// Carries an E-SPEC-* code, human-readable message, and optional TOML path.
    #[error("E-SPEC: {0}")]
    Spec(#[from] SpecError),

    /// E-SPEC-001: Sensor spec file not found.
    #[error("E-SPEC-001: sensor spec not found: {path}")]
    SpecNotFound { path: String },

    /// E-SPEC-002: Sensor spec validation failed.
    #[error("E-SPEC-002: sensor spec validation failed for {path}: {detail}")]
    SpecValidationFailed { path: String, detail: String },

    /// E-SPEC-010: Spec engine hot-reload failed.
    #[error("E-SPEC-010: spec hot-reload failed: {detail}")]
    SpecHotReloadFailed { detail: String },

    // -------------------------------------------------------------------------
    // E-IOC — IOC / threat intel errors
    // -------------------------------------------------------------------------
    /// E-IOC-001: IOC feed parse error.
    #[error("E-IOC-001: IOC feed parse error from {feed}: {detail}")]
    IocFeedParseFailed { feed: String, detail: String },

    /// E-IOC-002: IOC lookup failed.
    #[error("E-IOC-002: IOC lookup failed for {indicator}: {detail}")]
    IocLookupFailed { indicator: String, detail: String },

    // -------------------------------------------------------------------------
    // E-INFUSE — Infusion enrichment errors (S-1.14)
    // -------------------------------------------------------------------------
    /// Infusion enrichment error (BC-2.19.001 through BC-2.19.005).
    #[error("infusion error: {0}")]
    Infusion(#[from] InfusionError),

    // -------------------------------------------------------------------------
    // Catch-all for unexpected internal errors
    // -------------------------------------------------------------------------
    /// E-INT-001: Internal invariant violated — indicates a bug.
    #[error("E-INT-001: internal error: {detail}")]
    Internal { detail: String },
}

// ---------------------------------------------------------------------------
// E-SPEC — Spec engine structured error types (S-1.11)
// ---------------------------------------------------------------------------

/// E-SPEC-* error codes from BC-2.16.001, BC-2.16.002, BC-2.16.009.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecErrorCode {
    /// E-SPEC-001: TOML parse error or schema/variable-reference validation error.
    ESpec001,
    /// E-SPEC-004: Duplicate table_name within a sensor spec.
    ESpec004,
    /// E-SPEC-008: Custom adapter panic caught via catch_unwind.
    ESpec008,
    /// E-SPEC-009: Duplicate sensor_id across spec files.
    ESpec009,
    /// E-SPEC-010: Variable interpolation failure at runtime.
    ESpec010,
    /// E-SPEC-011: Write endpoint pipe_verb collides with reserved PrismQL keyword (BC-2.16.009, S-1.13).
    ESpec011,
}

/// A structured spec validation or runtime error carrying an E-SPEC-* code,
/// a human-readable message, and an optional TOML path for actionable correction.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("spec error {code:?} at {toml_path:?}: {message}")]
pub struct SpecError {
    pub code: SpecErrorCode,
    pub message: String,
    /// TOML path for user-actionable correction (e.g., `sensor.tables[0].steps[1].path_template`).
    pub toml_path: Option<String>,
    /// Source file path, if known.
    pub file_path: Option<String>,
    /// Line number in the source file, if known.
    pub line_number: Option<u32>,
}

// ---------------------------------------------------------------------------
// E-INFUSE — Infusion enrichment framework errors (S-1.14)
// ---------------------------------------------------------------------------

/// E-INFUSE-* error codes from BC-2.19.001 through BC-2.19.005.
///
/// These errors are produced by `InfusionRegistry` and `InfusionLoader` during
/// spec loading, hot reload, and credential resolution.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InfusionError {
    /// E-INFUSE-001: Unknown infusion name referenced in a query or pipe stage.
    #[error(
        "E-INFUSE-001: Unknown infusion '{name}'. Run list_infusions to see available enrichments."
    )]
    UnknownInfusion { name: String },

    /// E-INFUSE-002: Duplicate UDF name across multiple infusion specs.
    #[error("E-INFUSE-002: Duplicate UDF name '{udf_name}' in '{path2}' — already registered from '{path1}'.")]
    DuplicateUdfName {
        udf_name: String,
        path1: String,
        path2: String,
    },

    /// E-INFUSE-003: Missing required field in infusion spec.
    #[error("E-INFUSE-003: Missing required field '{field}' in infusion spec '{spec_path}'.")]
    MissingRequiredField { field: String, spec_path: String },

    /// E-INFUSE-004: Unknown source type in infusion spec.
    #[error("E-INFUSE-004: Unknown source type '{type_name}'. Valid types: maxmind_mmdb, csv, json_lookup, plugin.")]
    UnknownSourceType { type_name: String },

    /// E-INFUSE-005: Credential cannot be resolved.
    /// NOTE: The message MUST NOT include the credential value — only the field name,
    /// infusion_id, and env_var_name are safe to log (BC-2.19.005).
    #[error("E-INFUSE-005: Credential '{field_name}' for infusion '{infusion_id}' could not be resolved. Ensure '{env_var_name}' is set.")]
    CredentialUnresolved {
        field_name: String,
        infusion_id: String,
        env_var_name: String,
    },

    /// E-RULE-012: Detection rule filter references an API-backed infusion UDF.
    #[error("E-RULE-012: Detection rule filter references API-backed infusion UDF '{udf_name}' (from infusion '{infusion_id}', type 'plugin'). API-backed infusions cannot be used in detection rules — use a local_lookup infusion instead.")]
    ApiBackedUdfInDetectionRule {
        udf_name: String,
        infusion_id: String,
    },
}
