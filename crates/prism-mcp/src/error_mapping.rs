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

        // E-QUERY-004: Query timeout → -32001 Timeout (taxonomy v1.69 assignment;
        // pre-v1.69 the timeout condition was mislabeled E-QUERY-005, which is
        // the materialization limit)
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

        // E-QUERY-033: Limit exceeded → -32602 Invalid params (validation failure).
        // The variant Display IS the taxonomy v1.70 verbatim Message Format
        // ("E-QUERY-033: limit {requested} exceeds maximum of {max} (BC-2.11.001)")
        // mandated by the BC-2.11.001 Error Cases row — do not re-format here.
        PrismError::QueryLimitExceeded { .. } => (codes::INVALID_PARAMS, format!("{err}")),

        // E-QUERY-003: Query security limit exceeded → -32602 Invalid params
        // (error-taxonomy.md v1.72 / ADR-038 v1.3 §P5-02). Caller-resolvable:
        // narrow or simplify the query. EXPLICIT arm required: PrismError is
        // #[non_exhaustive]; letting this variant fall to the catch-all would
        // regress to opaque -32000 INTERNAL_ERROR and violate BC-2.11.006's
        // structured caller-visible limit responses.
        PrismError::QuerySecurityLimitExceeded { .. } => (codes::INVALID_PARAMS, format!("{err}")),

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

        // E-QUERY-036: Unknown source table → -32602 Invalid params (caller-resolvable)
        // MUST be explicit: #[non_exhaustive] fall-through would regress to opaque -32000.
        // Caller can fix by checking spelling or registering the sensor in prism.toml.
        // P6-02 adjudication 2026-06-11; error-taxonomy.md v1.73 E-QUERY-036.
        PrismError::UnknownSourceTable { .. } => (codes::INVALID_PARAMS, format!("{err}")),

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

        // E-CFG-100: Client not found → -32602 Invalid params (ADR-038 D4).
        // EXPLICIT arm required: PrismError is #[non_exhaustive]; letting this
        // variant fall to the catch-all would regress to opaque -32000
        // INTERNAL_ERROR and violate BC-2.10.004 et al. (caller-visible
        // structured error for an unknown client_id).
        PrismError::ClientNotFound { .. } => (codes::INVALID_PARAMS, format!("{err}")),

        // E-CFG-106: Invalid capability path → -32602 Invalid params
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

        // E-CFG-102..105: Config errors → -32000 Internal (operator-resolvable,
        // not caller-resolvable; ADR-038 D4 — arm covers only the four
        // operator-class variants after the ClientNotFound split).
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

        // E-QUERY-002/034/005/010 + E-WATCHDOG-001: Query planning/execution/
        // materialization-limit/memory errors → -32000 Internal
        PrismError::QueryPlanFailed { .. }
        | PrismError::QueryExecutionFailed { .. }
        | PrismError::QueryMaterializationLimitExceeded { .. }
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

        // E-WATCH-*/E-WATCHDOG-*: Watchdog errors → -32000 Internal
        PrismError::WatchdogHeartbeatMissed { .. }
        | PrismError::WatchdogRestartLimitExceeded { .. }
        | PrismError::WatchdogKilled { .. } => (
            codes::INTERNAL_ERROR,
            "Internal error; see audit log".to_owned(),
        ),

        // E-AUDIT-001: Audit persistence failure → -32000 Internal.
        // The variant Display IS the taxonomy-verbatim structured error
        // ("E-AUDIT-001: Audit emission failed; write operation blocked. Retry
        // the operation. ...") mandated by the BC-2.05.001 DEC-014 fail-closed
        // contract: write-classified tools abort with this structured error
        // when audit emission fails (P5-02, 2026-06-10 review pass-5). Surfaced
        // verbatim (not the generic internal-error suppression): the message
        // carries no sensitive detail and the agent caller needs the code +
        // retry suggestion to act on the transient, retryable condition.
        PrismError::AuditPersistenceFailed => (codes::INTERNAL_ERROR, format!("{err}")),

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

// ---------------------------------------------------------------------------
// S-5.02 stub API surface (BC-2.10.007 v1.5 — structured error responses)
// ---------------------------------------------------------------------------
//
// STUB ZONE — these functions expose the correct public signatures required by
// the BC-2.10.007 / BC-2.10.004 contracts so that `tests/tool_dispatch_tests.rs`
// Red Gate tests compile.  Bodies intentionally return wrong/placeholder data:
// the implementer (S-5.02 green phase) replaces these with correct logic.
// Stubs must NOT use panic-macro aliases forbidden by POL-12 (see tool_dispatch_tests.rs
// test_AC_10_no_todo_in_production_code which scans src/ for those strings).
// The preferred approach is a wrong-value stub that compiles but returns incorrect data,
// causing Red Gate test assertions to fail rather than panicking at runtime.

/// S-5.02 stub — BC-2.10.007 v1.5 wire shape.
///
/// The 9 fields required inside `structuredContent.error` plus `_meta.trust_level`.
/// Implementer populates this from real error data; stub leaves fields at placeholder
/// values so tests asserting the correct values fail (Red Gate).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructuredErrorFields {
    /// Canonical error code (e.g. `"E-MCP-001"`, `"E-CFG-100"`).
    pub code: String,
    /// Human-readable error message (never contains raw sensor text — DI-006).
    pub message: String,
    /// Error category (`"validation"`, `"configuration"`, `"internal"`, `"sensor"`).
    pub category: String,
    /// Whether the caller may retry.
    pub retryable: bool,
    /// Seconds to wait before retry; `null` when not applicable (BC-2.10.007 null-not-absent).
    pub retry_after_seconds: Option<u64>,
    /// Actionable suggestion for the caller.
    pub suggestion: String,
    /// Error source identifier (e.g. `"prism_mcp"`).
    pub source: String,
    /// Whether the original request parameters were structurally valid.
    pub original_params_valid: bool,
    /// Raw upstream sensor message; `null` for errors originating in Prism (DI-006).
    pub upstream_message: Option<String>,
}

/// Build the nested BC-2.10.007 `structuredContent.error` envelope as a `CallToolResult`.
///
/// **S-5.02 stub** — returns a placeholder `CallToolResult` with an EMPTY `structuredContent`
/// object (no `error` key, no `_meta` key).  Tests asserting the full 9-field shape and
/// `_meta.trust_level: "internal"` will FAIL → Red Gate holds.
///
/// The implementer replaces this with the full envelope construction.
///
/// # Parameters
/// - `fields`: the 9 structured error fields per BC-2.10.007 v1.5
/// - `content_text`: the human-readable `content[].text` string ("`ERROR: [{category}] - ...`")
pub fn build_structured_error_response(
    _fields: StructuredErrorFields,
    _content_text: String,
) -> rmcp::model::CallToolResult {
    // STUB: returns a structured_error CallToolResult with a placeholder structuredContent
    // that lacks the `error` key and `_meta` key required by BC-2.10.007 v1.5.
    // Tests asserting `structuredContent.error.*` and `_meta.trust_level` will FAIL.
    rmcp::model::CallToolResult::structured_error(
        serde_json::json!({"_stub": "S-5.02 not implemented"}),
    )
}

/// Like `to_error_data` but also extracts `retry_after_ms` from `SensorRateLimited`.
///
/// **S-5.02 stub** — always returns `(ErrorData, None)`.  Tests asserting
/// `retry_after_seconds: 30` will fail (they get `None`) → Red Gate holds.
///
/// The implementer splits `SensorRateLimited { retry_after_ms }` out of the multi-arm
/// sensor group and threads the value through.
pub fn to_error_data_with_retry(err: PrismError) -> (ErrorData, Option<u64>) {
    // STUB: drops retry_after_ms; always returns None.
    let (code, message) = map_prism_error(err);
    (ErrorData::new(ErrorCode(code), message, None), None)
}

// ---------------------------------------------------------------------------
// Unit tests for error_mapping
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::PrismError;

    /// P6-02: UnknownSourceTable (E-QUERY-036) must map to -32602 INVALID_PARAMS.
    ///
    /// EXPLICIT arm required: `PrismError` is `#[non_exhaustive]`; without the
    /// explicit arm the variant would fall through to the catch-all `-32000`
    /// INTERNAL_ERROR, losing the caller-actionable E-QUERY-036 guidance.
    #[test]
    fn test_unknown_source_table_maps_to_invalid_params() {
        let err = PrismError::UnknownSourceTable {
            source_name: "ghost_sensor.table".to_string(),
        };
        let (code, message) = map_prism_error(err);
        assert_eq!(
            code,
            codes::INVALID_PARAMS,
            "UnknownSourceTable must map to INVALID_PARAMS (-32602), got: {code}"
        );
        assert!(
            message.contains("E-QUERY-036"),
            "message must contain 'E-QUERY-036'; got: {message}"
        );
        assert!(
            message.contains("ghost_sensor.table"),
            "message must include the source_name; got: {message}"
        );
    }

    /// UnknownSourceTable must NOT fall through to the catch-all -32000 arm.
    ///
    /// This test is distinct from the code-value test above: it explicitly confirms
    /// the error is NOT -32000, providing a mutation-resistant assertion that the
    /// explicit arm is load-bearing (not just incidentally green via fall-through).
    #[test]
    fn test_unknown_source_table_does_not_map_to_internal_error() {
        let err = PrismError::UnknownSourceTable {
            source_name: "unknown.devices".to_string(),
        };
        let (code, _) = map_prism_error(err);
        assert_ne!(
            code,
            codes::INTERNAL_ERROR,
            "UnknownSourceTable must NOT map to INTERNAL_ERROR (-32000); got: {code}"
        );
    }
}
