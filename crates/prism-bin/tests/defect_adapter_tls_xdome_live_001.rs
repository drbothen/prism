// SPDX-License-Identifier: Apache-2.0
//! Red Gate tests for DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — integration surface.
//!
//! # Tests in this file
//!
//! | RG   | Test Name                                                    | RED reason (pre-fix)                                                      |
//! |------|--------------------------------------------------------------|---------------------------------------------------------------------------|
//! | RG-005 | test_probe_connectivity_403_returns_up_not_down            | `map_spec_engine_error_to_sensor_error` maps HttpRequestFailed → Internal |
//! |         |                                                              | Internal → catch-all → Down; assertion `status == Up` FAILS              |
//! | RG-007 | test_sensor_health_wire_shape_403_reachable_auth_invalid  | Follows RG-005 failure; if Up/403, wire shape must carry reachable=true   |
//! |         |                                                              | and auth_valid=false; SID-2 wire-level JSON assertion.                    |
//! | RG-008 | test_reqwest_http2_feature_active                         | RED pre-fix (rustls-tls only; http2 disabled); GREEN post-fix (http2 on). |
//! | RG-014 | test_probe_connectivity_503_returns_degraded               | 5xx→Internal→Down pre-fix; `status == Degraded` assertion FAILS           |
//! |         |                                                              | (F-P25-OBS-001 coverage gap — no test covered the 5xx→Degraded branch).   |
//! | RG-015 | test_BC_2_01_013_EC_01_029_cookie_roundtrip_401_auth_invalid_wire_shape | CookieAuthFailed→Internal→Down pre-fix; `"reachable":true` and            |
//! |         |                                                              | `"auth_valid":false` wire assertions FAIL (F-P31-OBS-001).                |
//! | LOW-1  | test_sensor_health_wire_shape_401_auth_invalid_production_path | OAuth2 401→Internal→Down pre-fix; `"reachable":true` and                  |
//! |         |                                                              | `"auth_valid":false` wire assertions FAIL (LOW-1 companion to RG-007).    |
//! | RG-019 | test_BC_2_08_002_degraded_reachable_wire_shape             | check_one sets `reachable = connectivity == Up`; Degraded (5xx) yields    |
//! |         |                                                              | `reachable:false`; assertion `"reachable":true` FAILS → RED (HS-007).     |
//! | RG-020 | test_BC_2_08_002_degraded_envelope_summary_matches_overall_status | server.rs `fully_healthy_count` lacks `&& s.error.is_none()` (T-SERVER-1) |
//! |         |                                                              | → Degraded counted as healthy → summary "1 of 1" contradicts              |
//! |         |                                                              | `overall_status:"partial"`; assertions "1 of 1" ABSENT + "0 of 1"         |
//! |         |                                                              | PRESENT FAIL → RED (AC-WIRE-003).                                         |
//!
//! # Test seam
//!
//! Tests wire a real `SpecDrivenSensorAdapter` against a `wiremock` mock server
//! returning HTTP 403 with body `b"forbidden"`. The adapter is registered in a
//! real `AdapterRegistry`. `probe_connectivity` is called with the real org_id.
//!
//! The fixture uses:
//! - sensor_id `"xdome"`, table `"devices"` → `source_table = "xdome_devices"` (probe sentinel path)
//! - `AuthType::CustomViaPlugin` + `MockAuthProvider` so the Plugin auth path is taken
//!   (BearerStatic path short-circuits with Internal before any HTTP; Plugin path makes real calls)
//!
//! BCs: BC-2.08.002
//! Story: DEFECT-ADAPTER-TLS-XDOME-LIVE-001

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    non_snake_case,
    dead_code,
    unused_imports
)]
extern crate toml;

use std::sync::Arc;

use prism_bin::spec_driven_adapter::{
    AdapterAuthStrategy, SpecDrivenSensorAdapter, build_http_client_with_timeout,
};
use prism_core::column::ColumnType;
use prism_core::{OrgId, OrgSlug, SensorId};
use prism_mcp::PrismContext;
use prism_mcp::health::SensorHealthChecker;
use prism_mcp::health::connectivity::{ConnectivityStatus, probe_connectivity};
use prism_mcp::resources::SensorHealthResult;
use prism_sensors::AdapterRegistry;
use prism_spec_engine::{
    auth_provider::MockAuthProvider,
    overlay::{OverlayLoader, SensorInstanceOverlay},
    spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// ---------------------------------------------------------------------------
// Shared fixtures
// ---------------------------------------------------------------------------

/// Build a `SensorSpec` for sensor "xdome" with a single "devices" table
/// at GET /api/v1/devices using `CustomViaPlugin` auth.
///
/// Using `CustomViaPlugin` ensures `SpecDrivenSensorAdapter::fetch()` takes
/// the Plugin auth path (which issues real HTTP calls), rather than the
/// BearerStatic path (which short-circuits on ProbeAuth downcast mismatch
/// and returns Internal before any network contact).
fn make_xdome_spec(base_url: &str) -> SensorSpec {
    SensorSpec::new(
        "xdome",
        "xDome Sensor (RG fixture)",
        AuthType::CustomViaPlugin,
        base_url,
        vec![TableSpec::new_point_in_time(
            "devices",
            "network_activity",
            vec![ColumnSpec::new(
                "device_id",
                ColumnType::String,
                None,
                vec![],
            )],
            vec![FetchStep::new(
                "fetch_devices",
                "GET",
                "/api/v1/devices",
                None,
                "$.items",
                None,
                vec![],
                None,
                None,
            )],
        )],
        None,
        "1.0.0",
        vec![],
    )
}

/// Wrap a `SensorSpec` in a `ResolvedSensorSpec` for `org_slug`.
///
/// Uses `OverlayLoader::merge_overlay_onto_type_spec` — the only documented
/// external construction path for `ResolvedSensorSpec` (`#[non_exhaustive]`).
fn make_resolved(spec: SensorSpec, org_slug: &str) -> prism_spec_engine::ResolvedSensorSpec {
    let toml = format!(
        "extends = \"{sid}\"\ninstance_id = \"{sid}@{org}\"",
        sid = spec.sensor_id,
        org = org_slug
    );
    let overlay: SensorInstanceOverlay =
        toml::from_str(&toml).expect("RG fixture: SensorInstanceOverlay TOML parse failed");
    OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, OrgSlug::new(org_slug))
}

// ---------------------------------------------------------------------------
// RG-005 — probe_connectivity must return Up + http_status=403 on 403 response
// ---------------------------------------------------------------------------

/// RG-005: `probe_connectivity` MUST return `ConnectivityStatus::Up` and
/// `http_status = Some(403)` when the sensor API returns HTTP 403.
///
/// Call chain: `probe_connectivity` → `SpecDrivenSensorAdapter::fetch()` →
/// `PipelineExecutor::execute()` → receives 403 → `SpecEngineError::HttpRequestFailed
/// { status_code: 403, .. }` → `map_spec_engine_error_to_sensor_error` →
/// (after fix) `SensorError::HttpError { status: 403, .. }` → probe classifies
/// as `ConnectivityStatus::Up` (4xx = reachable, auth-invalid).
///
/// FAIL reason before fix: `map_spec_engine_error_to_sensor_error` maps ALL
/// `SpecEngineError` variants to `SensorError::Internal` →
/// probe catch-all arm `Err(e)` → `ConnectivityStatus::Down, http_status: None`
/// → `assert_eq!(outcome.status, ConnectivityStatus::Up)` panics → RED.
///
/// BC-2.08.002 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-005
#[tokio::test]
async fn test_probe_connectivity_403_returns_up_not_down() {
    let mock_server = MockServer::start().await;

    // Wiremock: any GET /api/v1/devices → 403 Forbidden with body "forbidden".
    Mock::given(method("GET"))
        .and(path("/api/v1/devices"))
        .respond_with(ResponseTemplate::new(403).set_body_bytes(b"forbidden"))
        .mount(&mock_server)
        .await;

    // Build and register the adapter.
    let spec = make_xdome_spec(&mock_server.uri());
    let resolved = make_resolved(spec, "rg005-org");
    let auth_strategy =
        AdapterAuthStrategy::Plugin(Arc::new(MockAuthProvider::new("rg005-token"))
            as Arc<dyn prism_spec_engine::AuthProvider>);
    let http_client = build_http_client_with_timeout().expect("RG-005: http client build failed");
    let adapter = Arc::new(SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        auth_strategy,
        http_client,
    ));

    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);

    let sensor_id = SensorId::from("xdome");

    // Call probe_connectivity with a real AdapterRegistry.
    let outcome = probe_connectivity(&registry, org_id, &sensor_id, "rg005-client")
        .await
        .expect("RG-005: probe_connectivity must not return Err(PrismError)");

    // ASSERTION 1: status must be Up (sensor reachable, auth-invalid).
    // FAIL before fix: SensorError::Internal → catch-all → Down.
    assert_eq!(
        outcome.status,
        ConnectivityStatus::Up,
        "RG-005: probe must return ConnectivityStatus::Up for 403 response. \
         Got: {:?}. \
         Root cause: map_spec_engine_error_to_sensor_error maps HttpRequestFailed → Internal \
         instead of HttpError. Fix: match HttpRequestFailed {{ status_code }} when status_code > 0 \
         and return SensorError::HttpError (BC-2.08.002 DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-005).",
        outcome.status
    );

    // ASSERTION 2: http_status must be Some(403).
    // FAIL before fix: Internal → catch-all → http_status: None.
    assert_eq!(
        outcome.http_status,
        Some(403),
        "RG-005: probe.http_status must be Some(403) for 403 response. \
         Got: {:?}. \
         Fix: SensorError::HttpError must carry status_code from HttpRequestFailed \
         (BC-2.08.002 DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-005).",
        outcome.http_status
    );
}

// ---------------------------------------------------------------------------
// RG-007 — SensorHealthResult wire-shape: reachable=true, auth_valid=false
// ---------------------------------------------------------------------------

/// RG-007: `SensorHealthChecker::check_one` (production path) MUST return a
/// `SensorHealthResult` that serializes to JSON containing `"reachable":true`
/// and `"auth_valid":false` for a 403 response.
///
/// REWORKED (OBS-5): previously hand-built `SensorHealthResult` and re-implemented the
/// `AuthStatus → Option<bool>` mapping in-test, so `"auth_valid":false` was not load-bearing
/// (only the serde shape was exercised). This version goes through `check_one` end-to-end,
/// so a regression in `check_one`'s value derivation (e.g., 403 → auth_valid=true) WOULD
/// fail this test.
///
/// 403 semantics flowing through `check_one`:
/// 1. `probe_auth_with_routing` → 403 → `AuthStatus::Invalid`, `ConnectivityStatus::Up`
/// 2. `check_one` maps `Up → reachable = true`, `Invalid → auth_valid_opt = Some(false)`
/// 3. `SensorHealthResult.with_auth_valid(false)` → `"auth_valid":false` in serialized JSON
///
/// FAIL reason before fix: `map_spec_engine_error_to_sensor_error` maps HttpRequestFailed →
/// Internal → probe classifies as Down → `check_one` sets `reachable = false` →
/// `"reachable":false` in JSON → assertion FAILS → RED.
///
/// BC-2.08.002 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-007
#[tokio::test]
async fn test_sensor_health_wire_shape_403_reachable_auth_invalid() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/devices"))
        .respond_with(ResponseTemplate::new(403).set_body_bytes(b"forbidden"))
        .mount(&mock_server)
        .await;

    let spec = make_xdome_spec(&mock_server.uri());
    let resolved = make_resolved(spec, "rg007-org");
    let auth_strategy =
        AdapterAuthStrategy::Plugin(Arc::new(MockAuthProvider::new("rg007-token"))
            as Arc<dyn prism_spec_engine::AuthProvider>);
    let http_client = build_http_client_with_timeout().expect("RG-007: http client build failed");
    let adapter = Arc::new(SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        auth_strategy,
        http_client,
    ));

    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);
    let sensor_id = SensorId::from("xdome");

    // PRODUCTION PATH (OBS-5): use SensorHealthChecker::check_one so the
    // AuthStatus → Option<bool> derivation is exercised by production code,
    // not re-implemented in-test. A regression in check_one's mapping WOULD fail here.
    let checker = SensorHealthChecker::new(Arc::new(registry));
    let context = PrismContext::new();
    let health = checker
        .check_one(org_id, "rg007-org", &sensor_id, &context)
        .await;

    // SID-2 wire-shape assertion: serialize to compact JSON and check key:value pairs.
    let json = serde_json::to_string(&health).expect("RG-007: SensorHealthResult must serialize");

    // ASSERTION 1: "reachable":true — sensor responded (Up), not Down.
    // FAIL before fix: probe returns Down → check_one sets reachable=false → "reachable":false.
    assert!(
        json.contains("\"reachable\":true"),
        "RG-007 (SID-2 wire-shape): JSON must contain '\"reachable\":true' for 403 response. \
         Got JSON: {json}. \
         Root cause: map_spec_engine_error_to_sensor_error maps HttpRequestFailed → Internal \
         instead of HttpError, so check_one sets reachable=false. \
         Fix: return SensorError::HttpError for 4xx responses \
         (BC-2.08.002 DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-007)."
    );

    // ASSERTION 2: "auth_valid":false — driven by production check_one path.
    // Load-bearing: if check_one's AuthStatus::Invalid → Some(false) mapping regressed
    // (e.g., returned Some(true) for 403), this assertion would fail.
    assert!(
        json.contains("\"auth_valid\":false"),
        "RG-007 (SID-2 wire-shape): JSON must contain '\"auth_valid\":false' for 403 response \
         via production check_one path. Got JSON: {json}."
    );
}

// ---------------------------------------------------------------------------
// RG-008 — reqwest http2 feature MUST be active on the reqwest node (AC-H2-001)
// ---------------------------------------------------------------------------

/// RG-008: The `reqwest` dependency in `Cargo.lock` MUST list `h2` as a
/// direct dependency of the `reqwest` package — confirming the `http2`
/// feature is active on the reqwest node (ADR-050 §D5).
///
/// Scope: checks ONLY the `[[package]] name = "reqwest"` block —
/// NOT the whole file (h2 is already present transitively via hyper 1.x,
/// so a whole-file grep would be GREEN before this fix, which is invalid).
///
/// RED before fix: `"http2"` not yet in any production reqwest Cargo.toml
/// entry → `h2` absent from reqwest's own Cargo.lock block → assertion fails.
///
/// GREEN after fix: `"http2"` added to THREE production reqwest [dependencies]
/// entries (prism-spec-engine/Cargo.toml, prism-sensors/Cargo.toml,
/// prism-bin/Cargo.toml — one production entry, the AC-9 shared client) plus
/// prism-bin/Cargo.toml [dev-dependencies] (which also explicitly lists `http2`)
/// → Cargo resolves h2 as a direct dep of reqwest → `h2` appears in reqwest
/// block → passes.
///
/// This replaces the inverted `test_reqwest_http2_not_enabled` (was
/// GREEN-by-design before adding http2) with the correct RED-then-GREEN gate.
///
/// AC-H2-001 | BC-2.16.002 HTTP Client Compliance (ADR-050 §D5) |
/// DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-008
#[test]
fn test_reqwest_http2_feature_active() {
    // Cargo.lock lives at the workspace root, two levels above prism-bin's Cargo.toml.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let cargo_lock_path = std::path::PathBuf::from(manifest_dir)
        .join("..") // crates/
        .join("..") // workspace root
        .join("Cargo.lock");

    let lock_content = std::fs::read_to_string(&cargo_lock_path).unwrap_or_else(|e| {
        panic!(
            "RG-008: could not read Cargo.lock at {}: {}",
            cargo_lock_path.display(),
            e
        )
    });

    // Locate the [[package]] block for reqwest.
    // The Cargo.lock TOML format uses `[[package]]` entries, separated by blank lines.
    // Find the block starting with `name = "reqwest"`.
    let reqwest_block = lock_content
        .split("\n[[package]]")
        .find(|block| block.contains("name = \"reqwest\""))
        .unwrap_or_else(|| {
            panic!("RG-008: could not find [[package]] block for 'reqwest' in Cargo.lock")
        });

    // The reqwest block MUST contain "h2" in its dependencies list.
    // When the http2 feature is active, the block includes a line like:
    //   "h2 0.4.N (registry+...)",
    // (currently absent from reqwest's block — h2 is only under hyper's block).
    let has_h2_direct_dep = reqwest_block.contains("\"h2 ") || reqwest_block.contains(" \"h2\"");

    assert!(
        has_h2_direct_dep,
        "RG-008 (AC-H2-001): reqwest Cargo.lock block MUST list h2 as a direct dependency \
         to confirm the `http2` feature is active on the reqwest node (ADR-050 §D5). \
         Fix: add \"http2\" to the `features` list in THREE production reqwest \
         [dependencies] entries: prism-spec-engine/Cargo.toml, prism-sensors/Cargo.toml, \
         prism-bin/Cargo.toml (one production entry — the AC-9 shared client). \
         Also add \"http2\" to prism-bin/Cargo.toml [dev-dependencies] (which explicitly \
         lists `http2` too). Keep `default-features = false` and `rustls-tls` \
         (ADR-050 D1/D2 — native-tls stays forbidden). \
         Reqwest block contents:\n{reqwest_block}"
    );
}

// ---------------------------------------------------------------------------
// LOW-1 companion — production 401→auth_valid:false wire-shape (RG-007 variant)
// ---------------------------------------------------------------------------

/// LOW-1 companion for RG-007 (REWORKED OBS-5): `SensorHealthChecker::check_one` (production
/// path) MUST return a `SensorHealthResult` that serializes to JSON with `"reachable":true`
/// and `"auth_valid":false` for a 401 response.
///
/// REWORKED (OBS-5): the prior version used `probe_auth_with_routing` and then manually
/// re-implemented the `AuthStatus → Option<bool>` mapping, so the wire shape was not
/// load-bearing against a regression in `check_one`'s derivation. This version calls
/// `check_one` directly so the full production path is exercised:
///
/// `check_one` → `probe_auth_with_routing` → 401 → `AuthStatus::Invalid` →
/// `check_one` maps `Invalid → auth_valid_opt = Some(false)` →
/// `SensorHealthResult.with_auth_valid(false)` → `"auth_valid":false` in JSON.
///
/// BC-2.08.002 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOW-1 companion
#[tokio::test]
async fn test_sensor_health_wire_shape_401_auth_invalid_production_path() {
    let mock_server = MockServer::start().await;

    // wiremock 401 — triggers 401 → AuthStatus::Invalid path in check_one.
    Mock::given(method("GET"))
        .and(path("/api/v1/devices"))
        .respond_with(ResponseTemplate::new(401).set_body_bytes(b"unauthorized"))
        .mount(&mock_server)
        .await;

    let spec = make_xdome_spec(&mock_server.uri());
    let resolved = make_resolved(spec, "low1-org");
    let auth_strategy =
        AdapterAuthStrategy::Plugin(Arc::new(MockAuthProvider::new("low1-token"))
            as Arc<dyn prism_spec_engine::AuthProvider>);
    let http_client = build_http_client_with_timeout().expect("LOW-1: http client build failed");
    let adapter = Arc::new(SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        auth_strategy,
        http_client,
    ));

    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);
    let sensor_id = SensorId::from("xdome");

    // PRODUCTION PATH (OBS-5): SensorHealthChecker::check_one end-to-end so the
    // AuthStatus → Option<bool> mapping is exercised by production code, not re-implemented.
    // A regression (e.g., 401 → auth_valid=true) WOULD fail this test.
    let checker = SensorHealthChecker::new(Arc::new(registry));
    let context = PrismContext::new();
    let health = checker
        .check_one(org_id, "low1-org", &sensor_id, &context)
        .await;

    // SID-2 wire-shape assertion: exact JSON must contain the correct key:value pairs.
    let json = serde_json::to_string(&health).expect("LOW-1: SensorHealthResult must serialize");

    // ASSERTION 1: "reachable":true — sensor responded with 401 (Up, not Down).
    assert!(
        json.contains("\"reachable\":true"),
        "LOW-1 (SID-2 wire-shape): JSON must contain '\"reachable\":true' for 401 response. \
         Got JSON: {json}"
    );

    // ASSERTION 2: "auth_valid":false — driven by PRODUCTION check_one path.
    // Load-bearing: if check_one's AuthStatus::Invalid → Some(false) mapping regressed
    // (e.g., returned Some(true) for 401), this assertion would fail.
    assert!(
        json.contains("\"auth_valid\":false"),
        "LOW-1 (SID-2 wire-shape): JSON must contain '\"auth_valid\":false' for 401 response \
         via production check_one path. Got JSON: {json}. \
         Root cause if failing: check_one does not map AuthStatus::Invalid → Some(false) \
         for HTTP 401 responses (BC-2.08.002)."
    );
}

// ---------------------------------------------------------------------------
// RG-014 (end-to-end) — probe_connectivity returns Degraded + http_status=503 on 503
// ---------------------------------------------------------------------------

/// RG-014: `probe_connectivity` MUST return `ConnectivityStatus::Degraded` and
/// `http_status = Some(503)` when the sensor API returns HTTP 503.
///
/// Call chain: `probe_connectivity` → `SpecDrivenSensorAdapter::fetch()` →
/// `PipelineExecutor::execute()` → receives 503 → `SpecEngineError::HttpRequestFailed
/// { status_code: 503, .. }` → `map_spec_engine_error_to_sensor_error` →
/// `SensorError::HttpError { status: 503, .. }` → `probe_connectivity` classifies
/// as `ConnectivityStatus::Degraded` (status >= 500 branch in connectivity.rs).
///
/// Before this story, 5xx flowed to `SensorError::Internal` → catch-all →
/// `ConnectivityStatus::Down, http_status: None`. No test covered this path (F-P25-OBS-001).
///
/// This mirrors the RG-005 403→Up test pattern but for the 5xx→Degraded branch.
///
/// BC-2.08.002 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-014
#[tokio::test]
async fn test_probe_connectivity_503_returns_degraded() {
    let mock_server = MockServer::start().await;

    // Wiremock: any GET /api/v1/devices → 503 Service Unavailable.
    Mock::given(method("GET"))
        .and(path("/api/v1/devices"))
        .respond_with(ResponseTemplate::new(503).set_body_bytes(b"service unavailable"))
        .mount(&mock_server)
        .await;

    // Build and register the adapter (same fixture pattern as RG-005).
    let spec = make_xdome_spec(&mock_server.uri());
    let resolved = make_resolved(spec, "rg014-org");
    let auth_strategy =
        AdapterAuthStrategy::Plugin(Arc::new(MockAuthProvider::new("rg014-token"))
            as Arc<dyn prism_spec_engine::AuthProvider>);
    let http_client = build_http_client_with_timeout().expect("RG-014: http client build failed");
    let adapter = Arc::new(SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        auth_strategy,
        http_client,
    ));

    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);

    let sensor_id = SensorId::from("xdome");

    let outcome = probe_connectivity(&registry, org_id, &sensor_id, "rg014-client")
        .await
        .expect("RG-014: probe_connectivity must not return Err(PrismError)");

    // ASSERTION 1: status must be Degraded (5xx = server error, sensor reachable).
    // Before this story: SensorError::Internal → catch-all → Down, http_status: None.
    assert_eq!(
        outcome.status,
        ConnectivityStatus::Degraded,
        "RG-014: probe must return ConnectivityStatus::Degraded for 503 response. \
         Got: {:?}. \
         Root cause: map_spec_engine_error_to_sensor_error must route HttpRequestFailed \
         {{ status_code: 503 }} to SensorError::HttpError, which connectivity.rs then \
         classifies as Degraded (status >= 500). \
         (BC-2.08.002 DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-014).",
        outcome.status
    );

    // ASSERTION 2: http_status must be Some(503).
    // Before this story: Internal → catch-all → http_status: None.
    assert_eq!(
        outcome.http_status,
        Some(503),
        "RG-014: probe.http_status must be Some(503) for 503 response. \
         Got: {:?}. \
         Fix: SensorError::HttpError must carry status_code from HttpRequestFailed \
         (BC-2.08.002 DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-014).",
        outcome.http_status
    );
}

// ---------------------------------------------------------------------------
// CookieRoundtrip helper
// ---------------------------------------------------------------------------

/// Build a `SensorSpec` for sensor "cyberint" using `CookieRoundtrip` auth.
///
/// `CookieRoundtrip` acquires the API key from the credential store (no HTTP call)
/// and injects it as `Cookie: access_token={token}` on every data request.
/// When the data endpoint returns 401, the pipeline immediately surfaces
/// `SpecEngineError::CookieAuthFailed` (no retry — static-auth path,
/// BC-2.01.017 EC-017-002).
///
/// # Table naming: "devices" matches the probe sentinel
///
/// `SensorHealthChecker::check_one` resolves the probe source table via:
///   - `resolved_spec_map` lookup (production) — not wired in this test
///   - Legacy sentinel fallback: `{sensor_id}_devices` (both `probe_table` and
///     `first_table_name` are `None` when `resolved_spec_map` is absent)
///
/// `SpecDrivenSensorAdapter::fetch()` strips the `{sensor_id}_` prefix from
/// `spec.source_table` to find the matching table in the spec. Using table name
/// "devices" ensures `cyberint_devices` → strip → "devices" → table match →
/// the pipeline executes and issues the real HTTP request (driving the 401 path).
/// A table named "alerts" would silently return `Ok([])` without any HTTP call.
fn make_cyberint_spec(base_url: &str) -> SensorSpec {
    SensorSpec::new(
        "cyberint",
        "Cyberint Sensor (RG fixture)",
        AuthType::CookieRoundtrip,
        base_url,
        vec![TableSpec::new_point_in_time(
            "devices",
            "network_activity",
            vec![ColumnSpec::new(
                "device_id",
                ColumnType::String,
                None,
                vec![],
            )],
            vec![FetchStep::new(
                "fetch_devices",
                "GET",
                "/api/v1/devices",
                None,
                "$.items",
                None,
                vec![],
                None,
                None,
            )],
        )],
        None,
        "1.0.0",
        vec![],
    )
}

// ---------------------------------------------------------------------------
// F-P31-OBS-001 (SAP-3 arm reachability) — CookieRoundtrip 401→auth_invalid wire coverage
// ---------------------------------------------------------------------------

/// F-P31-OBS-001 / SAP-3 / BC-2.01.013 EC-01-029:
/// `SensorHealthChecker::check_one` (production path) MUST return a `SensorHealthResult`
/// that serializes to JSON with `"reachable":true` and `"auth_valid":false` when a
/// CookieRoundtrip-auth sensor receives HTTP 401 on its data endpoint.
///
/// This covers the CookieRoundtrip arm of the auth-invalid path. The only pre-existing
/// 401→auth_invalid wire test (`test_sensor_health_wire_shape_401_auth_invalid_production_path`)
/// uses `CustomViaPlugin` (`AuthRefreshFailed` error variant). This test drives the distinct
/// `CookieRoundtrip → CookieAuthFailed` variant — the actual path claroty uses — end-to-end.
///
/// Call chain (CookieRoundtrip 401 path):
/// 1. `check_one` → `probe_auth_with_routing` → `probe_connectivity`
/// 2. `probe_connectivity` → `SpecDrivenSensorAdapter::fetch()`
/// 3. `fetch()` → `PipelineExecutor::execute()` via `AdapterAuthStrategy::StaticCookie`
/// 4. `PipelineExecutor::acquire_token()` → `MockAuthProvider` returns token (no HTTP call)
/// 5. Pipeline injects `Cookie: access_token={mock-token}`, issues `GET /api/v1/devices`
/// 6. Mock server returns HTTP 401
/// 7. `CookieRoundtrip` discriminator fires → `SpecEngineError::CookieAuthFailed`
///    (BC-2.01.017 EC-017-002 — static-auth no-retry, NOT `AuthRefreshFailed`)
/// 8. `map_spec_engine_error_to_sensor_error` Arm 2 → `SensorError::HttpError { status: 401 }`
/// 9. `probe_connectivity` → `ConnectivityStatus::Up`
/// 10. `check_one` → `SensorHealthResult { reachable: true, auth_valid: false }`
///
/// Wire assertions (SID-2): `"reachable":true` and `"auth_valid":false` only.
/// No `detail`/`http_status`/`"401"` field asserted — outside the ratified
/// `SensorHealthResult` wire contract (BC-2.08.002 EC-08-006).
///
/// BC-2.01.013 EC-01-029 | BC-2.08.002 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 F-P31-OBS-001
#[tokio::test]
async fn test_BC_2_01_013_EC_01_029_cookie_roundtrip_401_auth_invalid_wire_shape() {
    let mock_server = MockServer::start().await;

    // CookieRoundtrip does NOT issue a login HTTP call — no POST /login mock needed.
    // StaticCookie: acquire_token() reads from MockAuthProvider (zero HTTP calls).
    // The data endpoint returns 401, driving CookieAuthFailed on the first request.
    // Path: /api/v1/devices — matches table name "devices" via probe sentinel
    // `cyberint_devices` (see make_cyberint_spec doc for the probe-sentinel rationale).
    Mock::given(method("GET"))
        .and(path("/api/v1/devices"))
        .respond_with(ResponseTemplate::new(401).set_body_bytes(b"unauthorized"))
        .mount(&mock_server)
        .await;

    let spec = make_cyberint_spec(&mock_server.uri());
    let resolved = make_resolved(spec, "cookie-401-org");

    // AdapterAuthStrategy::StaticCookie drives the CookieRoundtrip pipeline path.
    // MockAuthProvider substitutes for StaticCookieAuthProvider in test scope.
    let auth_strategy =
        AdapterAuthStrategy::StaticCookie(Arc::new(MockAuthProvider::new("cookie-test-api-key"))
            as Arc<dyn prism_spec_engine::AuthProvider>);
    let http_client =
        build_http_client_with_timeout().expect("F-P31-OBS-001: http client build failed");
    let adapter = Arc::new(SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        auth_strategy,
        http_client,
    ));

    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);
    let sensor_id = SensorId::from("cyberint");

    // PRODUCTION PATH: SensorHealthChecker::check_one end-to-end.
    // Exercises the full CookieRoundtrip arm:
    //   StaticCookie token acquisition (no HTTP) →
    //   GET /api/v1/devices with Cookie header →
    //   401 → CookieAuthFailed →
    //   map_spec_engine_error_to_sensor_error Arm 2 → HttpError{401} →
    //   probe_connectivity Up → check_one AuthStatus::Invalid → auth_valid=false.
    let checker = SensorHealthChecker::new(Arc::new(registry));
    let context = PrismContext::new();
    let health = checker
        .check_one(org_id, "cookie-401-org", &sensor_id, &context)
        .await;

    // SID-2 wire-shape assertion: exact JSON key:value pairs at the wire level.
    let json =
        serde_json::to_string(&health).expect("F-P31-OBS-001: SensorHealthResult must serialize");

    // ASSERTION 1: "reachable":true — sensor responded (401 = reachable, not a transport error).
    // If map_spec_engine_error_to_sensor_error regressed (CookieAuthFailed → SensorError::Internal
    // → probe classifies as Down → reachable=false), this assertion would fail and detect it.
    assert!(
        json.contains("\"reachable\":true"),
        "F-P31-OBS-001 (SID-2 wire-shape): JSON must contain '\"reachable\":true' for \
         CookieRoundtrip 401 response. Got JSON: {json}. \
         Root cause: CookieAuthFailed must reach map_spec_engine_error_to_sensor_error Arm 2 \
         (SensorError::HttpError{{status:401}}), not fall through to Arm 3 (SensorError::Internal). \
         (BC-2.01.013 EC-01-029, DEFECT-ADAPTER-TLS-XDOME-LIVE-001 F-P31-OBS-001)."
    );

    // ASSERTION 2: "auth_valid":false — driven by the full check_one production path.
    // Load-bearing: if check_one's AuthStatus::Invalid → Some(false) mapping regressed
    // for the CookieRoundtrip arm, this assertion would fail.
    assert!(
        json.contains("\"auth_valid\":false"),
        "F-P31-OBS-001 (SID-2 wire-shape): JSON must contain '\"auth_valid\":false' for \
         CookieRoundtrip 401 response via production check_one path. Got JSON: {json}. \
         (BC-2.01.013 EC-01-029, DEFECT-ADAPTER-TLS-XDOME-LIVE-001 F-P31-OBS-001)."
    );
}

// ---------------------------------------------------------------------------
// RG-019 — SensorHealthResult wire-shape: reachable=true for HTTP 5xx (Degraded)
// ---------------------------------------------------------------------------

/// RG-019: `SensorHealthChecker::check_one` MUST serialize HTTP 5xx (ConnectivityStatus::Degraded)
/// to the `check_sensor_health` MCP wire as `"reachable":true`, `"auth_valid":true`, and
/// `"error":"service_unavailable"`. MUST NOT contain `"reachable":false`.
///
/// # BC authority
///
/// - BC-2.08.002 v1.8 §Postconditions — 5xx Degraded wire contract (EC-08-009, HS-007 re-gate)
/// - AC-WIRE-002 — Degraded wire MUST contain `"reachable":true`, `"auth_valid":true`,
///   `"error":"service_unavailable"`, NOT `"reachable":false`
///
/// # Why `reachable:true` for 5xx
///
/// HTTP 5xx means: TCP connection succeeded, HTTP exchange occurred, the sensor IS reachable
/// at the network level. The service returned a server error — but the sensor IS reachable.
/// This is distinct from `ConnectivityStatus::Down` (no TCP/HTTP exchange — truly unreachable).
/// An LLM agent consuming `"reachable":true + "error":"service_unavailable"` correctly
/// infers "wait and retry" rather than "check network" (EC-08-009).
///
/// # Failure reason (genuine RED on current code)
///
/// `check_one` in `health/mod.rs` currently computes:
/// ```ignore
/// let reachable = probe.connectivity == ConnectivityStatus::Up;
/// ```
/// For `ConnectivityStatus::Degraded` (HTTP 5xx), this evaluates to `false` — producing
/// `"reachable":false` in the serialized wire, which is WRONG. The required fix is:
/// ```ignore
/// let reachable = probe.connectivity != ConnectivityStatus::Down;
/// ```
/// which yields `reachable:true` for both `Up` and `Degraded`, `reachable:false` only for `Down`.
///
/// RG-019 ASSERTION 1 (`"reachable":true`) and ASSERTION 4 (`!"reachable":false`) FAIL on the
/// current HEAD — genuine RED. ASSERTIONS 2 and 3 (`"auth_valid":true`, `"error":"service_unavailable"`)
/// already PASS on current HEAD (auth.rs correctly returns `AuthStatus::Valid` for 5xx, and
/// check_one already calls `with_error("service_unavailable")` for Degraded).
///
/// # Code path (5xx / Degraded)
///
/// 1. `probe_auth_with_routing` → `probe_connectivity` → `SpecDrivenSensorAdapter::fetch()`
/// 2. wiremock returns HTTP 503
/// 3. `SensorError::HttpError { status: 503 }` → connectivity.rs `status >= 500` arm →
///    `ConnectivityStatus::Degraded`, `http_status: Some(503)`
/// 4. auth.rs: `Degraded` arm → `http_status = Some(503)` (not 401/403) → `AuthStatus::Valid`
/// 5. `check_one` (current, BUGGY): `reachable = Degraded == Up` → `false` → `"reachable":false`
/// 6. `check_one` (after fix): `reachable = Degraded != Down` → `true` → `"reachable":true`
/// 7. `auth_valid_opt = Valid → Some(true)` → `"auth_valid":true`
/// 8. `with_error("service_unavailable")` → `"error":"service_unavailable"`
///
/// # SAP-3 compliance
///
/// Test reaches `check_one` end-to-end from the `SensorHealthChecker` public surface
/// (same pattern as RG-007/RG-015). No synthetic AST injection.
///
/// # Wire-assertion discipline (SID-2)
///
/// All assertions operate on the FULL serialized compact JSON output.
///
/// BC-2.08.002 v1.8 §Postconditions EC-08-009 | AC-WIRE-002 | HS-007 |
/// DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-019
#[tokio::test]
async fn test_BC_2_08_002_degraded_reachable_wire_shape() {
    let mock_server = MockServer::start().await;

    // Wiremock: GET /api/v1/devices → 503 Service Unavailable (drives ConnectivityStatus::Degraded).
    Mock::given(method("GET"))
        .and(path("/api/v1/devices"))
        .respond_with(ResponseTemplate::new(503).set_body_bytes(b"service unavailable"))
        .mount(&mock_server)
        .await;

    // Build and register the adapter (same fixture pattern as RG-007 / LOW-1).
    let spec = make_xdome_spec(&mock_server.uri());
    let resolved = make_resolved(spec, "rg019-org");
    let auth_strategy =
        AdapterAuthStrategy::Plugin(Arc::new(MockAuthProvider::new("rg019-token"))
            as Arc<dyn prism_spec_engine::AuthProvider>);
    let http_client = build_http_client_with_timeout().expect("RG-019: http client build failed");
    let adapter = Arc::new(SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        auth_strategy,
        http_client,
    ));

    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);
    let sensor_id = SensorId::from("xdome");

    // PRODUCTION PATH: SensorHealthChecker::check_one end-to-end.
    // Exercises the full 5xx path:
    //   GET /api/v1/devices → 503 → SensorError::HttpError{503} →
    //   connectivity.rs status>=500 → ConnectivityStatus::Degraded →
    //   auth.rs http_status=503 (not 401/403) → AuthStatus::Valid →
    //   check_one: reachable derivation (BUGGY: == Up → false; FIX: != Down → true)
    let checker = SensorHealthChecker::new(Arc::new(registry));
    let context = PrismContext::new();
    let health = checker
        .check_one(org_id, "rg019-org", &sensor_id, &context)
        .await;

    // SID-2 wire-shape assertion: serialize to compact JSON and check key:value pairs.
    let json = serde_json::to_string(&health).expect("RG-019: SensorHealthResult must serialize");

    // ASSERTION 1 (RED on current code): "reachable":true — TCP connection succeeded, HTTP
    // exchange occurred; sensor IS reachable at the network level. 5xx is NOT unreachable.
    //
    // FAILS on current HEAD: `reachable = connectivity == Up` → Degraded → false →
    // wire contains `"reachable":false` instead of `"reachable":true`.
    //
    // GREEN after fix: `reachable = connectivity != Down` → Degraded → true → `"reachable":true`.
    assert!(
        json.contains("\"reachable\":true"),
        "RG-019 EC-08-009 FAIL (SID-2 wire-shape, RED): JSON MUST contain '\"reachable\":true' \
         for HTTP 503 (ConnectivityStatus::Degraded) response per BC-2.08.002 v1.8 EC-08-009 (HS-007). \
         \nRoot cause (current code): `let reachable = probe.connectivity == ConnectivityStatus::Up` \
         in check_one yields `false` for Degraded — sensor IS reachable (TCP+HTTP exchange occurred). \
         \nRequired fix: `let reachable = probe.connectivity != ConnectivityStatus::Down`. \
         \nFull wire: {json}"
    );

    // ASSERTION 2 (GREEN on current code — regression guard): "auth_valid":true — HTTP 5xx is
    // NOT an auth rejection. auth.rs correctly maps http_status=503 (not 401/403) → AuthStatus::Valid.
    assert!(
        json.contains("\"auth_valid\":true"),
        "RG-019 EC-08-009 FAIL (SID-2 wire-shape): JSON MUST contain '\"auth_valid\":true' \
         for HTTP 503. 5xx is NOT an auth rejection (credentials were not refused). \
         auth.rs maps http_status=503 → AuthStatus::Valid → Some(true). \
         \nFull wire: {json}"
    );

    // ASSERTION 3 (GREEN on current code — regression guard): "error":"service_unavailable" —
    // check_one already calls `with_error("service_unavailable")` for Degraded probes.
    assert!(
        json.contains("\"error\":\"service_unavailable\""),
        "RG-019 EC-08-009 FAIL (SID-2 wire-shape): JSON MUST contain \
         '\"error\":\"service_unavailable\"' for HTTP 503 (Degraded) per BC-2.08.002 v1.8 EC-08-009. \
         check_one sets `result = result.with_error(\"service_unavailable\")` for Degraded. \
         \nFull wire: {json}"
    );

    // ASSERTION 4 (RED on current code — negative gate): MUST NOT contain "reachable":false.
    // `"reachable":false` is reserved for ConnectivityStatus::Down (no TCP/HTTP exchange at all).
    // A 5xx response (HTTP exchange DID occur) MUST NOT produce `"reachable":false`.
    //
    // FAILS on current HEAD: Degraded → reachable=false → wire contains `"reachable":false`.
    // GREEN after fix: Degraded → reachable=true → `"reachable":false` absent from wire.
    assert!(
        !json.contains("\"reachable\":false"),
        "RG-019 EC-08-009 FAIL (SID-2 negative gate, RED): JSON MUST NOT contain \
         '\"reachable\":false' for HTTP 503. `\"reachable\":false` is reserved for \
         ConnectivityStatus::Down (no TCP/HTTP exchange). A 5xx response means the HTTP \
         exchange DID occur — the sensor IS reachable. \
         \nRoot cause: `reachable = connectivity == Up` yields false for Degraded. \
         \nRequired fix: `reachable = connectivity != Down`. \
         \nFull wire: {json}"
    );
}

// ---------------------------------------------------------------------------
// RG-020 — check_sensor_health envelope: Degraded sensor MUST NOT be counted as healthy
// ---------------------------------------------------------------------------

/// RG-020: `check_sensor_health` for a Degraded (5xx) sensor MUST NOT count it as
/// healthy in the prose summary, `overall_status` MUST be `"partial"`, AND
/// `summary_counts.healthy_count` MUST be 0 (not 1).
///
/// # Assertion status (F-P43 strengthening)
///
/// - Assertions 1, 2, 3: GREEN (T-SERVER-1 fixed server.rs summary predicate).
/// - Assertions 4, 5 (NEW — F-P43): RED on current HEAD.
///   `resources.rs::HealthSummary::from_results` has the same predicate bug:
///   `reachable == Some(true) && auth_valid == Some(true) && rate_limit.is_none()`
///   without `&& error.is_none()` — Degraded sensor still reports as
///   `summary_counts.healthy_count=1` and `unhealthy_count=0`.
///
/// # Problems fixed / remaining
///
/// ## T-SERVER-1 (FIXED — commit 21df2f6d4)
///
/// `server.rs` `fully_healthy_count` predicate lacked `&& s.error.is_none()`.
/// Degraded (reachable=true, auth_valid=true, rate_limit=None, error="service_unavailable")
/// was miscounted as healthy → summary "1 of 1 sensor(s) healthy" contradicted
/// `overall_status: "partial"`. Fixed by adding the gate.
///
/// ## T-COUNTS-1 (NOT YET FIXED — drives assertions 4/5)
///
/// `resources.rs::HealthSummary::from_results` has the same structural bug:
/// ```ignore
/// let healthy_count = results.iter().filter(|r| {
///     r.reachable == Some(true) && r.auth_valid == Some(true) && r.rate_limit.is_none()
///     // MISSING: && r.error.is_none()   ← T-COUNTS-1 fix target
/// }).count();
/// ```
/// Wire: `"summary_counts":{"healthy_count":1,"unhealthy_count":0,...}` — WRONG.
/// Correct after fix: `"healthy_count":0,"unhealthy_count":1`.
///
/// # Wiring
///
/// Uses `PrismServer::new().with_query_engine(engine).with_health_checker(checker)`.
/// `engine` carries a `TableRegistry` with `"xdome"` registered, so `check_sensor_health`
/// enumerates `["xdome"]` via the single-tenant fallback path.
/// `probe_connectivity_inner` uses the nil-UUID fallback → `get_all_for_sensor("xdome")`
/// → finds the `SpecDrivenSensorAdapter` → wiremock returns 503 → `Degraded`.
///
/// # SAP-3 compliance
///
/// Test reaches `check_sensor_health` end-to-end from the `PrismServer` public surface.
///
/// # Wire-assertion discipline (SID-2)
///
/// All assertions operate on the FULL serialized `CallToolResult` JSON.
///
/// BC-2.08.002 v1.8 §Postconditions EC-08-009 | AC-WIRE-003 | AC-WIRE-004 |
/// HS-007 | T-SERVER-1 | T-COUNTS-1 |
/// DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-020
#[tokio::test]
async fn test_BC_2_08_002_degraded_envelope_summary_matches_overall_status() {
    use prism_credentials::InMemoryCredentialStore;
    use prism_mcp::server::{CheckSensorHealthParams, PrismServer};
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig},
        table_registry::TableRegistry,
    };
    use rmcp::handler::server::wrapper::Parameters;

    // Wiremock: GET /api/v1/devices → 503 Service Unavailable (ConnectivityStatus::Degraded).
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/devices"))
        .respond_with(ResponseTemplate::new(503).set_body_bytes(b"service unavailable"))
        .mount(&mock_server)
        .await;

    // Build adapter — same spec/resolved/auth pattern as RG-019.
    // make_xdome_spec called twice: once for make_resolved (moves spec), once for TableRegistry.
    let spec_for_registry = make_xdome_spec(&mock_server.uri());
    let spec_for_adapter = make_xdome_spec(&mock_server.uri());
    let resolved = make_resolved(spec_for_adapter, "rg020-org");
    let auth_strategy =
        AdapterAuthStrategy::Plugin(Arc::new(MockAuthProvider::new("rg020-token"))
            as Arc<dyn prism_spec_engine::AuthProvider>);
    let http_client = build_http_client_with_timeout().expect("RG-020: http client build failed");
    let adapter = Arc::new(SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        auth_strategy,
        http_client,
    ));

    let org_id = OrgId::new();
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, adapter);
    let adapter_registry = Arc::new(adapter_registry);

    // TableRegistry: register "xdome" so check_sensor_health enumerates it via the
    // single-tenant fallback path (query_engine.table_registry().registered_sensor_ids()).
    let table_registry = TableRegistry::new();
    table_registry
        .register_sensor(&spec_for_registry)
        .expect("RG-020: register xdome in TableRegistry");

    // QueryEngine wired with the table_registry for sensor ID enumeration.
    let engine = QueryEngine::new(
        Arc::clone(&adapter_registry),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
    .with_table_registry(Arc::new(table_registry));

    let checker = SensorHealthChecker::new(Arc::clone(&adapter_registry));

    // PrismServer with both query_engine (for sensor enumeration) and health_checker (live probe).
    // No OrgRegistry wired → single-tenant mode → org_id = Uuid::nil() sentinel →
    // probe_connectivity_inner falls back to get_all_for_sensor("xdome") → finds the adapter.
    let server = PrismServer::new()
        .with_query_engine(Arc::new(engine))
        .with_health_checker(checker);

    let params = CheckSensorHealthParams::for_client("rg020-org");
    let call_result = server
        .check_sensor_health(Parameters(params))
        .await
        .expect("RG-020: check_sensor_health MUST succeed for Degraded (5xx) sensor");

    // SID-2 wire-shape assertion: serialize CallToolResult to JSON.
    let json_str =
        serde_json::to_string(&call_result).expect("RG-020: CallToolResult must serialize to JSON");

    // ASSERTION 1 (GREEN — T-SERVER-1 fixed server.rs summary predicate):
    // Summary MUST NOT count Degraded sensor as healthy.
    assert!(
        !json_str.contains("1 of 1 sensor(s) healthy"),
        "RG-020 AC-WIRE-003 FAIL (SID-2): summary MUST NOT contain \
         '1 of 1 sensor(s) healthy' for a Degraded (5xx) sensor. \
         T-SERVER-1 should have fixed server.rs `fully_healthy_count`. Regression detected. \
         Full wire: {:.800}",
        json_str
    );

    // ASSERTION 2 (GREEN — T-SERVER-1 fixed server.rs summary predicate):
    // Summary MUST contain "0 of 1".
    assert!(
        json_str.contains("0 of 1"),
        "RG-020 AC-WIRE-003 FAIL (SID-2): summary MUST contain '0 of 1' — Degraded \
         sensor excluded from healthy count by T-SERVER-1 fix. Regression detected. \
         Full wire: {:.800}",
        json_str
    );

    // ASSERTION 3 (GREEN — T-WIRE-2 in mod.rs aggregate already applied):
    // overall_status MUST be "partial".
    assert!(
        json_str.contains(r#""overall_status":"partial""#)
            || json_str.contains(r#""overall_status": "partial""#),
        "RG-020 AC-WIRE-003 FAIL (SID-2): overall_status MUST be 'partial' for a Degraded \
         (5xx) fleet. T-WIRE-2 regression detected. Full wire: {:.800}",
        json_str
    );

    // ASSERTION 4 (RED on current HEAD — T-COUNTS-1 not yet applied):
    // summary_counts.healthy_count MUST be 0 for a single Degraded sensor.
    //
    // Current bug: `resources.rs::HealthSummary::from_results` uses the SAME predicate
    // as the old server.rs bug — lacks `&& r.error.is_none()`:
    //   filter(|r| r.reachable == Some(true) && r.auth_valid == Some(true) && r.rate_limit.is_none())
    // Degraded (reachable=true, auth_valid=true, rate_limit=None, error="service_unavailable")
    // satisfies the BUGGY predicate → healthy_count = 1. [WRONG]
    //
    // FAILS on current HEAD: wire contains "healthy_count":1.
    // GREEN after T-COUNTS-1 fix: add `&& r.error.is_none()` → Degraded excluded →
    // healthy_count = 0.
    assert!(
        json_str.contains(r#""healthy_count":0"#) || json_str.contains(r#""healthy_count": 0"#),
        "RG-020 AC-WIRE-004 FAIL (SID-2, RED): summary_counts.healthy_count MUST be 0 \
         for a Degraded (5xx) sensor. \
         Current bug: resources.rs HealthSummary::from_results predicate lacks \
         `&& r.error.is_none()` — Degraded (error=service_unavailable) miscounted as \
         healthy → healthy_count=1. Fix (T-COUNTS-1): add `&& r.error.is_none()` to \
         HealthSummary::from_results filter. \
         Full wire: {:.800}",
        json_str
    );

    // ASSERTION 5 (RED on current HEAD — T-COUNTS-1 not yet applied):
    // summary_counts.unhealthy_count MUST be 1 for a single Degraded sensor.
    //
    // Current bug: unhealthy_count = total_count.saturating_sub(healthy_count).
    // With healthy_count=1 (wrong), unhealthy_count = 1 - 1 = 0. [WRONG]
    //
    // FAILS on current HEAD: wire contains "unhealthy_count":0.
    // GREEN after T-COUNTS-1 fix: healthy_count = 0 → unhealthy_count = 1 - 0 = 1.
    assert!(
        json_str.contains(r#""unhealthy_count":1"#) || json_str.contains(r#""unhealthy_count": 1"#),
        "RG-020 AC-WIRE-004 FAIL (SID-2, RED): summary_counts.unhealthy_count MUST be 1 \
         for a single Degraded (5xx) sensor. \
         Current bug: unhealthy_count = total - healthy_count; with the buggy healthy_count=1, \
         unhealthy_count = 0. Fix (T-COUNTS-1): healthy_count=0 → unhealthy_count=1. \
         Full wire: {:.800}",
        json_str
    );
}
