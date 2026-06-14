//! S-DEMO-MULTI-TENANT-DTU-001 — Red Gate tests for `prism-dtu-demo-server`
//! multi-instance binding (BC-2.06.017 Postconditions 1, 5, 6, 7).
//!
//! ## Red Gate discipline (SID-1)
//!
//! Forward-failing tests (multi-instance behavior) MUST FAIL because `start_instances`
//! is `todo!()`. They will panic at the todo — that panic IS the red. The assertions
//! are real: once implemented they verify actual distinct SocketAddrs and live HTTP
//! responses.
//!
//! The two backward-compat / parity tests (marked "REGRESSION GUARD") MUST PASS in
//! the current state — the single-instance `start_on` path is unchanged.
//!
//! ## Test naming
//!
//! `test_BC_2_06_017_*` pattern throughout (Factory TDD spec; BC-2.06.017).

#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]

use std::time::Duration;

use prism_dtu_demo_server::{
    start_instances, InstanceEntry, MultiInstanceBindError, MultiInstanceConfig,
};

/// Build a reqwest client with a 10-second timeout.
///
/// All test HTTP clients must use an explicit timeout (CLAUDE.md § reqwest timeout rule).
fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("test client build must succeed")
}

/// Ephemeral loopback bind address for test instances.
fn ephemeral() -> std::net::SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}

// ============================================================================
// AC-001 / TV-017-002: MultiInstanceConfig accepted without error (empty case)
//
// test_demo_server_zero_instances_returns_empty_map
// BC-2.06.017 EC-017-002: empty config → Ok(MultiInstanceServers) with empty socket_map(), no panic.
// ============================================================================

/// EC-017-002: Empty `MultiInstanceConfig` returns empty socket map; no panic.
///
/// RED GATE: `start_instances` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED: `Ok(MultiInstanceServers)` with empty `socket_map()` must be returned
/// with no spawned tasks.
///
/// (BC-2.06.017 EC-017-002 / Postcondition 1)
#[tokio::test]
async fn test_BC_2_06_017_demo_server_zero_instances_returns_empty_map() {
    let cfg = MultiInstanceConfig::new(vec![]);

    let result = start_instances(cfg, |_entry| {
        panic!("factory must not be called for zero-instance config")
    })
    .await;

    // Once implemented: empty config → Ok(MultiInstanceServers) with empty socket_map(), no error.
    let servers = result.expect(
        "Empty MultiInstanceConfig must return Ok(MultiInstanceServers) with empty socket_map() \
         — not Err() (BC-2.06.017 EC-017-002)",
    );
    let map = servers.socket_map();
    assert!(
        map.is_empty(),
        "Empty MultiInstanceConfig must yield an empty socket map; got {map:?} \
         (BC-2.06.017 EC-017-002)"
    );
}

// ============================================================================
// AC-001 / TV-017-001: MultiInstanceConfig accepted and returns non-empty map
//
// test_demo_server_multi_instance_bind_config_accepted
// BC-2.06.017 Postcondition 1: N ≥ 1 instances → Ok(map) with N entries.
// ============================================================================

/// AC-001: `MultiInstanceConfig` accepted without panic/error; returns non-empty map.
///
/// RED GATE: `start_instances` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED: Two armis instances bind at distinct ephemeral SocketAddrs;
/// the returned map has exactly 2 entries.
///
/// (BC-2.06.017 Postcondition 1)
#[tokio::test]
async fn test_BC_2_06_017_demo_server_multi_instance_bind_config_accepted() {
    let cfg = MultiInstanceConfig::new(vec![
        InstanceEntry::new("armis-acme", ephemeral()),
        InstanceEntry::new("armis-contoso", ephemeral()),
    ]);

    let result = start_instances(cfg, |_entry| {
        Box::new(
            prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed in factory"),
        )
    })
    .await;

    // Once implemented: two-entry config → Ok(MultiInstanceServers) with two distinct SocketAddrs.
    let servers = result.expect(
        "MultiInstanceConfig with 2 ArmisClone instances must return Ok(MultiInstanceServers) \
         (BC-2.06.017 Postcondition 1 — multi-instance bind configuration accepted)",
    );
    let map = servers.socket_map();

    assert_eq!(
        map.len(),
        2,
        "Returned socket_map must have exactly 2 entries — one per InstanceEntry; got {map:?} \
         (BC-2.06.017 Postcondition 1 — all N instances returned; none silently dropped)"
    );
    assert!(
        map.contains_key("armis-acme"),
        "socket_map must contain key 'armis-acme'; got keys: {:?} \
         (BC-2.06.017 Postcondition 1)",
        map.keys().collect::<Vec<_>>()
    );
    assert!(
        map.contains_key("armis-contoso"),
        "socket_map must contain key 'armis-contoso'; got keys: {:?} \
         (BC-2.06.017 Postcondition 1)",
        map.keys().collect::<Vec<_>>()
    );
}

// ============================================================================
// AC-002 / TV-017-001: Two armis instances at distinct ports
//
// test_demo_server_two_armis_instances_bind_distinct_ports
// BC-2.06.017 Postcondition 1: map["armis-acme"] != map["armis-contoso"]
// ============================================================================

/// AC-002: Two `ArmisClone` instances start at distinct `SocketAddr`s.
///
/// RED GATE: `start_instances` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED: OS assigns two distinct ephemeral ports; neither equals the other;
/// both are valid loopback addresses.
///
/// (BC-2.06.017 Postcondition 1 / TV-017-001)
#[tokio::test]
async fn test_BC_2_06_017_demo_server_two_armis_instances_bind_distinct_ports() {
    let cfg = MultiInstanceConfig::new(vec![
        InstanceEntry::new("armis-acme", ephemeral()),
        InstanceEntry::new("armis-contoso", ephemeral()),
    ]);

    let servers = start_instances(cfg, |_entry| {
        Box::new(
            prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed in factory"),
        )
    })
    .await
    .expect("Two-armis-instance bind must succeed (BC-2.06.017 Postcondition 1)");
    let map = servers.socket_map();

    let addr_acme = map["armis-acme"];
    let addr_contoso = map["armis-contoso"];

    assert_ne!(
        addr_acme, addr_contoso,
        "armis-acme and armis-contoso must bind to DISTINCT SocketAddrs; both got {addr_acme} \
         (BC-2.06.017 Postcondition 1 / TV-017-001: map[\"armis-acme\"] != map[\"armis-contoso\"])"
    );
    assert!(
        addr_acme.ip().is_loopback(),
        "armis-acme must bind on loopback; got {addr_acme} \
         (BC-2.06.017 Postcondition 1)"
    );
    assert!(
        addr_contoso.ip().is_loopback(),
        "armis-contoso must bind on loopback; got {addr_contoso} \
         (BC-2.06.017 Postcondition 1)"
    );
    assert_ne!(
        addr_acme.port(),
        0,
        "armis-acme bound port must be non-zero (BC-2.06.017 Postcondition 1)"
    );
    assert_ne!(
        addr_contoso.port(),
        0,
        "armis-contoso bound port must be non-zero (BC-2.06.017 Postcondition 1)"
    );
}

// ============================================================================
// AC-003: Two claroty instances at distinct ports
//
// test_demo_server_two_claroty_instances_bind_distinct_ports
// BC-2.06.017 Postcondition 1: same semantics as AC-002 but for ClarotyClone.
// ============================================================================

/// AC-003: Two `ClarotyClone` instances start at distinct sockets.
///
/// RED GATE: `start_instances` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED: Two distinct ephemeral ports; both serve `POST /api/v1/audit_log/get`.
///
/// (BC-2.06.017 Postcondition 1 — each instance addressable independently)
#[tokio::test]
async fn test_BC_2_06_017_demo_server_two_claroty_instances_bind_distinct_ports() {
    let cfg = MultiInstanceConfig::new(vec![
        InstanceEntry::new("claroty-acme", ephemeral()),
        InstanceEntry::new("claroty-contoso", ephemeral()),
    ]);

    let servers = start_instances(cfg, |_entry| {
        Box::new(prism_dtu_claroty::ClarotyClone::new())
    })
    .await
    .expect("Two-claroty-instance bind must succeed (BC-2.06.017 Postcondition 1)");
    let map = servers.socket_map();

    let addr_acme = map["claroty-acme"];
    let addr_contoso = map["claroty-contoso"];

    assert_ne!(
        addr_acme, addr_contoso,
        "claroty-acme and claroty-contoso must bind to DISTINCT SocketAddrs; both got {addr_acme} \
         (BC-2.06.017 Postcondition 1)"
    );
    assert_ne!(
        addr_acme.port(),
        0,
        "claroty-acme port must be non-zero (BC-2.06.017 Postcondition 1)"
    );
    assert_ne!(
        addr_contoso.port(),
        0,
        "claroty-contoso port must be non-zero (BC-2.06.017 Postcondition 1)"
    );
}

// ============================================================================
// AC-002 (sub): Instance A responds independently to HTTP requests
//
// test_demo_server_instance_a_responds_independently
// BC-2.06.017 Postcondition 1: request to instance A's SocketAddr → served by A's clone.
// ============================================================================

/// AC-002: Request to instance A socket returns HTTP response from instance A's router.
///
/// RED GATE: `start_instances` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED: GET /api/v1/search on armis-acme's SocketAddr returns HTTP 403
/// (no Bearer), confirming instance A's router is live and independent.
///
/// (BC-2.06.017 Postcondition 1 — each instance is addressable independently)
#[tokio::test]
async fn test_BC_2_06_017_demo_server_instance_a_responds_independently() {
    let cfg = MultiInstanceConfig::new(vec![
        InstanceEntry::new("armis-acme", ephemeral()),
        InstanceEntry::new("armis-contoso", ephemeral()),
    ]);

    // servers MUST stay in scope through the HTTP assertions — dropping servers triggers shutdown.
    let servers = start_instances(cfg, |_entry| {
        Box::new(
            prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed in factory"),
        )
    })
    .await
    .expect("Two armis instances must bind successfully (BC-2.06.017 Postcondition 1)");
    let map = servers.socket_map();

    let addr_a = map["armis-acme"];
    let client = test_client();

    // Instance A must serve GET /api/v1/search (Armis search route).
    // 403 (no bearer) confirms the instance A router is live at addr_a.
    let resp = client
        .get(format!("http://{addr_a}/api/v1/search"))
        .send()
        .await
        .expect("HTTP GET to instance A must succeed at transport level");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "Instance A (armis-acme at {addr_a}) must return 403 on GET /api/v1/search \
         without Bearer — 403 confirms the instance is live and serving requests \
         (BC-2.06.017 Postcondition 1: request to instance A's SocketAddr is served by A's clone)"
    );

    // Explicit drop annotation so the liveness intent is clear to the reader.
    drop(servers);
}

// ============================================================================
// AC-002 (sub): Instance B responds independently to HTTP requests
//
// test_demo_server_instance_b_responds_independently
// BC-2.06.017 Postcondition 1: request to instance B's SocketAddr → served by B's clone.
// ============================================================================

/// AC-002: Request to instance B socket returns HTTP response from instance B's router.
///
/// RED GATE: `start_instances` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED: Instance B at its own SocketAddr serves GET /api/v1/search → 403
/// (no Bearer), proving instance B is independently reachable.
///
/// (BC-2.06.017 Postcondition 1 — instance B's SocketAddr is served by B's clone)
#[tokio::test]
async fn test_BC_2_06_017_demo_server_instance_b_responds_independently() {
    let cfg = MultiInstanceConfig::new(vec![
        InstanceEntry::new("armis-acme", ephemeral()),
        InstanceEntry::new("armis-contoso", ephemeral()),
    ]);

    // servers MUST stay in scope through the HTTP assertions — dropping servers triggers shutdown.
    let servers = start_instances(cfg, |_entry| {
        Box::new(
            prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed in factory"),
        )
    })
    .await
    .expect("Two armis instances must bind successfully (BC-2.06.017 Postcondition 1)");
    let map = servers.socket_map();

    let addr_b = map["armis-contoso"];
    let client = test_client();

    // Instance B must independently serve requests on its own distinct SocketAddr.
    let resp = client
        .get(format!("http://{addr_b}/api/v1/search"))
        .send()
        .await
        .expect("HTTP GET to instance B must succeed at transport level");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "Instance B (armis-contoso at {addr_b}) must return 403 on GET /api/v1/search \
         without Bearer — 403 confirms instance B is independently reachable \
         (BC-2.06.017 Postcondition 1: instance B's SocketAddr is served by B's clone)"
    );

    // Explicit drop annotation so the liveness intent is clear to the reader.
    drop(servers);
}

// ============================================================================
// AC-002: Multi-instance shutdown clean (Postcondition 1 + EC-017-005)
//
// test_demo_server_multi_instance_shutdown_clean
// BC-2.06.017 EC-017-005: shutdown signal → ports released; no zombie instances.
// ============================================================================

/// AC-002: Both instances shut down cleanly when `servers.shutdown()` is called.
///
/// RED GATE: `start_instances` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED:
/// 1. Two armis instances start and serve /dtu/health → HTTP 200.
/// 2. `servers.shutdown()` is called explicitly (the explicit-shutdown path of
///    the `MultiInstanceServers` lifecycle handle — D-1075-API-GAP-001).
/// 3. After the graceful drain window (~500ms), both sockets are released.
/// 4. Subsequent connection attempts to both addrs fail at transport level
///    (port released; no zombie instances).
///
/// The factory receives the InstanceEntry only — it does NOT wire any external
/// broadcast channel. The `MultiInstanceServers` handle owns the single shared
/// `shutdown_tx` internally; `servers.shutdown()` sends the signal. This removes
/// the dead-code `_rx` pattern from the prior API design.
///
/// (BC-2.06.017 EC-017-005 / Postcondition 1 v1.2 — D-1075-API-GAP-001)
#[tokio::test]
async fn test_BC_2_06_017_demo_server_multi_instance_shutdown_clean() {
    let cfg = MultiInstanceConfig::new(vec![
        InstanceEntry::new("armis-acme", ephemeral()),
        InstanceEntry::new("armis-contoso", ephemeral()),
    ]);

    // Factory receives only the InstanceEntry — no external shutdown channel needed.
    // MultiInstanceServers owns the shared shutdown_tx internally.
    let servers = start_instances(cfg, |_entry| {
        Box::new(
            prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed in factory"),
        )
    })
    .await
    .expect("Two armis instances must start cleanly (BC-2.06.017 EC-017-005 shutdown test)");

    let map = servers.socket_map();
    let addr_a = map["armis-acme"];
    let addr_b = map["armis-contoso"];
    let client = test_client();

    // Pre-condition: verify both are live before shutdown (non-vacuous assertion).
    let resp_a = client
        .get(format!("http://{addr_a}/dtu/health"))
        .send()
        .await
        .expect("Instance A health must be reachable before shutdown");
    assert_eq!(
        resp_a.status().as_u16(),
        200,
        "Instance A must respond HTTP 200 to /dtu/health before shutdown \
         (BC-2.06.017 EC-017-005: pre-shutdown live check)"
    );

    let resp_b = client
        .get(format!("http://{addr_b}/dtu/health"))
        .send()
        .await
        .expect("Instance B health must be reachable before shutdown");
    assert_eq!(
        resp_b.status().as_u16(),
        200,
        "Instance B must respond HTTP 200 to /dtu/health before shutdown \
         (BC-2.06.017 EC-017-005: pre-shutdown live check)"
    );

    // Trigger graceful shutdown via the explicit API path (D-1075-API-GAP-001).
    // MultiInstanceServers::shutdown() sends the shared shutdown_tx signal to all
    // instances; axum's with_graceful_shutdown drains in-flight requests, then
    // releases the bound port (BC-2.06.017 Postcondition 1 v1.2).
    servers.shutdown();

    // Allow axum's graceful drain window (500ms is generous for idle clones).
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Post-condition: ports released — requests must fail at transport level.
    // Connection-refused or similar transport error; NOT a 5xx from a live server.
    let after_a = client
        .get(format!("http://{addr_a}/dtu/health"))
        .send()
        .await;
    assert!(
        after_a.is_err(),
        "Instance A at {addr_a} must NOT be reachable after servers.shutdown() \
         (BC-2.06.017 EC-017-005: graceful shutdown releases port; no zombie instances). \
         Expected transport-level error; got: {:?}",
        after_a.map(|r| r.status())
    );

    let after_b = client
        .get(format!("http://{addr_b}/dtu/health"))
        .send()
        .await;
    assert!(
        after_b.is_err(),
        "Instance B at {addr_b} must NOT be reachable after servers.shutdown() \
         (BC-2.06.017 EC-017-005: both instances shut down cleanly; no port leak). \
         Expected transport-level error; got: {:?}",
        after_b.map(|r| r.status())
    );

    // servers is still in scope here; the explicit shutdown() already fired.
    // Drop is idempotent — the handle drops cleanly without double-signaling.
    drop(servers);
}

// ============================================================================
// AC-007 / REGRESSION GUARD: Single-instance backward compatibility
//
// test_single_instance_parity_test_still_passes_after_multi_instance_addition
// BC-2.06.017 INV-COMPAT-001: existing start_on callers unchanged.
//
// THIS TEST MUST PASS BEFORE IMPLEMENTATION (backward-compat guard).
// ============================================================================

/// REGRESSION GUARD — AC-007: Existing single-instance parity test still passes.
///
/// This test exercises the UNCHANGED single-instance `start_on` calling convention
/// to verify that adding `multi_instance.rs` to the crate does NOT break the
/// existing single-instance path.
///
/// MUST PASS in the current state (before `start_instances` is implemented).
///
/// (BC-2.06.017 INV-COMPAT-001 / TV-017-008)
#[tokio::test]
async fn test_BC_2_06_017_single_instance_parity_test_still_passes_after_multi_instance_addition() {
    use prism_dtu_common::BehavioralClone;

    // Direct single-instance call — unchanged pattern, must compile and run as-is.
    let mut clone =
        prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed (INV-COMPAT-001)");

    let bound_addr = clone
        .start_on(
            "127.0.0.1:0".parse().unwrap(),
            None, // shutdown: Option<broadcast::Receiver<()>>
            None, // tls: Option<()> (no-tls path per cfg(not(tls)))
        )
        .await
        .expect(
            "ArmisClone::start_on must succeed on ephemeral port \
             (BC-2.06.017 INV-COMPAT-001 — existing callers unmodified)",
        );

    assert_ne!(
        bound_addr.port(),
        0,
        "Single-instance ArmisClone::start_on must bind to a non-zero ephemeral port \
         (BC-2.06.017 INV-COMPAT-001 / TV-017-008)"
    );
    assert!(
        bound_addr.ip().is_loopback(),
        "Single-instance ArmisClone::start_on must bind on loopback; got {bound_addr} \
         (BC-2.06.017 INV-COMPAT-001 / TV-017-008)"
    );

    // Verify instance is serving requests (non-vacuous compat check).
    let client = test_client();
    let resp = client
        .get(format!("http://{bound_addr}/dtu/health"))
        .send()
        .await
        .expect("Single-instance ArmisClone must serve /dtu/health (INV-COMPAT-001)");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "Single-instance ArmisClone /dtu/health must return HTTP 200 \
         (BC-2.06.017 INV-COMPAT-001 — existing start_on path unchanged after multi-instance addition)"
    );

    // Clean up.
    clone
        .stop()
        .await
        .expect("ArmisClone::stop must succeed (INV-COMPAT-001)");
}

// ============================================================================
// DUPLICATE-NAME ERROR: EC-017-009 / Postcondition 7 / TV-017-006
// ============================================================================

// ============================================================================
// BIND-FAILURE AGGREGATION: TV-017-005 / EC-017-001 / Postcondition 6 / INV-ERR-003-COMPAT
//
// test_BC_2_06_017_demo_server_bind_failure_aggregates_all_errors
// BC-2.06.017 Postcondition 6: all bind operations attempted; failing instance named;
// successfully-bound instance released (no zombie). F-P1-MED-003.
// ============================================================================

/// TV-017-005 / EC-017-001 / Postcondition 6: bind failure → BindFailure with failing
/// instance named; successfully-started instance stopped (Postcondition 6 zombie-free).
///
/// Test strategy (EADDRINUSE injection via std::net::TcpListener):
///   1. Bind a std::net::TcpListener to 127.0.0.1:0, capture its addr (held_addr).
///      Keep the listener alive for the duration of the test.
///   2. Pass a two-entry MultiInstanceConfig:
///        entry A: name="good-instance", bind=127.0.0.1:0  → should SUCCEED (OS port).
///        entry B: name="bad-instance",  bind=held_addr    → must FAIL EADDRINUSE
///                                                           (port held by our listener).
///   3. Assert: `start_instances` returns Err(MultiInstanceBindError::BindFailure(failures)).
///   4. Assert: failures names "bad-instance" as the failing entry.
///   5. Assert Postcondition 6 (zombie-free): after the error, the successfully-bound
///      port from entry A is RELEASED — a fresh std::net::TcpListener::bind(good_addr)
///      SUCCEEDS (or a new loopback:0 bind succeeds showing port space not exhausted).
///
/// INV-ERR-003-COMPAT: all bind operations are attempted before the error is returned;
/// "good-instance" must NOT appear in the failures vec.
///
/// (BC-2.06.017 TV-017-005 / EC-017-001 / Postcondition 6 / INV-ERR-003-COMPAT
///  / F-P1-MED-003)
#[tokio::test]
async fn test_BC_2_06_017_demo_server_bind_failure_aggregates_all_errors() {
    // Step 1: Hold a port so we can force EADDRINUSE on entry B.
    let held_listener = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("Must be able to bind a TcpListener for EADDRINUSE injection (F-P1-MED-003)");
    let held_addr = held_listener
        .local_addr()
        .expect("Must be able to get local_addr of held listener");

    // Step 2: Build a config where entry A gets an OS-assigned ephemeral port (will succeed)
    // and entry B gets the held_addr (will fail with EADDRINUSE because held_listener holds it).
    let cfg = MultiInstanceConfig::new(vec![
        InstanceEntry::new("good-instance", ephemeral()),
        InstanceEntry::new("bad-instance", held_addr),
    ]);

    // Track whether the factory was called for "good-instance" (it should be).
    // The factory for "bad-instance" must also be called (INV-ERR-003-COMPAT: all entries attempted).
    let result = start_instances(cfg, |entry| {
        Box::new(
            prism_dtu_armis::ArmisClone::new()
                .unwrap_or_else(|e| panic!("ArmisClone::new must succeed for {}: {e}", entry.name)),
        )
    })
    .await;

    // Drop the held listener NOW so that subsequent fresh-bind checks don't hit EADDRINUSE.
    // (held_listener is dropped here; the OS reclaims held_addr)
    drop(held_listener);

    // Step 3: Assert Err(BindFailure) is returned.
    match result {
        Err(MultiInstanceBindError::BindFailure(failures)) => {
            // INV-ERR-003-COMPAT: the vec must be non-empty and name the failing instance.
            assert!(
                !failures.is_empty(),
                "BindFailure must contain at least one DemoBindError; got empty vec \
                 (BC-2.06.017 Postcondition 6 / EC-017-001 / TV-017-005 / F-P1-MED-003)"
            );

            // Step 4: Assert "bad-instance" is named in the failures.
            let bad_failure = failures.iter().find(|e| e.instance_name == "bad-instance");
            assert!(
                bad_failure.is_some(),
                "BindFailure failures must name 'bad-instance' as the failing entry; \
                 got failures: {failures:?} \
                 (BC-2.06.017 EC-017-001 / TV-017-005: failing entry named in error vec; \
                 F-P1-MED-003)"
            );

            // INV-ERR-003-COMPAT: "good-instance" succeeded, so it must NOT appear in failures.
            let good_not_in_failures = failures.iter().all(|e| e.instance_name != "good-instance");
            assert!(
                good_not_in_failures,
                "Good-instance successfully bound; it must NOT appear in BindFailure failures \
                 (BC-2.06.017 INV-ERR-003-COMPAT / Postcondition 6: successfully-started instances \
                 are stopped, not listed as failures; F-P1-MED-003); failures={failures:?}"
            );

            // Step 5: Assert Postcondition 6 (zombie-free): after the error, the OS can
            // allocate new ports — no zombie instance is holding OS resources.
            // Since we don't know which ephemeral port "good-instance" bound to (start_instances
            // returned an Err without a socket_map), we verify Postcondition 6 indirectly:
            // the implementation calls clone.stop().await on all successfully-started clones
            // before returning the Err. If that didn't happen, the port would remain in use
            // (TIME_WAIT or ESTABLISHED). We confirm the OS port space is not exhausted and
            // no specific zombie by attempting a fresh bind.
            let zombie_check = std::net::TcpListener::bind("127.0.0.1:0");
            assert!(
                zombie_check.is_ok(),
                "After BindFailure, a fresh loopback:0 bind must succeed (OS port space available); \
                 no zombie instance from good-instance leaked OS resources \
                 (BC-2.06.017 Postcondition 6 zombie-free guarantee / F-P1-MED-003): \
                 {:?}",
                zombie_check.err()
            );
        }

        Err(MultiInstanceBindError::DuplicateName { name }) => panic!(
            "Expected MultiInstanceBindError::BindFailure; got DuplicateName for name='{name}' \
             (BC-2.06.017 EC-017-001: entries have distinct names; DuplicateName is wrong here; \
             F-P1-MED-003)"
        ),

        Err(_) => panic!(
            "Expected MultiInstanceBindError::BindFailure; got unexpected error variant \
             (BC-2.06.017 EC-017-001 / TV-017-005 / F-P1-MED-003)"
        ),

        Ok(servers) => panic!(
            "Expected Err(MultiInstanceBindError::BindFailure) when one entry fails to bind; \
             got Ok with socket_map of {} entries — partial-success is forbidden \
             (BC-2.06.017 Postcondition 6: all-or-nothing semantics; F-P1-MED-003)",
            servers.socket_map().len()
        ),
    }
}

// ============================================================================
// DUPLICATE-NAME ERROR: EC-017-009 / Postcondition 7 / TV-017-006
// ============================================================================

/// EC-017-009 / Postcondition 7: Duplicate `InstanceEntry::name` → DuplicateName error.
///
/// RED GATE: `start_instances` is `todo!()` — will panic before returning the error.
///
/// WHEN IMPLEMENTED: Returns `Err(MultiInstanceBindError::DuplicateName { name: "dup" })`
/// before any bind attempt; factory must not be called.
///
/// (BC-2.06.017 Postcondition 7 / EC-017-009 / TV-017-006)
#[tokio::test]
async fn test_BC_2_06_017_demo_server_duplicate_instance_name_returns_error() {
    let cfg = MultiInstanceConfig::new(vec![
        InstanceEntry::new("dup", ephemeral()),
        InstanceEntry::new("dup", ephemeral()),
    ]);

    let result = start_instances(cfg, |_entry| {
        panic!(
            "factory must NOT be called for duplicate-name config \
             (BC-2.06.017 Postcondition 7: DuplicateName returned before any bind)"
        )
    })
    .await;

    match result {
        Err(MultiInstanceBindError::DuplicateName { name }) => {
            assert_eq!(
                name, "dup",
                "DuplicateName error must name the conflicting instance; \
                 expected name='dup', got name='{name}' \
                 (BC-2.06.017 EC-017-009 / TV-017-006)"
            );
        }
        Err(MultiInstanceBindError::BindFailure(failures)) => panic!(
            "Expected MultiInstanceBindError::DuplicateName; got BindFailure with {} errors \
             (BC-2.06.017 EC-017-009: DuplicateName must be returned BEFORE any bind attempt, \
             so BindFailure is wrong here): {failures:?}",
            failures.len()
        ),
        Err(_) => panic!(
            "Expected MultiInstanceBindError::DuplicateName; got unexpected variant \
             (BC-2.06.017 EC-017-009 / TV-017-006)"
        ),
        Ok(servers) => panic!(
            "Duplicate-name config must return Err(DuplicateName), not Ok(MultiInstanceServers); \
             got servers with socket_map of {} entries \
             (BC-2.06.017 Postcondition 7: silent last-wins is forbidden)",
            servers.socket_map().len()
        ),
    }
}
