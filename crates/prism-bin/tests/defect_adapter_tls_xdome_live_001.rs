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
//! | RG-008 | test_reqwest_http2_not_enabled (regression guard)          | GREEN-BY-DESIGN today; would fail if http2 feature added to reqwest dep.  |
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
/// GREEN after fix: `"http2"` added to all four production reqwest entries in
/// prism-spec-engine, prism-sensors, prism-bin (two entries) → Cargo resolves
/// h2 as a direct dep of reqwest → `h2` appears in reqwest block → passes.
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
         Fix: add \"http2\" to the `features` list in ALL FOUR production reqwest \
         [dependencies] entries: prism-spec-engine/Cargo.toml, prism-sensors/Cargo.toml, \
         prism-bin/Cargo.toml (two entries). Keep `default-features = false` and \
         `rustls-tls` (ADR-050 D1/D2 — native-tls stays forbidden). \
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
