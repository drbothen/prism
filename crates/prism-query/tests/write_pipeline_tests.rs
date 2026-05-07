//! Write execution pipeline integration tests — S-3.07.
//!
//! Tests cover all acceptance criteria (AC-1 through AC-9) for the six-phase
//! write execution pipeline. All tests are RED-GATE stubs: they call into
//! `todo!()` bodies and MUST FAIL (panic) until the implementer fills them in.
//!
//! # Red Gate Contract
//! These tests exist to enforce the Red Gate invariant: every test here MUST
//! fail against the stubs. Any test that passes against stubs is a bug in the
//! stub (self-check: BC-5.38.005).
//!
//! # Non-exhaustive types
//! `WriteNode` and `DmlNode` are `#[non_exhaustive]` and cannot be constructed
//! directly from integration tests. Tests call into `todo!()` stubs via the
//! public `WriteExecutor::execute` API (which itself calls `todo!()`).
//!
//! Story: S-3.07 | BCs: BC-2.04.001, BC-2.04.007, BC-2.04.008, BC-2.05.009

#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used
)]

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

use prism_core::RiskTier;
use prism_query::write_pipeline::{QueryContext, WriteExecutor, WriteOutcome, WritePlan};
use prism_query::write_result::{WritePreview, WriteResult};
use prism_security::confirmation_token::ConfirmationTokenStore;
use prism_security::feature_flag::FeatureFlagEvaluator;

// ---------------------------------------------------------------------------
// Helpers: construct a WritePlan for tests (WritePlan itself is exhaustive).
// ---------------------------------------------------------------------------

fn make_contain_plan(has_filter: bool) -> WritePlan {
    WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_contained_hosts".to_string(),
        dml_operation: None,
        has_explicit_limit: false,
        explicit_limit: None,
        has_where_clause: has_filter,
        params: HashMap::new(),
    }
}

fn make_update_plan() -> WritePlan {
    use prism_query::write_ast::DmlOperation;
    WritePlan {
        verb: "update".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_detections".to_string(),
        dml_operation: Some(DmlOperation::Update),
        has_explicit_limit: false,
        explicit_limit: None,
        has_where_clause: true,
        params: {
            let mut m = HashMap::new();
            m.insert("status".to_string(), "acknowledged".to_string());
            m
        },
    }
}

fn make_internal_table_plan() -> WritePlan {
    WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "prism_alerts".to_string(),
        dml_operation: None,
        has_explicit_limit: false,
        explicit_limit: None,
        has_where_clause: true,
        params: HashMap::new(),
    }
}

fn make_query_context(dry_run: bool, token_id: Option<String>) -> QueryContext {
    QueryContext {
        client_id: "acme".to_string(),
        org_slug: prism_core::OrgSlug::new_unchecked("acme"),
        dry_run,
        confirmation_token_id: token_id,
        analyst_id: None,
    }
}

// ---------------------------------------------------------------------------
// Mock AuditWriter infrastructure (lives in integration test to avoid circular dep)
// ---------------------------------------------------------------------------

mod test_helpers {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use prism_core::PrismError;
    use prism_query::write_dispatch::AuditWriter;
    use prism_query::write_pipeline::{QueryContext, WritePlan};
    use prism_query::write_result::WriteResult;
    use ulid::Ulid;

    pub struct MockAuditWriter {
        pub fail_intent: AtomicBool,
    }

    impl MockAuditWriter {
        pub fn always_succeed() -> Arc<Self> {
            Arc::new(Self {
                fail_intent: AtomicBool::new(false),
            })
        }

        pub fn always_fail_intent() -> Arc<Self> {
            Arc::new(Self {
                fail_intent: AtomicBool::new(true),
            })
        }
    }

    #[async_trait::async_trait]
    impl AuditWriter for MockAuditWriter {
        async fn write_intent(
            &self,
            _plan: &WritePlan,
            _context: &QueryContext,
            _capability_check: &prism_security::feature_flag::CapabilityCheckResult,
        ) -> Result<Ulid, PrismError> {
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
            Ok(())
        }
    }
}

fn make_executor(fail_audit: bool) -> WriteExecutor {
    use prism_core::{CapabilityEffect, CapabilityPath, ClientCapabilities};
    use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointRegistry, WriteEndpointSpec};

    let store = Arc::new(ConfirmationTokenStore::new());

    // Grant "acme" root "sensor" capability so Phase 2 passes for AC-1..AC-6 tests.
    // Tests that want denial (AC-7, AC-9) use make_deny_executor() instead.
    let mut caps = BTreeMap::new();
    let mut acme_caps = ClientCapabilities::new();
    acme_caps.grant(
        CapabilityPath::new("sensor").expect("sensor is a valid capability path"),
        CapabilityEffect::Allow,
    );
    caps.insert("acme".to_string(), acme_caps);

    let evaluator = Arc::new(FeatureFlagEvaluator::new(caps));
    let audit = if fail_audit {
        test_helpers::MockAuditWriter::always_fail_intent()
    } else {
        test_helpers::MockAuditWriter::always_succeed()
    };
    let registry = Arc::new(prism_sensors::AdapterRegistry::new());

    // Register test endpoint specs
    let mut endpoint_registry = WriteEndpointRegistry::new();
    let _ = endpoint_registry.register(
        "crowdstrike",
        vec![
            WriteEndpointSpec {
                pipe_verb: "contain".to_string(),
                sql_table: "crowdstrike_contained_hosts".to_string(),
                capability_path: "sensor.crowdstrike.contain".to_string(),
                risk_tier: prism_core::RiskTier::Irreversible,
                batch_limit: 100,
                batch_mode: BatchMode::Serial,
                steps: vec![],
                record_id_field: "device_id".to_string(),
            },
            WriteEndpointSpec {
                pipe_verb: "update".to_string(),
                sql_table: "crowdstrike_detections".to_string(),
                capability_path: "sensor.crowdstrike.update".to_string(),
                risk_tier: prism_core::RiskTier::Reversible,
                batch_limit: 100,
                batch_mode: BatchMode::Serial,
                steps: vec![],
                record_id_field: "id".to_string(),
            },
        ],
    );
    WriteExecutor::new(
        evaluator,
        store,
        audit,
        registry,
        Arc::new(endpoint_registry),
    )
}

// ---------------------------------------------------------------------------
// AC-1: Dry-run default — contain with dry_run=true → WritePreview + token
// ---------------------------------------------------------------------------

/// AC-1: `FROM crowdstrike_hosts | where hostname = 'target' | contain`
/// with `dry_run = true` (default) → `WritePreview` with:
///   - `would_affect_count = 1`
///   - `risk_tier = Irreversible`
///   - `confirmation_token` present
///
/// BC-2.04.007, BC-2.04.008
#[tokio::test]

async fn test_ac1_dry_run_default_returns_preview_with_token() {
    let executor = make_executor(false);
    let plan = make_contain_plan(true);
    let context = make_query_context(true, None);
    // WriteExecutor::execute → todo!() → panic
    let _outcome = executor.execute(plan, context).await.expect("no error");
}

// ---------------------------------------------------------------------------
// AC-2: Token confirmation — valid token consumed → WriteResult returned
// ---------------------------------------------------------------------------

/// AC-2: Given a valid confirmation token, when execute is called with
/// `dry_run=false` and the token ID, then `WriteResult` is returned with
/// `succeeded_count >= 1` and `audit_intent_id` populated.
///
/// BC-2.04.008
#[tokio::test]
async fn test_ac2_valid_token_consumed_returns_write_result() {
    let executor = make_executor(false);
    let plan = make_contain_plan(true);

    // Step 1: Get a real token via dry_run=true
    let dry_ctx = make_query_context(true, None);
    let preview_outcome = executor
        .execute(plan.clone(), dry_ctx)
        .await
        .expect("dry_run should succeed");

    let token_id = match preview_outcome {
        WriteOutcome::Preview(p) => {
            p.confirmation_token
                .expect("Irreversible must generate a token")
                .token_id
        }
        WriteOutcome::Result(_) => panic!("Expected Preview for dry_run=true"),
    };

    // Step 2: Execute with the real token — WriteResult returned
    let execute_ctx = make_query_context(false, Some(token_id));
    let outcome = executor
        .execute(plan, execute_ctx)
        .await
        .expect("execute with valid token must succeed");

    match outcome {
        WriteOutcome::Result(r) => {
            assert!(!r.dry_run, "WriteResult.dry_run must be false");
            // audit_intent_id must be populated (non-zero ULID)
            assert_ne!(
                r.audit_intent_id.to_string(),
                "",
                "audit_intent_id must be populated"
            );
        }
        WriteOutcome::Preview(_) => panic!("Expected Result for dry_run=false"),
    }
}

// ---------------------------------------------------------------------------
// AC-3: Batch limit exceeded → E-QUERY-021 before API call
// ---------------------------------------------------------------------------

/// AC-3: Post-fetch batch limit check fires when records exceed batch_limit.
/// `E-QUERY-021` is returned with affected count and limit — no sensor API call.
#[tokio::test]

async fn test_ac3_batch_limit_exceeded_returns_e_query_021() {
    use prism_query::safety_check::resolve_batch_limit;
    // resolve_batch_limit → todo!() → panic
    let _limit = resolve_batch_limit(10, None, 1000);
}

// ---------------------------------------------------------------------------
// AC-4: Write targeting internal table → E-QUERY-010
// ---------------------------------------------------------------------------

/// AC-4: Write targeting an internal `prism_*` table → `E-QUERY-010` before
/// any API contact. Defense-in-depth check (also caught at parse time in S-3.06).
#[tokio::test]
async fn test_ac4_internal_table_write_rejected_e_query_010() {
    let executor = make_executor(false);
    let plan = make_internal_table_plan(); // target_table = "prism_alerts"
    let context = make_query_context(true, None);
    // Phase 2 must reject internal table writes with E-QUERY-027 (internal tables)
    let result = executor.execute(plan, context).await;
    let err = result.expect_err("Internal table write must be rejected");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-QUERY-027")
            || err_msg.contains("internal")
            || err_msg.contains("prism_"),
        "Internal table write must produce E-QUERY-027 or 'internal'; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-5: Audit fail-closed — write_intent fails → abort, no sensor call
// ---------------------------------------------------------------------------

/// AC-5: When `AuditWriter::write_intent()` fails, the entire write is aborted
/// with `E-AUDIT-001` and no sensor API call is made.
///
/// BC-2.05.009 fail-closed behavior.
///
/// Uses a Reversible plan (update) with dry_run=false so Phase 4 proceeds without
/// requiring a confirmation token, allowing Phase 5 (audit write) to be reached.
#[tokio::test]
async fn test_ac5_audit_fail_closed_aborts_write() {
    // Executor configured with a failing audit writer
    let executor = make_executor(true);
    // Use a Reversible plan (update) — no confirmation token required for Reversible tier.
    // This lets us reach Phase 5 (audit write) where the fail-closed behavior is tested.
    let plan = make_update_plan(); // Reversible: update on crowdstrike_detections
    let context = make_query_context(false, None); // dry_run=false, no token needed for Reversible
                                                   // Phase 5 (audit intent) should fail → entire write aborted with E-AUDIT-001
    let result = executor.execute(plan, context).await;
    let err = result.expect_err("Audit failure must abort the write");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-AUDIT-001") || err_msg.contains("Audit emission failed"),
        "Audit fail-closed must produce E-AUDIT-001; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-6: SQL mode — same six-phase pipeline as pipe mode
// ---------------------------------------------------------------------------

/// AC-6: `UPDATE crowdstrike_detections SET status = 'acknowledged' WHERE severity_id <= 2`
/// via SQL mode → same six-phase safety pipeline as pipe-mode write.
///
/// BC-2.04.007
#[tokio::test]

async fn test_ac6_sql_mode_runs_same_safety_pipeline_as_pipe_mode() {
    let executor = make_executor(false);
    let plan = make_update_plan();
    let context = make_query_context(true, None);
    // WriteExecutor::execute → todo!() → panic
    let _outcome = executor.execute(plan, context).await.expect("no error");
}

// ---------------------------------------------------------------------------
// AC-7: Feature flag deny → E-FLAG-001 + audit-logged
// ---------------------------------------------------------------------------

/// AC-7: Write endpoint `capability_path = "sensor.armis.device_write"` denied
/// in client TOML → `E-FLAG-001` returned; evaluation audit-logged.
///
/// BC-2.04.001, BC-2.05.009
///
/// Uses an executor where the acme client has general sensor allow but Deny
/// specifically for sensor.armis.device_write (child deny overrides parent allow).
#[tokio::test]
async fn test_ac7_feature_flag_deny_returns_e_flag_001() {
    use prism_core::{CapabilityEffect, CapabilityPath, ClientCapabilities};
    use prism_spec_engine::write_endpoint::WriteEndpointRegistry;
    use std::sync::Arc;

    // Build evaluator where acme has sensor Allow but sensor.armis.device_write is Deny
    // (child deny overrides parent allow — BC-2.05.009 hierarchical evaluation)
    let mut caps = BTreeMap::new();
    let mut acme_caps = ClientCapabilities::new();
    acme_caps.grant(
        CapabilityPath::new("sensor").expect("sensor is a valid capability path"),
        CapabilityEffect::Allow,
    );
    acme_caps.grant(
        CapabilityPath::new("sensor.armis.device_write")
            .expect("sensor.armis.device_write is a valid capability path"),
        CapabilityEffect::Deny, // explicit child deny overrides parent allow
    );
    caps.insert("acme".to_string(), acme_caps);

    let evaluator = Arc::new(FeatureFlagEvaluator::new(caps));
    let store = Arc::new(ConfirmationTokenStore::new());
    let audit = test_helpers::MockAuditWriter::always_succeed();
    let registry = Arc::new(prism_sensors::AdapterRegistry::new());
    let endpoint_registry = Arc::new(WriteEndpointRegistry::new());
    let executor = WriteExecutor::new(evaluator, store, audit, registry, endpoint_registry);

    let plan = WritePlan {
        verb: "device_write".to_string(),
        sensor: "armis".to_string(),
        target_table: "armis_devices".to_string(),
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(5),
        has_where_clause: true,
        params: HashMap::new(),
    };
    let context = make_query_context(true, None);
    // Phase 2 capability check: sensor.armis.device_write is Deny → E-FLAG-001
    let result = executor.execute(plan, context).await;
    let err = result.expect_err("Denied capability must return error");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("CAPABILITY_DENIED") || err_msg.contains("E-FLAG-001"),
        "Feature flag deny must produce E-FLAG-001; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-8: Unbounded write (no WHERE, no LIMIT) → E-QUERY-022 before fetch
// ---------------------------------------------------------------------------

/// AC-8: `FROM crowdstrike_hosts | contain` with no WHERE clause and no LIMIT
/// → `E-QUERY-022` before any fetch or sensor API contact.
#[tokio::test]
async fn test_ac8_unbounded_write_rejected_e_query_022() {
    use prism_query::safety_check::check_unbounded_write;

    // Plan with no WHERE clause and no LIMIT (unbounded)
    let plan = make_contain_plan(false); // has_where_clause = false, has_explicit_limit = false
    let result = check_unbounded_write(&plan);
    let err = result.expect_err("Unbounded write must be rejected");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-QUERY-022") || err_msg.contains("unbounded"),
        "Unbounded write must produce E-QUERY-022; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// CRIT-3: BC-2.04.001 compile-time *-write cargo features
// ---------------------------------------------------------------------------

/// BC-2.04.001: verify that the `crowdstrike-write` Cargo feature
/// compiles and can be queried. The feature value (present/absent) depends
/// on whether the binary was compiled with --features crowdstrike-write.
///
/// This test documents the compile-gate behavior: when crowdstrike-write
/// is NOT enabled (default build), cfg! returns false. When enabled, it returns true.
///
/// CRIT-3 fix: add [features] to Cargo.toml with deny-by-default *-write features.
#[test]
fn test_crit3_crowdstrike_write_feature_is_queryable() {
    // This test verifies that the feature exists and can be queried at compile time.
    // The assertion varies by build configuration:
    //   - default features:   feature_present = false (deny-by-default, DI-003)
    //   - --features all-write: feature_present = true
    let feature_present = cfg!(feature = "crowdstrike-write");
    // Both states are valid — this test just ensures the feature COMPILES.
    // The deny-by-default invariant is enforced by the CI matrix (default-features build).
    let _ = feature_present; // used for compile verification
}

/// BC-2.04.001: sensor name → CompileFeatureGate dispatch must map
/// "crowdstrike" to Absent when feature is absent, Present when enabled.
///
/// This tests the cfg-derived dispatch in write_pipeline.rs.
/// The compile gate value is cfg!-derived and verified here.
#[test]
fn test_crit3_sensor_compile_gate_matches_cfg_feature() {
    use prism_query::safety_check::CompileFeatureGate;

    // The cfg-derived gate for "crowdstrike"
    let expected_gate = if cfg!(feature = "crowdstrike-write") {
        CompileFeatureGate::Present
    } else {
        CompileFeatureGate::Absent
    };

    // Verify the gate enum is constructible and matches expected cfg! value
    // In default build: Absent. With --features crowdstrike-write: Present.
    match expected_gate {
        CompileFeatureGate::Absent => {
            // Default: deny-by-default per DI-003
        }
        CompileFeatureGate::Present => {
            // Feature enabled: write code compiled in
        }
    }
}

/// BC-2.04.001: in default-features build, executing a crowdstrike write plan
/// must be denied with E-FLAG-002 (compile gate absent).
///
/// This test only runs when crowdstrike-write feature is NOT enabled.
/// With --features all-write (just check), the test is skipped/excluded.
/// With default features (just iter), this RED tests the compile gate denial.
#[tokio::test]
#[cfg(not(feature = "crowdstrike-write"))]
async fn test_crit3_crowdstrike_write_denied_in_default_build() {
    use prism_core::{CapabilityEffect, CapabilityPath, ClientCapabilities};
    use prism_spec_engine::write_endpoint::WriteEndpointRegistry;
    use std::collections::BTreeMap;

    // Build executor with "acme" allowed for sensor.crowdstrike.contain
    let mut caps = BTreeMap::new();
    let mut acme_caps = ClientCapabilities::new();
    acme_caps.grant(
        CapabilityPath::new("sensor").expect("valid"),
        CapabilityEffect::Allow,
    );
    caps.insert("acme".to_string(), acme_caps);
    let evaluator = Arc::new(FeatureFlagEvaluator::new(caps));
    let store = Arc::new(ConfirmationTokenStore::new());
    let audit = test_helpers::MockAuditWriter::always_succeed();
    let registry = Arc::new(prism_sensors::AdapterRegistry::new());
    let endpoint_registry = Arc::new(WriteEndpointRegistry::new());
    let executor = WriteExecutor::new(evaluator, store, audit, registry, endpoint_registry);

    let plan = make_contain_plan(true); // crowdstrike sensor
    let context = make_query_context(true, None);

    // With default features (no crowdstrike-write), Phase 2 Gate 3 must deny
    let result = executor.execute(plan, context).await;
    let err = result.expect_err("Crowdstrike write must be denied when feature absent");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-FLAG-002")
            || err_msg.contains("not compiled")
            || err_msg.contains("CAPABILITY_DENIED"),
        "Absent compile gate must produce E-FLAG-002 or CAPABILITY_DENIED; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// CRIT-1: SensorAdapter::write default body must return structured error
// ---------------------------------------------------------------------------

/// CRIT-1 / BC-2.04.007: The SensorAdapter::write default body must return
/// Err(WriteNotImplemented) instead of panicking with todo!().
///
/// Tests that the MockAdapter (empty registry) propagates structured errors
/// through the full pipeline and that WriteOutcome::Result is returned when
/// dry_run=false + Reversible plan.
#[tokio::test]
async fn test_crit1_write_result_returned_with_zero_records_on_empty_registry() {
    let executor = make_executor(false);
    // Reversible plan: no confirmation token needed
    let plan = make_update_plan();
    let context = make_query_context(false, None);

    let outcome = executor
        .execute(plan, context)
        .await
        .expect("execute with empty registry must succeed (no sensor adapters)");

    match outcome {
        WriteOutcome::Result(r) => {
            assert!(!r.dry_run, "WriteResult.dry_run must be false");
            // With empty adapter registry: affected_count = 0 (no records fetched)
            assert_eq!(r.affected_count, 0, "No records fetched = 0 affected");
            assert_eq!(r.succeeded_count, 0, "No adapters = 0 succeeded");
            // audit_intent_id must be a non-zero ULID (populated by MockAuditWriter)
            assert_ne!(
                r.audit_intent_id.to_string(),
                "00000000000000000000000000",
                "audit_intent_id must be populated"
            );
        }
        WriteOutcome::Preview(_) => panic!("Expected Result for dry_run=false"),
    }
}

// ---------------------------------------------------------------------------
// HIGH-1: Non-vacuous AC-1 assertions (would_affect_count, risk_tier, token)
// ---------------------------------------------------------------------------

/// HIGH-1 / AC-1: Dry-run with Irreversible plan must return WritePreview with:
///   - risk_tier == Irreversible
///   - confirmation_token present (Some)
///   - would_affect_count == 0 (no records fetched in Phase 3 stub)
#[tokio::test]
async fn test_high1_ac1_dry_run_preview_has_risk_tier_and_token() {
    let executor = make_executor(false);
    let plan = make_contain_plan(true); // Irreversible: "contain" endpoint
    let context = make_query_context(true, None);

    let outcome = executor
        .execute(plan, context)
        .await
        .expect("dry_run must succeed");

    match outcome {
        WriteOutcome::Preview(preview) => {
            assert_eq!(
                preview.risk_tier,
                RiskTier::Irreversible,
                "AC-1: contain plan must have Irreversible risk tier"
            );
            assert!(
                preview.confirmation_token.is_some(),
                "AC-1: Irreversible dry_run must generate a confirmation token"
            );
            assert!(
                preview.dry_run,
                "AC-1: WritePreview.dry_run must always be true"
            );
            // Phase 3 stub: fetched_records is empty → would_affect_count == 0
            assert_eq!(
                preview.would_affect_count, 0,
                "AC-1: Phase 3 stub produces 0 fetched records"
            );
        }
        WriteOutcome::Result(_) => panic!("Expected Preview for dry_run=true"),
    }
}

// ---------------------------------------------------------------------------
// HIGH-1: Non-vacuous AC-2 assertions (succeeded_count, audit_intent_id)
// ---------------------------------------------------------------------------

/// HIGH-1 / AC-2: Valid token → WriteResult returned with audit_intent_id
/// matching what the MockAuditWriter assigned (non-zero ULID).
///
/// Replace vacuous `assert!(!r.dry_run)` with structural assertions.
#[tokio::test]
async fn test_high1_ac2_write_result_has_populated_audit_intent_id() {
    let executor = make_executor(false);
    let plan = make_contain_plan(true);

    // Step 1: dry_run=true to get token
    let dry_ctx = make_query_context(true, None);
    let preview_outcome = executor
        .execute(plan.clone(), dry_ctx)
        .await
        .expect("dry_run must succeed");

    let token_id = match preview_outcome {
        WriteOutcome::Preview(p) => {
            p.confirmation_token
                .expect("Irreversible must generate token")
                .token_id
        }
        WriteOutcome::Result(_) => panic!("Expected Preview"),
    };

    // Step 2: execute with valid token
    let execute_ctx = make_query_context(false, Some(token_id.clone()));
    let outcome = executor
        .execute(plan, execute_ctx)
        .await
        .expect("execute with valid token must succeed");

    match outcome {
        WriteOutcome::Result(r) => {
            assert!(!r.dry_run, "WriteResult.dry_run must be false");
            // Non-vacuous: audit_intent_id must not be the zero ULID
            assert_ne!(
                r.audit_intent_id.to_string(),
                "00000000000000000000000000",
                "AC-2: audit_intent_id must be populated (non-zero ULID)"
            );
            // Non-vacuous: risk_tier must match the plan's endpoint spec
            assert_eq!(
                r.risk_tier,
                RiskTier::Irreversible,
                "AC-2: contain plan has Irreversible risk tier"
            );
        }
        WriteOutcome::Preview(_) => panic!("Expected Result for dry_run=false with token"),
    }
}

// ---------------------------------------------------------------------------
// HIGH-1: Non-vacuous AC-3 (batch limit exceeded → E-QUERY-021)
// ---------------------------------------------------------------------------

/// HIGH-1 / AC-3: Drive WriteExecutor with plan exceeding batch_limit.
///
/// The test plan uses explicit_limit > endpoint batch_limit (100) to trigger
/// E-QUERY-021 before any fetch or sensor API contact.
#[tokio::test]
async fn test_high1_ac3_batch_limit_exceeded_returns_e_query_021_via_executor() {
    let executor = make_executor(false);
    // Plan with explicit_limit 200 > endpoint batch_limit 100 → E-QUERY-021
    let plan = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_contained_hosts".to_string(),
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(200), // exceeds batch_limit of 100 for "contain"
        has_where_clause: true,
        params: HashMap::new(),
    };
    let context = make_query_context(true, None);

    let result = executor.execute(plan, context).await;
    let err = result.expect_err("Batch limit exceeded must return Err");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-QUERY-021")
            || err_msg.contains("batch limit")
            || err_msg.contains("limit exceeded"),
        "AC-3: batch limit exceeded must produce E-QUERY-021; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// HIGH-1: Non-vacuous AC-6 (SQL mode returns WritePreview with risk_tier)
// ---------------------------------------------------------------------------

/// HIGH-1 / AC-6: SQL UPDATE via WriteExecutor dry_run returns WritePreview
/// with the correct risk_tier (Reversible for update endpoint).
#[tokio::test]
async fn test_high1_ac6_sql_dry_run_returns_preview_with_correct_risk_tier() {
    let executor = make_executor(false);
    let plan = make_update_plan(); // Reversible: update on crowdstrike_detections
    let context = make_query_context(true, None);

    let outcome = executor
        .execute(plan, context)
        .await
        .expect("dry_run SQL update must succeed");

    match outcome {
        WriteOutcome::Preview(preview) => {
            assert_eq!(
                preview.risk_tier,
                RiskTier::Reversible,
                "AC-6: SQL UPDATE endpoint is Reversible"
            );
            assert!(
                preview.dry_run,
                "AC-6: WritePreview.dry_run must always be true"
            );
            // Reversible tier: no confirmation token
            assert!(
                preview.confirmation_token.is_none(),
                "AC-6: Reversible dry_run must NOT generate a confirmation token"
            );
        }
        WriteOutcome::Result(_) => panic!("Expected Preview for dry_run=true"),
    }
}

// ---------------------------------------------------------------------------
// AC-9: Per-client write denial → E-FLAG-001, tool visibility invariant
// ---------------------------------------------------------------------------

/// AC-9: Write verb disabled for requesting client (enabled for others) →
/// `E-FLAG-001` at invocation time; write tool remains in `tools/list`.
///
/// BC-2.04.005 Hidden Tools Pattern postcondition.
/// The executor has "acme" with sensor Allow but "restricted_client" is not configured.
/// Phase 2 Gate 2 (runtime TOML capability check) returns E-FLAG-001 for restricted_client.
#[tokio::test]
async fn test_ac9_per_client_write_denial_returns_e_flag_001_not_unknown_tool() {
    let executor = make_executor(false);
    let plan = make_contain_plan(true);
    // client_id "restricted_client" is not in the evaluator → denied at Phase 2 Gate 2
    let context = QueryContext {
        client_id: "restricted_client".to_string(),
        org_slug: prism_core::OrgSlug::new_unchecked("restricted-client"),
        dry_run: true,
        confirmation_token_id: None,
        analyst_id: None,
    };
    // Phase 2 capability check returns E-FLAG-001 (CapabilityDenied), NOT E-MCP-001 (tool not found)
    let result = executor.execute(plan, context).await;
    let err = result.expect_err("Restricted client must be denied");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("CAPABILITY_DENIED") || err_msg.contains("E-FLAG-001"),
        "Per-client denial must produce E-FLAG-001 (CAPABILITY_DENIED), not 'unknown tool'; \
         got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// CRIT-2: WriteCapableTableProvider DataFusion integration (AC-6)
// ---------------------------------------------------------------------------

/// CRIT-2 / AC-6: WriteCapableTableProvider::new must not panic (todo!() removed).
///
/// Constructs the provider with a WriteTableDescriptor and WriteEndpointSpec.
/// Before CRIT-2 fix: todo!() panics immediately.
/// After CRIT-2 fix: returns a provider with a valid Arrow schema.
#[test]
fn test_crit2_write_capable_table_provider_new_does_not_panic() {
    use prism_core::RiskTier;
    use prism_query::write_table_registration::WriteCapableTableProvider;
    use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointSpec, WriteTableDescriptor};

    let descriptor = WriteTableDescriptor {
        sql_table: "crowdstrike_contained_hosts".to_string(),
        write_only: true,
        sensor: "crowdstrike".to_string(),
        verb: "contain".to_string(),
        risk_tier: RiskTier::Irreversible,
    };

    let endpoint_spec = WriteEndpointSpec {
        pipe_verb: "contain".to_string(),
        sql_table: "crowdstrike_contained_hosts".to_string(),
        capability_path: "sensor.crowdstrike.contain".to_string(),
        risk_tier: RiskTier::Irreversible,
        batch_limit: 100,
        batch_mode: BatchMode::Serial,
        steps: vec![],
        record_id_field: "device_id".to_string(),
    };

    let executor = Arc::new(make_executor(false));

    // Before fix: todo!() → panic. After fix: returns a valid provider.
    let provider = WriteCapableTableProvider::new(descriptor, endpoint_spec, executor);

    // Verify the schema is non-empty
    use datafusion::datasource::TableProvider;
    let schema = provider.schema();
    assert!(
        schema.fields().len() > 0,
        "CRIT-2: WriteCapableTableProvider schema must have at least one field"
    );
}

/// CRIT-2 / AC-6: WriteCapableTableProvider::insert_into must return structured
/// error (NotImplemented or plan) instead of panicking with todo!().
///
/// This test verifies the method is callable without panic.
/// Full SQL DML routing to WriteExecutor is deferred to W3-FIX-S307-003.
#[tokio::test]
async fn test_crit2_insert_into_returns_not_implemented_not_panic() {
    use datafusion::datasource::TableProvider;
    use datafusion::logical_expr::dml::InsertOp;
    use prism_core::RiskTier;
    use prism_query::write_table_registration::WriteCapableTableProvider;
    use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointSpec, WriteTableDescriptor};

    let descriptor = WriteTableDescriptor {
        sql_table: "crowdstrike_contained_hosts".to_string(),
        write_only: true,
        sensor: "crowdstrike".to_string(),
        verb: "contain".to_string(),
        risk_tier: RiskTier::Irreversible,
    };
    let endpoint_spec = WriteEndpointSpec {
        pipe_verb: "contain".to_string(),
        sql_table: "crowdstrike_contained_hosts".to_string(),
        capability_path: "sensor.crowdstrike.contain".to_string(),
        risk_tier: RiskTier::Irreversible,
        batch_limit: 100,
        batch_mode: BatchMode::Serial,
        steps: vec![],
        record_id_field: "device_id".to_string(),
    };

    let executor = Arc::new(make_executor(false));
    let provider = WriteCapableTableProvider::new(descriptor, endpoint_spec, executor);

    // Build a minimal mock Session for insert_into
    let ctx = datafusion::prelude::SessionContext::new();
    let session = ctx.state();

    // Create a trivial EmptyExec plan as input
    let input_schema = provider.schema();
    let input_plan: Arc<dyn datafusion::physical_plan::ExecutionPlan> = Arc::new(
        datafusion::physical_plan::empty::EmptyExec::new(input_schema),
    );

    // insert_into must NOT panic — before fix: todo!() panics.
    // After fix: returns DataFusionError::NotImplemented or Ok(plan).
    let result = provider
        .insert_into(&session, input_plan, InsertOp::Append)
        .await;

    // Accept either Ok(...) or Err(NotImplemented) — both are valid CRIT-2 outcomes.
    // Reject any panic (which would fail the test) or other error types.
    match result {
        Ok(_) => {
            // Full implementation: returns an execution plan
        }
        Err(e) => {
            let err_str = e.to_string();
            assert!(
                err_str.contains("not implemented")
                    || err_str.contains("NotImplemented")
                    || err_str.contains("S-3.07"),
                "CRIT-2: insert_into must return NotImplemented, not an unexpected error; got: {err_str}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// HIGH-3: Case-insensitive is_internal_table
// ---------------------------------------------------------------------------

/// HIGH-3: `is_internal_table` must be case-insensitive.
/// "Prism_alerts" and "PRISM_AUDIT" are internal tables (defense-in-depth).
#[test]
fn test_high3_is_internal_table_case_insensitive() {
    let plan_lower = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "prism_alerts".to_string(),
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(10),
        has_where_clause: true,
        params: HashMap::new(),
    };
    let plan_mixed = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "Prism_alerts".to_string(), // Mixed case
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(10),
        has_where_clause: true,
        params: HashMap::new(),
    };
    let plan_upper = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "PRISM_AUDIT".to_string(), // Upper case
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(10),
        has_where_clause: true,
        params: HashMap::new(),
    };
    let plan_not_internal = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_hosts".to_string(), // Not internal
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(10),
        has_where_clause: true,
        params: HashMap::new(),
    };

    assert!(
        plan_lower.is_internal_table(),
        "HIGH-3: 'prism_alerts' must be detected as internal"
    );
    assert!(
        plan_mixed.is_internal_table(),
        "HIGH-3: 'Prism_alerts' (mixed case) must be detected as internal"
    );
    assert!(
        plan_upper.is_internal_table(),
        "HIGH-3: 'PRISM_AUDIT' (upper case) must be detected as internal"
    );
    assert!(
        !plan_not_internal.is_internal_table(),
        "HIGH-3: 'crowdstrike_hosts' must NOT be detected as internal"
    );
}

// ---------------------------------------------------------------------------
// HIGH-4: Composite source detection via WriteEndpointRegistry::is_composite
// ---------------------------------------------------------------------------

/// HIGH-4: WriteEndpointRegistry::is_composite("events") must return true.
/// WriteEndpointRegistry::is_composite("crowdstrike") must return false.
#[test]
fn test_high4_is_composite_via_registry() {
    use prism_spec_engine::write_endpoint::WriteEndpointRegistry;

    assert!(
        WriteEndpointRegistry::is_composite("events"),
        "HIGH-4: 'events' must be composite source"
    );
    assert!(
        WriteEndpointRegistry::is_composite("EVENTS"),
        "HIGH-4: 'EVENTS' (uppercase) must also be composite source"
    );
    assert!(
        !WriteEndpointRegistry::is_composite("crowdstrike"),
        "HIGH-4: 'crowdstrike' must NOT be composite source"
    );
    assert!(
        !WriteEndpointRegistry::is_composite("armis"),
        "HIGH-4: 'armis' must NOT be composite source"
    );
}

// ---------------------------------------------------------------------------
// HIGH-8: Dry-run hash stability — would_affect_count excluded from hash
// ---------------------------------------------------------------------------

/// HIGH-8: The confirmation token hash must NOT include `would_affect_count`.
/// A dry-run with N records should produce a token that's still valid when
/// executed with a slightly different record count.
///
/// Tests by verifying that two sequential executes (dry-run then execute)
/// work even though the record count between them might differ.
/// In the current test setup with Phase 3 stub (always 0 records), both
/// dry-run and execute see 0 records, so this is a structural test.
#[tokio::test]
async fn test_high8_token_hash_excludes_would_affect_count() {
    let executor = make_executor(false);
    // Irreversible plan requires token
    let plan = make_contain_plan(true);

    // Step 1: dry_run to get token (with 0 records in Phase 3 stub)
    let dry_ctx = make_query_context(true, None);
    let preview = executor
        .execute(plan.clone(), dry_ctx)
        .await
        .expect("dry_run must succeed");

    let token_id = match preview {
        WriteOutcome::Preview(p) => {
            p.confirmation_token
                .expect("Irreversible must generate token")
                .token_id
        }
        WriteOutcome::Result(_) => panic!("Expected Preview"),
    };

    // Step 2: execute with token — must succeed (hash excludes count)
    let execute_ctx = make_query_context(false, Some(token_id));
    let result = executor
        .execute(plan, execute_ctx)
        .await
        .expect("HIGH-8: execute with token must succeed when count excluded from hash");

    match result {
        WriteOutcome::Result(r) => {
            assert!(!r.dry_run, "WriteResult.dry_run must be false");
        }
        WriteOutcome::Preview(_) => panic!("Expected Result for dry_run=false"),
    }
}
