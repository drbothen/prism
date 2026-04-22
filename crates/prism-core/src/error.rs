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
#[derive(Debug, Error)]
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
    SensorRateLimited {
        sensor: String,
        retry_after_ms: u64,
    },

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

    /// E-OCSF-023: Sensor record_type not in the mapper's supported set.
    ///
    /// Returned by `ClarotyMapper` and `ArmisMapper` when the record_type is not one
    /// of their declared supported types. (BC-2.02.005, BC-2.02.006, S-1.05 Edge Cases)
    #[error("E-OCSF-023: unknown record type for sensor={sensor}: record_type={record_type}")]
    OcsfUnknownRecordType { sensor: String, record_type: String },

    /// E-OCSF-024: Timestamp field could not be parsed using any supported format.
    ///
    /// Returned by `CyberintMapper` when `created_date` fails all four parse attempts.
    /// (BC-2.02.004, AC-4, S-1.05 Edge Cases)
    #[error("E-OCSF-024: timestamp parse failed for field={field}: raw value={raw}")]
    OcsfTimestampParseError { field: String, raw: String },


    // -------------------------------------------------------------------------
    // E-CRED — Credential management errors
    // -------------------------------------------------------------------------
    /// E-CRED-001: Credential name failed validation.
    #[error("E-CRED-001: invalid credential name: {name}")]
    InvalidCredentialName { name: String },

    /// E-CRED-002: Credential not found.
    #[error("E-CRED-002: credential not found: {name}")]
    CredentialNotFound { name: String },

    /// E-CRED-003: Credential access denied (AI-opaque boundary enforced).
    #[error("E-CRED-003: credential access denied for {name} — credential values never transit AI context")]
    CredentialAccessDenied { name: String },

    /// E-CRED-010: Keyring backend error.
    #[error("E-CRED-010: keyring error: {detail}")]
    KeyringError { detail: String },

    // -------------------------------------------------------------------------
    // E-FLAG — Feature flag errors
    // -------------------------------------------------------------------------
    /// E-FLAG-001: Feature flag not found.
    #[error("E-FLAG-001: feature flag not found: {flag}")]
    FeatureFlagNotFound { flag: String },

    /// E-FLAG-002: Feature flag disabled — write operation blocked.
    #[error("E-FLAG-002: feature flag {flag} is disabled; write operations are locked")]
    FeatureFlagDisabled { flag: String },

    /// E-FLAG-010: Feature flag evaluation error.
    #[error("E-FLAG-010: feature flag evaluation error for {flag}: {detail}")]
    FeatureFlagEvalError { flag: String, detail: String },

    // -------------------------------------------------------------------------
    // E-FLAG-003..008 — Confirmation token errors (S-1.09, BC-2.04.009..012)
    // -------------------------------------------------------------------------
    /// E-FLAG-003: Confirmation token expired (BC-2.04.011).
    ///
    /// Returned when `confirm_action` is called with a token whose `expires_at`
    /// is in the past (`now >= expires_at`). The `action_summary` from the
    /// original token is included so the agent can re-request intelligently.
    #[error(
        "E-FLAG-003: confirmation token expired for action '{action_summary}'; \
         call the original write tool to generate a new token"
    )]
    TokenExpired {
        /// The `action_summary` from the expired token.
        action_summary: String,
        /// `retryable: false` — agent must call the original write tool again.
        retryable: bool,
    },

    /// E-FLAG-004: Confirmation token already consumed (BC-2.04.010; VP-008).
    #[error(
        "E-FLAG-004: confirmation token '{token_id}' already consumed; \
         call the original write tool to generate a new token if needed"
    )]
    TokenAlreadyConsumed { token_id: String, retryable: bool },

    /// E-FLAG-005: Confirmation token content hash mismatch (BC-2.04.012; VP-009).
    ///
    /// The action parameters supplied to `confirm_action` do not match the
    /// SHA-256 hash stored in the token — tampering or substitution detected.
    #[error(
        "E-FLAG-005: confirmation token '{token_id}' content hash mismatch; \
         request a new token for the intended action"
    )]
    TokenContentHashMismatch { token_id: String, retryable: bool },

    /// E-FLAG-007: Confirmation token store capacity exceeded (BC-2.04.009; VP-010).
    ///
    /// The store holds 100 active tokens. After sweeping expired tokens the cap
    /// is still reached. No eviction occurs — the caller must wait.
    #[error(
        "E-FLAG-007: token store capacity reached (100 active tokens); \
         wait for existing tokens to expire or confirm/cancel pending actions"
    )]
    TokenCapExceeded,

    /// E-FLAG-008: Confirmation token not found in store (BC-2.04.010).
    #[error(
        "E-FLAG-008: confirmation token not found: '{token_id}'; \
         it may have expired and been cleaned up"
    )]
    TokenNotFound { token_id: String },

    /// E-MCP-004: client_id mismatch on confirm_action (BC-2.04.010).
    ///
    /// The `client_id` passed to `confirm_action` does not match the
    /// `client_id` embedded in the token at generation time.
    #[error(
        "E-MCP-004: client_id mismatch on confirm_action for token '{token_id}'; \
         use the same client_id that was used when the token was generated"
    )]
    ConfirmClientIdMismatch { token_id: String, retryable: bool },

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

    /// E-STORE-020: Cursor cap exceeded.
    #[error("E-STORE-020: cursor cap exceeded: max {max} rows, got {count}")]
    CursorCapExceeded { max: u64, count: u64 },

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
    ScheduleConflict {
        id: String,
        conflicting_id: String,
    },

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
    WatchdogHeartbeatMissed {
        component: String,
        elapsed_ms: u64,
    },

    /// E-WATCH-002: Watchdog restart limit exceeded.
    #[error("E-WATCH-002: watchdog restart limit exceeded for {component}: {count} restarts")]
    WatchdogRestartLimitExceeded { component: String, count: u32 },

    // -------------------------------------------------------------------------
    // E-SPEC — Spec engine errors
    // -------------------------------------------------------------------------
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
    // Catch-all for unexpected internal errors
    // -------------------------------------------------------------------------
    /// E-INT-001: Internal invariant violated — indicates a bug.
    #[error("E-INT-001: internal error: {detail}")]
    Internal { detail: String },
}
