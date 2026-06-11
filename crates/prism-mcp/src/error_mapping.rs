//! PrismError → MCP error code mapping (ADR-022 §F).
//!
//! Every `PrismError` variant maps to a structured MCP error code per the
//! error-taxonomy.md table. This module is pure (BC-2.10.007 purity classification):
//! deterministic variant → code mapping, no I/O.
//!
//! MCP error codes used:
//! - `-32602` Invalid params  — parse errors, missing required fields, validation failures
//! - `-32003` NotImplemented  — write not supported for sensor, prism-operations not merged
//! - `-32002` Forbidden       — feature flag denied, permission denied, injection detected
//! - `-32001` Timeout         — query execution timeout
//! - `-32000` Internal error  — all other PrismError variants (audit log has detail)

use prism_core::error::PrismError;
use rmcp::model::{ErrorCode, ErrorData};

/// Map a `PrismError` to an MCP-compatible error representation.
///
/// Returns `(code, message)` directly (test-assertable shape). Production callers
/// needing `rmcp::ErrorData` use the `to_error_data` wrapper below; this two-function
/// split exists so unit tests can assert error codes without instantiating rmcp types.
///
/// Per ADR-022 §F error mapping table.
pub fn map_prism_error(err: PrismError) -> (i32, String) {
    match &err {
        // E-QUERY-001: Query parse error → -32602 Invalid params
        // Message format: "PrismQL parse error: {detail}" per AC-5.
        PrismError::QueryParseFailed { detail, .. } => (
            codes::INVALID_PARAMS,
            format!("PrismQL parse error: {detail}"),
        ),

        // E-MCP-002: MCP parameter validation failed → -32602 Invalid params
        // Message includes tool name and field detail per AC-4.
        PrismError::McpParameterInvalid { tool, detail } => (
            codes::INVALID_PARAMS,
            format!("Invalid parameter for tool '{tool}': {detail}"),
        ),

        // E-QUERY-005: Query timeout → -32001 Timeout
        PrismError::QueryTimeout { .. } => (codes::TIMEOUT, "Query timeout exceeded".to_owned()),

        // E-FLAG-001 (runtime tier) / E-FLAG-002 (compile tier): capability
        // denied → -32002 Forbidden. Both tiers surface as CapabilityDenied
        // (BC-2.04.015; P2-03 2026-06-10 review pass-2 — the spec-unbacked
        // FeatureFlagDisabled variant was removed from PrismError).
        // Display includes full context per BC-2.10.007.
        PrismError::CapabilityDenied { .. } => (codes::FORBIDDEN, format!("{err}")),

        // E-FLAG-010: Feature flag eval error → -32002 Forbidden
        PrismError::FeatureFlagEvalError { flag, detail } => (
            codes::FORBIDDEN,
            format!("Feature flag evaluation error for '{flag}': {detail}"),
        ),

        // E-AUTH-020: Unauthorized → -32002 Forbidden
        PrismError::Unauthorized { action } => {
            (codes::FORBIDDEN, format!("Unauthorized: {action}"))
        }

        // E-MCP-010: Prompt injection detected → -32002 Forbidden
        PrismError::McpPromptInjectionDetected { tool } => (
            codes::FORBIDDEN,
            format!("Input rejected: prompt injection detected in tool '{tool}'"),
        ),

        // E-FLAG-003..008: Confirmation token errors → -32002 Forbidden (token lifecycle)
        PrismError::TokenExpired { .. }
        | PrismError::TokenAlreadyConsumed { .. }
        | PrismError::TokenContentHashMismatch { .. }
        | PrismError::TokenCapExceeded
        | PrismError::TokenNotFound { .. }
        | PrismError::ConfirmClientIdMismatch { .. } => (codes::FORBIDDEN, format!("{err}")),

        // E-FLAG-006: Write requires client_id → -32002 Forbidden
        PrismError::WriteRequiresClientId => (codes::FORBIDDEN, format!("{err}")),

        // E-CRED-003: Credential access denied → -32002 Forbidden (safety boundary)
        PrismError::CredentialAccessDenied { .. } => (codes::FORBIDDEN, format!("{err}")),

        // E-QUERY-011: Audit table access denied → -32002 Forbidden
        PrismError::AuditTableAccessDenied => (codes::FORBIDDEN, format!("{err}")),

        // E-MCP-001: Tool not found → -32602 InvalidParams. MCP convention treats tool-not-found
        // as a caller-supplied invalid 'name' parameter rather than JSON-RPC -32601 'method not
        // found' (which would imply protocol-level method, not tool-name).
        PrismError::McpToolNotFound { tool } => {
            (codes::INVALID_PARAMS, format!("MCP tool not found: {tool}"))
        }

        // E-QUERY-033: Limit exceeded → -32602 Invalid params (validation failure)
        PrismError::QueryLimitExceeded { requested, max } => (
            codes::INVALID_PARAMS,
            format!("Invalid parameter: limit {requested} exceeds maximum of {max}"),
        ),

        // E-QUERY-022: Unbounded write → -32602 Invalid params
        PrismError::WriteUnbounded => (codes::INVALID_PARAMS, format!("{err}")),

        // E-QUERY-020..030: Write errors that are configuration/validation issues → -32602
        PrismError::WriteTargetCompositeSource { .. }
        | PrismError::WriteBatchLimitExceeded { .. }
        | PrismError::WriteTargetingInternalTable { .. }
        | PrismError::WriteVerbNotAvailable { .. }
        | PrismError::WriteTargetTableUnknown { .. }
        | PrismError::WriteAdapterNotConfiguredForClient { .. } => {
            (codes::INVALID_PARAMS, format!("{err}"))
        }

        // E-QUERY-032: Sensor not registered for org → -32602 Invalid params.
        // SURFACED (NOT redacted): the org slug and sensor name are safe to expose to
        // the MCP caller — they contain no credential values (AD-017). This is an
        // operational configuration error, not a sensor infrastructure failure.
        // Distinct from E-SENSOR-* variants which ARE redacted per AD-017.
        // Reference: ADR-007 §2.2 + BC-3.2.001 postcondition 5.
        PrismError::SensorNotRegisteredForOrg {
            sensor_id,
            org_slug,
        } => (
            codes::INVALID_PARAMS,
            format!("E-QUERY-032: Sensor '{sensor_id}' is not registered for org '{org_slug}'"),
        ),

        // E-ALIAS-* errors → -32602 Invalid params (alias name/cycle/depth validation)
        PrismError::AliasNotFound { .. }
        | PrismError::AliasCycleDetected { .. }
        | PrismError::AliasDepthExceeded { .. }
        | PrismError::AliasParameterInvalid { .. }
        | PrismError::AliasDependentsExist { .. }
        | PrismError::AliasNameConflict { .. } => (codes::INVALID_PARAMS, format!("{err}")),

        // E-ALIAS-QUERY cursor errors → -32602 Invalid params
        PrismError::CursorExpired
        | PrismError::CursorPageSizeInvalid
        | PrismError::CursorTokenUnknown
        | PrismError::CursorCapExceeded => (codes::INVALID_PARAMS, format!("{err}")),

        // E-CFG-020: Invalid capability path → -32602 Invalid params
        PrismError::InvalidCapabilityPath { .. } => (codes::INVALID_PARAMS, format!("{err}")),

        // E-AUTH-001..003: Identity validation failures → -32602 Invalid params
        PrismError::InvalidOrgSlug { .. }
        | PrismError::InvalidAnalystId { .. }
        | PrismError::InvalidClientId { .. } => (codes::INVALID_PARAMS, format!("{err}")),

        // E-AUTH-010..011: Auth token invalid/expired → -32000 Internal
        // (authentication infrastructure failures, not caller-param issues)
        PrismError::AuthTokenExpired | PrismError::AuthTokenInvalid { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-SPEC-*: Spec engine errors → -32000 Internal
        // (configuration issues that the API caller cannot resolve)
        PrismError::Spec(_)
        | PrismError::SpecNotFound { .. }
        | PrismError::SpecValidationFailed { .. }
        | PrismError::SpecHotReloadFailed { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-CFG-*: Config errors → -32000 Internal
        PrismError::ConfigNotFound { .. }
        | PrismError::ConfigParseFailed { .. }
        | PrismError::ConfigValidationFailed { .. }
        | PrismError::ConfigSnapshotStale { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-STORE-*: Storage errors → -32000 Internal
        PrismError::StorageOpenFailed { .. }
        | PrismError::StorageWriteFailed { .. }
        | PrismError::StorageReadFailed { .. }
        | PrismError::StorageDomainNotFound { .. }
        | PrismError::StorageKeyNotFound { .. }
        | PrismError::StorageLockHeld { .. }
        | PrismError::StorageHealthCheckFailed { .. }
        | PrismError::SchemaMismatch { .. }
        | PrismError::StorageBatchFailed { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-SENSOR-*: Sensor adapter errors → -32000 Internal
        // (external service failures; detail in audit log)
        PrismError::SensorHttpError { .. }
        | PrismError::SensorTimeout { .. }
        | PrismError::SensorResponseParse { .. }
        | PrismError::SensorRateLimited { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-OCSF-*: OCSF normalization errors → -32000 Internal
        PrismError::OcsfFieldMissing { .. }
        | PrismError::OcsfFieldTypeMismatch { .. }
        | PrismError::OcsfUnknownClassUid { .. }
        | PrismError::OcsfProtobufEncode { .. }
        | PrismError::OcsfProtobufDecode { .. }
        | PrismError::OcsfUnknownEventClass { .. }
        | PrismError::OcsfNormalizationFailed { .. }
        | PrismError::OcsfDescriptorNotFound { .. }
        | PrismError::OcsfUnknownRecordType { .. }
        | PrismError::OcsfTimestampParseError { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-CRED-*: Credential errors → -32000 Internal
        // (NEVER leak credential details in MCP responses)
        PrismError::InvalidCredentialName { .. }
        | PrismError::CredentialNotFound { .. }
        | PrismError::CredentialStoreError { .. }
        | PrismError::CredentialEncryptionError { .. }
        | PrismError::EncryptionKeyMissing { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-IO-001: I/O error → -32000 Internal
        PrismError::Io(_) => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-MCP-003: MCP serialization error → -32000 Internal
        PrismError::McpSerializationError { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-SAFETY-*: Safety boundary violations → -32000 Internal
        // (safety violations are logged; do not surface detail to caller)
        PrismError::SafetyContextContamination { .. }
        | PrismError::SafetyDataExfiltration { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-QUERY-002..004: Query planning/execution/memory errors → -32000 Internal
        PrismError::QueryPlanFailed { .. }
        | PrismError::QueryExecutionFailed { .. }
        | PrismError::QueryMemoryBudgetExceeded { .. }
        | PrismError::QueryVirtualFieldFailed { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-QUERY-008: Query denylisted → -32000 Internal
        PrismError::QueryDenylisted { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-QUERY-025: Write partial failure → -32000 Internal
        PrismError::WritePartialFailure { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-SCHED-*: Scheduler errors → -32000 Internal
        PrismError::ScheduleNotFound { .. }
        | PrismError::ScheduleConflict { .. }
        | PrismError::ScheduleCronInvalid { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-DET-*: Detection errors → -32000 Internal
        PrismError::DetectionRuleParseFailed { .. }
        | PrismError::DetectionRuleNotFound { .. }
        | PrismError::DetectionStateCorrupt { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-CASE-*: Case management errors → -32000 Internal
        PrismError::CaseNotFound { .. } | PrismError::CaseStateTransitionInvalid { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-WATCH-*: Watchdog errors → -32000 Internal
        PrismError::WatchdogHeartbeatMissed { .. }
        | PrismError::WatchdogRestartLimitExceeded { .. }
        | PrismError::WatchdogKilled { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-AUDIT-001: Audit persistence failure → -32000 Internal
        PrismError::AuditPersistenceFailed => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-INFUSE-*: Infusion errors → -32000 Internal
        PrismError::Infusion(_) => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-PLUGIN-*: WASM plugin errors → -32000 Internal
        PrismError::Plugin(_) => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-IOC-*: IOC errors → -32000 Internal
        PrismError::IocFeedParseFailed { .. } | PrismError::IocLookupFailed { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-INT-001: Internal invariant violated → -32000 Internal
        // Detail is suppressed — audit log has it.
        PrismError::Internal { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // Catch-all for future PrismError variants added after this match
        // was written (non_exhaustive enum). Defaults to -32000 Internal.
        _ => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),
    }
}

/// MCP error code constants per ADR-022 §F.
pub mod codes {
    /// Invalid parameters — parse errors, missing required fields.
    pub const INVALID_PARAMS: i32 = -32602;
    /// Feature not implemented — write not supported, prism-operations not merged.
    pub const NOT_IMPLEMENTED: i32 = -32003;
    /// Forbidden — feature flag denied, permission denied, injection detected.
    pub const FORBIDDEN: i32 = -32002;
    /// Timeout — query execution exceeded configured limit.
    pub const TIMEOUT: i32 = -32001;
    /// Internal error — all other variants; audit log has detail.
    pub const INTERNAL_ERROR: i32 = -32000;
}

/// Map a `PrismError` to an rmcp `ErrorData` for protocol-level MCP error responses.
///
/// This function converts a `PrismError` into an `rmcp::ErrorData` suitable for
/// returning from `ServerHandler` methods that return `Result<T, ErrorData>`.
///
/// The code mapping is identical to `map_prism_error` — this function exists as
/// the rmcp-native companion so that callers don't need to construct `ErrorData`
/// manually from the `(i32, String)` tuple.
///
/// # Note on test compatibility
///
/// `map_prism_error` returns `(i32, String)` for test assertability without the
/// rmcp dependency in test scope. This function is the production-path variant
/// that returns rmcp types. Both use the same code table (ADR-022 §F).
pub fn to_error_data(err: PrismError) -> ErrorData {
    let (code, message) = map_prism_error(err);
    ErrorData::new(ErrorCode(code), message, None)
}
