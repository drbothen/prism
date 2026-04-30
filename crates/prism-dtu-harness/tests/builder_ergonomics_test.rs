//! Builder Ergonomics Test Suite — S-3.3.05 Red Gate
//!
//! Covers:
//!   AC-001 — CustomerSpec new() defaults all four override fields to None
//!   AC-002 — with_customer + with_customer_overrides applies closure to existing spec
//!   AC-003 — with_failure shorthand sets initial_failure on the matching CustomerSpec
//!   AC-004 — Failure injected via with_failure is observable on first request
//!   AC-005 — with_failure with unknown slug defers error to build(), no panic on call
//!   AC-006 — Override fields take precedence over CustomerSpec field defaults
//!   AC-007 — Builder is fully fluent (all methods return Self; single chain compiles)
//!   AC-008 — Rustdoc examples compile (doc test exercises with_customer_overrides +
//!            with_failure in a chain; panics via todo!() when run — RED)
//!
//! BCs exercised:
//!   BC-3.5.001 precondition 2 (at least one customer registered)
//!   BC-3.5.002 (ergonomics layer over network isolation)
//!   BC-3.6.001 postcondition 1 (initial_failure applied before first request)
//!   BC-3.6.001 EC-001 (unknown org deferred to build)
//!
//! Verification properties:
//!   VP-122 (endpoints entry count) — endpoint counts verify no duplicate-slug insertion
//!   VP-128 (failure injection isolation) — AC-004 exercises initial_failure scope
//!
//! # Red Gate expectations by test
//!
//! | Test | Expected RED failure | Reason |
//! |------|---------------------|--------|
//! | AC-001 (field defaults) | PASSES — fields already None in stub | Locks default contract |
//! | AC-002 (deduplicated endpoints) | FAILS — assert endpoints.len()==4 fails (stub inserts 2 specs → 8 endpoints) | Stub creates duplicate spec |
//! | AC-002 (single-dtu deduplicate) | FAILS — endpoints.len()==1 fails (stub inserts 2 specs → 2 endpoints) | Stub creates duplicate spec |
//! | EC-003 (last-write-wins) | FAILS — endpoints.len() is wrong | Duplicate spec insertion |
//! | AC-003 / AC-004 / AC-007 / AC-008 | PANICS — `with_failure` has `todo!()` body | todo!() panic |
//! | AC-005 | PANICS — `with_failure` panics instead of deferring | todo!() panic |
//! | AC-006 | FAILS — duplicate spec, scale on second spec not the first | Duplicate spec |
//! | EC-002 / EC-002 clear | PANICS — `with_failure` has `todo!()` body | todo!() panic |
//! | "with_customer_overrides creates new spec" | PASSES — existing stub behavior | Baseline guard |
//!
//! # Test naming
//!
//! `test_BC_S_SS_NNN_xxx()` pattern throughout (Factory TDD spec).
// Allow test-file conventions: expect() in assertions and BC-tracing names.
#![allow(clippy::expect_used, non_snake_case)]

use prism_dtu_common::FailureMode;
use prism_dtu_harness::{CustomerSpec, DtuType, HarnessError, IsolationMode};

// ============================================================================
// AC-001 — CustomerSpec::new() defaults all four override fields to None
//
// BC-3.5.001 precondition 2 (spec constructed with no overrides before builder call)
// S-3.3.05 Task 1
// ============================================================================

/// AC-001: CustomerSpec constructed via `CustomerSpec::new(org_id, slug)` must have
/// `archetype`, `scale`, `seed_override`, and `initial_failure` all equal to `None`.
///
/// The four fields were added in S-3.3.05. Tests that pre-date S-3.3.05 use
/// `CustomerSpec::new` without setting these fields; the `None` default ensures those
/// test sites remain valid without any change.
///
/// (S-3.3.05 Task 1; BC-3.5.001 precondition 2; AC-001)
///
/// NOTE: Expected to PASS at the Red Gate — the stub already sets defaults to `None`.
/// This test locks the contract so the implementer cannot accidentally change defaults.
#[test]
fn test_BC_3_5_001_customer_spec_new_override_fields_default_to_none() {
    use prism_core::ids::OrgId;
    use prism_core::tenant::OrgSlug;

    let org_id = OrgId::new();
    let org_slug = OrgSlug::new("acme-corp");
    let spec = CustomerSpec::new(org_id, org_slug);

    assert!(
        spec.archetype.is_none(),
        "CustomerSpec::new must set archetype = None (AC-001; S-3.3.05 Task 1)"
    );
    assert!(
        spec.scale.is_none(),
        "CustomerSpec::new must set scale = None (AC-001; S-3.3.05 Task 1)"
    );
    assert!(
        spec.seed_override.is_none(),
        "CustomerSpec::new must set seed_override = None (AC-001; S-3.3.05 Task 1)"
    );
    assert!(
        spec.initial_failure.is_none(),
        "CustomerSpec::new must set initial_failure = None (AC-001; S-3.3.05 Task 1)"
    );
}

/// AC-001 (Default impl): `CustomerSpec::default()` also produces all-None override fields.
///
/// (S-3.3.05 Task 1)
///
/// NOTE: Expected to PASS at the Red Gate.
#[test]
fn test_BC_3_5_001_customer_spec_default_override_fields_all_none() {
    let spec = CustomerSpec::default();

    assert!(
        spec.archetype.is_none(),
        "CustomerSpec::default must set archetype = None"
    );
    assert!(
        spec.scale.is_none(),
        "CustomerSpec::default must set scale = None"
    );
    assert!(
        spec.seed_override.is_none(),
        "CustomerSpec::default must set seed_override = None"
    );
    assert!(
        spec.initial_failure.is_none(),
        "CustomerSpec::default must set initial_failure = None"
    );
}

// ============================================================================
// AC-002 — with_customer + with_customer_overrides applies to existing spec
//
// BC-3.5.001 precondition 2; S-3.3.05 Task 2; story EC-003 (multiple overrides)
//
// RED: current `with_customer_overrides` creates a NEW spec instead of mutating
// the one inserted by `with_customer`. The harness will have 2 specs for the same
// slug → 8 endpoints instead of 4. The assertions on endpoint count detect this.
// ============================================================================

/// AC-002: `.with_customer("alpha")` followed by
/// `.with_customer_overrides("alpha", |c| { c.dtu_types = vec![DtuType::Claroty]; })`
/// must produce a harness with exactly ONE entry for "alpha" in the DTU type
/// Claroty — specifically ONE endpoint, not TWO (which would indicate two specs
/// were inserted for the same slug, one with default 4 DTUs and one with 1 DTU).
///
/// Observable proxy: we override `dtu_types` to `[DtuType::Claroty]` only.
/// If the slug is deduplicated correctly → 1 endpoint.
/// If the stub creates a duplicate spec → 2 endpoints (4 from first + 1 from second,
/// but since Claroty overlaps the count lands at 1+4 = 5 or 4+1 = 5 minus deduplication...).
/// Actually the (OrgId, DtuType) key is unique per OrgId, and each call to
/// `with_customer` / `with_customer_overrides` creates a *new* OrgId. So two specs
/// means two different OrgId keys → both sets of endpoints coexist → 4+1=5 endpoints.
///
/// With correct deduplication: 1 spec for "alpha" with dtu_types=[Claroty] → 1 endpoint.
///
/// (S-3.3.05 Task 2; BC-3.5.001 precondition 2; AC-002)
///
/// RED: stub creates 2 specs (different OrgIds), endpoints.len() == 5, not 1.
#[tokio::test]
async fn test_BC_3_5_001_with_customer_then_overrides_deduplicates_slug() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("alpha")
        .with_customer_overrides("alpha", |c| {
            // Reduce to 1 DTU type so the endpoint count distinguishes
            // 1 spec (1 endpoint) from 2 specs (4 + 1 = 5 endpoints, as each
            // spec gets a fresh OrgId and thus non-overlapping keys).
            c.dtu_types = vec![DtuType::Claroty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let endpoints = harness.endpoints();
    assert_eq!(
        endpoints.len(),
        1,
        "with_customer + with_customer_overrides for same slug must produce exactly 1 endpoint \
         (1 org × 1 DTU type after override); got {} — stub inserts 2 specs (4+1=5 endpoints) \
         because with_customer_overrides creates a second spec instead of mutating the first; \
         implementer must deduplicate by slug (AC-002)",
        endpoints.len()
    );
}

/// AC-002 (override values applied): When `with_customer_overrides` mutates the existing
/// spec, the `scale` and `seed_override` fields written by the closure must be reflected
/// in the harness's behavior. We verify via a build success check — the harness must
/// build with the overridden dtu_types (1 type → 1 endpoint).
///
/// (S-3.3.05 Task 2; AC-002)
///
/// RED: stub creates duplicate spec → 5 endpoints, assertion fails.
#[tokio::test]
async fn test_BC_3_5_001_with_customer_overrides_scale_and_seed_applied_to_existing_spec() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("alpha")
        .with_customer_overrides("alpha", |c| {
            c.scale = Some(2.0);
            c.seed_override = Some(42);
            // Also restrict to 1 DTU type so endpoint count is 1 (detects deduplication).
            c.dtu_types = vec![DtuType::Claroty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    // If the override was applied to the existing spec, there's 1 endpoint.
    // If the stub created a duplicate spec, there are 5 endpoints.
    assert_eq!(
        harness.endpoints().len(),
        1,
        "with_customer + with_customer_overrides must deduplicate; expected 1 endpoint, \
         got {} (AC-002; S-3.3.05 Task 2)",
        harness.endpoints().len()
    );
}

/// EC-003: Multiple `with_customer_overrides` calls for the same slug apply in order;
/// last write wins for overlapping fields.
///
/// Observable: restrict dtu_types in the last override → 1 endpoint.
/// If any of the three calls creates a new spec: 4 + 4 + 1 = 9 endpoints.
///
/// (S-3.3.05 EC-003; AC-002)
///
/// RED: stub inserts duplicate specs → > 1 endpoint.
#[tokio::test]
async fn test_BC_3_5_001_multiple_overrides_same_slug_last_write_wins() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("alpha")
        .with_customer_overrides("alpha", |c| {
            // First override: seed_override = 1
            c.seed_override = Some(1);
        })
        .with_customer_overrides("alpha", |c| {
            // Second override: seed_override = 99 (must win) AND narrow DTU to 1 type
            c.seed_override = Some(99);
            c.dtu_types = vec![DtuType::Claroty];
        })
        .build()
        .await
        .expect("3-override chain must build");

    // If all three calls (with_customer + 2× with_customer_overrides) deduplicate to 1 spec:
    // dtu_types = [Claroty] → 1 endpoint.
    // If each creates a new spec: 4 + 4 + 1 = 9 endpoints (three different OrgIds).
    assert_eq!(
        harness.endpoints().len(),
        1,
        "3 calls for same slug must deduplicate to 1 spec with 1 endpoint; \
         got {} (EC-003; last-write-wins)",
        harness.endpoints().len()
    );
}

// ============================================================================
// AC-003 — with_failure shorthand sets initial_failure on the matching spec
//
// BC-3.6.001 postcondition 1; S-3.3.05 Task 3; ADR-011 §2.7
//
// RED: `with_failure` body is `todo!()` — panics immediately when called.
// ============================================================================

/// AC-003: `.with_customer("alpha").with_failure("alpha", DtuType::Claroty, FailureMode::AuthReject)`
/// must build a harness where alpha's Claroty clone returns HTTP 401 on the first request.
///
/// We verify the failure was pre-injected by making an HTTP request immediately
/// after `build()` — no explicit `inject_failure` call needed.
///
/// (BC-3.6.001 postcondition 1; S-3.3.05 Task 3; AC-003)
///
/// RED: `with_failure` panics via `todo!("S-3.3.05: with_failure ...")`.
#[tokio::test]
async fn test_BC_3_6_001_with_failure_pre_injects_failure_into_named_dtu() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("alpha")
        .with_failure("alpha", DtuType::Claroty, FailureMode::AuthReject)
        .build()
        .await
        .expect("harness must build with with_failure (AC-003)");

    let addr = harness
        .endpoint_for("alpha", DtuType::Claroty)
        .expect("alpha/Claroty endpoint must exist");

    let status = reqwest::get(format!("http://{}/assets/v1/assets", addr))
        .await
        .expect("HTTP GET must not fail at network level")
        .status()
        .as_u16();

    assert_eq!(
        status, 401,
        "alpha/Claroty must return 401 on first request — failure pre-injected via \
         with_failure without a separate inject_failure call (AC-003; BC-3.6.001 pc-1)"
    );
}

/// AC-003 (Network mode): with_failure works identically under IsolationMode::Network.
///
/// (BC-3.5.002 — failure injection works identically in both modes; BC-3.6.001)
///
/// RED: `with_failure` panics via `todo!()`.
#[tokio::test]
async fn test_BC_3_6_001_with_failure_works_in_network_mode() {
    // Build the harness; the network mode handler for with_failure must also work.
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer("beta")
        .with_failure("beta", DtuType::Claroty, FailureMode::MalformedResponse)
        .build()
        .await
        .expect("network-mode harness must build with with_failure (AC-003)");

    let addr = harness
        .endpoint_for("beta", DtuType::Claroty)
        .expect("beta/Claroty endpoint must exist in network mode");

    let body = reqwest::get(format!("http://{}/assets/v1/assets", addr))
        .await
        .expect("HTTP GET must not fail at network level")
        .text()
        .await
        .expect("response body as text");

    let parse_result: Result<serde_json::Value, _> = serde_json::from_str(&body);
    assert!(
        parse_result.is_err(),
        "beta/Claroty must return malformed JSON — failure pre-injected via with_failure \
         in network mode (AC-003; BC-3.5.002)"
    );
}

// ============================================================================
// AC-004 — Failure injected via with_failure is observable on first request
//
// BC-3.6.001 postcondition 1; AC-004; ADR-011 §2.7
//
// RED: `with_failure` panics via `todo!()` before build() is even called.
// ============================================================================

/// AC-004: After `.with_failure("alpha", DtuType::Claroty, FailureMode::AuthReject)`,
/// the first HTTP GET to alpha's Claroty clone endpoint must return HTTP 401.
///
/// No explicit `harness.inject_failure(...)` call is required — the failure mode is
/// pre-applied during `build()` (BC-3.6.001 postcondition 1; ADR-011 §2.7).
///
/// (BC-3.6.001 postcondition 1; AC-004; S-3.3.05 Task 3)
///
/// RED: `with_failure` panics via `todo!()`.
#[tokio::test]
async fn test_BC_3_6_001_with_failure_auth_reject_observable_on_first_request() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("alpha")
        .with_failure("alpha", DtuType::Claroty, FailureMode::AuthReject)
        .build()
        .await
        .expect("harness build must succeed (AC-004)");

    // The FIRST request to alpha's Claroty clone must see HTTP 401 without any
    // explicit inject_failure call (BC-3.6.001 postcondition 1).
    let addr = harness
        .endpoint_for("alpha", DtuType::Claroty)
        .expect("alpha/Claroty endpoint must exist after build");

    let status = reqwest::get(format!("http://{}/assets/v1/assets", addr))
        .await
        .expect("HTTP GET must not fail at network level")
        .status()
        .as_u16();

    assert_eq!(
        status, 401,
        "first request to alpha/Claroty must return 401 — pre-injected AuthReject via \
         with_failure (AC-004; BC-3.6.001 postcondition 1)"
    );
}

/// AC-004 (Timeout variant): `with_failure(..., FailureMode::NetworkTimeout { after_ms: 2000 })`
/// causes the first request to the target clone to delay by ~2s.
///
/// A second org's clone is queried concurrently; its latency must be < 200ms,
/// proving failure injection is scoped to the target `(org, dtu)`.
///
/// (BC-3.6.001 postcondition 1 + Invariant 1; AC-004; VP-128)
///
/// RED: `with_failure` panics via `todo!()`.
#[tokio::test]
async fn test_BC_3_6_001_with_failure_timeout_observable_and_scoped_to_target_org() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("alpha")
        .with_failure(
            "alpha",
            DtuType::Claroty,
            FailureMode::NetworkTimeout { after_ms: 2000 },
        )
        .with_customer("beta")
        .build()
        .await
        .expect("harness build must succeed");

    // OrgB (beta) must respond promptly — failure injection must not bleed across orgs.
    let beta_addr = harness
        .endpoint_for("beta", DtuType::Claroty)
        .expect("beta/Claroty endpoint must exist");

    let start = std::time::Instant::now();
    let beta_status = reqwest::get(format!("http://{}/assets/v1/assets", beta_addr))
        .await
        .expect("HTTP GET to beta must not fail at network level")
        .status()
        .as_u16();
    let elapsed = start.elapsed();

    assert_eq!(
        beta_status, 200,
        "beta/Claroty must return HTTP 200 — unaffected by alpha's timeout injection (VP-128)"
    );
    assert!(
        elapsed.as_millis() < 200,
        "beta/Claroty must respond in < 200ms; got {}ms — timeout injection must not affect \
         other orgs (AC-004; VP-128; BC-3.6.001 Invariant 1)",
        elapsed.as_millis()
    );
}

// ============================================================================
// AC-005 — Unknown slug defers error to build(), no panic on builder call
//
// BC-3.6.001 EC-001; S-3.3.05 Task 3; story EC-001
//
// RED: `with_failure` currently panics via `todo!()` instead of deferring.
// After implementation: must NOT panic on the builder call; must return
// Err(HarnessError::UnknownOrg) from build().
// ============================================================================

/// AC-005: Calling `.with_failure("unknown_slug", DtuType::Claroty, FailureMode::AuthReject)`
/// for a slug that was never registered via `with_customer` must NOT panic at the
/// `with_failure` call site. The error is deferred to `build()`, which returns
/// `Err(HarnessError::UnknownOrg { slug: "unknown_slug" })`.
///
/// (BC-3.6.001 EC-001; S-3.3.05 Task 3; AC-005; story EC-001)
///
/// RED: the current stub panics via `todo!()` at the `with_failure` call site, so
/// the test panics before reaching the assertion.
/// After implementation: `with_failure` returns `self` (defers); `build()` returns Err.
#[tokio::test]
async fn test_BC_3_6_001_with_failure_unknown_slug_deferred_to_build() {
    // This call must NOT panic — the builder call is infallible (AC-005).
    // RED: panics via todo!() in the stub.
    let result = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("alpha")
        .with_failure("unknown_slug", DtuType::Claroty, FailureMode::AuthReject)
        .build()
        .await;

    assert!(
        matches!(result, Err(HarnessError::UnknownOrg { ref slug }) if slug == "unknown_slug"),
        "build() must return Err(UnknownOrg {{ slug: \"unknown_slug\" }}) when with_failure \
         references an unregistered slug (AC-005; BC-3.6.001 EC-001)"
    );
}

/// AC-005 (variant): `with_failure` for an unknown slug when NO customers are registered.
///
/// (BC-3.6.001 EC-001; AC-005)
///
/// RED: panics via `todo!()`.
#[tokio::test]
async fn test_BC_3_6_001_with_failure_no_customers_deferred_to_build() {
    let result = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_failure("nobody", DtuType::Claroty, FailureMode::AuthReject)
        .build()
        .await;

    assert!(
        matches!(result, Err(HarnessError::UnknownOrg { .. })),
        "build() must return UnknownOrg when with_failure references a slug \
         and no customers are registered (AC-005)"
    );
}

// ============================================================================
// AC-006 — Override fields take precedence over CustomerSpec field defaults
//
// S-3.3.05 Task 1 + Task 2; BC-3.5.001 precondition 2
//
// Observable proxy: restrict dtu_types to 1 via override, verify endpoint count = 1.
// If the override applied to the existing spec → dtu_types=[Claroty] → 1 endpoint.
// If a duplicate spec is created → the original spec has 4 types, the override spec has
// 1 type → 5 endpoints total (different OrgIds → non-overlapping keys).
//
// RED: stub creates duplicate spec → 5 endpoints.
// ============================================================================

/// AC-006: Override fields applied via `with_customer_overrides` (called after
/// `with_customer`) take precedence over the default values in `CustomerSpec::new`.
///
/// Test verifies via dtu_types override: default is 4 types, override sets 1 type.
/// Correct deduplication → 1 endpoint. Duplicate insertion → 5 endpoints.
///
/// (S-3.3.05 Tasks 1 & 2; BC-3.5.001 precondition 2; AC-006)
///
/// RED: stub creates duplicate spec → 5 endpoints → assertion fails.
#[tokio::test]
async fn test_BC_3_5_001_override_fields_take_precedence_over_defaults() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("gamma")
        .with_customer_overrides("gamma", |c| {
            // Override dtu_types (default: 4 types) to 1 type.
            // This is the observable proxy for "override wins over default".
            c.dtu_types = vec![DtuType::CrowdStrike];
            // Also verify scale and seed_override can be set together without panic.
            c.scale = Some(3.0);
            c.seed_override = Some(999);
        })
        .build()
        .await
        .expect("override harness must build (AC-006)");

    assert_eq!(
        harness.endpoints().len(),
        1,
        "override dtu_types=[CrowdStrike] must yield 1 endpoint; got {} — \
         if > 1, the override was applied to a duplicate spec rather than the existing spec \
         (AC-006; S-3.3.05 Task 2)",
        harness.endpoints().len()
    );
}

// ============================================================================
// AC-007 — Builder is fully fluent (all methods return Self; chain compiles + runs)
//
// BC-3.5.001 precondition 2; S-3.3.05 Task 4
//
// RED: chain includes `with_failure` → todo!() panic.
// ============================================================================

/// AC-007: A 5-step fluent chain compiles and, after implementation, builds a valid
/// harness. The chain must not require any intermediate `let` bindings.
///
/// Exercises: `isolation`, `with_customer`, `with_customer_overrides`,
/// `with_failure`, `build().await`.
///
/// (BC-3.5.001 precondition 2; S-3.3.05 Task 4; AC-007)
///
/// RED: `with_failure` panics via `todo!()` — the chain panics before `build()`.
#[tokio::test]
async fn test_BC_3_5_001_fluent_chain_compiles_and_builds() {
    // All 5 builder methods in a single unbroken chain.
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("acme-corp")
        .with_customer_overrides("acme-corp", |c| {
            c.dtu_types = vec![DtuType::Claroty];
            c.seed_override = Some(42);
        })
        .with_customer("globex")
        .with_failure("globex", DtuType::Claroty, FailureMode::AuthReject)
        .build()
        .await
        .expect("5-step fluent chain must build successfully (AC-007)");

    // Sanity: both orgs have endpoints.
    // acme-corp: 1 endpoint (Claroty only, per override).
    // globex: 4 endpoints (default dtu_types from with_customer, no override narrows them).
    let endpoints = harness.endpoints();
    assert!(
        endpoints.len() >= 2,
        "fluent-chain harness must have at least 2 endpoints (acme claroty + globex × 4); \
         got {} (AC-007)",
        endpoints.len()
    );
}

/// AC-007 (minimal chain): The simplest valid 2-method chain compiles and runs.
///
/// `Harness::builder().with_customer("x").build().await`
///
/// Guards against regression in the existing `with_customer` method.
///
/// (AC-007)
///
/// NOTE: Expected to PASS at the Red Gate — `with_customer` and `build()` already
/// work in the stub. Locks the baseline ergonomics contract.
#[tokio::test]
async fn test_BC_3_5_001_minimal_two_method_chain_builds() {
    let harness = prism_dtu_harness::Harness::builder()
        .with_customer("solo")
        .build()
        .await
        .expect("minimal 2-method chain must build successfully (AC-007)");

    assert!(
        !harness.endpoints().is_empty(),
        "minimal chain harness must have at least one endpoint (AC-007)"
    );
}

// ============================================================================
// EC-002 — with_failure with FailureMode::None is a no-op
//
// BC-3.6.001 EC-002; BC-3.6.001 Invariant 4
//
// RED: `with_failure` panics via `todo!()`.
// ============================================================================

/// EC-002: Calling `.with_failure(slug, dtu_type, FailureMode::None)` must be a no-op:
/// the clone must return HTTP 200 on the first request (no failure injected).
///
/// `FailureMode::None` must not result in any injected failure on the built clone.
/// Consistent with `clear_failure` semantics per BC-3.6.001 Invariant 4.
///
/// (BC-3.6.001 EC-002; Invariant 4; S-3.3.05 Task 3)
///
/// RED: `with_failure` panics via `todo!()`.
#[tokio::test]
async fn test_BC_3_6_001_with_failure_none_mode_is_noop() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("delta")
        .with_failure("delta", DtuType::Claroty, FailureMode::None)
        .build()
        .await
        .expect("harness with FailureMode::None must build (EC-002)");

    // FailureMode::None must not inject any failure — clone returns HTTP 200.
    let addr = harness
        .endpoint_for("delta", DtuType::Claroty)
        .expect("delta/Claroty endpoint must exist");

    let status = reqwest::get(format!("http://{}/assets/v1/assets", addr))
        .await
        .expect("HTTP GET must not fail at network level")
        .status()
        .as_u16();

    assert_eq!(
        status, 200,
        "delta/Claroty must return HTTP 200 after with_failure(None) — \
         FailureMode::None is a no-op (BC-3.6.001 EC-002; Invariant 4)"
    );
}

/// EC-002 (clears prior injection): `with_failure(slug, dtu, AuthReject)` followed by
/// `with_failure(slug, dtu, FailureMode::None)` must result in HTTP 200 on first request.
///
/// (BC-3.6.001 EC-002; Invariant 4)
///
/// RED: `with_failure` panics via `todo!()`.
#[tokio::test]
async fn test_BC_3_6_001_with_failure_none_clears_previously_set_failure() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("epsilon")
        // Set a real failure first.
        .with_failure("epsilon", DtuType::Armis, FailureMode::MalformedResponse)
        // Then clear it with None.
        .with_failure("epsilon", DtuType::Armis, FailureMode::None)
        .build()
        .await
        .expect("harness must build after with_failure(None) clears prior failure (EC-002)");

    let addr = harness
        .endpoint_for("epsilon", DtuType::Armis)
        .expect("epsilon/Armis endpoint must exist");

    let body = reqwest::get(format!("http://{}/api/v1/devices", addr))
        .await
        .expect("HTTP GET must not fail")
        .text()
        .await
        .expect("response body as text");

    let parse_result: Result<serde_json::Value, _> = serde_json::from_str(&body);
    assert!(
        parse_result.is_ok(),
        "epsilon/Armis must return valid JSON after with_failure(None) clears prior \
         MalformedResponse injection (EC-002)"
    );
}

// ============================================================================
// AC-008 — Rustdoc example chain compiles and exercises the full API
//
// S-3.3.05 Task 5
//
// RED: `with_failure` panics via `todo!()`.
// ============================================================================

/// AC-008 (integration proxy for doc test): A code snippet matching the shape of
/// the `with_failure` doc-test compiles and, after implementation, runs to completion.
///
/// The pattern: builder chain → `with_customer` → `with_customer_overrides` →
/// `with_failure` → `build().await`.
///
/// This mirrors the doc-example that will appear in builder.rs rustdoc.
///
/// (S-3.3.05 Task 5; AC-008)
///
/// RED: `with_failure` panics via `todo!()`.
#[tokio::test]
async fn test_BC_3_5_001_doc_example_chain_compiles_and_runs() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("acme-corp")
        .with_customer_overrides("acme-corp", |c| {
            c.dtu_types = vec![DtuType::Claroty];
        })
        .with_customer("globex")
        .with_failure("globex", DtuType::Claroty, FailureMode::AuthReject)
        .build()
        .await
        .expect("doc-example chain must build successfully (AC-008)");

    // Verify the doc-example harness is functional.
    assert!(
        harness
            .endpoint_for("acme-corp", DtuType::Claroty)
            .is_some(),
        "doc-example harness must have acme-corp/Claroty endpoint (AC-008)"
    );
    assert!(
        harness.endpoint_for("globex", DtuType::Claroty).is_some(),
        "doc-example harness must have globex/Claroty endpoint (AC-008)"
    );
}

// ============================================================================
// Additional BC traceability tests
// ============================================================================

/// BC-3.6.001 postcondition 1 + VP-128: pre-injected failure affects only the
/// target (org, dtu) clone; all other clones return HTTP 200.
///
/// Exercises `with_failure` in a 3-org harness where only one org/dtu has an
/// injected failure. Other orgs' clones must be unaffected (BC-3.6.001 Invariant 1).
///
/// (BC-3.6.001 postcondition 1 + Invariant 1; VP-128; AC-004)
///
/// RED: `with_failure` panics via `todo!()`.
#[tokio::test]
async fn test_BC_3_6_001_pre_injected_failure_does_not_bleed_to_other_orgs() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("org-a")
        .with_failure("org-a", DtuType::Claroty, FailureMode::AuthReject)
        .with_customer_overrides("org-b", |c| {
            c.dtu_types = vec![DtuType::Claroty];
        })
        .with_customer_overrides("org-c", |c| {
            c.dtu_types = vec![DtuType::Claroty];
        })
        .build()
        .await
        .expect("3-org partial-failure harness must build (BC-3.6.001 postcondition 1)");

    // org-a/Claroty must return 401 (pre-injected).
    let org_a_addr = harness
        .endpoint_for("org-a", DtuType::Claroty)
        .expect("org-a/Claroty must exist");
    let org_a_status = reqwest::get(format!("http://{}/assets/v1/assets", org_a_addr))
        .await
        .expect("GET org-a")
        .status()
        .as_u16();
    assert_eq!(
        org_a_status, 401,
        "org-a/Claroty must return 401 — pre-injected AuthReject (BC-3.6.001 pc-1)"
    );

    // org-b and org-c must return 200 — failure must be scoped to org-a only.
    let org_b_addr = harness
        .endpoint_for("org-b", DtuType::Claroty)
        .expect("org-b/Claroty must exist");
    let org_b_status = reqwest::get(format!("http://{}/assets/v1/assets", org_b_addr))
        .await
        .expect("GET org-b")
        .status()
        .as_u16();
    assert_eq!(
        org_b_status, 200,
        "org-b/Claroty must return 200 — unaffected by org-a failure injection (VP-128)"
    );

    let org_c_addr = harness
        .endpoint_for("org-c", DtuType::Claroty)
        .expect("org-c/Claroty must exist");
    let org_c_status = reqwest::get(format!("http://{}/assets/v1/assets", org_c_addr))
        .await
        .expect("GET org-c")
        .status()
        .as_u16();
    assert_eq!(
        org_c_status, 200,
        "org-c/Claroty must return 200 — unaffected by org-a failure injection (VP-128)"
    );
}

/// BC-3.5.001 precondition 2 (guard): `with_customer_overrides` called without a
/// prior `with_customer` for a new slug still registers the customer (the closure
/// applies to a freshly created spec). This is the existing stub behavior and must
/// continue to work after the "mutate existing" code path is added.
///
/// (BC-3.5.001 precondition 2; S-3.3.05 Task 2 — backward compatibility)
///
/// NOTE: Expected to PASS at the Red Gate — this is the stub's existing behavior.
#[tokio::test]
async fn test_BC_3_5_001_with_customer_overrides_without_prior_with_customer_creates_new_spec() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("zeta", |c| {
            c.dtu_types = vec![DtuType::CrowdStrike];
            c.seed = 77;
        })
        .build()
        .await
        .expect("with_customer_overrides alone must build a harness");

    // One org × one DTU type = 1 endpoint.
    assert_eq!(
        harness.endpoints().len(),
        1,
        "with_customer_overrides without prior with_customer must register 1 spec → 1 endpoint"
    );
    assert!(
        harness.endpoint_for("zeta", DtuType::CrowdStrike).is_some(),
        "zeta/CrowdStrike endpoint must be accessible after with_customer_overrides"
    );
}
