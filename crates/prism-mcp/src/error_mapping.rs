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

        // E-SENSOR-020: Sensor rate limited — EXPLICIT arm required.
        // BC-2.10.007 §115-116: bind both fields; sensor→source (used in
        // prism_error_to_structured_call_result), retry_after_ms/1000→retry_after_seconds.
        // Kept separate so `sensor` is bound for the structured error caller to use.
        PrismError::SensorRateLimited {
            sensor,
            retry_after_ms,
        } => (
            codes::INTERNAL_ERROR,
            format!("E-SENSOR-020: sensor '{sensor}' rate limited; retry after {retry_after_ms}ms"),
        ),

        // E-SENSOR-001..003: Other sensor adapter errors → -32000 Internal
        // (external service failures; detail in audit log)
        PrismError::SensorHttpError { .. }
        | PrismError::SensorTimeout { .. }
        | PrismError::SensorResponseParse { .. } => (
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
// BC-2.10.007 v1.5 — structured error envelope API
// ---------------------------------------------------------------------------

/// BC-2.10.007 v1.5 wire shape — 9 fields inside `structuredContent.error`.
///
/// Carries the structured error envelope that every user-visible MCP tool error response
/// must include (BC-2.10.007 postcondition). The builder [`build_structured_error_response`]
/// serialises these fields into `structuredContent.error` with an explicit-null
/// invariant for `retry_after_seconds` and `upstream_message` (null-not-absent).
///
/// # Construction
///
/// `#[non_exhaustive]` prevents external struct-literal construction, which would
/// need updating every time a field is added. External callers (including tests)
/// MUST use [`StructuredErrorFields::new`]:
///
/// ```
/// use prism_mcp::error_mapping::StructuredErrorFields;
/// let fields = StructuredErrorFields::new(
///     "E-MCP-001", "invalid client_id format: ''", "validation",
///     false, None, "Provide a client_id matching [a-zA-Z0-9_-]{1,64}.", "prism_mcp",
///     false, None,
/// );
/// ```
#[non_exhaustive]
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

impl StructuredErrorFields {
    /// Construct all 9 BC-2.10.007 v1.5 structured error fields.
    ///
    /// External callers MUST use this constructor — struct literal syntax is blocked by
    /// `#[non_exhaustive]` (HC-3, S-5.02).
    ///
    /// # Arguments (positional, matching field order)
    /// 1. `code` — canonical E-* error code (e.g. `"E-MCP-001"`)
    /// 2. `message` — human-readable message (no raw sensor text, DI-006)
    /// 3. `category` — error class: `"validation"`, `"authorization"`, `"timeout"`, `"sensor"`, `"configuration"`, `"internal"`
    /// 4. `retryable` — whether the caller may retry
    /// 5. `retry_after_seconds` — wait hint (null when not applicable)
    /// 6. `suggestion` — actionable suggestion for the caller
    /// 7. `source` — error source identifier (e.g. `"prism_mcp"`)
    /// 8. `original_params_valid` — whether the original request params were structurally valid
    /// 9. `upstream_message` — raw upstream sensor text (null for Prism-originating errors, DI-006)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        category: impl Into<String>,
        retryable: bool,
        retry_after_seconds: Option<u64>,
        suggestion: impl Into<String>,
        source: impl Into<String>,
        original_params_valid: bool,
        upstream_message: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            category: category.into(),
            retryable,
            retry_after_seconds,
            suggestion: suggestion.into(),
            source: source.into(),
            original_params_valid,
            upstream_message,
        }
    }
}

/// Build the nested BC-2.10.007 `structuredContent.error` envelope as an error `CallToolResult`.
///
/// This is the PRODUCTION tool error boundary: tool handlers return
/// `Ok(build_structured_error_response(...))` for all user-visible domain errors so that
/// `structuredContent.error` carries the 9-field schema and `_meta.trust_level:"internal"`.
///
/// Protocol-level errors (injection rejection, write-tool audit fail-closed, rmcp framework
/// errors) remain as `Err(ErrorData)` — those are returned before the tool handler body
/// executes and are not user-visible at the domain level.
///
/// Produces the BC-2.10.007 v1.5 wire shape:
/// ```json
/// {
///   "isError": true,
///   "content": [{"type": "text", "text": "<content_text>"}],
///   "structuredContent": {
///     "error": {
///       "code": "...", "message": "...", "category": "...",
///       "retryable": false, "retry_after_seconds": null,
///       "suggestion": "...", "source": "...",
///       "original_params_valid": false, "upstream_message": null
///     },
///     "_meta": {"trust_level": "internal"}
///   }
/// }
/// ```
///
/// `retry_after_seconds` and `upstream_message` are always present in the JSON
/// as explicit `null` when not applicable (null-not-absent invariant, BC-2.10.007 v1.5).
///
/// # Parameters
/// - `fields`: the 9 structured error fields per BC-2.10.007 v1.5
/// - `content_text`: the human-readable `content[].text` string ("`ERROR: [{category}] - ...`")
pub fn build_structured_error_response(
    fields: StructuredErrorFields,
    content_text: String,
) -> rmcp::model::CallToolResult {
    // Build retry_after_seconds and upstream_message as explicit JSON null when None.
    // serde_json::Value::Null ensures the field is PRESENT in the object as `null`,
    // not absent (BC-2.10.007 null-not-absent invariant).
    let retry_after_seconds = match fields.retry_after_seconds {
        Some(s) => serde_json::Value::Number(s.into()),
        None => serde_json::Value::Null,
    };
    let upstream_message = match fields.upstream_message {
        Some(msg) => serde_json::Value::String(msg),
        None => serde_json::Value::Null,
    };

    let structured_content = serde_json::json!({
        "error": {
            "code": fields.code,
            "message": fields.message,
            "category": fields.category,
            "retryable": fields.retryable,
            "retry_after_seconds": retry_after_seconds,
            "suggestion": fields.suggestion,
            "source": fields.source,
            "original_params_valid": fields.original_params_valid,
            "upstream_message": upstream_message,
        },
        "_meta": {
            "trust_level": "internal"
        }
    });

    // CallToolResult is #[non_exhaustive] — cannot use struct literal from external crate.
    // Use structured_error() as the constructor, then replace the content vector with
    // the human-readable content_text instead of the JSON-serialized structured content.
    let mut result = rmcp::model::CallToolResult::structured_error(structured_content);
    result.content = vec![rmcp::model::Content::text(content_text)];
    result
}

/// Like `to_error_data` but also extracts `retry_after_ms` from `SensorRateLimited`.
///
/// For `PrismError::SensorRateLimited { retry_after_ms, .. }`, returns
/// `Some(retry_after_ms)` as the second tuple element so the caller can convert
/// to `retry_after_seconds` (ms / 1000) for the BC-2.10.007 structured error envelope.
///
/// For all other `PrismError` variants, returns `None`.
pub fn to_error_data_with_retry(err: PrismError) -> (ErrorData, Option<u64>) {
    // Extract retry_after_ms BEFORE consuming err via map_prism_error.
    let retry_after_ms = match &err {
        PrismError::SensorRateLimited { retry_after_ms, .. } => Some(*retry_after_ms),
        _ => None,
    };
    let (code, message) = map_prism_error(err);
    (
        ErrorData::new(ErrorCode(code), message, None),
        retry_after_ms,
    )
}

/// Convert a `PrismError` into a BC-2.10.007 structured `CallToolResult` (is_error=true).
///
/// This is the PRODUCTION domain-error boundary for tool handlers. All user-visible
/// domain errors (QueryEngine failures, WriteExecutor rejections, validation failures)
/// route through here so the caller receives `structuredContent.error` with all
/// 9 required fields + `_meta.trust_level:"internal"`.
///
/// Protocol-level errors (injection rejection, write-tool fail-closed audit) remain as
/// `Err(ErrorData)` because they fire before or around the tool handler body.
///
/// # Usage in tool handlers
///
/// ```ignore
/// // BEFORE (flat error — violates BC-2.10.007):
/// return Err(to_error_data(PrismError::QueryTimeout { elapsed_ms: 30_000 }));
///
/// // AFTER (structured error — BC-2.10.007 compliant):
/// return Ok(prism_error_to_structured_call_result(PrismError::QueryTimeout { elapsed_ms: 30_000 }));
/// ```
pub fn prism_error_to_structured_call_result(err: PrismError) -> rmcp::model::CallToolResult {
    // Inspect err by reference BEFORE consuming it with map_prism_error.
    // Temporary struct to capture variant-level metadata.
    //
    // BC-2.10.007 §category legal enum:
    //   transient | authentication | validation | not_found | permission |
    //   upstream_error | configuration | safety
    // BC-2.10.007 §81 source values:
    //   "prism_mcp" for MCP-layer errors; sensor API name for sensor errors;
    //   "prism_config" for configuration errors.
    // BC-2.10.007 DI-006 / EC-10-013: raw sensor text goes in upstream_message ONLY.
    struct VariantMeta {
        category: &'static str,
        suggestion: &'static str,
        retryable: bool,
        retry_after_seconds: Option<u64>,
        original_params_valid: bool,
        /// Override source for sensor errors; `None` → default "prism_mcp".
        source_override: Option<String>,
        /// Raw upstream sensor text for DI-006 isolation; `None` for Prism-originating errors.
        upstream_message: Option<String>,
    }
    let meta = match &err {
        // ── Validation errors: caller-supplied bad parameters ────────────────
        // ClientNotFound is intentionally EXCLUDED from this group per BC-2.10.004 §87:
        // a well-formed-but-unregistered client_id is a configuration error, not a
        // bad-parameter error — `original_params_valid: true`.
        PrismError::QueryParseFailed { .. }
        | PrismError::McpParameterInvalid { .. }
        | PrismError::McpToolNotFound { .. }
        | PrismError::InvalidCapabilityPath { .. }
        | PrismError::InvalidOrgSlug { .. }
        | PrismError::InvalidAnalystId { .. }
        | PrismError::InvalidClientId { .. }
        | PrismError::QueryLimitExceeded { .. }
        | PrismError::QuerySecurityLimitExceeded { .. }
        | PrismError::WriteUnbounded
        | PrismError::WriteTargetCompositeSource { .. }
        | PrismError::WriteBatchLimitExceeded { .. }
        | PrismError::WriteTargetingInternalTable { .. }
        | PrismError::WriteVerbNotAvailable { .. }
        | PrismError::WriteTargetTableUnknown { .. }
        | PrismError::WriteAdapterNotConfiguredForClient { .. }
        | PrismError::UnknownSourceTable { .. }
        | PrismError::SensorNotRegisteredForOrg { .. }
        | PrismError::AliasNotFound { .. }
        | PrismError::AliasCycleDetected { .. }
        | PrismError::AliasDepthExceeded { .. }
        | PrismError::AliasParameterInvalid { .. }
        | PrismError::AliasDependentsExist { .. }
        | PrismError::AliasNameConflict { .. }
        | PrismError::CursorExpired
        | PrismError::CursorPageSizeInvalid
        | PrismError::CursorTokenUnknown
        | PrismError::CursorCapExceeded => VariantMeta {
            category: "validation",
            suggestion: "Check the request parameters and retry.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: false,
            source_override: None,
            upstream_message: None,
        },

        // ── Configuration errors: well-formed params but not in config ───────
        // BC-2.10.004 §87 case (c): ClientNotFound → category "configuration",
        // original_params_valid: true (params were structurally valid).
        // BC-2.10.007 §81 source: "prism_config" for configuration errors.
        PrismError::ClientNotFound { .. }
        | PrismError::Spec(_)
        | PrismError::SpecNotFound { .. }
        | PrismError::SpecValidationFailed { .. }
        | PrismError::SpecHotReloadFailed { .. }
        | PrismError::ConfigNotFound { .. }
        | PrismError::ConfigParseFailed { .. }
        | PrismError::ConfigValidationFailed { .. }
        | PrismError::ConfigSnapshotStale { .. } => VariantMeta {
            category: "configuration",
            suggestion: "Check operator configuration; see audit log for details.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: Some("prism_config".to_owned()),
            upstream_message: None,
        },

        // ── Permission errors: capability denied, auth failures ──────────────
        // BC-2.10.007 legal category: "permission" (not "authorization").
        PrismError::CapabilityDenied { .. }
        | PrismError::FeatureFlagEvalError { .. }
        | PrismError::Unauthorized { .. }
        | PrismError::McpPromptInjectionDetected { .. }
        | PrismError::TokenExpired { .. }
        | PrismError::TokenAlreadyConsumed { .. }
        | PrismError::TokenContentHashMismatch { .. }
        | PrismError::TokenCapExceeded
        | PrismError::TokenNotFound { .. }
        | PrismError::ConfirmClientIdMismatch { .. }
        | PrismError::WriteRequiresClientId
        | PrismError::CredentialAccessDenied { .. }
        | PrismError::AuditTableAccessDenied => VariantMeta {
            category: "permission",
            suggestion:
                "Check capability configuration or confirm the operation through the correct flow.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
        },

        // ── Transient errors: retryable, no permanent fix needed ─────────────
        // BC-2.10.007 legal category: "transient" (not "timeout" or "internal").
        PrismError::QueryTimeout { .. } => VariantMeta {
            category: "transient",
            suggestion: "Retry the query with a shorter time range or narrower scope.",
            retryable: true,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
        },

        // BC-2.10.007 §115: SensorRateLimited requires explicit arm binding both fields.
        // BC-2.10.007 §81: source = sensor name (not "prism_mcp").
        // BC-2.10.007 DI-006: raw sensor display text → upstream_message (not message/content).
        // BC-2.10.007 legal category: "transient" (retryable 429 → transient, not "sensor").
        PrismError::SensorRateLimited {
            sensor,
            retry_after_ms,
        } => VariantMeta {
            category: "transient",
            suggestion: "Retry after the indicated delay.",
            retryable: true,
            retry_after_seconds: Some(retry_after_ms / 1000),
            original_params_valid: true,
            source_override: Some(sensor.clone()),
            upstream_message: Some(format!(
                "sensor '{sensor}' rate limited; retry after {retry_after_ms}ms"
            )),
        },

        // BC-2.10.007 §81: source = sensor name; DI-006: body → upstream_message.
        // BC-2.10.007 legal category: "upstream_error" (external service failure).
        PrismError::SensorHttpError {
            sensor,
            status,
            body,
        } => VariantMeta {
            category: "upstream_error",
            suggestion: "Check sensor API status. If the problem persists, see audit log.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: Some(sensor.clone()),
            // Raw body text → upstream_message ONLY (DI-006 injection isolation, EC-10-013).
            upstream_message: Some(format!("HTTP {status}: {body}")),
        },

        // BC-2.10.007 §81: source = sensor name; "upstream_error" for sensor timeouts.
        PrismError::SensorTimeout { sensor, .. }
        | PrismError::SensorResponseParse { sensor, .. } => VariantMeta {
            category: "upstream_error",
            suggestion: "Check sensor API status. If the problem persists, see audit log.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: Some(sensor.clone()),
            upstream_message: None,
        },

        // AuditPersistenceFailed is retryable and transient (not permanent "internal").
        PrismError::AuditPersistenceFailed => VariantMeta {
            category: "transient",
            suggestion:
                "Retry the operation. If the problem persists, check the audit log storage.",
            retryable: true,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
        },

        // ── Catch-all: unknown variants → "upstream_error" (legal BC category) ──
        // "internal" is not in the BC-2.10.007 legal category enum.
        // "upstream_error" is the safest legal fallback for infrastructure failures
        // that don't have a more specific classification.
        _ => VariantMeta {
            category: "upstream_error",
            suggestion: "See audit log for details.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
        },
    };

    // Now consume err to get the canonical code + message.
    let (code_i32, message) = map_prism_error(err);
    // Derive E-* code string from the i32 code.
    let ec_code = if message.starts_with("E-") {
        message.split(':').next().unwrap_or("E-INT-001").to_owned()
    } else {
        match code_i32 {
            codes::INVALID_PARAMS => "E-MCP-002".to_owned(),
            codes::FORBIDDEN => "E-FLAG-001".to_owned(),
            codes::TIMEOUT => "E-QUERY-004".to_owned(),
            codes::NOT_IMPLEMENTED => "E-MCP-003".to_owned(),
            _ => "E-INT-001".to_owned(),
        }
    };

    let source = meta
        .source_override
        .unwrap_or_else(|| "prism_mcp".to_owned());

    let fields = StructuredErrorFields {
        code: ec_code,
        message,
        category: meta.category.to_owned(),
        retryable: meta.retryable,
        retry_after_seconds: meta.retry_after_seconds,
        suggestion: meta.suggestion.to_owned(),
        source,
        original_params_valid: meta.original_params_valid,
        upstream_message: meta.upstream_message,
    };
    let content_text = format!(
        "ERROR: [{}] - {}. {}",
        fields.category, fields.message, fields.suggestion
    );
    build_structured_error_response(fields, content_text)
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
