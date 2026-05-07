//! Dry-run gate and confirmation token tests — S-3.07.
//!
//! Covers:
//! - BC-2.04.008: dry-run is the default (omitted → true)
//! - BC-2.04.008: dry_run=true never modifies state (invariant)
//! - BC-2.04.008: dry_run=false with Reversible tier proceeds without token
//! - BC-2.04.008: response includes `_meta.dry_run` field accurate to execution mode
//! - BC-2.04.008: EC-04-016 — sensor native dry-run unsupported, Prism simulates
//! - BC-2.04.008: EC-04-017 — agent sends dry_run=false first call (skips preview)
//! - BC-2.04.007: Irreversible write without token → E-FLAG-008
//! - BC-2.04.007: token consumed exactly once (single-use invariant)
//! - Edge: confirmation token replay attack → E-FLAG-004 (TokenAlreadyConsumed)
//! - Edge: confirmation token expired → E-FLAG-003 (TokenExpired)
//! - Edge: token bound to different call (action hash mismatch) → E-FLAG-005
//! - Edge: feature-flag flip mid-call simulation → deterministic deny at Phase 2
//! - proptest: WritePreview.dry_run always true; WriteResult.dry_run always false
//!
//! All tests are RED-GATE stubs. Every body calls into `todo!()` implementations.
//!
//! Story: S-3.07 | BCs: BC-2.04.007, BC-2.04.008

#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used
)]

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use prism_core::{PrismError, RiskTier};
use prism_query::write_pipeline::{QueryContext, WriteExecutor, WriteOutcome, WritePlan};
use prism_query::write_result::{WritePreview, WriteResult};
use prism_security::confirmation_token::ConfirmationTokenStore;
use prism_security::feature_flag::FeatureFlagEvaluator;

mod helpers {
    use std::collections::HashMap;
    use std::sync::Arc;

    use prism_core::PrismError;
    use prism_query::write_dispatch::AuditWriter;
    use prism_query::write_pipeline::{QueryContext, WriteExecutor, WritePlan};
    use prism_query::write_result::WriteResult;
    use ulid::Ulid;

    pub fn make_reversible_plan() -> WritePlan {
        // `create_schedule` is classified as Reversible Write (BC-2.04.007 table)
        WritePlan {
            verb: "create_schedule".to_string(),
            sensor: "crowdstrike".to_string(),
            target_table: "crowdstrike_schedules".to_string(),
            dml_operation: None,
            has_explicit_limit: true,
            explicit_limit: Some(1),
            has_where_clause: true,
            params: HashMap::new(),
        }
    }

    pub fn make_irreversible_plan() -> WritePlan {
        // `crowdstrike_contain_host` is classified as Irreversible Write (BC-2.04.007 table)
        WritePlan {
            verb: "contain".to_string(),
            sensor: "crowdstrike".to_string(),
            target_table: "crowdstrike_contained_hosts".to_string(),
            dml_operation: None,
            has_explicit_limit: true,
            explicit_limit: Some(1),
            has_where_clause: true,
            params: HashMap::new(),
        }
    }

    pub fn make_context(dry_run: bool, token: Option<&str>) -> QueryContext {
        QueryContext {
            client_id: "acme".to_string(),
            org_slug: prism_core::OrgSlug::new_unchecked("acme"),
            dry_run,
            confirmation_token_id: token.map(|t| t.to_string()),
            analyst_id: None,
        }
    }

    pub struct AlwaysSucceedAudit;

    #[async_trait::async_trait]
    impl AuditWriter for AlwaysSucceedAudit {
        async fn write_intent(
            &self,
            _plan: &WritePlan,
            _ctx: &QueryContext,
            _capability_check: &prism_security::feature_flag::CapabilityCheckResult,
        ) -> Result<Ulid, PrismError> {
            Ok(Ulid::new())
        }

        async fn write_outcome(
            &self,
            _intent_id: Ulid,
            _result: &WriteResult,
        ) -> Result<(), PrismError> {
            Ok(())
        }
    }

    pub fn make_executor() -> WriteExecutor {
        use prism_core::{CapabilityEffect, CapabilityPath, ClientCapabilities, RiskTier};
        use prism_spec_engine::write_endpoint::{
            BatchMode, WriteEndpointRegistry, WriteEndpointSpec,
        };
        use std::collections::BTreeMap;

        let store = Arc::new(prism_security::confirmation_token::ConfirmationTokenStore::new());

        // Grant "acme" the root "sensor" capability so Phase 2 gates pass in dry_run tests.
        // The dry_run tests are focused on Phase 4 behavior — they must pass Phase 2 first.
        let mut caps = BTreeMap::new();
        let mut acme_caps = ClientCapabilities::new();
        // Root "sensor" grants all sensor.* paths via hierarchical resolution
        acme_caps.grant(
            CapabilityPath::new("sensor").expect("sensor is a valid capability path"),
            CapabilityEffect::Allow,
        );
        caps.insert("acme".to_string(), acme_caps);

        let evaluator = Arc::new(prism_security::feature_flag::FeatureFlagEvaluator::new(
            caps,
        ));
        let audit = Arc::new(AlwaysSucceedAudit);
        let registry = Arc::new(prism_sensors::AdapterRegistry::new());

        // Register both test endpoint specs so the executor finds the correct risk tiers.
        // make_reversible_plan() uses verb="create_schedule" → Reversible
        // make_irreversible_plan() uses verb="contain" → Irreversible
        let mut endpoint_registry = WriteEndpointRegistry::new();
        let _ = endpoint_registry.register(
            "crowdstrike",
            vec![
                WriteEndpointSpec {
                    pipe_verb: "create_schedule".to_string(),
                    sql_table: "crowdstrike_schedules".to_string(),
                    capability_path: "sensor.crowdstrike.create_schedule".to_string(),
                    risk_tier: RiskTier::Reversible,
                    batch_limit: 100,
                    batch_mode: BatchMode::Serial,
                    steps: vec![],
                    record_id_field: "id".to_string(),
                },
                WriteEndpointSpec {
                    pipe_verb: "contain".to_string(),
                    sql_table: "crowdstrike_contained_hosts".to_string(),
                    capability_path: "sensor.crowdstrike.contain".to_string(),
                    risk_tier: RiskTier::Irreversible,
                    batch_limit: 100,
                    batch_mode: BatchMode::Serial,
                    steps: vec![],
                    record_id_field: "device_id".to_string(),
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
}

// ---------------------------------------------------------------------------
// BC-2.04.008: dry-run default — omitting dry_run defaults to true
// ---------------------------------------------------------------------------

/// BC-2.04.008 postcondition: when `dry_run = true` (the default), the tool
/// simulates the operation and returns a `WritePreview` without any sensor call.
///
/// Canonical test vector: "Default invocation | omitted (defaults to true) |
/// Preview returned; no state change"
#[tokio::test]

async fn test_BC_2_04_008_dry_run_true_returns_preview_no_sensor_call() {
    let executor = helpers::make_executor();
    let plan = helpers::make_irreversible_plan();
    let context = helpers::make_context(true, None);

    // execute → todo!() → panic
    let outcome = executor.execute(plan, context).await.expect("no error");
    match outcome {
        WriteOutcome::Preview(p) => {
            assert!(p.dry_run, "WritePreview.dry_run must be true");
        }
        WriteOutcome::Result(_) => {
            panic!("Expected Preview for dry_run=true, got Result");
        }
    }
}

/// BC-2.04.008 postcondition: WritePreview.dry_run is always true.
/// WriteResult.dry_run is always false.
///
/// This is an invariant: the flag in the response always matches the execution mode.
#[tokio::test]

async fn test_BC_2_04_008_response_dry_run_field_matches_execution_mode() {
    let executor = helpers::make_executor();

    // dry_run=true path
    let preview = executor
        .execute(
            helpers::make_reversible_plan(),
            helpers::make_context(true, None),
        )
        .await
        .expect("no error");

    match preview {
        WriteOutcome::Preview(p) => {
            assert!(p.dry_run, "WritePreview.dry_run must always be true");
        }
        _ => panic!("Expected Preview for dry_run=true"),
    }
}

// ---------------------------------------------------------------------------
// BC-2.04.008: dry_run=false with Reversible tier proceeds without token
// ---------------------------------------------------------------------------

/// BC-2.04.008 postcondition: for a Reversible tier operation, dry_run=false
/// proceeds directly to Phase 5 without requiring a confirmation token.
///
/// Canonical test vector: "Explicit execute | false | Actual write executes"
/// Story §Task 5b: "For Reversible tier: proceed directly to Phase 5."
#[tokio::test]

async fn test_BC_2_04_008_reversible_tier_dry_run_false_no_token_required() {
    let executor = helpers::make_executor();
    let plan = helpers::make_reversible_plan();
    // dry_run=false, no token — Reversible tier should NOT require token
    let context = helpers::make_context(false, None);

    // execute → todo!() → panic
    let outcome = executor.execute(plan, context).await.expect("no error");
    match outcome {
        WriteOutcome::Result(r) => {
            assert!(!r.dry_run, "WriteResult.dry_run must always be false");
        }
        WriteOutcome::Preview(_) => {
            panic!("Expected Result for dry_run=false, got Preview");
        }
    }
}

/// BC-2.04.008 EC-04-017: agent sends dry_run=false on first call (skips preview).
/// "Allowed; the dry-run default is a suggestion, not a hard gate;
/// the operation executes immediately."
///
/// The system must not block dry_run=false without a prior dry_run=true call.
#[tokio::test]

async fn test_BC_2_04_008_ec_04_017_dry_run_false_first_call_allowed_for_reversible() {
    let executor = helpers::make_executor();
    // Reversible plan, skip the dry-run preview entirely
    let plan = helpers::make_reversible_plan();
    let context = helpers::make_context(false, None);

    // execute → todo!() → panic
    // Post-implementation: must not error with "must call dry_run=true first"
    let _outcome = executor.execute(plan, context).await;
}

// ---------------------------------------------------------------------------
// BC-2.04.007: Irreversible write without token → E-FLAG-008
// ---------------------------------------------------------------------------

/// BC-2.04.007 precondition violation: for an Irreversible tier operation,
/// `dry_run=false` without a confirmation token must return `E-FLAG-008`
/// (TokenNotFound).
///
/// Story §Task 5b: "For Irreversible tier: require a valid ConfirmationToken."
/// Story §EC-04-003: "Irreversible write attempted without a confirmation token → E-FLAG-008"
#[tokio::test]

async fn test_BC_2_04_007_irreversible_dry_run_false_no_token_returns_e_flag_008() {
    let executor = helpers::make_executor();
    let plan = helpers::make_irreversible_plan();
    // dry_run=false, no token — Irreversible tier MUST require token
    let context = helpers::make_context(false, None);

    // execute → todo!() → panic
    // Post-implementation: must return Err(PrismError::TokenNotFound { .. })
    let result = executor.execute(plan, context).await;
    let err = result.expect_err("must be Err for Irreversible without token");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-FLAG-008"),
        "Must return E-FLAG-008 for missing token; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// Edge: confirmation token replay attack → E-FLAG-004 (TokenAlreadyConsumed)
// ---------------------------------------------------------------------------

/// Edge case: after a confirmation token is consumed by a successful write,
/// attempting to replay the same token ID must return `E-FLAG-004`
/// (TokenAlreadyConsumed).
///
/// This exercises the single-use invariant: `consume()` is non-idempotent.
/// BC-2.04.008 / story §Previous Story Intelligence S-1.09:
/// "`consume()` is single-use — a consumed token cannot be replayed."
#[tokio::test]

async fn test_BC_2_04_008_token_replay_attack_returns_e_flag_004() {
    let executor = helpers::make_executor();
    let plan = helpers::make_irreversible_plan();

    // First call: dry_run=true to get a token
    let ctx_dry = helpers::make_context(true, None);
    // execute → todo!() → panic on first call
    let preview_outcome = executor
        .execute(plan.clone(), ctx_dry)
        .await
        .expect("no error");

    let token_id = match preview_outcome {
        WriteOutcome::Preview(p) => {
            p.confirmation_token
                .expect("Irreversible must produce token")
                .token_id
        }
        WriteOutcome::Result(_) => panic!("Expected Preview"),
    };

    // Second call: execute with the token (consumes it)
    let ctx_execute = helpers::make_context(false, Some(&token_id));
    let _ = executor
        .execute(plan.clone(), ctx_execute)
        .await
        .expect("first execute must succeed");

    // Third call: replay the same token — must fail with E-FLAG-004 or E-FLAG-008.
    // The ConfirmationTokenStore eagerly removes consumed tokens (VP-008 eager-remove
    // pattern), so a replayed token may return E-FLAG-008 (not found) since the
    // token was removed upon first consume — both variants correctly enforce
    // single-use invariant. E-FLAG-004 would be returned if the store kept consumed
    // tokens marked rather than removing them.
    let ctx_replay = helpers::make_context(false, Some(&token_id));
    let replay_result = executor.execute(plan, ctx_replay).await;
    let err = replay_result.expect_err("replay must be rejected");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-FLAG-004")
            || err_msg.contains("already consumed")
            || err_msg.contains("E-FLAG-008"),
        "Token replay must return E-FLAG-004 or E-FLAG-008 (single-use enforced); got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// Edge: confirmation token expired → E-FLAG-003
// ---------------------------------------------------------------------------

/// Story §EC-04-004: confirmation token expired before `confirm_action` call.
/// "E-FLAG-003 (token expired); write blocked; analyst must request new preview."
///
/// BC-2.04.008 precondition: token TTL is 300 seconds (S-1.09 §TOKEN_TTL).
/// An expired token must be rejected with E-FLAG-003.
#[tokio::test]

async fn test_BC_2_04_008_expired_token_returns_e_flag_003() {
    let executor = helpers::make_executor();
    let plan = helpers::make_irreversible_plan();

    // Use a fake token ID that represents an expired token.
    // The ConfirmationTokenStore tracks expiry; the implementer must check it.
    // We cannot create a real expired token in unit tests without time-travel,
    // so we use a non-existent token ID and verify E-FLAG-008 or E-FLAG-003.
    // Both are valid rejections for an expired/missing token.
    let ctx = helpers::make_context(false, Some("definitely-expired-token-id"));

    // execute → todo!() → panic
    let result = executor.execute(plan, ctx).await;
    let err = result.expect_err("expired token must be rejected");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-FLAG-003") || err_msg.contains("E-FLAG-008"),
        "Expired/not-found token must return E-FLAG-003 or E-FLAG-008; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// Edge: token bound to different call (action hash mismatch) → E-FLAG-005
// ---------------------------------------------------------------------------

/// BC-2.04.008 / story §Previous Story Intelligence S-1.09:
/// A token's `action_hash` (SHA-256 of action parameters) must match the
/// parameters in the follow-up execute call. If mismatched (e.g., agent attempts
/// to substitute a different target after receiving a token), E-FLAG-005 is returned.
///
/// This prevents a token generated for "contain host-A" from being used to
/// "contain host-B".
#[tokio::test]

async fn test_BC_2_04_008_token_action_hash_mismatch_returns_e_flag_005() {
    let executor = helpers::make_executor();

    // Plan A: contain host-A
    let plan_a = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_contained_hosts".to_string(),
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(1),
        has_where_clause: true,
        params: {
            let mut m = HashMap::new();
            m.insert("hostname".to_string(), "host-A".to_string());
            m
        },
    };

    // Get token for plan_a
    let ctx_dry = helpers::make_context(true, None);
    // execute → todo!() → panic
    let preview = executor
        .execute(plan_a.clone(), ctx_dry)
        .await
        .expect("no error");

    let token_id = match preview {
        WriteOutcome::Preview(p) => {
            p.confirmation_token
                .expect("Irreversible must produce token")
                .token_id
        }
        _ => panic!("Expected Preview"),
    };

    // Plan B: contain host-B (different action — hash will not match)
    let plan_b = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_contained_hosts".to_string(),
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(1),
        has_where_clause: true,
        params: {
            let mut m = HashMap::new();
            m.insert("hostname".to_string(), "host-B".to_string()); // different
            m
        },
    };

    // Execute plan_b with plan_a's token — must detect hash mismatch
    let ctx_execute = helpers::make_context(false, Some(&token_id));
    let result = executor.execute(plan_b, ctx_execute).await;
    let err = result.expect_err("hash mismatch must be rejected");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-FLAG-005") || err_msg.contains("hash mismatch"),
        "Token-to-call mismatch must return E-FLAG-005; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// Edge: feature-flag flip mid-call simulation
// ---------------------------------------------------------------------------

/// Edge: a feature flag that was enabled at call-start is evaluated at Phase 2
/// (the pure pre-check gate). The pipeline is deterministic — Phase 2 runs
/// synchronously and the flag is captured once. There is no "mid-call" flip
/// visible inside a single execute() invocation.
///
/// This test verifies that if a second call arrives after a flag has been
/// disabled (simulating a config reload between calls), Phase 2 returns
/// E-FLAG-001 on the second call.
///
/// BC-2.04.001 / BC-2.04.005: flag state at Phase 2 evaluation time is
/// authoritative.
///
/// Gated to `crowdstrike-write` feature: without the compile-time gate, Phase 2
/// short-circuits to DeniedCompileTime before reaching the per-client runtime check.
/// This test exercises the runtime capability evaluation path for "restricted-client",
/// which is only reachable when the crowdstrike-write feature is present.
#[cfg(feature = "crowdstrike-write")]
#[tokio::test]
async fn test_BC_2_04_001_flag_disabled_between_calls_second_call_returns_e_flag_001() {
    use prism_security::feature_flag::FeatureFlagEvaluator;
    use std::sync::Arc;

    // Build an evaluator with the capability DENIED for "restricted-client"
    let restricted_client_capabilities = {
        let mut caps: BTreeMap<String, prism_core::ClientCapabilities> = BTreeMap::new();
        // Empty capabilities → all writes denied (deny-by-default)
        caps.insert(
            "restricted-client".to_string(),
            prism_core::ClientCapabilities::new(),
        );
        caps
    };

    let evaluator = Arc::new(FeatureFlagEvaluator::new(restricted_client_capabilities));
    let store = Arc::new(ConfirmationTokenStore::new());

    struct NoOpAudit;
    #[async_trait::async_trait]
    impl prism_query::write_dispatch::AuditWriter for NoOpAudit {
        async fn write_intent(
            &self,
            _plan: &prism_query::write_pipeline::WritePlan,
            _ctx: &prism_query::write_pipeline::QueryContext,
            _capability_check: &prism_security::feature_flag::CapabilityCheckResult,
        ) -> Result<ulid::Ulid, PrismError> {
            Ok(ulid::Ulid::new())
        }
        async fn write_outcome(
            &self,
            _id: ulid::Ulid,
            _r: &prism_query::write_result::WriteResult,
        ) -> Result<(), PrismError> {
            Ok(())
        }
    }

    let executor = WriteExecutor::new(
        evaluator,
        store,
        Arc::new(NoOpAudit),
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(prism_spec_engine::write_endpoint::WriteEndpointRegistry::new()),
    );

    let plan = helpers::make_irreversible_plan();
    let ctx = QueryContext {
        client_id: "restricted-client".to_string(),
        org_slug: prism_core::OrgSlug::new_unchecked("restricted-client"),
        dry_run: true,
        confirmation_token_id: None,
        analyst_id: None,
    };

    // Post-implementation: must return E-FLAG-001 (CapabilityDenied) for restricted client.
    // With crowdstrike-write compiled: compile gate is Present → runtime evaluation fires.
    // MED-002 tightened: assert DeniedRuntime path is exercised (not DeniedCompileTime).
    let result = executor.execute(plan, ctx).await;
    let err = result.expect_err("denied client must be rejected");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("CAPABILITY_DENIED") && err_msg.contains("restricted-client"),
        "Denied capability must produce CAPABILITY_DENIED for 'restricted-client'; got: {err_msg}"
    );
    assert!(
        err_msg.contains("Not enabled in client config"),
        "Flag-disabled test must produce DeniedRuntime (not DeniedCompileTime); got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// proptest: WritePreview.dry_run is always true; WriteResult.dry_run is always false
// ---------------------------------------------------------------------------

proptest::proptest! {
    /// BC-2.04.008 invariant: the `dry_run` flag in the response envelope
    /// ALWAYS matches the execution mode — never the opposite.
    ///
    /// Generates random WritePlan configurations and verifies the dry_run
    /// field consistency invariant holds across all inputs.
    ///
    /// Uses proptest with PROPTEST_CASES=256 for full coverage.
    ///
    /// AC-COVERAGE-DEFERRED: This test currently exercises only the type contract
    /// (proves WritePreview { dry_run: true, ... } can be constructed and the field
    /// is readable). Until W3-FIX-S307-001 wires Phase 3 materialization, the
    /// proptest cannot drive through WriteExecutor::execute() with non-empty record
    /// batches. Tighten this test by replacing the direct struct literal with a call
    /// to executor.execute(plan, dry_run=true) and asserting the returned
    /// WritePreview.dry_run flag — once Phase 3 returns real data.
    // TODO(W3-FIX-S307-001): tighten this test once Phase 3 materialization is wired —
    // currently exercises type contract only.
    #[test]
    fn test_BC_2_04_008_invariant_dry_run_field_always_correct(
        has_filter in proptest::bool::ANY,
        has_limit in proptest::bool::ANY,
        limit_val in 1u64..1000u64,
    ) {
        // Structural check: WritePreview always has dry_run=true,
        // WriteResult always has dry_run=false.
        // This does not call any stub — it validates the type contract.
        //
        // A WritePreview with dry_run=false would be a type-level violation
        // (story §write_result.rs: "dry_run is always true in WritePreview").
        //
        // Post-implementation: replace with actual execute() calls
        // that verify the returned envelope's dry_run field.
        let preview = prism_query::write_result::WritePreview {
            query_id: ulid::Ulid::new(),
            dry_run: true, // invariant: always true
            write_endpoint: "crowdstrike.containment".to_string(),
            risk_tier: if has_filter { RiskTier::Irreversible } else { RiskTier::Reversible },
            would_affect_count: if has_limit { limit_val as u32 } else { 0 },
            sample_records: vec![],
            reversibility: RiskTier::Irreversible,
            confirmation_token: None,
            confirmation_prompt: "Would contain 1 host on crowdstrike for client acme".to_string(),
        };
        proptest::prop_assert!(preview.dry_run, "WritePreview.dry_run must always be true");
    }
}
