//! End-to-end FanOutTarget → DTU instance routing isolation test.
//!
//! Closes PR-LEVEL finding F-PR3-HIGH-001 (architect-adjudicated 2026-06-14):
//! The AC-006 isolation tests in `prism-dtu-harness/tests/multi_instance_harness_tests.rs`
//! prove that the harness binds DISTINCT listeners and those listeners don't cross-receive
//! (distinct-listener TCP isolation), but they drive HTTP requests directly to the socket
//! address obtained from `socket_map()` — they do NOT exercise the PRODUCTION dispatch path
//! (`fan_out_with_overlay_map` consuming `ResolvedSensorSpec.base_url`).
//!
//! This test drives the REAL production dispatch path:
//!
//! 1. Two `ArmisClone` instances started via `MultiInstanceHarness::start` at
//!    distinct ephemeral sockets S_A (acme) and S_B (contoso).
//! 2. Overlays written via `write_overlay_temp_dir` and loaded via the production
//!    `OverlayLoader::load_overlays` path → `ResolvedSensorSpec` map.
//! 3. `fan_out_with_overlay_map(targets_for_acme, ...)` dispatches via a
//!    `RealHttpDispatchAdapter` that reads `sensor_config["base_url"]` and issues
//!    `GET {base_url}/dtu/health` — a REAL HTTP request to whichever DTU instance
//!    the overlay resolution directed it to.
//! 4. Server-side `GET /dtu/request-count` counters (delta scheme from
//!    `multi_instance_harness_tests.rs`) prove which instance received the requests.
//!
//! ASSERTION that FAILS on routing bug:
//!   acme dispatch → S_A receives N vendor requests; S_B receives ZERO.
//!   contoso dispatch → S_B receives N vendor requests; S_A receives ZERO.
//!   If `fan_out_with_overlay_map` ignores the overlay `base_url` and uses the
//!   TYPE spec URL (or routes to the wrong instance), delta_b > 0 for the acme
//!   window → assertion FAILS.
//!
//! # Dependency direction (INV-PERIMETER-001)
//!
//! `prism-sensors` (prod) does NOT depend on `prism-dtu-harness`. Only this test
//! file (in `tests/`, a dev-dep scope) imports `prism-dtu-harness`. The permitted
//! direction is: `prism-sensors [dev-dep] → prism-dtu-harness`. The FORBIDDEN
//! direction (`prism-dtu-harness → prism-sensors`) does NOT exist.
//!
//! Story: S-DEMO-MULTI-TENANT-DTU-001 | BC-2.06.014, BC-2.06.017 INV-ISOLATION-001
//! Finding: F-PR3-HIGH-001 (PR-level, architect-adjudicated)

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    non_snake_case,
    clippy::await_holding_lock
)]

use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::{OrgId, OrgRegistry, OrgSlug, SensorId};
use prism_sensors::{
    adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    fanout::{fan_out_with_overlay_map, CredentialResolver, FanOutTarget},
    AdapterRegistry,
};
use prism_spec_engine::overlay::OverlayLoader;

use prism_dtu_harness::{write_overlay_temp_dir, HarnessEntry, MultiInstanceHarness};

// ---------------------------------------------------------------------------
// Minimal Armis TYPE spec (mirrors the helper in fanout.rs unit tests)
// ---------------------------------------------------------------------------

const ARMIS_TYPE_SPEC_TOML: &str = r#"
sensor_id = "armis"
name = "Armis test"
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

// ---------------------------------------------------------------------------
// RealHttpDispatchAdapter
//
// A SensorAdapter that makes a REAL HTTP GET to {sensor_config["base_url"]}/dtu/health.
//
// This is the load-bearing part of F-PR3-HIGH-001: the adapter reads the
// base_url that fan_out_with_overlay_map injects into sensor_config and
// issues a genuine network request to whichever DTU instance socket it resolves
// to. The server-side /dtu/request-count counter on each DTU instance records
// every hit, so cross-tenant leakage is proven impossible to hide.
//
// Timeout: 10 seconds (CLAUDE.md reqwest timeout rule for test clients).
// ---------------------------------------------------------------------------

/// Real HTTP dispatch adapter for the end-to-end routing isolation test.
///
/// On each `fetch()` call, reads `sensor_config["base_url"]` and issues N_HITS
/// `GET {base_url}/dtu/health` requests to whichever ArmisClone socket
/// `fan_out_with_overlay_map` directed it to (via the overlay base_url injection).
///
/// N_HITS is configurable per instance so each fan-out call generates a known
/// number of server-side hits, enabling the delta-counting assertion.
struct RealHttpDispatchAdapter {
    /// The sensor ID this adapter handles (must be "armis" for OrgRegistry lookup).
    sensor_id: SensorId,
    /// Number of `GET /dtu/health` requests to issue per `fetch()` call.
    hits_per_fetch: u64,
    /// Shared reqwest client with explicit timeout (CLAUDE.md § reqwest timeout rule).
    client: reqwest::Client,
}

impl RealHttpDispatchAdapter {
    fn new(hits_per_fetch: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("RealHttpDispatchAdapter: reqwest client build must succeed");
        Self {
            sensor_id: SensorId::from("armis"),
            hits_per_fetch,
            client,
        }
    }
}

#[async_trait]
impl SensorAdapter for RealHttpDispatchAdapter {
    fn sensor_type(&self) -> SensorId {
        self.sensor_id.clone()
    }

    fn sensor_name(&self) -> &'static str {
        "real-http-dispatch-adapter"
    }

    async fn fetch(
        &self,
        spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<FetchOutput, SensorError> {
        // Read the base_url that fan_out_with_overlay_map injected into sensor_config.
        // This is the REAL overlay base_url pointing to the per-org DTU instance socket.
        // If the overlay injection is broken, this will be the TYPE spec URL
        // (https://armis.default.example.com) which is not listening → connection error.
        let base_url = spec
            .sensor_config
            .get("base_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SensorError::Internal {
                detail: "RealHttpDispatchAdapter: sensor_config[\"base_url\"] missing — \
                         fan_out_with_overlay_map must inject overlay base_url before dispatch \
                         (BC-2.06.014 Case A)"
                    .to_string(),
            })?;

        // Issue hits_per_fetch real HTTP requests to {base_url}/dtu/health.
        // Each request hits the server-side request counter on the target DTU instance.
        for i in 0..self.hits_per_fetch {
            let url = format!("{base_url}/dtu/health");
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| SensorError::Internal {
                    detail: format!(
                        "RealHttpDispatchAdapter: GET {url} (hit {i}) failed — \
                         this means the overlay base_url '{base_url}' is not reachable. \
                         If base_url is 'https://armis.default.example.com', the overlay \
                         injection is BROKEN (F-PR3-HIGH-001): {e}"
                    ),
                })?;

            if resp.status().as_u16() != 200 {
                return Err(SensorError::HttpError {
                    sensor: "armis".to_string(),
                    status: resp.status().as_u16(),
                    body: format!(
                        "GET {base_url}/dtu/health (hit {i}) returned non-200: {}",
                        resp.status()
                    ),
                });
            }
        }

        // Return empty success — the routing proof is in the server-side counter, not data.
        Ok(FetchOutput::new(vec![], false))
    }
}

// ---------------------------------------------------------------------------
// Stub CredentialResolver
// ---------------------------------------------------------------------------

/// Stub credential resolver — returns a no-op SensorAuth for any (client, sensor).
///
/// Credentials are irrelevant for this routing test: the DTU instances don't
/// enforce auth for /dtu/health, and we only care about which instance receives
/// the request.
struct NoopCreds;

impl CredentialResolver for NoopCreds {
    fn resolve(
        &self,
        _client_id: &str,
        _sensor_id: SensorId,
    ) -> Result<Box<dyn SensorAuth>, SensorError> {
        struct NoopAuth;
        impl SensorAuth for NoopAuth {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn auth_type_name(&self) -> &'static str {
                "custom_via_plugin"
            }
        }
        Ok(Box::new(NoopAuth))
    }
}

// ---------------------------------------------------------------------------
// Server-side counter helper (mirrors multi_instance_harness_tests.rs)
// ---------------------------------------------------------------------------

/// Read the per-instance server-side request counter from GET /dtu/request-count.
///
/// This IS itself a request — it increments the counter on the target instance.
/// Use the delta scheme (baseline → dispatch → final) to isolate the window.
async fn read_request_count(client: &reqwest::Client, addr: SocketAddr) -> u64 {
    let body = client
        .get(format!("http://{addr}/dtu/request-count"))
        .send()
        .await
        .unwrap_or_else(|e| {
            panic!(
                "GET /dtu/request-count on {addr} must succeed (F-PR3-HIGH-001 isolation probe): {e}"
            )
        })
        .text()
        .await
        .unwrap_or_else(|e| panic!("GET /dtu/request-count body read failed: {e}"));
    body.trim().parse::<u64>().unwrap_or_else(|e| {
        panic!(
            "GET /dtu/request-count response '{body}' must be a plain decimal u64; \
             parse error: {e} (F-PR3-HIGH-001)"
        )
    })
}

// ---------------------------------------------------------------------------
// The load-bearing end-to-end test
// ---------------------------------------------------------------------------

/// End-to-end FanOutTarget DTU routing isolation test (F-PR3-HIGH-001).
///
/// Proves that `fan_out_with_overlay_map` dispatches to the CORRECT per-org DTU
/// instance by driving the production dispatch path (overlay injection → real HTTP)
/// and reading server-side counters on each ArmisClone instance.
///
/// # What this proves (load-bearing, not a paper-fix)
///
/// - The overlay base_url IS injected into `sensor_config["base_url"]` at fan-out
///   (BC-2.06.014 Case A / ADR-029).
/// - The `RealHttpDispatchAdapter::fetch` actually sends HTTP requests to the
///   resolved URL (not to a TYPE spec URL that would fail to connect).
/// - The server-side counter on each DTU instance records every hit on THAT instance's
///   router — cross-tenant leakage is impossible to hide:
///     - acme window: S_A delta = N+1 (N vendor hits + 1 final probe), S_B delta = 1
///       (only the final probe — zero vendor hits from acme dispatch).
///     - contoso window (symmetric): S_B delta = N+1, S_A delta = 1.
///
/// # Failure semantics
///
/// The test FAILS if:
/// - `fan_out_with_overlay_map` does NOT inject the overlay base_url
///   → `RealHttpDispatchAdapter::fetch` returns `SensorError::Internal` (URL missing)
///     or `SensorError::Internal` (connection refused to TYPE spec host) → fan_out errors.
/// - `fan_out_with_overlay_map` routes acme to S_B instead of S_A
///   → S_B delta > 1, S_A delta ≠ N+1 → delta assertions FAIL.
/// - `fan_out_with_overlay_map` routes to the wrong instance for contoso
///   → symmetric failure.
///
/// Story: S-DEMO-MULTI-TENANT-DTU-001 | BC-2.06.014, BC-2.06.017 INV-ISOLATION-001
/// Finding: F-PR3-HIGH-001
#[tokio::test]
async fn test_fan_out_with_overlay_map_routes_to_correct_dtu_instance() {
    // --- Step 1: Start two ArmisClone instances via MultiInstanceHarness ---
    // S_A = acme's instance, S_B = contoso's instance (distinct ephemeral sockets).
    let entries = vec![
        HarnessEntry::new(
            "acme",
            "armis",
            Box::new(
                prism_dtu_armis::ArmisClone::new()
                    .expect("ArmisClone::new must succeed for acme (F-PR3-HIGH-001)"),
            ),
        ),
        HarnessEntry::new(
            "contoso",
            "armis",
            Box::new(
                prism_dtu_armis::ArmisClone::new()
                    .expect("ArmisClone::new must succeed for contoso (F-PR3-HIGH-001)"),
            ),
        ),
    ];

    let harness = MultiInstanceHarness::start(entries).await.expect(
        "MultiInstanceHarness::start must succeed for 2 ArmisClone entries (F-PR3-HIGH-001)",
    );

    let socket_map = harness.socket_map();
    let addr_acme = socket_map[&("acme".to_string(), "armis".to_string())];
    let addr_contoso = socket_map[&("contoso".to_string(), "armis".to_string())];

    // Structural precondition: distinct sockets (required for meaningful isolation proof).
    assert_ne!(
        addr_acme, addr_contoso,
        "F-PR3-HIGH-001 precondition: acme and contoso must have DISTINCT SocketAddrs; \
         both are {addr_acme} (MultiInstanceHarness must assign distinct ephemeral ports)"
    );

    // --- Step 2: Write overlays and load via production OverlayLoader path ---
    // write_overlay_temp_dir → customers/acme/armis.sensor.toml (base_url = http://addr_acme)
    //                        → customers/contoso/armis.sensor.toml (base_url = http://addr_contoso)
    let tempdir = tempfile::tempdir().expect("tempdir must be created (F-PR3-HIGH-001)");
    write_overlay_temp_dir(&harness, tempdir.path()).expect(
        "write_overlay_temp_dir must succeed — writes per-org overlay TOML files \
         (BC-2.06.017 Postcondition 3 / F-PR3-HIGH-001)",
    );

    // Build type_specs map and register both orgs.
    let armis_type_spec = prism_spec_engine::spec_parser::SpecLoader::parse(ARMIS_TYPE_SPEC_TOML)
        .expect("Armis TYPE spec must parse (F-PR3-HIGH-001)");
    let mut type_specs = HashMap::new();
    type_specs.insert("armis".to_string(), armis_type_spec);

    // OrgRegistry: register acme and contoso with distinct OrgIds.
    let org_registry = OrgRegistry::new();
    let org_id_acme = OrgId::new();
    let org_id_contoso = OrgId::new();

    org_registry
        .register(OrgSlug::new("acme"), org_id_acme)
        .expect("registering acme must succeed (F-PR3-HIGH-001)");
    org_registry
        .register(OrgSlug::new("contoso"), org_id_contoso)
        .expect("registering contoso must succeed (F-PR3-HIGH-001)");

    // Load overlays via the production OverlayLoader path.
    let customers_dir = tempdir.path().join("customers");
    let overlay_result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &org_registry);

    assert!(
        overlay_result.errors.is_empty(),
        "OverlayLoader must produce zero errors for valid overlay TOML files (F-PR3-HIGH-001); \
         got errors: {:#?}",
        overlay_result.errors
    );

    let resolved_spec_map = Arc::new(overlay_result.resolved);

    // --- Step 3: Build AdapterRegistry with per-org RealHttpDispatchAdapters ---
    // N_VENDOR_HITS: number of real HTTP requests each fan_out call issues to the target DTU.
    // Must be large enough to distinguish from noise (1 final probe); 5 is sufficient.
    const N_VENDOR_HITS: u64 = 5;

    // Register a RealHttpDispatchAdapter for each org.
    // IMPORTANT: both adapters have sensor_type() = "armis" but are keyed by DISTINCT OrgIds.
    // This is the BC-3.2.001 composite (OrgId, SensorId) key invariant — the registry
    // must route acme → acme's adapter and contoso → contoso's adapter.
    let mut registry = AdapterRegistry::new();
    let adapter_acme = Arc::new(RealHttpDispatchAdapter::new(N_VENDOR_HITS));
    let adapter_contoso = Arc::new(RealHttpDispatchAdapter::new(N_VENDOR_HITS));

    registry.register(
        org_id_acme,
        Arc::clone(&adapter_acme) as Arc<dyn SensorAdapter>,
    );
    registry.register(
        org_id_contoso,
        Arc::clone(&adapter_contoso) as Arc<dyn SensorAdapter>,
    );
    let registry = Arc::new(registry);

    // Shared credential resolver and org_registry Arc.
    let creds: Arc<dyn CredentialResolver> = Arc::new(NoopCreds);
    let org_registry_arc = Arc::new(org_registry);

    // Probe client for reading server-side counters (with 10s timeout per CLAUDE.md rule).
    let probe_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("probe client build must succeed (F-PR3-HIGH-001)");

    // ---------------------------------------------------------------------------
    // Phase A: acme dispatch — fan_out_with_overlay_map for org_id_acme
    //
    // Expected routing: overlay for (acme, armis) has base_url = http://addr_acme
    // → RealHttpDispatchAdapter::fetch receives base_url pointing to S_A
    // → N_VENDOR_HITS GET /dtu/health requests land on S_A's router
    // ---------------------------------------------------------------------------

    // Step A1: Read baseline from both instances (each probe increments that instance's counter).
    let baseline_a = read_request_count(&probe_client, addr_acme).await;
    let baseline_b = read_request_count(&probe_client, addr_contoso).await;

    // Step A2: Build FanOutTarget for acme.
    // spec.sensor_config["base_url"] is set to the TYPE spec URL (the PRE-overlay URL).
    // fan_out_with_overlay_map MUST replace it with the overlay base_url before dispatch.
    // This is the invariant under test: if replacement doesn't happen, the adapter tries
    // to connect to "https://armis.default.example.com" which is not listening → error.
    #[allow(deprecated)]
    let target_acme = FanOutTarget {
        org_id: org_id_acme,
        client_id: "acme".to_string(),
        sensor_id: SensorId::from("armis"),
        spec: SensorSpec {
            source_table: "armis_devices".to_string(),
            org_id: org_id_acme,
            client_id: "acme".to_string(),
            // TYPE spec URL — must be replaced by overlay URL before HTTP dispatch.
            sensor_config: serde_json::json!({ "base_url": "https://armis.default.example.com" }),
        },
        params: QueryParams::default(),
    };

    // Step A3: Dispatch via the PRODUCTION fan_out_with_overlay_map path.
    let result_acme = fan_out_with_overlay_map(
        vec![target_acme],
        Arc::clone(&registry),
        Arc::clone(&creds),
        Arc::clone(&org_registry_arc),
        Arc::clone(&resolved_spec_map),
    )
    .await;

    assert!(
        result_acme.is_ok(),
        "fan_out_with_overlay_map for acme must succeed — if it errors, \
         the overlay base_url was NOT injected (F-PR3-HIGH-001 routing bug): {result_acme:?}"
    );
    let result_acme = result_acme.unwrap();
    assert!(
        result_acme.errors.is_empty(),
        "fan_out_with_overlay_map for acme must return zero errors (F-PR3-HIGH-001); \
         got: {:?}",
        result_acme.errors
    );

    // Step A4: Read final from both instances (each probe increments that instance's counter).
    let final_a = read_request_count(&probe_client, addr_acme).await;
    let final_b = read_request_count(&probe_client, addr_contoso).await;

    // Delta accounting (mirrors multi_instance_harness_tests.rs scheme):
    //   delta_a = final_a - baseline_a = N_VENDOR_HITS + 1  (N vendor hits + 1 final probe on S_A)
    //   delta_b = final_b - baseline_b = 1                   (only the final probe on S_B; ZERO vendor hits from acme)
    //
    // If fan_out_with_overlay_map routes acme → S_B instead of S_A:
    //   delta_b > 1 → this assertion FAILS.
    let delta_a = final_a.saturating_sub(baseline_a);
    let delta_b = final_b.saturating_sub(baseline_b);

    assert_eq!(
        delta_a,
        N_VENDOR_HITS + 1,
        "Instance S_A ({addr_acme}) must have received EXACTLY {} requests in the acme window \
         ({N_VENDOR_HITS} vendor hits from fan_out_with_overlay_map + 1 final probe); \
         got delta_a={delta_a} \
         (F-PR3-HIGH-001: server-side counter on S_A proves acme dispatch reached S_A via \
         overlay base_url injection in fan_out_with_overlay_map — BC-2.06.014 Case A / ADR-029)",
        N_VENDOR_HITS + 1
    );

    assert_eq!(
        delta_b, 1,
        "Instance S_B ({addr_contoso}) must have received EXACTLY 1 request in the acme window \
         (the final probe only — ZERO acme vendor requests must reach S_B's router); \
         got delta_b={delta_b} \
         (F-PR3-HIGH-001 LOAD-BEARING ISOLATION ASSERTION: \
         server-side counter on S_B proves INV-ISOLATION-001 for the production dispatch path — \
         if delta_b > 1, fan_out_with_overlay_map routed acme requests to S_B and the \
         production routing is BROKEN. This CANNOT be hidden by the CapturingAdapter pattern \
         in the existing unit test.)",
    );

    // ---------------------------------------------------------------------------
    // Phase B: contoso dispatch — symmetric proof
    //
    // Expected routing: overlay for (contoso, armis) has base_url = http://addr_contoso
    // → RealHttpDispatchAdapter::fetch receives base_url pointing to S_B
    // → N_VENDOR_HITS GET /dtu/health requests land on S_B's router; ZERO on S_A
    // ---------------------------------------------------------------------------

    // Step B1: Read baseline from both instances.
    let baseline_a2 = read_request_count(&probe_client, addr_acme).await;
    let baseline_b2 = read_request_count(&probe_client, addr_contoso).await;

    // Step B2: Build FanOutTarget for contoso.
    #[allow(deprecated)]
    let target_contoso = FanOutTarget {
        org_id: org_id_contoso,
        client_id: "contoso".to_string(),
        sensor_id: SensorId::from("armis"),
        spec: SensorSpec {
            source_table: "armis_devices".to_string(),
            org_id: org_id_contoso,
            client_id: "contoso".to_string(),
            // TYPE spec URL — must be replaced by contoso's overlay URL before dispatch.
            sensor_config: serde_json::json!({ "base_url": "https://armis.default.example.com" }),
        },
        params: QueryParams::default(),
    };

    // Step B3: Dispatch via the PRODUCTION fan_out_with_overlay_map path.
    let result_contoso = fan_out_with_overlay_map(
        vec![target_contoso],
        Arc::clone(&registry),
        Arc::clone(&creds),
        Arc::clone(&org_registry_arc),
        Arc::clone(&resolved_spec_map),
    )
    .await;

    assert!(
        result_contoso.is_ok(),
        "fan_out_with_overlay_map for contoso must succeed — if it errors, \
         the overlay base_url was NOT injected for contoso (F-PR3-HIGH-001): {result_contoso:?}"
    );
    let result_contoso = result_contoso.unwrap();
    assert!(
        result_contoso.errors.is_empty(),
        "fan_out_with_overlay_map for contoso must return zero errors (F-PR3-HIGH-001); \
         got: {:?}",
        result_contoso.errors
    );

    // Step B4: Read final from both instances.
    let final_a2 = read_request_count(&probe_client, addr_acme).await;
    let final_b2 = read_request_count(&probe_client, addr_contoso).await;

    let delta_a2 = final_a2.saturating_sub(baseline_a2);
    let delta_b2 = final_b2.saturating_sub(baseline_b2);

    assert_eq!(
        delta_b2,
        N_VENDOR_HITS + 1,
        "Instance S_B ({addr_contoso}) must have received EXACTLY {} requests in the contoso window \
         ({N_VENDOR_HITS} vendor hits from fan_out_with_overlay_map + 1 final probe); \
         got delta_b2={delta_b2} \
         (F-PR3-HIGH-001: server-side counter on S_B proves contoso dispatch reached S_B via \
         overlay base_url injection in fan_out_with_overlay_map — BC-2.06.014 Case A / ADR-029)",
        N_VENDOR_HITS + 1
    );

    assert_eq!(
        delta_a2, 1,
        "Instance S_A ({addr_acme}) must have received EXACTLY 1 request in the contoso window \
         (the final probe only — ZERO contoso vendor requests must reach S_A's router); \
         got delta_a2={delta_a2} \
         (F-PR3-HIGH-001 LOAD-BEARING ISOLATION ASSERTION (symmetric): \
         server-side counter on S_A proves INV-ISOLATION-001 for the contoso dispatch path — \
         if delta_a2 > 1, fan_out_with_overlay_map routed contoso requests to S_A and \
         the production routing is BROKEN.)",
    );
}
