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

/// RG-007: After calling `probe_connectivity`, the `SensorHealthResult` built
/// from the outcome MUST serialize to JSON containing `"reachable":true` and
/// `"auth_valid":false` for a 403 response.
///
/// This is the SID-2 wire-shape assertion: the exact JSON bytes an LLM agent
/// receives must contain the correct values — not just the Rust structs.
///
/// 403 semantics:
/// - `reachable = true` (server responded → sensor is up)
/// - `auth_valid = false` (403 Forbidden = auth failure or missing permission)
///
/// FAIL reason before fix: RG-005 probe returns Down (http_status: None) →
/// `reachable = (status == Up)` → false → `"reachable":false` in JSON →
/// `assert!(json.contains("\"reachable\":true"), ...)` panics → RED.
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

    let outcome = probe_connectivity(&registry, org_id, &sensor_id, "rg007-client")
        .await
        .expect("RG-007: probe_connectivity must not return Err(PrismError)");

    // Build SensorHealthResult from probe outcome.
    //
    // reachable = sensor responded (Up or Degraded)
    // auth_valid = false for 403 (server rejected credentials)
    let reachable =
        outcome.status == ConnectivityStatus::Up || outcome.status == ConnectivityStatus::Degraded;
    let auth_valid = outcome.http_status.map(|s| s < 400).unwrap_or(false);

    let health = SensorHealthResult::new("xdome", "rg007-client")
        .with_reachable(reachable)
        .with_auth_valid(auth_valid);

    // SID-2 wire-shape assertion: serialize to compact JSON and check key:value pairs.
    let json = serde_json::to_string(&health).expect("RG-007: SensorHealthResult must serialize");

    // ASSERTION 1: "reachable":true in JSON
    // FAIL before fix: probe returns Down → reachable=false → "reachable":false in JSON.
    assert!(
        json.contains("\"reachable\":true"),
        "RG-007 (SID-2 wire-shape): JSON must contain '\"reachable\":true' for 403 response. \
         Got JSON: {json}. \
         Root cause: probe returns Down before fix (http_status=None, Internal error). \
         Fix: map_spec_engine_error_to_sensor_error must return SensorError::HttpError for 403 \
         (BC-2.08.002 DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-007)."
    );

    // ASSERTION 2: "auth_valid":false in JSON
    // This should also fail before fix because auth_valid=false depends on knowing http_status=403;
    // if http_status is None (pre-fix), auth_valid=false anyway (false by default) — so this
    // assertion may pass even before fix. The critical gate is "reachable":true above.
    assert!(
        json.contains("\"auth_valid\":false"),
        "RG-007 (SID-2 wire-shape): JSON must contain '\"auth_valid\":false' for 403 response. \
         Got JSON: {json}."
    );
}

// ---------------------------------------------------------------------------
// RG-008 — reqwest http2 feature must not be enabled (regression guard)
// ---------------------------------------------------------------------------

/// RG-008: The `reqwest` dependency in `Cargo.lock` MUST NOT list `h2` as a
/// direct dependency (which would indicate the `http2` feature is enabled).
///
/// This is GREEN-BY-DESIGN today (reqwest does not have h2 as a direct dep
/// in the workspace Cargo.lock). It serves as a regression guard: if someone
/// adds `features = ["http2"]` to a reqwest entry, h2 would appear in
/// reqwest's Cargo.lock block and this test would fail.
///
/// Rationale: enabling HTTP/2 via reqwest:
/// - Reintroduces the h2/hyper TLS complexity that caused xDome live failures
/// - Requires TLS configuration changes that conflict with rustls-tls + rustls-tls-webpki
/// - Adds ~65s macOS Keychain init overhead on some platforms (ADR-050 D2)
///
/// Scope: checks ONLY the `[[package]] name = "reqwest"` Cargo.lock block —
/// NOT the whole file (h2 is present transitively via hyper, which is correct).
///
/// BC-2.16.014 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-008 (GREEN-BY-DESIGN regression guard)
#[test]
fn test_reqwest_http2_not_enabled() {
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

    // The reqwest block must NOT contain "h2" in its dependencies list.
    //
    // If http2 were enabled, the block would contain a line like:
    //   "h2 0.4.N (registry+...)",
    // Check for both quoted and unquoted forms.
    let has_h2_direct_dep = reqwest_block.contains("\"h2 ") || reqwest_block.contains(" \"h2\"");

    assert!(
        !has_h2_direct_dep,
        "RG-008 (regression guard): reqwest Cargo.lock block MUST NOT list h2 as a direct \
         dependency. This indicates the http2 feature is enabled somewhere in the workspace. \
         That conflicts with ADR-050 (rustls-tls only) and reintroduces the TLS fragility \
         that caused xDome live failures (DEFECT-ADAPTER-TLS-XDOME-LIVE-001). \
         Search all Cargo.toml files for 'http2' in reqwest features and remove it. \
         Reqwest block contents:\n{reqwest_block}"
    );
}
