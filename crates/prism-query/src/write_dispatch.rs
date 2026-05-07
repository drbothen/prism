//! Phase 5 Write Dispatch — intent log, semaphore, fan-out, per-record tracking.
//!
//! Implements the Phase 5 write dispatch pipeline (story §Task 6):
//!
//! a. Audit INTENT record (fail-closed): written before any sensor API contact.
//! b. Write semaphore acquisition: capacity 4 (separate from read semaphore).
//! c. Write fan-out: per-(client_id, sensor_name) pair via `SensorAdapter::write()`.
//! d. Per-record result tracking: partial failure is normal, not an abort.
//! e. Audit OUTCOME record: written after all records attempted.
//!
//! # Architecture Compliance
//! - Audit INTENT write is a synchronous `await` that resolves before any
//!   sensor HTTP call is initiated — no fire-and-forget (story §Architecture).
//! - Write semaphore capacity MUST be 4 — not shared with read semaphore (10).
//! - `SensorAdapter::write()` uses the same authenticated HTTP client as read.
//! - Partial batch failure does NOT abort the batch; errors go to `sensor_errors`.
//! - Only infrastructure failures (audit, semaphore) are `Err()` returns.
//!
//! Story: S-3.07 | BCs: BC-2.05.009, BC-2.04.007

// Stub module: all non-trivial bodies are todo!() pending implementation.
#![allow(dead_code, unused_variables)]

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::PrismError;
use prism_core::RiskTier;
use prism_sensors::AdapterRegistry;
use prism_spec_engine::write_endpoint::WriteEndpointSpec;
use tokio::sync::Semaphore;
use ulid::Ulid;

use crate::write_pipeline::{QueryContext, WritePlan};
use crate::write_result::{SensorWriteError, WriteResult};
use prism_sensors::RecordWriteResult;

/// Write semaphore capacity — MUST be 4 (story §Architecture Compliance Rule 3).
///
/// Separate from the read semaphore capacity of 10 from S-3.02.
/// Write operations are throttled more aggressively.
pub const WRITE_SEMAPHORE_CAPACITY: usize = 4;

// ---------------------------------------------------------------------------
// WriteDispatcher
// ---------------------------------------------------------------------------

/// Phase 5 write dispatcher.
///
/// Holds the write semaphore, adapter registry, and audit writer references
/// needed for the fail-closed intent → fan-out → outcome audit trail.
pub struct WriteDispatcher {
    /// Bounded write concurrency semaphore (capacity 4, per-`WriteExecutor` instance).
    pub(crate) write_semaphore: Arc<Semaphore>,
    /// Sensor adapter registry for write fan-out.
    pub(crate) adapter_registry: Arc<AdapterRegistry>,
    /// Audit writer for intent and outcome persistence (BC-2.05.009).
    pub(crate) audit_writer: Arc<dyn AuditWriter>,
}

// ---------------------------------------------------------------------------
// AuditWriter trait — abstraction over the RocksDB audit-buffer write
// ---------------------------------------------------------------------------

/// Trait abstracting write-phase audit persistence.
///
/// Implementations write `AuditEntry` records to the RocksDB `audit_buffer` CF
/// (AD-016 pattern). This trait is defined here (rather than in `prism-audit`)
/// to avoid circular dependency: prism-query → prism-audit, not the reverse.
///
/// # Fail-Closed Contract (BC-2.05.009)
/// `write_intent` MUST resolve before any sensor HTTP call. If it returns `Err`,
/// the entire write is aborted.
///
/// `write_outcome` failure is logged but does NOT unwind the write (sensor API
/// calls are already complete).
///
/// # Object Safety
/// This trait is object-safe via `#[async_trait]` — no generic methods.
#[async_trait]
pub trait AuditWriter: Send + Sync + 'static {
    /// Write the write INTENT record to the audit buffer.
    ///
    /// Called in Phase 5a, before any sensor API contact.
    /// Returns the assigned `audit_intent_id` on success.
    ///
    /// # Fail-Closed (BC-2.05.009)
    /// If this returns `Err(PrismError::AuditPersistenceFailed)`, the dispatcher
    /// MUST abort with `E-AUDIT-001` and MUST NOT contact any sensor API.
    async fn write_intent(
        &self,
        plan: &WritePlan,
        context: &QueryContext,
    ) -> Result<Ulid, PrismError>;

    /// Write the write OUTCOME record to the audit buffer.
    ///
    /// Called in Phase 5e, after all records have been attempted.
    ///
    /// # Non-unwinding on failure
    /// If this returns `Err`, the failure is logged but the write result is still
    /// returned to the caller — the sensor API calls are already complete.
    async fn write_outcome(&self, intent_id: Ulid, result: &WriteResult) -> Result<(), PrismError>;
}

// ---------------------------------------------------------------------------
// DispatchInputs — bundled Phase 5 arguments
// ---------------------------------------------------------------------------

/// Bundled inputs to `WriteDispatcher::dispatch` to stay within the 7-argument clippy limit.
pub struct DispatchInputs<'a> {
    /// Write plan for the current operation.
    pub plan: &'a WritePlan,
    /// Per-call context (client_id, org_slug, analyst_id).
    pub context: &'a QueryContext,
    /// Resolved risk tier from Phase 2.
    pub risk_tier: &'a RiskTier,
    /// Confirmation token ID consumed for irreversible operations.
    pub confirmed_by_token: Option<String>,
    /// Resolved write endpoint spec from prism-spec-engine.
    pub endpoint_spec: &'a WriteEndpointSpec,
    /// Records fetched in Phase 3 to be dispatched to the sensor API.
    pub fetched_records: &'a [RecordBatch],
    /// Resolved write endpoint identifier string.
    pub write_endpoint: &'a str,
}

// ---------------------------------------------------------------------------
// WriteDispatcher implementation
// ---------------------------------------------------------------------------

impl WriteDispatcher {
    /// Construct a `WriteDispatcher` with the provided dependencies.
    pub fn new(audit_writer: Arc<dyn AuditWriter>, adapter_registry: Arc<AdapterRegistry>) -> Self {
        Self {
            write_semaphore: Arc::new(Semaphore::new(WRITE_SEMAPHORE_CAPACITY)),
            adapter_registry,
            audit_writer,
        }
    }

    /// Execute Phase 5: write dispatch.
    ///
    /// Steps:
    /// 1. Write audit INTENT (fail-closed — abort if fails).
    /// 2. Acquire write semaphore permit.
    /// 3. Fan-out: call `SensorAdapter::write()` for each (client_id, sensor) pair.
    /// 4. Accumulate per-record results.
    /// 5. Write audit OUTCOME.
    /// 6. Construct and return `WriteResult`.
    ///
    /// # Error Returns
    /// Only infrastructure failures produce `Err()`:
    /// - `E-AUDIT-001` — audit intent write failed.
    /// - Semaphore exhausted — structured error (story §EC-04-002).
    ///
    /// Partial batch failure (some records fail) is NOT an error return.
    pub async fn dispatch(&self, inputs: DispatchInputs<'_>) -> Result<WriteResult, PrismError> {
        todo!("S-3.07 — WriteDispatcher::dispatch: Phase 5 intent → fan-out → outcome")
    }

    /// Phase 5a: write audit INTENT record (fail-closed).
    ///
    /// Returns the assigned `audit_intent_id` or `Err(E-AUDIT-001)`.
    async fn write_audit_intent(
        &self,
        plan: &WritePlan,
        context: &QueryContext,
    ) -> Result<Ulid, PrismError> {
        todo!("S-3.07 — WriteDispatcher::write_audit_intent")
    }

    /// Phase 5c: fan-out write calls to sensor adapters.
    ///
    /// Runs within the write semaphore permit (acquired in dispatch()).
    /// Parallel within the bound of `WRITE_SEMAPHORE_CAPACITY`.
    ///
    /// Returns accumulated per-record results and sensor errors.
    /// A failed per-record write does NOT abort the batch.
    async fn fan_out(
        &self,
        context: &QueryContext,
        endpoint_spec: &WriteEndpointSpec,
        records: &[RecordBatch],
    ) -> (Vec<RecordWriteResult>, Vec<SensorWriteError>) {
        todo!("S-3.07 — WriteDispatcher::fan_out: parallel sensor write calls")
    }

    /// Phase 5e: write audit OUTCOME record.
    ///
    /// Logs failure but does NOT unwind the write (calls are already complete).
    async fn write_audit_outcome(&self, intent_id: Ulid, result: &WriteResult) {
        todo!("S-3.07 — WriteDispatcher::write_audit_outcome")
    }
}
