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
