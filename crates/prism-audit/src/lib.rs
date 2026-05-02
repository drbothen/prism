//! prism-audit — Compliance audit layer for the Prism platform (S-2.04, SS-05).
//!
//! This crate implements the audit middleware that wraps every MCP tool
//! invocation, constructing SOC 2 Type II and ISO 27001-compliant audit entries
//! with credential redaction and append-only storage into the RocksDB
//! `audit_buffer` CF.
//!
//! # Public API surface (S-2.04)
//!
//! - [`audit_entry::AuditEntry`] — compliance audit record
//! - [`audit_entry::AuditOutcome`] — success/failure outcome
//! - [`audit_entry::DataClassification`] — ISO 27001 data sensitivity levels
//! - [`audit_entry::CapabilityCheckRecord`] — per-capability check record
//! - [`audit_entry::CapabilityCheckResult`] — granted/denied decision
//! - [`audit_emitter::AuditEmitterLayer`] — Tower `Layer` factory
//! - [`audit_emitter::AuditEmitterService`] — Tower `Service` implementation
//! - [`audit_emitter::AuditedRequest`] — request envelope
//! - [`audit_emitter::AuditedResponse`] — response envelope
//! - [`audit_emitter::ToolClass`] — read vs. write classification
//! - [`audit_emitter::ToolClassificationRegistry`] — tool name → class map
//! - [`redaction::redact`] — credential redaction for parameters
//! - [`redaction::is_credential_key`] — key pattern classifier
//! - [`redaction::REDACTED_SENTINEL`] — `"[REDACTED]"` (BC-2.05.003 canonical, S-2.04 v1.5)
//! - [`write_audit::WriteAuditDetail`] — write-op detail embedded in parameters
//! - [`write_audit::WriteOutcome`] — committed/rolled-back/aborted/dry-run
//! - [`write_audit::CapabilityCheckResult`] — granted/denied (write-level)
//!
//! # Public API surface (S-2.05)
//!
//! - [`credential_events::CredentialAccessDetail`] — credential access event detail
//! - [`credential_events::CredentialAccessType`] — credential operation type
//! - [`credential_events::CredentialAccessResult`] — operation result
//! - [`credential_events::RequestingContext`] — parent invocation context
//! - [`credential_events::emit_credential_event`] — credential access emitter
//! - [`vector_compat::VectorAuditEntry`] — Vector pipeline newtype wrapper
//! - [`vector_compat::to_vector_json`] — produce Vector-compatible JSON view
//! - [`vector_compat::outcome_to_log_level`] — map AuditOutcome to log level string
//! - [`flag_events::FlagEvalDetail`] — feature flag evaluation detail
//! - [`flag_events::FlagResolutionStep`] — single resolution chain step
//! - [`flag_events::FlagEvalContext`] — write invocation context for flag eval
//! - [`flag_events::emit_flag_eval`] — flag evaluation emitter
//! - [`token_events::TokenLifecycleDetail`] — confirmation token lifecycle detail
//! - [`token_events::TokenEvent`] — token lifecycle event type
//! - [`token_events::TokenEventContext`] — original write operation context
//! - [`token_events::emit_token_generated`] — token issuance emitter
//! - [`token_events::emit_token_consumed`] — token consumption emitter
//! - [`token_events::emit_token_expired`] — token expiry emitter
//!
//! # Architecture compliance (S-2.04, non-negotiable)
//!
//! - Depends ONLY on `prism-core` and `prism-storage`.
//! - `AuditEmitter` is a Tower `Layer` — not ad-hoc function wrapping.
//! - `StorageDomain::AuditBuffer` is NEVER passed to `StorageBackend::remove()`.
//! - Credential redaction happens BEFORE serialization (redact() → AuditEntry).
//! - Fail-closed check for write tools occurs BEFORE the inner handler is called.

// cfg(kani) is set by the Kani verification toolchain, not by Cargo features.
#![allow(unexpected_cfgs)]

pub mod audit_emitter;
pub mod audit_entry;
pub mod credential_events;
pub mod flag_events;
pub mod org_slug_guard;
pub mod redaction;
pub mod token_events;
pub mod vector_compat;
pub mod write_audit;

// ── Test modules (empty declarations — bodies added by Test Writer) ───────────
#[cfg(test)]
pub mod tests;

// ── Re-exports ────────────────────────────────────────────────────────────────
pub use audit_emitter::{
    AuditEmitterLayer, AuditEmitterService, AuditedRequest, AuditedResponse, ToolClass,
    ToolClassificationRegistry,
};
pub use audit_entry::{
    AuditEntry, AuditOutcome, CapabilityCheckRecord, CapabilityCheckResult, DataClassification,
};
pub use credential_events::{
    emit_credential_event, CredentialAccessDetail, CredentialAccessResult, CredentialAccessType,
    RequestingContext,
};
pub use flag_events::{emit_flag_eval, FlagEvalContext, FlagEvalDetail, FlagResolutionStep};
pub use redaction::{is_credential_key, redact, REDACTED_SENTINEL};
pub use token_events::{
    emit_token_consumed, emit_token_expired, emit_token_generated, TokenEvent, TokenEventContext,
    TokenLifecycleDetail,
};
pub use vector_compat::{outcome_to_log_level, to_vector_json, VectorAuditEntry};
pub use write_audit::{WriteAuditDetail, WriteOutcome};
