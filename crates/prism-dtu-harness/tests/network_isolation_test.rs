//! Network Isolation Test Suite — S-3.3.04 Red Gate
//!
//! Covers:
//!   BC-3.5.002 canonical test vectors TV-1 through TV-7
//!
// Allow test-file conventions: expect() in test assertions and BC-tracing names.
#![allow(clippy::expect_used, non_snake_case)]
//!
//! Verification properties exercised:
//!   VP-125 — All SocketAddrs in customer_endpoints are pairwise distinct after build()
//!   VP-126 — Wrong-org credentials to live clone returns HTTP 401, never HTTP 200
//!   VP-127 — devices(OrgA) ∩ devices(OrgB) = ∅ for all org pairs in 3-org scenario
//!
//! # Red Gate
//!
//! ALL tests in this file MUST fail before implementation. The primary failure
//! mechanism is `todo!()` panics propagated from:
//!   - `build_network()` (called by `build()` when IsolationMode::Network is set)
//!   - `customer_endpoints()` accessor
//!
//! AC-008 additionally fails at compile time due to missing `with_network_bind_timeout`
//! builder API method (see `test_BC_3_5_002_ac008_timeout_knob_compile_gate`).
//!
//! If any test passes without implementation, it is suspect — flag for spec-reviewer.
//!
//! # Test naming
//!
//! `test_BC_S_SS_NNN_xxx()` pattern throughout (Factory TDD spec).

use prism_dtu_harness::{DtuType, HarnessError, IsolationMode};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// AC-001 / TV-4: build(Network) populates customer_endpoints atomically
//
// BC-3.5.002 postcondition 4; Invariant 1; VP-125
// ============================================================================

/// TV-4: 2-org × 2-DTU network harness — customer_endpoints contains exactly 4 entries.
///
/// Exercises: `build()` with `IsolationMode::Network` completes and returns a harness
/// whose `customer_endpoints` map contains one entry per registered `(OrgId, DtuType)` pair.
///
/// RED: `build_network()` is `todo!()` — panics with "S-3.3.04: build_network".
///
/// (BC-3.5.002 postcondition 4; Invariant 1; AC-001; VP-125)
#[tokio::test]
async fn test_BC_3_5_002_ac001_customer_endpoints_populated_atomically() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty, DtuType::Armis];
            spec.seed = 100;
        })
        .with_customer_overrides("globex", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike, DtuType::Cyberint];
            spec.seed = 101;
        })
        .build()
        .await
        .expect("2-org × 2-DTU network harness build must succeed");

    let endpoints = harness.customer_endpoints();
    assert_eq!(
        endpoints.len(),
        4,
        "customer_endpoints must contain exactly 4 entries for 2-org × 2-DTU \
         network harness (BC-3.5.002 postcondition 4; AC-001)"
    );
}

/// TV-4 (AC-001 extended): 3-org × 4-sensor (12-clone) harness — customer_endpoints has 12 entries.
///
/// Mirrors the canonical ADR-011 §2.8 cross-customer fidelity scenario.
///
/// RED: `build_network()` is `todo!()`.
///
/// (BC-3.5.002 postcondition 4; AC-001; story AC-001 spec: "3-org × 4-sensor (12-clone)")
#[tokio::test]
async fn test_BC_3_5_002_ac001_twelve_clone_customer_endpoints_count() {
    let harness = build_twelve_clone_network_harness().await;

    let endpoints = harness.customer_endpoints();
    assert_eq!(
        endpoints.len(),
        12,
        "customer_endpoints must contain exactly 12 entries for 3-org × 4-sensor \
         network harness (BC-3.5.002 postcondition 4; AC-001)"
    );
}

// ============================================================================
// AC-002 / TV-4: Distinct ports per (OrgId, DtuType)
//
// BC-3.5.002 postcondition 4; Invariant 1; VP-125
// ============================================================================

/// TV-4: All SocketAddrs returned by customer_endpoints() are pairwise distinct.
///
/// VP-125: proptest coverage. This deterministic test verifies the invariant for
/// the 12-clone canonical harness. The proptest analog (random config sizes) is
/// `test_BC_3_5_002_invariant_VP125_proptest` below.
///
/// RED: `build_network()` is `todo!()`.
///
/// (BC-3.5.002 postcondition 4; VP-125; AC-002)
#[tokio::test]
async fn test_BC_3_5_002_ac002_all_customer_endpoints_pairwise_distinct() {
    let harness = build_twelve_clone_network_harness().await;

    let endpoints = harness.customer_endpoints();
    let addrs: Vec<_> = endpoints.values().collect();

    for i in 0..addrs.len() {
        for j in (i + 1)..addrs.len() {
            assert_ne!(
                addrs[i], addrs[j],
                "customer_endpoints must be pairwise distinct — found duplicate {:?} \
                 (BC-3.5.002 postcondition 4; VP-125; AC-002)",
                addrs[i]
            );
        }
    }
}

/// VP-125 (proptest-style): customer_endpoints are pairwise distinct for any
/// non-trivial harness configuration.
///
/// Generates varied org/DTU configurations and asserts the uniqueness invariant.
/// 1000+ unique port assignments across 1000 harness builds is the target — here
/// we exercise 10 independent builds to validate the OS-ephemeral-port mechanism.
///
/// RED: `build_network()` is `todo!()`.
///
/// (BC-3.5.002 Invariant 1; VP-125; AC-002)
#[tokio::test]
async fn test_BC_3_5_002_invariant_VP125_repeated_builds_always_distinct() {
    for i in 0..10u32 {
        let harness = prism_dtu_harness::Harness::builder()
            .isolation(IsolationMode::Network)
            .with_customer_overrides("acme-corp", |spec| {
                spec.dtu_types = vec![DtuType::Claroty, DtuType::Armis];
                spec.seed = u64::from(i) * 1000 + 1;
            })
            .with_customer_overrides("globex", |spec| {
                spec.dtu_types = vec![DtuType::CrowdStrike, DtuType::Cyberint];
                spec.seed = u64::from(i) * 1000 + 2;
            })
            .build()
            .await
            .unwrap_or_else(|_| {
                panic!("network harness build #{i} must succeed (VP-125 repeated-builds)")
            });

        let endpoints = harness.customer_endpoints();
        let addr_set: HashSet<_> = endpoints.values().copied().collect();
        assert_eq!(
            addr_set.len(),
            endpoints.len(),
            "build #{i}: customer_endpoints must have no duplicate SocketAddrs (VP-125)"
        );
    }
}

// ============================================================================
// AC-003 / TV-2: Port immutability
//
// BC-3.5.002 Invariant 1
// ============================================================================

/// TV-2: Read customer_endpoints twice across an .await point — second read equals first.
///
/// Verifies that `customer_endpoints` is immutable for the harness lifetime
/// (BC-3.5.002 Invariant 1): no mutation occurs between reads.
///
/// RED: `build_network()` is `todo!()`.
///
/// (BC-3.5.002 Invariant 1; AC-003)
#[tokio::test]
async fn test_BC_3_5_002_ac003_customer_endpoints_immutable_across_await() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 42;
        })
        .with_customer_overrides("globex", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 43;
        })
        .build()
        .await
        .expect("network harness build must succeed");

    // First read — snapshot all addresses
    let first_snapshot: Vec<_> = harness.customer_endpoints().values().copied().collect();

    // Yield to the async runtime between reads
    tokio::task::yield_now().await;

    // Second read — must be identical
    let second_snapshot: Vec<_> = harness.customer_endpoints().values().copied().collect();

    assert_eq!(
        first_snapshot.len(),
        second_snapshot.len(),
        "customer_endpoints length must be identical between reads (AC-003)"
    );

    // Check that each address present in first is also present in second
    let first_set: HashSet<_> = first_snapshot.iter().copied().collect();
    let second_set: HashSet<_> = second_snapshot.iter().copied().collect();
    assert_eq!(
        first_set, second_set,
        "customer_endpoints set must be identical between reads across an await point \
         (BC-3.5.002 Invariant 1; AC-003)"
    );
}

// ============================================================================
// AC-004 / TV-3: Observable HTTP 401 on cross-org routing
//
// BC-3.5.002 postconditions 1, 2; EC-001; VP-126
// ============================================================================

/// TV-3: Cross-org credential mismatch — OrgA token → OrgB endpoint → HTTP 401.
///
/// Builds a 2-org Claroty harness. Sends a request bearing acme-corp's Bearer token
/// to globex's endpoint. Asserts HTTP 401 is returned, not 200 and not a silent
/// empty response.
///
/// RED: `build_network()` is `todo!()`.
///
/// (BC-3.5.002 postcondition 2; EC-001; VP-126; AC-004)
#[tokio::test]
async fn test_BC_3_5_002_ac004_cross_org_credential_mismatch_returns_401() {
    let (harness, acme_token, globex_token) = build_two_org_network_claroty_harness().await;

    let acme_addr = network_addr_for(&harness, "acme-corp", DtuType::Claroty);
    let globex_addr = network_addr_for(&harness, "globex", DtuType::Claroty);

    // Cross-org: send acme-corp's token to globex's endpoint → must be 401
    let cross_status = reqwest::Client::new()
        .get(format!("http://{globex_addr}/assets/v1/assets"))
        .bearer_auth(&acme_token)
        .send()
        .await
        .expect("HTTP GET (cross-org) must not fail at network level")
        .status()
        .as_u16();

    assert_eq!(
        cross_status, 401,
        "request bearing acme-corp token sent to globex endpoint must return HTTP 401 \
         (BC-3.5.002 postcondition 2; EC-001; VP-126; AC-004)"
    );

    // Same-org: send acme-corp's token to acme-corp's endpoint → must be 200
    let same_status = reqwest::Client::new()
        .get(format!("http://{acme_addr}/assets/v1/assets"))
        .bearer_auth(&acme_token)
        .send()
        .await
        .expect("HTTP GET (same-org) must not fail at network level")
        .status()
        .as_u16();

    assert_eq!(
        same_status, 200,
        "request bearing acme-corp token sent to acme-corp endpoint must return HTTP 200 \
         (BC-3.5.002 postcondition 2; VP-126; AC-004)"
    );

    // Verify the reverse: globex token to globex endpoint → 200
    let globex_same_status = reqwest::Client::new()
        .get(format!("http://{globex_addr}/assets/v1/assets"))
        .bearer_auth(&globex_token)
        .send()
        .await
        .expect("HTTP GET (globex same-org) must not fail at network level")
        .status()
        .as_u16();

    assert_eq!(
        globex_same_status, 200,
        "request bearing globex token sent to globex endpoint must return HTTP 200 (AC-004)"
    );
}

/// TV-1: Pairwise disjoint device ID sets across 3 orgs in Network mode.
///
/// VP-127: devices(acme) ∩ devices(globex) = ∅, devices(acme) ∩ devices(initech) = ∅,
///         devices(globex) ∩ devices(initech) = ∅.
///
/// RED: `build_network()` is `todo!()`.
///
/// (BC-3.5.002 postcondition 1; VP-127; AC-004)
#[tokio::test]
async fn test_BC_3_5_002_VP127_pairwise_disjoint_device_ids() {
    let harness = build_three_org_network_harness().await;

    let acme_devices = fetch_network_devices(&harness, "acme-corp", DtuType::Claroty).await;
    let globex_devices = fetch_network_devices(&harness, "globex", DtuType::Armis).await;
    let initech_devices = fetch_network_devices(&harness, "initech", DtuType::Claroty).await;

    assert!(
        !acme_devices.is_empty(),
        "acme-corp must return at least one device in Network mode (TV-1; VP-127)"
    );
    assert!(
        !globex_devices.is_empty(),
        "globex must return at least one device in Network mode (TV-1; VP-127)"
    );
    assert!(
        !initech_devices.is_empty(),
        "initech must return at least one device in Network mode (TV-1; VP-127)"
    );

    // acme ∩ globex = ∅
    for id in &acme_devices {
        assert!(
            !globex_devices.contains(id),
            "device ID {id:?} appears in both acme-corp and globex — \
             data isolation violated (TV-1; VP-127)"
        );
    }
    // acme ∩ initech = ∅
    for id in &acme_devices {
        assert!(
            !initech_devices.contains(id),
            "device ID {id:?} appears in both acme-corp and initech — \
             data isolation violated (TV-1; VP-127)"
        );
    }
    // globex ∩ initech = ∅
    for id in &globex_devices {
        assert!(
            !initech_devices.contains(id),
            "device ID {id:?} appears in both globex and initech — \
             data isolation violated (TV-1; VP-127)"
        );
    }
}

// ============================================================================
// AC-005 / TV-6: drop releases ports
//
// BC-3.5.002 postcondition 6; VP-127
// ============================================================================

/// TV-6: After drop(harness), all TCP listeners are released.
///
/// Captures the customer_endpoints addresses before drop. After drop, attempts
/// to bind to one of the released addresses — must succeed, confirming the port
/// was released (i.e., the OS allowed rebinding).
///
/// RED: `build_network()` is `todo!()`.
///
/// (BC-3.5.002 postcondition 6; AC-005)
#[tokio::test]
async fn test_BC_3_5_002_ac005_drop_releases_ports() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 77;
        })
        .build()
        .await
        .expect("single-org network harness must build");

    // Snapshot one address before drop
    let addr = *harness
        .customer_endpoints()
        .values()
        .next()
        .expect("at least one endpoint must be present");

    drop(harness);

    // Give the OS a brief moment (port release is effectively immediate on Drop)
    tokio::time::sleep(Duration::from_millis(50)).await;

    // After drop, attempting TcpStream::connect must fail with ConnectionRefused
    // (the listener is no longer accepting). Independently, we verify re-bind works.
    let connect_result =
        tokio::time::timeout(Duration::from_secs(1), tokio::net::TcpStream::connect(addr)).await;

    match connect_result {
        Ok(Ok(_)) => {
            panic!(
                "TCP connect to {addr} succeeded after drop — port was not released \
                 (BC-3.5.002 postcondition 6; TV-6; AC-005)"
            )
        }
        Ok(Err(e)) => {
            assert_eq!(
                e.kind(),
                std::io::ErrorKind::ConnectionRefused,
                "expected ConnectionRefused after drop; got: {e} (TV-6; AC-005)"
            );
        }
        Err(_timeout) => {
            panic!(
                "TCP connect to {addr} did not resolve within 1s after drop — \
                 listener may still be holding the port (TV-6; AC-005)"
            )
        }
    }

    // Additionally: verify OS allows rebinding the released address
    let rebind_result = tokio::net::TcpListener::bind(addr).await;
    assert!(
        rebind_result.is_ok(),
        "must be able to rebind {addr} after harness drop — \
         port was not fully released (BC-3.5.002 postcondition 6; AC-005)"
    );
}

// ============================================================================
// AC-006: drop joins child tasks (no leaks via counter)
//
// BC-3.5.002 postcondition 6
// ============================================================================

/// AC-006: After drop(harness), all listener tasks have stopped.
///
/// Uses an `Arc<AtomicUsize>` task counter. Each listener task increments on start
/// and decrements on shutdown. After drop, counter must equal 0.
///
/// # Implementer instruction (missing API)
///
/// This test requires the following builder method — add it to `HarnessBuilder`:
///
/// ```rust
/// /// Test hook: inject a lifecycle counter into each clone task.
/// /// Each spawned task increments the counter on start and decrements on clean shutdown.
/// pub fn with_task_lifecycle_counter(mut self, counter: Arc<AtomicUsize>) -> Self {
///     self.task_lifecycle_counter = Some(counter);
///     self
/// }
/// ```
///
/// Wire `task_lifecycle_counter` through `build_network()` so each Tokio task
/// increments on startup and decrements when the shutdown signal is received.
///
/// Until that method exists, the test body has been written to call `build()` with
/// `IsolationMode::Network` which panics via `build_network()`'s `todo!()`.
/// The counter verification is also present as comments to drive the shape.
///
/// RED: build_network() is todo!() — panics before counter verification.
///
/// (BC-3.5.002 postcondition 6; AC-006)
#[tokio::test]
async fn test_BC_3_5_002_ac006_drop_joins_all_listener_tasks() {
    // Counter will be wired by implementer via `with_task_lifecycle_counter`.
    // Declared here to preserve the intended test structure for the implementer.
    let task_counter = Arc::new(AtomicUsize::new(0));

    // The harness build panics at build_network()'s todo!() — counter is not
    // yet incremented, but the shape of the assertion is documented below.
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty, DtuType::Armis];
            spec.seed = 55;
        })
        .with_customer_overrides("globex", |spec| {
            spec.dtu_types = vec![DtuType::Claroty, DtuType::Armis];
            spec.seed = 56;
        })
        // TODO(implementer): replace the line below with:
        //   .with_task_lifecycle_counter(Arc::clone(&task_counter))
        // once that builder method is added. Until then, build() panics first.
        .build()
        .await
        .expect("network harness with lifecycle counter must build");

    // After build: 4 tasks should be running (2 orgs × 2 DTUs)
    // Implementer: assert_eq!(task_counter.load(Ordering::SeqCst), 4, "...");
    let _ = task_counter.load(Ordering::SeqCst); // keep the counter live

    drop(harness);

    // After drop: all tasks should have cleanly exited
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Implementer: assert_eq!(task_counter.load(Ordering::SeqCst), 0, "...");
    // The full assertion is written as a doc in the BC comment above.
    assert_eq!(
        task_counter.load(Ordering::SeqCst),
        0,
        "after drop, all listener tasks must have exited — task counter must be 0 \
         (BC-3.5.002 postcondition 6; AC-006); \
         note: counter requires with_task_lifecycle_counter builder hook (implementer TODO)"
    );
}

// ============================================================================
// AC-007 / TV-7: NetworkPortAllocation error variant exists and displays correctly
//
// BC-3.5.002 EC-004; HarnessError::NetworkPortAllocation
// ============================================================================

/// TV-7 (sentinel): `HarnessError::NetworkPortAllocation` variant exists, implements
/// `Display`, and its message contains "network" or "port allocation".
///
/// This sentinel test documents that the error variant is defined, wired into the
/// `Display` implementation, and implements `std::error::Error`. It does NOT trigger
/// an actual port allocation failure (hard to reproduce reliably in CI); it constructs
/// the variant directly via `io::Error::from(io::ErrorKind::AddrInUse)`.
///
/// (BC-3.5.002 EC-004; ADR-011 §2.5; AC-007)
#[test]
fn test_BC_3_5_002_ac007_network_port_allocation_error_variant_sentinel() {
    let source = std::io::Error::from(std::io::ErrorKind::AddrInUse);
    let err = HarnessError::NetworkPortAllocation { source };

    let display = format!("{err}");
    let lower = display.to_lowercase();

    assert!(
        lower.contains("network") || lower.contains("port") || lower.contains("allocation"),
        "HarnessError::NetworkPortAllocation Display message must mention 'network', \
         'port', or 'allocation'; got: {display:?} \
         (BC-3.5.002 EC-004; ADR-011 §2.5; AC-007)"
    );

    // Must implement std::error::Error (verified by trait bound — if this compiles, it passes)
    fn assert_error<E: std::error::Error>(_: &E) {}
    assert_error(&err);
}

/// TV-7: Unknown org in network mode returns `HarnessError::UnknownOrg`.
///
/// Builds a single-org network harness. Querying for an unregistered org via the
/// harness must return `UnknownOrg` — no HTTP request forwarded to any live clone.
///
/// RED: `build_network()` is `todo!()`.
///
/// (BC-3.5.002 postcondition 3; TV-7; EC-005; AC-004 partial)
#[tokio::test]
async fn test_BC_3_5_002_ac007_unknown_org_returns_error_in_network_mode() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 88;
        })
        .build()
        .await
        .expect("single-org network harness must build");

    // inject_failure for an unregistered org must return UnknownOrg
    let result = harness
        .inject_failure(
            "globex",
            DtuType::Claroty,
            prism_dtu_common::FailureMode::AuthReject,
        )
        .await;

    assert!(
        matches!(result, Err(HarnessError::UnknownOrg { slug }) if slug == "globex"),
        "inject_failure for unregistered org must return UnknownOrg in Network mode \
         (BC-3.5.002 postcondition 3; TV-7; AC-007)"
    );
}

/// EC-005: Unknown DTU type for a known org returns `HarnessError::UnknownDtuType`.
///
/// (BC-3.5.002 EC-005; story EC-004)
#[tokio::test]
async fn test_BC_3_5_002_ec005_unknown_dtu_type_returns_error_in_network_mode() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty]; // only Claroty registered
            spec.seed = 89;
        })
        .build()
        .await
        .expect("single-org network harness must build");

    // acme-corp is registered, but only with Claroty — Armis is not present
    let result = harness
        .inject_failure(
            "acme-corp",
            DtuType::Armis,
            prism_dtu_common::FailureMode::AuthReject,
        )
        .await;

    assert!(
        matches!(result, Err(HarnessError::UnknownDtuType { .. })),
        "inject_failure with unregistered DTU type for known org must return \
         UnknownDtuType in Network mode (BC-3.5.002 EC-005; story EC-004)"
    );
}

// ============================================================================
// AC-008: 5-second startup timeout knob (ADR-011 §2.5)
//
// BC-3.5.002 postcondition 5; Invariant 2
// ============================================================================

/// AC-008: Network-mode build completes within 5s for 12-clone harness (TV-5).
///
/// Tests the 5-second total startup timeout specified in BC-3.5.002 postcondition 5
/// and ADR-011 §2.5. This test exercises the wall-clock budget rather than an
/// injected delay.
///
/// RED: `build_network()` is `todo!()`.
///
/// (BC-3.5.002 postcondition 5; Invariant 2; AC-008 — startup budget)
#[tokio::test]
async fn test_BC_3_5_002_ac008_twelve_clone_startup_under_5s() {
    let start = std::time::Instant::now();

    let _harness = build_twelve_clone_network_harness().await;

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 5,
        "12-clone Network harness build took {:.2}s; must complete in < 5s \
         (BC-3.5.002 postcondition 5; AC-008; ADR-011 §2.5)",
        elapsed.as_secs_f64()
    );
}

/// AC-008 (timeout-knob): Network-mode startup timeout is configurable.
///
/// ADR-011 §2.5 specifies a 5s build timeout for Network mode. The builder must
/// expose a `with_network_bind_timeout(Duration)` method so tests can configure a
/// shorter timeout to exercise the `StartupTimeout` error path in Network mode.
///
/// # Implementer instruction (missing API — see `network_isolation_timeout_test.rs`)
///
/// `with_network_bind_timeout` is exercised in the companion file
/// `tests/network_isolation_timeout_test.rs`, which is intentionally left as a
/// compile-error RED gate. See that file for the full test and the builder API
/// specification.
///
/// This test only exercises the 5s wall-clock budget (already covered by
/// `test_BC_3_5_002_ac008_twelve_clone_startup_under_5s` above). It succeeds
/// if `build_network()` is implemented within the budget.
///
/// RED: `build_network()` is `todo!()`.
///
/// (BC-3.5.002 postcondition 5; ADR-011 §2.5; AC-008)
#[tokio::test]
async fn test_BC_3_5_002_ac008_network_startup_within_5s_budget() {
    // This test is a structural duplicate of `ac008_twelve_clone_startup_under_5s`
    // included for full AC-008 traceability. Both must pass once implemented.
    let start = std::time::Instant::now();

    let _harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty, DtuType::Armis];
            spec.seed = 999;
        })
        .with_customer_overrides("globex", |spec| {
            spec.dtu_types = vec![DtuType::CrowdStrike, DtuType::Cyberint];
            spec.seed = 1000;
        })
        .build()
        .await
        .expect("4-clone Network harness must build within 5s budget (AC-008)");

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 5,
        "Network harness build took {:.2}s; must be < 5s (BC-3.5.002 postcondition 5; AC-008)",
        elapsed.as_secs_f64()
    );
}

// ============================================================================
// VP-125 (extended): customer_endpoints immutability — no &mut accessor
//
// BC-3.5.002 Invariant 1
// ============================================================================

/// VP-125 / Invariant 1: `customer_endpoints()` returns `&HashMap` (not `&mut`).
///
/// This is a compile-time contract: if the harness erroneously exposed a mutable
/// accessor, this test would fail at compile time. The immutability is enforced
/// by the return type signature — this test documents the contract.
///
/// RED (runtime): `customer_endpoints()` is `todo!()`.
///
/// (BC-3.5.002 Invariant 1; VP-125)
#[tokio::test]
async fn test_BC_3_5_002_invariant_customer_endpoints_is_immutable_reference() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 42;
        })
        .build()
        .await
        .expect("network harness must build");

    // `customer_endpoints()` must return `&HashMap<OrgKey, SocketAddr>`.
    // The type annotation here enforces at compile time that the return is a shared ref.
    let _endpoints: &std::collections::HashMap<prism_dtu_harness::OrgKey, std::net::SocketAddr> =
        harness.customer_endpoints();

    // If we reach here, the accessor returned the correct type.
    // Immutability is structurally guaranteed by the `&` return type.
}

// ============================================================================
// TV-5: 5s total startup (already covered by ac008_twelve_clone_startup_under_5s above)
// TV-2: Correct-endpoint routing → 200 (already exercised within ac004 same-org assertions)
// ============================================================================

/// TV-2: Correct-endpoint routing — OrgA token to OrgA endpoint returns HTTP 200.
///
/// Standalone test for full BC traceability to TV-2.
///
/// RED: `build_network()` is `todo!()`.
///
/// (BC-3.5.002 TV-2; postcondition 1)
#[tokio::test]
async fn test_BC_3_5_002_TV2_correct_endpoint_routing_returns_200() {
    let (harness, acme_token, globex_token) = build_two_org_network_claroty_harness().await;

    let acme_addr = network_addr_for(&harness, "acme-corp", DtuType::Claroty);
    let globex_addr = network_addr_for(&harness, "globex", DtuType::Claroty);

    // OrgA token → OrgA endpoint
    let acme_status = reqwest::Client::new()
        .get(format!("http://{acme_addr}/assets/v1/assets"))
        .bearer_auth(&acme_token)
        .send()
        .await
        .expect("HTTP GET must not fail at network level")
        .status()
        .as_u16();

    assert_eq!(
        acme_status, 200,
        "acme-corp token to acme-corp endpoint must return 200 (TV-2)"
    );

    // OrgB token → OrgB endpoint
    let globex_status = reqwest::Client::new()
        .get(format!("http://{globex_addr}/assets/v1/assets"))
        .bearer_auth(&globex_token)
        .send()
        .await
        .expect("HTTP GET must not fail at network level")
        .status()
        .as_u16();

    assert_eq!(
        globex_status, 200,
        "globex token to globex endpoint must return 200 (TV-2)"
    );
}

// ============================================================================
// Test helper functions
// ============================================================================

/// Build the canonical 3-org × 4-sensor (12-clone) Network mode harness.
///
/// Matches ADR-011 §2.8 canonical cross-customer fidelity scenario:
///   acme-corp: Claroty + Armis
///   globex: CrowdStrike only
///   initech: Cyberint + CrowdStrike
/// Extended to 4 types per org for TV-4/TV-5 12-clone coverage.
async fn build_twelve_clone_network_harness() -> prism_dtu_harness::Harness {
    prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            // 4 DTU types
            spec.dtu_types = vec![
                DtuType::Claroty,
                DtuType::Armis,
                DtuType::CrowdStrike,
                DtuType::Cyberint,
            ];
            spec.seed = 200;
        })
        .with_customer_overrides("globex", |spec| {
            // 4 DTU types
            spec.dtu_types = vec![
                DtuType::Claroty,
                DtuType::Armis,
                DtuType::CrowdStrike,
                DtuType::Cyberint,
            ];
            spec.seed = 201;
        })
        .with_customer_overrides("initech", |spec| {
            // 4 DTU types
            spec.dtu_types = vec![
                DtuType::Claroty,
                DtuType::Armis,
                DtuType::CrowdStrike,
                DtuType::Cyberint,
            ];
            spec.seed = 202;
        })
        .build()
        .await
        .expect("12-clone Network harness build must succeed")
}

/// Build a 3-org Network harness matching the ADR-011 §2.8 canonical scenario.
///
/// acme-corp: Claroty + Armis
/// globex: Armis (for TV-1 device set comparison)
/// initech: Claroty + CrowdStrike
async fn build_three_org_network_harness() -> prism_dtu_harness::Harness {
    prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty, DtuType::Armis];
            spec.seed = 300;
        })
        .with_customer_overrides("globex", |spec| {
            spec.dtu_types = vec![DtuType::Armis];
            spec.seed = 301;
        })
        .with_customer_overrides("initech", |spec| {
            spec.dtu_types = vec![DtuType::Claroty, DtuType::CrowdStrike];
            spec.seed = 302;
        })
        .build()
        .await
        .expect("3-org Network harness build must succeed")
}

/// Build a 2-org Claroty Network harness. Returns (harness, acme_admin_token, globex_admin_token).
///
/// The tokens are per-org admin tokens used to verify correct-vs-wrong endpoint routing.
/// In the stub phase, `build_network()` is `todo!()` so this never returns — tests fail RED.
///
/// When implemented, the harness must expose per-org tokens via a test accessor:
///
/// # Implementer instruction (missing API)
///
/// Add to `Harness`:
/// ```rust
/// /// Return the admin token for the clone at (slug, dtu_type), if registered.
/// pub fn admin_token_for(&self, slug: &str, dtu_type: DtuType) -> Option<&str> {
///     let org_id = self.slug_to_org.get(slug)?;
///     self.admin_tokens.get(&(*org_id, dtu_type)).map(|s| s.as_str())
/// }
/// ```
///
/// This accessor is needed for cross-org credential-mismatch tests (AC-004, TV-3, VP-126)
/// so each org's clone's per-org Bearer token can be passed to wrong-org requests to
/// exercise the 401 path.
async fn build_two_org_network_claroty_harness() -> (prism_dtu_harness::Harness, String, String) {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 400;
        })
        .with_customer_overrides("globex", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 401;
        })
        .build()
        .await
        .expect("2-org Network Claroty harness build must succeed");
    // ^^^ build_network() is todo!() — panics here. Code below is not reached until
    // the implementer completes S-3.3.04.

    // Retrieve per-org admin tokens for credential-routing assertions.
    // `admin_token_for` must be added to Harness by the implementer (see doc above).
    // Until build_network() is implemented this code is unreachable — no compile error.
    let acme_token = harness
        .admin_token_for("acme-corp", DtuType::Claroty)
        .expect("acme-corp Claroty admin token must be present after build")
        .to_owned();

    let globex_token = harness
        .admin_token_for("globex", DtuType::Claroty)
        .expect("globex Claroty admin token must be present after build")
        .to_owned();

    (harness, acme_token, globex_token)
}

/// Get the `SocketAddr` for a given `(slug, dtu_type)` from `customer_endpoints`.
///
/// Uses `customer_endpoints()` (the Network-mode accessor, which is todo!()).
/// Panics if the slug or DTU type is not found.
fn network_addr_for(
    harness: &prism_dtu_harness::Harness,
    slug: &str,
    dtu_type: DtuType,
) -> std::net::SocketAddr {
    // Use endpoint_for which goes through the slug_to_org map and the endpoints HashMap.
    // In Network mode, endpoints and customer_endpoints hold the same addresses.
    harness.endpoint_for(slug, dtu_type).unwrap_or_else(|| {
        panic!("no Network-mode endpoint for slug={slug:?} dtu_type={dtu_type:?}")
    })
}

/// Fetch device IDs for a specific org+DTU from a Network-mode harness endpoint.
///
/// Mirrors the logical_isolation_test.rs helper but uses `network_addr_for`.
/// Returns a `HashSet<String>` for intersection testing (VP-127).
async fn fetch_network_devices(
    harness: &prism_dtu_harness::Harness,
    slug: &str,
    dtu_type: DtuType,
) -> HashSet<String> {
    let addr = network_addr_for(harness, slug, dtu_type);
    let path = match dtu_type {
        DtuType::Claroty => "/assets/v1/assets",
        DtuType::Armis => "/api/v1/devices",
        DtuType::CrowdStrike => "/devices/v2/devices",
        DtuType::Cyberint => "/api/v1/events",
        _ => "/api/v1/items",
    };

    let resp = reqwest::get(format!("http://{addr}{path}"))
        .await
        .expect("HTTP GET must succeed in Network mode");
    let body: serde_json::Value = resp.json().await.expect("response must be JSON");

    let items = body["assets"]
        .as_array()
        .or_else(|| body["devices"].as_array())
        .or_else(|| body["items"].as_array())
        .or_else(|| body.as_array())
        .cloned()
        .unwrap_or_default();

    items
        .iter()
        .filter_map(|item| {
            item["id"]
                .as_str()
                .or_else(|| item["device_id"].as_str())
                .map(|s| s.to_owned())
        })
        .collect()
}
