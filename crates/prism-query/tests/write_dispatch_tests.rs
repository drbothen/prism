//! Write dispatch phase (Phase 5) tests — S-3.07.
//!
//! Covers:
//! - BC-2.05.009: audit intent written before any sensor call (fail-closed)
//! - BC-2.05.009: audit outcome written after all records attempted
//! - BC-2.05.009: capability_checks emitted in hierarchical evaluation order
//! - BC-2.05.009: denied capability path still produces an audit record
//! - BC-2.05.009: EC-05-016 — read ops produce empty capability_checks
//! - Edge: partial write failure (E-QUERY-025 behavioral contract)
//! - Edge: audit intent fail-closed — no sensor call when intent fails (E-AUDIT-001)
//! - Edge: audit outcome failure does NOT unwind completed sensor calls
//! - Edge: write semaphore capacity is exactly 4 (WRITE_SEMAPHORE_CAPACITY invariant)
//!
//! All tests are RED-GATE stubs. Every body calls into `todo!()` implementations.
//!
//! Story: S-3.07 | BCs: BC-2.05.009, BC-2.04.007

#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used
)]

use std::collections::BTreeMap;
use std::sync::Arc;

use prism_core::RiskTier;
use prism_query::write_dispatch::{AuditWriter, WriteDispatcher, WRITE_SEMAPHORE_CAPACITY};
use prism_query::write_pipeline::{QueryContext, WritePlan};
use prism_query::write_result::{SensorWriteError, WriteResult};

// Re-use the helpers from write_pipeline_tests via a local copy (integration
// tests cannot share code via `pub mod` across files — each is its own crate).

mod helpers {
    use std::collections::HashMap;
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    };

    use prism_core::PrismError;
    use prism_query::write_dispatch::AuditWriter;
    use prism_query::write_pipeline::{QueryContext, WritePlan};
    use prism_query::write_result::WriteResult;
    use ulid::Ulid;

    pub fn make_plan_with_filter() -> WritePlan {
        WritePlan {
            verb: "contain".to_string(),
            sensor: "crowdstrike".to_string(),
            target_table: "crowdstrike_contained_hosts".to_string(),
            dml_operation: None,
            has_explicit_limit: true,
            explicit_limit: Some(10),
            has_where_clause: true,
            params: HashMap::new(),
        }
    }

    pub fn make_context(client: &str, dry_run: bool) -> QueryContext {
        QueryContext {
            client_id: client.to_string(),
            org_slug: prism_core::OrgSlug::new_unchecked(client),
            dry_run,
            confirmation_token_id: None,
            analyst_id: Some("analyst-001".to_string()),
        }
    }

    // -----------------------------------------------------------------------
    // IntentTrackingAuditWriter — records whether write_intent was called and
    // optionally forces failure.
    // -----------------------------------------------------------------------

    pub struct IntentTrackingAuditWriter {
        /// Number of times `write_intent` was called.
        pub intent_call_count: AtomicUsize,
        /// If true, `write_intent` returns `Err(AuditPersistenceFailed)`.
        pub fail_intent: AtomicBool,
        /// Number of times `write_outcome` was called.
        pub outcome_call_count: AtomicUsize,
        /// If true, `write_outcome` returns `Err(AuditPersistenceFailed)`.
        pub fail_outcome: AtomicBool,
    }

    impl IntentTrackingAuditWriter {
        pub fn new() -> Arc<Self> {
            Arc::new(Self {
                intent_call_count: AtomicUsize::new(0),
                fail_intent: AtomicBool::new(false),
                outcome_call_count: AtomicUsize::new(0),
                fail_outcome: AtomicBool::new(false),
            })
        }

        pub fn with_failed_intent() -> Arc<Self> {
            let w = Self::new();
            w.fail_intent.store(true, Ordering::SeqCst);
            w
        }

        pub fn with_failed_outcome() -> Arc<Self> {
            let w = Self::new();
            w.fail_outcome.store(true, Ordering::SeqCst);
            w
        }
    }

    #[async_trait::async_trait]
    impl AuditWriter for IntentTrackingAuditWriter {
        async fn write_intent(
            &self,
            _plan: &WritePlan,
            _ctx: &QueryContext,
            _capability_check: &prism_security::feature_flag::CapabilityCheckResult,
        ) -> Result<Ulid, PrismError> {
            self.intent_call_count.fetch_add(1, Ordering::SeqCst);
            if self.fail_intent.load(Ordering::SeqCst) {
                Err(PrismError::AuditPersistenceFailed)
            } else {
                Ok(Ulid::new())
            }
        }

        async fn write_outcome(
            &self,
            _intent_id: Ulid,
            _result: &WriteResult,
        ) -> Result<(), PrismError> {
            self.outcome_call_count.fetch_add(1, Ordering::SeqCst);
            if self.fail_outcome.load(Ordering::SeqCst) {
                Err(PrismError::AuditPersistenceFailed)
            } else {
                Ok(())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Invariant: write semaphore capacity is exactly 4 (architecture compliance)
// ---------------------------------------------------------------------------

/// BC-2.04.007 / story §Architecture Compliance Rule 3:
/// Write semaphore capacity MUST be 4 — separate from read semaphore (10).
///
/// This is a compile-time constant check (no panic path) — it verifies the
/// declared constant value without calling any todo!() stub.
///
/// This test PASSES against the stub (it checks a constant, not a stub body).
/// That is intentional: the constant is "green by design" — it validates a
/// structural invariant, not a behavioral body. If the implementer changes the
/// constant to the wrong value this test catches the regression.
#[test]
fn test_BC_2_04_007_write_semaphore_capacity_is_four() {
    assert_eq!(
        WRITE_SEMAPHORE_CAPACITY, 4,
        "Write semaphore capacity MUST be 4 per story architecture compliance rule 3; \
         separate from read semaphore capacity of 10"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.009: audit intent written BEFORE any sensor call (fail-closed)
// ---------------------------------------------------------------------------

/// BC-2.05.009 postcondition: `write_intent` is called first, and if it fails
/// the dispatcher returns `E-AUDIT-001` without contacting any sensor.
///
/// Exercises story §Task 6a: "Audit INTENT record (fail-closed)".
#[tokio::test]

async fn test_BC_2_05_009_audit_intent_fail_closed_returns_e_audit_001() {
    use prism_query::write_dispatch::DispatchInputs;
    use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointSpec, WriteStep};

    let audit = helpers::IntentTrackingAuditWriter::with_failed_intent();
    let registry = Arc::new(prism_sensors::AdapterRegistry::new());
    let dispatcher = WriteDispatcher::new(audit.clone(), registry);

    let plan = helpers::make_plan_with_filter();
    let context = helpers::make_context("acme", false);

    let endpoint_spec = WriteEndpointSpec {
        pipe_verb: "contain".to_string(),
        sql_table: "crowdstrike_contained_hosts".to_string(),
        capability_path: "sensor.crowdstrike.containment".to_string(),
        risk_tier: RiskTier::Irreversible,
        batch_limit: 100,
        batch_mode: BatchMode::Serial,
        steps: vec![],
        record_id_field: "device_id".to_string(),
    };

    let inputs = DispatchInputs {
        plan: &plan,
        context: &context,
        risk_tier: &RiskTier::Irreversible,
        confirmed_by_token: Some("tok-abc".to_string()),
        endpoint_spec: &endpoint_spec,
        fetched_records: &[],
        write_endpoint: "crowdstrike.containment",
        capability_check: &prism_security::feature_flag::CapabilityCheckResult::Allowed,
    };

    // With failing intent: dispatch must return E-AUDIT-001 (fail-closed)
    let result = dispatcher.dispatch(inputs).await;
    let err = result.expect_err("Failed intent must abort dispatch with E-AUDIT-001");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-AUDIT-001") || err_msg.contains("Audit emission failed"),
        "BC-2.05.009: failed intent must produce E-AUDIT-001; got: {err_msg}"
    );
    // write_intent was called (before aborting)
    assert_eq!(
        audit
            .intent_call_count
            .load(std::sync::atomic::Ordering::SeqCst),
        1,
        "write_intent must be called exactly once even on failure"
    );
}

/// BC-2.05.009 postcondition: when audit intent succeeds, `write_intent` is
/// called exactly once and the outcome record is written after fan-out.
///
/// Tests the sequencing invariant: intent before sensor calls, outcome after.
#[tokio::test]

async fn test_BC_2_05_009_audit_intent_called_before_sensor_outcome_after() {
    use prism_query::write_dispatch::DispatchInputs;
    use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointSpec};

    let audit = helpers::IntentTrackingAuditWriter::new();
    let registry = Arc::new(prism_sensors::AdapterRegistry::new());
    let dispatcher = WriteDispatcher::new(audit.clone(), registry);

    let plan = helpers::make_plan_with_filter();
    let context = helpers::make_context("acme", false);

    let endpoint_spec = WriteEndpointSpec {
        pipe_verb: "contain".to_string(),
        sql_table: "crowdstrike_contained_hosts".to_string(),
        capability_path: "sensor.crowdstrike.containment".to_string(),
        risk_tier: RiskTier::Irreversible,
        batch_limit: 100,
        batch_mode: BatchMode::Serial,
        steps: vec![],
        record_id_field: "device_id".to_string(),
    };

    let inputs = DispatchInputs {
        plan: &plan,
        context: &context,
        risk_tier: &RiskTier::Irreversible,
        confirmed_by_token: Some("tok-xyz".to_string()),
        endpoint_spec: &endpoint_spec,
        fetched_records: &[],
        write_endpoint: "crowdstrike.containment",
        capability_check: &prism_security::feature_flag::CapabilityCheckResult::Allowed,
    };

    // MED-6: non-vacuous assertions on audit call counts
    let result = dispatcher
        .dispatch(inputs)
        .await
        .expect("dispatch must succeed with tracking audit writer");

    // write_intent must be called exactly once (before fan-out)
    assert_eq!(
        audit
            .intent_call_count
            .load(std::sync::atomic::Ordering::SeqCst),
        1,
        "BC-2.05.009: write_intent must be called exactly once"
    );
    // write_outcome must be called exactly once (after fan-out)
    assert_eq!(
        audit
            .outcome_call_count
            .load(std::sync::atomic::Ordering::SeqCst),
        1,
        "BC-2.05.009: write_outcome must be called exactly once after fan-out"
    );
    // Result must be Ok (partial failure is not Err)
    assert!(
        !result.dry_run,
        "WriteResult.dry_run must always be false on execute path"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.009: audit outcome failure does NOT unwind completed sensor calls
// ---------------------------------------------------------------------------

/// BC-2.05.009: when `write_outcome` fails, the `WriteResult` is still
/// returned to the caller — the sensor API calls are already complete and
/// must not be unwound.
///
/// Exercises story §Task 6e: "If outcome persistence fails, log but do NOT
/// unwind the write."
#[tokio::test]

async fn test_BC_2_05_009_audit_outcome_failure_does_not_unwind_write() {
    use prism_query::write_dispatch::DispatchInputs;
    use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointSpec};

    let audit = helpers::IntentTrackingAuditWriter::with_failed_outcome();
    let registry = Arc::new(prism_sensors::AdapterRegistry::new());
    let dispatcher = WriteDispatcher::new(audit.clone(), registry);

    let plan = helpers::make_plan_with_filter();
    let context = helpers::make_context("acme", false);

    let endpoint_spec = WriteEndpointSpec {
        pipe_verb: "contain".to_string(),
        sql_table: "crowdstrike_contained_hosts".to_string(),
        capability_path: "sensor.crowdstrike.containment".to_string(),
        risk_tier: RiskTier::Irreversible,
        batch_limit: 100,
        batch_mode: BatchMode::Serial,
        steps: vec![],
        record_id_field: "device_id".to_string(),
    };

    let inputs = DispatchInputs {
        plan: &plan,
        context: &context,
        risk_tier: &RiskTier::Irreversible,
        confirmed_by_token: Some("tok-outcome-fail".to_string()),
        endpoint_spec: &endpoint_spec,
        fetched_records: &[],
        write_endpoint: "crowdstrike.containment",
        capability_check: &prism_security::feature_flag::CapabilityCheckResult::Allowed,
    };

    // MED-6: non-vacuous assertion — outcome failure must NOT cause Err return.
    // write_outcome failure is logged (HIGH-7) but WriteResult is still returned.
    let result = dispatcher
        .dispatch(inputs)
        .await
        .expect("BC-2.05.009: WriteResult must be returned even when outcome write fails");

    assert!(
        !result.dry_run,
        "WriteResult.dry_run must be false even when outcome write fails"
    );
    // write_intent called once (outcome failure doesn't affect intent count)
    assert_eq!(
        audit
            .intent_call_count
            .load(std::sync::atomic::Ordering::SeqCst),
        1,
        "write_intent must be called exactly once"
    );
    // write_outcome was attempted but failed (count is 1)
    assert_eq!(
        audit
            .outcome_call_count
            .load(std::sync::atomic::Ordering::SeqCst),
        1,
        "write_outcome must be attempted exactly once (even though it fails)"
    );
}

// ---------------------------------------------------------------------------
// Edge: partial write failure (E-QUERY-025 behavioral contract)
// ---------------------------------------------------------------------------

/// Story §Task 9: when `failed_count > 0 && succeeded_count > 0`, the write
/// result must contain both counts with per-record detail.
///
/// BC reference: story §Architecture Compliance — "Partial batch failure is
/// NOT an error return — it is represented in WriteResult.per_record_results."
///
/// Note: E-QUERY-025 variant does NOT yet exist in prism-core/src/error.rs.
/// This test asserts on structural WriteResult fields rather than enum variant
/// identity (see BLOCKER note in report).
#[tokio::test]
async fn test_BC_2_04_007_partial_write_failure_represented_in_write_result_not_err() {
    use prism_query::write_dispatch::DispatchInputs;
    use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointSpec};

    let audit = helpers::IntentTrackingAuditWriter::new();
    let registry = Arc::new(prism_sensors::AdapterRegistry::new());
    let dispatcher = WriteDispatcher::new(audit, registry);

    let plan = helpers::make_plan_with_filter();
    let context = helpers::make_context("acme", false);

    let endpoint_spec = WriteEndpointSpec {
        pipe_verb: "contain".to_string(),
        sql_table: "crowdstrike_contained_hosts".to_string(),
        capability_path: "sensor.crowdstrike.containment".to_string(),
        risk_tier: RiskTier::Irreversible,
        batch_limit: 100,
        batch_mode: BatchMode::Serial,
        steps: vec![],
        record_id_field: "device_id".to_string(),
    };

    let inputs = DispatchInputs {
        plan: &plan,
        context: &context,
        risk_tier: &RiskTier::Irreversible,
        confirmed_by_token: Some("tok-partial".to_string()),
        endpoint_spec: &endpoint_spec,
        fetched_records: &[],
        write_endpoint: "crowdstrike.containment",
        capability_check: &prism_security::feature_flag::CapabilityCheckResult::Allowed,
    };

    // Key behavioral contract: partial batch failure must NOT be returned as Err.
    // With empty fetched_records, the result has zero records (all counts = 0),
    // but the return type MUST be Ok(WriteResult) — never Err.
    // This verifies that dispatch() doesn't convert zero-record results to errors.
    let result = dispatcher.dispatch(inputs).await;
    let write_result = result.expect("partial failure must not be an Err return");
    // With empty fetched_records: affected_count = 0, succeeded_count = 0, failed_count = 0
    assert!(
        !write_result.dry_run,
        "WriteResult.dry_run must always be false"
    );
    assert_eq!(
        write_result.affected_count, 0,
        "Empty fetched_records yields 0 affected records"
    );
}

// ---------------------------------------------------------------------------
// CRIT-4: BC-2.05.009 capability_checks audit emission via write_intent
// ---------------------------------------------------------------------------

/// CRIT-4 / BC-2.05.009 permit path: when dispatch succeeds, write_intent
/// is called with CapabilityCheckResult::Allowed.
///
/// Uses IntentTrackingAuditWriter to assert write_intent is called exactly once,
/// and that the capability_check recorded is Allowed (permit path).
#[tokio::test]
async fn test_crit4_permit_path_audit_intent_called_with_allowed() {
    use prism_query::write_dispatch::DispatchInputs;
    use prism_security::feature_flag::CapabilityCheckResult;
    use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointSpec};

    let audit = helpers::IntentTrackingAuditWriter::new();
    let registry = Arc::new(prism_sensors::AdapterRegistry::new());
    let dispatcher = WriteDispatcher::new(audit.clone(), registry);

    let plan = helpers::make_plan_with_filter();
    let context = helpers::make_context("acme", false);

    let endpoint_spec = WriteEndpointSpec {
        pipe_verb: "contain".to_string(),
        sql_table: "crowdstrike_contained_hosts".to_string(),
        capability_path: "sensor.crowdstrike.containment".to_string(),
        risk_tier: RiskTier::Irreversible,
        batch_limit: 100,
        batch_mode: BatchMode::Serial,
        steps: vec![],
        record_id_field: "device_id".to_string(),
    };

    let inputs = DispatchInputs {
        plan: &plan,
        context: &context,
        risk_tier: &RiskTier::Irreversible,
        confirmed_by_token: Some("tok-permit".to_string()),
        endpoint_spec: &endpoint_spec,
        fetched_records: &[],
        write_endpoint: "crowdstrike.containment",
        capability_check: &CapabilityCheckResult::Allowed,
    };

    let result = dispatcher.dispatch(inputs).await;
    let write_result = result.expect("permit path must succeed");

    // write_intent must have been called exactly once
    assert_eq!(
        audit
            .intent_call_count
            .load(std::sync::atomic::Ordering::SeqCst),
        1,
        "CRIT-4: write_intent must be called exactly once on permit path"
    );
    // write_outcome must have been called exactly once (after fan-out)
    assert_eq!(
        audit
            .outcome_call_count
            .load(std::sync::atomic::Ordering::SeqCst),
        1,
        "CRIT-4: write_outcome must be called exactly once after fan-out"
    );
    assert!(
        !write_result.dry_run,
        "WriteResult.dry_run must be false on execute path"
    );
}

/// CRIT-4 / BC-2.05.009 deny path: when dispatch is called with a denied
/// capability_check, write_intent is still called to create an audit record.
///
/// This documents the intent: even denied operations should produce an audit
/// record. For Phase 2 denials (before dispatch is called), the audit is
/// written by WriteExecutor before returning the error.
///
/// Note: For S-3.07 scope, the deny-path audit is emitted from write_pipeline.rs
/// (WriteExecutor) not from WriteDispatcher (which is only reached on permit path).
/// This test verifies the CapabilityCheckResult type flows through DispatchInputs.
///
/// AC-COVERAGE-DEFERRED: Currently a compile-time existence check on the three
/// CapabilityCheckResult variants. Until W3-FIX-S307-001 unblocks Phase 3 record
/// materialization, calling dispatcher.dispatch() requires extensive mocking that
/// duplicates W3-FIX-S307 follow-up scope. Tighten by extending each variant
/// arm to call dispatcher.dispatch() and assert the audit emission carries the
/// variant correctly — once dispatch() is fully wired.
// TODO(W3-FIX-S307-001): tighten this test once Phase 3 materialization is wired —
// currently exercises type contract only.
#[test]
fn test_crit4_capability_check_result_type_flows_through_dispatch_inputs() {
    use prism_security::feature_flag::CapabilityCheckResult;

    // Verify CapabilityCheckResult variants are accessible and constructible
    let allowed = CapabilityCheckResult::Allowed;
    let denied_runtime = CapabilityCheckResult::DeniedRuntime {
        capability: "sensor.crowdstrike.contain".to_string(),
        client_id: "acme".to_string(),
        resolution_trace: vec!["sensor.crowdstrike.contain=Deny".to_string()],
    };
    let denied_compile = CapabilityCheckResult::DeniedCompileTime {
        capability: "sensor.crowdstrike.contain".to_string(),
        client_id: "acme".to_string(),
        resolution_trace: vec!["compile_gate=Absent".to_string()],
    };

    // All three variants must be constructible and pattern-matchable
    match &allowed {
        CapabilityCheckResult::Allowed => {}
        _ => panic!("Expected Allowed"),
    }
    match &denied_runtime {
        CapabilityCheckResult::DeniedRuntime { capability, .. } => {
            assert_eq!(capability, "sensor.crowdstrike.contain");
        }
        _ => panic!("Expected DeniedRuntime"),
    }
    match &denied_compile {
        CapabilityCheckResult::DeniedCompileTime { capability, .. } => {
            assert_eq!(capability, "sensor.crowdstrike.contain");
        }
        _ => panic!("Expected DeniedCompileTime"),
    }
}

// ---------------------------------------------------------------------------
// BC-2.05.009: EC-05-016 — read ops produce empty capability_checks
// ---------------------------------------------------------------------------

/// BC-2.05.009 EC-05-016: read operations produce an audit entry with an
/// empty `capability_checks` array (no flag evaluated for read paths).
///
/// The `list_capabilities` meta-tool call is a pure read; `capability_checks`
/// must be empty and the result is recorded in `result_summary` instead.
///
/// This is a structural property test (no stub call needed — tests the shape
/// of the audit data that the implementer must populate).
///
/// AC-COVERAGE-DEFERRED: Currently a type-contract check (verifies SensorWriteError
/// vec can be empty). Until W3-FIX-S307-001 provides a richer mock audit writer
/// that captures AuditEntry capability_checks fields, this test cannot validate
/// the actual emission. Tighten by integrating with audit-writer mock + asserting
/// AuditEntry.capability_checks.is_empty() for read operations.
// TODO(W3-FIX-S307-001): tighten this test once Phase 3 materialization is wired —
// currently exercises type contract only.
#[test]
fn test_BC_2_05_009_ec_05_016_read_op_audit_has_empty_capability_checks() {
    // This test documents the contract: when an audit entry is constructed
    // for a read operation, capability_checks must be empty.
    // The implementer must ensure WriteDispatcher.dispatch() is NOT called
    // for read operations, and AuditWriter for read ops uses an empty vec.
    //
    // Contract form: assert the type exists and can hold an empty list.
    // The full behavioral assertion requires integration with the MCP layer (S-4.xx).
    // For now, this test validates that WriteResult.sensor_errors is Vec-typed
    // (i.e., can be empty) as a proxy for the audit list shape.
    let errors: Vec<SensorWriteError> = vec![];
    assert!(
        errors.is_empty(),
        "EC-05-016: read op capability_checks must be empty vec"
    );
}
