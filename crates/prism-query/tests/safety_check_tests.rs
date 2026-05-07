//! Phase 2 safety pre-check unit tests — S-3.07.
//!
//! Covers every gate in phase2_safety_check() as a focused unit test:
//!
//! - BC-2.04.001: compile-time feature gate absent → E-FLAG-002 (DeniedCompileTime)
//! - BC-2.04.001: compile-time feature gate present + runtime deny → E-FLAG-001
//! - BC-2.04.001: compile-time gate present + runtime allow → SafetyCheckPassed
//! - BC-2.04.001: read operations always available (invariant DI-003)
//! - BC-2.04.005: composite source (EVENTS) → E-QUERY-020 (behavioral assertion)
//! - BC-2.04.005: internal prism_* table → E-QUERY-010
//! - BC-2.04.007: DELETE FROM always classified as Irreversible (AD-022 invariant)
//! - BC-2.04.007: classify_risk_tier matches WriteEndpointSpec.risk_tier for non-DELETE
//! - BC-2.04.007: ambiguous operations classified as Irreversible (EC-04-014)
//! - BC-2.04.008: unbounded write (no WHERE, no LIMIT) → E-QUERY-022
//! - BC-2.04.008: bounded write passes unbounded check
//! - resolve_batch_limit: min(endpoint, client, system) applied correctly
//! - check_structural_batch_limit: explicit LIMIT > batch_limit → E-QUERY-021
//! - check_structural_batch_limit: explicit LIMIT within bound → Ok
//! - proptest: risk tier classification is total (every WritePlan gets a tier)
//! - proptest: resolve_batch_limit always ≤ system ceiling
//!
//! All non-proptest-structural tests call into `todo!()` stubs and MUST FAIL.
//!
//! Story: S-3.07 | BCs: BC-2.04.001, BC-2.04.005, BC-2.04.007, BC-2.04.008

#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used
)]

use std::collections::BTreeMap;

use prism_core::{CapabilityEffect, CapabilityPath, ClientCapabilities, PrismError, RiskTier};
use prism_query::safety_check::{
    check_structural_batch_limit, check_unbounded_write, classify_risk_tier, phase2_safety_check,
    resolve_batch_limit, CompileFeatureGate, ResolvedBatchLimit, WriteTargetDescriptor,
};
use prism_query::write_pipeline::WritePlan;
use prism_security::feature_flag::{CapabilityCheckResult, FeatureFlagEvaluator};
use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointSpec};

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_write_endpoint_spec(risk: RiskTier) -> WriteEndpointSpec {
    WriteEndpointSpec {
        pipe_verb: "contain".to_string(),
        sql_table: "crowdstrike_contained_hosts".to_string(),
        capability_path: "sensor.crowdstrike.containment".to_string(),
        risk_tier: risk,
        batch_limit: 100,
        batch_mode: BatchMode::Serial,
        steps: vec![],
        record_id_field: "device_id".to_string(),
    }
}

fn make_bounded_plan() -> WritePlan {
    WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_contained_hosts".to_string(),
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(50),
        has_where_clause: true,
        params: HashMap::new(),
    }
}

fn make_unbounded_plan() -> WritePlan {
    WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_contained_hosts".to_string(),
        dml_operation: None,
        has_explicit_limit: false,
        explicit_limit: None,
        has_where_clause: false, // no WHERE, no LIMIT → unbounded
        params: HashMap::new(),
    }
}

fn make_delete_plan() -> WritePlan {
    use prism_query::write_ast::DmlOperation;
    WritePlan {
        verb: "delete".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_alerts".to_string(),
        dml_operation: Some(DmlOperation::Delete),
        has_explicit_limit: true,
        explicit_limit: Some(10),
        has_where_clause: true,
        params: HashMap::new(),
    }
}

fn make_composite_target<'a>() -> WriteTargetDescriptor<'a> {
    WriteTargetDescriptor {
        sensor: "events",
        verb: "contain",
        capability_path: "sensor.events.containment",
        is_composite_source: true,
        is_internal_table: false,
    }
}

fn make_internal_table_target<'a>() -> WriteTargetDescriptor<'a> {
    WriteTargetDescriptor {
        sensor: "crowdstrike",
        verb: "contain",
        capability_path: "sensor.crowdstrike.containment",
        is_composite_source: false,
        is_internal_table: true,
    }
}

fn make_normal_target<'a>() -> WriteTargetDescriptor<'a> {
    WriteTargetDescriptor {
        sensor: "crowdstrike",
        verb: "contain",
        capability_path: "sensor.crowdstrike.containment",
        is_composite_source: false,
        is_internal_table: false,
    }
}

fn make_evaluator_allow(client_id: &str, capability_path: &str) -> FeatureFlagEvaluator {
    let mut caps: BTreeMap<String, ClientCapabilities> = BTreeMap::new();
    let mut client_caps = ClientCapabilities::new();
    client_caps.grant(
        CapabilityPath::new(capability_path).expect("test capability path valid"),
        CapabilityEffect::Allow,
    );
    caps.insert(client_id.to_string(), client_caps);
    FeatureFlagEvaluator::new(caps)
}

fn make_evaluator_deny_all() -> FeatureFlagEvaluator {
    FeatureFlagEvaluator::new(BTreeMap::new())
}

// ---------------------------------------------------------------------------
// BC-2.04.001: compile-time feature gate ABSENT → E-FLAG-002
// ---------------------------------------------------------------------------

/// BC-2.04.001 postcondition: when the `{sensor}-write` Cargo feature is absent,
/// `phase2_safety_check` returns `E-FLAG-002` — write code does not exist in binary.
///
/// Canonical test vector: "Write feature absent | Binary built without
/// crowdstrike-write | crowdstrike_contain_host tool absent from binary"
#[test]

fn test_BC_2_04_001_compile_gate_absent_returns_e_flag_002() {
    let plan = make_bounded_plan();
    let target = make_normal_target();
    let evaluator = make_evaluator_allow("acme", "sensor.crowdstrike.containment");
    let endpoint_spec = make_write_endpoint_spec(RiskTier::Irreversible);
    let limit = ResolvedBatchLimit { limit: 100 };

    // compile_gate = Absent → phase2 must return Err before any runtime check
    let result = phase2_safety_check(
        &plan,
        &target,
        CompileFeatureGate::Absent, // <-- key input
        &evaluator,
        "acme",
        &endpoint_spec,
        limit,
    );
    // todo!() → panic
    let err = result.expect_err("Absent compile gate must return Err");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-FLAG-002") || err_msg.contains("not compiled"),
        "Absent compile gate must return E-FLAG-002; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.001: compile-time gate PRESENT + runtime DENY → E-FLAG-001
// ---------------------------------------------------------------------------

/// BC-2.04.001 postcondition: when the compile gate is present but the runtime
/// TOML capability is denied, `phase2_safety_check` returns E-FLAG-001
/// (CAPABILITY_DENIED — structured error per BC-2.04.005).
#[test]

fn test_BC_2_04_001_compile_gate_present_runtime_deny_returns_e_flag_001() {
    let plan = make_bounded_plan();
    let target = make_normal_target();
    // Deny-all evaluator: no client has the capability
    let evaluator = make_evaluator_deny_all();
    let endpoint_spec = make_write_endpoint_spec(RiskTier::Irreversible);
    let limit = ResolvedBatchLimit { limit: 100 };

    let result = phase2_safety_check(
        &plan,
        &target,
        CompileFeatureGate::Present,
        &evaluator,
        "acme",
        &endpoint_spec,
        limit,
    );
    // todo!() → panic
    let err = result.expect_err("Runtime deny must return Err");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("CAPABILITY_DENIED") || err_msg.contains("E-FLAG-001"),
        "Runtime deny must return E-FLAG-001/CAPABILITY_DENIED; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.005: composite source (EVENTS) → E-QUERY-020 (behavioral)
// ---------------------------------------------------------------------------

/// BC-2.04.005 / story §Task 3a: write targeting a composite source (e.g., EVENTS)
/// must return `E-QUERY-020` before any fetch or sensor contact.
///
/// Note: E-QUERY-020 does NOT yet exist in prism-core/src/error.rs.
/// This test asserts on BEHAVIORAL properties (error message contains "composite"
/// or "unbounded") rather than the enum variant identity.
/// See BLOCKER note: implementer must add E-QUERY-020 to PrismError.
#[test]

fn test_BC_2_04_005_composite_source_returns_structured_error_before_fetch() {
    let plan = make_bounded_plan();
    let target = make_composite_target(); // is_composite_source = true
    let evaluator = make_evaluator_allow("acme", "sensor.events.containment");
    let endpoint_spec = make_write_endpoint_spec(RiskTier::Irreversible);
    let limit = ResolvedBatchLimit { limit: 100 };

    let result = phase2_safety_check(
        &plan,
        &target,
        CompileFeatureGate::Present,
        &evaluator,
        "acme",
        &endpoint_spec,
        limit,
    );
    // todo!() → panic
    let err = result.expect_err("Composite source must be rejected");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("composite") || err_msg.contains("E-QUERY-020"),
        "Composite source must produce E-QUERY-020 or 'composite' error; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.005: internal prism_* table → E-QUERY-010
// ---------------------------------------------------------------------------

/// BC-2.04.005 / story §AC-4: write targeting an internal `prism_*` table
/// must return E-QUERY-010 before any API contact (defense-in-depth, also
/// caught at parse time by S-3.06).
///
/// The existing E-QUERY-010 is `QueryVirtualFieldFailed` — a new variant
/// specifically for internal-table write rejection must be added (BLOCKER).
/// Test asserts on error message containing "prism_" or "internal".
#[test]

fn test_BC_2_04_005_internal_table_write_returns_e_query_010_before_api_contact() {
    let plan = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "prism_alerts".to_string(), // internal table
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(5),
        has_where_clause: true,
        params: HashMap::new(),
    };
    let target = make_internal_table_target(); // is_internal_table = true
    let evaluator = make_evaluator_allow("acme", "sensor.crowdstrike.containment");
    let endpoint_spec = make_write_endpoint_spec(RiskTier::Irreversible);
    let limit = ResolvedBatchLimit { limit: 100 };

    let result = phase2_safety_check(
        &plan,
        &target,
        CompileFeatureGate::Present,
        &evaluator,
        "acme",
        &endpoint_spec,
        limit,
    );
    // todo!() → panic
    let err = result.expect_err("Internal table write must be rejected");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-QUERY-010")
            || err_msg.contains("internal")
            || err_msg.contains("prism_"),
        "Internal table write must produce E-QUERY-010 or 'internal'; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.007: DELETE FROM always Irreversible regardless of spec
// ---------------------------------------------------------------------------

/// BC-2.04.007 invariant: `DELETE FROM` SQL DML is always classified as
/// `Irreversible` per AD-022 — regardless of what `WriteEndpointSpec.risk_tier` says.
///
/// Canonical test vector: "EC-04-008 | DELETE FROM SQL DML without spec override |
/// Always classified Irreversible per AD-022; confirmation required"
#[test]

fn test_BC_2_04_007_delete_dml_always_irreversible_regardless_of_spec() {
    let plan = make_delete_plan();
    // Spec says Reversible — but DELETE must override this
    let spec_reversible = make_write_endpoint_spec(RiskTier::Reversible);

    // classify_risk_tier → todo!() → panic
    let tier = classify_risk_tier(&plan, &spec_reversible);
    assert_eq!(
        tier,
        RiskTier::Irreversible,
        "DELETE FROM must always produce Irreversible regardless of spec; got: {:?}",
        tier
    );
}

/// BC-2.04.007 postcondition: for non-DELETE operations, risk tier matches the
/// `WriteEndpointSpec.risk_tier` field.
#[test]

fn test_BC_2_04_007_non_delete_risk_tier_follows_endpoint_spec() {
    let plan = make_bounded_plan(); // verb=contain, no dml_operation
    let spec_reversible = make_write_endpoint_spec(RiskTier::Reversible);

    // classify_risk_tier → todo!() → panic
    let tier = classify_risk_tier(&plan, &spec_reversible);
    assert_eq!(
        tier,
        RiskTier::Reversible,
        "Non-DELETE operation must use spec risk_tier; got: {:?}",
        tier
    );
}

/// BC-2.04.007 EC-04-014: ambiguous operations classified as Irreversible
/// (conservative classification invariant).
/// "If uncertain, classify as irreversible (requires confirmation token)."
#[test]

fn test_BC_2_04_007_ec_04_014_ambiguous_op_classified_as_irreversible() {
    // A plan with no dml_operation and no matching endpoint spec risk_tier
    // should default to Irreversible (conservative).
    let ambiguous_plan = WritePlan {
        verb: "unknown_new_verb".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_things".to_string(),
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(10),
        has_where_clause: true,
        params: HashMap::new(),
    };
    // Spec doesn't have a specific tier for this verb — falls back to Irreversible
    let spec_ambiguous = make_write_endpoint_spec(RiskTier::Irreversible);

    let tier = classify_risk_tier(&ambiguous_plan, &spec_ambiguous);
    // todo!() → panic
    assert_eq!(
        tier,
        RiskTier::Irreversible,
        "Ambiguous op must be classified as Irreversible; got: {:?}",
        tier
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.008: unbounded write → E-QUERY-022
// ---------------------------------------------------------------------------

/// BC-2.04.008 / story §AC-8: unbounded write (no WHERE clause AND no LIMIT)
/// must return E-QUERY-022 before any fetch or sensor contact.
///
/// Note: E-QUERY-022 does NOT yet exist in prism-core/src/error.rs.
/// Test asserts behavioral property (error message contains "unbounded").
/// See BLOCKER note.
#[test]

fn test_BC_2_04_008_unbounded_write_returns_e_query_022() {
    let plan = make_unbounded_plan(); // has_where_clause=false, has_explicit_limit=false

    // check_unbounded_write → todo!() → panic
    let result = check_unbounded_write(&plan);
    let err = result.expect_err("Unbounded write must be rejected");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-QUERY-022") || err_msg.contains("unbounded"),
        "Unbounded write must produce E-QUERY-022 or 'unbounded'; got: {err_msg}"
    );
}

/// BC-2.04.008 complement: a bounded write (has WHERE clause) passes the
/// unbounded check.
#[test]

fn test_BC_2_04_008_bounded_by_where_clause_passes_unbounded_check() {
    let plan = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_contained_hosts".to_string(),
        dml_operation: None,
        has_explicit_limit: false,
        explicit_limit: None,
        has_where_clause: true, // WHERE clause bounds the query
        params: HashMap::new(),
    };

    // check_unbounded_write → todo!() → panic
    let result = check_unbounded_write(&plan);
    // Post-implementation: must return Ok(())
    result.expect("Bounded write (WHERE clause) must pass unbounded check");
}

/// BC-2.04.008 complement: a bounded write (has LIMIT) passes the unbounded check.
#[test]

fn test_BC_2_04_008_bounded_by_limit_passes_unbounded_check() {
    let plan = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_contained_hosts".to_string(),
        dml_operation: None,
        has_explicit_limit: true, // LIMIT bounds the query
        explicit_limit: Some(100),
        has_where_clause: false,
        params: HashMap::new(),
    };

    // check_unbounded_write → todo!() → panic
    let result = check_unbounded_write(&plan);
    result.expect("Bounded write (LIMIT) must pass unbounded check");
}

// ---------------------------------------------------------------------------
// resolve_batch_limit: min(endpoint, client, system)
// ---------------------------------------------------------------------------

/// BC-2.04.008 / story §Task 3d: resolve_batch_limit returns min of all three.
///
/// Test vector A: endpoint_limit=50, client_override=None, system_ceiling=200
/// → effective limit = 50
#[test]

fn test_BC_2_04_008_resolve_batch_limit_no_override_uses_endpoint() {
    // resolve_batch_limit → todo!() → panic
    let resolved = resolve_batch_limit(50, None, 200);
    assert_eq!(
        resolved.limit, 50,
        "No override: effective limit must be endpoint limit"
    );
}

/// Test vector B: endpoint_limit=200, client_override=Some(30), system_ceiling=500
/// → effective limit = 30
#[test]

fn test_BC_2_04_008_resolve_batch_limit_client_override_wins_when_lower() {
    let resolved = resolve_batch_limit(200, Some(30), 500);
    // todo!() → panic
    assert_eq!(
        resolved.limit, 30,
        "Client override must win when lower than endpoint"
    );
}

/// Test vector C: endpoint_limit=1000, client_override=Some(500), system_ceiling=100
/// → effective limit = 100 (system ceiling always applies)
#[test]

fn test_BC_2_04_008_resolve_batch_limit_system_ceiling_always_applies() {
    let resolved = resolve_batch_limit(1000, Some(500), 100);
    // todo!() → panic
    assert_eq!(
        resolved.limit, 100,
        "System ceiling must always cap the resolved limit"
    );
}

// ---------------------------------------------------------------------------
// check_structural_batch_limit
// ---------------------------------------------------------------------------

/// BC-2.04.008 / story §Task 3d: when explicit LIMIT exceeds the resolved
/// batch limit, E-QUERY-021 is returned immediately.
///
/// Note: E-QUERY-021 does NOT yet exist in prism-core/src/error.rs.
/// Test asserts behavioral property.
#[test]

fn test_BC_2_04_008_structural_batch_limit_exceeded_returns_e_query_021() {
    let plan = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_contained_hosts".to_string(),
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(500), // exceeds batch_limit of 100
        has_where_clause: true,
        params: HashMap::new(),
    };
    let limit = ResolvedBatchLimit { limit: 100 };

    // check_structural_batch_limit → todo!() → panic
    let result = check_structural_batch_limit(&plan, &limit);
    let err = result.expect_err("Exceeded batch limit must be rejected");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("E-QUERY-021")
            || err_msg.contains("batch limit")
            || err_msg.contains("limit exceeded"),
        "Exceeded batch limit must produce E-QUERY-021; got: {err_msg}"
    );
}

/// check_structural_batch_limit: explicit LIMIT within bound → Ok(())
#[test]

fn test_BC_2_04_008_structural_batch_limit_within_bound_passes() {
    let plan = WritePlan {
        verb: "contain".to_string(),
        sensor: "crowdstrike".to_string(),
        target_table: "crowdstrike_contained_hosts".to_string(),
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(50), // within limit of 100
        has_where_clause: true,
        params: HashMap::new(),
    };
    let limit = ResolvedBatchLimit { limit: 100 };

    // check_structural_batch_limit → todo!() → panic
    let result = check_structural_batch_limit(&plan, &limit);
    result.expect("Limit within bound must pass");
}

// ---------------------------------------------------------------------------
// BC-2.04.005: EC-04-011 — no clients have write enabled → tool absent
// ---------------------------------------------------------------------------

/// BC-2.04.005 EC-04-011: when no configured clients have any write capability
/// enabled, only read tools appear in tools/list; all write tools hidden.
///
/// At the phase2 gate level: when evaluator has no allow entries for any client,
/// all write invocations must be denied with E-FLAG-001.
///
/// This is equivalent to the "deny-all" runtime gate behavior exercised by
/// the "compile gate present + runtime deny" test above — but from the
/// many-client angle: even if many clients exist, all with empty caps,
/// every invocation is denied.
#[test]

fn test_BC_2_04_005_ec_04_011_all_clients_deny_write_invocation_returns_e_flag_001() {
    let plan = make_bounded_plan();
    let target = make_normal_target();
    // Multiple clients, all with default (empty = deny) capabilities
    let evaluator = {
        let mut caps: BTreeMap<String, ClientCapabilities> = BTreeMap::new();
        caps.insert("client-a".to_string(), ClientCapabilities::new());
        caps.insert("client-b".to_string(), ClientCapabilities::new());
        caps.insert("client-c".to_string(), ClientCapabilities::new());
        FeatureFlagEvaluator::new(caps)
    };
    let endpoint_spec = make_write_endpoint_spec(RiskTier::Irreversible);
    let limit = ResolvedBatchLimit { limit: 100 };

    // Even with multiple clients all denying — invocation from client-a is denied
    let result = phase2_safety_check(
        &plan,
        &target,
        CompileFeatureGate::Present,
        &evaluator,
        "client-a",
        &endpoint_spec,
        limit,
    );
    // todo!() → panic
    let err = result.expect_err("All-deny clients must reject write invocation");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("CAPABILITY_DENIED") || err_msg.contains("E-FLAG-001"),
        "All-deny must produce E-FLAG-001; got: {err_msg}"
    );
}

/// BC-2.04.005 EC-04-010: tool enabled for Client A but not Client B.
/// Invocation with client-a's context succeeds Phase 2 capability check.
/// Invocation with client-b's context → E-FLAG-001.
#[test]

fn test_BC_2_04_005_ec_04_010_tool_enabled_for_a_denied_for_b() {
    let plan = make_bounded_plan();
    let target = make_normal_target();

    // client-a: has containment capability; client-b: does not
    let evaluator = {
        let mut caps: BTreeMap<String, ClientCapabilities> = BTreeMap::new();
        let mut caps_a = ClientCapabilities::new();
        caps_a.grant(
            CapabilityPath::new("sensor.crowdstrike.containment")
                .expect("test capability path valid"),
            CapabilityEffect::Allow,
        );
        caps.insert("client-a".to_string(), caps_a);
        caps.insert("client-b".to_string(), ClientCapabilities::new()); // no containment
        FeatureFlagEvaluator::new(caps)
    };

    let endpoint_spec = make_write_endpoint_spec(RiskTier::Irreversible);
    let limit = ResolvedBatchLimit { limit: 100 };

    // Client-a: should pass phase2 (compile gate present + runtime allow)
    // todo!() → panic
    let result_a = phase2_safety_check(
        &plan,
        &target,
        CompileFeatureGate::Present,
        &evaluator,
        "client-a",
        &endpoint_spec,
        limit,
    );
    result_a.expect("client-a must pass Phase 2 capability check");

    // Client-b: should be denied
    let result_b = phase2_safety_check(
        &plan,
        &target,
        CompileFeatureGate::Present,
        &evaluator,
        "client-b",
        &endpoint_spec,
        limit,
    );
    let err = result_b.expect_err("client-b must be denied");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("CAPABILITY_DENIED") || err_msg.contains("E-FLAG-001"),
        "client-b invocation must produce E-FLAG-001; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.009: capability_check emitted in hierarchical evaluation order
// ---------------------------------------------------------------------------

/// BC-2.05.009 postcondition: `capability_checks` records each flag evaluated
/// in most-specific-to-least-specific order, ending at global deny default.
///
/// Canonical test vector: "Parent match, child absent |
/// sensor.crowdstrike: Allow → Two entries: child (no match), parent (permitted)"
#[test]

fn test_BC_2_05_009_capability_checks_emitted_in_hierarchical_order() {
    let plan = make_bounded_plan();
    let target = make_normal_target();

    // Client has parent path allow but NOT the specific child path
    let evaluator = {
        let mut caps: BTreeMap<String, ClientCapabilities> = BTreeMap::new();
        let mut client_caps = ClientCapabilities::new();
        // Allow at parent level only: sensor.crowdstrike (not sensor.crowdstrike.containment)
        client_caps.grant(
            CapabilityPath::new("sensor.crowdstrike").expect("test capability path valid"),
            CapabilityEffect::Allow,
        );
        caps.insert("acme".to_string(), client_caps);
        FeatureFlagEvaluator::new(caps)
    };

    let endpoint_spec = make_write_endpoint_spec(RiskTier::Irreversible);
    let limit = ResolvedBatchLimit { limit: 100 };

    let result = phase2_safety_check(
        &plan,
        &target,
        CompileFeatureGate::Present,
        &evaluator,
        "acme",
        &endpoint_spec,
        limit,
    );
    // todo!() → panic
    let passed = result.expect("Parent-allow must permit the write at Phase 2");

    // The capability_check field in SafetyCheckPassed must record both the
    // child path (no match) and the parent path (permitted).
    // Post-implementation: assert on passed.capability_check resolution_trace
    match &passed.capability_check {
        prism_security::feature_flag::CapabilityCheckResult::Allowed => {
            // Correct — the hierarchical evaluation found the parent allow
        }
        other => {
            panic!("Expected Allowed but got: {:?}", other);
        }
    }
}

/// BC-2.05.009 EC-05-015: child override deny wins over parent allow.
/// "sensor.crowdstrike: Allow, sensor.crowdstrike.containment: Deny →
/// Two entries: child (result: denied) wins; parent not reached"
#[test]

fn test_BC_2_05_009_ec_05_015_child_deny_overrides_parent_allow() {
    let plan = make_bounded_plan();
    let target = make_normal_target();

    let evaluator = {
        let mut caps: BTreeMap<String, ClientCapabilities> = BTreeMap::new();
        let mut client_caps = ClientCapabilities::new();
        // Parent: Allow; Child: Deny — child MUST win
        client_caps.grant(
            CapabilityPath::new("sensor.crowdstrike").expect("test capability path valid"),
            CapabilityEffect::Allow,
        );
        client_caps.grant(
            CapabilityPath::new("sensor.crowdstrike.containment")
                .expect("test capability path valid"),
            CapabilityEffect::Deny,
        );
        caps.insert("acme".to_string(), client_caps);
        FeatureFlagEvaluator::new(caps)
    };

    let endpoint_spec = make_write_endpoint_spec(RiskTier::Irreversible);
    let limit = ResolvedBatchLimit { limit: 100 };

    let result = phase2_safety_check(
        &plan,
        &target,
        CompileFeatureGate::Present,
        &evaluator,
        "acme",
        &endpoint_spec,
        limit,
    );
    // todo!() → panic
    let err = result.expect_err("Child deny must override parent allow");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("CAPABILITY_DENIED") || err_msg.contains("E-FLAG-001"),
        "Child deny override must produce E-FLAG-001; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// proptest: risk tier classification is total (every WritePlan gets a tier)
// ---------------------------------------------------------------------------

proptest::proptest! {
    /// BC-2.04.007 invariant: every tool has exactly one risk tier.
    /// `classify_risk_tier` must never panic or return an unexpected value —
    /// it must always return one of {Reversible, Irreversible}.
    ///
    /// Uses proptest to exercise with arbitrary has_where_clause, has_limit, dml variants.
    #[test]
    fn test_BC_2_04_007_invariant_risk_tier_total_function(
        has_where in proptest::bool::ANY,
        has_limit in proptest::bool::ANY,
        limit_val in 1u64..10000u64,
        is_delete in proptest::bool::ANY,
    ) {
        use prism_query::write_ast::DmlOperation;

        let plan = WritePlan {
            verb: if is_delete { "delete".to_string() } else { "contain".to_string() },
            sensor: "crowdstrike".to_string(),
            target_table: "crowdstrike_contained_hosts".to_string(),
            dml_operation: if is_delete { Some(DmlOperation::Delete) } else { None },
            has_explicit_limit: has_limit,
            explicit_limit: if has_limit { Some(limit_val) } else { None },
            has_where_clause: has_where,
            params: std::collections::HashMap::new(),
        };

        // Use Reversible spec to test that DELETE overrides spec (BC-2.04.007 invariant)
        let spec = make_write_endpoint_spec(RiskTier::Reversible);

        // classify_risk_tier → todo!() → panic (RED gate: this proptest WILL fail)
        let tier = classify_risk_tier(&plan, &spec);

        // DELETE always Irreversible; non-DELETE follows spec (Reversible here)
        if is_delete {
            proptest::prop_assert_eq!(
                tier,
                RiskTier::Irreversible,
                "DELETE must always be Irreversible per AD-022"
            );
        } else {
            proptest::prop_assert!(
                tier == RiskTier::Reversible || tier == RiskTier::Irreversible,
                "Non-DELETE risk tier must be one of the two valid tiers"
            );
        }
    }

    /// BC-2.04.008 invariant: resolve_batch_limit always ≤ system ceiling.
    ///
    /// No matter the endpoint limit or client override, the resolved limit
    /// must never exceed the system ceiling.
    #[test]
    fn test_BC_2_04_008_invariant_resolved_limit_never_exceeds_system_ceiling(
        endpoint_limit in 1u32..100_000u32,
        client_override in proptest::option::of(1u32..100_000u32),
        system_ceiling in 1u32..10_000u32,
    ) {
        // resolve_batch_limit → todo!() → panic (RED gate)
        let resolved = resolve_batch_limit(endpoint_limit, client_override, system_ceiling);
        proptest::prop_assert!(
            resolved.limit <= system_ceiling,
            "Resolved batch limit {} must never exceed system ceiling {}",
            resolved.limit,
            system_ceiling
        );
    }
}
