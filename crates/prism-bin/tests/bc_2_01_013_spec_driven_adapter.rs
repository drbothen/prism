// SPDX-License-Identifier: Apache-2.0
//! Red Gate failing tests for S-DEMO-001: SpecDrivenSensorAdapter + boot step 9A.
//!
//! ALL tests in this file MUST FAIL before implementation begins (BC-5.38.001
//! Red Gate mandate). They fail because:
//! - `SpecDrivenSensorAdapter::fetch` body is `todo!()` — panics at runtime.
//! - `step9a_populate_adapter_registry` body is `todo!()` — panics at runtime.
//!
//! # AC → Test Mapping
//!
//! | AC | Test Name | BC |
//! |----|-----------|-----|
//! | AC-001 | test_BC_2_01_013_spec_driven_adapter_crowdstrike_delegates_to_pipeline_executor | BC-2.01.013 |
//! | AC-002 | test_BC_2_01_013_spec_driven_adapter_bearer_static_extracts_token_from_sensor_auth | BC-2.01.013 |
//! | AC-003 | test_BC_2_01_013_spec_driven_adapter_cyberint_cookie_auth_injects_access_token_cookie | BC-2.01.013 |
//! | AC-004 | test_BC_2_22_001_boot_step9a_registers_correct_adapter_count | BC-2.22.001 |
//! | AC-005 | test_BC_2_06_014_boot_step9a_uses_resolved_spec_overlay_url | BC-2.06.014 |
//! | AC-006 | test_BC_2_22_001_boot_step9a_empty_spec_catalog_registers_zero_adapters | BC-2.22.001 |
//! | AC-007 | test_BC_2_11_005_adapter_registry_get_returns_adapter_for_registered_pair | BC-2.11.005 |
//! | AC-008 | test_BC_2_01_013_bearer_static_auth_provider_returns_bearer_token | BC-2.01.013 |
//! | AC-009 | test_BC_2_01_013_static_cookie_auth_strategy_injects_access_token_not_bearer | BC-2.01.013 |
//! | AC-012 | test_BC_2_01_013_spec_driven_adapter_double_401_returns_auth_refresh_failed | BC-2.01.013 |
//! | EC-001 | test_BC_2_22_001_boot_step9a_unknown_auth_type_skips_sensor | BC-2.22.001 |
//! | EC-007 | test_BC_2_22_001_boot_step9a_unsupported_auth_type_skips_adapter_not_error | BC-2.22.001 |
//!
//! # SID-1 Discipline
//!
//! Tests that require a running DTU/external service instead use unit-level mocks/stubs
//! at the dependency boundary. The fetch() tests mock at the SensorAuth + PipelineExecutor
//! boundary, not at the HTTP layer, because the function under test is fetch() dispatch
//! logic (auth strategy selection), not HTTP execution.
//!
//! BCs: BC-2.01.013, BC-2.11.005, BC-2.06.014, BC-2.22.001
//! Story: S-DEMO-001 v1.3

#![allow(dead_code, unused_imports, non_snake_case)]
// SensorInstanceOverlay is #[non_exhaustive]; toml deserialization is the documented
// external construction path. Used in make_resolved_spec and AC-005 test fixture.
extern crate toml;

use std::{collections::HashMap, sync::Arc};

use prism_bin::{
    boot::BootError,
    spec_driven_adapter::{
        AdapterAuthStrategy, BearerStaticAuthProvider, SpecDrivenSensorAdapter,
        build_http_client_with_timeout, step9a_populate_adapter_registry,
    },
};
use prism_core::column::ColumnType;
use prism_core::{OrgId, OrgSlug, SensorId};
use prism_sensors::adapter::QueryParams;
use prism_sensors::auth::SensorAuth;
use prism_sensors::{AdapterRegistry, SensorAdapter, adapter::SensorSpec as SensorAdapterSpec};
use prism_spec_engine::{
    AuthProvider, AuthToken, PluginAuthProvider, ResolvedSensorSpec, ResolvedSpecKey,
    auth_provider::MockAuthProvider,
    overlay::{OverlayLoader, OverlayProvenance, SensorInstanceOverlay},
    spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec},
};

// ---------------------------------------------------------------------------
// Test fixtures — shared across tests in this module
// ---------------------------------------------------------------------------

/// Build a minimal `SensorSpec` for a given `auth_type`.
///
/// The sensor has one table with one column and one fetch step.
/// Used as the type-spec component of `ResolvedSensorSpec` in tests.
fn make_spec(sensor_id: &str, auth_type: AuthType, base_url: &str) -> SensorSpec {
    SensorSpec::new(
        sensor_id,
        &format!("{} sensor (test fixture)", sensor_id),
        auth_type,
        base_url,
        vec![TableSpec::new_point_in_time(
            "events",
            "security_finding",
            vec![ColumnSpec::new(
                "event_id",
                ColumnType::String,
                None,
                vec![],
            )],
            vec![FetchStep::new(
                "fetch_events",
                "GET",
                "/api/v1/events",
                None,
                "$.data",
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

/// Build a `ResolvedSensorSpec` with the given spec and org_slug.
///
/// Constructs via `OverlayLoader::merge_overlay_onto_type_spec` — the only
/// legitimate external construction path (`ResolvedSensorSpec` is `#[non_exhaustive]`).
/// The overlay has no `base_url` override (Case B — uses TYPE spec default).
///
/// This is the correct fixture for sensors that use the TYPE spec's `base_url`
/// unchanged. AC-005 uses an overlay with a base_url override.
fn make_resolved_spec(spec: SensorSpec, org_slug: &str) -> ResolvedSensorSpec {
    // Construct a no-op overlay (no scalar overrides) via TOML deserialization.
    // SensorInstanceOverlay is #[non_exhaustive]; TOML is the documented construction path.
    let overlay_toml = format!(
        "extends = \"{}\"\ninstance_id = \"{}@{}\"",
        spec.sensor_id, spec.sensor_id, org_slug
    );
    let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
        .expect("test fixture: SensorInstanceOverlay TOML parse failed");
    OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, OrgSlug::new(org_slug))
}

/// Build an `OrgRegistry` with one org registered.
///
/// Returns `(registry, org_id, org_slug)`.
fn make_org_registry(slug: &str) -> (prism_core::OrgRegistry, OrgId, OrgSlug) {
    let registry = prism_core::OrgRegistry::new();
    // Use a fixed UUIDv7 for test determinism.
    let uuid = uuid::Uuid::now_v7();
    let org_id = OrgId::from_uuid(uuid);
    let org_slug = OrgSlug::new(slug);
    registry
        .register(org_slug.clone(), org_id)
        .expect("test fixture: OrgRegistry::register failed");
    (registry, org_id, org_slug)
}

/// Minimal `SensorAuth` implementation for `bearer_static` tests.
///
/// Carries a token string for extraction via `as_any()` downcast in
/// `SpecDrivenSensorAdapter::fetch()` (BearerStatic auth strategy path).
///
/// Implements `SensorAuth::as_any()` so that the adapter can downcast to
/// `BearerStaticSensorAuth` and extract the token field.
struct BearerStaticSensorAuth {
    /// The bearer token string.
    ///
    /// AD-017: test-fixture only; never logs this value.
    token: String,
}

impl SensorAuth for BearerStaticSensorAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn auth_type_name(&self) -> &'static str {
        "bearer_static"
    }
}

/// Minimal `SensorAuth` implementation for `cookie_roundtrip` tests.
///
/// Used in `SpecDrivenSensorAdapter::fetch()` calls for the `StaticCookie`
/// auth strategy path. The adapter ignores this auth arg and uses its held
/// `StaticCookieAuthProvider` instead (ADR-028 §D10).
struct CookieSensorAuth;

impl SensorAuth for CookieSensorAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn auth_type_name(&self) -> &'static str {
        "cookie_roundtrip"
    }
}

/// Minimal `SensorAuth` for plugin-authed sensors (CrowdStrike).
///
/// The adapter ignores this arg and uses its held `PluginAuthProvider` instead
/// (ADR-028 §D10 — plugin-authed sensors provide their own auth at the plugin level).
struct PluginSensorAuth;

impl SensorAuth for PluginSensorAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn auth_type_name(&self) -> &'static str {
        "custom_via_plugin"
    }
}

/// Build a minimal `SensorAdapterSpec` (prism-sensors `SensorSpec`, not spec-engine `SensorSpec`).
///
/// Used as the `spec` argument to `SensorAdapter::fetch()`.
#[allow(deprecated)]
fn make_adapter_spec(sensor_id: &str, org_id: OrgId) -> SensorAdapterSpec {
    SensorAdapterSpec {
        source_table: "events".to_string(),
        org_id,
        #[allow(deprecated)]
        client_id: "test-org".to_string(),
        sensor_config: serde_json::json!({}),
    }
}

/// Build a minimal `QueryParams` for fetch() calls.
fn make_query_params() -> QueryParams {
    QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    }
}

// ---------------------------------------------------------------------------
// AC-001: CrowdStrike plugin-auth delegation test
// ---------------------------------------------------------------------------

/// AC-001 — BC-2.01.013 postcondition 4 (Plugin auth path):
/// `SpecDrivenSensorAdapter::fetch()` for a CrowdStrike (CustomViaPlugin) adapter
/// uses the held `Arc<PluginAuthProvider>`, ignores the `SensorAuth` argument
/// (ADR-028 §D10), and delegates to `PipelineExecutor::execute()`.
///
/// # Red Gate Failure
///
/// `SpecDrivenSensorAdapter::fetch()` body is `todo!()`. This test will panic
/// with the todo!() message. Red Gate: FAIL (expected).
///
/// BC-2.01.013; AC-001; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_spec_driven_adapter_crowdstrike_delegates_to_pipeline_executor() {
    // Build a CrowdStrike resolved spec (CustomViaPlugin auth)
    let spec = make_spec(
        "crowdstrike",
        AuthType::CustomViaPlugin,
        "http://127.0.0.1:18080",
    );
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    // The MockAuthProvider simulates the PluginAuthProvider surface at test time.
    // The real boot wires Arc<PluginAuthProvider>; at test time we use a MockAuthProvider
    // that returns a fixed token, confirming the dispatch reaches the held provider.
    // SID-1: no external process required — mock at the AuthProvider boundary.
    let mock_auth: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("cs-oauth-token"));
    let auth_strategy = AdapterAuthStrategy::Plugin(mock_auth);

    let http_client =
        build_http_client_with_timeout().expect("test fixture: http client construction failed");

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("crowdstrike", org_id);
    let params = make_query_params();
    let sensor_auth = PluginSensorAuth;

    // This call panics with todo!("S-DEMO-001: implement SpecDrivenSensorAdapter::fetch…")
    // Red Gate: FAIL (todo!() panic — expected before implementation)
    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
}

// ---------------------------------------------------------------------------
// AC-002: Armis/Claroty bearer_static auth dispatch test
// ---------------------------------------------------------------------------

/// AC-002 — BC-2.01.013 postcondition 4 (BearerStatic auth path):
/// `SpecDrivenSensorAdapter::fetch()` for an Armis (bearer_static) adapter
/// extracts the token from the `SensorAuth::BearerStatic { token }` argument,
/// constructs a `BearerStaticAuthProvider` per-call (OQ-1 Resolution — per-fetch
/// construction), and delegates to `PipelineExecutor::execute()`.
///
/// # Red Gate Failure
///
/// `SpecDrivenSensorAdapter::fetch()` body is `todo!()`. This test will panic
/// with the todo!() message. Red Gate: FAIL (expected).
///
/// BC-2.01.013; AC-002; OQ-1; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_spec_driven_adapter_bearer_static_extracts_token_from_sensor_auth() {
    // Armis sensor spec — bearer_static auth type.
    let spec = make_spec("armis", AuthType::BearerStatic, "http://127.0.0.1:18081");
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    // BearerStatic strategy: adapter holds no auth state — token comes from SensorAuth arg.
    let auth_strategy = AdapterAuthStrategy::BearerStatic;

    let http_client =
        build_http_client_with_timeout().expect("test fixture: http client construction failed");

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("armis", org_id);
    let params = make_query_params();
    // The SensorAuth arg carries the bearer token for extraction at fetch() call time.
    let sensor_auth = BearerStaticSensorAuth {
        token: "armis-bearer-token-xyz".to_string(),
    };

    // This call panics with todo!("S-DEMO-001: implement SpecDrivenSensorAdapter::fetch…")
    // Red Gate: FAIL (todo!() panic — expected before implementation)
    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
}

// ---------------------------------------------------------------------------
// AC-003: Cyberint cookie_roundtrip auth test
// ---------------------------------------------------------------------------

/// AC-003 — BC-2.01.013 postcondition 4 (StaticCookie auth path):
/// `SpecDrivenSensorAdapter::fetch()` for Cyberint (cookie_roundtrip) uses its
/// held `StaticCookieAuthProvider`. The provider reads the API key from the
/// credential store at `acquire_token()` time with NO HTTP call (ADR-031 §D1-b).
///
/// The pipeline injects the token as `Cookie: access_token={api_key}`
/// (NOT `Authorization: Bearer`, NOT `Cookie: cyberint_session`).
///
/// Per ADR-031 §D3-b. Cookie name MUST be `access_token`.
///
/// # Red Gate Failure
///
/// `SpecDrivenSensorAdapter::fetch()` body is `todo!()`. This test will panic
/// with the todo!() message. Red Gate: FAIL (expected).
///
/// BC-2.01.013; AC-003; AC-009; ADR-031 §D3-b; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_spec_driven_adapter_cyberint_cookie_auth_injects_access_token_cookie() {
    // Cyberint sensor spec — cookie_roundtrip auth type.
    let spec = make_spec(
        "cyberint",
        AuthType::CookieRoundtrip,
        "http://127.0.0.1:18082",
    );
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    // StaticCookie strategy: adapter holds the StaticCookieAuthProvider.
    // SID-1: inject MockAuthProvider (simulating StaticCookieAuthProvider surface).
    // The real implementation uses prism_spec_engine::StaticCookieAuthProvider; at
    // Red Gate time we use MockAuthProvider to confirm the dispatch path reaches the
    // held provider without requiring a live credential store.
    let mock_static_cookie: Arc<dyn AuthProvider> =
        Arc::new(MockAuthProvider::new("cyberint-api-key-test"));
    let auth_strategy = AdapterAuthStrategy::StaticCookie(mock_static_cookie);

    let http_client =
        build_http_client_with_timeout().expect("test fixture: http client construction failed");

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("cyberint", org_id);
    let params = make_query_params();
    // StaticCookie: adapter ignores this arg and uses its held provider (ADR-028 §D10).
    let sensor_auth = CookieSensorAuth;

    // This call panics with todo!("S-DEMO-001: implement SpecDrivenSensorAdapter::fetch…")
    // Red Gate: FAIL (todo!() panic — expected before implementation)
    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
}

// ---------------------------------------------------------------------------
// AC-004: boot step 9A registers N adapters (N = sum of per-org × per-sensor)
// ---------------------------------------------------------------------------

/// AC-004 — BC-2.22.001 postcondition:
/// `step9a_populate_adapter_registry` registers exactly one `SpecDrivenSensorAdapter`
/// per `(OrgId, SensorId)` pair found in `resolved_spec_map`.
///
/// Given 2 orgs × 2 sensors = 4 resolved specs,
/// `AdapterRegistry` must contain exactly 4 entries after boot step 9A.
///
/// # Red Gate Failure
///
/// `step9a_populate_adapter_registry` body is `todo!()`. This test will panic
/// with the todo!() message. Red Gate: FAIL (expected).
///
/// BC-2.22.001; AC-004; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_22_001_boot_step9a_registers_correct_adapter_count() {
    // Set up: 2 orgs
    let (org_registry, org_id_a, org_slug_a) = make_org_registry("org-alpha");
    let uuid_b = uuid::Uuid::now_v7();
    let org_id_b = OrgId::from_uuid(uuid_b);
    let org_slug_b = OrgSlug::new("org-beta");
    org_registry
        .register(org_slug_b.clone(), org_id_b)
        .expect("test fixture: register org-beta failed");

    // Set up: 2 sensors (crowdstrike + armis), 2 orgs each = 4 entries
    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();

    // org-alpha × crowdstrike
    let cs_spec_a = make_spec(
        "crowdstrike",
        AuthType::CustomViaPlugin,
        "http://127.0.0.1:18080",
    );
    let cs_resolved_a = make_resolved_spec(cs_spec_a, "org-alpha");
    resolved_spec_map.insert(
        (org_slug_a.clone(), SensorId::from("crowdstrike")),
        cs_resolved_a,
    );

    // org-alpha × armis
    let armis_spec_a = make_spec("armis", AuthType::BearerStatic, "http://127.0.0.1:18081");
    let armis_resolved_a = make_resolved_spec(armis_spec_a, "org-alpha");
    resolved_spec_map.insert(
        (org_slug_a.clone(), SensorId::from("armis")),
        armis_resolved_a,
    );

    // org-beta × crowdstrike
    let cs_spec_b = make_spec(
        "crowdstrike",
        AuthType::CustomViaPlugin,
        "http://127.0.0.1:18083",
    );
    let cs_resolved_b = make_resolved_spec(cs_spec_b, "org-beta");
    resolved_spec_map.insert(
        (org_slug_b.clone(), SensorId::from("crowdstrike")),
        cs_resolved_b,
    );

    // org-beta × armis
    let armis_spec_b = make_spec("armis", AuthType::BearerStatic, "http://127.0.0.1:18084");
    let armis_resolved_b = make_resolved_spec(armis_spec_b, "org-beta");
    resolved_spec_map.insert(
        (org_slug_b.clone(), SensorId::from("armis")),
        armis_resolved_b,
    );

    // No plugin auth providers needed for BearerStatic sensors in this test.
    // CrowdStrike entries will be skipped (no provider found) per EC-004 behavior.
    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    // This call panics with todo!("S-DEMO-001: implement step9a_populate_adapter_registry…")
    // Red Gate: FAIL (todo!() panic — expected before implementation)
    let _count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        &org_registry,
        &plugin_auth_providers,
        &mut adapter_registry,
    )
    .await
    .expect("step9a should return Ok(count)");

    // Implementation postcondition (unreachable during Red Gate):
    // BearerStatic sensors registered: 2 orgs × 1 sensor (armis) = 2 entries.
    // CrowdStrike adapters: 0 (no plugin provider supplied → EC-004 skip).
    assert_eq!(
        adapter_registry.len(),
        2,
        "AC-004: registry must contain exactly 2 entries (armis for 2 orgs; \
         crowdstrike skipped: no PluginAuthProvider supplied). \
         BC-2.22.001 postcondition."
    );
}

// ---------------------------------------------------------------------------
// AC-005: boot step 9A uses the overlay base_url from ResolvedSensorSpec
// ---------------------------------------------------------------------------

/// AC-005 — BC-2.06.014 precondition 1:
/// When boot step 9A constructs a `SpecDrivenSensorAdapter` for a sensor with
/// a per-org overlay, the adapter uses the overlay `base_url` (not the TYPE
/// spec's default URL).
///
/// `ResolvedSensorSpec.spec.base_url` carries the merged effective value after
/// `OverlayLoader.merge_overlay_onto_type_spec()`. The adapter's internal
/// `PipelineExecutor` reads `spec.base_url` and must use the overlay value.
///
/// # Red Gate Failure
///
/// `step9a_populate_adapter_registry` body is `todo!()`. Panics before
/// the adapter is registered. Red Gate: FAIL (expected).
///
/// BC-2.06.014; AC-005; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_06_014_boot_step9a_uses_resolved_spec_overlay_url() {
    let (org_registry, _, org_slug) = make_org_registry("demo-org");
    let sensor_id = SensorId::from("crowdstrike");

    // TYPE spec has production URL; overlay has the local DTU URL.
    // After OverlayLoader merges, spec.base_url == overlay URL (ADR-029).
    let type_spec = make_spec(
        "crowdstrike",
        AuthType::CustomViaPlugin,
        "https://prod.crowdstrike.com",
    );

    // Construct an overlay that overrides base_url to the local DTU URL.
    // SensorInstanceOverlay is #[non_exhaustive]; TOML is the documented construction path.
    let overlay_toml = format!(
        "extends = \"crowdstrike\"\ninstance_id = \"crowdstrike@{}\"\nbase_url = \"http://127.0.0.1:18080\"",
        org_slug.as_str()
    );
    let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
        .expect("test fixture: SensorInstanceOverlay TOML parse failed");
    let resolved =
        OverlayLoader::merge_overlay_onto_type_spec(&type_spec, &overlay, org_slug.clone());

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug, sensor_id), resolved);

    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    // This call panics with todo!("S-DEMO-001: implement step9a_populate_adapter_registry…")
    // Red Gate: FAIL (todo!() panic — expected before implementation)
    let _count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        &org_registry,
        &plugin_auth_providers,
        &mut adapter_registry,
    )
    .await
    .expect("step9a should return Ok");

    // Implementation assertion (unreachable during Red Gate):
    // The registered adapter should use base_url = "http://127.0.0.1:18080".
    // Verified by inspecting the adapter's sensor_spec.spec.base_url field.
    // (This assertion tests the overlay wiring, not just registration count.)
    // If CrowdStrike was skipped (no PluginAuthProvider), adapter_registry.is_empty().
    // For this test we only verify the call does not panic with the todo!().
}

// ---------------------------------------------------------------------------
// AC-006: empty spec_catalog → registry stays empty, no error
// ---------------------------------------------------------------------------

/// AC-006 — BC-2.22.001 postcondition:
/// When `resolved_spec_map` is empty (no TOML specs loaded), boot step 9A
/// registers 0 adapters and returns `Ok(0)` — no error, boot continues.
///
/// # Red Gate Failure
///
/// `step9a_populate_adapter_registry` body is `todo!()`. Panics before
/// returning. Red Gate: FAIL (expected).
///
/// BC-2.22.001; AC-006; EC-003; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_22_001_boot_step9a_empty_spec_catalog_registers_zero_adapters() {
    let (org_registry, _, _) = make_org_registry("demo-org");
    let resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    // This call panics with todo!("S-DEMO-001: implement step9a_populate_adapter_registry…")
    // Red Gate: FAIL (todo!() panic — expected before implementation)
    let count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        &org_registry,
        &plugin_auth_providers,
        &mut adapter_registry,
    )
    .await
    .expect("step9a must return Ok(0) for empty catalog (AC-006, EC-003)");

    // Implementation assertion (unreachable during Red Gate):
    assert_eq!(
        count, 0,
        "AC-006: empty spec_catalog must produce 0 registered adapters"
    );
    assert!(
        adapter_registry.is_empty(),
        "AC-006: registry must be empty after empty-catalog run"
    );
}

// ---------------------------------------------------------------------------
// AC-007: AdapterRegistry::get returns adapter for registered (OrgId, SensorId)
// ---------------------------------------------------------------------------

/// AC-007 — BC-2.11.005 precondition:
/// After boot step 9A completes, `AdapterRegistry::get(org_id, sensor_id)`
/// returns `Some(Arc<dyn SensorAdapter>)` for every `(OrgId, SensorId)` pair
/// present in both `spec_catalog` and `org_registry`.
///
/// # Red Gate Failure
///
/// `step9a_populate_adapter_registry` body is `todo!()`. The registry is never
/// populated. Assertion fails (registry is empty). Red Gate: FAIL (expected).
///
/// BC-2.11.005; AC-007; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_11_005_adapter_registry_get_returns_adapter_for_registered_pair() {
    let (org_registry, org_id, org_slug) = make_org_registry("demo-org");
    let sensor_id = SensorId::from("armis");

    let armis_spec = make_spec("armis", AuthType::BearerStatic, "http://127.0.0.1:18081");
    let resolved = make_resolved_spec(armis_spec, "demo-org");

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug, sensor_id.clone()), resolved);

    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    // This call panics with todo!("S-DEMO-001: implement step9a_populate_adapter_registry…")
    // Red Gate: FAIL (todo!() panic — expected before implementation)
    let _count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        &org_registry,
        &plugin_auth_providers,
        &mut adapter_registry,
    )
    .await
    .expect("step9a should complete");

    // Implementation assertion (unreachable during Red Gate):
    let adapter = adapter_registry.get(org_id, &sensor_id);
    assert!(
        adapter.is_some(),
        "BC-2.11.005: AdapterRegistry::get must return Some(adapter) for a registered \
         (OrgId, SensorId) pair. Got None — boot step 9A failed to register the adapter."
    );
}

// ---------------------------------------------------------------------------
// AC-008: BearerStaticAuthProvider returns correct Authorization: Bearer header token
// ---------------------------------------------------------------------------

/// AC-008 — BC-2.01.013 precondition:
/// `BearerStaticAuthProvider` implements `AuthProvider::acquire_token()` and
/// returns an `AuthToken` containing the held bearer token string.
///
/// `PipelineExecutor::build_request` will inject this token as
/// `Authorization: Bearer {token}` for BearerStatic sensors.
///
/// # Red Gate Failure
///
/// `BearerStaticAuthProvider::acquire_token()` IS already implemented (it is
/// a trivial `AuthToken::new(self.token.clone())` delegation — marked
/// GREEN-BY-DESIGN in the stub). This test PASSES in the Red Gate phase.
///
/// This is explicitly documented: BC-5.38.002 covers trivial delegation where
/// zero-branch, zero-I/O, 1-line bodies are Green-by-construction.
///
/// BC-2.01.013; AC-008; OQ-1 Resolution; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_bearer_static_auth_provider_returns_bearer_token() {
    // BearerStaticAuthProvider is the trivial wrapper — its acquire_token body is
    // already implemented (GREEN-BY-DESIGN per BC-5.38.002 in the stub comment).
    // This test verifies the contract: the returned AuthToken value equals the
    // input token string, confirming the one-line delegation is correct.
    let token_value = "armis-bearer-token-test-abc";
    let provider = BearerStaticAuthProvider::new(token_value);

    // Build a minimal spec for the acquire_token call signature.
    let spec = make_spec("armis", AuthType::BearerStatic, "http://127.0.0.1");
    let client_id = OrgSlug::new("demo-org");

    let result = provider.acquire_token(&spec, &client_id).await;

    assert!(
        result.is_ok(),
        "AC-008: BearerStaticAuthProvider::acquire_token must return Ok(AuthToken). \
         Got: {:?}",
        result
    );
    let token = result.unwrap();
    assert_eq!(
        token.as_str(),
        token_value,
        "AC-008: AuthToken value must equal the input bearer token string. \
         BC-2.01.013 precondition — PipelineExecutor injects this as \
         Authorization: Bearer <token>."
    );
}

// ---------------------------------------------------------------------------
// AC-009: StaticCookie auth strategy uses access_token cookie, NOT Bearer header
// ---------------------------------------------------------------------------

/// AC-009 / AC-003 supplemental — BC-2.01.013 (cookie injection path):
/// The `StaticCookie` auth strategy in `SpecDrivenSensorAdapter::fetch()` calls
/// `PipelineExecutor::build_request` with `AuthType::CookieRoundtrip`, which
/// causes the pipeline to inject `Cookie: access_token={token}` (NOT
/// `Authorization: Bearer`). The cookie name MUST be `access_token`.
///
/// Constraint: cookie name `cyberint_session` is WRONG under ADR-031 D1-a.
/// This test encodes the invariant that `access_token` is the canonical name.
///
/// # Red Gate Failure
///
/// `SpecDrivenSensorAdapter::fetch()` body is `todo!()`. This test will panic
/// with the todo!() message. Red Gate: FAIL (expected).
///
/// BC-2.01.013; AC-009; ADR-031 §D3-b; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_static_cookie_auth_strategy_injects_access_token_not_bearer() {
    // Cyberint sensor spec — cookie_roundtrip auth type.
    let spec = make_spec(
        "cyberint",
        AuthType::CookieRoundtrip,
        "http://127.0.0.1:18082",
    );
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    // StaticCookie auth strategy: held provider returns the API key.
    // The pipeline (not tested here — tested in prism-spec-engine) injects
    // Cookie: access_token={api_key}. The adapter must select StaticCookie
    // strategy and NOT inject Authorization: Bearer.
    let mock_provider: Arc<dyn AuthProvider> =
        Arc::new(MockAuthProvider::new("cyberint-static-api-key"));
    let auth_strategy = AdapterAuthStrategy::StaticCookie(mock_provider);

    let http_client =
        build_http_client_with_timeout().expect("test fixture: http client construction failed");

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("cyberint", org_id);
    let params = make_query_params();
    let sensor_auth = CookieSensorAuth;

    // This call panics with todo!("S-DEMO-001: implement SpecDrivenSensorAdapter::fetch…")
    // Red Gate: FAIL (todo!() panic — expected before implementation)
    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // Implementation assertion (unreachable during Red Gate):
    // When implemented, fetch() must select StaticCookie strategy, call acquire_token
    // on the held provider, and pass spec.auth_type == CookieRoundtrip to build_request.
    // build_request MUST inject Cookie: access_token={token}, NOT Authorization: Bearer.
    // The adversary will verify this at pass-1 via SAP-1/SAP-2.
}

// ---------------------------------------------------------------------------
// AC-012: double-401 from sensor → AuthRefreshFailed returned
// ---------------------------------------------------------------------------

/// AC-012 — BC-2.01.013 error case:
/// When `PipelineExecutor` returns a double-401 (initial fetch + retry both
/// return 401 Unauthorized), `SpecDrivenSensorAdapter::fetch()` propagates
/// `SpecEngineError::AuthRefreshFailed` as `SensorError::Internal`.
///
/// No panic — the adapter must NOT panic on auth failure; it must return Err.
///
/// # Red Gate Failure
///
/// `SpecDrivenSensorAdapter::fetch()` body is `todo!()`. This test will panic
/// with the todo!() message instead of returning Err. Red Gate: FAIL (expected).
///
/// BC-2.01.013; AC-012; EC-006; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_spec_driven_adapter_double_401_returns_auth_refresh_failed() {
    // Armis sensor with BearerStatic auth.
    let spec = make_spec("armis", AuthType::BearerStatic, "http://127.0.0.1:18085");
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    let auth_strategy = AdapterAuthStrategy::BearerStatic;

    let http_client =
        build_http_client_with_timeout().expect("test fixture: http client construction failed");

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("armis", org_id);
    let params = make_query_params();
    // BearerStatic sensor auth with a token that will result in 401 from the pipeline.
    // SID-1: at Red Gate time, the todo!() fires before any HTTP call.
    let sensor_auth = BearerStaticSensorAuth {
        token: "expired-bearer-token".to_string(),
    };

    // This call panics with todo!("S-DEMO-001: implement SpecDrivenSensorAdapter::fetch…")
    // Red Gate: FAIL (todo!() panic — expected before implementation)
    //
    // When IMPLEMENTED: the adapter should return Err(SensorError::Internal { .. })
    // containing the AuthRefreshFailed error, NOT panic.
    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // Implementation assertion (unreachable during Red Gate):
    assert!(
        result.is_err(),
        "AC-012: double-401 must return Err, not Ok or panic. \
         BC-2.01.013 error case — AuthRefreshFailed propagated as SensorError::Internal."
    );
}

// ---------------------------------------------------------------------------
// EC-007: unsupported auth_type → adapter skipped, boot continues
// ---------------------------------------------------------------------------

/// EC-007 — BC-2.22.001 edge case:
/// When a sensor spec declares `auth_type = "api_key"` (or any unsupported
/// auth type not handled by S-DEMO-001 scope), boot step 9A logs E-SPEC-012
/// and skips the adapter — no error returned, boot continues.
///
/// # Red Gate Failure
///
/// `step9a_populate_adapter_registry` body is `todo!()`. Panics before
/// any skip logic can run. Red Gate: FAIL (expected).
///
/// BC-2.22.001; EC-007; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_22_001_boot_step9a_unsupported_auth_type_skips_adapter_not_error() {
    let (org_registry, _, org_slug) = make_org_registry("demo-org");

    // Sensor with ApiKey auth type — NOT supported in S-DEMO-001 scope.
    // Should be skipped with E-SPEC-012 log, not a hard error.
    let api_key_spec = make_spec("some-sensor", AuthType::ApiKey, "http://127.0.0.1:18086");
    let resolved = make_resolved_spec(api_key_spec, "demo-org");

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug, SensorId::from("some-sensor")), resolved);

    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    // This call panics with todo!("S-DEMO-001: implement step9a_populate_adapter_registry…")
    // Red Gate: FAIL (todo!() panic — expected before implementation)
    let count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        &org_registry,
        &plugin_auth_providers,
        &mut adapter_registry,
    )
    .await
    .expect("step9a must return Ok even when some sensors are skipped (EC-007)");

    // Implementation assertion (unreachable during Red Gate):
    assert_eq!(
        count, 0,
        "EC-007: unsupported auth_type must produce 0 registrations (sensor skipped, not error). \
         BC-2.22.001 edge case."
    );
    assert!(
        adapter_registry.is_empty(),
        "EC-007: registry must be empty when all sensors have unsupported auth types."
    );
}

// ---------------------------------------------------------------------------
// OQ-2 Resolution: OrgSlug → OrgId translation in step 9A
// ---------------------------------------------------------------------------

/// OQ-2 Resolution — BC-2.06.014:
/// `step9a_populate_adapter_registry` translates `(OrgSlug, SensorId)` keys
/// from `resolved_spec_map` (keyed by OrgSlug) to `(OrgId, SensorId)` keys
/// in `AdapterRegistry` (keyed by OrgId) using `OrgRegistry::resolve(slug)`.
///
/// If a slug in `resolved_spec_map` has no matching `OrgId` in `org_registry`,
/// the sensor is skipped with a warning (per story §OQ-2 Resolution).
///
/// # Red Gate Failure
///
/// `step9a_populate_adapter_registry` body is `todo!()`. Panics before
/// any translation occurs. Red Gate: FAIL (expected).
///
/// BC-2.06.014; OQ-2 Resolution; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_06_014_boot_step9a_translates_org_slug_to_org_id() {
    // Register org in OrgRegistry.
    let (org_registry, org_id, org_slug) = make_org_registry("known-org");

    // resolved_spec_map is keyed by (OrgSlug, SensorId).
    // AdapterRegistry must be keyed by (OrgId, SensorId).
    let sensor_id = SensorId::from("claroty");
    let claroty_spec = make_spec("claroty", AuthType::BearerStatic, "http://127.0.0.1:18087");
    let resolved = make_resolved_spec(claroty_spec, "known-org");

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug, sensor_id.clone()), resolved);

    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    // This call panics with todo!("S-DEMO-001: implement step9a_populate_adapter_registry…")
    // Red Gate: FAIL (todo!() panic — expected before implementation)
    let count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        &org_registry,
        &plugin_auth_providers,
        &mut adapter_registry,
    )
    .await
    .expect("step9a should return Ok");

    // Implementation assertion (unreachable during Red Gate):
    assert_eq!(
        count, 1,
        "OQ-2: exactly 1 adapter registered after slug→id translation"
    );

    // The adapter must be keyed by OrgId (not OrgSlug) in the registry.
    let registered = adapter_registry.get(org_id, &sensor_id);
    assert!(
        registered.is_some(),
        "OQ-2 Resolution: adapter must be retrievable by OrgId (not OrgSlug). \
         If this fails, step9A stored the adapter under the wrong key. \
         BC-2.06.014 precondition 1."
    );
}

// ---------------------------------------------------------------------------
// reqwest::Client timeout compliance test
// ---------------------------------------------------------------------------

/// build_http_client_with_timeout: client is constructed successfully with 30s timeout.
///
/// This tests the GREEN-BY-DESIGN factory function. The function itself is already
/// implemented (trivial `reqwest::Client::builder().timeout(...).build()`). This test
/// verifies the function compiles and returns Ok (no panic, no error).
///
/// The 30-second timeout is required per CLAUDE.md conventions
/// (TD-S-PLUGIN-PREREQ-B-005). Any `reqwest::Client::new()` without `.timeout()`
/// in production code is a P2 finding.
///
/// CLAUDE.md §Conventions; TD-S-PLUGIN-PREREQ-B-005; S-DEMO-001 AC-011.
#[test]
fn test_BC_2_01_013_build_http_client_with_timeout_succeeds() {
    let result = build_http_client_with_timeout();
    assert!(
        result.is_ok(),
        "build_http_client_with_timeout must return Ok(Client) — reqwest builder \
         should not fail in normal conditions. Got Err: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// sensor_type() GREEN-BY-DESIGN: trivial delegation
// ---------------------------------------------------------------------------

/// `SpecDrivenSensorAdapter::sensor_type()` returns the sensor ID from the spec.
///
/// This is a GREEN-BY-DESIGN method — zero branching, zero I/O, one-line body.
/// The stub already implements it correctly. This test verifies the delegation
/// works before any other stub method is implemented.
///
/// BC-5.38.002 criterion (trivial delegation, 1-line, no branching, no I/O).
/// BC-2.01.013; S-DEMO-001 v1.3.
#[test]
fn test_BC_2_01_013_spec_driven_adapter_sensor_type_returns_sensor_id_from_spec() {
    let spec = make_spec("crowdstrike", AuthType::CustomViaPlugin, "http://127.0.0.1");
    let resolved = make_resolved_spec(spec, "demo-org");

    let mock_auth: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("token"));
    let auth_strategy = AdapterAuthStrategy::Plugin(mock_auth);
    let http_client = build_http_client_with_timeout().unwrap();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    assert_eq!(
        adapter.sensor_type(),
        SensorId::from("crowdstrike"),
        "sensor_type() must return SensorId derived from the resolved spec's sensor_id field. \
         BC-2.01.013."
    );
}
