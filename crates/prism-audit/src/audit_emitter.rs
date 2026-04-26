//! AuditEmitter — Tower middleware wrapping every MCP tool invocation.
//!
//! Implements the fail-closed audit contract (BC-2.05.001):
//! - For **write tools**: if `emit()` fails, the write is aborted before
//!   the inner handler executes and `E-AUDIT-001` is returned.
//! - For **read tools**: if `emit()` fails, the failure is logged at ERROR
//!   level and the read proceeds.
//!
//! # Architecture compliance (S-2.04)
//!
//! - `AuditEmitter` is a Tower `Layer` + `Service` (not ad-hoc function wrapping).
//! - `emit()` is awaited BEFORE the inner handler for write tools — the write
//!   MUST NOT proceed without a successful audit record.
//! - `StorageDomain::AuditBuffer` is NEVER passed to `remove()` anywhere in this
//!   module. See the `#[deny(unused_must_use)]` lint note in `emit()`.
//!
//! # Storage
//!
//! Uses `prism_storage::audit_buffer::append_audit_entry()` (from S-2.02) rather
//! than `StorageBackend::put()` directly, ensuring the key format
//! (`audit:{ts}:{id}`) is consistent and overflow checks fire correctly.
//!
//! # Cloneability
//!
//! `AuditEmitterLayer<B>` wraps `Arc<B>` so cloning is cheap (Tower requires
//! `Clone` on service factories).

use std::sync::Arc;
use std::task::{Context, Poll};

use chrono::Utc;
use prism_core::PrismError;
use prism_storage::audit_buffer;
use prism_storage::backend::RocksStorageBackend;
use tower::{Layer, Service};
use uuid::Uuid;

use crate::audit_entry::{AuditEntry, AuditOutcome, CapabilityCheckRecord, DataClassification};
use crate::redaction::redact;

/// Classification of an MCP tool as read-only or write/mutation.
///
/// Used by `AuditEmitter` to apply the fail-closed contract for write tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolClass {
    /// Read-only tool — audit failure does not abort the operation.
    ReadTool,
    /// Write/mutation tool — audit failure aborts the operation (BC-2.05.001).
    WriteTool,
}

/// Registry mapping MCP tool names to their `ToolClass`.
///
/// Populated at startup from the MCP tool manifest. Static lifetime because
/// tool classification does not change at runtime.
pub type ToolClassificationRegistry = std::collections::HashMap<&'static str, ToolClass>;

/// Request envelope for the audited MCP service.
///
/// Carries the tool name, client_id, user_identity, (already-redacted)
/// parameters, and capability checks gathered before audit emission.
#[derive(Debug, Clone)]
pub struct AuditedRequest {
    pub tool_name: String,
    pub client_id: String,
    pub user_identity: String,
    /// Parameters with credential values already redacted (redact() called by caller).
    pub parameters: serde_json::Value,
    pub data_classification: DataClassification,
    pub capability_checks: Vec<CapabilityCheckRecord>,
    pub safety_flags: Vec<String>,
}

/// Response envelope from the inner handler.
#[derive(Debug, Clone)]
pub struct AuditedResponse {
    /// Raw outcome.
    pub outcome: AuditOutcome,
    /// Human-readable result summary (e.g., `"dry_run_preview"`, `"committed"`).
    pub result_summary: String,
    /// Structured error code if outcome is Failure.
    pub error_code: Option<String>,
}

// ── Tower Layer ───────────────────────────────────────────────────────────────

/// Tower `Layer` that wraps a service with the audit middleware.
///
/// `B` must implement `RocksStorageBackend` (the production storage trait from
/// S-2.01). Use `Arc<B>` for shared ownership across clones.
pub struct AuditEmitterLayer<B: RocksStorageBackend> {
    backend: Arc<B>,
    registry: Arc<ToolClassificationRegistry>,
}

impl<B: RocksStorageBackend> AuditEmitterLayer<B> {
    /// Construct an `AuditEmitterLayer` with the given storage backend and
    /// tool classification registry.
    pub fn new(backend: Arc<B>, registry: Arc<ToolClassificationRegistry>) -> Self {
        Self { backend, registry }
    }
}

impl<B, S> Layer<S> for AuditEmitterLayer<B>
where
    B: RocksStorageBackend,
    S: Clone,
{
    type Service = AuditEmitterService<B, S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuditEmitterService {
            inner,
            backend: Arc::clone(&self.backend),
            registry: Arc::clone(&self.registry),
        }
    }
}

impl<B: RocksStorageBackend> Clone for AuditEmitterLayer<B> {
    fn clone(&self) -> Self {
        Self {
            backend: Arc::clone(&self.backend),
            registry: Arc::clone(&self.registry),
        }
    }
}

// ── Tower Service ─────────────────────────────────────────────────────────────

/// Tower `Service` wrapping an MCP tool handler with the audit middleware.
pub struct AuditEmitterService<B: RocksStorageBackend, S> {
    inner: S,
    backend: Arc<B>,
    registry: Arc<ToolClassificationRegistry>,
}

impl<B: RocksStorageBackend, S: Clone> Clone for AuditEmitterService<B, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            backend: Arc::clone(&self.backend),
            registry: Arc::clone(&self.registry),
        }
    }
}

impl<B, S> Service<AuditedRequest> for AuditEmitterService<B, S>
where
    B: RocksStorageBackend + Clone,
    S: Service<AuditedRequest, Response = AuditedResponse, Error = PrismError>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = AuditedResponse;
    type Error = PrismError;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<AuditedResponse, PrismError>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: AuditedRequest) -> Self::Future {
        let tool_class = self
            .registry
            .get(req.tool_name.as_str())
            .copied()
            .unwrap_or(ToolClass::ReadTool);

        let backend = Arc::clone(&self.backend);
        let mut inner = self.inner.clone();
        let start_time = Utc::now();
        let trace_id = Uuid::now_v7();

        Box::pin(async move {
            // Redact parameters before they ever reach AuditEntry.
            // Redaction must occur before construction, not after.
            // (Architecture Compliance: credential redaction MUST happen before
            // serialization — BC-2.05.003, S-2.04 compliance rule.)
            let redacted_params = redact(req.parameters.clone());

            // For WRITE tools: emit audit BEFORE calling inner handler (fail-closed).
            if tool_class == ToolClass::WriteTool {
                let pre_entry =
                    build_pre_invocation_entry(trace_id, start_time, &req, redacted_params.clone());
                if let Err(e) = emit(&*backend, &pre_entry) {
                    // Audit persistence failed — abort the write (BC-2.05.001).
                    tracing::error!(
                        trace_id = %trace_id,
                        tool = %req.tool_name,
                        error = %e,
                        "audit emission failed for write tool — write ABORTED (E-AUDIT-001)"
                    );
                    return Err(PrismError::AuditPersistenceFailed);
                }
            }

            // Call the inner handler.
            let inner_start = Utc::now();
            let inner_result = inner.call(req.clone()).await;
            let duration_ms = (Utc::now() - inner_start).num_milliseconds().max(0) as u64;

            let (response, outcome, result_summary, error_code) = match inner_result {
                Ok(resp) => {
                    let os = resp.outcome.clone();
                    let rs = resp.result_summary.clone();
                    let ec = resp.error_code.clone();
                    (resp, os, rs, ec)
                }
                Err(ref e) => {
                    let outcome = AuditOutcome::Failure {
                        error_code: e.to_string(),
                    };
                    let result_summary = format!("error: {e}");
                    let error_code = Some(e.to_string());
                    // We still need to emit a completion entry for read tools;
                    // for write tools the pre-entry was already emitted.
                    let _ = (outcome.clone(), result_summary.clone(), error_code.clone());
                    // Re-derive for the emit below.
                    (
                        AuditedResponse {
                            outcome: AuditOutcome::Failure {
                                error_code: e.to_string(),
                            },
                            result_summary: format!("error: {e}"),
                            error_code: Some(e.to_string()),
                        },
                        AuditOutcome::Failure {
                            error_code: e.to_string(),
                        },
                        format!("error: {e}"),
                        Some(e.to_string()),
                    )
                }
            };

            // Emit completion entry (for read tools; for write tools this is
            // the "post" entry recording the actual outcome).
            let completion_entry = AuditEntry::new(
                trace_id,
                start_time,
                req.tool_name.clone(),
                req.client_id.clone(),
                req.user_identity.clone(),
                redacted_params,
                outcome,
                result_summary,
                duration_ms,
                error_code,
                req.data_classification.clone(),
                req.capability_checks.clone(),
                req.safety_flags.clone(),
            );

            if let Err(e) = emit(&*backend, &completion_entry) {
                if tool_class == ToolClass::ReadTool {
                    // Read tools: log failure but do NOT abort (BC-2.05.001).
                    tracing::error!(
                        trace_id = %trace_id,
                        tool = %req.tool_name,
                        error = %e,
                        "audit emission failed for read tool — read proceeds with audit_warning"
                    );
                    // The caller receives a response with _meta.audit_warning set,
                    // but that annotation is applied at the MCP transport layer, not here.
                } else {
                    // Write tools already emitted pre-entry; log but don't abort again.
                    tracing::warn!(
                        trace_id = %trace_id,
                        tool = %req.tool_name,
                        error = %e,
                        "audit completion entry failed after write — pre-entry already persisted"
                    );
                }
            }

            Ok(response)
        })
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Persist an `AuditEntry` to the `audit_buffer` CF via `append_audit_entry`.
///
/// Uses the S-2.02 `append_audit_entry()` function for consistent key format
/// and overflow checking. Returns `PrismError::AuditPersistenceFailed` if the
/// underlying storage write fails.
///
/// # Append-only invariant (BC-2.05.006)
///
/// This function ONLY calls `append_audit_entry()` — it NEVER calls
/// `StorageBackend::remove()` with `StorageDomain::AuditBuffer`.
/// DO NOT add any `remove()` call for `AuditBuffer` in this crate.
fn emit<B: RocksStorageBackend>(backend: &B, entry: &AuditEntry) -> Result<(), PrismError> {
    // todo!("AC-1 / BC-2.05.001: serialize AuditEntry and call append_audit_entry")
    //
    // Translate the compliance AuditEntry (this crate) to the storage-layer
    // AuditEntry (prism-storage::audit_buffer::AuditEntry) and persist via
    // append_audit_entry(). The two types are distinct: this crate's type has
    // full compliance fields; the storage type is a lightweight key+payload envelope.
    let timestamp_ns = entry.timestamp.timestamp_nanos_opt().unwrap_or(0) as u64;
    let payload_json = serde_json::to_string(entry).map_err(|e| PrismError::Internal {
        detail: format!("AuditEntry serialization failed: {e}"),
    })?;
    let mut payload_map = std::collections::BTreeMap::new();
    payload_map.insert("json".to_owned(), payload_json);

    let storage_entry = audit_buffer::AuditEntry {
        timestamp_ns,
        trace_id: entry.trace_id.to_string(),
        payload: payload_map,
    };

    audit_buffer::append_audit_entry(backend, &storage_entry)
        .map_err(|_| PrismError::AuditPersistenceFailed)
}

/// Build a pre-invocation `AuditEntry` for write tools (emitted BEFORE inner
/// handler is called). Records that the write is about to execute with the
/// given capability checks.
fn build_pre_invocation_entry(
    trace_id: Uuid,
    timestamp: chrono::DateTime<Utc>,
    req: &AuditedRequest,
    redacted_params: serde_json::Value,
) -> AuditEntry {
    // todo!("AC-2 / BC-2.05.001: build pre-invocation entry for write fail-closed contract")
    AuditEntry::new(
        trace_id,
        timestamp,
        req.tool_name.clone(),
        req.client_id.clone(),
        req.user_identity.clone(),
        redacted_params,
        AuditOutcome::Success, // Preliminary — will be updated by completion entry.
        "write_pre_invocation".to_owned(),
        0,
        None,
        req.data_classification.clone(),
        req.capability_checks.clone(),
        req.safety_flags.clone(),
    )
}

#[cfg(test)]
mod tests {}
