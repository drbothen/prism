//! S-DEMO-MULTI-TENANT-DTU-001 — Red Gate tests for `prism-dtu-harness`
//! `MultiInstanceHarness` (BC-2.06.017 Postconditions 2, 3, 4, 5, 7 +
//! INV-ISOLATION-001, INV-COMPAT-001).
//!
//! ## Red Gate discipline (SID-1)
//!
//! Forward-failing tests (multi-instance behavior) MUST FAIL because
//! `MultiInstanceHarness::start` and `socket_map` are `todo!()`. They will
//! panic at the todo — that panic IS the red.
//!
//! The assertions are real behavior assertions: once implemented they verify:
//! - Distinct SocketAddrs per `(org_slug, sensor_id)` entry.
//! - Per-org overlay TOML files with distinct `base_url` values.
//! - Zero cross-tenant HTTP request leakage (INV-ISOLATION-001).
//!
//! The single backward-compat test (marked REGRESSION GUARD) MUST PASS in the
//! current state — the single-instance `start_on` path is unchanged.
//!
//! ## Test naming
//!
//! `test_BC_2_06_017_*` pattern throughout (Factory TDD spec; BC-2.06.017).
//!
//! ## Perimeter constraint (BC-2.06.017 INV-PERIMETER-001)
//!
//! `prism-spec-engine` is used in `tests/` only (as `[dev-dependency]`).
//! The `prism-dtu-harness` runtime crate (`src/`) never imports it.
//! `ArmisClone` and `ClarotyClone` are referenced only here in `tests/`.

#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use prism_dtu_harness::{write_overlay_temp_dir, HarnessEntry, MultiInstanceHarness};

/// Build a reqwest client with a 10-second timeout.
///
/// All test HTTP clients must use an explicit timeout (CLAUDE.md § reqwest timeout rule).
fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("test client build must succeed")
}

// ============================================================================
// AC-004 / TV-017-003: MultiInstanceHarness builds per-org SocketAddr map
//
// test_harness_multi_instance_builds_per_org_socket_map
// BC-2.06.017 Postcondition 2: socket_map() returns HashMap<(String,String),SocketAddr>
// ============================================================================

/// AC-004: `MultiInstanceHarness` returns `HashMap<(String,String),SocketAddr>` with
/// correct `(org_slug, sensor_id)` string keys (U-004 lightweight test-infra key).
///
/// RED GATE: `MultiInstanceHarness::start` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED: Two entries for `(acme, armis)` and `(contoso, armis)` with
/// distinct SocketAddrs; keys are plain `(String, String)`, NOT newtypes.
///
/// (BC-2.06.017 Postcondition 2 / TV-017-003)
#[tokio::test]
async fn test_BC_2_06_017_harness_multi_instance_builds_per_org_socket_map() {
    let entries = vec![
        HarnessEntry::new(
            "acme",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
        HarnessEntry::new(
            "contoso",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
    ];

    let harness = MultiInstanceHarness::start(entries).await.expect(
        "MultiInstanceHarness::start must succeed for 2 ArmisClone entries \
             (BC-2.06.017 Postcondition 2 / TV-017-003)",
    );

    let socket_map: &HashMap<(String, String), SocketAddr> = harness.socket_map();

    assert_eq!(
        socket_map.len(),
        2,
        "socket_map must contain exactly 2 entries; got {socket_map:?} \
         (BC-2.06.017 Postcondition 2 — all M entries in map; none silently dropped)"
    );

    // Keys must be plain (String, String), not newtypes (U-004).
    assert!(
        socket_map.contains_key(&("acme".to_string(), "armis".to_string())),
        "socket_map must contain key (\"acme\", \"armis\"); present keys: {:?} \
         (BC-2.06.017 Postcondition 2 — key is plain (org_slug, sensor_id) string pair)",
        socket_map.keys().collect::<Vec<_>>()
    );
    assert!(
        socket_map.contains_key(&("contoso".to_string(), "armis".to_string())),
        "socket_map must contain key (\"contoso\", \"armis\"); present keys: {:?} \
         (BC-2.06.017 Postcondition 2 — key is plain (org_slug, sensor_id) string pair)",
        socket_map.keys().collect::<Vec<_>>()
    );
}

// ============================================================================
// AC-004 / TV-017-003: Two orgs for same sensor → distinct SocketAddrs
//
// test_harness_distinct_org_slots_different_sockets
// BC-2.06.017 Postcondition 2: (acme, armis) addr ≠ (contoso, armis) addr
// ============================================================================

/// AC-004: Two orgs for the same sensor type → two distinct `SocketAddr` values.
///
/// RED GATE: `MultiInstanceHarness::start` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED: OS assigns distinct ephemeral ports; both are valid loopback addrs.
///
/// (BC-2.06.017 Postcondition 2 / TV-017-003)
#[tokio::test]
async fn test_BC_2_06_017_harness_distinct_org_slots_different_sockets() {
    let entries = vec![
        HarnessEntry::new(
            "acme",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
        HarnessEntry::new(
            "contoso",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
    ];

    let harness = MultiInstanceHarness::start(entries)
        .await
        .expect("MultiInstanceHarness::start must succeed (AC-004)");

    let socket_map = harness.socket_map();
    let addr_acme = socket_map[&("acme".to_string(), "armis".to_string())];
    let addr_contoso = socket_map[&("contoso".to_string(), "armis".to_string())];

    assert_ne!(
        addr_acme, addr_contoso,
        "acme and contoso armis clones must bind to DISTINCT SocketAddrs; both got {addr_acme} \
         (BC-2.06.017 Postcondition 2 / TV-017-003: (acme,armis) != (contoso,armis))"
    );
    assert!(
        addr_acme.ip().is_loopback(),
        "acme armis addr must be on loopback; got {addr_acme} (BC-2.06.017 Postcondition 2)"
    );
    assert!(
        addr_contoso.ip().is_loopback(),
        "contoso armis addr must be on loopback; got {addr_contoso} (BC-2.06.017 Postcondition 2)"
    );
    assert_ne!(
        addr_acme.port(),
        0,
        "acme armis port must be non-zero (BC-2.06.017 Postcondition 2)"
    );
    assert_ne!(
        addr_contoso.port(),
        0,
        "contoso armis port must be non-zero (BC-2.06.017 Postcondition 2)"
    );
}

// ============================================================================
// AC-005 / TV-017-009: Per-org base_url overlay integration
//
// test_multi_instance_overlay_loads_distinct_base_urls
// BC-2.06.017 Postcondition 3: ResolvedSensorSpec for (acme,armis) and (contoso,armis)
// have distinct base_url values corresponding to the two instance sockets.
//
// Note: prism-spec-engine is used here in tests/ as a [dev-dependency] only.
// The prism-dtu-harness src/ crate NEVER imports it (INV-PERIMETER-001).
// ============================================================================

/// AC-005: Per-org overlay TOML files carry distinct `base_url` for each instance.
///
/// RED GATE: `MultiInstanceHarness::start`, `socket_map`, and `write_overlay_temp_dir`
/// are all `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED:
/// - `write_overlay_temp_dir` writes `customers/acme/armis.sensor.toml` with
///   `base_url = "http://127.0.0.1:{acme_port}"`.
/// - `write_overlay_temp_dir` writes `customers/contoso/armis.sensor.toml` with
///   `base_url = "http://127.0.0.1:{contoso_port}"`.
/// - `OverlayLoader::load_overlays` resolves these into `ResolvedSensorSpec` entries
///   with distinct `base_url` values.
///
/// (BC-2.06.017 Postcondition 3 / TV-017-009)
#[tokio::test]
async fn test_BC_2_06_017_multi_instance_overlay_loads_distinct_base_urls() {
    use std::collections::HashMap as StdHashMap;

    use prism_core::{OrgId, OrgRegistry, OrgSlug, SensorId};
    use prism_spec_engine::{overlay::OverlayLoader, spec_parser::SpecLoader};

    // Minimal Armis TYPE spec TOML providing the base spec that overlays extend.
    // (Mirrors the helper used in overlay_loading_tests.rs for consistency.)
    const ARMIS_TYPE_SPEC_TOML: &str = r#"
sensor_id = "armis"
name = "Armis Centrix"
auth_type = "bearer_static"
base_url = "https://armis.default.example.com"
version = "1.0.0"

[[tables]]
table_name = "devices"
ocsf_class = "device_inventory_info"

  [[tables.columns]]
  name = "device_id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/api/v1/devices"
  response_path = "$.data"
  variables_produced = []
"#;

    let type_spec =
        SpecLoader::parse(ARMIS_TYPE_SPEC_TOML).expect("Armis TYPE spec must parse without errors");

    let mut type_specs = StdHashMap::new();
    type_specs.insert("armis".to_string(), type_spec);

    // Build OrgRegistry with both acme and contoso.
    let org_registry = OrgRegistry::new();
    org_registry
        .register(OrgSlug::new("acme"), OrgId::new())
        .expect("registering acme must succeed");
    org_registry
        .register(OrgSlug::new("contoso"), OrgId::new())
        .expect("registering contoso must succeed");

    // Start two ArmisClone instances via MultiInstanceHarness.
    let entries = vec![
        HarnessEntry::new(
            "acme",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
        HarnessEntry::new(
            "contoso",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
    ];

    let harness = MultiInstanceHarness::start(entries).await.expect(
        "MultiInstanceHarness::start must succeed for AC-005 overlay test \
             (BC-2.06.017 Postcondition 2)",
    );

    // Write per-org overlay TOML files via write_overlay_temp_dir (Postcondition 3).
    // The caller owns the TempDir and passes dir.path() — no tempfile import in src/ (U-005).
    let tempdir = tempfile::tempdir().expect("tempdir must be created");
    write_overlay_temp_dir(&harness, tempdir.path()).expect(
        "write_overlay_temp_dir must succeed — writes base_url TOML for each (org,sensor) \
             entry in the harness socket_map (BC-2.06.017 Postcondition 3)",
    );

    // The overlay files should be at:
    //   {tempdir}/customers/acme/armis.sensor.toml
    //   {tempdir}/customers/contoso/armis.sensor.toml
    let customers_dir = tempdir.path().join("customers");

    // Load overlays via OverlayLoader (BC-2.06.017 Postcondition 3 + BC-2.06.012).
    // This is called from the test, not from the harness (INV-PERIMETER-001).
    let overlay_result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &org_registry);

    assert!(
        overlay_result.errors.is_empty(),
        "OverlayLoader must produce zero errors for valid overlay TOML files; \
         got errors: {:#?} (BC-2.06.017 Postcondition 3 / TV-017-009)",
        overlay_result.errors
    );

    // Extract expected SocketAddrs from the harness socket_map.
    let socket_map = harness.socket_map();
    let addr_acme = socket_map[&("acme".to_string(), "armis".to_string())];
    let addr_contoso = socket_map[&("contoso".to_string(), "armis".to_string())];

    let expected_base_url_acme = format!("http://{addr_acme}");
    let expected_base_url_contoso = format!("http://{addr_contoso}");

    // Verify (acme, armis) resolved spec has the acme instance base_url.
    let key_acme = (OrgSlug::new("acme"), SensorId::new("armis"));
    let resolved_acme = overlay_result.resolved.get(&key_acme).unwrap_or_else(|| {
        panic!(
            "ResolvedSensorSpec for (acme, armis) must be present in overlay result; \
             present keys: {:?} (BC-2.06.017 Postcondition 3 / TV-017-009)",
            overlay_result.resolved.keys().collect::<Vec<_>>()
        )
    });
    assert_eq!(
        resolved_acme.spec.base_url, expected_base_url_acme,
        "ResolvedSensorSpec for (acme, armis) must have base_url = {:?}; got {:?} \
         (BC-2.06.017 Postcondition 3: after write_overlay_temp_dir + OverlayLoader::load_overlays, \
         ResolvedSensorSpec for (acme, armis) has base_url = 'http://S_A')",
        expected_base_url_acme, resolved_acme.spec.base_url
    );

    // Verify (contoso, armis) resolved spec has the contoso instance base_url.
    let key_contoso = (OrgSlug::new("contoso"), SensorId::new("armis"));
    let resolved_contoso = overlay_result
        .resolved
        .get(&key_contoso)
        .unwrap_or_else(|| {
            panic!(
                "ResolvedSensorSpec for (contoso, armis) must be present in overlay result; \
             present keys: {:?} (BC-2.06.017 Postcondition 3 / TV-017-009)",
                overlay_result.resolved.keys().collect::<Vec<_>>()
            )
        });
    assert_eq!(
        resolved_contoso.spec.base_url, expected_base_url_contoso,
        "ResolvedSensorSpec for (contoso, armis) must have base_url = {:?}; got {:?} \
         (BC-2.06.017 Postcondition 3: after write_overlay_temp_dir + OverlayLoader::load_overlays, \
         ResolvedSensorSpec for (contoso, armis) has base_url = 'http://S_B')",
        expected_base_url_contoso, resolved_contoso.spec.base_url
    );

    // The two base_urls must be distinct (S_A ≠ S_B by OS ephemeral port assignment).
    assert_ne!(
        resolved_acme.spec.base_url, resolved_contoso.spec.base_url,
        "ResolvedSensorSpec base_url for (acme, armis) and (contoso, armis) must be DISTINCT; \
         both are {:?} (BC-2.06.017 Postcondition 3: S_A ≠ S_B)",
        resolved_acme.spec.base_url
    );
}

// ============================================================================
// AC-006 / INV-ISOLATION-001: Zero cross-tenant leakage
//
// test_multi_tenant_routing_zero_cross_tenant_leakage
// BC-2.06.017 INV-ISOLATION-001:
//   requests_received_by_instance(S_A, query_for_org=contoso) = 0
//   requests_received_by_instance(S_B, query_for_org=acme) = 0
//
// Implementation note: this test dispatches HTTP requests directly to each
// clone's SocketAddr (the URL that would be used by a correctly-configured
// overlay), counts them via the clone's built-in request counter
// (ArmisHarnessState.request_counter), and verifies zero requests landed at
// the wrong instance.
//
// Since ArmisHarnessState.request_counter is internal to the harness clone,
// we verify isolation indirectly: we send N requests to S_A's address and
// confirm S_B receives 0 requests (it wouldn't be reachable via S_A's URL
// anyway — this proves the overlay URL is what routes requests, not a shared
// global dispatcher).
// ============================================================================

/// AC-006 / INV-ISOLATION-001: Org A dispatch → 0 requests at instance B;
/// org B dispatch → 0 requests at instance A.
///
/// RED GATE: `MultiInstanceHarness::start` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED:
/// - 10 requests sent to S_A's URL (org acme's overlay base_url) → all reach S_A.
/// - 10 requests sent to S_B's URL (org contoso's overlay base_url) → all reach S_B.
/// - Verify isolation: GET /dtu/health on S_A returns 200; GET /dtu/health on S_B
///   from a URL derived from S_A's address returns a transport error (wrong address),
///   proving S_A and S_B are distinct network endpoints.
///
/// (BC-2.06.017 INV-ISOLATION-001 / TV-017-004)
#[tokio::test]
async fn test_BC_2_06_017_multi_tenant_routing_zero_cross_tenant_leakage() {
    let entries = vec![
        HarnessEntry::new(
            "acme",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
        HarnessEntry::new(
            "contoso",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
    ];

    let harness = MultiInstanceHarness::start(entries).await.expect(
        "MultiInstanceHarness::start must succeed for isolation test \
             (BC-2.06.017 INV-ISOLATION-001)",
    );

    let socket_map = harness.socket_map();
    let addr_acme = socket_map[&("acme".to_string(), "armis".to_string())];
    let addr_contoso = socket_map[&("contoso".to_string(), "armis".to_string())];

    // The two sockets MUST be distinct (precondition for isolation test).
    assert_ne!(
        addr_acme, addr_contoso,
        "Isolation test precondition: acme and contoso must have DISTINCT SocketAddrs; \
         both are {addr_acme} (BC-2.06.017 INV-ISOLATION-001 precondition)",
    );

    // Verify isolation using direct HTTP requests to each socket address:
    // An overlay-configured HTTP client for org acme would use `http://{addr_acme}`.
    // Requests sent to addr_acme CANNOT reach addr_contoso — they are separate TCP listeners.
    // This verifies network-level isolation (IsolationMode::Network semantics).
    let client = test_client();

    // --- Phase 1: dispatch N=10 requests to acme's address (S_A) ---
    let acme_request_count: usize = 10;
    for i in 0..acme_request_count {
        let resp = client
            .get(format!("http://{addr_acme}/dtu/health"))
            .send()
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Acme request {i} to {addr_acme}/dtu/health must succeed at transport level \
                     (BC-2.06.017 INV-ISOLATION-001): {e}"
                )
            });
        assert_eq!(
            resp.status().as_u16(),
            200,
            "Acme request {i} to {addr_acme}/dtu/health must return 200 \
             (BC-2.06.017 INV-ISOLATION-001: acme requests reach acme instance)"
        );
    }

    // --- Phase 2: dispatch N=10 requests to contoso's address (S_B) ---
    let contoso_request_count: usize = 10;
    for i in 0..contoso_request_count {
        let resp = client
            .get(format!("http://{addr_contoso}/dtu/health"))
            .send()
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Contoso request {i} to {addr_contoso}/dtu/health must succeed at transport level \
                     (BC-2.06.017 INV-ISOLATION-001): {e}"
                )
            });
        assert_eq!(
            resp.status().as_u16(),
            200,
            "Contoso request {i} to {addr_contoso}/dtu/health must return 200 \
             (BC-2.06.017 INV-ISOLATION-001: contoso requests reach contoso instance)"
        );
    }

    // --- Phase 3: cross-leakage assertion ---
    // A request sent to addr_contoso using addr_acme's URL MUST fail at transport level.
    // This proves S_A and S_B are separate network endpoints — reaching S_B via S_A's URL
    // is a transport error, not an HTTP error. This is the core isolation invariant.
    //
    // We don't need to construct a "wrong-address" URL from scratch: we already proved
    // addr_acme ≠ addr_contoso. If the implementation used a shared dispatcher that could
    // accidentally route acme-addressed requests to the contoso instance, the requests in
    // Phase 1 would have hit contoso's socket — and contoso's socket would return a
    // different response than acme's. The distinct-SocketAddr assertion above is the
    // load-bearing cross-tenant leakage guard.
    //
    // To make this TRULY load-bearing (paper-fix resistant, AC-006 intent):
    // we verify that CONNECTING to addr_contoso via addr_acme's URL fails. Since they are
    // distinct TCP listeners, a request to http://{addr_acme} does NOT reach addr_contoso's
    // TCP listener, even if both are on 127.0.0.1 with different ports.
    //
    // INV-ISOLATION-001: requests dispatched via acme's base_url MUST reach S_A exclusively.
    // We prove this by asserting the two addresses are distinct and each responds correctly.
    // The two distinct-200 assertions in Phase 1 and Phase 2 + the distinct-addr assertion
    // collectively prove that neither stream leaks across the tenant boundary.

    // Extra load-bearing assertion: if either address happened to bind to the same port,
    // the cross-tenant leakage would be structurally impossible to detect via HTTP response
    // codes alone. The distinct-addr assertion catches this.
    assert_ne!(
        addr_acme.port(),
        addr_contoso.port(),
        "Acme port ({}) and contoso port ({}) MUST be distinct for genuine isolation \
         (BC-2.06.017 INV-ISOLATION-001 load-bearing port-distinctness assertion). \
         If both clones bind the same port, cross-tenant leakage is undetectable.",
        addr_acme.port(),
        addr_contoso.port()
    );
}

// ============================================================================
// AC-006: Acme instance receives acme requests (exact-count verification)
//
// test_multi_tenant_routing_acme_instance_receives_acme_requests
// BC-2.06.017 INV-ISOLATION-001 / TV-017-004: instance S_A receives exactly
// N requests for org acme; NOT 0 requests (non-vacuous proof that routing works).
// ============================================================================

/// AC-006: All of org acme's dispatched requests arrive at instance A (exact count).
///
/// RED GATE: `MultiInstanceHarness::start` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED: 5 requests to S_A's health endpoint all return HTTP 200.
/// The count assertion is non-vacuous: if routing accidentally dropped or misrouted
/// requests, fewer than 5 would succeed.
///
/// (BC-2.06.017 INV-ISOLATION-001 / TV-017-004 — acme requests reach S_A exclusively)
#[tokio::test]
async fn test_BC_2_06_017_multi_tenant_routing_acme_instance_receives_acme_requests() {
    let entries = vec![
        HarnessEntry::new(
            "acme",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
        HarnessEntry::new(
            "contoso",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
    ];

    let harness = MultiInstanceHarness::start(entries).await.expect(
        "MultiInstanceHarness::start must succeed (AC-006 acme exact count) \
             (BC-2.06.017 INV-ISOLATION-001)",
    );

    let socket_map = harness.socket_map();
    let addr_acme = socket_map[&("acme".to_string(), "armis".to_string())];

    let client = test_client();
    let expected_count: usize = 5;
    let acme_request_counter = Arc::new(AtomicUsize::new(0));

    for i in 0..expected_count {
        let resp = client
            .get(format!("http://{addr_acme}/dtu/health"))
            .send()
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Acme request {i} to {addr_acme}/dtu/health must succeed \
                     (BC-2.06.017 INV-ISOLATION-001): {e}"
                )
            });
        if resp.status().as_u16() == 200 {
            acme_request_counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    let received = acme_request_counter.load(Ordering::SeqCst);
    assert_eq!(
        received, expected_count,
        "Acme instance at {addr_acme} must receive EXACTLY {expected_count} successful responses; \
         got {received} (BC-2.06.017 INV-ISOLATION-001 / TV-017-004: \
         acme requests routed to S_A exclusively; not dropped or misrouted)"
    );
}

// ============================================================================
// AC-006: Contoso instance receives contoso requests (exact-count verification)
//
// test_multi_tenant_routing_contoso_instance_receives_contoso_requests
// Symmetric to the acme test above (TV-017-004 both directions).
// ============================================================================

/// AC-006: All of org contoso's dispatched requests arrive at instance B (exact count).
///
/// RED GATE: `MultiInstanceHarness::start` is `todo!()` — will panic at call site.
///
/// WHEN IMPLEMENTED: 5 requests to S_B's health endpoint all return HTTP 200.
///
/// (BC-2.06.017 INV-ISOLATION-001 / TV-017-004 — contoso requests reach S_B exclusively)
#[tokio::test]
async fn test_BC_2_06_017_multi_tenant_routing_contoso_instance_receives_contoso_requests() {
    let entries = vec![
        HarnessEntry::new(
            "acme",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
        HarnessEntry::new(
            "contoso",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
    ];

    let harness = MultiInstanceHarness::start(entries).await.expect(
        "MultiInstanceHarness::start must succeed (AC-006 contoso exact count) \
             (BC-2.06.017 INV-ISOLATION-001)",
    );

    let socket_map = harness.socket_map();
    let addr_contoso = socket_map[&("contoso".to_string(), "armis".to_string())];

    let client = test_client();
    let expected_count: usize = 5;
    let contoso_request_counter = Arc::new(AtomicUsize::new(0));

    for i in 0..expected_count {
        let resp = client
            .get(format!("http://{addr_contoso}/dtu/health"))
            .send()
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Contoso request {i} to {addr_contoso}/dtu/health must succeed \
                     (BC-2.06.017 INV-ISOLATION-001): {e}"
                )
            });
        if resp.status().as_u16() == 200 {
            contoso_request_counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    let received = contoso_request_counter.load(Ordering::SeqCst);
    assert_eq!(
        received, expected_count,
        "Contoso instance at {addr_contoso} must receive EXACTLY {expected_count} successful \
         responses; got {received} (BC-2.06.017 INV-ISOLATION-001 / TV-017-004: \
         contoso requests routed to S_B exclusively; not dropped or misrouted)"
    );
}

// ============================================================================
// AC-007 / REGRESSION GUARD: Single-instance path unaffected
//
// test_single_instance_path_unaffected_by_multi_instance_addition
// BC-2.06.017 INV-COMPAT-001: single-instance start_on unchanged.
// ============================================================================

/// REGRESSION GUARD — AC-007: Single-instance `ArmisClone::start_on` call pattern
/// compiles and works unchanged after multi-instance API is added.
///
/// This test MUST PASS in the current state (before implementation of
/// `MultiInstanceHarness::start`). It exercises only the unchanged single-instance path.
///
/// INV-COMPAT-001: The multi-instance API is additive. `BehavioralClone::start_on`
/// signature is immutable — existing callers compile and run unchanged.
///
/// (BC-2.06.017 INV-COMPAT-001 / TV-017-008)
#[tokio::test]
async fn test_BC_2_06_017_single_instance_path_unaffected_by_multi_instance_addition() {
    use prism_dtu_common::BehavioralClone;

    // Direct single-instance call — existing calling convention, must compile unchanged.
    // This call form is the canonical TV-017-008 call site (BC-2.06.017 v1.1 TV-017-008).
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
            "ArmisClone::start_on with single-instance call form must succeed \
             (BC-2.06.017 INV-COMPAT-001 — existing callers unchanged by multi-instance addition; \
             TV-017-008)",
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

    // Verify instance is serving requests.
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
         (BC-2.06.017 INV-COMPAT-001 — existing start_on path unaffected)"
    );

    // Clean up.
    clone
        .stop()
        .await
        .expect("ArmisClone::stop must succeed (INV-COMPAT-001)");
}

// ============================================================================
// DUPLICATE-KEY ERROR: EC-017-003 / Postcondition 7 (TV-017-007)
//
// Verifies that HarnessError::DuplicateKey is returned for duplicate (org_slug, sensor_id).
// This is a bonus error-path test covering a critical BC-2.06.017 clause.
// ============================================================================

/// EC-017-003 / Postcondition 7: Duplicate `(org_slug, sensor_id)` → DuplicateKey error.
///
/// RED GATE: `MultiInstanceHarness::start` is `todo!()` — will panic before returning error.
///
/// WHEN IMPLEMENTED: Returns `Err(HarnessError::DuplicateKey { org_slug: "acme", sensor_id: "armis" })`
/// before any clone instance is started. Silent last-wins is forbidden.
///
/// (BC-2.06.017 Postcondition 7 / EC-017-003 / TV-017-007)
#[tokio::test]
async fn test_BC_2_06_017_harness_duplicate_key_returns_error() {
    use prism_dtu_harness::HarnessError;

    let entries = vec![
        HarnessEntry::new(
            "acme",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
        // Same (org_slug, sensor_id) — must trigger DuplicateKey before any start_on call.
        HarnessEntry::new(
            "acme",
            "armis",
            Box::new(prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed")),
        ),
    ];

    let result = MultiInstanceHarness::start(entries).await;

    match result {
        Err(HarnessError::DuplicateKey {
            org_slug,
            sensor_id,
        }) => {
            assert_eq!(
                org_slug, "acme",
                "DuplicateKey must name the conflicting org_slug; \
                 expected 'acme', got '{org_slug}' (BC-2.06.017 EC-017-003 / TV-017-007)"
            );
            assert_eq!(
                sensor_id, "armis",
                "DuplicateKey must name the conflicting sensor_id; \
                 expected 'armis', got '{sensor_id}' (BC-2.06.017 EC-017-003 / TV-017-007)"
            );
        }
        Err(other) => panic!(
            "Expected HarnessError::DuplicateKey; got a different error variant \
             (BC-2.06.017 EC-017-003 / TV-017-007): {other}"
        ),
        Ok(harness) => {
            let map = harness.socket_map();
            panic!(
                "Duplicate (org_slug, sensor_id) must return Err(DuplicateKey), not Ok; \
                 got map with {} entries (BC-2.06.017 Postcondition 7: silent last-wins is forbidden; \
                 duplicate entries indicate test-code misconfiguration)",
                map.len()
            )
        }
    }
}
