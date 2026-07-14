//! Exhaustive `PrismError` variant → category sentinel.
//!
//! # Purpose
//!
//! `PrismError` is `#[non_exhaustive]`, which means **exhaustive** matching is only
//! possible from **within** the defining crate. Integration tests in `crates/*/tests/`
//! compile as separate crates and cannot omit the wildcard arm.
//!
//! This file lives inside `prism-core`'s `#[cfg(test)] pub mod tests` tree so that the
//! compiler enforces coverage at compile time — any new variant added to `PrismError`
//! must be given an arm here, or the build breaks.
//!
//! # Maintainer contract
//!
//! When you add a new variant to `PrismError` in `crates/prism-core/src/error.rs`:
//!
//! 1. Add a corresponding arm to the match in `assert_all_prism_error_variants_categorized`
//!    below with a comment showing its intended MCP category.
//! 2. Add a corresponding explicit arm to `prism_error_to_structured_call_result` in
//!    `crates/prism-mcp/src/error_mapping.rs` (Group 1–4 as appropriate).
//!
//! Failure to do step 1 → compile error here.
//! Failure to do step 2 → new variant silently falls to catch-all "upstream_error".
//!
//! Reference: F-MCPRS-PRL10-OBS-003 (fix-burst 22).

use crate::error::PrismError;

/// Exhaustive match over every `PrismError` variant without a wildcard arm.
///
/// This function is never called — it exists solely as a compile-time coverage
/// sentinel. If a new variant is added to `PrismError` and not listed here, the
/// build fails with an "non-exhaustive patterns" error.
///
/// The comment on each arm records the MCP structured-error category that variant
/// maps to in `prism_error_to_structured_call_result`. Keep the comment in sync
/// when changing an arm's category.
#[allow(dead_code)]
fn assert_all_prism_error_variants_categorized(err: PrismError) {
    match err {
        // ── E-AUTH: identity format / token validity ─────────────────────────
        PrismError::InvalidOrgSlug { .. } => {} // "authentication"
        PrismError::InvalidAnalystId { .. } => {} // "authentication"
        PrismError::InvalidClientId { .. } => {} // "authentication"
        PrismError::AuthTokenExpired => {}      // "authentication"
        PrismError::AuthTokenInvalid { .. } => {} // "authentication"
        PrismError::Unauthorized { .. } => {}   // "authentication"

        // ── E-SENSOR: upstream sensor HTTP/timeout/parse/rate ────────────────
        PrismError::SensorHttpError { .. } => {} // "upstream_error" (4xx→authentication; 503→transient; 429→transient)
        PrismError::SensorTimeout { .. } => {}   // "transient"
        PrismError::SensorResponseParse { .. } => {} // "upstream_error"
        PrismError::SensorRateLimited { .. } => {} // "transient"

        // ── E-OCSF: OCSF data-shape / protobuf / descriptor ─────────────────
        PrismError::OcsfFieldMissing { .. } => {} // "upstream_error"
        PrismError::OcsfFieldTypeMismatch { .. } => {} // "upstream_error"
        PrismError::OcsfUnknownClassUid { .. } => {} // "upstream_error"
        PrismError::OcsfProtobufEncode { .. } => {} // "internal"
        PrismError::OcsfProtobufDecode { .. } => {} // "internal"
        PrismError::OcsfUnknownEventClass { .. } => {} // "upstream_error"
        PrismError::OcsfNormalizationFailed { .. } => {} // "upstream_error"
        PrismError::OcsfDescriptorNotFound { .. } => {} // "internal"
        PrismError::OcsfUnknownRecordType { .. } => {} // "upstream_error"
        PrismError::OcsfTimestampParseError { .. } => {} // "upstream_error"

        // ── E-CRED: credential name / lookup / store / encryption ─────────────
        PrismError::InvalidCredentialName { .. } => {} // "configuration"
        PrismError::CredentialNotFound { .. } => {}    // "configuration"
        PrismError::CredentialAccessDenied { .. } => {} // "permission"
        PrismError::CredentialStoreError { .. } => {}  // "internal"
        PrismError::CredentialEncryptionError { .. } => {} // "internal"
        PrismError::EncryptionKeyMissing { .. } => {}  // "configuration"

        // ── E-IO ──────────────────────────────────────────────────────────────
        PrismError::Io(_) => {} // "internal"

        // ── E-FLAG / capability / token ───────────────────────────────────────
        PrismError::CapabilityDenied { .. } => {} // "permission"
        PrismError::WriteRequiresClientId => {}   // "permission"
        PrismError::FeatureFlagEvalError { .. } => {} // "permission"
        PrismError::TokenExpired { .. } => {}     // "permission"
        PrismError::TokenAlreadyConsumed { .. } => {} // "permission"
        PrismError::TokenContentHashMismatch { .. } => {} // "permission"
        PrismError::TokenCapExceeded => {}        // "permission"
        PrismError::TokenNotFound { .. } => {}    // "permission"
        PrismError::ConfirmClientIdMismatch { .. } => {} // "permission"

        // ── E-STORE: RocksDB / schema / cursor-cap ────────────────────────────
        PrismError::StorageOpenFailed { .. } => {} // "internal"
        PrismError::StorageWriteFailed { .. } => {} // "internal"
        PrismError::StorageReadFailed { .. } => {} // "internal"
        PrismError::StorageDomainNotFound { .. } => {} // "internal"
        PrismError::StorageKeyNotFound { .. } => {} // "internal"
        PrismError::StorageLockHeld { .. } => {}   // "internal"
        PrismError::StorageHealthCheckFailed { .. } => {} // "internal"
        PrismError::SchemaMismatch { .. } => {}    // "internal"
        PrismError::StorageBatchFailed { .. } => {} // "internal"
        PrismError::CursorCapExceeded => {}        // "internal"

        // ── E-CFG: configuration / capability-path ────────────────────────────
        PrismError::ClientNotFound { .. } => {} // "configuration"
        PrismError::ConfigNotFound { .. } => {} // "configuration"
        PrismError::ConfigParseFailed { .. } => {} // "configuration"
        PrismError::ConfigValidationFailed { .. } => {} // "configuration"
        PrismError::ConfigSnapshotStale { .. } => {} // "configuration"
        PrismError::InvalidCapabilityPath { .. } => {} // "validation"

        // ── E-MCP: tool / parameter / serialization / injection ──────────────
        PrismError::McpToolNotFound { .. } => {} // "validation"
        PrismError::McpParameterInvalid { .. } => {} // "validation"
        PrismError::McpSerializationError { .. } => {} // "internal"
        PrismError::McpPromptInjectionDetected { .. } => {} // "permission"

        // ── E-SAFETY: injection-detection boundary ───────────────────────────
        PrismError::SafetyContextContamination { .. } => {} // "safety"
        PrismError::SafetyDataExfiltration { .. } => {}     // "safety"

        // ── E-QUERY: parse / plan / type / security / execution ──────────────
        PrismError::QueryParseFailed { .. } => {} // "validation"
        PrismError::QueryPlanFailed { .. } => {}  // "internal"
        PrismError::QueryTypeMismatch { .. } => {} // "validation"
        PrismError::QuerySecurityLimitExceeded { .. } => {} // "validation"
        PrismError::QueryExecutionFailed { .. } => {} // "internal"
        PrismError::QueryMemoryBudgetExceeded { .. } => {} // "internal"
        PrismError::QueryTimeout { .. } => {}     // "transient"
        PrismError::QueryMaterializationLimitExceeded { .. } => {} // "internal"
        PrismError::QueryVirtualFieldFailed { .. } => {} // "internal"
        PrismError::WriteTargetCompositeSource { .. } => {} // "validation"
        PrismError::WriteBatchLimitExceeded { .. } => {} // "validation"
        PrismError::WriteUnbounded => {}          // "validation"
        PrismError::WriteTargetingInternalTable { .. } => {} // "validation"
        PrismError::WriteVerbNotAvailable { .. } => {} // "validation"
        PrismError::WriteAdapterNotConfiguredForClient { .. } => {} // "validation"
        PrismError::WriteTargetTableUnknown { .. } => {} // "validation"
        PrismError::WritePartialFailure { .. } => {} // "upstream_error"
        PrismError::QueryLimitExceeded { .. } => {} // "validation"
        PrismError::AuditTableAccessDenied => {}  // "permission"
        PrismError::CursorExpired => {}           // "validation"
        PrismError::CursorPageSizeInvalid => {}   // "validation"
        PrismError::CursorTokenUnknown => {}      // "validation"
        PrismError::UnknownSourceTable(_) => {}   // "validation"
        PrismError::ColumnNotFound(_) => {}       // "validation"
        PrismError::EnrichUdfNotFound(_) => {}    // "validation"
        PrismError::TableNotAvailable(_) => {}    // "validation"
        PrismError::SensorNotRegisteredForOrg { .. } => {} // "permission"
        PrismError::RedundantRowLimit { .. } => {} // "validation"
        PrismError::TemporalLiteralUnparseable { .. } => {} // "validation"
        PrismError::TemporalLiteralInvalidPosition { .. } => {} // "validation"
        PrismError::ExprInSubqueryProjectionNotSupported { .. } => {} // "validation"
        PrismError::QueryDenylisted { .. } => {}  // "internal"

        // ── E-SCHED: schedule management ─────────────────────────────────────
        PrismError::ScheduleNotFound { .. } => {} // "internal"
        PrismError::ScheduleConflict { .. } => {} // "validation"
        PrismError::ScheduleCronInvalid { .. } => {} // "validation"

        // ── E-DET: detection engine ──────────────────────────────────────────
        PrismError::DetectionRuleParseFailed { .. } => {} // "internal"
        PrismError::DetectionRuleNotFound { .. } => {}    // "internal"
        PrismError::DetectionStateCorrupt { .. } => {}    // "internal"

        // ── E-CASE: case management ──────────────────────────────────────────
        PrismError::CaseNotFound { .. } => {} // "internal"
        PrismError::CaseStateTransitionInvalid { .. } => {} // "validation"

        // ── E-WATCH: watchdog ────────────────────────────────────────────────
        PrismError::WatchdogHeartbeatMissed { .. } => {} // "internal"
        PrismError::WatchdogRestartLimitExceeded { .. } => {} // "internal"
        PrismError::WatchdogKilled { .. } => {}          // "internal"

        // ── E-SPEC: sensor spec hot-reload ───────────────────────────────────
        PrismError::Spec(_) => {}                     // "configuration"
        PrismError::SpecNotFound { .. } => {}         // "configuration"
        PrismError::SpecValidationFailed { .. } => {} // "configuration"
        PrismError::SpecHotReloadFailed { .. } => {}  // "configuration"

        // ── E-INFUSE / E-PLUGIN: framework failures ──────────────────────────
        PrismError::Infusion(_) => {} // "internal"
        PrismError::Plugin(_) => {}   // "internal"

        // ── E-IOC: IOC feed / lookup ──────────────────────────────────────────
        PrismError::IocFeedParseFailed { .. } => {} // "upstream_error"
        PrismError::IocLookupFailed { .. } => {}    // "upstream_error"

        // ── E-AUDIT: audit persistence ───────────────────────────────────────
        PrismError::AuditPersistenceFailed => {} // "transient"

        // ── E-ALIAS: alias registry ───────────────────────────────────────────
        PrismError::AliasNotFound { .. } => {} // "validation"
        PrismError::AliasCycleDetected { .. } => {} // "validation"
        PrismError::AliasDepthExceeded { .. } => {} // "validation"
        PrismError::AliasParameterInvalid { .. } => {} // "validation"
        PrismError::AliasDependentsExist { .. } => {} // "validation"
        PrismError::AliasNameConflict { .. } => {} // "validation"

        // ── E-INT: internal catch-all ─────────────────────────────────────────
        PrismError::Internal { .. } => {} // "internal"
    }
}
