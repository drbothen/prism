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
//! - [`redaction::REDACTED_SENTINEL`] — `"***REDACTED***"`
//! - [`write_audit::WriteAuditDetail`] — write-op detail embedded in parameters
//! - [`write_audit::WriteOutcome`] — committed/rolled-back/aborted/dry-run
//! - [`write_audit::CapabilityCheckResult`] — granted/denied (write-level)
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
pub mod redaction;
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
pub use redaction::{is_credential_key, redact, REDACTED_SENTINEL};
pub use write_audit::{WriteAuditDetail, WriteOutcome};
