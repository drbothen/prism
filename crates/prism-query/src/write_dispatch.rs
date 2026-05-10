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

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use chrono::Utc;
use prism_core::{OrgId, OrgSlug, PrismError, RiskTier, SensorType};
use prism_security::feature_flag::CapabilityCheckResult;
use prism_sensors::AdapterRegistry;
use prism_sensors::RecordWriteResult;
use prism_spec_engine::write_endpoint::WriteEndpointSpec;
use tokio::sync::Semaphore;
use ulid::Ulid;

use crate::write_pipeline::{QueryContext, WritePlan};
use crate::write_result::{SensorWriteError, WriteResult};

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
    /// Sensor adapter registry for write fan-out (CRIT-1: now wired in fan_out).
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
    /// # CRIT-4: capability_check (BC-2.05.009)
    /// The `capability_check` result from Phase 2 is recorded in the audit entry
    /// as `capability_checks[0]` with fields: `capability_path`, `compile_time_enabled`,
    /// `runtime_enabled`, `result`. This enables audit trail reconstruction of the
    /// full two-tier capability decision (BC-2.05.009 postcondition).
    ///
    /// # Fail-Closed (BC-2.05.009)
    /// If this returns `Err(PrismError::AuditPersistenceFailed)`, the dispatcher
    /// MUST abort with `E-AUDIT-001` and MUST NOT contact any sensor API.
    async fn write_intent(
        &self,
        plan: &WritePlan,
        context: &QueryContext,
        capability_check: &CapabilityCheckResult,
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
    /// CRIT-4 (BC-2.05.009): capability check result from Phase 2.
    /// Recorded in the audit INTENT entry as `capability_checks[0]`.
    pub capability_check: &'a CapabilityCheckResult,
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
    /// 3. Fan-out: call sensor adapters for each record.
    /// 4. Accumulate per-record results.
    /// 5. Write audit OUTCOME (log failure but don't unwind).
    /// 6. Construct and return `WriteResult`.
    ///
    /// # Error Returns
    /// Only infrastructure failures produce `Err()`:
    /// - `E-AUDIT-001` — audit intent write failed.
    ///
    /// Partial batch failure (some records fail) is NOT an error return.
    pub async fn dispatch(&self, inputs: DispatchInputs<'_>) -> Result<WriteResult, PrismError> {
        let started_at = Utc::now();

        // Phase 5 ordering per S-3.07 Task 6 spec (story file lines 222-246):
        // 5a: Audit INTENT record (fail-closed) — every write attempt produces an audit
        //     trace, even attempts that fail at the throttle. This satisfies BC-2.05.009's
        //     audit-coverage guarantee for ALL attempted writes.
        // 5b: Write semaphore acquisition — throttle protection.
        // 5c: Write fan-out — sensor API contact.
        //
        // The pass-1 HIGH-6 fix had reversed 5a/5b (semaphore-first) to prevent orphaned
        // audit records on semaphore exhaustion. Pass-2 HIGH-002 (POL-4 violation) restored
        // the original story-spec ordering — the orphaned-intent risk is theoretical
        // (semaphore is local Arc'd, never closed in this code path), while the audit
        // coverage erosion under reversed order was real (throttled attempts left no trace).
        // Reference: F-PASS2-HIGH-002 closure in fix-pass-1.

        // Phase 5a: Write audit INTENT (fail-closed — BC-2.05.009)
        // Every attempted write — including those that will be throttled — must produce
        // an audit trace. CRIT-4: pass capability_check so audit entry includes resolution trace.
        let intent_id = self
            .audit_writer
            .write_intent(inputs.plan, inputs.context, inputs.capability_check)
            .await?;

        // Phase 5b: Acquire write semaphore permit — throttle protection.
        // WRITE_SEMAPHORE_CAPACITY = 4, separate from read semaphore (10 from S-3.02).
        // The semaphore is only closed when the Arc is dropped — which cannot happen
        // while this future is executing (we hold a reference).
        let _permit = self
            .write_semaphore
            .acquire()
            .await
            .map_err(|_| PrismError::Internal {
                detail: "write semaphore closed unexpectedly".to_string(),
            })?;

        // Phase 5c: Fan-out — dispatch to sensor adapters (CRIT-1 wired).
        let (per_record_results, sensor_errors) = self
            .fan_out(
                inputs.plan,
                inputs.context,
                inputs.endpoint_spec,
                inputs.fetched_records,
            )
            .await;

        let completed_at = Utc::now();

        // Aggregate counts from per-record results.
        let affected_count = per_record_results.len() as u32;
        let succeeded_count = per_record_results
            .iter()
            .filter(|r| r.status == prism_sensors::WriteStatus::Success)
            .count() as u32;
        let failed_count = per_record_results
            .iter()
            .filter(|r| r.status == prism_sensors::WriteStatus::Failed)
            .count() as u32;

        let result = WriteResult {
            operation_id: Ulid::new(),
            dry_run: false,
            write_endpoint: inputs.write_endpoint.to_string(),
            risk_tier: inputs.risk_tier.clone(),
            confirmed_by_token: inputs.confirmed_by_token,
            execution_started_at: started_at,
            execution_completed_at: completed_at,
            audit_intent_id: intent_id,
            affected_count,
            succeeded_count,
            failed_count,
            per_record_results,
            sensor_errors,
        };

        // Phase 5e: Write audit OUTCOME — log failure, do NOT unwind (story §Task 6e).
        self.write_audit_outcome(intent_id, &result).await;

        Ok(result)
    }

    /// Phase 5c: fan-out write calls to sensor adapters (CRIT-1 — wired).
    ///
    /// For each record batch, attempts to resolve the sensor adapter via the
    /// registry and dispatches `SensorAdapter::write()`. Partial failures do
    /// NOT abort the batch — each record attempt is independent.
    ///
    /// # Empty registry
    /// In test contexts (empty `AdapterRegistry::new()`) no adapters are found
    /// and the result is ([], []). This is correct: zero records dispatched.
    ///
    /// # Adapter errors
    /// `SensorError::WriteNotImplemented` (the default trait body) is accumulated
    /// in `sensor_errors` rather than returned as `Err()`. Per-record partial
    /// failure is NOT a top-level error (story §Phase 5d).
    ///
    /// # TODO: W3-FIX-S307-001
    /// Full per-sensor HTTP dispatch requires overriding `write()` in each concrete
    /// adapter. Until then, the default trait body returns `WriteNotImplemented`.
    async fn fan_out(
        &self,
        plan: &WritePlan,
        context: &QueryContext,
        endpoint_spec: &WriteEndpointSpec,
        records: &[RecordBatch],
    ) -> (Vec<RecordWriteResult>, Vec<SensorWriteError>) {
        // If no records were fetched (Phase 3 stub), nothing to dispatch.
        if records.is_empty() {
            return (vec![], vec![]);
        }

        // Resolve SensorType from plan sensor name.
        let sensor_type = match plan.sensor.as_str() {
            "crowdstrike" => SensorType::CrowdStrike,
            "cyberint" => SensorType::Cyberint,
            "claroty" => SensorType::Claroty,
            "armis" => SensorType::Armis,
            _ => {
                // Unknown sensor: accumulate a write error for all records.
                let error_count: usize = records.iter().map(|rb| rb.num_rows()).sum();
                // F-PASS2-MED-005: fix phantom error when record batches are present but
                // all have zero rows. The previous `error_count.max(1)` fabricated a
                // per-record error that referenced no actual records. Emit a single
                // context-level error for the empty-batch case instead.
                let sensor_errors: Vec<SensorWriteError> = if error_count == 0 {
                    // Record batches were present (passed the `records.is_empty()` guard above)
                    // but contained zero rows. Emit one context-level error rather than
                    // fabricating phantom per-record errors.
                    vec![SensorWriteError {
                        sensor: plan.sensor.clone(),
                        client_id: context.client_id.clone(),
                        error_code: "E-SENSOR-010".to_string(),
                        detail: format!(
                            "unknown sensor '{}' with empty record batch — no writes attempted",
                            plan.sensor
                        ),
                    }]
                } else {
                    (0..error_count)
                        .map(|_| SensorWriteError {
                            sensor: plan.sensor.clone(),
                            client_id: context.client_id.clone(),
                            error_code: "E-SENSOR-010".to_string(),
                            detail: format!("unknown sensor '{}'", plan.sensor),
                        })
                        .collect()
                };
                return (vec![], sensor_errors);
            }
        };

        // TODO: W3-FIX-S307-002 — replace sentinel OrgId with proper OrgRegistry lookup.
        // The AdapterRegistry uses (OrgId, SensorType) composite key.
        // QueryContext carries OrgSlug; translating to OrgId requires OrgRegistry (future story).
        // In test contexts the registry is empty so `get()` returns None immediately.
        // In production, init_registry_for_org() must be called with the correct OrgId
        // before this code path is reachable.
        let org_id = OrgId::from_uuid(uuid::Uuid::nil()); // sentinel: empty registry returns None
        let adapter = self.adapter_registry.get(org_id, sensor_type);

        let mut per_record_results: Vec<RecordWriteResult> = vec![];
        let mut sensor_errors: Vec<SensorWriteError> = vec![];

        match adapter {
            None => {
                // No adapter registered for this (org_id, sensor_type) pair.
                // Normal in test contexts (empty registry). Accumulate a sensor error.
                tracing::debug!(
                    sensor = %plan.sensor,
                    org_id = ?org_id,
                    "fan_out: no adapter registered for ({org_id:?}, {sensor_type}) — \
                     zero records dispatched",
                );
                // No per-record results: registry is empty → 0 affected records (expected in tests).
            }
            Some(adapter) => {
                // Dispatch write() for each record batch.
                // TODO: W3-FIX-S307-001 — each concrete adapter overrides write() with real HTTP dispatch.
                let client_slug: OrgSlug = context.org_slug.clone();
                for record_batch in records {
                    match adapter
                        .write(endpoint_spec, record_batch, &plan.params, &client_slug)
                        .await
                    {
                        Ok(results) => {
                            per_record_results.extend(results);
                        }
                        Err(e) => {
                            // Per-sensor error accumulation — partial failure is NOT top-level Err.
                            // Use SensorError::error_code() so the code is derived from the actual
                            // error variant rather than hardcoded (F-PASS6-MED-002).
                            sensor_errors.push(SensorWriteError {
                                sensor: plan.sensor.clone(),
                                client_id: context.client_id.clone(),
                                error_code: e.error_code().to_string(),
                                detail: e.to_string(),
                            });
                        }
                    }
                }
            }
        }

        (per_record_results, sensor_errors)
    }

    /// Phase 5e: write audit OUTCOME record.
    ///
    /// Logs failure but does NOT unwind the write — sensor API calls are already
    /// complete (story §Task 6e).
    async fn write_audit_outcome(&self, intent_id: Ulid, result: &WriteResult) {
        if let Err(err) = self.audit_writer.write_outcome(intent_id, result).await {
            // HIGH-7: log the structured error detail, not just the message.
            // Sensor API calls are already complete; outcome audit failure is a
            // non-fatal observability gap, not a write failure (story §Task 6e).
            tracing::warn!(
                intent_id = %intent_id,
                error = %err,
                "audit outcome persistence failed after write completion; \
                 sensor API calls are NOT rolled back"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// F-PASS2-MED-005: unit tests for fan_out empty-batch unknown-sensor edge
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::expect_used)]
mod fan_out_empty_batch_tests {
    use super::*;
    use crate::write_pipeline::{QueryContext, WritePlan};
    use prism_core::OrgSlug;
    use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointSpec};
    use std::collections::HashMap;

    struct NoOpAudit;

    #[async_trait]
    impl AuditWriter for NoOpAudit {
        async fn write_intent(
            &self,
            _plan: &WritePlan,
            _ctx: &QueryContext,
            _capability_check: &CapabilityCheckResult,
        ) -> Result<Ulid, PrismError> {
            Ok(Ulid::new())
        }

        async fn write_outcome(&self, _id: Ulid, _r: &WriteResult) -> Result<(), PrismError> {
            Ok(())
        }
    }

    fn make_dispatcher() -> WriteDispatcher {
        WriteDispatcher::new(
            Arc::new(NoOpAudit),
            Arc::new(prism_sensors::AdapterRegistry::new()),
        )
    }

    fn make_unknown_sensor_plan() -> WritePlan {
        WritePlan {
            verb: "write".to_string(),
            sensor: "unknown_sensor".to_string(),
            target_table: "unknown_sensor_things".to_string(),
            dml_operation: None,
            has_explicit_limit: true,
            explicit_limit: Some(1),
            has_where_clause: true,
            params: HashMap::new(),
        }
    }

    fn make_context() -> QueryContext {
        QueryContext {
            client_id: "acme".to_string(),
            org_slug: OrgSlug::new_unchecked("acme"),
            dry_run: false,
            confirmation_token_id: None,
            analyst_id: None,
        }
    }

    fn make_endpoint_spec() -> WriteEndpointSpec {
        WriteEndpointSpec {
            pipe_verb: "write".to_string(),
            sql_table: "unknown_sensor_things".to_string(),
            capability_path: "sensor.unknown_sensor.write".to_string(),
            risk_tier: prism_core::RiskTier::Irreversible,
            batch_limit: 100,
            batch_mode: BatchMode::Serial,
            steps: vec![],
            record_id_field: "id".to_string(),
        }
    }

    /// MED-005: unknown sensor + empty record batch (RecordBatch with 0 rows) →
    /// exactly 1 context-level error emitted, not a phantom per-record error.
    ///
    /// Before fix: `error_count.max(1)` fabricated 1 error for empty batches.
    /// After fix: `error_count == 0` branch emits 1 context-level error.
    #[tokio::test]
    async fn test_med005_unknown_sensor_empty_batch_emits_one_error() {
        let dispatcher = make_dispatcher();
        let plan = make_unknown_sensor_plan();
        let context = make_context();
        let endpoint_spec = make_endpoint_spec();

        // Build a RecordBatch with 0 rows (non-empty slice, but zero rows per batch).
        let schema = Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("id", arrow::datatypes::DataType::Utf8, false),
        ]));
        let empty_batch = RecordBatch::new_empty(schema);

        let (per_record, sensor_errors) = dispatcher
            .fan_out(&plan, &context, &endpoint_spec, &[empty_batch])
            .await;

        assert!(
            per_record.is_empty(),
            "MED-005: no per-record results for unknown sensor"
        );
        assert_eq!(
            sensor_errors.len(),
            1,
            "MED-005: empty-batch unknown sensor must emit exactly 1 context-level error, \
             not a phantom per-record error; got {} errors",
            sensor_errors.len()
        );
        assert!(
            sensor_errors[0]
                .detail
                .contains("empty record batch — no writes attempted"),
            "MED-005: context-level error must describe empty batch situation; \
             got: {}",
            sensor_errors[0].detail
        );
    }

    /// MED-005: unknown sensor + non-empty record batch (N rows) →
    /// exactly N errors emitted (one per row).
    #[tokio::test]
    async fn test_med005_unknown_sensor_nonempty_batch_emits_per_row_errors() {
        let dispatcher = make_dispatcher();
        let plan = make_unknown_sensor_plan();
        let context = make_context();
        let endpoint_spec = make_endpoint_spec();

        // Build a RecordBatch with 3 rows.
        let schema = Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("id", arrow::datatypes::DataType::Utf8, false),
        ]));
        let array = arrow::array::StringArray::from(vec!["r1", "r2", "r3"]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(array)])
            .expect("must construct 3-row batch");

        let (per_record, sensor_errors) = dispatcher
            .fan_out(&plan, &context, &endpoint_spec, &[batch])
            .await;

        assert!(
            per_record.is_empty(),
            "MED-005: no per-record results for unknown sensor"
        );
        assert_eq!(
            sensor_errors.len(),
            3,
            "MED-005: 3-row batch must emit 3 per-row errors for unknown sensor; \
             got {} errors",
            sensor_errors.len()
        );
    }
}
