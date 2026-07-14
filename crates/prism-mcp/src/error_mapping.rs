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
        // (error-taxonomy.md §E-QUERY-003 / ADR-038 §P5-02). Caller-resolvable:
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
        // P6-02 adjudication 2026-06-11; error-taxonomy.md §E-QUERY-036.
        PrismError::UnknownSourceTable(..) => (codes::INVALID_PARAMS, format!("{err}")),

        // E-QUERY-038: Column not found → -32602 INVALID_PARAMS (caller-resolvable).
        //
        // MUST be explicit: `PrismError` is `#[non_exhaustive]`; without this arm the
        // variant would fall through to the catch-all `-32000 INTERNAL_ERROR`, losing
        // the caller-actionable E-QUERY-038 guidance (available_columns, did_you_mean).
        //
        // Maps to INVALID_PARAMS (-32602): the caller supplied a query referencing a
        // column that does not exist in the table — caller-resolvable by correcting the
        // column name or calling `prism_describe` to enumerate available columns.
        //
        // Gate ordering: fires AFTER E-QUERY-037 passes (table must exist first).
        // Reference: S-DEMO-PRISMQL-ONBOARDING-001-B; BC-2.11.016; error-taxonomy.md E-QUERY-038.
        PrismError::ColumnNotFound(..) => (codes::INVALID_PARAMS, format!("{err}")),

        // E-QUERY-039: Enrichment UDF not found → -32602 Invalid params (caller-resolvable).
        //
        // The caller used an enrichment function name that is not registered in the
        // `InfusionRegistry` — commonly an infusion_id (e.g. `threat_intel`) used as if
        // it were a callable per-field UDF name (e.g. `threat_score`).
        //
        // MUST be explicit: without this arm, `EnrichUdfNotFound` falls to the catch-all
        // `-32000 INTERNAL_ERROR`, losing the caller-actionable available_infusions guidance.
        //
        // Maps to INVALID_PARAMS (-32602): caller-resolvable by using a per-field UDF name
        // from `prism_describe` or the PQL reference resource.
        //
        // Reference: S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B; BC-2.11.019; error-taxonomy.md E-QUERY-039.
        PrismError::EnrichUdfNotFound(..) => (codes::INVALID_PARAMS, format!("{err}")),

        // E-QUERY-002: Query type mismatch → -32602 Invalid params (caller-resolvable).
        //
        // The caller used an operator that is not valid for the column's ColumnType —
        // e.g., `severity > 5` on a String column. Caller-resolvable by switching to
        // a valid operator (use `valid_operators_for_type` in the structured response).
        //
        // MUST be explicit: without this arm, `QueryTypeMismatch` falls to the catch-all
        // `-32000 INTERNAL_ERROR` arm, losing the caller-actionable type-mismatch guidance.
        //
        // Reference: S-DEMO-PRISMQL-ONBOARDING-001-B; BC-2.11.017; error-taxonomy.md E-QUERY-002.
        PrismError::QueryTypeMismatch { .. } => (codes::INVALID_PARAMS, format!("{err}")),

        // E-QUERY-037: Table not available → -32602 Invalid params (caller-resolvable).
        //
        // MUST be explicit: `PrismError` is `#[non_exhaustive]`; without this arm the
        // variant would fall through to the catch-all `-32000 INTERNAL_ERROR`, losing the
        // caller-actionable E-QUERY-037 guidance (available sensor list, did_you_mean).
        //
        // The error message is the full Display of the variant — it contains the table name,
        // sensor name, available sensors list, available tables list, and the Levenshtein
        // suggestion. All fields are safe to surface to the MCP caller (no credential values).
        //
        // Maps to INVALID_PARAMS (-32602): the caller supplied a query referencing a sensor
        // that is not configured — caller-resolvable by adding the sensor to prism.toml.
        //
        // Reference: S-3.13 AC-2; BC-2.11.001; error-taxonomy.md E-QUERY-037.
        PrismError::TableNotAvailable(..) => (codes::INVALID_PARAMS, format!("{err}")),

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
        PrismError::AuthTokenExpired | PrismError::AuthTokenInvalid { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-SPEC-*: Spec engine errors → -32000 Internal
        // (configuration issues that the API caller cannot resolve)
        PrismError::Spec(_)
        | PrismError::SpecNotFound { .. }
        | PrismError::SpecValidationFailed { .. }
        | PrismError::SpecHotReloadFailed { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-CFG-102..105: Config errors → -32000 Internal (operator-resolvable,
        // not caller-resolvable; ADR-038 D4 — arm covers only the four
        // operator-class variants after the ClientNotFound split).
        PrismError::ConfigNotFound { .. }
        | PrismError::ConfigParseFailed { .. }
        | PrismError::ConfigValidationFailed { .. }
        | PrismError::ConfigSnapshotStale { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-STORE-*: Storage errors → -32000 Internal
        PrismError::StorageOpenFailed { .. }
        | PrismError::StorageWriteFailed { .. }
        | PrismError::StorageReadFailed { .. }
        | PrismError::StorageDomainNotFound { .. }
        | PrismError::StorageKeyNotFound { .. }
        | PrismError::StorageLockHeld { .. }
        | PrismError::StorageHealthCheckFailed { .. }
        | PrismError::SchemaMismatch { .. }
        | PrismError::StorageBatchFailed { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-SENSOR-020: Sensor rate limited — EXPLICIT arm required.
        // BC-2.10.007 §115-116: bind both fields; sensor→source (used in
        // prism_error_to_structured_call_result), retry_after_ms/1000→retry_after_seconds.
        // Kept separate so `sensor` is bound for the structured error caller to use.
        // SEC-002 (CWE-200): message must NOT contain sensor name or retry_after_ms —
        // those are sensor-identifying details that belong in upstream_message only
        // (which is null per DI-006 — the rate limit notice is synthesized by Prism,
        // not raw upstream text). Generic message prevents dual-channel disclosure.
        PrismError::SensorRateLimited { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-SENSOR-001..003: Other sensor adapter errors → -32000 Internal
        // (external service failures; detail in audit log)
        PrismError::SensorHttpError { .. }
        | PrismError::SensorTimeout { .. }
        | PrismError::SensorResponseParse { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

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
        | PrismError::OcsfTimestampParseError { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-CRED-*: Credential errors → -32000 Internal
        // (NEVER leak credential details in MCP responses)
        PrismError::InvalidCredentialName { .. }
        | PrismError::CredentialNotFound { .. }
        | PrismError::CredentialStoreError { .. }
        | PrismError::CredentialEncryptionError { .. }
        | PrismError::EncryptionKeyMissing { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-IO-001: I/O error → -32000 Internal
        PrismError::Io(_) => (codes::INTERNAL_ERROR, "Internal error".to_owned()),

        // E-MCP-003: MCP serialization error → -32000 Internal
        PrismError::McpSerializationError { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-SAFETY-*: Safety boundary violations → -32000 Internal
        // (safety violations are logged; do not surface detail to caller)
        PrismError::SafetyContextContamination { .. }
        | PrismError::SafetyDataExfiltration { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-QUERY-002/034/005/010 + E-WATCHDOG-001: Query planning/execution/
        // materialization-limit/memory errors → -32000 Internal
        PrismError::QueryPlanFailed { .. }
        | PrismError::QueryExecutionFailed { .. }
        | PrismError::QueryMaterializationLimitExceeded { .. }
        | PrismError::QueryMemoryBudgetExceeded { .. }
        | PrismError::QueryVirtualFieldFailed { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-QUERY-008: Query denylisted → -32000 Internal
        PrismError::QueryDenylisted { .. } => (codes::INTERNAL_ERROR, "Internal error".to_owned()),

        // E-QUERY-025: Write partial failure → -32000 Internal
        PrismError::WritePartialFailure { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-SCHED-*: Scheduler errors → -32000 Internal
        PrismError::ScheduleNotFound { .. }
        | PrismError::ScheduleConflict { .. }
        | PrismError::ScheduleCronInvalid { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-DET-*: Detection errors → -32000 Internal
        PrismError::DetectionRuleParseFailed { .. }
        | PrismError::DetectionRuleNotFound { .. }
        | PrismError::DetectionStateCorrupt { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-CASE-*: Case management errors → -32000 Internal
        PrismError::CaseNotFound { .. } | PrismError::CaseStateTransitionInvalid { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-WATCH-*/E-WATCHDOG-*: Watchdog errors → -32000 Internal
        PrismError::WatchdogHeartbeatMissed { .. }
        | PrismError::WatchdogRestartLimitExceeded { .. }
        | PrismError::WatchdogKilled { .. } => (codes::INTERNAL_ERROR, "Internal error".to_owned()),

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
        PrismError::Infusion(_) => (codes::INTERNAL_ERROR, "Internal error".to_owned()),

        // E-PLUGIN-*: WASM plugin errors → -32000 Internal
        PrismError::Plugin(_) => (codes::INTERNAL_ERROR, "Internal error".to_owned()),

        // E-IOC-*: IOC errors → -32000 Internal
        PrismError::IocFeedParseFailed { .. } | PrismError::IocLookupFailed { .. } => {
            (codes::INTERNAL_ERROR, "Internal error".to_owned())
        }

        // E-QUERY-040: SQL→Pipe redundant row limit → -32602 INVALID_PARAMS (ADR-043).
        //
        // MUST be explicit: `PrismError` is `#[non_exhaustive]`; without this arm the
        // variant would fall through to the catch-all `-32000 INTERNAL_ERROR`, losing the
        // caller-actionable E-QUERY-040 guidance.
        //
        // Caller-resolvable: remove either the SQL `LIMIT n` or the pipe `| limit m`.
        // Reference: BC-2.11.020; ADR-043 §C; error-taxonomy.md E-QUERY-040.
        PrismError::RedundantRowLimit { .. } => (codes::INVALID_PARAMS, format!("{err}")),

        // E-QUERY-041: Temporal literal not parseable as RFC-3339 UTC → -32602 INVALID_PARAMS.
        //
        // MUST be explicit: `PrismError` is `#[non_exhaustive]`; without this arm the
        // variant would fall through to the catch-all `-32000 INTERNAL_ERROR`, losing the
        // caller-actionable datetime format guidance (ADR-052 D4; BC-2.11.021 §MCP mapping).
        //
        // Caller-resolvable: supply a full RFC-3339 timestamp with UTC offset
        // (e.g., '2026-07-03T00:00:00Z'). Date-only and offset-less forms are rejected.
        // Reference: BC-2.11.021; ADR-052 D4; error-taxonomy.md E-QUERY-041.
        PrismError::TemporalLiteralUnparseable { .. } => (codes::INVALID_PARAMS, format!("{err}")),

        // E-QUERY-042: Temporal literal in structurally invalid position → -32602 INVALID_PARAMS.
        //
        // Covers three positions per ADR-052 §D4 v1.10:
        //   - TemporalLiteralPosition::GroupBy: `GROUP BY '2026-06-24'` — constant has no effect
        //   - TemporalLiteralPosition::OrderBy: `ORDER BY '2026-06-24'` — constant has no effect
        //   - TemporalLiteralPosition::NonColumnLhsComparison: non-Field LHS with date-like RHS
        //
        // Caller-resolvable: use a column name in GROUP BY/ORDER BY, or use RFC-3339 for
        // datetime comparisons with bare column references. See Display message for guidance.
        //
        // MUST be explicit: `PrismError` is `#[non_exhaustive]`; without this arm the variant
        // falls through to catch-all `-32000 INTERNAL_ERROR`, making a caller-resolvable
        // query mistake appear as an internal server error.
        //
        // Reference: error-taxonomy.md §E-QUERY-042 v2.14; ADR-052 §D4 v1.10.
        PrismError::TemporalLiteralInvalidPosition { .. } => {
            (codes::INVALID_PARAMS, format!("{err}"))
        }

        // E-QUERY-043: IN subquery in SELECT projection, GROUP BY, or ORDER BY → -32602 INVALID_PARAMS.
        //
        // DataFusion 53.1.0 cannot execute `Expr::InSubquery` in scalar expression positions.
        // The plan-time gate `check_expr_insubquery_projection` fires before DataFusion
        // planning and returns this structured error (F-CSD-P4-001 Option A adjudication).
        //
        // Caller-resolvable: rewrite as `WHERE field IN (SELECT ...)`. The hint field carries
        // the actionable guidance.
        //
        // MUST be explicit: without this arm the variant falls through to catch-all
        // `-32000 INTERNAL_ERROR`, losing the caller-actionable rewrite guidance.
        //
        // Reference: F-CSD-P4-001 adjudication 2026-07-10; error-taxonomy.md §E-QUERY-043.
        PrismError::ExprInSubqueryProjectionNotSupported { .. } => {
            (codes::INVALID_PARAMS, format!("{err}"))
        }

        // E-INT-001: Internal invariant violated → -32000 Internal
        // Detail is suppressed — audit log has it.
        PrismError::Internal { .. } => (codes::INTERNAL_ERROR, "Internal error".to_owned()),

        // Catch-all for future PrismError variants added after this match
        // was written (non_exhaustive enum). Defaults to -32000 Internal.
        _ => (codes::INTERNAL_ERROR, "Internal error".to_owned()),
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
// BC-2.10.007 — structured error envelope API
// ---------------------------------------------------------------------------

/// BC-2.10.007 wire shape — 9 fields inside `structuredContent.error`.
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
    /// Error category — must be a legal BC-2.10.007 §77 enum value:
    /// `"transient"` | `"authentication"` | `"validation"` | `"not_found"` |
    /// `"permission"` | `"upstream_error"` | `"configuration"` | `"safety"` |
    /// `"internal"` (v1.7: Prism-side infrastructure/invariant failures; sensor not reached).
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
    /// Near-text snippet from the query at the parse-error offset (E-QUERY-001 / BC-2.11.017 AC-003).
    ///
    /// Set to `Some(token)` when the error is a `QueryParseFailed` and the original query
    /// string is available. The snippet is truncated to ≤50 chars (DI-006). `None` for all
    /// other error types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub near_text: Option<String>,
    /// Reference pointer for query syntax documentation (E-QUERY-001 / BC-2.11.017 AC-003).
    ///
    /// Set to `Some("prismql://reference")` when the error is a `QueryParseFailed`.
    /// `None` for all other error types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_pointer: Option<String>,
    /// Valid operators for the column type involved in a type-mismatch error
    /// (E-QUERY-002 / BC-2.11.017 AC-003 `valid_operators_for_type`).
    ///
    /// Populated when the error carries enough column-type context to call
    /// `prism_query::engine::valid_operators_for_type(column_type)`.
    /// `None` for all other error types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_operators_for_type: Option<Vec<String>>,
    /// How-to-fix guidance for security-limit errors (E-QUERY-003 / BC-2.11.017 AC-003).
    ///
    /// Set via `prism_query::engine::how_to_fix_for_security_limit(detail)` for
    /// `QuerySecurityLimitExceeded` errors. `None` for all other error types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_to_fix: Option<String>,
    /// Column names available in the table for this client (E-QUERY-038 / BC-2.11.016 AC-001).
    ///
    /// ALWAYS present (never null, never omitted) for `ColumnNotFound` errors.
    /// The LLM agent uses this array to self-correct the column name.
    /// `None` for all other error types (field absent from JSON via `skip_serializing_if`).
    /// Org-scoped per DI-008: only columns for this client's registered schema.
    /// Injection-safe: column names originate from operator TOML specs, not sensor API responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_columns: Option<Vec<String>>,
    /// Levenshtein-based spelling suggestion for column names (E-QUERY-038 / BC-2.11.016 AC-001).
    ///
    /// Present as the best-match column name (e.g. `"severity"`) when Levenshtein distance ≤ 3.
    /// ABSENT (not null, not empty — key omitted from JSON) when no match is within threshold.
    /// `None` for all other error types (field absent from JSON via `skip_serializing_if`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_you_mean: Option<String>,
    /// Canonical re-serialized PQL for three-mode bridge errors (ADR-046, BC-2.11.023 AC-010).
    ///
    /// When the mode-bridge produces a D1 error (wrong parsing mode selected), this field
    /// carries the normalized PQL string showing the correct form. The LLM agent uses this
    /// to self-correct its next query without manual reformulation.
    ///
    /// ABSENT (key omitted from JSON via `skip_serializing_if`) for all non-mode-bridge errors.
    /// `None` for all error types that do not involve a mode mismatch.
    ///
    /// Reference: ADR-046 §D1; BC-2.11.023 AC-010; S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_pql: Option<String>,
}

impl StructuredErrorFields {
    /// Construct all 9 BC-2.10.007 structured error fields.
    ///
    /// External callers MUST use this constructor — struct literal syntax is blocked by
    /// `#[non_exhaustive]` (HC-3, S-5.02).
    ///
    /// # Arguments (positional, matching field order)
    /// 1. `code` — canonical E-* error code (e.g. `"E-MCP-001"`)
    /// 2. `message` — human-readable message (no raw sensor text, DI-006)
    /// 3. `category` — legal BC-2.10.007 §77 enum value: `"transient"` | `"authentication"` |
    ///    `"validation"` | `"not_found"` | `"permission"` | `"upstream_error"` |
    ///    `"configuration"` | `"safety"` | `"internal"` (v1.7)
    /// 4. `retryable` — whether the caller may retry
    /// 5. `retry_after_seconds` — wait hint (null when not applicable)
    /// 6. `suggestion` — actionable suggestion for the caller
    /// 7. `source` — error source identifier (e.g. `"prism_mcp"`)
    /// 8. `original_params_valid` — whether the original request params were structurally valid
    /// 9. `upstream_message` — raw upstream sensor text (null for Prism-originating errors, DI-006)
    ///
    /// # Bool layout (F-11)
    ///
    /// Two `bool` args: position 4 = `retryable`, position 8 = `original_params_valid`.
    /// Use `StructuredErrorFields::builder()` when the call site has no adjacent type context
    /// to disambiguate the two booleans.
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
            // BC-2.11.017 AC-003: near_text and reference_pointer default to None.
            // Only set for QueryParseFailed via prism_error_to_structured_call_result.
            near_text: None,
            reference_pointer: None,
            // BC-2.11.017 AC-003: valid_operators_for_type and how_to_fix default to None.
            // Set for specific error variants via prism_error_to_structured_call_result.
            valid_operators_for_type: None,
            how_to_fix: None,
            // BC-2.11.016 AC-001: available_columns and did_you_mean default to None.
            // Only set for ColumnNotFound via prism_error_to_structured_call_result.
            available_columns: None,
            did_you_mean: None,
            // ADR-046 BC-2.11.023 AC-010: normalized_pql defaults to None.
            // Only set for mode-bridge D1 errors via prism_error_to_structured_call_result.
            normalized_pql: None,
        }
    }

    /// Named-field builder for `StructuredErrorFields` (F-11).
    ///
    /// Prefer this over `new()` at call sites where both `bool` args cannot be
    /// verified from adjacent context.  Named setters eliminate the bool-swap risk.
    pub fn builder() -> StructuredErrorFieldsBuilder {
        StructuredErrorFieldsBuilder::default()
    }
}

/// Builder for [`StructuredErrorFields`] (F-11 — prevents bool-swap risk).
///
/// Obtain via [`StructuredErrorFields::builder()`].
#[derive(Debug, Default)]
pub struct StructuredErrorFieldsBuilder {
    code: Option<String>,
    message: Option<String>,
    category: Option<String>,
    retryable: bool,
    retry_after_seconds: Option<u64>,
    suggestion: Option<String>,
    source: Option<String>,
    original_params_valid: bool,
    upstream_message: Option<String>,
    available_columns: Option<Vec<String>>,
    did_you_mean: Option<String>,
    normalized_pql: Option<String>,
}

impl StructuredErrorFieldsBuilder {
    pub fn code(mut self, v: impl Into<String>) -> Self {
        self.code = Some(v.into());
        self
    }
    pub fn message(mut self, v: impl Into<String>) -> Self {
        self.message = Some(v.into());
        self
    }
    pub fn category(mut self, v: impl Into<String>) -> Self {
        self.category = Some(v.into());
        self
    }
    pub fn retryable(mut self, v: bool) -> Self {
        self.retryable = v;
        self
    }
    pub fn retry_after_seconds(mut self, v: Option<u64>) -> Self {
        self.retry_after_seconds = v;
        self
    }
    pub fn suggestion(mut self, v: impl Into<String>) -> Self {
        self.suggestion = Some(v.into());
        self
    }
    pub fn source(mut self, v: impl Into<String>) -> Self {
        self.source = Some(v.into());
        self
    }
    pub fn original_params_valid(mut self, v: bool) -> Self {
        self.original_params_valid = v;
        self
    }
    pub fn upstream_message(mut self, v: Option<String>) -> Self {
        self.upstream_message = v;
        self
    }
    pub fn available_columns(mut self, v: Option<Vec<String>>) -> Self {
        self.available_columns = v;
        self
    }
    pub fn did_you_mean(mut self, v: Option<String>) -> Self {
        self.did_you_mean = v;
        self
    }
    pub fn normalized_pql(mut self, v: Option<String>) -> Self {
        self.normalized_pql = v;
        self
    }
    /// Build the `StructuredErrorFields`.
    ///
    /// # Panics
    ///
    /// Panics if `code`, `message`, `category`, `suggestion`, or `source` were not set.
    pub fn build(self) -> StructuredErrorFields {
        StructuredErrorFields {
            code: self
                .code
                .expect("StructuredErrorFieldsBuilder: code is required"),
            message: self
                .message
                .expect("StructuredErrorFieldsBuilder: message is required"),
            category: self
                .category
                .expect("StructuredErrorFieldsBuilder: category is required"),
            retryable: self.retryable,
            retry_after_seconds: self.retry_after_seconds,
            suggestion: self
                .suggestion
                .expect("StructuredErrorFieldsBuilder: suggestion is required"),
            source: self
                .source
                .expect("StructuredErrorFieldsBuilder: source is required"),
            original_params_valid: self.original_params_valid,
            upstream_message: self.upstream_message,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: self.available_columns,
            did_you_mean: self.did_you_mean,
            normalized_pql: self.normalized_pql,
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
/// Produces the BC-2.10.007 wire shape:
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
/// as explicit `null` when not applicable (null-not-absent invariant, BC-2.10.007).
///
/// # Parameters
/// - `fields`: the 9 structured error fields per BC-2.10.007
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

    // BC-2.11.017 AC-003: near_text and reference_pointer are optional enrichment fields.
    // Only included in the JSON object when Some; omitted (not null) to avoid polluting
    // every error response with null fields (skip_serializing_if = "Option::is_none").
    let mut error_obj = serde_json::json!({
        "code": fields.code,
        "message": fields.message,
        "category": fields.category,
        "retryable": fields.retryable,
        "retry_after_seconds": retry_after_seconds,
        "suggestion": fields.suggestion,
        "source": fields.source,
        "original_params_valid": fields.original_params_valid,
        "upstream_message": upstream_message,
    });
    if let Some(nt) = fields.near_text {
        error_obj["near_text"] = serde_json::Value::String(nt);
    }
    if let Some(rp) = fields.reference_pointer {
        error_obj["reference_pointer"] = serde_json::Value::String(rp);
    }
    if let Some(ops) = fields.valid_operators_for_type {
        error_obj["valid_operators_for_type"] =
            serde_json::Value::Array(ops.into_iter().map(serde_json::Value::String).collect());
    }
    if let Some(htf) = fields.how_to_fix {
        error_obj["how_to_fix"] = serde_json::Value::String(htf);
    }
    // BC-2.11.016 AC-001: available_columns emitted as JSON array when Some.
    // ALWAYS present for ColumnNotFound (set to Some(vec![...])), absent for all other errors.
    // did_you_mean emitted as JSON string when Some; absent (not null) when None.
    if let Some(cols) = fields.available_columns {
        error_obj["available_columns"] =
            serde_json::Value::Array(cols.into_iter().map(serde_json::Value::String).collect());
    }
    if let Some(dym) = fields.did_you_mean {
        error_obj["did_you_mean"] = serde_json::Value::String(dym);
    }
    // ADR-046 BC-2.11.023 AC-010: normalized_pql for mode-bridge D1 errors.
    // Absent (key omitted) for all non-mode-bridge error types.
    if let Some(npql) = fields.normalized_pql {
        error_obj["normalized_pql"] = serde_json::Value::String(npql);
    }
    let structured_content = serde_json::json!({
        "error": error_obj,
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

/// BC-2.10.007 spec R2 flat-path helper: extract `retry_after_ms` from `SensorRateLimited`
/// alongside the flat [`ErrorData`] representation.
///
/// # Design role (BC-2.10.007 §"to_error_data_with_retry helper contract")
///
/// BC-2.10.007 §108 mandates a helper function **"or equivalent inline mapping"** for
/// extracting `retry_after_ms` from `SensorRateLimited`. This function is the **flat-path**
/// variant of that helper (return type `(ErrorData, u64)`) — it covers the case where a
/// caller already operates on the flat `Err(ErrorData)` MCP error boundary and needs the
/// `retry_after_ms` value alongside the error data.
///
/// The **structured path** ([`prism_error_to_structured_call_result`]) satisfies the same
/// BC requirement via the "equivalent inline mapping" clause: it binds `retry_after_ms`
/// directly in its `SensorRateLimited` arm and applies the SEC-001 `.max(1)` floor
/// (`(retry_after_ms / 1000).max(1)`). Both paths are BC-compliant; they serve different
/// error-surface boundaries.
///
/// # Contract
///
/// - Intended for use with `PrismError::SensorRateLimited { .. }`. For any other variant,
///   returns `retry_after_ms = 0` (graceful — no panic). OBS-1 de-footgun fix: the prior
///   `panic!` on misuse made this public function unsafe to call from match arms that include
///   non-rate-limited variants.
/// - Returns `(ErrorData, retry_after_ms_raw_u64)` — the `u64` is the raw millisecond value.
///   Callers converting to `retry_after_seconds` MUST apply `.max(1)` (SEC-001 floor).
/// - For non-`SensorRateLimited` variants: returns `retry_after_ms = 0`.
///   Callers applying `.max(1)` will produce 1s minimum retry hint.
///
/// # Note
///
/// This function has no production caller in the current server implementation because
/// `SensorRateLimited` is always routed to `prism_error_to_structured_call_result` (the
/// structured path). It remains part of the public API as the BC-2.10.007 spec R2 flat-path
/// helper contract, exercised by `test_BC_2_10_007_sensor_rate_limited_retry_after_seconds_ms_to_s_conversion`.
pub fn to_error_data_with_retry(err: PrismError) -> (ErrorData, u64) {
    // Extract retry_after_ms BEFORE consuming err via map_prism_error.
    // For non-SensorRateLimited variants, return 0 (no retry hint) — no panic.
    // OBS-1: the prior panic! was a public-function footgun; 0u64 is the graceful
    // (ErrorData, u64) equivalent of BC-2.10.007 §111 "return None for other variants".
    // Callers applying .max(1) floor will produce 1s minimum.
    let retry_after_ms = match &err {
        PrismError::SensorRateLimited { retry_after_ms, .. } => *retry_after_ms,
        _other => 0u64, // graceful: no retry hint; callers apply .max(1) SEC-001 floor
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
    // BC-2.10.007 §category legal enum (9 values):
    //   transient | authentication | validation | not_found | permission |
    //   upstream_error | configuration | safety | internal
    // BC-2.10.007 §81 source values:
    //   "prism_mcp" for MCP-layer errors; sensor API name for sensor errors;
    //   "prism_config" for configuration errors.
    // BC-2.10.007 DI-006 / EC-10-013: raw sensor text goes in upstream_message ONLY.
    struct VariantMeta {
        category: &'static str,
        /// Static suggestion string for variants with no per-instance guidance.
        suggestion: &'static str,
        /// Runtime-owned suggestion override — used when a variant carries its own
        /// actionable guidance that must be surfaced verbatim (e.g. CapabilityDenied.suggestion).
        /// When `Some`, takes precedence over `suggestion`.
        /// MED-1 (BC-2.10.007): threads CapabilityDenied's own actionable guidance
        /// through instead of discarding it in favour of a static string.
        owned_suggestion: Option<String>,
        retryable: bool,
        retry_after_seconds: Option<u64>,
        original_params_valid: bool,
        /// Override source for sensor errors; `None` → default "prism_mcp".
        source_override: Option<String>,
        /// Raw upstream sensor text for DI-006 isolation; `None` for Prism-originating errors.
        upstream_message: Option<String>,
        /// Pin the canonical E-* error code directly (F-1 fix).
        /// When `Some`, bypasses message-string-based code inference in `map_prism_error`.
        /// Required for variants where `map_prism_error` returns the generic
        /// "Internal error" message (no E- prefix to infer from).
        ec_code_override: Option<&'static str>,
        /// Near-text snippet for QueryParseFailed (BC-2.11.017 AC-003 / E-QUERY-001).
        /// None for all other variants.
        near_text: Option<String>,
        /// Reference pointer for QueryParseFailed (BC-2.11.017 AC-003).
        /// None for all other variants.
        reference_pointer: Option<&'static str>,
        /// Valid operators for the column type in a type-mismatch error (E-QUERY-002).
        /// None for all other variants.
        valid_operators_for_type: Option<Vec<String>>,
        /// How-to-fix guidance for security-limit errors (E-QUERY-003).
        /// Set via `how_to_fix_for_security_limit(detail)` for QuerySecurityLimitExceeded.
        /// None for all other variants.
        how_to_fix: Option<String>,
        /// Available column names for the table (E-QUERY-038 / BC-2.11.016 AC-001).
        /// Some(vec![...]) always populated for ColumnNotFound; None for all other variants.
        available_columns: Option<Vec<String>>,
        /// Levenshtein spelling suggestion (E-QUERY-038 / BC-2.11.016 AC-001).
        /// Some(best_match) when distance ≤ 3; None (omitted from JSON) otherwise.
        did_you_mean: Option<String>,
        /// Canonical normalized PQL for mode-bridge D1 errors (ADR-046 / BC-2.11.023 AC-010).
        /// None for all non-mode-bridge error types.
        normalized_pql: Option<String>,
    }
    let meta = match &err {
        // ── Authentication errors: credential invalid or identity format failure ─
        // BC-2.10.007 §Category rule: "Credential invalid or identity validation
        // failure" → category "authentication". LLM-agent strategy: re-authenticate;
        // check credential_ref.
        //
        // Two sub-cases:
        //   (a) Identity FORMAT failures (InvalidOrgSlug, InvalidAnalystId, InvalidClientId):
        //       The identity string was malformed — original_params_valid: false.
        //       E-AUTH-001/002/003 codes are inferred from the Display prefix (map_prism_error
        //       returns the formatted message which starts with "E-AUTH-NNN: ...").
        //
        //   (b) Valid-format credential failures (AuthTokenExpired, AuthTokenInvalid):
        //       The token format was structurally valid but the credential is expired/invalid.
        //       original_params_valid: true (params were well-formed; the credential failed).
        //       ec_code_override required: map_prism_error returns INTERNAL_ERROR with
        //       "Internal error" for these variants — no E- prefix to infer.
        //       Pin E-AUTH-010/011 directly.
        //
        // HIGH-1 fix (BC-2.10.007 §Category rule):
        //   - InvalidOrgSlug/InvalidAnalystId/InvalidClientId: moved FROM "validation"
        //   - AuthTokenExpired/AuthTokenInvalid: moved FROM catch-all "upstream_error"

        // (a) Identity format failures: malformed → original_params_valid: false
        PrismError::InvalidOrgSlug { .. }
        | PrismError::InvalidAnalystId { .. }
        | PrismError::InvalidClientId { .. } => VariantMeta {
            category: "authentication",
            suggestion: "Check the identity format and re-authenticate.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: false,
            source_override: None,
            upstream_message: None,
            // E-AUTH-001/002/003 inferred from map_prism_error Display prefix (starts "E-AUTH-").
            owned_suggestion: None,
            ec_code_override: None,
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // (b) Valid-format credential failures: token expired/invalid → original_params_valid: true
        PrismError::AuthTokenExpired => VariantMeta {
            category: "authentication",
            suggestion: "The auth token has expired. Re-authenticate and obtain a fresh token.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            // map_prism_error returns INTERNAL_ERROR/"Internal error" for this
            // variant — no E- prefix. Pin E-AUTH-010 directly.
            owned_suggestion: None,
            ec_code_override: Some("E-AUTH-010"),
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        PrismError::AuthTokenInvalid { .. } => VariantMeta {
            category: "authentication",
            suggestion: "The auth token is invalid. Re-authenticate and obtain a valid token.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            // map_prism_error returns INTERNAL_ERROR/"Internal error" for this
            // variant — no E- prefix. Pin E-AUTH-011 directly.
            owned_suggestion: None,
            ec_code_override: Some("E-AUTH-011"),
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // ── Validation errors: caller-supplied bad parameters ────────────────
        // ClientNotFound is intentionally EXCLUDED from this group per BC-2.10.004 §87:
        // a well-formed-but-unregistered client_id is a configuration error, not a
        // bad-parameter error — `original_params_valid: true`.
        // Write-policy variants (WriteUnbounded, WriteBatchLimitExceeded, etc.) are
        // EXCLUDED from this group per F-3: the params are structurally valid but
        // the write policy denied them — `original_params_valid: true`.
        // InvalidOrgSlug/InvalidAnalystId/InvalidClientId are EXCLUDED from this group
        // per HIGH-1 fix: identity FORMAT failures map to "authentication" (BC-2.10.007).
        // AuthTokenExpired/AuthTokenInvalid are EXCLUDED: moved to "authentication" arm above.
        // SensorNotRegisteredForOrg is EXCLUDED from this group per OBS-1 (BC-2.10.007):
        // cross-org sensor isolation is a scoping/permission denial, NOT a param-validation
        // failure. The org slug and sensor name are structurally valid. Moved to "permission".
        // ── E-QUERY-001 parse error: extract near_text + reference_pointer ───
        // BC-2.11.017 AC-003: the near_text snippet (≤50 chars token at `offset`)
        // and reference_pointer ("prismql://reference") must appear in the MCP error
        // envelope for QueryParseFailed. Uses the `query` field added to the variant
        // (S-DEMO-PRISMQL-ONBOARDING-001-B) to compute the snippet.
        PrismError::QueryParseFailed {
            ref query,
            offset,
            ..
        } => {
            // BC-2.11.017 AC-003: compute near_text as the first whitespace-delimited
            // token at or before `offset`. The parser reports the position of the
            // UNEXPECTED token (e.g. `*` at position 6 for "SELCT * FROM …"), but the
            // meaningful token for the user is the PRECEDING word ("SELCT") that caused
            // the parser to be in a state where `*` was unexpected.
            //
            // Algorithm: find the start of the word that CONTAINS offset-1 by scanning
            // backward from `offset` to the last whitespace. If offset=0, use extract_near_text
            // at 0 (already at start of query). DI-006: truncated to ≤50 chars by extract_near_text.
            let effective_offset = if *offset == 0 {
                0
            } else {
                // Walk backward from (offset - 1) to find the start of the preceding token.
                // Two-step algorithm:
                //   1. Skip any whitespace immediately before offset (e.g. "SELCT " → skip ' ').
                //   2. Then find the last whitespace before the preceding non-whitespace run.
                //      That gives the start of the last complete token before offset.
                let before_offset = query.get(..*offset).unwrap_or(query.as_str());
                // Find last non-whitespace position (skip trailing spaces before offset).
                let last_non_ws = before_offset
                    .rfind(|c: char| !c.is_whitespace())
                    .unwrap_or(0); // all whitespace or empty → use 0
                // Find the start of the word ending at last_non_ws.
                //
                // SAFETY (F-001B-PASS-CRIT-001): `rfind(char::is_whitespace)` returns the first
                // byte of the whitespace char. For multibyte WS (e.g. U+00A0=2 bytes,
                // U+3000=3 bytes), `pos + 1` is mid-char; advance by the full char width.
                //
                // OBS-1 (symmetric fix): `rfind(!c.is_whitespace())` also returns the FIRST
                // byte of the last non-whitespace char. `get(..=last_non_ws)` is equivalent to
                // `get(..last_non_ws+1)` — mid-codepoint for multibyte non-WS chars (e.g. `é`
                // U+00E9 = 0xC3 0xA9). That causes `get` to return `None`, which collapses
                // `preceding_word_start` to 0 and produces the wrong start-of-query near_text.
                // Fix: advance `last_non_ws` by the full char width before slicing.
                let non_ws_char_end = last_non_ws
                    + before_offset[last_non_ws..]
                        .chars()
                        .next()
                        .map_or(1, |c| c.len_utf8());
                let preceding_word_start = before_offset.get(..non_ws_char_end)
                    .and_then(|s| {
                        s.rfind(|c: char| c.is_whitespace()).map(|pos| {
                            // Advance past the full whitespace char (char-boundary safe,
                            // F-001B-PASS-CRIT-001).
                            let ws_char = s[pos..].chars().next().map_or(1, |c| c.len_utf8());
                            pos + ws_char
                        })
                    })
                    .unwrap_or(0);
                preceding_word_start
            };
            let near_text = prism_query::engine::extract_near_text(query, effective_offset);
            VariantMeta {
                category: "validation",
                suggestion: "Check the request parameters and retry.",
                retryable: false,
                retry_after_seconds: None,
                original_params_valid: false,
                source_override: None,
                upstream_message: None,
                owned_suggestion: None,
                // F-198-FRESH-MED-001 fix: pin E-QUERY-001 directly.
                // Without this override, map_prism_error returns
                // "PrismQL parse error: {detail}" (no "E-" prefix), so the code
                // derivation falls to `match INVALID_PARAMS => "E-MCP-002"` —
                // which is semantically wrong (means "tool not available").
                // BC-2.11.017 §E-QUERY-001 + AC-003 mandate code="E-QUERY-001".
                ec_code_override: Some("E-QUERY-001"),
                // BC-2.11.017 EC-11-046: near_text must be PRESENT as "" (empty string)
                // at end-of-input, NOT absent. `Some(near_text)` preserves the key for
                // both mid-input tokens ("token") and end-of-input ("").
                near_text: Some(near_text),
                reference_pointer: Some("prismql://reference"),
                valid_operators_for_type: None,
                how_to_fix: None,
                available_columns: None,
                did_you_mean: None,
                // BC-2.11.023 AC-010 (CRIT-002): wire mode_bridge_normalized_pql into the
                // QueryParseFailed arm. For D1 mode-bridge errors (SQL+pipe mix), this
                // computes a valid Pipe-mode rewrite so the agent can self-correct.
                // Returns None for non-mode-bridge parse errors (unknown keywords, etc.).
                // Canonical implementation lives in prism_query::error_recovery (OBS-1).
                normalized_pql: prism_query::error_recovery::mode_bridge_normalized_pql(query),
            }
        }

        // ── E-QUERY-037 table-not-found: use suggestion from TableNotAvailableDetails ─
        // BC-2.11.017 AC-004: the `suggestion` field of `TableNotAvailableDetails`
        // already contains the `prism_describe` pointer (set by `check_availability_gate`).
        // Surface it as the `owned_suggestion` so the MCP envelope carries "prism_describe".
        PrismError::TableNotAvailable(ref d) => VariantMeta {
            category: "validation",
            suggestion: "Check the request parameters and retry.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: false,
            source_override: None,
            upstream_message: None,
            owned_suggestion: if d.suggestion.is_empty() {
                None
            } else {
                Some(d.suggestion.clone())
            },
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // ── E-QUERY-003 security-limit error: wire how_to_fix_for_security_limit ──
        // BC-2.11.017 AC-003: the `how_to_fix_for_security_limit(detail)` helper must
        // appear in the MCP error envelope for QuerySecurityLimitExceeded errors.
        // The helper returns operator-actionable remediation guidance for the specific
        // security limit that was exceeded (e.g. depth limit, breadth limit).
        PrismError::QuerySecurityLimitExceeded { ref detail } => VariantMeta {
            category: "validation",
            suggestion: "Check the request parameters and retry.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: false,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: Some(prism_query::engine::how_to_fix_for_security_limit(detail)),
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // E-QUERY-041: plan-time temporal literal pre-validator (ADR-052 D4).
        // Dedicated arm: RFC-3339 format guidance must appear in the suggestion field for
        // analyst-actionable structured output. ec_code_override: None because the Display
        // starts with "E-QUERY-041:" so the message.starts_with("E-") inference path
        // correctly derives the code without explicit override.
        // original_params_valid: false — the bad date-format literal IS the invalid parameter.
        // TD-VSDD-060 sibling-site: RedundantRowLimit (E-QUERY-040) was in shared group but
        // TemporalLiteralUnparseable requires dedicated arm for analyst guidance (pass-3 HIGH-1).
        PrismError::TemporalLiteralUnparseable { .. } => VariantMeta {
            category: "validation",
            suggestion: "Use RFC-3339 format with UTC offset.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: false,
            source_override: None,
            upstream_message: None,
            owned_suggestion: Some(concat!(
                "Use RFC-3339 format with UTC offset (e.g., '2026-07-03T00:00:00Z'). ",
                "Date-only and offset-less forms are rejected. ",
                "For relative time filters, use NOW() - INTERVAL 'Nh'.",
            ).to_owned()),
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // E-QUERY-042: temporal literal in structurally invalid position (ADR-052 §D4 v1.10).
        //
        // category: "validation" — the malpositioned literal IS the caller-resolvable bad input.
        // original_params_valid: false — the temporal literal in GROUP BY/ORDER BY/non-column-LHS
        //   is the invalid parameter; the caller must correct the query structure.
        // ec_code_override: None — Display starts with "E-QUERY-042:" so the inference path
        //   (`message.starts_with("E-")` → split ':' → take first part) derives "E-QUERY-042".
        //
        // Reference: error-taxonomy.md §E-QUERY-042 v2.14; ADR-052 §D4 v1.10.
        PrismError::TemporalLiteralInvalidPosition { .. } => VariantMeta {
            category: "validation",
            suggestion: "Use a column name in GROUP BY/ORDER BY, or RFC-3339 for datetime column comparisons.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: false,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        PrismError::McpParameterInvalid { .. }
        | PrismError::McpToolNotFound { .. }
        | PrismError::InvalidCapabilityPath { .. }
        | PrismError::QueryLimitExceeded { .. }
        | PrismError::UnknownSourceTable(..)
        // E-QUERY-040: SQL→Pipe redundant row limit (ADR-043). Both SQL LIMIT and
        // pipe | limit specified; caller must remove one. original_params_valid: false
        // (the combined query structure violates the FORBID-BOTH invariant).
        | PrismError::RedundantRowLimit { .. }
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
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // ── Write-policy errors: structurally valid params, policy denied ────
        // F-3 fix: these are NOT malformed-parameter errors — the params were
        // structurally valid. The write policy denied the operation (e.g., missing
        // WHERE clause, batch too large). `original_params_valid: true`.
        PrismError::WriteUnbounded
        | PrismError::WriteTargetCompositeSource { .. }
        | PrismError::WriteBatchLimitExceeded { .. }
        | PrismError::WriteTargetingInternalTable { .. }
        | PrismError::WriteVerbNotAvailable { .. }
        | PrismError::WriteTargetTableUnknown { .. }
        | PrismError::WriteAdapterNotConfiguredForClient { .. } => VariantMeta {
            category: "validation",
            suggestion: "Check the write policy constraints and retry with a bounded query.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
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
            owned_suggestion: None,
            ec_code_override: None,
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // ── Permission errors: capability denied, auth failures, org-scoping ──
        // BC-2.10.007 legal category: "permission" (not "authorization").
        // MED-1 (BC-2.10.007): each sub-class of permission error carries its own
        // suggestion text. The OBS-1 fix incorrectly shared the org-scoping string across
        // ALL permission variants. Fixed by splitting into three dedicated sub-arms:
        //   (a) SensorNotRegisteredForOrg — org-scoping guidance (the OBS-1 intent)
        //   (b) CapabilityDenied — threads the variant's own suggestion field verbatim
        //   (c) All other permission variants — generic permission/confirmation guidance

        // (a) SensorNotRegisteredForOrg: org-scoping guidance.
        // OBS-1 (BC-2.10.007): cross-org sensor isolation is a scoping/permission denial.
        // The org slug and sensor name are structurally valid; access was refused at the
        // org-scoping boundary. original_params_valid: true. LLM-agent: verify sensor is
        // registered under the target org.
        PrismError::SensorNotRegisteredForOrg { .. } => VariantMeta {
            category: "permission",
            suggestion:
                "Check sensor registration for the target org. Verify the sensor is configured \
                 under the requested org slug in prism.toml.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // (b) CapabilityDenied: thread the variant's own suggestion field verbatim.
        // MED-1 (BC-2.10.007): CapabilityDenied carries an actionable "exact TOML path
        // + restart instruction" suggestion generated by the capability resolver at check time.
        // This guidance is variant-specific and must not be discarded. owned_suggestion threads
        // it through to the structured response; suggestion is a never-used fallback.
        PrismError::CapabilityDenied { suggestion, .. } => VariantMeta {
            category: "permission",
            suggestion: "Inspect capability configuration; see audit log for details.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: Some(suggestion.clone()),
            ec_code_override: None,
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // (c) All other permission variants: generic permission/confirmation guidance.
        // FeatureFlagEvalError, Unauthorized, McpPromptInjectionDetected, token variants,
        // WriteRequiresClientId, CredentialAccessDenied, AuditTableAccessDenied.
        // None of these carry sensor-registration context; org-scoping text would actively
        // misdirect the LLM agent (e.g., for prompt-injection rejection or expired tokens).
        PrismError::FeatureFlagEvalError { .. }
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
            suggestion: "Inspect permissions and use the confirmation flow if required.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
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
            owned_suggestion: None,
            ec_code_override: None,
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // BC-2.10.007 §115: SensorRateLimited requires explicit arm binding both fields.
        // BC-2.10.007 §81: source = sensor name (not "prism_mcp").
        // BC-2.10.007 DI-006: upstream_message must be null for SensorRateLimited —
        //   the rate-limit notice is synthesized by Prism, not raw upstream text (F-5 fix).
        //   A 429 response from the upstream sensor typically has no body with specific
        //   detail to preserve; the Retry-After value is captured in retry_after_seconds.
        // BC-2.10.007 legal category: "transient" (retryable 429 → transient).
        // SEC-001 fix: apply .max(1) floor so sub-second ms values produce 1s, not 0s
        //   (prevents immediate retry storms, CWE-400).
        // SEC-002 fix: source_override carries the sensor name for audit purposes, but
        //   the message field uses the generic redacted string from map_prism_error (which
        //   now returns "Internal error" for this variant — DI-006 / CWE-200).
        PrismError::SensorRateLimited {
            sensor,
            retry_after_ms,
        } => VariantMeta {
            category: "transient",
            suggestion: "Retry after the indicated delay.",
            retryable: true,
            // SEC-001: .max(1) floor prevents 0-second retry hints for sub-second values.
            retry_after_seconds: Some((retry_after_ms / 1000).max(1)),
            original_params_valid: true,
            source_override: Some(sensor.clone()),
            // F-5 / DI-006: upstream_message must be null — Prism synthesizes the rate-limit
            // notice; there is no raw upstream body text to preserve here.
            upstream_message: None,
            // F-1: pin canonical code directly (map_prism_error returns generic message for this).
            owned_suggestion: None,
            ec_code_override: Some("E-SENSOR-020"),
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // BC-2.10.007 §81: source = sensor name; DI-006: body → upstream_message.
        // BC-2.10.007 Canonical Test Vector: 401 → category "authentication", retryable: false.
        // 403 is also an authentication/authorization failure (bad credentials or insufficient
        // scope) — same category. All other HTTP status codes remain "upstream_error".
        // F-1: pin canonical code E-SENSOR-001 directly (map_prism_error returns generic message).
        // SEC-004: cap body at 4096 bytes before embedding in upstream_message (CWE-400).
        PrismError::SensorHttpError {
            sensor,
            status,
            body,
        } => {
            let (category, suggestion) = match status {
                401 | 403 => (
                    "authentication",
                    "Check sensor credential configuration in prism.toml. See audit log.",
                ),
                _ => (
                    "upstream_error",
                    "Check sensor API status. If the problem persists, see audit log.",
                ),
            };
            // SEC-004 (CWE-400): cap upstream_message at 4096 bytes to prevent unbounded
            // allocation when sensor returns a large HTML error page or other verbose body.
            // The cap is on the FINAL string length (including the "HTTP N: " prefix).
            const UPSTREAM_MSG_CAP: usize = 4096;
            let suffix = "…[truncated]";
            let raw_body = format!("HTTP {status}: {body}");
            let capped_body = if raw_body.len() > UPSTREAM_MSG_CAP {
                // Truncate to (cap - suffix_len) to ensure final string <= cap.
                let cut = UPSTREAM_MSG_CAP.saturating_sub(suffix.len());
                // Find the last valid UTF-8 boundary at or before `cut` (rfind is idiomatic
                // for DoubleEndedIterator — avoids clippy::double_ended_iterator_last).
                let cut = raw_body
                    .char_indices()
                    .map(|(i, _)| i)
                    .rfind(|&i| i <= cut)
                    .unwrap_or(0);
                let mut truncated = raw_body[..cut].to_owned();
                truncated.push_str(suffix);
                truncated
            } else {
                raw_body
            };
            // BC-2.10.007 §RETRYABLE-503: only explicitly transient HTTP status codes are
            // retryable. Transient set: 408 (Request Timeout), 425 (Too Early),
            // 429 (Too Many Requests), 500 (Internal Server Error), 502 (Bad Gateway),
            // 503 (Service Unavailable), 504 (Gateway Timeout).
            // Permanent client errors (400/404/422/etc.) and auth failures requiring re-auth
            // (401/403) are non-retryable. Pre-existing gap: prior arm set retryable: false
            // unconditionally. Spec correction per RETRYABLE-503 adjudication v1.16
            // (coordinator-raised overbroad-rule finding).
            let retryable = matches!(status, 408 | 425 | 429 | 500 | 502 | 503 | 504);
            VariantMeta {
                category,
                suggestion,
                retryable,
                retry_after_seconds: None,
                original_params_valid: true,
                source_override: Some(sensor.clone()),
                // Raw body text → upstream_message ONLY (DI-006 injection isolation, EC-10-013).
                upstream_message: Some(capped_body),
                // F-1: pin canonical code directly (map_prism_error returns generic message).
                owned_suggestion: None,
                ec_code_override: Some("E-SENSOR-001"),
                near_text: None,
                reference_pointer: None,
                valid_operators_for_type: None,
                how_to_fix: None,
                available_columns: None,
                did_you_mean: None,
                normalized_pql: None,
            }
        }

        // BC-2.10.007 §81: source = sensor name; "upstream_error" for sensor timeouts/parse.
        // F-1: pin canonical codes E-SENSOR-002 / E-SENSOR-003 directly
        //   (map_prism_error returns "Internal error" for these variants;
        //   without the override, the fallback fires and produces "E-INT-001").
        PrismError::SensorTimeout { sensor, .. } => VariantMeta {
            category: "upstream_error",
            suggestion: "Check sensor API status. If the problem persists, see audit log.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: Some(sensor.clone()),
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: Some("E-SENSOR-002"),
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        PrismError::SensorResponseParse { sensor, .. } => VariantMeta {
            category: "upstream_error",
            suggestion: "Check sensor API status. If the problem persists, see audit log.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: Some(sensor.clone()),
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: Some("E-SENSOR-003"),
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
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
            owned_suggestion: None,
            ec_code_override: None,
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // ── Prism-side infrastructure failures → category "internal" ────────
        // BC-2.10.007 §F-4: these variants indicate a failure in Prism's own
        // runtime (disk I/O, RocksDB, internal invariant). The sensor was NEVER
        // reached. Emitting "upstream_error" for these was semantically incorrect:
        // it told LLM agents to investigate sensor health for a Prism-internal fault.
        // "internal" is the 9th legal BC-2.10.007 category value added in v1.7.
        //
        // BC-2.10.007 canonical list:
        //   Internal, Io, StorageOpenFailed, StorageWriteFailed, StorageReadFailed,
        //   StorageDomainNotFound, StorageKeyNotFound, StorageLockHeld,
        //   StorageHealthCheckFailed, SchemaMismatch, StorageBatchFailed
        PrismError::Internal { .. }
        | PrismError::Io(_)
        | PrismError::StorageOpenFailed { .. }
        | PrismError::StorageWriteFailed { .. }
        | PrismError::StorageReadFailed { .. }
        | PrismError::StorageDomainNotFound { .. }
        | PrismError::StorageKeyNotFound { .. }
        | PrismError::StorageLockHeld { .. }
        | PrismError::StorageHealthCheckFailed { .. }
        | PrismError::SchemaMismatch { .. }
        | PrismError::StorageBatchFailed { .. } => VariantMeta {
            category: "internal",
            suggestion:
                "Prism infrastructure failure. Contact Prism operator; see audit log for details.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // ── MCP serialization error → category "internal" ────────────────────
        // BC-2.10.007 OBS-002: Prism's own MCP response serialization layer
        // failed; the sensor was never involved. Fault domain is Prism-internal.
        // ec_code_override: Some("E-MCP-003") required — without it, the E-INT-001
        // fallback inference fires (map_prism_error returns "Internal error" with no
        // E- prefix, and INTERNAL_ERROR code maps to "E-INT-001" via catch-all).
        // McpSerializationError Display prefix is "E-MCP-003:" per prism-core error.rs.
        PrismError::McpSerializationError { .. } => VariantMeta {
            category: "internal",
            suggestion:
                "Prism MCP serialization failure. Contact Prism operator; see audit log for details.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: Some("E-MCP-003"),
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // ── Process-supervision watchdog failures → category "internal" ────────
        // BC-2.10.007 §OBS-2: Watchdog variants are Prism-side process supervision
        // failures. WatchdogKilled is reachable on user-visible MCP tool paths via the
        // query execution path (prism-storage::watchdog::check_query → ? propagation →
        // tool handler → prism_error_to_structured_call_result). Category "internal"
        // is correct: the fault domain is Prism's own memory budget, not a sensor failure.
        // Catch-all "upstream_error" was semantically wrong — it directed LLM agents to
        // investigate sensor health for a Prism-internal resource constraint.
        // WatchdogHeartbeatMissed and WatchdogRestartLimitExceeded share the same
        // "Prism-side process supervision failure" fault domain and are categorized
        // identically for forward compatibility.
        PrismError::WatchdogKilled { .. }
        | PrismError::WatchdogHeartbeatMissed { .. }
        | PrismError::WatchdogRestartLimitExceeded { .. } => VariantMeta {
            category: "internal",
            suggestion: "Prism process supervision failure (memory or watchdog). \
                 Contact Prism operator; see audit log for details.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // ── E-QUERY-002 type-mismatch: wire valid_operators_for_type from ColumnType ──
        //
        // BC-2.11.017 AC-003: The `valid_operators_for_type` field MUST be populated with
        // the TYPE-SPECIFIC operator set derived from the variant's `actual_type` field.
        // The error-mapping layer calls `valid_operators_for_type(actual_type)` here —
        // the same helper used by the detection gate — so the two are guaranteed to agree.
        //
        // This is the genuine fix for F-PRL-CRIT-002: type-specific operators derived from
        // the ColumnType in the error, NOT a hardcoded superset.
        //
        // ec_code_override: map_prism_error returns the variant's Display which starts
        // with "E-QUERY-002: type mismatch" — the ec_code can be inferred from the prefix.
        // ec_code_override is NOT set; the inference path (`message.starts_with("E-")`) fires.
        //
        // Reference: S-DEMO-PRISMQL-ONBOARDING-001-B; BC-2.11.017; error-taxonomy.md E-QUERY-002.
        PrismError::QueryTypeMismatch { ref actual_type, .. } => VariantMeta {
            category: "validation",
            suggestion: "Use prism_describe('<client_id>') to inspect column types and \
                         choose a valid operator from the valid_operators_for_type list.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: false,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            // TYPE-SPECIFIC operator set derived from the ColumnType carried in the error.
            // This is the load-bearing assertion for F-PRL-CRIT-002: the operators array
            // MUST match valid_operators_for_type(actual_type), not a hardcoded superset.
            valid_operators_for_type: Some(
                prism_query::engine::valid_operators_for_type(actual_type.clone())
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
        },

        // ── Query engine failures → category "internal" ─────────────────────────────
        // BC-2.10.007 §LOW-002: Six DataFusion/query-engine variants. The sensor
        // dispatch has completed (data is in MemTables) or was never relevant; the failure
        // is in Prism's own query planning/execution/materialization/virtual-field/denylist
        // layer. Category "internal" is correct. "upstream_error" (catch-all default) was
        // semantically wrong — it directed LLM agents to investigate sensor health when the
        // fault domain is Prism's own query engine.
        //
        // Prior to v1.12, QueryPlanFailed had a dedicated arm with category "validation" and
        // an analyst-facing suggestion. That was also semantically wrong: query planning
        // failures are Prism engine failures, not caller-parameter errors. The QueryTypeMismatch
        // variant (E-QUERY-002 type-mismatch subcase, added S-DEMO-PRISMQL-ONBOARDING-001-B)
        // handles the caller-actionable type-mismatch case; generic QueryPlanFailed is internal.
        //
        // ec_code_override per variant: map_prism_error returns "Internal error" for ALL six
        // (message field MUST be "Internal error" per BC-2.10.007 Rule 1; Display strings are
        // NOT used as the message). Without per-variant pins, the code inference would fall
        // through to "E-INT-001" for all. Each variant's Display DOES carry its E-QUERY-NNN /
        // E-WATCHDOG-NNN prefix, but only the ec_code_override path (not message inference)
        // can surface it given the "Internal error" redaction. A nested match provides the
        // per-variant code without duplicating the shared VariantMeta fields.
        //
        // original_params_valid: false — the caller's query triggered the engine failure in
        // all six cases. This signals to the LLM agent that reformulating the query might be
        // warranted before escalating to the operator.
        //
        // Reference: BC-2.10.007 §LOW-002; error-taxonomy.md E-QUERY-002/034/005/010/
        //            008 + E-WATCHDOG-001; F-MCPRS-PRL2-LOW-002.
        PrismError::QueryPlanFailed { .. }
        | PrismError::QueryExecutionFailed { .. }
        | PrismError::QueryMaterializationLimitExceeded { .. }
        | PrismError::QueryMemoryBudgetExceeded { .. }
        | PrismError::QueryVirtualFieldFailed { .. }
        | PrismError::QueryDenylisted { .. } => {
            let ec_code: &'static str = match &err {
                PrismError::QueryPlanFailed { .. } => "E-QUERY-002",
                PrismError::QueryExecutionFailed { .. } => "E-QUERY-034",
                PrismError::QueryMaterializationLimitExceeded { .. } => "E-QUERY-005",
                PrismError::QueryMemoryBudgetExceeded { .. } => "E-WATCHDOG-001",
                PrismError::QueryVirtualFieldFailed { .. } => "E-QUERY-010",
                PrismError::QueryDenylisted { .. } => "E-QUERY-008",
                _ => unreachable!("outer OR-pattern guarantees only the six query-engine variants"),
            };
            VariantMeta {
                category: "internal",
                suggestion: "Prism query engine failure. Contact Prism operator; see audit log for details.",
                retryable: false,
                retry_after_seconds: None,
                original_params_valid: false,
                source_override: None,
                upstream_message: None,
                owned_suggestion: None,
                ec_code_override: Some(ec_code),
                near_text: None,
                reference_pointer: None,
                valid_operators_for_type: None,
                how_to_fix: None,
                available_columns: None,
                did_you_mean: None,
                normalized_pql: None,
            }
        }

        // E-QUERY-039: EnrichUdfNotFound → "validation", original_params_valid: false.
        //
        // The caller used an enrichment function name that is not registered in the
        // `InfusionRegistry` — commonly an infusion_id (e.g. `threat_intel`) used as if
        // it were a callable per-field UDF name (e.g. `threat_score`).
        //
        // HIGH-2 fix (BC-2.11.019 §MCP surface): bind the boxed details (ref d) to
        // thread the available_infusions list into the structured suggestion text, and
        // `did_you_mean` into the `did_you_mean` field. Without this arm, `EnrichUdfNotFound`
        // falls to the catch-all with category "upstream_error", original_params_valid: true,
        // and a generic suggestion — losing all E-QUERY-039 pedagogical guidance.
        //
        // Suggestion text per BC-2.11.019 §MCP surface (NO brackets around list):
        //   non-empty: "Use one of the registered enrichment functions: {infusions}. Call
        //               prism_describe('<client_id>') to see pql_hints including available
        //               enrichment functions."
        //   empty:     "No enrichment functions are registered. Enrichment is not available
        //               in this deployment."
        //
        // NOTE: the top-level Display error message (EnrichUdfNotFoundDetails::fmt) keeps
        // brackets around the list — "available: [...]" — as that is the taxonomy template
        // format (error-taxonomy.md E-QUERY-039). Only the §MCP-surface SUGGESTION drops
        // brackets: the suggestion provides a comma-joined list for readability.
        //
        // available_infusions is inlined in the suggestion (BC does not define a separate
        // structured field for it). did_you_mean is the best-match name (Option<String>),
        // threaded into StructuredErrorFields.did_you_mean for agent self-correction.
        //
        // Explicit arm required: `PrismError` is `#[non_exhaustive]`; without this arm
        // the variant falls to the catch-all `-32000 INTERNAL_ERROR`, losing the
        // caller-actionable E-QUERY-039 guidance.
        //
        // Reference: S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B HIGH-2; BC-2.11.019;
        //            error-taxonomy.md E-QUERY-039.
        PrismError::EnrichUdfNotFound(ref d) => {
            let infusions_list = d.available_infusions.join(", ");
            let suggestion = if d.available_infusions.is_empty() {
                "No enrichment functions are registered. Enrichment is not available in this deployment.".to_owned()
            } else {
                format!(
                    "Use one of the registered enrichment functions: {infusions_list}. \
                     Call prism_describe('<client_id>') to see pql_hints including available \
                     enrichment functions."
                )
            };
            VariantMeta {
                category: "validation",
                suggestion: "Use a registered enrichment function name.",
                retryable: false,
                retry_after_seconds: None,
                original_params_valid: false,
                source_override: None,
                upstream_message: None,
                // Thread the BC-canonical suggestion verbatim via owned_suggestion
                // (takes precedence over the static `suggestion` fallback above).
                owned_suggestion: Some(suggestion),
                // Pin E-QUERY-039 directly: map_prism_error returns the Display string
                // "E-QUERY-039: enrichment infusion '...' is not registered; available: [...]"
                // which DOES start with "E-" so the code inference path would infer "E-QUERY-039"
                // correctly. ec_code_override is set explicitly for clarity + mutation resistance.
                ec_code_override: Some("E-QUERY-039"),
                near_text: None,
                reference_pointer: None,
                valid_operators_for_type: None,
                how_to_fix: None,
                available_columns: None,
                // BC-2.11.019 §EC-11-059: did_you_mean is the best-match infusion name (Option<String>).
                // Present when Levenshtein ≤ 3 of any registered InfusionField.name.
                // Omitted (not null) when None — consistent with E-QUERY-037/038 convention.
                did_you_mean: d.did_you_mean.clone(),
                normalized_pql: None,
            }
        }

        // E-QUERY-038: ColumnNotFound → "validation", original_params_valid: false.
        //
        // The caller supplied a column name that does not exist in the target table —
        // a bad parameter (the column string is malformed or mistyped). Structurally
        // valid request (table exists, client_id valid) but wrong column.
        //
        // F-PRL-CRIT-001 fix: bind the boxed details (ref d) to thread available_columns
        // and did_you_mean into the MCP structured payload. The LLM agent uses these fields
        // to self-correct the column name without a follow-up prism_describe call.
        // BC-2.11.016 §payload: available_columns ALWAYS present; did_you_mean omitted when None.
        //
        // Explicit arm required: `PrismError` is `#[non_exhaustive]`; without this arm
        // the variant falls to the catch-all with category "upstream_error" and a
        // generic suggestion, losing the `prism_describe` guidance required by BC-2.11.016.
        //
        // Reference: S-DEMO-PRISMQL-ONBOARDING-001-B; BC-2.11.016; error-taxonomy.md E-QUERY-038.
        PrismError::ColumnNotFound(ref d) => VariantMeta {
            category: "validation",
            suggestion: "Call prism_describe('<client_id>') to see available columns, \
                         or use the available_columns field in this error to correct the column name.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: false,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: Some("E-QUERY-038"),
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            // BC-2.11.016 AC-001: available_columns ALWAYS present; did_you_mean omitted when None.
            // The StructuredErrorFields::available_columns field uses #[serde(skip_serializing_if)]
            // but we always set it to Some(...) here — the array is always in the JSON payload.
            // did_you_mean uses #[serde(skip_serializing_if = "Option::is_none")] so None → absent.
            available_columns: Some(d.available_columns.clone()),
            did_you_mean: d.did_you_mean.clone(),
            normalized_pql: None,
        },

        // E-QUERY-043: IN subquery in SELECT projection, GROUP BY, or ORDER BY position.
        //
        // category: "validation" — the mispositioned IN-subquery IS the caller-resolvable
        // bad input; the analyst must rewrite the query to use WHERE clause form.
        // original_params_valid: false — projection-position IN-subquery violates
        // the DataFusion 53.1.0 execution constraint; the caller must correct the query.
        // ec_code_override: None — Display starts with "E-QUERY-043:" so the inference
        // path derives "E-QUERY-043" correctly.
        //
        // Reference: F-CSD-P4-001 adjudication 2026-07-10; error-taxonomy.md §E-QUERY-043.
        PrismError::ExprInSubqueryProjectionNotSupported { .. } => VariantMeta {
            category: "validation",
            suggestion: "Rewrite as a WHERE clause subquery: `WHERE field IN (SELECT ...)`.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: false,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // ── Safety boundary violations → category "safety" ──────────────────────
        // BC-2.10.007 §MED-001 (F-MCPRS-PRL3-MED-001): SafetyContextContamination
        // and SafetyDataExfiltration previously fell to the `_ =>` catch-all with
        // category: "upstream_error" and ec_code: "E-INT-001". This was semantically wrong:
        // these are Prism-side safety boundary detections, not upstream sensor failures.
        //
        // map_prism_error returns INTERNAL_ERROR/"Internal error" for BOTH variants per
        // BC-2.10.007 Rule 1 redaction (see map_prism_error ~lines 318-321). Code inference
        // reads the map_prism_error message ("Internal error"), not the variant Display.
        // Without ec_code_override, both fall to "E-INT-001". Per-variant ec_code_override
        // required via nested match (same pattern as §LOW-002 query engine arm above).
        //
        // original_params_valid: true — the tool call parameters were structurally valid
        // (well-formed query, valid tool invocation); the safety boundary detected malicious
        // CONTENT, not malformed SHAPE. Analogous to CapabilityDenied (category "permission",
        // original_params_valid: true). LLM-agent strategy: do not retry; report to operator.
        //
        // upstream_message: null — safety violations are detected by Prism's own safety
        // layer; no upstream sensor was contacted. DI-006: raw detection detail suppressed.
        //
        // RULE 1 INVARIANT: map_prism_error MUST continue to return "Internal error" for
        // both variants. This is CORRECT per Rule 1 redaction. The message field in the
        // structured error stays "Internal error". Only ec_code_override, category, and
        // suggestion are addressed here. Do NOT change map_prism_error for these variants.
        //
        // Reference: BC-2.10.007 §MED-001; error-taxonomy.md E-SAFETY-001/002;
        //            F-MCPRS-PRL3-MED-001.
        PrismError::SafetyContextContamination { .. }
        | PrismError::SafetyDataExfiltration { .. } => {
            let ec_code: &'static str = match &err {
                PrismError::SafetyContextContamination { .. } => "E-SAFETY-001",
                PrismError::SafetyDataExfiltration { .. } => "E-SAFETY-002",
                _ => unreachable!("outer OR-pattern guarantees only the two safety variants"),
            };
            VariantMeta {
                category: "safety",
                suggestion: "Do not retry; report to operator.",
                retryable: false,
                retry_after_seconds: None,
                original_params_valid: true,
                source_override: None,
                upstream_message: None,
                owned_suggestion: None,
                ec_code_override: Some(ec_code),
                near_text: None,
                reference_pointer: None,
                valid_operators_for_type: None,
                how_to_fix: None,
                available_columns: None,
                did_you_mean: None,
                normalized_pql: None,
            }
        }

        // ── F-MCPRS-PRL10-OBS-003: 28 explicit arms for variants previously falling ──
        // to the catch-all. Four groups: internal (12), configuration (3), validation (3),
        // upstream_error explicit (10). The catch-all below is retained for the
        // #[non_exhaustive] compiler requirement (future variants only).
        //
        // map_prism_error returns "Internal error" for all 28 variants, so ec_code_override
        // is left None (all produce "E-INT-001" from the code-inference fallback path).
        // The behaviour change here is category + suggestion only.

        // ── Group 1: internal Prism framework failures ────────────────────────────
        // These represent Prism subsystem failures an operator must investigate via
        // audit log — enrichment, WASM plugin, OCSF protobuf, credential backend,
        // scheduling, detection engine, and case management.
        PrismError::Infusion(_) => VariantMeta {
            category: "internal",
            suggestion: "Prism enrichment framework failure. Contact Prism operator; see audit log.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        PrismError::Plugin(_) => VariantMeta {
            category: "internal",
            suggestion: "Prism plugin framework failure. Contact Prism operator; see audit log.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // E-OCSF-010, E-OCSF-011, E-OCSF-022: OCSF protobuf encode/decode/descriptor
        // failures indicate internal Prism serialization layer failures, not upstream
        // sensor data problems.
        PrismError::OcsfProtobufEncode { .. }
        | PrismError::OcsfProtobufDecode { .. }
        | PrismError::OcsfDescriptorNotFound { .. } => VariantMeta {
            category: "internal",
            suggestion: "Prism OCSF protobuf encoding failure. Contact Prism operator; see audit log.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // E-CRED-004, E-CRED-006: credential store/encryption backend failures;
        // these indicate the credential store infrastructure itself failed, not
        // the credential configuration (those are Group 2 "configuration").
        PrismError::CredentialStoreError { .. }
        | PrismError::CredentialEncryptionError { .. } => VariantMeta {
            category: "internal",
            suggestion: "Prism credential backend failure. Contact Prism operator; see audit log.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // E-SCHED-001, E-DET-001/002/010, E-CASE-001: internal state failures for
        // scheduling, detection engine, and case management subsystems.
        PrismError::ScheduleNotFound { .. }
        | PrismError::DetectionRuleParseFailed { .. }
        | PrismError::DetectionRuleNotFound { .. }
        | PrismError::DetectionStateCorrupt { .. }
        | PrismError::CaseNotFound { .. } => VariantMeta {
            category: "internal",
            suggestion: "Prism internal state error. Contact Prism operator; see audit log.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // ── Group 2: configuration errors ─────────────────────────────────────────
        // Credential name validation, lookup failures, and missing encryption key.
        // original_params_valid: false — the credential parameters themselves are wrong.
        // source_override: "prism_config" — operator should check prism.toml, not upstream.
        PrismError::InvalidCredentialName { .. }
        | PrismError::CredentialNotFound { .. }
        | PrismError::EncryptionKeyMissing { .. } => VariantMeta {
            category: "configuration",
            suggestion: "Check credential configuration in prism.toml.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: false,
            source_override: Some("prism_config".to_owned()),
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // ── Group 3: validation errors ─────────────────────────────────────────────
        // Schedule conflicts/cron-syntax errors and case state-transition violations.
        // original_params_valid: false — the caller supplied an invalid parameter value.
        PrismError::ScheduleConflict { .. }
        | PrismError::ScheduleCronInvalid { .. }
        | PrismError::CaseStateTransitionInvalid { .. } => VariantMeta {
            category: "validation",
            suggestion: "Fix the invalid parameter and retry.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: false,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // ── Group 4: upstream_error (explicit, semantics unchanged) ───────────────
        // These variants previously fell to the catch-all "upstream_error". Now explicit
        // to document intent and enable future per-variant tuning without catching future
        // variants by accident.

        // E-OCSF-001/002/003/020/023/024/021: OCSF data-shape problems indicate the
        // upstream sensor returned data that doesn't conform to the expected OCSF schema.
        // These are sensor-side problems, not Prism internals (contrast Group 1 protobuf).
        PrismError::OcsfFieldMissing { .. }
        | PrismError::OcsfFieldTypeMismatch { .. }
        | PrismError::OcsfUnknownClassUid { .. }
        | PrismError::OcsfUnknownEventClass { .. }
        | PrismError::OcsfUnknownRecordType { .. }
        | PrismError::OcsfTimestampParseError { .. }
        | PrismError::OcsfNormalizationFailed { .. } => VariantMeta {
            category: "upstream_error",
            suggestion: "Check sensor API status. If the problem persists, see audit log.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // E-QUERY-025: partial write failure — some records were not written to the
        // sensor endpoint; check sensor API status.
        PrismError::WritePartialFailure { .. } => VariantMeta {
            category: "upstream_error",
            suggestion: "Check sensor API status. If the problem persists, see audit log.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // E-IOC-001, E-IOC-002: IOC feed parse failures and lookup failures are
        // upstream-source problems.
        PrismError::IocFeedParseFailed { .. }
        | PrismError::IocLookupFailed { .. } => VariantMeta {
            category: "upstream_error",
            suggestion: "Check IOC feed source and retry.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },

        // ── Catch-all: unknown variants → "upstream_error" (legal BC category) ──
        // "upstream_error" is the safest legal fallback for variants that don't fit
        // the specific categories above (non_exhaustive catch-all).
        _ => VariantMeta {
            category: "upstream_error",
            suggestion: "See audit log for details.",
            retryable: false,
            retry_after_seconds: None,
            original_params_valid: true,
            source_override: None,
            upstream_message: None,
            owned_suggestion: None,
            ec_code_override: None,
            near_text: None,
            reference_pointer: None,
            valid_operators_for_type: None,
            how_to_fix: None,
            available_columns: None,
            did_you_mean: None,
            normalized_pql: None,
        },
    };

    // Now consume err to get the canonical code + message.
    let (code_i32, message) = map_prism_error(err);
    // Derive E-* code string.
    // F-1 fix: if the variant pinned an explicit ec_code_override, use it directly.
    // This is required for variants where map_prism_error returns the generic
    // "Internal error" message (no E- prefix to infer the code from).
    // Without the override, the fallback "E-INT-001" fires incorrectly for
    // SensorHttpError (should be E-SENSOR-001), SensorTimeout (E-SENSOR-002), etc.
    let ec_code = if let Some(pinned_code) = meta.ec_code_override {
        pinned_code.to_owned()
    } else if message.starts_with("E-") {
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
        // MED-1 (BC-2.10.007): use owned_suggestion when the variant carries its own
        // actionable guidance (e.g. CapabilityDenied.suggestion); fall back to static string.
        suggestion: meta
            .owned_suggestion
            .unwrap_or_else(|| meta.suggestion.to_owned()),
        source,
        original_params_valid: meta.original_params_valid,
        upstream_message: meta.upstream_message,
        // BC-2.11.017 AC-003: near_text and reference_pointer come from VariantMeta.
        near_text: meta.near_text,
        reference_pointer: meta.reference_pointer.map(str::to_owned),
        // BC-2.11.017 AC-003: valid_operators_for_type and how_to_fix come from VariantMeta.
        valid_operators_for_type: meta.valid_operators_for_type,
        how_to_fix: meta.how_to_fix,
        // BC-2.11.016 AC-001: available_columns and did_you_mean come from VariantMeta.
        // Only Some for ColumnNotFound; None for all other variants (absent from JSON).
        available_columns: meta.available_columns,
        did_you_mean: meta.did_you_mean,
        // ADR-046 BC-2.11.023 AC-010: normalized_pql comes from VariantMeta.
        // Only Some for mode-bridge D1 errors; None for all other variants (absent from JSON).
        normalized_pql: meta.normalized_pql,
    };
    let content_text = format!(
        "ERROR: [{}] - {}. {}",
        fields.category, fields.message, fields.suggestion
    );
    build_structured_error_response(fields, content_text)
}

// ---------------------------------------------------------------------------
// S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001: map_prism_error_to_structured
// ---------------------------------------------------------------------------

/// Map a `PrismError` to `StructuredErrorFields`, including the `normalized_pql`
/// field for D1 mode-bridge errors (BC-2.11.023 AC-010, ADR-046 §D1).
///
/// For `PrismError::QueryParseFailed` with a D1 mode-bridge error in the query string,
/// the returned `StructuredErrorFields.normalized_pql` is `Some(rewrite)` where
/// `rewrite` is the best-effort pipe-mode rewrite of the SQL query.
///
/// **D1 detection and rewrite algorithm (ADR-046 §D1):**
/// When the error detail contains the mode-bridge marker (E-QUERY-001 + pipe-stage
/// diagnostic text), or when the original query starts with SELECT and contains an
/// unquoted `|`, we attempt to produce a pipe-mode rewrite:
///
/// 1. Try to re-parse the original query via `PrismQlParser::parse`. If it now succeeds
///    (e.g., as `Ast::SqlPipe` after the SqlPipe grammar landed in BC-2.11.020),
///    call `prism_query::engine::normalize_pql` to get the canonical normalized form.
/// 2. If re-parse fails (genuine D1 with non-stage keyword after `|`), attempt a
///    best-effort string heuristic for simple `SELECT * FROM t WHERE … | …` patterns:
///    - Extract: table from `FROM <table>`, predicate from `WHERE <predicate>`, stages after `|`
///    - Reassemble as `FROM <table> | where <predicate> | <stages>`
///    - Re-parse the reassembled string to verify round-trip; set `normalized_pql` to `None`
///      if the rewrite itself fails to parse (BC-2.11.023 postcondition — must be valid PrismQL).
///
/// For all other errors, `normalized_pql` is `None`.
///
/// `original_query` is the query string as submitted to the parser.
pub fn map_prism_error_to_structured(
    err: &prism_core::error::PrismError,
    original_query: &str,
) -> StructuredErrorFields {
    use prism_core::error::PrismError;
    use prism_query::PrismQlParser;

    // Compute normalized_pql for QueryParseFailed on mode-bridge errors.
    // Delegates to canonical implementation in prism_query::error_recovery (OBS-1).
    let normalized_pql = if matches!(err, PrismError::QueryParseFailed { .. }) {
        prism_query::error_recovery::mode_bridge_normalized_pql(original_query)
    } else {
        None
    };

    // Build the StructuredErrorFields.
    // Derive code and message directly from the error variant (no clone needed).
    let (code, message) = match err {
        PrismError::QueryParseFailed { detail, .. } => (
            "E-QUERY-001".to_string(),
            format!("PrismQL parse error: {detail}"),
        ),
        PrismError::QueryTimeout { .. } => (
            "E-QUERY-004".to_string(),
            "Query timeout exceeded".to_string(),
        ),
        _ => ("E-QUERY-001".to_string(), format!("{err}")),
    };

    let near_text = if matches!(err, PrismError::QueryParseFailed { .. }) {
        // Compute near_text at offset 0 (conservative; the full offset computation
        // is in prism_error_to_structured_call_result which is the production MCP path).
        Some(prism_query::engine::extract_near_text(original_query, 0))
    } else {
        None
    };

    let reference_pointer = if matches!(err, PrismError::QueryParseFailed { .. }) {
        Some("prismql://reference".to_string())
    } else {
        None
    };

    let mut fields = StructuredErrorFields::new(
        code,
        message,
        "validation",
        false,
        None,
        "Check the PrismQL reference at prismql://reference for the correct syntax.",
        "prism_mcp",
        false,
        None,
    );
    // Inject the fields that StructuredErrorFields::new sets to None.
    // These are `pub` fields on the `#[non_exhaustive]` struct; direct assignment
    // is allowed within the same crate.
    fields.near_text = near_text;
    fields.reference_pointer = reference_pointer.map(|s| s.to_string());
    fields.normalized_pql = normalized_pql;
    fields
}

// mode_bridge_normalized_pql and find_first_unquoted_pipe have been relocated to
// prism_query::error_recovery per BC-2.11.023 Architecture Anchors and the story
// File Structure (OBS-1). This file now delegates to that canonical location.
// See: crates/prism-query/src/error_recovery.rs

// ---------------------------------------------------------------------------
// Unit tests for error_mapping
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::{PrismError, UnknownSourceTableDetails};

    /// P6-02: UnknownSourceTable (E-QUERY-036) must map to -32602 INVALID_PARAMS.
    ///
    /// EXPLICIT arm required: `PrismError` is `#[non_exhaustive]`; without the
    /// explicit arm the variant would fall through to the catch-all `-32000`
    /// INTERNAL_ERROR, losing the caller-actionable E-QUERY-036 guidance.
    #[test]
    fn test_unknown_source_table_maps_to_invalid_params() {
        let err = PrismError::UnknownSourceTable(Box::new(UnknownSourceTableDetails::new(
            "ghost_sensor.table",
            vec!["crowdstrike".to_string()],
            Some("crowdstrike".to_string()),
        )));
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
        let err = PrismError::UnknownSourceTable(Box::new(UnknownSourceTableDetails::new(
            "unknown.devices",
            vec![],
            None,
        )));
        let (code, _) = map_prism_error(err);
        assert_ne!(
            code,
            codes::INTERNAL_ERROR,
            "UnknownSourceTable must NOT map to INTERNAL_ERROR (-32000); got: {code}"
        );
    }

    // ── BC-2.10.007 Canonical Test Vectors — SensorHttpError auth mis-categorization ──

    /// BC-2.10.007 Canonical Test Vector: Sensor API returns 401 → category "authentication".
    ///
    /// Verifies that `SensorHttpError { status: 401 }` produces `category: "authentication"`,
    /// `retryable: false` in `prism_error_to_structured_call_result`. Before this fix the arm
    /// was unconditional `"upstream_error"`, causing analysts to see a misleading "outage/retry"
    /// signal for a credential failure.
    #[test]
    fn test_BC_2_10_007_sensor_http_401_category_is_authentication() {
        let err = PrismError::SensorHttpError {
            sensor: "crowdstrike_falcon_api".to_owned(),
            status: 401,
            body: "Unauthorized".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        // Extract structuredContent.error from the CallToolResult.
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "authentication",
            "SensorHttpError{{status:401}} must map to category 'authentication' (BC-2.10.007 Canonical Test Vector); got '{category}'"
        );
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("structuredContent.error.retryable must be a bool");
        assert!(
            !retryable,
            "SensorHttpError{{status:401}} must be retryable:false (auth failures are not transient)"
        );
        // upstream_message must still carry the raw body per DI-006.
        let upstream_message = error_obj
            .get("upstream_message")
            .expect("structuredContent.error.upstream_message must be present (null-not-absent)");
        assert!(
            upstream_message.is_string(),
            "upstream_message must be a string (not null) for SensorHttpError; got: {upstream_message:?}"
        );
        assert!(
            upstream_message.as_str().unwrap().contains("401"),
            "upstream_message must include the HTTP status; got: {upstream_message}"
        );
    }

    /// BC-2.10.007: SensorHttpError { status: 403 } → category "authentication".
    ///
    /// 403 is Forbidden / insufficient scope — a credential/auth failure, not an upstream
    /// service outage. Analysts must receive the same "authentication" signal as 401.
    #[test]
    fn test_BC_2_10_007_sensor_http_403_category_is_authentication() {
        let err = PrismError::SensorHttpError {
            sensor: "armis_api".to_owned(),
            status: 403,
            body: "Forbidden: insufficient scope".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "authentication",
            "SensorHttpError{{status:403}} must map to category 'authentication' (BC-2.10.007); got '{category}'"
        );
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            !retryable,
            "SensorHttpError{{status:403}} must be retryable:false"
        );
    }

    /// BC-2.10.007 control case: SensorHttpError { status: 502 } → category "upstream_error".
    ///
    /// Ensures the 401/403 branch is correctly scoped — other HTTP errors must NOT be
    /// re-categorized as "authentication". 502 Bad Gateway is a genuine upstream outage.
    #[test]
    fn test_BC_2_10_007_sensor_http_502_category_is_upstream_error() {
        let err = PrismError::SensorHttpError {
            sensor: "claroty_api".to_owned(),
            status: 502,
            body: "Bad Gateway".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "upstream_error",
            "SensorHttpError{{status:502}} must remain 'upstream_error' (not 'authentication'); got '{category}'"
        );
    }

    // ── BC-2.10.007 Canonical Test Vectors — "internal" category for Prism infra failures ──

    /// BC-2.10.007 Test Vector: PrismError::Internal → category "internal".
    ///
    /// Before v1.7 the F-4 arm used "upstream_error" as a fallback, which told LLM
    /// agents to investigate sensor health for a Prism-side invariant failure. The
    /// sensor was never reached — Prism itself failed. "internal" is the correct
    /// semantic category per the v1.7 category decision rule table.
    #[test]
    fn test_BC_2_10_007_v1_7_internal_category_is_internal() {
        let err = PrismError::Internal {
            detail: "invariant violated in test".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "internal",
            "PrismError::Internal must map to category 'internal' (BC-2.10.007 F-4); got '{category}'"
        );
        // retryable must be false — Prism invariant failures are not transient.
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            !retryable,
            "PrismError::Internal must be retryable:false (invariant violation is not transient)"
        );
        // upstream_message must be null — no sensor was involved (DI-006).
        let upstream_message = error_obj
            .get("upstream_message")
            .expect("upstream_message must be present (null-not-absent invariant)");
        assert!(
            upstream_message.is_null(),
            "PrismError::Internal upstream_message must be null (sensor not reached); got: {upstream_message:?}"
        );
    }

    /// BC-2.10.007 Test Vector: PrismError::Io → category "internal".
    ///
    /// Prism I/O failure (disk, file system). The sensor was never reached.
    /// Before v1.7 this fell through to "upstream_error", which was semantically wrong.
    #[test]
    fn test_BC_2_10_007_v1_7_io_category_is_internal() {
        let err = PrismError::Io("disk read error in test".to_owned());
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "internal",
            "PrismError::Io must map to category 'internal' (BC-2.10.007 F-4); got '{category}'"
        );
        let upstream_message = error_obj
            .get("upstream_message")
            .expect("upstream_message must be present (null-not-absent invariant)");
        assert!(
            upstream_message.is_null(),
            "PrismError::Io upstream_message must be null (sensor not reached); got: {upstream_message:?}"
        );
    }

    /// BC-2.10.007 Test Vector: PrismError::StorageWriteFailed → category "internal".
    ///
    /// RocksDB / storage layer failure. The sensor was never reached.
    /// Before v1.7 this fell through to "upstream_error", which was semantically wrong.
    #[test]
    fn test_BC_2_10_007_v1_7_storage_write_failed_category_is_internal() {
        let err = PrismError::StorageWriteFailed {
            domain: "audit".to_owned(),
            detail: "RocksDB write error in test".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "internal",
            "PrismError::StorageWriteFailed must map to category 'internal' (BC-2.10.007 F-4); got '{category}'"
        );
        let upstream_message = error_obj
            .get("upstream_message")
            .expect("upstream_message must be present (null-not-absent invariant)");
        assert!(
            upstream_message.is_null(),
            "PrismError::StorageWriteFailed upstream_message must be null (sensor not reached); got: {upstream_message:?}"
        );
    }

    /// BC-2.10.007 Regression guard: PrismError::SensorHttpError → category "upstream_error".
    ///
    /// The F-4 fix MUST NOT change the "upstream_error" category for genuine sensor
    /// boundary failures. SensorHttpError (non-auth) remains "upstream_error".
    /// This test guards against an over-broad fix that reclassifies sensor errors as "internal".
    #[test]
    fn test_BC_2_10_007_v1_7_sensor_http_503_category_is_upstream_error_regression_guard() {
        let err = PrismError::SensorHttpError {
            sensor: "cyberint_api".to_owned(),
            status: 503,
            body: "Service Unavailable".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "upstream_error",
            "PrismError::SensorHttpError (non-auth) must remain 'upstream_error' — NOT 'internal' (BC-2.10.007 regression guard); got '{category}'"
        );
    }

    // ── BC-2.10.007 §RETRYABLE-503: SensorHttpError transient-only retryable whitelist ──

    /// BC-2.10.007 §RETRYABLE-503 — PRIMARY: SensorHttpError { status: 503 } → retryable: true.
    ///
    /// HTTP 503 Service Unavailable is an explicitly transient condition — the upstream
    /// sensor is temporarily unavailable and the LLM agent MAY retry after delay.
    /// Prior to v1.16 the arm set `retryable: false` unconditionally, causing agents to
    /// treat a transient sensor outage as a permanent failure (wasted analyst triage).
    ///
    /// Transient whitelist (BC-2.10.007 §RETRYABLE-503):
    ///   408 (Request Timeout) | 425 (Too Early) | 429 (Too Many Requests) |
    ///   500 (Internal Server Error) | 502 (Bad Gateway) | 503 (Service Unavailable) |
    ///   504 (Gateway Timeout).
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_sensor_http_error_503_retryable_is_true() {
        let err = PrismError::SensorHttpError {
            sensor: "crowdstrike_falcon_api".to_owned(),
            status: 503,
            body: "Service Unavailable".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        // Category unchanged — 503 remains "upstream_error" (sensor boundary, not Prism internal).
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "upstream_error",
            "SensorHttpError{{status:503}} must remain 'upstream_error' (BC-2.10.007 §RETRYABLE-503); got '{category}'"
        );
        // PRIMARY assertion: retryable must be true for a transient HTTP status.
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("structuredContent.error.retryable must be a bool");
        assert!(
            retryable,
            "SensorHttpError{{status:503}} must be retryable:true (BC-2.10.007 §RETRYABLE-503 — HTTP 503 is transient)"
        );
    }

    /// BC-2.10.007 §RETRYABLE-503 — COMPANION (transient): SensorHttpError { status: 429 } → retryable: true.
    ///
    /// HTTP 429 Too Many Requests is a rate-limit transient. The dedicated SensorRateLimited
    /// variant handles structured 429 with retry_after_seconds; this test covers the
    /// `SensorHttpError { status: 429 }` path for sensors that don't trigger the rate-limit
    /// variant. Both paths must produce retryable: true per v1.16.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_sensor_http_error_429_retryable_is_true() {
        let err = PrismError::SensorHttpError {
            sensor: "armis_cloud_api".to_owned(),
            status: 429,
            body: "Too Many Requests".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("structuredContent.error.retryable must be a bool");
        assert!(
            retryable,
            "SensorHttpError{{status:429}} must be retryable:true (BC-2.10.007 §RETRYABLE-503 — HTTP 429 is transient rate-limit)"
        );
    }

    /// BC-2.10.007 §RETRYABLE-503 — COMPANION (permanent): SensorHttpError { status: 404 } → retryable: false.
    ///
    /// HTTP 404 Not Found is a permanent client error. Retrying will not fix a missing
    /// resource. The v1.16 transient whitelist explicitly excludes 404 — marking it
    /// retryable would waste LLM-agent cycles re-querying a permanently absent endpoint.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_sensor_http_error_404_retryable_is_false() {
        let err = PrismError::SensorHttpError {
            sensor: "claroty_api".to_owned(),
            status: 404,
            body: "Not Found".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("structuredContent.error.retryable must be a bool");
        assert!(
            !retryable,
            "SensorHttpError{{status:404}} must be retryable:false (BC-2.10.007 §RETRYABLE-503 — HTTP 404 is permanent)"
        );
    }

    // ── BC-2.10.007 OBS-1: SensorNotRegisteredForOrg → category "permission" ──

    /// BC-2.10.007 OBS-1: SensorNotRegisteredForOrg maps to category "permission",
    /// original_params_valid: true (BC-2.10.007 §OBS-1 adjudication).
    ///
    /// Cross-org sensor isolation is a scoping/permission denial, NOT a parameter
    /// validation failure. The org slug and sensor name are structurally valid; access
    /// was refused at the org-scoping boundary. original_params_valid: true because the
    /// parameters were correct — the sensor is not registered under that org.
    ///
    /// JSON-RPC code (-32602) and error code (E-QUERY-032) are unchanged.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_sensor_not_registered_for_org_category_is_permission() {
        let err = PrismError::SensorNotRegisteredForOrg {
            sensor_id: "claroty".to_owned(),
            org_slug: "demo-org-a".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        // OBS-1: category must be "permission" (not "validation").
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "permission",
            "SensorNotRegisteredForOrg must map to category 'permission' (BC-2.10.007 OBS-1); got '{category}'"
        );

        // OBS-1: original_params_valid must be true.
        let original_params_valid = error_obj
            .get("original_params_valid")
            .and_then(|v| v.as_bool())
            .expect("structuredContent.error.original_params_valid must be a bool");
        assert!(
            original_params_valid,
            "SensorNotRegisteredForOrg must have original_params_valid:true (params were structurally \
             correct; access was refused at org-scoping boundary)"
        );

        // retryable must be false — org-scoping errors are not transient.
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            !retryable,
            "SensorNotRegisteredForOrg must be retryable:false"
        );

        // Error code must still be E-QUERY-032 (unchanged per OBS-1 adjudication).
        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.code must be a string");
        assert!(
            code.contains("E-QUERY-032"),
            "SensorNotRegisteredForOrg error code must contain 'E-QUERY-032' (unchanged by OBS-1); got '{code}'"
        );

        // upstream_message must be null — no sensor was reached (DI-006).
        let upstream_message = error_obj
            .get("upstream_message")
            .expect("upstream_message must be present (null-not-absent invariant)");
        assert!(
            upstream_message.is_null(),
            "SensorNotRegisteredForOrg upstream_message must be null (Prism-originating error, DI-006); got: {upstream_message:?}"
        );
    }

    // ── BC-2.10.007 OBS-2: Watchdog* → category "internal" ──

    /// BC-2.10.007 OBS-2: WatchdogKilled maps to category "internal",
    /// original_params_valid: true, retryable: false, upstream_message: null.
    ///
    /// WatchdogKilled is a Prism-side process supervision failure (memory budget exceeded).
    /// The query was killed by Prism's own watchdog — the sensor was never reached.
    /// "internal" is correct; "upstream_error" was semantically wrong (it directed
    /// LLM agents to investigate sensor health for a Prism-internal resource constraint).
    ///
    /// WatchdogKilled is reachable on user-visible MCP tool paths via:
    /// prism-storage::watchdog::check_query → ? propagation → tool handler →
    /// prism_error_to_structured_call_result.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_watchdog_killed_category_is_internal() {
        let err = PrismError::WatchdogKilled {
            budget_bytes: 512_000_000,
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        // OBS-2: category must be "internal".
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "internal",
            "WatchdogKilled must map to category 'internal' (BC-2.10.007 OBS-2); got '{category}'"
        );

        // retryable must be false — watchdog termination is not transient.
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            !retryable,
            "WatchdogKilled must be retryable:false (watchdog termination is not a transient condition)"
        );

        // upstream_message must be null — no sensor was reached (DI-006).
        let upstream_message = error_obj
            .get("upstream_message")
            .expect("upstream_message must be present (null-not-absent invariant)");
        assert!(
            upstream_message.is_null(),
            "WatchdogKilled upstream_message must be null (sensor not reached); got: {upstream_message:?}"
        );
    }

    /// BC-2.10.007 OBS-2: WatchdogHeartbeatMissed maps to category "internal",
    /// retryable: false, upstream_message: null.
    ///
    /// WatchdogHeartbeatMissed shares the explicit `|` arm with WatchdogKilled and
    /// WatchdogRestartLimitExceeded. This test closes the TD-VSDD-059 mutation-coverage hole:
    /// without it, a mutation dropping HeartbeatMissed from the arm would pass undetected.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_heartbeat_missed_maps_to_internal_category() {
        let err = PrismError::WatchdogHeartbeatMissed {
            component: "test-component".to_owned(),
            elapsed_ms: 5_000,
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        // OBS-2: category must be "internal".
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "internal",
            "WatchdogHeartbeatMissed must map to category 'internal' (BC-2.10.007 OBS-2); got '{category}'"
        );

        // retryable must be false — missed heartbeat is not a transient sensor condition.
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            !retryable,
            "WatchdogHeartbeatMissed must be retryable:false (watchdog supervision failure is not transient)"
        );

        // upstream_message must be null — no sensor was reached (DI-006).
        let upstream_message = error_obj
            .get("upstream_message")
            .expect("upstream_message must be present (null-not-absent invariant)");
        assert!(
            upstream_message.is_null(),
            "WatchdogHeartbeatMissed upstream_message must be null (sensor not reached); got: {upstream_message:?}"
        );
    }

    /// BC-2.10.007 OBS-2: WatchdogRestartLimitExceeded maps to category "internal",
    /// retryable: false, upstream_message: null.
    ///
    /// WatchdogRestartLimitExceeded shares the explicit `|` arm with WatchdogKilled and
    /// WatchdogHeartbeatMissed. This test closes the TD-VSDD-059 mutation-coverage hole:
    /// without it, a mutation dropping RestartLimitExceeded from the arm would pass undetected.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_restart_limit_exceeded_maps_to_internal_category() {
        let err = PrismError::WatchdogRestartLimitExceeded {
            component: "test-component".to_owned(),
            count: 3,
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        // OBS-2: category must be "internal".
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "internal",
            "WatchdogRestartLimitExceeded must map to category 'internal' (BC-2.10.007 OBS-2); got '{category}'"
        );

        // retryable must be false — restart limit exceeded is not a transient sensor condition.
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            !retryable,
            "WatchdogRestartLimitExceeded must be retryable:false (restart limit exceeded is not transient)"
        );

        // upstream_message must be null — no sensor was reached (DI-006).
        let upstream_message = error_obj
            .get("upstream_message")
            .expect("upstream_message must be present (null-not-absent invariant)");
        assert!(
            upstream_message.is_null(),
            "WatchdogRestartLimitExceeded upstream_message must be null (sensor not reached); got: {upstream_message:?}"
        );
    }

    // ── BC-2.10.007 MED-1: suggestion text correctness per-variant ──

    /// MED-1 (BC-2.10.007): SensorNotRegisteredForOrg suggestion must contain
    /// org-scoping guidance (keyword "org") — e.g., "Check sensor registration for the
    /// target org; verify the sensor is configured under the requested org slug in
    /// prism.toml."
    ///
    /// Before MED-1 this was correct, but the fix was shared with ALL permission variants.
    /// After MED-1 this must be in a DEDICATED sub-arm, not a shared one.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_sensor_not_registered_for_org_suggestion_contains_org_scoping() {
        let err = PrismError::SensorNotRegisteredForOrg {
            sensor_id: "crowdstrike".to_owned(),
            org_slug: "acme-corp".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.suggestion must be a string");
        assert!(
            suggestion.contains("org"),
            "SensorNotRegisteredForOrg suggestion must contain org-scoping guidance ('org'); \
             got '{suggestion}'"
        );
    }

    /// MED-1 (BC-2.10.007): McpPromptInjectionDetected suggestion must NOT contain
    /// the org-scoping text "Check sensor registration for the target org". Before MED-1
    /// the shared permission arm prepended the org-scoping string to ALL permission variants,
    /// actively misdirecting the LLM agent for injection rejections.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_mcp_prompt_injection_suggestion_does_not_contain_org_scoping_text() {
        let err = PrismError::McpPromptInjectionDetected {
            tool: "prism_query".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.suggestion must be a string");
        assert!(
            !suggestion.contains("sensor registration for the target org"),
            "McpPromptInjectionDetected suggestion must NOT contain org-scoping text \
             (MED-1: different variants need different suggestions); got '{suggestion}'"
        );
    }

    /// MED-1 (BC-2.10.007): CapabilityDenied must thread its own `suggestion` field
    /// through to the structured error response. Before MED-1 the shared permission arm
    /// discarded CapabilityDenied.suggestion in favour of the static org-scoping string.
    /// CapabilityDenied carries an actionable "exact TOML path + restart instruction"
    /// suggestion generated by the capability resolver — it must not be silently discarded.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_capability_denied_suggestion_threads_own_suggestion_field() {
        let err = PrismError::CapabilityDenied {
            capability: "sensor.crowdstrike.containment".to_owned(),
            client_id: "test-client".to_owned(),
            reason: "compile-time disabled".to_owned(),
            suggestion: "Enable sensor.crowdstrike.containment = true in prism.toml and rebuild."
                .to_owned(),
            resolution_trace: vec!["root=disabled".to_owned()],
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.suggestion must be a string");
        assert!(
            suggestion.contains("sensor.crowdstrike.containment"),
            "CapabilityDenied suggestion must thread the variant's own suggestion field through \
             (MED-1); got '{suggestion}'"
        );
        assert!(
            !suggestion.contains("sensor registration for the target org"),
            "CapabilityDenied suggestion must NOT contain org-scoping text (MED-1); got '{suggestion}'"
        );
    }

    // ── BC-2.10.007 explicit arm: Infusion → "internal" (F-MCPRS-PRL10-OBS-003) ──

    /// BC-2.10.007 explicit arm: `PrismError::Infusion` maps to category `"internal"`.
    ///
    /// F-MCPRS-PRL10-OBS-003 added a Group 1 explicit arm for `Infusion` in
    /// `prism_error_to_structured_call_result`. This test locks that behaviour.
    ///
    /// The catch-all `_ =>` arm remains in place for `#[non_exhaustive]` compliance —
    /// it covers any future variants added to `PrismError` that do not yet have an
    /// explicit arm. `Infusion` is no longer one of those variants.
    #[test]
    fn test_CRIT_B_infusion_error_maps_to_internal_category() {
        let err = PrismError::Infusion(prism_core::error::InfusionError::UnknownInfusion {
            name: "test_catch_all_enrichment".to_owned(),
        });
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "internal",
            "PrismError::Infusion must map to 'internal' via explicit Group 1 arm \
             (F-MCPRS-PRL10-OBS-003); got '{category}'"
        );
    }

    /// OBS-1 de-footgun: `to_error_data_with_retry` must NOT panic for non-SensorRateLimited
    /// variants (load-bearing regression guard).
    ///
    /// BC-2.10.007 §111 specifies graceful "return None for all other variants" semantics.
    /// The prior implementation `panic!`'d, making the public function a latent footgun.
    /// Non-SensorRateLimited variants return `retry_after_ms = 0` gracefully (no panic).
    ///
    /// Uses `PrismError::QueryTimeout` as a representative non-SensorRateLimited variant.
    #[test]
    fn test_to_error_data_with_retry_non_rate_limited_does_not_panic() {
        let err = PrismError::QueryTimeout { elapsed_ms: 30_000 };
        // Must NOT panic — graceful return with retry_after_ms = 0 (no retry hint).
        let (_error_data, retry_after_ms) = to_error_data_with_retry(err);
        assert_eq!(
            retry_after_ms, 0u64,
            "OBS-1: non-SensorRateLimited variant must return retry_after_ms=0 (graceful, \
             no panic); got {retry_after_ms}"
        );
    }

    /// OBS-1 companion: `to_error_data_with_retry` with `SensorRateLimited` still works
    /// correctly after the de-footgun refactor (regression guard on the happy path).
    #[test]
    fn test_to_error_data_with_retry_rate_limited_still_correct_after_obs1_fix() {
        let err = PrismError::SensorRateLimited {
            sensor: "armis".to_owned(),
            retry_after_ms: 60_000,
        };
        let (_error_data, retry_after_ms) = to_error_data_with_retry(err);
        assert_eq!(
            retry_after_ms, 60_000u64,
            "OBS-1 regression guard: SensorRateLimited path must still return retry_after_ms \
             unchanged after de-footgun fix; got {retry_after_ms}"
        );
    }

    // -----------------------------------------------------------------------
    // S-3.13 Red Gate tests: TableNotAvailable (E-QUERY-037) → -32602
    // -----------------------------------------------------------------------

    /// S-3.13 / BC-2.11.001 / AC-2: `PrismError::TableNotAvailable` MUST map to
    /// -32602 (INVALID_PARAMS), not the catch-all -32000 (INTERNAL_ERROR).
    ///
    /// LOAD-BEARING explicit arm: `PrismError` is `#[non_exhaustive]`; without the
    /// explicit arm this test would fail at -32000 (catch-all). The arm is in
    /// error_mapping.rs directly above the catch-all.
    ///
    /// Red Gate note (historical): this test required a fully-constructed `TableNotAvailableDetails`
    /// value; the stub state panicked at construction time. Now that S-3.13 is implemented,
    /// the test passes end-to-end.
    #[test]
    fn test_BC_2_11_001_e_query_037_mcp_maps_to_invalid_params() {
        // Construct PrismError::TableNotAvailable with all required fields.
        let err = PrismError::TableNotAvailable(Box::new(
            prism_core::error::TableNotAvailableDetails::new(
                "crowdstrike_alerts",
                "crowdstrike",
                "armis, claroty",
                "armis_alerts, claroty_devices",
                "",
                "",
            ),
        ));
        let (code, message) = map_prism_error(err);
        assert_eq!(
            code,
            codes::INVALID_PARAMS,
            "TableNotAvailable must map to INVALID_PARAMS (-32602); \
             without the explicit arm the catch-all produces -32000. got: {code}"
        );
        assert_ne!(
            code,
            codes::INTERNAL_ERROR,
            "TableNotAvailable must NOT map to INTERNAL_ERROR (-32000). got: {code}"
        );
        assert!(
            message.contains("E-QUERY-037"),
            "message must contain 'E-QUERY-037'; got: {message}"
        );
        assert!(
            message.contains("crowdstrike_alerts"),
            "message must include the table name; got: {message}"
        );
    }

    /// S-3.13 / BC-2.11.001: `TableNotAvailable` with a non-empty `did_you_mean`
    /// field preserves the suggestion in the MCP error message.
    ///
    /// Red Gate: structurally passes once the explicit arm is in place.
    /// The test validates that the Display impl threads `did_you_mean` correctly.
    #[test]
    fn test_BC_2_11_001_e_query_037_mcp_message_includes_did_you_mean() {
        let err = PrismError::TableNotAvailable(Box::new(
            prism_core::error::TableNotAvailableDetails::new(
                "crowdstrike_alert",
                "crowdstrike",
                "crowdstrike",
                "crowdstrike_alerts",
                " Did you mean: 'crowdstrike_alerts'?",
                "",
            ),
        ));
        let (code, message) = map_prism_error(err);
        assert_eq!(code, codes::INVALID_PARAMS);
        assert!(
            message.contains("Did you mean"),
            "message must include did_you_mean suggestion; got: {message}"
        );
    }

    // ── F-001B-PASS-CRIT-001 — multibyte-whitespace offset panic in near_text path ──

    /// F-001B-PASS-CRIT-001 (IDEOGRAPHIC SPACE case): `prism_error_to_structured_call_result`
    /// must NOT panic when the query contains TWO whitespace runs and the FIRST whitespace
    /// is a multibyte Unicode character (ideographic space U+3000, 3 bytes).
    ///
    /// Regression path: the `QueryParseFailed` arm computes `preceding_word_start`:
    ///   1. Slice `before_offset = query.get(..*offset)`.
    ///   2. Find `last_non_ws` = rfind(!whitespace) on before_offset.
    ///   3. Slice `before_offset.get(..=last_non_ws)` (up to the last non-WS char).
    ///   4. `rfind(whitespace)` on that slice → returns BYTE INDEX of a multibyte WS char.
    ///   5. `map(|pos| pos + 1)` → `pos + 1` falls MID-CHAR for multibyte WS.
    ///   6. `extract_near_text(query, preceding_word_start)` does `&input[mid_char..]`
    ///      → PANIC: "byte index N is not a char boundary".
    ///
    /// The panic triggers when:
    ///   - The query has TWO words separated by a multibyte whitespace (e.g. "alpha\u{3000}beta"),
    ///   - FOLLOWED by another whitespace + third word,
    ///   - And `offset` points to the THIRD word.
    ///
    /// Minimal reproducer:
    ///   query = "first\u{3000}word\u{3000}token"
    ///           bytes: f(0)i(1)r(2)s(3)t(4) U+3000(5,6,7) w(8)o(9)r(10)d(11) U+3000(12,13,14) t(15)…
    ///   offset = 15 (start of "token")
    ///   → before_offset = "first\u{3000}word\u{3000}" (bytes 0..15)
    ///   → last_non_ws = 11 ('d' byte)
    ///   → get(..=11) = "first\u{3000}word"
    ///   → rfind(whitespace) = Some(5)  (first byte of \u{3000})
    ///   → pos + 1 = 6 → NOT a char boundary for the 3-byte U+3000
    ///   → extract_near_text(query, 6) → &query[6..] → PANIC
    ///
    /// BC-2.11.017 AC-003 postcondition: `near_text` MUST be a valid UTF-8 string.
    /// Production path: `prism_error_to_structured_call_result` (F-001B-PASS-CRIT-001).
    ///
    /// Load-bearing (F-001B-PASS-CRIT-001): removing char-boundary-safe slicing from
    /// `extract_near_text` causes a panic when `pos + 1` lands mid-char.
    #[test]
    fn test_BC_2_11_017_near_text_no_panic_on_ideographic_space_multibyte_offset() {
        // query = "first\u{3000}word\u{3000}token"
        // bytes: f(0)i(1)r(2)s(3)t(4) U+3000(5,6,7) w(8)o(9)r(10)d(11) U+3000(12,13,14) t(15)o(16)k(17)e(18)n(19)
        let query = "first\u{3000}word\u{3000}token".to_string();
        // Verify byte layout
        assert_eq!(query.as_bytes()[5], 0xE3, "first U+3000 byte 0 = 0xE3");
        assert_eq!(query.as_bytes()[6], 0x80, "first U+3000 byte 1 = 0x80");
        assert_eq!(query.as_bytes()[7], 0x80, "first U+3000 byte 2 = 0x80");
        assert_eq!(query.as_bytes()[12], 0xE3, "second U+3000 byte 0 = 0xE3");
        // offset = 15 (start of "token")
        assert_eq!(&query[15..], "token", "byte 15 must be start of 'token'");
        let offset = 15usize;

        let err = PrismError::QueryParseFailed {
            offset,
            detail: "unexpected token".to_string(),
            query: query.clone(),
        };

        // Must NOT panic. Load-bearing: without char-boundary-safe slicing, the algorithm
        // does rfind(whitespace) on "first\u{3000}word", finds \u{3000} at byte 5, then
        // pos + 1 = 6 (mid-char), and extract_near_text(&query, 6) would panic.
        let result = prism_error_to_structured_call_result(err);

        // BC-2.11.017 AC-003: near_text must be present and valid UTF-8.
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.11.017)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let near_text = error_obj
            .get("near_text")
            .expect("near_text must be present for QueryParseFailed (BC-2.11.017 AC-003)");
        assert!(
            near_text.is_string(),
            "near_text must be a string (valid UTF-8), got: {near_text:?}"
        );
        // The near_text must equal "word" (the token preceding "token").
        let nt_str = near_text.as_str().unwrap();
        assert_eq!(
            nt_str, "word",
            "near_text must be 'word' (the preceding token); got '{nt_str}'"
        );
    }

    /// F-001B-PASS-CRIT-001 (NBSP case): same panic trigger with U+00A0 (NO-BREAK SPACE, 2 bytes).
    ///
    /// query = "alpha\u{00A0}beta\u{00A0}gamma"
    /// bytes: a(0)l(1)p(2)h(3)a(4) U+00A0(5,6) b(7)e(8)t(9)a(10) U+00A0(11,12) g(13)…
    /// offset = 13 (start of "gamma")
    /// → rfind(whitespace) on "alpha\u{00A0}beta" finds \u{00A0} at byte 5
    /// → pos + 1 = 6 → NOT a char boundary for the 2-byte U+00A0 → PANIC
    ///
    /// Load-bearing (F-001B-PASS-CRIT-001): removing char-boundary-safe slicing causes
    /// the same panic as the U+3000 case — `pos + 1 = 6` is mid-char for 2-byte U+00A0.
    #[test]
    fn test_BC_2_11_017_near_text_no_panic_on_nbsp_multibyte_offset() {
        // query = "alpha\u{00A0}beta\u{00A0}gamma"
        // U+00A0 = 0xC2 0xA0 (2 bytes each)
        let query = "alpha\u{00A0}beta\u{00A0}gamma".to_string();
        // Verify byte layout: "alpha" = 5 bytes, first U+00A0 at bytes 5-6, "beta" at 7-10
        assert_eq!(query.as_bytes()[5], 0xC2, "first U+00A0 byte 0 = 0xC2");
        assert_eq!(query.as_bytes()[6], 0xA0, "first U+00A0 byte 1 = 0xA0");
        assert_eq!(query.as_bytes()[11], 0xC2, "second U+00A0 byte 0 = 0xC2");
        // "gamma" starts at byte 13
        assert_eq!(&query[13..], "gamma", "byte 13 must be start of 'gamma'");
        let offset = 13usize;

        let err = PrismError::QueryParseFailed {
            offset,
            detail: "unexpected token after NBSP sequence".to_string(),
            query: query.clone(),
        };

        // Must NOT panic. Load-bearing: without char-boundary-safe slicing, rfind(whitespace)
        // on "alpha\u{00A0}beta" finds \u{00A0} at byte 5, pos + 1 = 6 (mid-char) → PANIC.
        let result = prism_error_to_structured_call_result(err);

        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.11.017)");
        let error_obj = sc.get("error").expect("error must be present");
        let near_text = error_obj
            .get("near_text")
            .expect("near_text must be present for QueryParseFailed");
        assert!(
            near_text.is_string(),
            "near_text must be a valid UTF-8 string; got: {near_text:?}"
        );
        // The near_text must equal "beta" (the token preceding "gamma").
        let nt_str = near_text.as_str().unwrap();
        assert_eq!(
            nt_str, "beta",
            "near_text must be 'beta' (the preceding token); got '{nt_str}'"
        );
    }

    // MED-002 find_first_unquoted_pipe tests have been relocated to
    // prism_query::error_recovery::mode_bridge_tests (OBS-1 relocation).

    // ── OBS-1 — multibyte non-whitespace trailing char in near_text path ──

    /// OBS-1 regression (é case): `prism_error_to_structured_call_result` must return the
    /// PRECEDING TOKEN as `near_text` when that token ends in a multibyte non-whitespace
    /// character (e.g. `é` = U+00E9, 2 bytes: 0xC3 0xA9).
    ///
    /// # Defect description
    ///
    /// The `preceding_word_start` computation in the `QueryParseFailed` arm does:
    ///   1. `last_non_ws = before_offset.rfind(|c: char| !c.is_whitespace())`
    ///      — returns the FIRST BYTE index of the last non-whitespace char.
    ///   2. `before_offset.get(..=last_non_ws)` → inclusive range equivalent to `..last_non_ws+1`.
    ///      When `last_non_ws` is the first byte of a multibyte char (e.g. `é` = 0xC3 0xA9),
    ///      `last_non_ws+1` is mid-codepoint, so `str::get` returns `None`.
    ///   3. `.and_then(...)` short-circuits to `None`, `preceding_word_start` falls back to 0.
    ///   4. `effective_offset = 0` → `extract_near_text(query, 0)` returns the start of the
    ///      query instead of the preceding token.
    ///
    /// # Test case
    ///
    /// query = "hello café bad"
    /// bytes: h(0)e(1)l(2)l(3)o(4) SP(5) c(6)a(7)f(8) é(9,10: 0xC3 0xA9) SP(11) b(12)a(13)d(14)
    /// offset = 12 (start of "bad", the error token)
    ///
    /// Before fix:
    ///   before_offset = "hello café " (bytes 0..12)
    ///   last_non_ws = byte 9 (first byte of é)
    ///   get(..=9) = get(..10) → byte 10 = 0xA9 (mid-codepoint) → None
    ///   preceding_word_start = 0 (fallback)
    ///   near_text = extract_near_text("hello café bad", 0) = "hello" (WRONG — start of query)
    ///
    /// After fix:
    ///   before_offset = "hello café " (bytes 0..12)
    ///   last_non_ws = byte 9 (first byte of é)
    ///   compute char-end: é.len_utf8() = 2, so char_end = 9 + 2 = 11
    ///   get(..11) = "hello café" → Ok (byte 11 is a char boundary — it's the space)
    ///   rfind(whitespace) on "hello café" = byte 5 (the space after "hello")
    ///   ws_char = SP.len_utf8() = 1, preceding_word_start = 5 + 1 = 6
    ///   near_text = extract_near_text("hello café bad", 6) = "café" (CORRECT)
    ///
    /// This is the SYMMETRIC counterpart to F-001B-PASS-CRIT-001 (multibyte-WHITESPACE case).
    ///
    /// Load-bearing: without char-end computation using `char_end = pos + char.len_utf8()`,
    /// the code falls back to `near_text = "hello"` (offset 0) instead of "café".
    #[test]
    fn test_BC_2_11_017_near_text_correct_when_preceding_token_ends_in_multibyte_nonws_char() {
        // query = "hello café bad"
        // é = U+00E9 = 0xC3 0xA9 (2 bytes)
        let query = "hello caf\u{00E9} bad".to_string();
        // Verify byte layout
        let bytes = query.as_bytes();
        assert_eq!(
            bytes[9], 0xC3,
            "byte 9 must be 0xC3 (first byte of é U+00E9)"
        );
        assert_eq!(
            bytes[10], 0xA9,
            "byte 10 must be 0xA9 (second byte of é U+00E9)"
        );
        assert_eq!(bytes[11], b' ', "byte 11 must be space");
        assert_eq!(&query[12..], "bad", "byte 12 must be start of 'bad'");

        let offset = 12usize; // error at "bad"

        let err = PrismError::QueryParseFailed {
            offset,
            detail: "unexpected token 'bad'".to_string(),
            query: query.clone(),
        };

        let result = prism_error_to_structured_call_result(err);

        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.11.017)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let near_text = error_obj
            .get("near_text")
            .expect("near_text must be present for QueryParseFailed (BC-2.11.017 AC-003)");

        assert!(
            near_text.is_string(),
            "near_text must be a valid UTF-8 string; got: {near_text:?}"
        );

        let nt_str = near_text.as_str().unwrap();

        // OBS-1 LOAD-BEARING: near_text must be "café" (the preceding token), NOT "hello"
        // (which is what the offset-0 fallback returns) and NOT "" (absent/empty).
        //
        // Current (broken) behavior: get(..=9) returns None because byte 10 is mid-codepoint,
        // so preceding_word_start falls back to 0, and extract_near_text returns "hello".
        //
        // Correct behavior after fix: compute char_end = 9 + é.len_utf8() = 11,
        // get(..11) succeeds, rfind(ws) finds byte 5, preceding_word_start = 6,
        // extract_near_text("hello café bad", 6) = "café".
        assert_eq!(
            nt_str,
            "caf\u{00E9}",
            "OBS-1 regression: near_text must be 'café' (the preceding token); \
             got '{nt_str}'. \
             If 'hello' is returned, the offset-0 fallback is active — \
             fix: replace `get(..=last_non_ws)` with `get(..last_non_ws + char_len)` \
             where char_len = before_offset[last_non_ws..].chars().next().map_or(1, |c| c.len_utf8()). \
             Symmetric counterpart to F-001B-PASS-CRIT-001 (multibyte-WS case)."
        );

        // DI-006: ≤50 chars
        assert!(
            nt_str.len() <= 50,
            "near_text must be ≤50 chars (DI-006); got {} chars: '{nt_str}'",
            nt_str.len()
        );
    }

    /// OBS-1 regression (em dash case): preceding token ending in U+2014 EM DASH (3 bytes: 0xE2 0x80 0x94).
    ///
    /// query = "field— bad" where `—` is U+2014 (3 bytes).
    /// bytes: f(0)i(1)e(2)l(3)d(4) —(5,6,7: 0xE2 0x80 0x94) SP(8) b(9)a(10)d(11)
    /// offset = 9 (start of "bad")
    ///
    /// Before fix:
    ///   last_non_ws = byte 5 (first byte of —)
    ///   get(..=5) = get(..6) → byte 6 = 0x80 (mid-codepoint of 3-byte U+2014) → None
    ///   preceding_word_start = 0 (fallback) → near_text = "field—" (offset-0, includes em dash)
    ///
    /// After fix:
    ///   char_end = 5 + 3 = 8, get(..8) = "field—"
    ///   rfind(whitespace) on "field—" → None (no whitespace before field—)
    ///   preceding_word_start = 0 (correct: the preceding token starts at 0)
    ///   near_text = extract_near_text(query, 0) = "field—" (correct: the whole preceding token)
    ///
    /// Both before and after fix return "field—" in this particular case (since there's no
    /// whitespace before it), but the before-fix path gets there via a WRONG route (None → 0
    /// fallback) while the after-fix path gets there via the CORRECT route (no ws found → 0).
    /// This test validates the em-dash case does NOT regress when the fix is applied.
    #[test]
    fn test_BC_2_11_017_near_text_correct_when_preceding_token_ends_in_em_dash() {
        // "field\u{2014} bad" — em dash is 3 bytes 0xE2 0x80 0x94
        let query = "field\u{2014} bad".to_string();
        let bytes = query.as_bytes();
        // "field" = 5 bytes, em dash at 5-7
        assert_eq!(bytes[5], 0xE2, "byte 5 must be 0xE2 (first byte of U+2014)");
        assert_eq!(
            bytes[6], 0x80,
            "byte 6 must be 0x80 (second byte of U+2014)"
        );
        assert_eq!(bytes[7], 0x94, "byte 7 must be 0x94 (third byte of U+2014)");
        assert_eq!(bytes[8], b' ', "byte 8 must be space");
        assert_eq!(&query[9..], "bad", "byte 9 must be start of 'bad'");

        let err = PrismError::QueryParseFailed {
            offset: 9,
            detail: "unexpected token 'bad'".to_string(),
            query: query.clone(),
        };

        let result = prism_error_to_structured_call_result(err);

        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present");
        let error_obj = sc.get("error").expect("error must be present");
        let near_text = error_obj
            .get("near_text")
            .expect("near_text must be present");

        assert!(near_text.is_string(), "near_text must be a string");
        let nt_str = near_text.as_str().unwrap();
        // "field—" starts at offset 0 (no whitespace before it), so near_text = "field—"
        assert_eq!(
            nt_str, "field\u{2014}",
            "near_text must be 'field\u{2014}' (the whole preceding token); got '{nt_str}'"
        );
    }

    // ── HIGH-2: BC-2.11.019 AC-N1B — E-QUERY-039 structured payload ────────────

    /// BC-2.11.019 AC-N1B HIGH-2 — `prism_error_to_structured_call_result` structured
    /// payload for `PrismError::EnrichUdfNotFound`.
    ///
    /// The existing `test_bc_2_11_019_n1b_mcp_maps_to_32602` (in the integration tests)
    /// only checks the flat `-32602` code returned by `map_prism_error`. This test verifies
    /// the STRUCTURED payload returned by `prism_error_to_structured_call_result`:
    ///
    /// - `category == "validation"` (not "upstream_error" catch-all)
    /// - `original_params_valid == false` (bad enrichment name — caller parameter)
    /// - `code == "E-QUERY-039"` (pinned via ec_code_override)
    /// - `suggestion` contains the available infusions list (BC §MCP surface non-empty form)
    /// - `did_you_mean` is present (threaded from EnrichUdfNotFoundDetails.did_you_mean)
    ///
    /// This test validates the HIGH-2 finding: without the dedicated `EnrichUdfNotFound`
    /// arm, the variant falls to the catch-all with category "upstream_error",
    /// original_params_valid: true, and a generic suggestion.
    #[test]
    fn test_bc_2_11_019_n1b_structured_payload_validation_category_and_suggestion() {
        use prism_core::error::EnrichUdfNotFoundDetails;

        let err = PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails::new(
            "threat_intel",
            vec![
                "threat_score".to_string(),
                "threat_is_known_malicious".to_string(),
            ],
            Some("threat_score".to_string()),
        )));

        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        // category must be "validation" (not "upstream_error" catch-all).
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "validation",
            "BC-2.11.019 AC-N1B HIGH-2: EnrichUdfNotFound must have category 'validation', \
             not 'upstream_error' catch-all. Got: '{category}'"
        );

        // original_params_valid must be false (bad enrichment name — caller parameter).
        let opv = error_obj
            .get("original_params_valid")
            .and_then(|v| v.as_bool())
            .expect("original_params_valid must be a bool");
        assert!(
            !opv,
            "BC-2.11.019 AC-N1B HIGH-2: EnrichUdfNotFound must have original_params_valid: false. \
             Got: true"
        );

        // code must be "E-QUERY-039".
        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("code must be a string");
        assert_eq!(
            code, "E-QUERY-039",
            "BC-2.11.019 AC-N1B HIGH-2: EnrichUdfNotFound must have code 'E-QUERY-039'. \
             Got: '{code}'"
        );

        // suggestion must contain the available infusions list (BC §MCP surface non-empty form).
        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("suggestion must be a string");
        assert!(
            suggestion.contains("threat_score") && suggestion.contains("threat_is_known_malicious"),
            "BC-2.11.019 AC-N1B HIGH-2: suggestion must contain available infusions. \
             Got: '{suggestion}'"
        );
        assert!(
            suggestion.contains("prism_describe"),
            "BC-2.11.019 AC-N1B HIGH-2: suggestion must reference prism_describe per BC §MCP surface. \
             Got: '{suggestion}'"
        );

        // did_you_mean must be present (threaded from EnrichUdfNotFoundDetails.did_you_mean).
        let did_you_mean = error_obj
            .get("did_you_mean")
            .and_then(|v| v.as_str())
            .expect(
                "BC-2.11.019 AC-N1B HIGH-2: did_you_mean must be present when EnrichUdfNotFoundDetails.did_you_mean is Some",
            );
        assert_eq!(
            did_you_mean, "threat_score",
            "BC-2.11.019 AC-N1B HIGH-2: did_you_mean must contain the best-match infusion name. \
             Got: '{did_you_mean}'"
        );
    }

    /// BC-2.11.019 AC-N1B HIGH-2 — empty available_infusions suggestion form.
    ///
    /// When `available_infusions` is empty, the suggestion must use the "not available"
    /// form: "No enrichment functions are registered. Enrichment is not available in this
    /// deployment."
    #[test]
    fn test_bc_2_11_019_n1b_structured_payload_empty_infusions_suggestion() {
        use prism_core::error::EnrichUdfNotFoundDetails;

        let err = PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails::new(
            "anything",
            vec![],
            None,
        )));

        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present");
        let error_obj = sc.get("error").expect("error must be present");

        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("suggestion must be a string");
        assert!(
            suggestion.contains("No enrichment functions are registered"),
            "BC-2.11.019 AC-N1B HIGH-2 empty form: suggestion must be the 'not available' form \
             when available_infusions is empty. Got: '{suggestion}'"
        );

        // did_you_mean must be absent when None (not null, key omitted).
        assert!(
            error_obj.get("did_you_mean").is_none(),
            "BC-2.11.019 AC-N1B HIGH-2: did_you_mean must be absent (key omitted) when \
             EnrichUdfNotFoundDetails.did_you_mean is None. \
             Got: {:?}",
            error_obj.get("did_you_mean")
        );
    }

    // -----------------------------------------------------------------------
    // RG-006 — S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 Red Gate test
    // -----------------------------------------------------------------------

    /// RG-006: `map_prism_error(PrismError::TemporalLiteralUnparseable { .. })` must return
    /// MCP error code `-32602` (`INVALID_PARAMS`), NOT `-32000` (`INTERNAL_ERROR`).
    ///
    /// # Red Gate pre-implementation failure
    /// No explicit arm for `PrismError::TemporalLiteralUnparseable` exists in
    /// `map_prism_error`. The variant falls through to the catch-all `_ => (INTERNAL_ERROR, ...)`
    /// arm → returns `-32000`. The assertion `code == codes::INVALID_PARAMS` FAILS with:
    ///   left:  `-32000` (INTERNAL_ERROR)
    ///   right: `-32602` (INVALID_PARAMS)
    ///
    /// # Why load-bearing (AC-006)
    /// E-QUERY-041 is a CALLER-RESOLVABLE error (the analyst sent a bad date format).
    /// Returning `-32000` (server-side internal error) misleads the MCP caller into
    /// thinking the error is transient or server-side. `-32602` INVALID_PARAMS signals
    /// the caller must fix their query.
    ///
    /// The explicit arm MUST use the symbolic constant `codes::INVALID_PARAMS`,
    /// NOT the bare literal `-32602` — repo convention (every existing arm uses
    /// `codes::` symbolic constants).
    ///
    /// # Negative assertion
    /// A separate assertion verifies the code is NOT `-32000` INTERNAL_ERROR —
    /// mutation-resistant proof that the explicit arm is load-bearing, not accidentally
    /// green via the catch-all.
    ///
    /// Traces to: BC-2.11.001 §E-QUERY-041 gate ordering + MCP -32602 constraint;
    /// ADR-052 §D4; error-taxonomy.md §E-QUERY-041.
    #[test]
    fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_map_prism_error_invalid_params() {
        let err = PrismError::TemporalLiteralUnparseable {
            value_prefix: "2026-06-24".to_string(),
        };

        let (code, message) = map_prism_error(err);

        // Primary: must map to INVALID_PARAMS (-32602).
        assert_eq!(
            code,
            codes::INVALID_PARAMS,
            "RG-006: PrismError::TemporalLiteralUnparseable must map to \
             codes::INVALID_PARAMS (-32602), not the catch-all INTERNAL_ERROR (-32000). \
             Got code: {code}. Fix: add explicit arm in map_prism_error \
             (Task 7 of S-PRISMQL-NATIVE-TEMPORAL-TYPING-001)."
        );

        // Negative: must NOT be INTERNAL_ERROR (-32000).
        assert_ne!(
            code,
            codes::INTERNAL_ERROR,
            "RG-006: TemporalLiteralUnparseable must NOT fall through to catch-all \
             INTERNAL_ERROR arm. E-QUERY-041 is caller-resolvable; returning -32000 \
             misleads the MCP caller. Got code: {code}."
        );

        // The message must mention E-QUERY-041 (from the PrismError Display impl).
        assert!(
            message.contains("E-QUERY-041"),
            "RG-006: map_prism_error message must include 'E-QUERY-041' from the \
             TemporalLiteralUnparseable Display. Got: {message:?}"
        );
    }

    // -----------------------------------------------------------------------
    // HIGH-1 — S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 structured-path test
    // -----------------------------------------------------------------------

    /// HIGH-1: `prism_error_to_structured_call_result(PrismError::TemporalLiteralUnparseable)`
    /// must produce a structured payload with `category == "validation"`,
    /// `original_params_valid == false`, and `code == "E-QUERY-041"`.
    ///
    /// # Why this is a real production defect (TD-VSDD-060 sibling-site miss)
    /// The flat `map_prism_error` path (RG-006) correctly maps `TemporalLiteralUnparseable`
    /// → INVALID_PARAMS (`-32602`). But the STRUCTURED path — `prism_error_to_structured_call_result`
    /// — which the `query` MCP tool actually uses (server.rs routes domain errors through it)
    /// has NO `VariantMeta` arm for `TemporalLiteralUnparseable`. It falls to the catch-all
    /// (~line 1924-1941): `category: "upstream_error"`, `original_params_valid: true`,
    /// `suggestion: "See audit log for details."` — semantically wrong for a
    /// caller-resolvable plan-time validation error.
    ///
    /// The sibling error E-QUERY-040 `RedundantRowLimit` IS correctly in the "validation"
    /// `VariantMeta` group; E-QUERY-041 was not swept in (TD-VSDD-060 sibling-site miss).
    ///
    /// # Pre-fix failure
    /// Without the dedicated arm:
    ///   `category == "upstream_error"` (NOT "validation")
    ///   `original_params_valid == true` (NOT false — E-QUERY-041 IS a bad-params error)
    ///   `code != "E-QUERY-041"` (falls to default code derivation)
    ///
    /// # Fix
    /// Add `PrismError::TemporalLiteralUnparseable { .. }` to the "validation" `VariantMeta`
    /// group alongside `RedundantRowLimit`, with `original_params_valid: false` and
    /// `ec_code_override: Some("E-QUERY-041")`.
    ///
    /// Mirrors: `test_bc_2_11_019_n1b_structured_payload_validation_category_and_suggestion`
    /// (EnrichUdfNotFound HIGH-2 fix, same pattern).
    ///
    /// Traces to: ADR-052 §D4; BC-2.11.001 §E-QUERY-041 gate ordering;
    /// error-taxonomy.md §E-QUERY-041; TD-VSDD-060 sibling-site sweep.
    #[test]
    fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_structured_path_validation_category() {
        let err = PrismError::TemporalLiteralUnparseable {
            value_prefix: "2026-06-24".to_string(),
        };

        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        // category must be "validation" (not "upstream_error" catch-all).
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "validation",
            "HIGH-1: TemporalLiteralUnparseable structured path must have category 'validation', \
             not 'upstream_error' catch-all. E-QUERY-041 is a caller-resolvable plan-time \
             validation error — wrong category misleads the MCP caller. Got: '{category}'"
        );

        // original_params_valid must be false — the date-only/offset-less literal IS the bad param.
        let opv = error_obj
            .get("original_params_valid")
            .and_then(|v| v.as_bool())
            .expect("original_params_valid must be a bool");
        assert!(
            !opv,
            "HIGH-1: TemporalLiteralUnparseable must have original_params_valid: false. \
             The query literal is the invalid parameter — the caller must fix it to RFC-3339. \
             Got: true (catch-all default)"
        );

        // code must be "E-QUERY-041" (via ec_code_override in the VariantMeta arm).
        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("code must be a string");
        assert_eq!(
            code, "E-QUERY-041",
            "HIGH-1: TemporalLiteralUnparseable structured path must have code 'E-QUERY-041'. \
             Got: '{code}'"
        );

        // suggestion must be analyst-actionable (RFC-3339 format guidance).
        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("suggestion must be a string");
        assert!(
            suggestion.contains("RFC-3339")
                || suggestion.contains("rfc3339")
                || suggestion.contains("UTC")
                || suggestion.contains("2026"),
            "HIGH-1: suggestion must contain RFC-3339 format guidance for the analyst. \
             Got: '{suggestion}'"
        );
    }

    // -----------------------------------------------------------------------
    // E-QUERY-042 — S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 MCP mapping tests
    // -----------------------------------------------------------------------

    /// E-QUERY-042 flat path test: `map_prism_error(PrismError::TemporalLiteralInvalidPosition)`
    /// MUST return `-32602 INVALID_PARAMS` for ALL three position variants (GroupBy, OrderBy,
    /// NonColumnLhsComparison) — MUST NOT fall through to catch-all `-32000 INTERNAL_ERROR`.
    ///
    /// Without an explicit arm in `map_prism_error`, the `#[non_exhaustive]` PrismError
    /// catch-all would map E-QUERY-042 to `-32000 INTERNAL_ERROR`, making an analyst-resolvable
    /// error appear to be an internal server error.
    ///
    /// Sibling of `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_map_prism_error_invalid_params`
    /// (same pattern for E-QUERY-041).
    ///
    /// Traces to: error-taxonomy.md §E-QUERY-042 v2.14 `map_prism_error` constraint;
    ///            ADR-052 §D4 v1.10; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001.
    #[test]
    fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_042_map_prism_error_all_positions_invalid_params(
    ) {
        use prism_core::error::TemporalLiteralPosition;

        let positions = [
            (TemporalLiteralPosition::GroupBy, "GroupBy"),
            (TemporalLiteralPosition::OrderBy, "OrderBy"),
            (
                TemporalLiteralPosition::NonColumnLhsComparison,
                "NonColumnLhsComparison",
            ),
        ];

        for (position, pos_name) in positions {
            let err = PrismError::TemporalLiteralInvalidPosition {
                position,
                value_prefix: "2026-06-24".to_string(),
            };

            let (code, message) = map_prism_error(err);

            // Primary: must map to INVALID_PARAMS (-32602).
            assert_eq!(
                code,
                codes::INVALID_PARAMS,
                "E-QUERY-042 ({pos_name}): PrismError::TemporalLiteralInvalidPosition must map \
                 to codes::INVALID_PARAMS (-32602), not the catch-all INTERNAL_ERROR (-32000). \
                 Got code: {code}."
            );

            // Negative: must NOT be INTERNAL_ERROR (-32000).
            assert_ne!(
                code,
                codes::INTERNAL_ERROR,
                "E-QUERY-042 ({pos_name}): TemporalLiteralInvalidPosition must NOT fall through \
                 to catch-all INTERNAL_ERROR. This is caller-resolvable — returning -32000 \
                 misleads the MCP caller. Got code: {code}."
            );

            // The message must mention E-QUERY-042 (from the PrismError Display impl).
            assert!(
                message.contains("E-QUERY-042"),
                "E-QUERY-042 ({pos_name}): map_prism_error message must include 'E-QUERY-042' \
                 from the TemporalLiteralInvalidPosition Display. Got: {message:?}"
            );
        }
    }

    /// E-QUERY-042 structured path test: `prism_error_to_structured_call_result` must
    /// produce `category == "validation"`, `original_params_valid == false`, and
    /// `code == "E-QUERY-042"` for all three position variants.
    ///
    /// Without a dedicated `VariantMeta` arm, `TemporalLiteralInvalidPosition` would fall
    /// to the catch-all with `category: "upstream_error"` and `original_params_valid: true`
    /// — semantically wrong for a caller-resolvable plan-time validation error.
    ///
    /// Sibling of `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_structured_path_validation_category`
    /// (same pattern for E-QUERY-041).
    ///
    /// Traces to: error-taxonomy.md §E-QUERY-042 v2.14; ADR-052 §D4 v1.10;
    ///            S-PRISMQL-NATIVE-TEMPORAL-TYPING-001.
    #[test]
    fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_042_structured_path_validation_category() {
        use prism_core::error::TemporalLiteralPosition;

        // Test with GroupBy as representative position (all positions use same VariantMeta).
        let err = PrismError::TemporalLiteralInvalidPosition {
            position: TemporalLiteralPosition::GroupBy,
            value_prefix: "2026-06-24".to_string(),
        };

        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        // category must be "validation" (not "upstream_error" catch-all).
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "validation",
            "E-QUERY-042: TemporalLiteralInvalidPosition structured path must have \
             category 'validation', not 'upstream_error' catch-all. \
             E-QUERY-042 is caller-resolvable. Got: '{category}'"
        );

        // original_params_valid must be false — the malpositioned literal IS the bad param.
        let opv = error_obj
            .get("original_params_valid")
            .and_then(|v| v.as_bool())
            .expect("original_params_valid must be a bool");
        assert!(
            !opv,
            "E-QUERY-042: TemporalLiteralInvalidPosition must have original_params_valid: false. \
             Got: true (catch-all default)"
        );

        // code must be "E-QUERY-042".
        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("code must be a string");
        assert_eq!(
            code, "E-QUERY-042",
            "E-QUERY-042: TemporalLiteralInvalidPosition structured path must have \
             code 'E-QUERY-042'. Got: '{code}'"
        );
    }

    // =========================================================================
    // DEFECT-MCP-ROWSHAPE-NULLS-001 — DEFECT 2 [H8b]: doubled "audit log" in
    // internal-redacted error content_text.
    //
    // Root cause: `content_text = format!("ERROR: [{}] - {}. {}", category, message, suggestion)`
    // in `prism_error_to_structured_call_result` (error_mapping.rs ~line 2122). For catch-all
    // internal variants:
    //   `message    = "Internal error; see audit log"`  (from map_prism_error catch-all)
    //   `suggestion = "See audit log for details."`     (from VariantMeta catch-all)
    // → "audit log" phrase appears TWICE in content_text.
    //
    // Fix direction (BC-2.10.007 message/suggestion semantics):
    //   `message`    → `"Internal error"` (terse; no actionable pointer)
    //   `suggestion` → `"See audit log for details."` (carries the actionable pointer)
    //
    // Spec authority: BC-2.10.007 §Postconditions; error-taxonomy.md message/suggestion split.
    //
    // These tests FAIL against current code (count == 2 for all catch-all variants).
    // =========================================================================

    /// Helper: extract the plain-text content from a `CallToolResult`.
    ///
    /// Mirrors `extract_text_content` from `server.rs` tests — NOT the same function;
    /// duplicated here so error_mapping tests have no cross-module test-helper dependency.
    fn extract_content_text_from_result(result: &rmcp::model::CallToolResult) -> String {
        result
            .content
            .iter()
            .filter_map(|c| c.as_text().map(|t| t.text.as_str().to_owned()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// BC-2.10.007 [H8b]: for a catch-all internal-redacted error (`QueryExecutionFailed`),
    /// the phrase "audit log" must appear EXACTLY ONCE in `content_text`.
    ///
    /// FAILS NOW: `message = "Internal error; see audit log"` AND
    /// `suggestion = "See audit log for details."` → "audit log" count == 2.
    ///
    /// PASSES after: `map_prism_error` catch-all returns `"Internal error"` (terse),
    /// leaving "audit log" only in `suggestion`.
    #[test]
    fn test_BC_2_10_007_H8b_internal_redacted_content_text_audit_log_appears_once() {
        let err = PrismError::QueryExecutionFailed {
            detail: "DataFusion plan execution aborted".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let content_text = extract_content_text_from_result(&result);

        let audit_log_count = content_text.to_lowercase().matches("audit log").count();
        assert_eq!(
            audit_log_count, 1,
            "[H8b] VIOLATION: 'audit log' appears {audit_log_count} times in content_text — \
             must appear exactly once. Current defect: message='Internal error; see audit log' \
             AND suggestion='See audit log for details.' both contain the phrase. \
             Fix: map_prism_error catch-all must return terse 'Internal error' with no \
             'see audit log' suffix. content_text was: {content_text:?}"
        );
    }

    /// BC-2.10.007 [H8b]: for `QueryExecutionFailed`, the structured envelope `message`
    /// field must be the TERSE redacted form — it must NOT contain "see audit log".
    ///
    /// The actionable pointer ("See audit log for details.") belongs in `suggestion` only.
    ///
    /// FAILS NOW: `message = "Internal error; see audit log"` contains the pointer in the
    /// wrong field.
    #[test]
    fn test_BC_2_10_007_H8b_query_execution_failed_message_field_is_terse() {
        let err = PrismError::QueryExecutionFailed {
            detail: "test detail".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.message must be a string");

        // BC-2.10.007: message is the terse human-readable description.
        // Suggestion carries the actionable pointer. They must not overlap.
        assert!(
            !message.to_lowercase().contains("see audit log"),
            "[H8b] VIOLATION: message field '{message}' contains 'see audit log' — \
             the actionable pointer belongs in suggestion only (BC-2.10.007 message/suggestion \
             split). Fix: map_prism_error catch-all must return 'Internal error' (terse)."
        );
        // The terse form must still communicate that this is an internal error.
        assert!(
            message.to_lowercase().contains("internal"),
            "message must still identify this as an internal error; got: '{message}'"
        );

        // suggestion must carry the audit log pointer.
        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.suggestion must be a string");
        assert!(
            suggestion.to_lowercase().contains("audit log"),
            "suggestion must contain 'audit log' pointer; got: '{suggestion}'"
        );
    }

    /// BC-2.10.007 [H8b/OBS-001] redundancy sweep: "audit log" appears exactly once in
    /// content_text for all affected variants; byte-verbatim suggestion locks per arm (POL-24).
    ///
    /// Two groups (BC-2.10.007):
    ///   - **Query engine arm** (LOW-002): `QueryExecutionFailed`, `QueryMemoryBudgetExceeded`,
    ///     `QueryDenylisted` → category "internal", suggestion "Prism query engine failure.
    ///     Contact Prism operator; see audit log for details."
    ///   - **Explicit upstream_error arm** (F-MCPRS-PRL10-OBS-003 Group 4):
    ///     `OcsfNormalizationFailed` → category "upstream_error",
    ///     suggestion "Check sensor API status. If the problem persists, see audit log."
    ///
    /// Both groups: "audit log" must appear exactly once in content_text (H8b redundancy
    /// property). Verbatim suggestion strings are the POL-24 lock added by F-MCPRS-PRL2-OBS-001.
    ///
    /// The three query-engine-arm assertions are RED before the LOW-002 arm is implemented
    /// (they currently produce suggestion "See audit log for details." from the catch-all).
    #[test]
    fn test_BC_2_10_007_H8b_redundancy_sweep_audit_log_once() {
        // ── Group 1: query engine arm variants (LOW-002) ──────────────────────────
        const QUERY_ENGINE_SUGGESTION: &str =
            "Prism query engine failure. Contact Prism operator; see audit log for details.";
        let query_engine_variants: Vec<(&str, PrismError)> = vec![
            (
                "QueryExecutionFailed",
                PrismError::QueryExecutionFailed {
                    detail: "DataFusion internal error".to_owned(),
                },
            ),
            (
                "QueryMemoryBudgetExceeded",
                PrismError::QueryMemoryBudgetExceeded {
                    limit_mb: 200,
                    used_mb: 250,
                },
            ),
            (
                "QueryDenylisted",
                PrismError::QueryDenylisted {
                    failure_count: 3,
                    reason: "repeated execution failure".to_owned(),
                    expiry_ts: 9_999_999_999,
                },
            ),
        ];

        for (variant_name, err) in query_engine_variants {
            let result = prism_error_to_structured_call_result(err);
            let content_text = extract_content_text_from_result(&result);

            let audit_log_count = content_text.to_lowercase().matches("audit log").count();
            assert_eq!(
                audit_log_count, 1,
                "[H8b] VIOLATION for {variant_name}: 'audit log' appears {audit_log_count} \
                 times in content_text — must appear exactly once. content_text: {content_text:?}"
            );

            // POL-24 verbatim suggestion lock (F-MCPRS-PRL2-OBS-001).
            let sc = result
                .structured_content
                .as_ref()
                .unwrap_or_else(|| panic!("[{variant_name}] structuredContent must be present"));
            let error_obj = sc.get("error").unwrap_or_else(|| {
                panic!("[{variant_name}] structuredContent.error must be present")
            });
            let suggestion = error_obj
                .get("suggestion")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("[{variant_name}] suggestion must be a string"));
            assert_eq!(
                suggestion, QUERY_ENGINE_SUGGESTION,
                "[OBS-001/POL-24] {variant_name} suggestion must be byte-verbatim \
                 '{QUERY_ENGINE_SUGGESTION}'; got '{suggestion}'"
            );
        }

        // ── Group 2: upstream_error explicit arm variants (F-MCPRS-PRL10-OBS-003) ────
        // OcsfNormalizationFailed now has an explicit Group 4 arm, not the catch-all.
        // Suggestion changed from "See audit log for details." to the upstream_error
        // sensor-status guidance. H8b property still holds: "audit log" appears once.
        const UPSTREAM_ERROR_EXPLICIT_SUGGESTION: &str =
            "Check sensor API status. If the problem persists, see audit log.";
        let upstream_explicit_variants: Vec<(&str, PrismError)> = vec![(
            "OcsfNormalizationFailed",
            PrismError::OcsfNormalizationFailed {
                source_id: "crowdstrike_alerts".to_owned(),
                reason: "unknown field".to_owned(),
            },
        )];

        for (variant_name, err) in upstream_explicit_variants {
            let result = prism_error_to_structured_call_result(err);
            let content_text = extract_content_text_from_result(&result);

            let audit_log_count = content_text.to_lowercase().matches("audit log").count();
            assert_eq!(
                audit_log_count, 1,
                "[H8b] VIOLATION for {variant_name}: 'audit log' appears {audit_log_count} \
                 times in content_text — must appear exactly once. content_text: {content_text:?}"
            );

            // POL-24 verbatim suggestion lock (F-MCPRS-PRL10-OBS-003 Group 4).
            let sc = result
                .structured_content
                .as_ref()
                .unwrap_or_else(|| panic!("[{variant_name}] structuredContent must be present"));
            let error_obj = sc.get("error").unwrap_or_else(|| {
                panic!("[{variant_name}] structuredContent.error must be present")
            });
            let suggestion = error_obj
                .get("suggestion")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("[{variant_name}] suggestion must be a string"));
            assert_eq!(
                suggestion, UPSTREAM_ERROR_EXPLICIT_SUGGESTION,
                "[OBS-001/POL-24] {variant_name} suggestion must be byte-verbatim \
                 '{UPSTREAM_ERROR_EXPLICIT_SUGGESTION}'; got '{suggestion}'"
            );
        }
    }

    // =========================================================================
    // SID-2 composed-output locks — auth-category variants (F-MCPNULL-P6-OBS-001)
    //
    // These tests assert the FULL composed content[].text for AuthTokenExpired and
    // AuthTokenInvalid. Auth-category VariantMeta suggestions carry re-authenticate
    // guidance, NOT audit-log pointers — four invariants per variant:
    //   (a) content_text contains "Re-authenticate"
    //   (b) message field is the terse "Internal error" with NO "audit log"
    //   (c) content_text does NOT contain "audit log" at all
    //   (d) no phrase from message repeats in suggestion (message/suggestion split)
    //
    // GREEN on arrival: both variants are already handled by explicit VariantMeta arms
    // (lines 1050-1090) with re-auth suggestions and ec_code_override; these tests lock
    // the behaviour so future refactors cannot regress to the catch-all audit-log pointer.
    // =========================================================================

    /// BC-2.10.007 SID-2 composed-output lock: `AuthTokenExpired` auth-category variant.
    ///
    /// Locks all four composed-output properties for `AuthTokenExpired`:
    /// (a) `content_text` contains "Re-authenticate" (BC-2.10.007 auth-category LLM-agent strategy)
    /// (b) `message` field is the terse "Internal error" with NO "audit log"
    /// (c) `content_text` does NOT contain "audit log" (auth suggestions carry re-auth guidance only)
    /// (d) No phrase from `message` repeats in `suggestion` (BC-2.10.007 message/suggestion split)
    ///
    /// Byte-verbatim suggestion check per POL-24: asserts the exact shipped static string
    /// `"The auth token has expired. Re-authenticate and obtain a fresh token."`.
    ///
    /// F-MCPNULL-P6-OBS-001: closes the SID-2 gap for the auth-expired variant.
    #[test]
    fn test_BC_2_10_007_auth_variant_composed_content_text_expired() {
        let err = PrismError::AuthTokenExpired;
        let result = prism_error_to_structured_call_result(err);
        let content_text = extract_content_text_from_result(&result);

        // (a) content_text must contain "Re-authenticate" per BC-2.10.007 auth-category strategy.
        assert!(
            content_text.contains("Re-authenticate"),
            "[SID-2][AuthTokenExpired] VIOLATION: content_text must contain 'Re-authenticate'. \
             Got: {content_text:?}"
        );

        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.message must be a string");

        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.suggestion must be a string");

        // (b) message must be the terse "Internal error" with NO "audit log".
        assert_eq!(
            message, "Internal error",
            "[SID-2][AuthTokenExpired] VIOLATION: message must be terse 'Internal error'. \
             Got: {message:?}"
        );
        assert!(
            !message.to_lowercase().contains("audit log"),
            "[SID-2][AuthTokenExpired] VIOLATION: message must NOT contain 'audit log'. \
             Got: {message:?}"
        );

        // (c) content_text must NOT contain "audit log" at all.
        assert!(
            !content_text.to_lowercase().contains("audit log"),
            "[SID-2][AuthTokenExpired] VIOLATION: content_text must NOT contain 'audit log'. \
             Auth variants carry re-auth guidance only, not audit-log pointers. \
             Got: {content_text:?}"
        );

        // Byte-verbatim suggestion check per POL-24.
        assert_eq!(
            suggestion, "The auth token has expired. Re-authenticate and obtain a fresh token.",
            "[SID-2][AuthTokenExpired] VIOLATION: suggestion must match exact shipped string \
             byte-verbatim. Got: {suggestion:?}"
        );

        // (d) Duplicate-phrase check: no phrase from message may repeat in suggestion.
        assert!(
            !suggestion.contains(message),
            "[SID-2][AuthTokenExpired] VIOLATION: phrase from message ({message:?}) repeats in \
             suggestion ({suggestion:?}). BC-2.10.007 message/suggestion split forbids this."
        );
    }

    /// BC-2.10.007 SID-2 composed-output lock: `AuthTokenInvalid` auth-category variant.
    ///
    /// Locks all four composed-output properties for `AuthTokenInvalid`:
    /// (a) `content_text` contains "Re-authenticate" (BC-2.10.007 auth-category LLM-agent strategy)
    /// (b) `message` field is the terse "Internal error" with NO "audit log"
    /// (c) `content_text` does NOT contain "audit log" (auth suggestions carry re-auth guidance only)
    /// (d) No phrase from `message` repeats in `suggestion` (BC-2.10.007 message/suggestion split)
    ///
    /// Byte-verbatim suggestion check per POL-24: asserts the exact shipped static string
    /// `"The auth token is invalid. Re-authenticate and obtain a valid token."`.
    ///
    /// The `reason` field is set to a test string; the MCP response must NOT surface it (DI-006).
    ///
    /// F-MCPNULL-P6-OBS-001: closes the SID-2 gap for the auth-invalid variant.
    #[test]
    fn test_BC_2_10_007_auth_variant_composed_content_text_invalid() {
        let err = PrismError::AuthTokenInvalid {
            reason: "signature mismatch".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let content_text = extract_content_text_from_result(&result);

        // (a) content_text must contain "Re-authenticate" per BC-2.10.007 auth-category strategy.
        assert!(
            content_text.contains("Re-authenticate"),
            "[SID-2][AuthTokenInvalid] VIOLATION: content_text must contain 'Re-authenticate'. \
             Got: {content_text:?}"
        );

        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.message must be a string");

        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.suggestion must be a string");

        // (b) message must be the terse "Internal error" with NO "audit log".
        assert_eq!(
            message, "Internal error",
            "[SID-2][AuthTokenInvalid] VIOLATION: message must be terse 'Internal error'. \
             Got: {message:?}"
        );
        assert!(
            !message.to_lowercase().contains("audit log"),
            "[SID-2][AuthTokenInvalid] VIOLATION: message must NOT contain 'audit log'. \
             Got: {message:?}"
        );

        // (c) content_text must NOT contain "audit log" at all.
        assert!(
            !content_text.to_lowercase().contains("audit log"),
            "[SID-2][AuthTokenInvalid] VIOLATION: content_text must NOT contain 'audit log'. \
             Auth variants carry re-auth guidance only, not audit-log pointers. \
             Got: {content_text:?}"
        );

        // Byte-verbatim suggestion check per POL-24.
        assert_eq!(
            suggestion, "The auth token is invalid. Re-authenticate and obtain a valid token.",
            "[SID-2][AuthTokenInvalid] VIOLATION: suggestion must match exact shipped string \
             byte-verbatim. Got: {suggestion:?}"
        );

        // (d) Duplicate-phrase check: no phrase from message may repeat in suggestion.
        assert!(
            !suggestion.contains(message),
            "[SID-2][AuthTokenInvalid] VIOLATION: phrase from message ({message:?}) repeats in \
             suggestion ({suggestion:?}). BC-2.10.007 message/suggestion split forbids this."
        );
    }

    /// BC-2.10.007 OBS-002 ruling applied: `McpSerializationError` maps to
    /// category `"internal"`, code `"E-MCP-003"`, terse `"Internal error"` message,
    /// byte-verbatim suggestion, retryable:false.
    ///
    /// SID-2 composed-output lock:
    /// (a) content_text contains "Prism operator" (internal-category operator-escalation guidance)
    /// (b) message = `"Internal error"` (terse; BC-2.10.007 Rule 1 — McpSerializationError is NOT
    ///     the AuditPersistenceFailed exception)
    /// (c) code = `"E-MCP-003"` (ec_code_override required — without pin, E-INT-001 fallback fires)
    /// (d) byte-verbatim suggestion (POL-24 lock)
    /// (e) no phrase from message repeats in suggestion (BC-2.10.007 message/suggestion split)
    ///
    /// BC-2.10.007 §OBS-002 + error-taxonomy v2.42 E-MCP-003.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_mcp_serialization_error_category_is_internal() {
        let err = PrismError::McpSerializationError {
            detail: "serde serialization failed: invalid type at field 'x'".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let content_text = extract_content_text_from_result(&result);

        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.message must be a string");
        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.code must be a string");
        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.suggestion must be a string");
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");

        // (a) Category must be "internal" — Prism's own serialization layer failed.
        assert_eq!(
            category, "internal",
            "[OBS-002] McpSerializationError must map to category 'internal' \
             (BC-2.10.007 OBS-002); got '{category}'"
        );

        // (b) Code must be "E-MCP-003" — pinned via ec_code_override.
        // Without the override, the E-INT-001 catch-all fires incorrectly.
        assert_eq!(
            code, "E-MCP-003",
            "[OBS-002] McpSerializationError code must be 'E-MCP-003' (ec_code_override); \
             got '{code}'. If this fails, check ec_code_override: Some(\"E-MCP-003\") in the \
             McpSerializationError VariantMeta arm."
        );

        // (c) Message must be the terse "Internal error" per BC-2.10.007 Rule 1.
        // McpSerializationError is NOT the AuditPersistenceFailed exhaustive exception.
        assert_eq!(
            message, "Internal error",
            "[OBS-002][SID-2] McpSerializationError message must be terse 'Internal error' \
             (BC-2.10.007 Rule 1; McpSerializationError is not the AuditPersistenceFailed \
             exception); got '{message}'"
        );

        // (d) Byte-verbatim suggestion (POL-24 lock).
        assert_eq!(
            suggestion,
            "Prism MCP serialization failure. Contact Prism operator; see audit log for details.",
            "[OBS-002][SID-2] McpSerializationError suggestion must match exact shipped string \
             byte-verbatim (BC-2.10.007 OBS-002); got '{suggestion}'"
        );

        // retryable must be false — Prism-internal serialization failures are not transient.
        assert!(
            !retryable,
            "[OBS-002] McpSerializationError must be retryable:false; got true"
        );

        // (a) SID-2: content_text must contain "Prism operator" (internal-category escalation guidance).
        assert!(
            content_text.contains("Prism operator"),
            "[SID-2][OBS-002] content_text must contain 'Prism operator'. Got: {content_text:?}"
        );

        // (e) No phrase from message repeats in suggestion (BC-2.10.007 message/suggestion split).
        assert!(
            !suggestion.contains(message),
            "[SID-2][OBS-002] phrase from message ({message:?}) repeats in suggestion \
             ({suggestion:?}). BC-2.10.007 message/suggestion split forbids this."
        );
    }

    /// BC-2.10.007 MED-001 regression fence: `AuditPersistenceFailed` is the ONE
    /// exhaustive exception to Rule 1.  Its structured `message` MUST be the full
    /// taxonomy-verbatim Display — NOT the terse `"Internal error"` form.
    ///
    /// A future "helpful" refactor that collapses this variant to `"Internal error"` MUST
    /// fail this test.  The message carries no sensitive detail (no credentials, no raw sensor
    /// text); the agent caller needs the code + retry guidance to act on this transient,
    /// retryable fail-closed condition (BC-2.05.001 DEC-014).
    ///
    /// Asserts:
    /// - `structuredContent.error.message` = byte-verbatim taxonomy Display
    /// - `map_prism_error` code = -32000 (INTERNAL_ERROR JSON-RPC code)
    /// - `structuredContent.error.category` = "transient"
    ///
    /// BC-2.10.007 Rule 1 exception + BC-2.05.001 DEC-014.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_audit_persistence_failed_message_carveout() {
        // BC-2.10.007 Rule 1 exception — the ONE exhaustive carve-out.
        // Byte-verbatim check: this string must match prism-core error.rs Display exactly.
        // If prism-core's Display ever changes, this test will catch the drift.
        const EXPECTED_MESSAGE: &str =
            "E-AUDIT-001: Audit emission failed; write operation blocked. \
             Retry the operation. If the error persists, check tracing subscriber health.";

        // Verify the map_prism_error code is -32000 (INTERNAL_ERROR).
        let (code_i32, map_message) = map_prism_error(PrismError::AuditPersistenceFailed);
        assert_eq!(
            code_i32,
            codes::INTERNAL_ERROR,
            "[MED-001] AuditPersistenceFailed must map to JSON-RPC code -32000 (INTERNAL_ERROR); \
             got {code_i32}"
        );
        // Verify map_prism_error message IS the taxonomy-verbatim Display (the BC-2.10.007
        // Rule 1 exception path: format!("{err}") instead of "Internal error").
        assert_eq!(
            map_message, EXPECTED_MESSAGE,
            "[MED-001] map_prism_error(AuditPersistenceFailed) must return the taxonomy-verbatim \
             Display; got '{map_message}'. If this fails, verify prism-core error.rs AuditPersistenceFailed \
             #[error] attribute still matches EXPECTED_MESSAGE."
        );

        // Verify the STRUCTURED ERROR message is also the full Display
        // (prism_error_to_structured_call_result uses map_prism_error message → structured field).
        let result = prism_error_to_structured_call_result(PrismError::AuditPersistenceFailed);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let structured_message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.message must be a string");

        // BC-2.10.007 Rule 1 exception: AuditPersistenceFailed emits the full
        // taxonomy-verbatim Display as message (NOT "Internal error").
        assert_eq!(
            structured_message, EXPECTED_MESSAGE,
            "[MED-001] AuditPersistenceFailed structured message must be the taxonomy-verbatim \
             Display (BC-2.10.007 Rule 1 exception; BC-2.05.001 DEC-014). \
             Got '{structured_message}'. A refactor to terse 'Internal error' MUST fail this test."
        );

        // category must be "transient" (retryable transient fail-closed condition).
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.category must be a string");
        assert_eq!(
            category, "transient",
            "[MED-001] AuditPersistenceFailed must have category 'transient'; got '{category}'"
        );

        // retryable must be true — this is a transient fail-closed condition.
        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            retryable,
            "[MED-001] AuditPersistenceFailed must be retryable:true (transient fail-closed); \
             got false"
        );

        // POL-24 byte-verbatim suggestion lock.
        //
        // message and suggestion are COMPLEMENTARY pointers (orchestrator adjudication of
        // F-MCPNULL-P8-OBS-001):
        //   - message = taxonomy-verbatim Display (BC-2.10.007 carve-out), ending
        //     "...check tracing subscriber health." — tells the agent WHERE the fail-closed
        //     trace is emitted.
        //   - suggestion = audit-log-storage pointer, owned by prism-mcp error_mapping.rs
        //     VariantMeta arm — tells the agent WHERE to look for persistence evidence.
        //
        // A future refactor that unifies message and suggestion to the same string MUST
        // fail this test.
        const EXPECTED_SUGGESTION: &str =
            "Retry the operation. If the problem persists, check the audit log storage.";
        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("structuredContent.error.suggestion must be a string");
        assert_eq!(
            suggestion, EXPECTED_SUGGESTION,
            "[POL-24] AuditPersistenceFailed suggestion must be the audit-log-storage pointer \
             (NOT the tracing-subscriber Display text). Got '{suggestion}'. \
             message and suggestion are complementary: message carries the taxonomy-verbatim \
             Display (BC-2.10.007 carve-out); suggestion carries the audit-log-storage \
             retry pointer (prism-mcp VariantMeta, F-MCPNULL-P8-OBS-001 adjudication)."
        );
    }

    // =========================================================================
    // F-MCPRS-PRL2-LOW-002 — Query engine variants: category "internal"
    //
    // BC-2.10.007 §LOW-002: Six DataFusion/query-engine variants that
    // previously fell to the `_ =>` catch-all (category "upstream_error") — or
    // had a misclassified dedicated arm (QueryPlanFailed: "validation") — now
    // all map to category "internal".
    //
    // These tests are RED before the implementation arm is added (catch-all
    // gives "upstream_error"; QueryPlanFailed's prior arm gives "validation").
    // =========================================================================

    /// BC-2.10.007 LOW-002: `QueryExecutionFailed` → category `"internal"`.
    ///
    /// Asserts full LOW-002 postconditions per BC §Canonical Test Vectors (POL-24
    /// byte-verbatim for suggestion and code strings).
    ///
    /// RED before implementation: current catch-all arm maps to `"upstream_error"`.
    #[test]
    fn test_BC_2_10_007_query_execution_failed_category_is_internal() {
        let err = PrismError::QueryExecutionFailed {
            detail: "DataFusion execution error".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "internal",
            "[LOW-002] QueryExecutionFailed must map to category 'internal' (not 'upstream_error'); \
             got '{category}'"
        );

        let original_params_valid = error_obj
            .get("original_params_valid")
            .and_then(|v| v.as_bool())
            .expect("original_params_valid must be a bool");
        assert!(
            !original_params_valid,
            "[LOW-002] QueryExecutionFailed must have original_params_valid:false"
        );

        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            !retryable,
            "[LOW-002] QueryExecutionFailed must be retryable:false"
        );

        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("code must be a string");
        assert_eq!(
            code, "E-QUERY-034",
            "[LOW-002/POL-24] QueryExecutionFailed code must be byte-verbatim 'E-QUERY-034'; \
             got '{code}'"
        );

        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("suggestion must be a string");
        assert_eq!(
            suggestion,
            "Prism query engine failure. Contact Prism operator; see audit log for details.",
            "[LOW-002/POL-24] QueryExecutionFailed suggestion must be byte-verbatim; \
             got '{suggestion}'"
        );

        // Rule 1 invariance: message MUST be "Internal error" (map_prism_error MUST NOT change).
        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("message must be a string");
        assert_eq!(
            message, "Internal error",
            "[LOW-002/Rule-1] QueryExecutionFailed message must be 'Internal error' \
             (Rule 1 redaction preserved; map_prism_error must NOT change for this variant); \
             got '{message}'"
        );
    }

    /// BC-2.10.007 LOW-002: `QueryPlanFailed` → category `"internal"`.
    ///
    /// Previously had a dedicated arm with `category: "validation"`. Per BC v1.12,
    /// query plan failures are Prism engine failures → `category: "internal"`.
    ///
    /// RED before implementation: current dedicated arm maps to `"validation"`.
    #[test]
    fn test_BC_2_10_007_query_plan_failed_category_is_internal() {
        let err = PrismError::QueryPlanFailed {
            detail: "plan error".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "internal",
            "[LOW-002] QueryPlanFailed must map to category 'internal' (not 'validation'); \
             got '{category}'"
        );

        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("code must be a string");
        assert_eq!(
            code, "E-QUERY-002",
            "[LOW-002/POL-24] QueryPlanFailed code must be 'E-QUERY-002'; got '{code}'"
        );

        // Rule 1 invariance: message MUST be "Internal error" (map_prism_error MUST NOT change).
        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("message must be a string");
        assert_eq!(
            message, "Internal error",
            "[LOW-002/Rule-1] QueryPlanFailed message must be 'Internal error' \
             (Rule 1 redaction preserved; map_prism_error must NOT change for this variant); \
             got '{message}'"
        );
    }

    /// BC-2.10.007 LOW-002: `QueryDenylisted` → category `"internal"`.
    ///
    /// Previously fell to the `_ =>` catch-all with `category: "upstream_error"`.
    /// Denylist rejection is a Prism-side engine decision → `category: "internal"`.
    ///
    /// RED before implementation: current catch-all arm maps to `"upstream_error"`.
    #[test]
    fn test_BC_2_10_007_query_denylisted_category_is_internal() {
        let err = PrismError::QueryDenylisted {
            failure_count: 3,
            reason: "repeated execution failure".to_owned(),
            expiry_ts: 9_999_999_999,
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "internal",
            "[LOW-002] QueryDenylisted must map to category 'internal' (not 'upstream_error'); \
             got '{category}'"
        );

        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("code must be a string");
        assert_eq!(
            code, "E-QUERY-008",
            "[LOW-002/POL-24] QueryDenylisted code must be 'E-QUERY-008'; got '{code}'"
        );

        // Rule 1 invariance: message MUST be "Internal error" (map_prism_error MUST NOT change).
        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("message must be a string");
        assert_eq!(
            message, "Internal error",
            "[LOW-002/Rule-1] QueryDenylisted message must be 'Internal error' \
             (Rule 1 redaction preserved; map_prism_error must NOT change for this variant); \
             got '{message}'"
        );
    }

    /// BC-2.10.007 LOW-002: `QueryMemoryBudgetExceeded` → category `"internal"`.
    ///
    /// Previously fell to the `_ =>` catch-all with `category: "upstream_error"`.
    /// Memory pool exhaustion is a Prism DataFusion engine failure → `category: "internal"`.
    ///
    /// RED before implementation: current catch-all arm maps to `"upstream_error"`.
    #[test]
    fn test_BC_2_10_007_query_memory_budget_exceeded_category_is_internal() {
        let err = PrismError::QueryMemoryBudgetExceeded {
            limit_mb: 200,
            used_mb: 210,
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "internal",
            "[LOW-002] QueryMemoryBudgetExceeded must map to category 'internal' (not 'upstream_error'); \
             got '{category}'"
        );

        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("code must be a string");
        assert_eq!(
            code, "E-WATCHDOG-001",
            "[LOW-002/POL-24] QueryMemoryBudgetExceeded code must be 'E-WATCHDOG-001'; \
             got '{code}'"
        );

        // Rule 1 invariance: message MUST be "Internal error" (map_prism_error MUST NOT change).
        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("message must be a string");
        assert_eq!(
            message, "Internal error",
            "[LOW-002/Rule-1] QueryMemoryBudgetExceeded message must be 'Internal error' \
             (Rule 1 redaction preserved; map_prism_error must NOT change for this variant); \
             got '{message}'"
        );
    }

    // =========================================================================
    // F-MCPRS-PRL3-LOW-001 — Missing LOW-002 test vectors: E-QUERY-005, E-QUERY-010
    //
    // BC-2.10.007 §LOW-001: v1.13 §Canonical Test Vectors carried only 3 of 6
    // LOW-002 query engine vectors; QueryMaterializationLimitExceeded (E-QUERY-005) and
    // QueryVirtualFieldFailed (E-QUERY-010) were missing. Added here as GREEN locks
    // (the query engine arm shipped in a prior burst; these tests prove the arm covers
    // the full 6-variant set including the two previously untested variants).
    // =========================================================================

    /// BC-2.10.007 LOW-001 / LOW-002: `QueryMaterializationLimitExceeded` → category
    /// `"internal"`, code `"E-QUERY-005"`.
    ///
    /// This test was absent from prior bursts (LOW-001 gap). The implementation arm has
    /// been present since the LOW-002 fix; this test locks in the correct behavior.
    ///
    /// Per BC §Canonical Test Vectors: category "internal", original_params_valid: false,
    /// retryable: false, code "E-QUERY-005",
    /// suggestion "Prism query engine failure. Contact Prism operator; see audit log for details."
    #[test]
    fn test_BC_2_10_007_query_materialization_limit_exceeded_category_is_internal() {
        let err = PrismError::QueryMaterializationLimitExceeded {
            count: 10001,
            max: 10000,
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "internal",
            "[LOW-001/LOW-002] QueryMaterializationLimitExceeded must map to category \
             'internal' (not 'upstream_error'); got '{category}'"
        );

        let original_params_valid = error_obj
            .get("original_params_valid")
            .and_then(|v| v.as_bool())
            .expect("original_params_valid must be a bool");
        assert!(
            !original_params_valid,
            "[LOW-001/LOW-002] QueryMaterializationLimitExceeded must have original_params_valid:false"
        );

        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            !retryable,
            "[LOW-001/LOW-002] QueryMaterializationLimitExceeded must be retryable:false"
        );

        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("code must be a string");
        assert_eq!(
            code, "E-QUERY-005",
            "[LOW-001/LOW-002/POL-24] QueryMaterializationLimitExceeded code must be \
             byte-verbatim 'E-QUERY-005'; got '{code}'"
        );

        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("suggestion must be a string");
        assert_eq!(
            suggestion,
            "Prism query engine failure. Contact Prism operator; see audit log for details.",
            "[LOW-001/LOW-002/POL-24] QueryMaterializationLimitExceeded suggestion must be \
             byte-verbatim; got '{suggestion}'"
        );

        // Rule 1 invariance: message MUST be "Internal error" (map_prism_error MUST NOT change).
        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("message must be a string");
        assert_eq!(
            message, "Internal error",
            "[LOW-001/LOW-002/Rule-1] QueryMaterializationLimitExceeded message must be \
             'Internal error' (Rule 1 redaction preserved; map_prism_error must NOT change \
             for this variant); got '{message}'"
        );
    }

    /// BC-2.10.007 LOW-001 / LOW-002: `QueryVirtualFieldFailed` → category
    /// `"internal"`, code `"E-QUERY-010"`.
    ///
    /// This test was absent from prior bursts (LOW-001 gap). The implementation arm has
    /// been present since the LOW-002 fix; this test locks in the correct behavior.
    ///
    /// Per BC §Canonical Test Vectors: category "internal", original_params_valid: false,
    /// retryable: false, code "E-QUERY-010",
    /// suggestion "Prism query engine failure. Contact Prism operator; see audit log for details."
    #[test]
    fn test_BC_2_10_007_query_virtual_field_failed_category_is_internal() {
        let err = PrismError::QueryVirtualFieldFailed {
            field: "device_id".to_owned(),
            detail: "resolution failed".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "internal",
            "[LOW-001/LOW-002] QueryVirtualFieldFailed must map to category 'internal' \
             (not 'upstream_error'); got '{category}'"
        );

        let original_params_valid = error_obj
            .get("original_params_valid")
            .and_then(|v| v.as_bool())
            .expect("original_params_valid must be a bool");
        assert!(
            !original_params_valid,
            "[LOW-001/LOW-002] QueryVirtualFieldFailed must have original_params_valid:false"
        );

        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            !retryable,
            "[LOW-001/LOW-002] QueryVirtualFieldFailed must be retryable:false"
        );

        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("code must be a string");
        assert_eq!(
            code, "E-QUERY-010",
            "[LOW-001/LOW-002/POL-24] QueryVirtualFieldFailed code must be byte-verbatim \
             'E-QUERY-010'; got '{code}'"
        );

        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("suggestion must be a string");
        assert_eq!(
            suggestion,
            "Prism query engine failure. Contact Prism operator; see audit log for details.",
            "[LOW-001/LOW-002/POL-24] QueryVirtualFieldFailed suggestion must be \
             byte-verbatim; got '{suggestion}'"
        );

        // Rule 1 invariance: message MUST be "Internal error" (map_prism_error MUST NOT change).
        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("message must be a string");
        assert_eq!(
            message, "Internal error",
            "[LOW-001/LOW-002/Rule-1] QueryVirtualFieldFailed message must be 'Internal error' \
             (Rule 1 redaction preserved; map_prism_error must NOT change for this variant); \
             got '{message}'"
        );
    }

    // =========================================================================
    // F-MCPRS-PRL3-MED-001 — Safety boundary arm: SafetyContextContamination /
    // SafetyDataExfiltration → category "safety" (BC-2.10.007 §MED-001)
    //
    // RED before safety arm is added: both variants fall to the `_ =>` catch-all
    // with category "upstream_error" and code "E-INT-001".
    // GREEN after: dedicated safety arm routes them to category "safety" with
    // per-variant ec_code_override (E-SAFETY-001 / E-SAFETY-002) and
    // suggestion "Do not retry; report to operator."
    // =========================================================================

    /// BC-2.10.007 MED-001: `SafetyContextContamination` → category `"safety"`,
    /// code `"E-SAFETY-001"`, suggestion `"Do not retry; report to operator."`.
    ///
    /// RED before implementation: catch-all maps to `"upstream_error"` / `"E-INT-001"`.
    /// GREEN after: dedicated safety arm per BC §MED-001 exact VariantMeta.
    ///
    /// Rule 1 invariant preserved: `map_prism_error` still returns `"Internal error"` for
    /// this variant (verified by `message` field assertion). Rule 1 redaction is UNCHANGED.
    #[test]
    fn test_BC_2_10_007_safety_context_contamination_category_is_safety() {
        let err = PrismError::SafetyContextContamination {
            detail: "test contamination".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "safety",
            "[MED-001] SafetyContextContamination must map to category 'safety' \
             (not 'upstream_error'); got '{category}'"
        );

        let original_params_valid = error_obj
            .get("original_params_valid")
            .and_then(|v| v.as_bool())
            .expect("original_params_valid must be a bool");
        assert!(
            original_params_valid,
            "[MED-001] SafetyContextContamination must have original_params_valid:true \
             (safety layer detected malicious CONTENT, not malformed SHAPE)"
        );

        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            !retryable,
            "[MED-001] SafetyContextContamination must be retryable:false"
        );

        let upstream_message = error_obj.get("upstream_message");
        assert_eq!(
            upstream_message,
            Some(&serde_json::Value::Null),
            "[MED-001/DI-006] SafetyContextContamination upstream_message must be null \
             (present-as-null, not absent); got {upstream_message:?}"
        );

        let source = error_obj
            .get("source")
            .and_then(|v| v.as_str())
            .expect("source must be a string");
        assert_eq!(
            source, "prism_mcp",
            "[MED-001] SafetyContextContamination source must be 'prism_mcp'; got '{source}'"
        );

        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("code must be a string");
        assert_eq!(
            code, "E-SAFETY-001",
            "[MED-001/POL-24] SafetyContextContamination code must be byte-verbatim \
             'E-SAFETY-001' (ec_code_override required; map_prism_error returns \
             'Internal error' per Rule 1); got '{code}'"
        );

        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("suggestion must be a string");
        assert_eq!(
            suggestion, "Do not retry; report to operator.",
            "[MED-001/POL-24] SafetyContextContamination suggestion must be byte-verbatim \
             'Do not retry; report to operator.'; got '{suggestion}'"
        );

        // Rule 1 invariance: message MUST be "Internal error" (map_prism_error MUST NOT change).
        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("message must be a string");
        assert_eq!(
            message, "Internal error",
            "[MED-001/Rule-1] SafetyContextContamination message must be 'Internal error' \
             (Rule 1 redaction preserved; map_prism_error must NOT change for this variant); \
             got '{message}'"
        );
    }

    /// BC-2.10.007 MED-001: `SafetyDataExfiltration` → category `"safety"`,
    /// code `"E-SAFETY-002"`, suggestion `"Do not retry; report to operator."`.
    ///
    /// RED before implementation: catch-all maps to `"upstream_error"` / `"E-INT-001"`.
    /// GREEN after: dedicated safety arm per BC §MED-001 exact VariantMeta.
    ///
    /// Rule 1 invariant preserved: `map_prism_error` still returns `"Internal error"` for
    /// this variant (verified by `message` field assertion). Rule 1 redaction is UNCHANGED.
    #[test]
    fn test_BC_2_10_007_safety_data_exfiltration_category_is_safety() {
        let err = PrismError::SafetyDataExfiltration {
            field: "api_key".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "safety",
            "[MED-001] SafetyDataExfiltration must map to category 'safety' \
             (not 'upstream_error'); got '{category}'"
        );

        let original_params_valid = error_obj
            .get("original_params_valid")
            .and_then(|v| v.as_bool())
            .expect("original_params_valid must be a bool");
        assert!(
            original_params_valid,
            "[MED-001] SafetyDataExfiltration must have original_params_valid:true"
        );

        let retryable = error_obj
            .get("retryable")
            .and_then(|v| v.as_bool())
            .expect("retryable must be a bool");
        assert!(
            !retryable,
            "[MED-001] SafetyDataExfiltration must be retryable:false"
        );

        let upstream_message = error_obj.get("upstream_message");
        assert_eq!(
            upstream_message,
            Some(&serde_json::Value::Null),
            "[MED-001/DI-006] SafetyDataExfiltration upstream_message must be null; \
             got {upstream_message:?}"
        );

        let source = error_obj
            .get("source")
            .and_then(|v| v.as_str())
            .expect("source must be a string");
        assert_eq!(
            source, "prism_mcp",
            "[MED-001] SafetyDataExfiltration source must be 'prism_mcp'; got '{source}'"
        );

        let code = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .expect("code must be a string");
        assert_eq!(
            code, "E-SAFETY-002",
            "[MED-001/POL-24] SafetyDataExfiltration code must be byte-verbatim \
             'E-SAFETY-002'; got '{code}'"
        );

        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .expect("suggestion must be a string");
        assert_eq!(
            suggestion, "Do not retry; report to operator.",
            "[MED-001/POL-24] SafetyDataExfiltration suggestion must be byte-verbatim \
             'Do not retry; report to operator.'; got '{suggestion}'"
        );

        // Rule 1 invariance: message MUST be "Internal error" (map_prism_error MUST NOT change).
        let message = error_obj
            .get("message")
            .and_then(|v| v.as_str())
            .expect("message must be a string");
        assert_eq!(
            message, "Internal error",
            "[MED-001/Rule-1] SafetyDataExfiltration message must be 'Internal error' \
             (Rule 1 redaction preserved; map_prism_error must NOT change for this variant); \
             got '{message}'"
        );
    }

    /// BC-2.10.007 LOW-001 — not-safety regression guard.
    ///
    /// Proves the safety arm is correctly scoped to only the 2 safety variants.
    /// `PrismError::WritePartialFailure` must produce category `"upstream_error"` — NOT
    /// `"safety"`.
    ///
    /// F-MCPRS-PRL10-OBS-003: WritePartialFailure now has an EXPLICIT Group 4
    /// "upstream_error" arm (no longer falls to the catch-all). The assertion value
    /// "upstream_error" is unchanged — this test still guards against accidentally
    /// widening the safety arm to capture non-safety variants.
    #[test]
    fn test_BC_2_10_007_catch_all_category_is_not_safety_regression_guard() {
        let err = PrismError::WritePartialFailure {
            sensor: "crowdstrike".to_owned(),
            endpoint: "/devices/entities/devices/v2".to_owned(),
            failed: 3,
            total: 10,
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present (BC-2.10.007)");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");

        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "upstream_error",
            "[LOW-001/MED-001 regression guard] WritePartialFailure is a genuinely \
             catch-all variant and must produce category 'upstream_error' — NOT 'safety'. \
             If this fails, the safety arm incorrectly captured a non-safety variant. \
             Got '{category}'"
        );
        assert_ne!(
            category, "safety",
            "[LOW-001/MED-001 regression guard] WritePartialFailure must NOT produce \
             category 'safety'; safety arm must be scoped to SafetyContextContamination \
             and SafetyDataExfiltration ONLY"
        );
    }

    // ── F-MCPRS-PRL10-OBS-003 RED-first evidence ──────────────────────────────────────────────
    //
    // Three representative variants — one per new category group — written as RED tests
    // before the explicit arms are inserted. They FAIL under current catch-all behaviour
    // (all three produce "upstream_error") and turn GREEN after the explicit arms are added.

    /// F-MCPRS-PRL10-OBS-003 RED → GREEN: `PrismError::Infusion` must map to category
    /// `"internal"` via an explicit arm.
    ///
    /// RED under current code: `Infusion` hits the catch-all and produces `"upstream_error"`.
    /// GREEN after: the explicit Group 1 arm emits `"internal"`.
    #[test]
    fn test_F_MCPRS_PRL10_OBS_003_infusion_maps_to_internal_category() {
        let err = PrismError::Infusion(prism_core::error::InfusionError::UnknownInfusion {
            name: "test_infusion_obs003".to_owned(),
        });
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "internal",
            "F-MCPRS-PRL10-OBS-003: Infusion must map to 'internal' via explicit arm; \
             got '{category}'"
        );
    }

    /// F-MCPRS-PRL10-OBS-003 RED → GREEN: `PrismError::CredentialNotFound` must map to
    /// category `"configuration"` via an explicit arm.
    ///
    /// RED under current code: hits the catch-all → `"upstream_error"`.
    /// GREEN after: explicit Group 2 arm emits `"configuration"`.
    #[test]
    fn test_F_MCPRS_PRL10_OBS_003_credential_not_found_maps_to_configuration_category() {
        let err = PrismError::CredentialNotFound {
            name: "test_cred_obs003".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "configuration",
            "F-MCPRS-PRL10-OBS-003: CredentialNotFound must map to 'configuration' via explicit arm; \
             got '{category}'"
        );
    }

    /// F-MCPRS-PRL10-OBS-003 RED → GREEN: `PrismError::ScheduleCronInvalid` must map to
    /// category `"validation"` via an explicit arm.
    ///
    /// RED under current code: hits the catch-all → `"upstream_error"`.
    /// GREEN after: explicit Group 3 arm emits `"validation"`.
    #[test]
    fn test_F_MCPRS_PRL10_OBS_003_schedule_cron_invalid_maps_to_validation_category() {
        let err = PrismError::ScheduleCronInvalid {
            expr: "*/invalid".to_owned(),
            detail: "test obs003".to_owned(),
        };
        let result = prism_error_to_structured_call_result(err);
        let sc = result
            .structured_content
            .as_ref()
            .expect("structuredContent must be present");
        let error_obj = sc
            .get("error")
            .expect("structuredContent.error must be present");
        let category = error_obj
            .get("category")
            .and_then(|v| v.as_str())
            .expect("category must be a string");
        assert_eq!(
            category, "validation",
            "F-MCPRS-PRL10-OBS-003: ScheduleCronInvalid must map to 'validation' via explicit arm; \
             got '{category}'"
        );
    }

    // ── BC-2.10.007 §RETRYABLE-503 — transient/permanent boundary lock ─────────────────────────
    //
    // F-MCPRS-PRL9-LOW-001: the `matches!(status, 408|425|429|500|502|503|504)` whitelist had
    // retryable:true locks only for 503 and 429 as individual tests; a mutation dropping any
    // other code from the whitelist (e.g. 408, 500, 502, 504) would pass all tests undetected.
    //
    // This single parameterized test locks the FULL transient whitelist AND verifies the
    // permanent/auth boundary, including the key 5xx-but-not-whitelisted code 501.
    //
    // Anchor: BC-2.10.007 §RETRYABLE-503 (version-agnostic citation).

    /// BC-2.10.007 §RETRYABLE-503: full transient whitelist + permanent boundary lock.
    ///
    /// Transient (retryable:true): 408, 425, 429, 500, 502, 503, 504.
    /// Permanent (retryable:false): 400, 401, 403, 404, 422, 501.
    ///
    /// 501 is the critical 5xx-but-not-whitelisted boundary: a mutation that extends the
    /// whitelist to cover all 5xx would cause 501 to flip to retryable:true and fail here.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_10_007_sensor_http_retryable_whitelist_boundary_lock() {
        // (status_code, expected_retryable, label)
        let cases: &[(u16, bool, &str)] = &[
            // ── transient whitelist — all must be retryable:true ───────────────────────────
            (408, true, "408 Request Timeout"),
            (425, true, "425 Too Early"),
            (429, true, "429 Too Many Requests"),
            (500, true, "500 Internal Server Error"),
            (502, true, "502 Bad Gateway"),
            (503, true, "503 Service Unavailable"),
            (504, true, "504 Gateway Timeout"),
            // ── permanent / auth boundary — all must be retryable:false ───────────────────
            (400, false, "400 Bad Request"),
            (401, false, "401 Unauthorized (auth)"),
            (403, false, "403 Forbidden (auth)"),
            (404, false, "404 Not Found"),
            (422, false, "422 Unprocessable Entity"),
            // 501 is the key 5xx-but-not-whitelisted boundary: extending the whitelist to
            // cover all 5xx codes would flip this to true and break the lock.
            (501, false, "501 Not Implemented (5xx but NOT whitelisted)"),
        ];

        for &(status, expected_retryable, label) in cases {
            let err = PrismError::SensorHttpError {
                sensor: "test_sensor".to_owned(),
                status,
                body: label.to_owned(),
            };
            let result = prism_error_to_structured_call_result(err);
            let sc = result.structured_content.as_ref().unwrap_or_else(|| {
                panic!("[{label}] structuredContent must be present (BC-2.10.007 §RETRYABLE-503)")
            });
            let error_obj = sc
                .get("error")
                .unwrap_or_else(|| panic!("[{label}] structuredContent.error must be present"));
            let retryable = error_obj
                .get("retryable")
                .and_then(|v| v.as_bool())
                .unwrap_or_else(|| panic!("[{label}] retryable must be a bool"));
            assert_eq!(
                retryable,
                expected_retryable,
                "[{label}] SensorHttpError{{status:{status}}} must be retryable:{expected_retryable} \
                 per BC-2.10.007 §RETRYABLE-503 transient whitelist \
                 (408|425|429|500|502|503|504 → true; all others → false)"
            );
        }
    }
}
