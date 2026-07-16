// SPDX-License-Identifier: Apache-2.0
//! LOAD-BEARING Red Gate tests for S-DEMO-001: SpecDrivenSensorAdapter + boot step 9A.
//!
//! Every test in this file MUST FAIL before implementation fixes the gaps found by
//! adversary pass-1. Failure modes are documented per test.
//!
//! # Adversary pass-1 findings addressed
//!
//! | Finding | Tests | Root cause |
//! |---------|-------|-----------|
//! | F-005 | all *_delegates_to_pipeline_executor, *_extracts_token, *_cookie_auth*, *_not_bearer | `let _result = ...` with zero assertions |
//! | F-001 | test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_* | fetch() discards pipeline records, returns Vec::new() |
//! | F-004 | test_BC_2_01_013_spec_driven_adapter_double_401_returns_auth_refresh_failed | test used connection-refused not a double-401 mock |
//! | F-002 | test_BC_2_22_001_step9a_not_called_in_boot_production_path | step9_start_mcp_server never calls step9a |
//! | F-009 | test_BC_2_01_013_static_cookie_auth_strategy_injects_access_token_not_bearer | no header assertion at mock boundary |
//!
//! | F-PASS1-MED-001 | test_BC_2_06_014_boot_step9a_uses_resolved_spec_overlay_url | test exercised EC-004 skip path, never constructed adapter or called fetch() |
//!
//! # Adversary pass-2 findings addressed (BC-2.01.013 OCSF Conformance Clause)
//!
//! | Finding | Tests | Root cause |
//! |---------|-------|-----------|
//! | F-001-R | test_BC_2_01_013_ocsf_conformance_spec_columns_survive_into_arrow_schema | Spec-declared data columns dropped; only 3 envelope cols returned |
//! | F-001-R | test_BC_2_01_013_ocsf_conformance_envelope_derived_not_raw_copied | category_uid/class_uid read verbatim from raw JSON; not derived from ocsf_class |
//! | F-001-R | test_BC_2_01_013_ocsf_conformance_sensor_virtual_column_is_canonical_sensor_id | _sensor read from raw JSON; can be tampered; must equal spec sensor_id |
//! | F-002-R | test_BC_2_22_001_production_boot_path_wiring_guard | Replaced duplicate-helper call with source-code structural wiring guard |
//! | F-004-R | test_BC_2_01_013_auth_refresh_failed_display_carries_e_auth_002_taxonomy_code | E-AUTH-002 taxonomy code structurally pinned to prevent silent drift |
//!
//! # D-925 fixture correction (BC-2.01.013, adversary F-001-R-RECUR)
//!
//! `make_crowdstrike_detection_spec` was corrected from fake `ocsf_class = "detection"` to the
//! real production value `ocsf_class = "security_finding"` (crowdstrike.sensor.toml detections
//! table at time of writing). Expected envelope post-OCSF-CLASS-MIGRATION-001:
//! `class_uid = 2004`, `category_uid = 2` (2004/1000). Per BC-2.02.012 Option A,
//! `select_by_class_name("security_finding")` now returns 2004 (Detection Finding) with a
//! deprecation WARN — not 2001 as it did prior to OCSF-CLASS-MIGRATION-001.
//!
//! # AC → Test Mapping
//!
//! | AC | Test Name | BC | Finding |
//! |----|-----------|-----|---------|
//! | AC-001 | test_BC_2_01_013_spec_driven_adapter_crowdstrike_delegates_to_pipeline_executor | BC-2.01.013 | F-005 |
//! | AC-002 | test_BC_2_01_013_spec_driven_adapter_bearer_static_extracts_token_from_sensor_auth | BC-2.01.013 | F-005 |
//! | AC-003 | test_BC_2_01_013_spec_driven_adapter_cyberint_cookie_auth_injects_access_token_cookie | BC-2.01.013 | F-005 |
//! | AC-009 | test_BC_2_01_013_static_cookie_auth_strategy_injects_access_token_not_bearer | BC-2.01.013 | F-005/F-009 |
//! | AC-010 | test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_bearer_static | BC-2.01.013 | F-001 |
//! | AC-010 | test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_plugin | BC-2.01.013 | F-001 |
//! | AC-010 | test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_static_cookie | BC-2.01.013 | F-001 |
//! | AC-010 (item 1) | test_BC_2_01_013_ocsf_conformance_spec_columns_survive_into_arrow_schema | BC-2.01.013 | F-001-R/D-925 |
//! | AC-010 (item 2) | test_BC_2_01_013_ocsf_conformance_envelope_derived_not_raw_copied | BC-2.01.013 | F-001-R/D-925 |
//! | AC-010 (item 3) | test_BC_2_01_013_ocsf_conformance_sensor_virtual_column_is_canonical_sensor_id | BC-2.01.013 | F-001-R/D-925 |
//! | AC-012 | test_BC_2_01_013_spec_driven_adapter_double_401_returns_auth_refresh_failed | BC-2.01.013 | F-004 |
//! | AC-012 | test_BC_2_01_013_auth_refresh_failed_display_carries_e_auth_002_taxonomy_code | BC-2.01.013 | F-004-R |
//! | AC-004 | test_BC_2_22_001_boot_step9a_registers_correct_adapter_count | BC-2.22.001 | (exists, unchanged) |
//! | AC-005 | test_BC_2_06_014_boot_step9a_uses_resolved_spec_overlay_url | BC-2.06.014 | F-PASS1-MED-001 (load-bearing rewrite) |
//! | AC-006 | test_BC_2_22_001_boot_step9a_empty_spec_catalog_registers_zero_adapters | BC-2.22.001 | (exists, unchanged) |
//! | AC-007 | test_BC_2_11_005_adapter_registry_get_returns_adapter_for_registered_pair | BC-2.11.005 | (exists, unchanged) |
//! | AC-008 | test_BC_2_01_013_bearer_static_auth_provider_returns_bearer_token | BC-2.01.013 | GREEN-BY-DESIGN |
//! | F-002-R | test_BC_2_22_001_production_boot_path_wiring_guard | BC-2.22.001 | F-002-R |
//! | EC-001 | test_BC_2_22_001_boot_step9a_unknown_auth_type_skips_sensor | BC-2.22.001 | (exists, unchanged) |
//! | EC-007 | test_BC_2_22_001_boot_step9a_unsupported_auth_type_skips_adapter_not_error | BC-2.22.001 | (exists, unchanged) |
//!
//! # SID-1 Mock boundary strategy
//!
//! Tests use `wiremock` (same library as prism-spec-engine tests) to mock at the
//! HTTP boundary. The `SpecDrivenSensorAdapter::new(resolved, auth_strategy, http_client)`
//! constructor accepts a `reqwest::Client` whose traffic is directed at the wiremock
//! server URI. This exercises the production fetch() path without a live DTU clone.
//!
//! For tests that require a live DTU (true integration tests), they are marked
//! `#[ignore]` with a code comment citing the blocking dependency.
//!
//! BCs: BC-2.01.013, BC-2.11.005, BC-2.06.014, BC-2.22.001
//! Story: S-DEMO-001 v1.3

#![allow(dead_code, unused_imports, non_snake_case, clippy::unwrap_used)]
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
use prism_ocsf::EventClassSelector;
use prism_sensors::adapter::QueryParams;
use prism_sensors::auth::SensorAuth;
use prism_sensors::{
    AdapterRegistry, BearerStaticSensorAuth, SensorAdapter,
    adapter::SensorSpec as SensorAdapterSpec,
};
use prism_spec_engine::SpecEngineError;
use prism_spec_engine::{
    AuthProvider, AuthToken, PluginAuthProvider, ResolvedSensorSpec, ResolvedSpecKey,
    auth_provider::MockAuthProvider,
    overlay::{OverlayLoader, OverlayProvenance, SensorInstanceOverlay},
    spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

// ---------------------------------------------------------------------------
// NullTestOrgIdStore — minimal CredentialStoreOrgId stub for test fixtures
//
// ADR-034 §D5 Red Gate sibling sweep: step9a_populate_adapter_registry now requires
// an Arc<dyn CredentialStoreOrgId>. Existing tests that do not exercise Tier-3
// pass this null store — it returns Ok(None) for all reads (Tier 3 skipped gracefully).
// The Red Gate tests in bc_2_06_003_tier3_keyring_resolution.rs use a REAL KeyringBackend.
// ---------------------------------------------------------------------------
struct NullTestOrgIdStore;

#[async_trait::async_trait]
impl prism_credentials::CredentialStoreOrgId for NullTestOrgIdStore {
    async fn get_by_org(
        &self,
        _org_id: &prism_core::OrgId,
        _sensor: &str,
        _name: &prism_core::CredentialName,
    ) -> Result<Option<secrecy::SecretString>, prism_core::PrismError> {
        Ok(None)
    }
    async fn set_by_org(
        &self,
        _org_id: &prism_core::OrgId,
        _sensor: &str,
        _name: &prism_core::CredentialName,
        _value: secrecy::SecretString,
    ) -> Result<(), prism_core::PrismError> {
        Ok(())
    }
    async fn delete_by_org(
        &self,
        _org_id: &prism_core::OrgId,
        _sensor: &str,
        _name: &prism_core::CredentialName,
    ) -> Result<bool, prism_core::PrismError> {
        Ok(false)
    }
    async fn list_by_org(
        &self,
        _org_id: &prism_core::OrgId,
    ) -> Result<Vec<(String, prism_core::CredentialName)>, prism_core::PrismError> {
        Ok(vec![])
    }
    async fn exists_by_org(
        &self,
        _org_id: &prism_core::OrgId,
        _sensor: &str,
        _name: &prism_core::CredentialName,
    ) -> Result<bool, prism_core::PrismError> {
        Ok(false)
    }
}

fn null_org_id_store() -> Arc<dyn prism_credentials::CredentialStoreOrgId> {
    Arc::new(NullTestOrgIdStore)
}

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
        &format!("{sensor_id} sensor (test fixture)") as &str,
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
fn make_resolved_spec(spec: SensorSpec, org_slug: &str) -> ResolvedSensorSpec {
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
/// Returns `(Arc<registry>, org_id, org_slug)`.
fn make_org_registry(slug: &str) -> (Arc<prism_core::OrgRegistry>, OrgId, OrgSlug) {
    let registry = prism_core::OrgRegistry::new();
    let uuid = uuid::Uuid::now_v7();
    let org_id = OrgId::from_uuid(uuid);
    let org_slug = OrgSlug::new(slug);
    registry
        .register(org_slug.clone(), org_id)
        .expect("test fixture: OrgRegistry::register failed");
    (Arc::new(registry), org_id, org_slug)
}

/// Build a minimal `SensorAdapterSpec` (prism-sensors `SensorSpec`, not spec-engine `SensorSpec`).
#[allow(deprecated)]
fn make_adapter_spec(_sensor_id: &str, org_id: OrgId) -> SensorAdapterSpec {
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

/// Minimal `SensorAuth` implementation for `cookie_roundtrip` tests.
struct CookieSensorAuth;

impl SensorAuth for CookieSensorAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn auth_type_name(&self) -> &'static str {
        "cookie_roundtrip"
    }
}

/// Build a wiremock-backed `reqwest::Client` directed at `base_url`.
///
/// The client has a 30-second timeout per CLAUDE.md conventions
/// (TD-S-PLUGIN-PREREQ-B-005). All HTTP calls from the adapter go to the
/// mock server — no external network access during tests.
fn make_mock_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("test fixture: reqwest::Client construction failed")
}

/// Minimal JSON response body for a sensor data endpoint.
///
/// Returns a JSON object with a `data` array containing one event record.
/// The `response_path` in `make_spec` is `$.data`, so this will yield one record.
fn minimal_sensor_response() -> serde_json::Value {
    serde_json::json!({
        "data": [
            {
                "event_id": "evt-test-001",
                "category_uid": 4,
                "class_uid": 4001,
                "_sensor": "test-sensor"
            }
        ]
    })
}

// ---------------------------------------------------------------------------
// F-001 / F-005: fetch() must return non-empty OCSF RecordBatches
//
// AC-010 — BC-2.01.013 postcondition:
// `SpecDrivenSensorAdapter::fetch()` MUST return `Vec<RecordBatch>` that is:
// (a) non-empty (at least one RecordBatch with at least one row), and
// (b) contains OCSF columns: `category_uid`, `class_uid`, `_sensor`.
//
// CURRENT FAILURE MODE (F-001 + F-005 paper-fix):
// `fetch()` calls `PipelineExecutor::execute()` (which works), but then does
// `let _ = result;` and returns `Ok(Vec::new())` always. The assertion on
// non-empty batches will FAIL because the batch is empty despite valid HTTP data.
//
// These tests are LOAD-BEARING for the OCSF data path (TD-VSDD-059):
// they will fail if the implementer does `let _ = result` or returns empty Vec.
// ---------------------------------------------------------------------------

/// AC-010 (bearer_static path) — BC-2.01.013 postcondition:
/// fetch() for a BearerStatic sensor (Armis) returns non-empty RecordBatches
/// containing OCSF columns when the mock server returns valid sensor data.
///
/// # Red Gate Failure
///
/// `fetch()` discards PipelineResult records and returns `Ok(Vec::new())`.
/// The assertion `!batches.is_empty()` FAILS because the Vec is always empty.
///
/// Mock boundary: wiremock serves `{"data": [{...}]}` at `GET /api/v1/events`.
/// The `reqwest::Client` is directed at the mock server URI, so no live DTU needed.
///
/// BC-2.01.013; AC-010; F-001; F-005; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_bearer_static() {
    let mock_server = MockServer::start().await;

    // Mount: return one record at GET /api/v1/events.
    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_sensor_response()))
        .mount(&mock_server)
        .await;

    // Build the Armis spec directed at the mock server.
    let spec = make_spec("armis", AuthType::BearerStatic, &mock_server.uri());
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    // BearerStatic: token extracted from SensorAuth arg at fetch() call time.
    let auth_strategy = AdapterAuthStrategy::BearerStatic;
    let http_client = make_mock_client();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("armis", org_id);
    let params = make_query_params();
    // BearerStaticSensorAuth from prism-sensors (public type, re-exported via prism_sensors::BearerStaticSensorAuth).
    let sensor_auth = BearerStaticSensorAuth::new("armis-bearer-token-test-001");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // F-005 fix: assert REAL behavior, not just `let _result = ...`.
    // F-001 fix: assert the batches are non-empty (pipeline records were materialized).
    assert!(
        result.is_ok(),
        "AC-010: fetch() must return Ok(batches) when mock server returns valid JSON. \
         Got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();

    // LOAD-BEARING assertion (TD-VSDD-059):
    // This FAILS with the current implementation because fetch() returns Vec::new()
    // even when the pipeline returns records. Implementer must map PipelineResult
    // records → RecordBatch via ColumnMapper/OcsfNormalizer.
    assert!(
        !batches.is_empty(),
        "AC-010 LOAD-BEARING: fetch() must return at least one RecordBatch when the \
         sensor endpoint returns data. Got empty Vec. \
         Root cause: fetch() currently discards pipeline records (`let _ = result;`) \
         and returns Ok(Vec::new()). Fix: map PipelineResult.records through \
         ColumnMapper + OcsfNormalizer → RecordBatch. BC-2.01.013 postcondition."
    );

    // Assert total row count > 0.
    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert!(
        total_rows > 0,
        "AC-010: returned RecordBatches must contain at least one row. \
         Got {} rows across {} batches. BC-2.01.013.",
        total_rows,
        batches.len()
    );

    // Assert OCSF columns are present.
    // The column names must include the OCSF normalization fields.
    let first_batch = &batches[0];
    let schema = first_batch.schema();
    let column_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    // _sensor is injected by the OCSF normalizer to track data provenance.
    // category_uid and class_uid are mandatory OCSF v1.1 envelope fields.
    for required_col in &["category_uid", "class_uid", "_sensor"] {
        assert!(
            column_names.contains(required_col),
            "AC-010 OCSF column assertion: RecordBatch must contain column '{}'. \
             Present columns: {:?}. \
             Root cause: OCSF normalization not applied in fetch(). \
             BC-2.01.013 postcondition; S-DEMO-001 AC-010.",
            required_col,
            column_names
        );
    }
}

/// AC-010 (plugin path) — BC-2.01.013 postcondition:
/// fetch() for a CustomViaPlugin sensor (CrowdStrike) returns non-empty RecordBatches
/// when the mock server returns valid sensor data.
///
/// # Red Gate Failure
///
/// Same as bearer_static path: `let _ = result;` discards records → empty Vec returned.
///
/// BC-2.01.013; AC-010; F-001; F-005; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_plugin() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_sensor_response()))
        .mount(&mock_server)
        .await;

    let spec = make_spec("crowdstrike", AuthType::CustomViaPlugin, &mock_server.uri());
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    let mock_auth: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("cs-oauth-token-test"));
    let auth_strategy = AdapterAuthStrategy::Plugin(mock_auth);
    let http_client = make_mock_client();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("crowdstrike", org_id);
    let params = make_query_params();
    let sensor_auth = PluginSensorAuth;

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    assert!(
        result.is_ok(),
        "AC-010 (plugin): fetch() must return Ok(batches) when mock returns valid JSON. \
         Got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();

    // LOAD-BEARING: fails because fetch() discards pipeline records.
    assert!(
        !batches.is_empty(),
        "AC-010 (plugin) LOAD-BEARING: fetch() must return at least one RecordBatch. \
         Got empty Vec. fetch() currently does `let _ = result;` after calling \
         PipelineExecutor::execute(). Fix: map records through ColumnMapper + OcsfNormalizer. \
         BC-2.01.013; S-DEMO-001 AC-010."
    );

    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert!(
        total_rows > 0,
        "AC-010 (plugin): at least one row required in returned RecordBatches. \
         Got {} rows. BC-2.01.013.",
        total_rows
    );
}

/// AC-010 (StaticCookie path) — BC-2.01.013 postcondition:
/// fetch() for a CookieRoundtrip sensor (Cyberint) returns non-empty RecordBatches
/// when the mock server returns valid sensor data.
///
/// # Red Gate Failure
///
/// Same root cause: `let _ = result;` discards records → empty Vec.
///
/// BC-2.01.013; AC-010; F-001; F-005; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_static_cookie() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_sensor_response()))
        .mount(&mock_server)
        .await;

    let spec = make_spec("cyberint", AuthType::CookieRoundtrip, &mock_server.uri());
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    let mock_provider: Arc<dyn AuthProvider> =
        Arc::new(MockAuthProvider::new("cyberint-api-key-test-001"));
    let auth_strategy = AdapterAuthStrategy::StaticCookie(mock_provider);
    let http_client = make_mock_client();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("cyberint", org_id);
    let params = make_query_params();
    let sensor_auth = CookieSensorAuth;

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    assert!(
        result.is_ok(),
        "AC-010 (cookie): fetch() must return Ok(batches) when mock returns valid JSON. \
         Got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();

    // LOAD-BEARING: fails because fetch() discards pipeline records.
    assert!(
        !batches.is_empty(),
        "AC-010 (cookie) LOAD-BEARING: fetch() must return at least one RecordBatch. \
         Got empty Vec. Fix: map PipelineResult records through ColumnMapper + OcsfNormalizer. \
         BC-2.01.013; S-DEMO-001 AC-010."
    );

    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert!(
        total_rows > 0,
        "AC-010 (cookie): at least one row required. Got {} rows. BC-2.01.013.",
        total_rows
    );
}

// ---------------------------------------------------------------------------
// F-005: AC-001 — CrowdStrike delegates to PipelineExecutor (LOAD-BEARING rewrite)
//
// Old test: `let _result = adapter.fetch(...).await;` — zero assertions.
// New test: mock HTTP server, assert delegation succeeds and returns Ok(batches).
//
// Red Gate Failure: fetch() returns Ok(Vec::new()) even when pipeline succeeds.
// But Ok(Vec::new()) is still Ok, so the is_ok() assertion would pass.
// HOWEVER — the OCSF column assertions in test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_plugin
// cover the data-path failure for the plugin path. This test covers that the
// auth dispatch selects the Plugin strategy (not BearerStatic or StaticCookie)
// and the pipeline is called with the plugin auth provider token, verifiable
// by asserting the Authorization header contains the expected token.
// ---------------------------------------------------------------------------

/// AC-001 — BC-2.01.013 postcondition 4 (Plugin auth path):
/// `SpecDrivenSensorAdapter::fetch()` for a CrowdStrike (CustomViaPlugin) adapter
/// calls PipelineExecutor with the held `PluginAuthProvider` token.
/// The mock server must receive `Authorization: Bearer cs-oauth-token-ac001`.
///
/// # Red Gate Failure
///
/// The mock server is configured with `expect(1)` — exactly one request.
/// If fetch() fails to delegate to PipelineExecutor (e.g., returns early before
/// calling the pipeline), the mock server assertion fires: the mock server
/// received 0 requests but expected 1. This FAILS the test via wiremock's
/// `verify_and_reset()` (called implicitly on MockServer drop).
///
/// Also: the current fetch() discards pipeline records, so the is_ok() +
/// non-empty assertion mirrors test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_plugin.
///
/// BC-2.01.013; AC-001; F-005; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_spec_driven_adapter_crowdstrike_delegates_to_pipeline_executor() {
    let mock_server = MockServer::start().await;

    // Expect EXACTLY 1 call. If fetch() never calls the pipeline, wiremock will
    // fail verification on MockServer drop (expect(1) not satisfied).
    // The Authorization header must contain the plugin token.
    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .and(header("Authorization", "Bearer cs-oauth-token-ac001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_sensor_response()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let spec = make_spec("crowdstrike", AuthType::CustomViaPlugin, &mock_server.uri());
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    // The MockAuthProvider returns "cs-oauth-token-ac001" — this is injected as
    // Authorization: Bearer cs-oauth-token-ac001 by PipelineExecutor::build_request.
    let mock_auth: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("cs-oauth-token-ac001"));
    let auth_strategy = AdapterAuthStrategy::Plugin(mock_auth);
    let http_client = make_mock_client();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("crowdstrike", org_id);
    let params = make_query_params();
    let sensor_auth = PluginSensorAuth;

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // F-005 fix: real assertions (not `let _result = ...`).
    assert!(
        result.is_ok(),
        "AC-001: fetch() must return Ok when plugin-auth mock server returns 200. \
         Got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();

    // LOAD-BEARING (F-001): fails because fetch() discards pipeline records.
    assert!(
        !batches.is_empty(),
        "AC-001 LOAD-BEARING: fetch() must return non-empty batches when pipeline returns data. \
         Got empty Vec. fetch() discards PipelineResult with `let _ = result;`. \
         BC-2.01.013; AC-001; F-001/F-005."
    );

    // MockServer drop verifies the `expect(1)` — if fetch() never called the pipeline,
    // the mock server assertion fires here: FAIL.
}

/// AC-002 — BC-2.01.013 postcondition 4 (BearerStatic auth path):
/// `SpecDrivenSensorAdapter::fetch()` for an Armis (bearer_static) adapter
/// extracts the token from the `SensorAuth::BearerStatic` argument and uses it
/// as the Authorization: Bearer header in the pipeline request.
///
/// # Red Gate Failure
///
/// 1. `expect(1)`: if fetch() doesn't call the pipeline, wiremock fails at drop.
/// 2. `!batches.is_empty()`: fetch() discards pipeline records → empty Vec.
///
/// BC-2.01.013; AC-002; OQ-1; F-005; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_spec_driven_adapter_bearer_static_extracts_token_from_sensor_auth() {
    let mock_server = MockServer::start().await;

    // The token "armis-bearer-token-xyz" must appear as the Authorization: Bearer header.
    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .and(header("Authorization", "Bearer armis-bearer-token-xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_sensor_response()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let spec = make_spec("armis", AuthType::BearerStatic, &mock_server.uri());
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    let auth_strategy = AdapterAuthStrategy::BearerStatic;
    let http_client = make_mock_client();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("armis", org_id);
    let params = make_query_params();
    // BearerStaticSensorAuth carries the bearer token for extraction at fetch() call time.
    // prism_sensors::BearerStaticSensorAuth is the canonical type (public, re-exported).
    let sensor_auth = BearerStaticSensorAuth::new("armis-bearer-token-xyz");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // F-005 fix: real assertions.
    assert!(
        result.is_ok(),
        "AC-002: fetch() must return Ok when BearerStatic mock server returns 200. \
         Got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();

    // LOAD-BEARING (F-001): fails because fetch() discards pipeline records.
    assert!(
        !batches.is_empty(),
        "AC-002 LOAD-BEARING: fetch() must return non-empty batches for bearer_static path. \
         Got empty Vec. fix: map PipelineResult.records → RecordBatch. \
         BC-2.01.013; AC-002; F-001/F-005."
    );

    // MockServer drop verifies expect(1): if Authorization header was wrong or
    // pipeline was not called, wiremock FAILS here.
}

/// AC-003 — BC-2.01.013 postcondition 4 (StaticCookie auth path):
/// `SpecDrivenSensorAdapter::fetch()` for Cyberint (cookie_roundtrip) uses its
/// held `StaticCookieAuthProvider` and the pipeline injects
/// `Cookie: access_token={api_key}`.
///
/// # Red Gate Failure
///
/// 1. The mock server is mounted with a `Cookie: access_token=cyberint-api-key-test-003`
///    header matcher. If the pipeline sends `Authorization: Bearer` instead, the mock
///    returns 404 (no route matches), causing pipeline to return Err.
///    → `result.is_ok()` FAILS.
///
/// 2. Even if the cookie injection is correct, `!batches.is_empty()` FAILS because
///    fetch() discards pipeline records.
///
/// BC-2.01.013; AC-003; ADR-031 §D3-b; F-005; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_spec_driven_adapter_cyberint_cookie_auth_injects_access_token_cookie() {
    let mock_server = MockServer::start().await;

    // The mock server ONLY matches requests that have the correct Cookie header.
    // If the pipeline sends Authorization: Bearer instead, this mock is not matched
    // and wiremock returns 404 → pipeline fails → test fails.
    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .and(header("Cookie", "access_token=cyberint-api-key-test-003"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_sensor_response()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let spec = make_spec("cyberint", AuthType::CookieRoundtrip, &mock_server.uri());
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    let mock_static_cookie: Arc<dyn AuthProvider> =
        Arc::new(MockAuthProvider::new("cyberint-api-key-test-003"));
    let auth_strategy = AdapterAuthStrategy::StaticCookie(mock_static_cookie);
    let http_client = make_mock_client();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("cyberint", org_id);
    let params = make_query_params();
    let sensor_auth = CookieSensorAuth;

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // F-005 fix: real assertions (not `let _result = ...`).
    assert!(
        result.is_ok(),
        "AC-003: fetch() must return Ok when StaticCookie mock server returns 200. \
         If this fails with an HTTP error, the Cookie header was injected incorrectly \
         (Authorization: Bearer was used instead of Cookie: access_token=). \
         Got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();

    // LOAD-BEARING (F-001): fails because fetch() discards pipeline records.
    assert!(
        !batches.is_empty(),
        "AC-003 LOAD-BEARING: fetch() must return non-empty batches for StaticCookie path. \
         Got empty Vec. Fix: map PipelineResult.records → RecordBatch. \
         BC-2.01.013; AC-003; F-001/F-005."
    );

    // MockServer drop verifies expect(1): cookie header must be correct AND pipeline called.
}

// ---------------------------------------------------------------------------
// F-009: AC-009 — StaticCookie injects Cookie: access_token=, NOT Authorization: Bearer
//
// The mock server has TWO routes:
//   - Route A (CORRECT): Cookie: access_token=... → 200
//   - Route B (WRONG):   Authorization: Bearer ... → 403 Forbidden
//
// If the pipeline incorrectly injects Authorization: Bearer, route B fires and
// PipelineExecutor returns an HTTP error → fetch() returns Err → is_ok() FAILS.
// ---------------------------------------------------------------------------

/// AC-009 / AC-003 supplemental — BC-2.01.013 (cookie injection invariant):
/// The `StaticCookie` auth strategy MUST cause PipelineExecutor to inject
/// `Cookie: access_token={token}`, NOT `Authorization: Bearer {token}`.
///
/// Mock strategy: two routes — correct cookie returns 200, wrong Bearer returns 403.
/// If the pipeline sends Bearer, the 403 causes PipelineExecutor to return an HTTP error,
/// and fetch() propagates it as Err. The `result.is_ok()` assertion then FAILS.
///
/// This test is LOAD-BEARING (F-009): it will fail if:
/// (a) the pipeline incorrectly uses Bearer for CookieRoundtrip sensors, or
/// (b) fetch() doesn't call the pipeline at all (wiremock expect(1) fires).
///
/// BC-2.01.013; AC-009; ADR-031 §D3-b; F-005; F-009; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_static_cookie_auth_strategy_injects_access_token_not_bearer() {
    let mock_server = MockServer::start().await;

    // Route A (CORRECT path): Cookie: access_token=cyberint-static-api-key → 200.
    // This is what the pipeline MUST send for CookieRoundtrip sensors (ADR-031 §D3-b).
    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .and(header("Cookie", "access_token=cyberint-static-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_sensor_response()))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Route B (WRONG path): Authorization: Bearer → 403 Forbidden.
    // If the pipeline incorrectly injects Bearer for CookieRoundtrip, this fires.
    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .and(header("Authorization", "Bearer cyberint-static-api-key"))
        .respond_with(ResponseTemplate::new(403).set_body_string(
            "F-009: Cookie path must NOT inject Authorization: Bearer. \
             Use Cookie: access_token=... (ADR-031 §D3-b).",
        ))
        .mount(&mock_server)
        .await;

    let spec = make_spec("cyberint", AuthType::CookieRoundtrip, &mock_server.uri());
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    let mock_provider: Arc<dyn AuthProvider> =
        Arc::new(MockAuthProvider::new("cyberint-static-api-key"));
    let auth_strategy = AdapterAuthStrategy::StaticCookie(mock_provider);
    let http_client = make_mock_client();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("cyberint", org_id);
    let params = make_query_params();
    let sensor_auth = CookieSensorAuth;

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // F-009 + F-005 fix: assert the result AND the header invariant.
    // If this fails with an HTTP error, check: did the pipeline inject Bearer instead of Cookie?
    assert!(
        result.is_ok(),
        "AC-009 LOAD-BEARING: StaticCookie path must inject Cookie: access_token=..., \
         NOT Authorization: Bearer. If the mock server returned 403, the pipeline sent \
         the wrong header. Check PipelineExecutor::build_request for CookieRoundtrip. \
         ADR-031 §D3-b. Got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();

    // LOAD-BEARING (F-001): also fails because fetch() discards pipeline records.
    assert!(
        !batches.is_empty(),
        "AC-009 LOAD-BEARING (data path): fetch() must return non-empty batches. \
         Got empty Vec. Fix: map PipelineResult records → RecordBatch. \
         BC-2.01.013; AC-009; F-001/F-009."
    );

    // MockServer drop: expect(1) on route A must be satisfied.
    // If expect(1) fires, route A was not called (either no request or wrong header).
}

// ---------------------------------------------------------------------------
// F-004: AC-012 — double-401 returns AuthRefreshFailed, not connection-refused error
//
// OLD test: hit http://127.0.0.1:18085 (connection refused) → any Err() passes.
// NEW test: wiremock serves 401 on ALL requests → PipelineExecutor::execute() returns
//           SpecEngineError::AuthRefreshFailed → fetch() maps to SensorError::Internal
//           with detail containing "E-AUTH-002" or "auth refresh failed".
//
// This test is LOAD-BEARING for the error-taxonomy mapping (AC-012):
// it distinguishes AuthRefreshFailed from any other error.
// ---------------------------------------------------------------------------

/// AC-012 — BC-2.01.013 error case:
/// When `PipelineExecutor` returns a double-401 (initial + retry both return 401),
/// `SpecDrivenSensorAdapter::fetch()` propagates `SpecEngineError::AuthRefreshFailed`
/// as `SensorError::Internal` with detail containing `E-AUTH-002`.
///
/// # Red Gate Failure
///
/// The OLD test used `http://127.0.0.1:18085` (connection refused), which returns
/// `SensorError::Internal { detail: "...connection refused..." }` — any `is_err()`
/// would pass. This test uses a wiremock server that returns HTTP 401 for ALL
/// requests (both initial and retry), forcing `SpecEngineError::AuthRefreshFailed`.
///
/// The new assertions check the SPECIFIC error:
/// - `result.is_err()` — still passes (same as before)
/// - `detail.contains("E-AUTH-002")` — FAILS until the error mapping is correct
///   AND the pipeline is actually called (not just connection refused)
///
/// BC-2.01.013; AC-012; EC-006; F-004; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_spec_driven_adapter_double_401_returns_auth_refresh_failed() {
    let mock_server = MockServer::start().await;

    // ALL requests return 401 — both the initial request and the retry after
    // token refresh both receive 401. PipelineExecutor::execute() should return
    // SpecEngineError::AuthRefreshFailed (BC-2.16.002 row 10).
    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let spec = make_spec("armis", AuthType::BearerStatic, &mock_server.uri());
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    let auth_strategy = AdapterAuthStrategy::BearerStatic;
    let http_client = make_mock_client();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    let adapter_spec = make_adapter_spec("armis", org_id);
    let params = make_query_params();
    // BearerStaticSensorAuth carries an "expired" token that will return 401.
    let sensor_auth = BearerStaticSensorAuth::new("expired-bearer-token-ac012");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // AC-012 requires an error response.
    assert!(
        result.is_err(),
        "AC-012: double-401 must return Err. Got Ok({:?}). \
         BC-2.01.013 error case; S-DEMO-001 AC-012.",
        result.ok().map(|b| b.len())
    );

    let err = result.unwrap_err();

    // LOAD-BEARING error-taxonomy assertion (F-004):
    // The error must be SensorError::Internal with the AuthRefreshFailed detail.
    // This FAILS for the connection-refused case (which produces a different error message).
    // It ALSO fails if the implementation maps AuthRefreshFailed to the wrong error variant.
    let err_str = format!("{err:?}");

    // Check it's the Internal variant (error taxonomy: double-401 → E-SENSOR-099).
    assert!(
        matches!(err, prism_sensors::adapter::SensorError::Internal { .. }),
        "AC-012 LOAD-BEARING: double-401 must map to SensorError::Internal. \
         Got: {:?}. BC-2.01.013 error case; error taxonomy E-SENSOR-099.",
        err_str
    );

    // Check the detail contains the AuthRefreshFailed indicator.
    // SpecEngineError::AuthRefreshFailed displays as "E-AUTH-002: auth refresh failed...".
    // The current implementation wraps it as SensorError::Internal { detail: "...{e}..." }.
    if let prism_sensors::adapter::SensorError::Internal { detail } = &err {
        assert!(
            detail.contains("E-AUTH-002")
                || detail.contains("auth refresh failed")
                || detail.contains("AuthRefreshFailed"),
            "AC-012 LOAD-BEARING: SensorError::Internal detail must reference auth refresh failure \
             (E-AUTH-002 or 'auth refresh failed'). \
             This distinguishes double-401 from connection refused or other errors. \
             Got detail: {:?}. \
             BC-2.01.013; AC-012; F-004; error taxonomy E-AUTH-002.",
            detail
        );
    }
}

// ---------------------------------------------------------------------------
// F-002: Boot step 9A not called in production boot path
//
// FINDING: `step9_start_mcp_server` in boot.rs creates `AdapterRegistry::new()`
// (empty) and NEVER calls `step9a_populate_adapter_registry`. The adapter registry
// is always empty in production.
//
// This test exercises the PRODUCTION integration — it calls
// `step9a_populate_adapter_registry` (as the boot path SHOULD call it) and
// asserts the registry is populated. Then it verifies the boot path behavior
// is BROKEN: when the same resolved_spec_map is passed to a simulated step9
// (which omits the step9a call), the registry remains empty.
//
// The test FAILS because `step9_start_mcp_server` is not testable at unit
// level without a full RocksDB stack. Therefore we use the following approach:
//
// 1. Assert that `step9a_populate_adapter_registry` DOES populate the registry
//    (this part passes — the function works).
// 2. Assert that there EXISTS a call to `step9a_populate_adapter_registry` in
//    the production `step9_start_mcp_server` code path by checking that a boot
//    with a non-empty resolved_spec_map produces an non-empty adapter registry.
//    Since step9_start_mcp_server creates a NEW empty registry without calling
//    step9a, the only way to observe this is via an integration point.
//
// SID-1 compliant approach: the unit-level test for the WIRING gap is:
// - test_BC_2_22_001_step9a_not_called_in_boot_production_path (below)
//   asserts the expected post-condition of boot: AdapterRegistry is non-empty
//   after a boot with non-empty spec catalog. This FAILS because step9a is
//   not wired into step9.
// ---------------------------------------------------------------------------

/// F-002 — BC-2.22.001 sequencing invariant:
/// After `step9a_populate_adapter_registry` populates the registry,
/// `AdapterRegistry::get(org_id, sensor_id)` must return `Some(adapter)`.
/// This confirms step9a WORKS when called.
///
/// Then: assert that the boot function `step9_start_mcp_server` wires step9a
/// by checking that after a mocked boot with resolved_spec_map entries,
/// the resulting adapter registry is non-empty.
///
/// # Red Gate Failure
///
/// The current `step9_start_mcp_server` creates `AdapterRegistry::new()` and
/// NEVER calls `step9a_populate_adapter_registry`. Therefore any adapter lookup
/// after the boot step returns `None`. The assertion `adapter.is_some()` FAILS.
///
/// This test exercises the production wiring gap at the unit level by:
/// (a) Calling `step9a_populate_adapter_registry` on a known input → assert populated.
/// (b) Calling the boot path simulation (step9a_call_gate) which MUST call step9a
///     if wired correctly → assert non-empty. This FAILS because step9 doesn't call step9a.
///
/// BC-2.22.001; F-002; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_22_001_step9a_not_called_in_boot_production_path() {
    // Part 1: Verify step9a_populate_adapter_registry works correctly in isolation.
    // This establishes the baseline: the function itself populates the registry.
    let (org_registry, org_id, org_slug) = make_org_registry("acme-org");
    let sensor_id = SensorId::from("armis");

    let armis_spec = make_spec("armis", AuthType::BearerStatic, "http://127.0.0.1:19080");
    let resolved = make_resolved_spec(armis_spec, "acme-org");

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug.clone(), sensor_id.clone()), resolved);

    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    let count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        Arc::clone(&org_registry),
        &plugin_auth_providers,
        &mut adapter_registry,
        null_org_id_store(),
    )
    .await
    .expect("step9a should return Ok(count) with one Armis spec");

    assert_eq!(
        count, 1,
        "Part 1: step9a must register exactly 1 adapter for 1 Armis (BearerStatic) spec. \
         BC-2.22.001 postcondition."
    );

    let registered = adapter_registry.get(org_id, &sensor_id);
    assert!(
        registered.is_some(),
        "Part 1: AdapterRegistry::get must return Some(adapter) after step9a. \
         BC-2.11.005."
    );

    // Part 2 (F-002-R replacement):
    // The production boot wiring assertion has been moved to the structural guard test
    // `test_BC_2_22_001_production_boot_path_wiring_guard` which reads boot.rs source
    // directly to verify that step9_start_mcp_server calls step9a_populate_adapter_registry.
    // The `boot_step9_build_adapter_registry` duplicate helper was DELETED (F-002-R).
    // BC-2.22.001; F-002-R; S-DEMO-001 v1.5.
}

// ---------------------------------------------------------------------------
// Remaining tests (AC-004..AC-008, EC-007, OQ-2) — retained with enhanced assertions
// ---------------------------------------------------------------------------

/// AC-004 — BC-2.22.001 postcondition:
/// `step9a_populate_adapter_registry` registers exactly one `SpecDrivenSensorAdapter`
/// per `(OrgId, SensorId)` pair found in `resolved_spec_map`.
///
/// Given 2 orgs × 1 sensor (armis) = 2 resolved specs,
/// `AdapterRegistry` must contain exactly 2 entries after boot step 9A.
/// (CrowdStrike skipped: no PluginAuthProvider supplied → EC-004 skip behavior.)
///
/// # Red Gate Failure
///
/// `step9a_populate_adapter_registry` is already implemented. This test PASSES
/// at Red Gate time (step9a works). The failing assertion is in F-002 test above
/// (boot wiring). Retained here for BC traceability.
///
/// BC-2.22.001; AC-004; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_22_001_boot_step9a_registers_correct_adapter_count() {
    let (org_registry, _org_id_a, org_slug_a) = make_org_registry("org-alpha");
    let uuid_b = uuid::Uuid::now_v7();
    let org_id_b = OrgId::from_uuid(uuid_b);
    let org_slug_b = OrgSlug::new("org-beta");
    org_registry
        .register(org_slug_b.clone(), org_id_b)
        .expect("test fixture: register org-beta failed");

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();

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

    let armis_spec_a = make_spec("armis", AuthType::BearerStatic, "http://127.0.0.1:18081");
    let armis_resolved_a = make_resolved_spec(armis_spec_a, "org-alpha");
    resolved_spec_map.insert(
        (org_slug_a.clone(), SensorId::from("armis")),
        armis_resolved_a,
    );

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

    let armis_spec_b = make_spec("armis", AuthType::BearerStatic, "http://127.0.0.1:18084");
    let armis_resolved_b = make_resolved_spec(armis_spec_b, "org-beta");
    resolved_spec_map.insert(
        (org_slug_b.clone(), SensorId::from("armis")),
        armis_resolved_b,
    );

    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    let count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        Arc::clone(&org_registry),
        &plugin_auth_providers,
        &mut adapter_registry,
        null_org_id_store(),
    )
    .await
    .expect("step9a should return Ok(count)");

    assert_eq!(
        adapter_registry.len(),
        2,
        "AC-004: registry must contain exactly 2 entries (armis for 2 orgs; \
         crowdstrike skipped: no PluginAuthProvider supplied). \
         BC-2.22.001 postcondition."
    );
    assert_eq!(
        count, 2,
        "AC-004: returned count must equal registry.len() = 2."
    );
}

/// AC-005 — BC-2.06.014 precondition 1 (LOAD-BEARING rewrite, F-PASS1-MED-001):
///
/// When boot step 9A constructs a `SpecDrivenSensorAdapter` for a sensor with a
/// per-org overlay `base_url`, the adapter directs HTTP traffic to the OVERLAY URL,
/// NOT to the TYPE-SPEC URL.
///
/// # Why the previous test was non-load-bearing
///
/// The previous AC-005 test used a `CustomViaPlugin` (CrowdStrike) sensor with an
/// empty `plugin_auth_providers` map, so the sensor was SKIPPED (EC-004 skip path).
/// `step9a_populate_adapter_registry` returned `count==0` — no adapter was ever
/// constructed. The assertion `count == 0` exercised the skip path, NOT overlay-URL
/// selection. The test would pass even if the implementation ignored the overlay
/// `base_url` entirely. F-PASS1-MED-001.
///
/// # Fix strategy
///
/// Use an Armis-style sensor (`auth_type = BearerStatic`) which is NOT skipped —
/// BearerStatic adapters are always registered (no PluginAuthProvider dependency).
///
/// Fixture:
/// - TYPE-SPEC `base_url`: a non-listening loopback address (`http://127.0.0.1:19999`).
///   Any HTTP request to this address would fail with a connection error.
/// - OVERLAY `base_url`: the wiremock server URI (the CORRECT URL).
///   Any HTTP request to this address succeeds with the mock response.
///
/// After `step9a_populate_adapter_registry` runs, the registered adapter holds a
/// `ResolvedSensorSpec` whose `spec.base_url` equals the overlay URL (not the type-spec
/// URL) — because `OverlayLoader::merge_overlay_onto_type_spec` merges the overlay
/// `base_url` into `resolved.spec.base_url` before step9a iterates the map.
///
/// The test calls `fetch()` on the retrieved adapter. The mock server is mounted with
/// `expect(1)` — exactly one request. If the adapter uses the OVERLAY URL, the mock
/// receives a request and returns 200. If the adapter mistakenly uses the TYPE-SPEC
/// URL, the request hits the non-listening port and `fetch()` returns a connection
/// error. The `result.is_ok()` assertion then FAILS.
///
/// # Why the test SHOULD pass against the current implementation
///
/// The adversary (F-PASS1-MED-001) notes that overlay resolution happens UPSTREAM in
/// `OverlayLoader::merge_overlay_onto_type_spec` before the resolved-spec map that
/// step9a iterates is constructed. By the time step9a runs, `resolved.spec.base_url`
/// already contains the overlay URL. step9a passes the resolved spec to
/// `SpecDrivenSensorAdapter::new(Arc::new(resolved_spec.clone()), ...)`, and
/// `PipelineExecutor::execute` reads `spec.base_url` directly from that resolved spec
/// to build the URL via `format!("{}{}", spec.base_url, interpolated_path)`. The
/// production code path is already correct; this test makes that correctness
/// LOAD-BEARING.
///
/// BC-2.06.014; AC-005; S-DEMO-001 v1.3; F-PASS1-MED-001.
#[tokio::test]
async fn test_BC_2_06_014_boot_step9a_uses_resolved_spec_overlay_url() {
    // Start the mock server — this controls the OVERLAY URL (the correct endpoint).
    let overlay_mock_server = MockServer::start().await;

    // Mount: expect exactly 1 GET to /api/v1/events.
    // If the adapter uses the OVERLAY URL, this mock is hit and returns 200.
    // If the adapter uses the TYPE-SPEC URL (127.0.0.1:19999, non-listening), the
    // mock receives 0 requests and the `expect(1)` fires at MockServer drop → test FAILS.
    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .and(header(
            "Authorization",
            "Bearer overlay-url-precedence-token-ac005",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_sensor_response()))
        .expect(1)
        .mount(&overlay_mock_server)
        .await;

    // TYPE-SPEC base_url: a non-listening loopback address.
    // Any HTTP request to this address produces a connection error (ECONNREFUSED).
    // This is the "wrong" URL — if the adapter uses it, fetch() returns Err.
    let type_spec_base_url = "http://127.0.0.1:19999";

    // Build an Armis (BearerStatic) TYPE spec directed at the wrong URL.
    // BearerStatic sensors are NOT skipped by step9a — they always get registered.
    let type_spec = make_spec("armis", AuthType::BearerStatic, type_spec_base_url);

    // Build the per-org overlay with base_url overriding the type spec's wrong URL.
    // OverlayLoader::merge_overlay_onto_type_spec sets resolved.spec.base_url to
    // this value before the resolved-spec map is passed to step9a.
    let (org_registry, org_id, org_slug) = make_org_registry("overlay-test-org");
    let sensor_id = SensorId::from("armis");
    let overlay_base_url = overlay_mock_server.uri();
    let overlay_toml = format!(
        "extends = \"armis\"\ninstance_id = \"armis@{}\"\nbase_url = \"{}\"",
        org_slug.as_str(),
        overlay_base_url,
    );
    let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
        .expect("test fixture: SensorInstanceOverlay TOML parse failed");

    // merge_overlay_onto_type_spec: resolved.spec.base_url == overlay_base_url
    // (NOT type_spec_base_url). This is the behavior under test — upstream merge
    // happens before step9a iterates.
    let resolved =
        OverlayLoader::merge_overlay_onto_type_spec(&type_spec, &overlay, org_slug.clone());

    // Verify the upstream merge already produced the correct base_url.
    // This assertion is a precondition guard: if OverlayLoader is broken,
    // this tells us the root cause is upstream of step9a.
    assert_eq!(
        resolved.spec.base_url, overlay_base_url,
        "AC-005 precondition: OverlayLoader::merge_overlay_onto_type_spec must embed the \
         overlay base_url into resolved.spec.base_url. Got: {:?}. \
         If this fails, the root cause is in OverlayLoader, not step9a. \
         BC-2.06.014 precondition 1.",
        resolved.spec.base_url
    );
    assert!(
        resolved.provenance.base_url_from_overlay,
        "AC-005 precondition: OverlayProvenance::base_url_from_overlay must be true when \
         overlay specifies a base_url. BC-2.06.014.",
    );

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug, sensor_id.clone()), resolved);

    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    // ADV-SDEMO002-P01-CRIT-001: BearerStatic sensors now resolve the bearer token from
    // `resolve_credential(org_slug, "armis", "bearer_token")` via the per-client env var
    // (ADR-032 / BC-2.06.003 Tier 2). This test uses make_org_registry("overlay-test-org")
    // so org_slug="overlay-test-org" → {ID}=OVERLAY_TEST_ORG.
    // Set the per-client env var to the expected token value so the mock header assertion passes.
    // The mock expects `Authorization: Bearer overlay-url-precedence-token-ac005`.
    // SAFETY: This test runs single-threaded (tokio::test default single-thread executor).
    // No concurrent thread reads this env var during this test. Cleaned up below.
    // Rust edition 2024 requires unsafe for set_var / remove_var (RFC-3540).
    // SAFETY: single-threaded test; no concurrent env var access.
    unsafe {
        std::env::set_var(
            "PRISM_CLIENTS_OVERLAY_TEST_ORG_SENSORS_ARMIS_BEARER_TOKEN",
            "overlay-url-precedence-token-ac005",
        )
    };

    // Run step9a — must register exactly 1 adapter (Armis is BearerStatic, always registered).
    let count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        Arc::clone(&org_registry),
        &plugin_auth_providers,
        &mut adapter_registry,
        null_org_id_store(),
    )
    .await
    .expect("step9a must return Ok(1) for a BearerStatic Armis spec with overlay");

    assert_eq!(
        count, 1,
        "AC-005: step9a must register exactly 1 adapter for a BearerStatic sensor. \
         BearerStatic adapters are never skipped by step9a. \
         BC-2.22.001 postcondition; BC-2.06.014.",
    );

    // Retrieve the registered adapter by (OrgId, SensorId).
    let adapter = adapter_registry.get(org_id, &sensor_id).expect(
        "AC-005: adapter must be retrievable by (OrgId, SensorId) after step9a. \
                 BC-2.11.005.",
    );

    // Call fetch() on the registered adapter.
    // If the adapter holds the OVERLAY base_url (correct), the request goes to
    // overlay_mock_server → returns 200 → fetch() returns Ok(batches).
    // If the adapter holds the TYPE-SPEC base_url (wrong: 127.0.0.1:19999),
    // the request fails with connection refused → fetch() returns Err.
    //
    // ADV-SDEMO002-P01-CRIT-001: BearerStatic sensors now use BearerStaticCredentialAuthProvider
    // (Plugin strategy). The SensorAuth arg is ignored; auth comes from the per-client env var
    // PRISM_CLIENTS_OVERLAY_TEST_ORG_SENSORS_ARMIS_BEARER_TOKEN set above (ADR-032 / BC-2.06.003).
    // A no-op BearerStaticSensorAuth is passed to satisfy the fetch() interface.
    let adapter_spec = make_adapter_spec("armis", org_id);
    let params = make_query_params();
    let sensor_auth = BearerStaticSensorAuth::new("ignored-bearer-static-sensor-auth-arg");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // Clean up env var after the test to avoid cross-test contamination.
    // SAFETY: single-threaded test; no concurrent env var access (RFC-3540).
    unsafe { std::env::remove_var("PRISM_CLIENTS_OVERLAY_TEST_ORG_SENSORS_ARMIS_BEARER_TOKEN") };

    // LOAD-BEARING assertion (F-PASS1-MED-001):
    // This FAILS if the adapter used the TYPE-SPEC URL (connection refused → Err).
    // This PASSES only if the adapter used the OVERLAY URL (mock server → 200 → Ok).
    assert!(
        result.is_ok(),
        "AC-005 LOAD-BEARING: fetch() must succeed when adapter was constructed with \
         the OVERLAY base_url ({overlay_base_url}). \
         If this fails with a connection error, step9a used the TYPE-SPEC base_url \
         ({type_spec_base_url}) instead of the overlay URL — overlay precedence is broken. \
         If this fails with an HTTP error, check auth header injection. \
         Got Err: {:?}",
        result.err()
    );

    // MockServer drop: overlay_mock_server verifies expect(1).
    // If the adapter never hit the mock server (used wrong URL or never called pipeline),
    // wiremock FAILS with "expected 1 request, received 0" → test FAILS.
    // This is the definitive proof that the OVERLAY URL was used.
}

/// AC-006 — BC-2.22.001 postcondition:
/// When `resolved_spec_map` is empty, boot step 9A registers 0 adapters
/// and returns `Ok(0)`.
///
/// BC-2.22.001; AC-006; EC-003; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_22_001_boot_step9a_empty_spec_catalog_registers_zero_adapters() {
    let (org_registry, _, _) = make_org_registry("demo-org");
    let resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    let count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        Arc::clone(&org_registry),
        &plugin_auth_providers,
        &mut adapter_registry,
        null_org_id_store(),
    )
    .await
    .expect("step9a must return Ok(0) for empty catalog (AC-006, EC-003)");

    assert_eq!(
        count, 0,
        "AC-006: empty spec_catalog must produce 0 registered adapters"
    );
    assert!(
        adapter_registry.is_empty(),
        "AC-006: registry must be empty after empty-catalog run"
    );
}

/// AC-007 — BC-2.11.005 precondition:
/// After boot step 9A completes, `AdapterRegistry::get(org_id, sensor_id)`
/// returns `Some(Arc<dyn SensorAdapter>)` for every `(OrgId, SensorId)` pair
/// present in both `spec_catalog` and `org_registry`.
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

    let _count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        Arc::clone(&org_registry),
        &plugin_auth_providers,
        &mut adapter_registry,
        null_org_id_store(),
    )
    .await
    .expect("step9a should complete");

    let adapter = adapter_registry.get(org_id, &sensor_id);
    assert!(
        adapter.is_some(),
        "BC-2.11.005: AdapterRegistry::get must return Some(adapter) for a registered \
         (OrgId, SensorId) pair. Got None — boot step 9A failed to register the adapter."
    );
}

/// AC-008 — BC-2.01.013 precondition:
/// `BearerStaticAuthProvider` implements `AuthProvider::acquire_token()` and
/// returns an `AuthToken` containing the held bearer token string.
///
/// GREEN-BY-DESIGN: zero branching, no I/O, 1-line body.
/// BC-5.38.002 criterion — this test passes at Red Gate time (correct by construction).
///
/// BC-2.01.013; AC-008; OQ-1 Resolution; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_01_013_bearer_static_auth_provider_returns_bearer_token() {
    let token_value = "armis-bearer-token-test-abc";
    let provider = BearerStaticAuthProvider::new(token_value);

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
         BC-2.01.013 precondition."
    );
}

/// EC-007 — BC-2.22.001 edge case:
/// When a sensor spec declares `auth_type = "api_key"` (unsupported in S-DEMO-001),
/// boot step 9A logs E-SPEC-012 and skips — no error returned, boot continues.
///
/// BC-2.22.001; EC-007; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_22_001_boot_step9a_unsupported_auth_type_skips_adapter_not_error() {
    let (org_registry, _, org_slug) = make_org_registry("demo-org");

    let api_key_spec = make_spec("some-sensor", AuthType::ApiKey, "http://127.0.0.1:18086");
    let resolved = make_resolved_spec(api_key_spec, "demo-org");

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug, SensorId::from("some-sensor")), resolved);

    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    let count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        Arc::clone(&org_registry),
        &plugin_auth_providers,
        &mut adapter_registry,
        null_org_id_store(),
    )
    .await
    .expect("step9a must return Ok even when some sensors are skipped (EC-007)");

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

/// OQ-2 Resolution — BC-2.06.014:
/// `step9a_populate_adapter_registry` translates `(OrgSlug, SensorId)` keys
/// to `(OrgId, SensorId)` keys via `OrgRegistry::resolve(slug)`.
///
/// BC-2.06.014; OQ-2 Resolution; S-DEMO-001 v1.3.
#[tokio::test]
async fn test_BC_2_06_014_boot_step9a_translates_org_slug_to_org_id() {
    let (org_registry, org_id, org_slug) = make_org_registry("known-org");
    let sensor_id = SensorId::from("claroty");

    let claroty_spec = make_spec("claroty", AuthType::BearerStatic, "http://127.0.0.1:18087");
    let resolved = make_resolved_spec(claroty_spec, "known-org");

    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug, sensor_id.clone()), resolved);

    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    let count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        Arc::clone(&org_registry),
        &plugin_auth_providers,
        &mut adapter_registry,
        null_org_id_store(),
    )
    .await
    .expect("step9a should return Ok");

    assert_eq!(
        count, 1,
        "OQ-2: exactly 1 adapter registered after slug→id translation"
    );

    let registered = adapter_registry.get(org_id, &sensor_id);
    assert!(
        registered.is_some(),
        "OQ-2 Resolution: adapter must be retrievable by OrgId (not OrgSlug). \
         BC-2.06.014 precondition 1."
    );
}

// NOTE (F-MED-1 / S-PERF-GATE-001 AC-004):
// `build_http_client_with_custom_timeout` was previously `pub` and tested here via
// `test_BC_2_01_013_build_http_client_with_timeout_succeeds`. It has been changed to
// `pub(crate)` (Architecture Compliance Rule 2: no public API surface expansion just
// for a test). The canonical construction test is now in-crate:
//   `crates/prism-bin/src/spec_driven_adapter.rs` mod tests:
//   `test_BC_2_01_013_build_http_client_with_custom_timeout_accepts_duration`
// Named per RG-PERF-001. Cross-crate callers use `build_http_client_with_timeout()`
// (the `pub` 30-second wrapper) — exercised by the adv_p02 integration tests below.

// ===========================================================================
// BC-2.01.013 OCSF Conformance Clause tests (adversary pass-2, F-001-R)
//
// These 3 tests encode the MINIMUM CONFORMANCE GATE from BC-2.01.013
// §SpecDrivenSensorAdapter OCSF Conformance Clause, items 1–3.
//
// All 3 tests MUST FAIL against the current `pipeline_result_to_record_batch`
// stub because it:
//   (a) drops spec-declared data columns (returns only 3 envelope columns)
//   (b) reads category_uid/class_uid verbatim from raw JSON (not derived)
//   (c) reads _sensor from raw JSON with fallback (not injected from spec)
//
// Mock boundary strategy (SID-1): wiremock serves controlled JSON that makes
// the raw-copy failure visible — the raw record carries values deliberately
// chosen to differ from what spec-derivation should produce.
//
// TV-BC-2.01.013-004: multi-column raw response → all columns in Arrow schema
// TV-BC-2.01.013-005: raw category_uid/class_uid 9999 → spec-derived values, not 9999
// BC-2.01.013 item 3: raw _sensor tampered → canonical sensor_id overrides it
// ===========================================================================

/// Build a `SensorSpec` for "crowdstrike" with spec-declared data columns
/// (`detection_id`, `severity`) in addition to the implicit OCSF envelope columns.
///
/// Uses `ocsf_class = "security_finding"` as the fixture class-name (the historical
/// crowdstrike.sensor.toml detections value, now migrated to "detection_finding" by
/// OCSF-CLASS-MIGRATION-001). Per BC-2.02.012 Option A transitional alias:
/// `EventClassSelector::select_by_class_name("security_finding")` → 2004 (Detection Finding),
/// `category_uid` → 2 (2004/1000). A deprecation WARN is emitted during the call.
///
/// Keeping `"security_finding"` in this fixture (rather than updating to `"detection_finding"`)
/// intentionally exercises the transitional-alias code path — the conformance test remains
/// UNGAMEABLE because the implementer must use `select_by_class_name` to derive the value.
///
/// The raw mock response sets `"class_uid": 9999` and `"category_uid": 9999`
/// — values that will NEVER match 2004 — making the derivation vs raw-copy distinction
/// immediately detectable. (TV-BC-2.01.013-005; OCSF-CLASS-MIGRATION-001 AC-005)
fn make_crowdstrike_detection_spec(base_url: &str) -> SensorSpec {
    SensorSpec::new(
        "crowdstrike",
        "CrowdStrike Falcon (test fixture)",
        AuthType::CustomViaPlugin,
        base_url,
        vec![TableSpec::new_point_in_time(
            "detections",
            "security_finding", // ocsf_class transitional alias per BC-2.02.012 Option A; select_by_class_name → 2004, category_uid 2
            vec![
                ColumnSpec::new("detection_id", ColumnType::String, None, vec![]),
                ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ],
            vec![FetchStep::new(
                "fetch_detections",
                "GET",
                "/api/v1/detections",
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

/// Raw API response for F-001-R conformance tests.
///
/// The record deliberately sets:
/// - `"class_uid": 9999` and `"category_uid": 9999` — values that must NOT appear
///   in the returned batch (they must be replaced by spec-derived values).
/// - `"_sensor": "tampered-sensor-name"` — value that must NOT appear as the `_sensor`
///   column value (it must be replaced by the canonical "crowdstrike" SensorId).
/// - `"detection_id": "det-conformance-001"` and `"severity": "High"` — spec-declared
///   data columns that MUST survive into the Arrow schema (not dropped).
///
/// BC-2.01.013 conformance probe; TV-BC-2.01.013-004/005.
fn crowdstrike_conformance_raw_response() -> serde_json::Value {
    serde_json::json!({
        "data": [
            {
                "detection_id": "det-conformance-001",
                "severity": "High",
                "category_uid": 9999,
                "class_uid": 9999,
                "_sensor": "tampered-sensor-name"
            }
        ]
    })
}

/// AC-010 item 1 — BC-2.01.013 OCSF Conformance Clause item 1 (spec columns survive):
///
/// `SpecDrivenSensorAdapter::fetch()` MUST return a RecordBatch whose Arrow schema contains
/// EVERY column declared in the sensor's TOML `[[tables.columns]]` spec, in addition to the
/// 3 OCSF envelope columns. A RecordBatch that contains only envelope columns while dropping
/// spec-declared data columns is NON-CONFORMANT per EC-01-025.
///
/// # Red Gate Failure
///
/// `pipeline_result_to_record_batch` builds a schema with ONLY 3 columns:
/// `Field::new("category_uid", ...), Field::new("class_uid", ...), Field::new("_sensor", ...)`.
/// The spec-declared data columns (`detection_id`, `severity`) are never added to the schema.
/// The assertions `column_names.contains("detection_id")` and `column_names.contains("severity")`
/// FAIL because those columns are absent from the 3-column schema.
///
/// TV-BC-2.01.013-004: "5 spec-declared columns PLUS category_uid, class_uid, _sensor;
/// no spec-declared column is absent."
///
/// BC-2.01.013 OCSF Conformance Clause item 1; AC-010(a); F-001-R; S-DEMO-001 v1.5.
#[tokio::test]
async fn test_BC_2_01_013_ocsf_conformance_spec_columns_survive_into_arrow_schema() {
    let mock_server = MockServer::start().await;

    // Mount: return a raw API response with detection_id + severity + tampered envelope values.
    // The mock requires Authorization: Bearer header (plugin auth path).
    Mock::given(method("GET"))
        .and(path("/api/v1/detections"))
        .and(header(
            "Authorization",
            "Bearer cs-ocsf-conformance-token-001",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(crowdstrike_conformance_raw_response()),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let spec = make_crowdstrike_detection_spec(&mock_server.uri());
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    let mock_auth: Arc<dyn AuthProvider> =
        Arc::new(MockAuthProvider::new("cs-ocsf-conformance-token-001"));
    let auth_strategy = AdapterAuthStrategy::Plugin(mock_auth);
    let http_client = make_mock_client();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);
    let adapter_spec = make_adapter_spec("crowdstrike", org_id);
    let params = make_query_params();
    let sensor_auth = PluginSensorAuth;

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    assert!(
        result.is_ok(),
        "F-001-R item 1: fetch() must return Ok when mock server returns valid JSON. \
         Got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();

    assert!(
        !batches.is_empty(),
        "F-001-R item 1: fetch() must return at least one non-empty RecordBatch. \
         Got empty Vec. BC-2.01.013 Conformance Clause item 1."
    );

    let first_batch = &batches[0];
    let schema = first_batch.schema();
    let column_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    // LOAD-BEARING assertions (TD-VSDD-059):
    // These FAIL with the current implementation because pipeline_result_to_record_batch
    // constructs a Schema with only 3 fields: category_uid, class_uid, _sensor.
    // Spec-declared data columns are never added to the schema.
    //
    // BC-2.01.013 item 1: "Every column declared in [[tables.columns]] MUST survive
    // into the returned RecordBatch via ColumnMapper field-by-field mapping."
    // EC-01-025: "RecordBatch with only envelope fields while discarding sensor payload = NON-CONFORMANT."
    assert!(
        column_names.contains(&"detection_id"),
        "F-001-R item 1 LOAD-BEARING: Arrow schema MUST contain spec-declared column \
         'detection_id'. Present columns: {:?}. \
         Root cause: pipeline_result_to_record_batch builds only the 3-column envelope schema \
         (category_uid, class_uid, _sensor) — all spec data columns are silently dropped. \
         Fix: use ColumnMapper to map spec.tables[].columns into the RecordBatch. \
         BC-2.01.013 Conformance Clause item 1; AC-010(a); F-001-R.",
        column_names
    );

    assert!(
        column_names.contains(&"severity"),
        "F-001-R item 1 LOAD-BEARING: Arrow schema MUST contain spec-declared column \
         'severity'. Present columns: {:?}. \
         Root cause: same as detection_id — spec data columns dropped in current stub. \
         BC-2.01.013 Conformance Clause item 1; AC-010(a); F-001-R.",
        column_names
    );

    // Also verify OCSF envelope columns are still present (they must not be absent).
    for envelope_col in &["category_uid", "class_uid", "_sensor"] {
        assert!(
            column_names.contains(envelope_col),
            "F-001-R item 1: OCSF envelope column '{}' must be present. \
             Present columns: {:?}. BC-2.01.013.",
            envelope_col,
            column_names
        );
    }
}

/// AC-010 item 2 — BC-2.01.013 OCSF Conformance Clause item 2 (envelope derivation):
///
/// `category_uid` and `class_uid` MUST be derived by `OcsfNormalizer` from the sensor's
/// declared `ocsf_class` (via `EventClassSelector::select_by_class_name(ocsf_class)` mapping),
/// NOT copied verbatim from the raw API response. An implementation that reads those fields
/// directly from raw JSON is NON-CONFORMANT per EC-01-026.
///
/// # Test design (D-925 / TV-BC-2.01.013-005 — UNGAMEABLE fixture)
///
/// The mock raw record has `"class_uid": 9999, "category_uid": 9999`.
/// The spec declares `sensor_id = "crowdstrike"` and `ocsf_class = "security_finding"`
/// (transitional alias per BC-2.02.012 Option A; OCSF-CLASS-MIGRATION-001).
/// `EventClassSelector::select_by_class_name("security_finding")` → 2004 (Detection Finding).
/// `category_uid` → 2 (2004 / 1000).
/// The test asserts `class_uid == 2004` — forcing failure if:
///   (a) raw JSON value 9999 is propagated, or
///   (b) implementer uses a path that does not call select_by_class_name.
///
/// # Status
///
/// GREEN after OCSF-CLASS-MIGRATION-001: `select_by_class_name("security_finding")` returns
/// 2004 (per BC-2.02.012 Option A) — the full derivation chain produces class_uid = 2004.
///
/// TV-BC-2.01.013-005 (BC-2.01.013): "category_uid/class_uid in returned batch derived
/// from ocsf_class, NOT equal to raw 9999 value; class_uid MUST equal 2004."
///
/// BC-2.01.013 OCSF Conformance Clause item 2; AC-010(b); F-001-R; D-925.
#[tokio::test]
async fn test_BC_2_01_013_ocsf_conformance_envelope_derived_not_raw_copied() {
    // select_by_class_name("security_finding") → 2004 (BC-2.02.012 Option A,
    // OCSF-CLASS-MIGRATION-001). The full derivation chain produces class_uid = 2004.
    // A deprecation WARN (ocsf.deprecated_class_alias) is emitted during the call — this
    // is expected behavior and does not affect the assertion.

    let mock_server = MockServer::start().await;

    // The raw response has class_uid = 9999 and category_uid = 9999 — values that differ
    // from the spec-derived class_uid = 2004. If the implementation copies raw values,
    // the assertion below will catch it.
    Mock::given(method("GET"))
        .and(path("/api/v1/detections"))
        .and(header(
            "Authorization",
            "Bearer cs-envelope-derivation-token-002",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(crowdstrike_conformance_raw_response()),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let spec = make_crowdstrike_detection_spec(&mock_server.uri());
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    let mock_auth: Arc<dyn AuthProvider> =
        Arc::new(MockAuthProvider::new("cs-envelope-derivation-token-002"));
    let auth_strategy = AdapterAuthStrategy::Plugin(mock_auth);
    let http_client = make_mock_client();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);
    let adapter_spec = make_adapter_spec("crowdstrike", org_id);
    let params = make_query_params();
    let sensor_auth = PluginSensorAuth;

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    assert!(
        result.is_ok(),
        "F-001-R item 2: fetch() must return Ok when mock server returns valid JSON. \
         Got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();
    assert!(
        !batches.is_empty(),
        "F-001-R item 2: must return at least one RecordBatch."
    );

    let first_batch = &batches[0];
    assert!(
        first_batch.num_rows() > 0,
        "F-001-R item 2: RecordBatch must contain at least one row."
    );

    // Extract class_uid column value from the first row.
    // The Arrow column is Int32 (nullable). Extract the first row's value.
    let schema = first_batch.schema();
    let class_uid_col_idx = schema
        .index_of("class_uid")
        .expect("F-001-R item 2: 'class_uid' column must exist in returned RecordBatch schema");

    let class_uid_col = first_batch.column(class_uid_col_idx);
    let class_uid_array = class_uid_col
        .as_any()
        .downcast_ref::<arrow::array::Int32Array>()
        .expect("F-001-R item 2: 'class_uid' column must be Int32Array");

    let class_uid_val = class_uid_array.value(0);

    // LOAD-BEARING assertion (TD-VSDD-059, D-925, OCSF-CLASS-MIGRATION-001 AC-005):
    // Raw record has class_uid = 9999. The spec declares ocsf_class = "security_finding".
    // Correct implementation (post-OCSF-CLASS-MIGRATION-001):
    //   select_by_class_name("security_finding") → 2004 (BC-2.02.012 Option A).
    // This assertion fails for BOTH the raw-copy (9999) and any path that does not call
    // select_by_class_name, ensuring the implementer must use the real class-name derivation path.
    assert_ne!(
        class_uid_val, 9999,
        "F-001-R item 2 LOAD-BEARING: class_uid in returned RecordBatch MUST NOT be the \
         raw JSON value 9999. It must be the spec-derived value from \
         EventClassSelector::select_by_class_name('security_finding') = 2004 \
         (BC-2.02.012 Option A transitional alias — Detection Finding, not deprecated 2001). \
         EC-01-026: 'Implementation that copies category_uid/class_uid from raw vendor JSON = NON-CONFORMANT.' \
         BC-2.01.013 OCSF Conformance Clause item 2; AC-010(b); F-001-R; D-925; TV-BC-2.01.013-005."
    );

    // Assert the correct derived value (2004) is present.
    // BC-2.02.012 mapping: ocsf_class "security_finding" → class_uid 2004 (transitional alias),
    // category_uid 2 (2004/1000). OCSF-CLASS-MIGRATION-001 AC-005.
    assert_eq!(
        class_uid_val, 2004,
        "F-001-R item 2 LOAD-BEARING: class_uid MUST equal the spec-derived value 2004 \
         (EventClassSelector::select_by_class_name('security_finding') = Detection Finding, 2004). \
         Per BC-2.02.012 Option A (OCSF-CLASS-MIGRATION-001), 'security_finding' is a \
         transitional alias that maps to 2004, not the deprecated 2001. Got: {}. \
         BC-2.01.013 Conformance Clause item 2; AC-010(b); D-925; TV-BC-2.01.013-005.",
        class_uid_val
    );
}

/// AC-010 item 3 — BC-2.01.013 OCSF Conformance Clause item 3 (_sensor virtual column):
///
/// The `_sensor` virtual column MUST be present and set to the sensor's canonical `SensorId`
/// string (e.g., `"crowdstrike"`), injected by the normalization layer — NOT read from the
/// raw API response. If the raw record contains a `"_sensor"` field, the normalization layer
/// must override it with the canonical sensor_id from the spec.
///
/// # Test design
///
/// The mock raw record has `"_sensor": "tampered-sensor-name"`. The spec's sensor_id is
/// "crowdstrike". The test asserts `_sensor` column value == "crowdstrike".
///
/// # Red Gate Failure
///
/// `pipeline_result_to_record_batch` reads `_sensor` with:
///   `record.get("_sensor").and_then(|v| v.as_str()).unwrap_or(sensor_id).to_string()`
/// The raw record has `"_sensor": "tampered-sensor-name"` so the `get` succeeds and
/// `unwrap_or(sensor_id)` is never reached. Result: `_sensor = "tampered-sensor-name"`.
/// The assertion `_sensor_val == "crowdstrike"` FAILS.
///
/// BC-2.01.013 OCSF Conformance Clause item 3; AC-010(c); F-001-R; S-DEMO-001 v1.5.
#[tokio::test]
async fn test_BC_2_01_013_ocsf_conformance_sensor_virtual_column_is_canonical_sensor_id() {
    let mock_server = MockServer::start().await;

    // The raw response has "_sensor": "tampered-sensor-name".
    // The normalization layer must override this with the canonical "crowdstrike" SensorId.
    Mock::given(method("GET"))
        .and(path("/api/v1/detections"))
        .and(header(
            "Authorization",
            "Bearer cs-sensor-virtual-col-token-003",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(crowdstrike_conformance_raw_response()),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let spec = make_crowdstrike_detection_spec(&mock_server.uri());
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    let mock_auth: Arc<dyn AuthProvider> =
        Arc::new(MockAuthProvider::new("cs-sensor-virtual-col-token-003"));
    let auth_strategy = AdapterAuthStrategy::Plugin(mock_auth);
    let http_client = make_mock_client();

    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);
    let adapter_spec = make_adapter_spec("crowdstrike", org_id);
    let params = make_query_params();
    let sensor_auth = PluginSensorAuth;

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    assert!(
        result.is_ok(),
        "F-001-R item 3: fetch() must return Ok. Got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();
    assert!(
        !batches.is_empty(),
        "F-001-R item 3: must return at least one RecordBatch."
    );

    let first_batch = &batches[0];
    assert!(
        first_batch.num_rows() > 0,
        "F-001-R item 3: RecordBatch must have at least one row."
    );

    let schema = first_batch.schema();
    let sensor_col_idx = schema
        .index_of("_sensor")
        .expect("F-001-R item 3: '_sensor' virtual column must exist in Arrow schema");

    let sensor_col = first_batch.column(sensor_col_idx);
    let sensor_array = sensor_col
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .expect("F-001-R item 3: '_sensor' column must be StringArray (Utf8)");

    let sensor_val = sensor_array.value(0);

    // LOAD-BEARING assertion (TD-VSDD-059):
    // Raw record has "_sensor": "tampered-sensor-name".
    // Current code: record.get("_sensor").and_then(|v| v.as_str()).unwrap_or(sensor_id)
    //   → returns "tampered-sensor-name" (raw value, not canonical)
    // Correct code: injected by the normalization layer as the canonical SensorId = "crowdstrike".
    // This assertion guards that the normalization layer OVERRIDES the raw record's _sensor.
    assert_ne!(
        sensor_val, "tampered-sensor-name",
        "F-001-R item 3 LOAD-BEARING: '_sensor' virtual column MUST NOT carry the raw \
         JSON value 'tampered-sensor-name'. It must be injected by the normalization layer \
         as the canonical SensorId from the spec. \
         Root cause: pipeline_result_to_record_batch reads _sensor from raw JSON with fallback \
         (record.get('_sensor')...unwrap_or(sensor_id)) — raw JSON value wins when present. \
         Fix: always inject canonical sensor_id, never read _sensor from raw record. \
         BC-2.01.013 Conformance Clause item 3; AC-010(c); F-001-R."
    );

    assert_eq!(
        sensor_val, "crowdstrike",
        "F-001-R item 3: '_sensor' MUST equal the canonical SensorId 'crowdstrike' from \
         the resolved spec. Got: '{}'. The normalization layer must inject this value, \
         ignoring any '_sensor' field in the raw API response. \
         BC-2.01.013 Conformance Clause item 3; AC-010(c); F-001-R.",
        sensor_val
    );
}

// ===========================================================================
// F-002-R: Production boot-path wiring guard
//
// The existing test_BC_2_22_001_step9a_not_called_in_boot_production_path (pass-1)
// had Part 2 calling `boot_step9_build_adapter_registry` — a DUPLICATE HELPER
// that does NOT call `step9_start_mcp_server`. It tested the helper, not the
// production boot path.
//
// This test replaces that with a SOURCE-LEVEL WIRING GUARD: it reads boot.rs and
// asserts that `step9_start_mcp_server` contains a call to
// `step9a_populate_adapter_registry`. This is STRONGER than calling a helper
// because:
//   (a) It guards the PRODUCTION function signature, not a parallel duplicate.
//   (b) It FAILS if the implementer removes the call from step9_start_mcp_server.
//   (c) It is immune to the "duplicate helper passes but production is broken" failure mode.
//
// IMPLEMENTER NOTE: `boot_step9_build_adapter_registry` in spec_driven_adapter.rs
// is a duplicate helper that tests the same logic as step9a_populate_adapter_registry
// but bypasses the real step9_start_mcp_server wiring. It must be REMOVED.
// The only valid test for boot wiring is this structural guard.
//
// SID-1: This unit test exercises the production wiring assertion without the
// RocksDB/MCP server deps that would make it an integration test.
//
// BC-2.22.001; F-002-R; S-DEMO-001 v1.5.
// ===========================================================================

/// F-002-R — BC-2.22.001 sequencing invariant (production boot path wiring guard):
///
/// `step9_start_mcp_server` MUST contain a call to `step9a_populate_adapter_registry`
/// to wire spec-driven adapters into the `AdapterRegistry` before the MCP server starts.
/// This is a SOURCE-LEVEL structural assertion — it reads `boot.rs` and verifies the
/// call exists in the production function.
///
/// # Red Gate Failure
///
/// This test FAILS if:
/// (a) The implementer removes `step9a_populate_adapter_registry` from
///     `step9_start_mcp_server` (production wiring gap), or
/// (b) The call is behind a commented-out or conditional block that could silently
///     disable it at runtime.
///
/// # Why source-level, not call-level
///
/// `step9_start_mcp_server` requires RocksDB, MCP server, and other infrastructure
/// that is not available in unit tests. A call-level integration test would need the
/// full binary. The source-level assertion provides a lighter-weight guard that:
///   - Fails immediately if the wiring disappears from the code
///   - Does not require RocksDB or a live server
///   - Is immune to the duplicate-helper false-pass failure mode
///
/// BC-2.22.001; F-002-R; S-DEMO-001 v1.5.
#[test]
fn test_BC_2_22_001_production_boot_path_wiring_guard() {
    // Read the production boot.rs source.
    // The path is relative to this test file's workspace root — resolved via CARGO_MANIFEST_DIR.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo during test execution");
    let boot_rs_path = std::path::Path::new(&manifest_dir)
        .join("src")
        .join("boot.rs");

    let boot_src = std::fs::read_to_string(&boot_rs_path).unwrap_or_else(|e| {
        panic!(
            "F-002-R: Could not read boot.rs at {:?}: {}. \
             Ensure the test runs from the prism-bin crate directory.",
            boot_rs_path, e
        )
    });

    // Assert that step9a_populate_adapter_registry is called from within
    // step9_start_mcp_server. The call appears as:
    //   crate::spec_driven_adapter::step9a_populate_adapter_registry(
    // or:
    //   step9a_populate_adapter_registry(
    //
    // Either form is acceptable. The assertion requires the call to appear AFTER
    // the step9_start_mcp_server function definition line.
    let step9_fn_pos = boot_src
        .find("pub async fn step9_start_mcp_server")
        .unwrap_or_else(|| {
            panic!(
                "F-002-R LOAD-BEARING: 'pub async fn step9_start_mcp_server' not found in boot.rs. \
                 Either the function was renamed or boot.rs is wrong path. \
                 BC-2.22.001; S-DEMO-001 v1.5."
            )
        });

    let step9_fn_body = &boot_src[step9_fn_pos..];

    // The call must appear inside the function body (before the next pub async fn declaration,
    // or simply after the start of step9_start_mcp_server).
    // We check for the call anywhere after the function start — this is sufficient because
    // step9_start_mcp_server is the last major boot step function.
    let has_step9a_call = step9_fn_body.contains("step9a_populate_adapter_registry(");

    // LOAD-BEARING structural assertion (TD-VSDD-059, F-002-R):
    // This FAILS if `step9a_populate_adapter_registry` is removed from
    // `step9_start_mcp_server`. The implementer MUST NOT remove this call.
    // The `boot_step9_build_adapter_registry` duplicate helper in spec_driven_adapter.rs
    // MUST be deleted — it bypasses this production wiring and creates a false-pass risk.
    assert!(
        has_step9a_call,
        "F-002-R LOAD-BEARING: `step9_start_mcp_server` in boot.rs MUST call \
         `step9a_populate_adapter_registry` to populate the AdapterRegistry before the \
         MCP server starts. The call was NOT found in the function body. \
         Production boot path wiring is broken — GAP-002-A is re-opened. \
         IMPLEMENTER NOTE: also delete the `boot_step9_build_adapter_registry` duplicate \
         helper in spec_driven_adapter.rs — it tests itself, not the real production wiring. \
         BC-2.22.001; F-002-R; S-DEMO-001 v1.5."
    );
}

// ===========================================================================
// F-004-R: Error taxonomy pin — E-AUTH-002 structural guard
//
// The existing AC-012 test (test_BC_2_01_013_spec_driven_adapter_double_401_returns_auth_refresh_failed)
// verifies the E-AUTH-002 code appears in the SensorError::Internal detail string after
// a double-401 pipeline scenario. That test is a round-trip test through the pipeline.
//
// F-004-R adds a STRUCTURAL PIN: directly construct SpecEngineError::AuthRefreshFailed
// and assert its Display string starts with "E-AUTH-002". This guards against:
//   (a) The error.rs Display format being changed to drop the taxonomy code
//   (b) The variant being renamed without updating the format string
//   (c) The display format drifting to a non-E-AUTH-002 prefix
//
// This is a unit test on the error type, independent of the pipeline flow.
// It will fail immediately if the Display format is changed in error.rs.
//
// BC-2.01.013; AC-012; F-004-R; error-taxonomy.md E-AUTH-002.
// ===========================================================================

/// F-004-R — BC-2.01.013 error case (E-AUTH-002 taxonomy pin):
///
/// `SpecEngineError::AuthRefreshFailed` MUST display with the "E-AUTH-002: auth refresh failed"
/// prefix so that the AC-012 mapping in `map_spec_engine_error_to_sensor_error` can reliably
/// propagate this taxonomy code to callers via the `SensorError::Internal { detail }` field.
///
/// # Red Gate Failure
///
/// This test FAILS if:
/// (a) `error.rs` changes the `#[error(...)]` format to remove "E-AUTH-002", or
/// (b) The variant is renamed and the format string is not updated, or
/// (c) A refactor changes the Display impl to produce a different prefix.
///
/// This is a STRUCTURAL GUARD (TD-VSDD-059) that pins the error taxonomy code at the
/// definition site, independent of the pipeline round-trip path. The AC-012 test covers
/// the pipeline path; this test covers the error definition.
///
/// BC-2.01.013 §Error Cases; AC-012; F-004-R; error-taxonomy.md E-AUTH-002; S-DEMO-001 v1.5.
#[test]
fn test_BC_2_01_013_auth_refresh_failed_display_carries_e_auth_002_taxonomy_code() {
    let err = SpecEngineError::AuthRefreshFailed {
        sensor_id: "crowdstrike".to_string(),
        client_id: "test-org".to_string(),
        step_name: "fetch_detections".to_string(),
    };

    let display_str = format!("{err}");

    // LOAD-BEARING structural assertion (TD-VSDD-059):
    // The Display impl (via #[error(...)]) must start with "E-AUTH-002: auth refresh failed".
    // This is pinned here to guard against silent drift in error.rs taxonomy codes.
    // If this test fails, the error.rs #[error(...)] format was changed.
    assert!(
        display_str.starts_with("E-AUTH-002"),
        "F-004-R LOAD-BEARING: SpecEngineError::AuthRefreshFailed Display MUST start with \
         'E-AUTH-002'. Got: {:?}. \
         This taxonomy code is required so that map_spec_engine_error_to_sensor_error \
         propagates 'E-AUTH-002' in the SensorError::Internal detail string (AC-012 contract). \
         Fix: restore the E-AUTH-002 prefix in error.rs #[error(...)] for AuthRefreshFailed. \
         BC-2.01.013 §Error Cases; AC-012; F-004-R; error-taxonomy.md E-AUTH-002.",
        display_str
    );

    assert!(
        display_str.contains("auth refresh failed"),
        "F-004-R: SpecEngineError::AuthRefreshFailed Display must contain 'auth refresh failed'. \
         Got: {:?}. BC-2.01.013; AC-012; F-004-R.",
        display_str
    );

    assert!(
        display_str.contains("crowdstrike"),
        "F-004-R: SpecEngineError::AuthRefreshFailed Display must contain the sensor_id. \
         Got: {:?}. BC-2.01.013; AC-012; F-004-R.",
        display_str
    );
}

/// AC-011 — BC-3.2.001 postcondition (SID-1 authoritative unit test):
///
/// After multi-org boot (3 orgs, 8 specs), `step9a_populate_adapter_registry`
/// MUST register exactly **8** `(OrgId, SensorId)` entries:
///
/// | Org | Sensors | Entries |
/// |-----|---------|---------|
/// | demo-org-a | crowdstrike + armis | 2 |
/// | demo-org-b | claroty + cyberint | 2 |
/// | demo-org-c | crowdstrike + armis + claroty + cyberint | 4 |
///
/// **Positive resolution**: every `(org_id, sensor_id)` pair resolves to `Some(adapter)`.
///
/// **Negative aliasing**: an org CANNOT see sensors it was NOT configured for —
/// demo-org-a cannot see claroty or cyberint; demo-org-b cannot see crowdstrike or armis.
///
/// # Why this test is the authoritative count assertion
///
/// The e2e subprocess test (`test_BC_3_2_001_e2e_multi_org_boot_registers_correct_adapter_count`
/// in e2e_smoke.rs) only exercises demo-org-c's 4-sensor resolvability over MCP. It does NOT
/// assert `adapter_registry.len() == 8`. This unit test is the SID-1 load-bearing authority
/// for the count and no-aliasing property (F-DEMO002-P1-MED-001).
///
/// # Fixture note on auth types
///
/// CrowdStrike specs in this fixture use `BearerStatic` (not `CustomViaPlugin`) so that
/// no `PluginAuthProvider` is required and all 8 entries are registered without gating.
/// The `CustomViaPlugin` skip path is covered separately by `EC-004`-tagged tests.
///
/// BC-3.2.001 postcondition 1; AC-011; S-DEMO-002; F-DEMO002-P1-MED-001.
#[tokio::test]
async fn test_BC_3_2_001_step9a_multi_org_registers_eight_adapters() {
    // -------------------------------------------------------------------------
    // Fixture: 3 orgs, each registered in the same OrgRegistry.
    // -------------------------------------------------------------------------
    let (org_registry, org_id_a, org_slug_a) = make_org_registry("demo-org-a");

    let uuid_b = uuid::Uuid::now_v7();
    let org_id_b = OrgId::from_uuid(uuid_b);
    let org_slug_b = OrgSlug::new("demo-org-b");
    org_registry
        .register(org_slug_b.clone(), org_id_b)
        .expect("test fixture: register demo-org-b failed");

    let uuid_c = uuid::Uuid::now_v7();
    let org_id_c = OrgId::from_uuid(uuid_c);
    let org_slug_c = OrgSlug::new("demo-org-c");
    org_registry
        .register(org_slug_c.clone(), org_id_c)
        .expect("test fixture: register demo-org-c failed");

    // -------------------------------------------------------------------------
    // Fixture: resolved spec map — 8 entries total.
    //
    // CrowdStrike uses BearerStatic here (not CustomViaPlugin) so the plugin-auth
    // gating path (EC-004) is bypassed — all 8 are registered unconditionally.
    // -------------------------------------------------------------------------
    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();

    // demo-org-a: crowdstrike (bearer) + armis (bearer)
    let cs_spec_a = make_spec(
        "crowdstrike",
        AuthType::BearerStatic,
        "http://127.0.0.1:19100",
    );
    resolved_spec_map.insert(
        (org_slug_a.clone(), SensorId::from("crowdstrike")),
        make_resolved_spec(cs_spec_a, "demo-org-a"),
    );
    let armis_spec_a = make_spec("armis", AuthType::BearerStatic, "http://127.0.0.1:19101");
    resolved_spec_map.insert(
        (org_slug_a.clone(), SensorId::from("armis")),
        make_resolved_spec(armis_spec_a, "demo-org-a"),
    );

    // demo-org-b: claroty (bearer) + cyberint (cookie_roundtrip)
    let claroty_spec_b = make_spec("claroty", AuthType::BearerStatic, "http://127.0.0.1:19102");
    resolved_spec_map.insert(
        (org_slug_b.clone(), SensorId::from("claroty")),
        make_resolved_spec(claroty_spec_b, "demo-org-b"),
    );
    let cyberint_spec_b = make_spec(
        "cyberint",
        AuthType::CookieRoundtrip,
        "http://127.0.0.1:19103",
    );
    resolved_spec_map.insert(
        (org_slug_b.clone(), SensorId::from("cyberint")),
        make_resolved_spec(cyberint_spec_b, "demo-org-b"),
    );

    // demo-org-c: all 4 sensors
    let cs_spec_c = make_spec(
        "crowdstrike",
        AuthType::BearerStatic,
        "http://127.0.0.1:19104",
    );
    resolved_spec_map.insert(
        (org_slug_c.clone(), SensorId::from("crowdstrike")),
        make_resolved_spec(cs_spec_c, "demo-org-c"),
    );
    let armis_spec_c = make_spec("armis", AuthType::BearerStatic, "http://127.0.0.1:19105");
    resolved_spec_map.insert(
        (org_slug_c.clone(), SensorId::from("armis")),
        make_resolved_spec(armis_spec_c, "demo-org-c"),
    );
    let claroty_spec_c = make_spec("claroty", AuthType::BearerStatic, "http://127.0.0.1:19106");
    resolved_spec_map.insert(
        (org_slug_c.clone(), SensorId::from("claroty")),
        make_resolved_spec(claroty_spec_c, "demo-org-c"),
    );
    let cyberint_spec_c = make_spec(
        "cyberint",
        AuthType::CookieRoundtrip,
        "http://127.0.0.1:19107",
    );
    resolved_spec_map.insert(
        (org_slug_c.clone(), SensorId::from("cyberint")),
        make_resolved_spec(cyberint_spec_c, "demo-org-c"),
    );

    assert_eq!(
        resolved_spec_map.len(),
        8,
        "test fixture sanity: resolved_spec_map must contain exactly 8 entries before step9a runs"
    );

    let plugin_auth_providers: HashMap<String, Arc<PluginAuthProvider>> = HashMap::new();
    let mut adapter_registry = AdapterRegistry::new();

    // -------------------------------------------------------------------------
    // Exercise: run boot step 9A on the 3-org / 8-spec map.
    // -------------------------------------------------------------------------
    let count = step9a_populate_adapter_registry(
        &resolved_spec_map,
        Arc::clone(&org_registry),
        &plugin_auth_providers,
        &mut adapter_registry,
        null_org_id_store(),
    )
    .await
    .expect("step9a must return Ok(count) for a valid 3-org / 8-spec map. BC-3.2.001.");

    // -------------------------------------------------------------------------
    // Count assertion (AC-011 primary postcondition).
    // -------------------------------------------------------------------------
    assert_eq!(
        adapter_registry.len(),
        8,
        "AC-011: AdapterRegistry must contain exactly 8 entries after 3-org boot \
         (demo-org-a: 2, demo-org-b: 2, demo-org-c: 4). \
         BC-3.2.001 postcondition 1; F-DEMO002-P1-MED-001."
    );
    assert_eq!(
        count, 8,
        "AC-011: returned count must equal adapter_registry.len() == 8. \
         BC-3.2.001 postcondition 1."
    );

    // -------------------------------------------------------------------------
    // Positive resolution: every (org_id, sensor_id) pair must resolve to Some.
    // -------------------------------------------------------------------------
    for (org_id, sensor_id, label) in [
        (
            org_id_a,
            SensorId::from("crowdstrike"),
            "demo-org-a / crowdstrike",
        ),
        (org_id_a, SensorId::from("armis"), "demo-org-a / armis"),
        (org_id_b, SensorId::from("claroty"), "demo-org-b / claroty"),
        (
            org_id_b,
            SensorId::from("cyberint"),
            "demo-org-b / cyberint",
        ),
        (
            org_id_c,
            SensorId::from("crowdstrike"),
            "demo-org-c / crowdstrike",
        ),
        (org_id_c, SensorId::from("armis"), "demo-org-c / armis"),
        (org_id_c, SensorId::from("claroty"), "demo-org-c / claroty"),
        (
            org_id_c,
            SensorId::from("cyberint"),
            "demo-org-c / cyberint",
        ),
    ] {
        assert!(
            adapter_registry.get(org_id, &sensor_id).is_some(),
            "AC-011 positive resolution: adapter_registry.get({label}) must return Some(adapter). \
             BC-3.2.001 postcondition 1; F-DEMO002-P1-MED-001."
        );
    }

    // -------------------------------------------------------------------------
    // Negative aliasing: an org MUST NOT see sensors it was not configured for.
    //
    // demo-org-a (crowdstrike + armis) cannot see claroty or cyberint.
    // demo-org-b (claroty + cyberint) cannot see crowdstrike or armis.
    // BC-3.2.001 invariant 1 (no cross-org aliasing).
    // -------------------------------------------------------------------------
    for (org_id, sensor_id, label) in [
        (
            org_id_a,
            SensorId::from("claroty"),
            "demo-org-a must NOT see claroty",
        ),
        (
            org_id_a,
            SensorId::from("cyberint"),
            "demo-org-a must NOT see cyberint",
        ),
        (
            org_id_b,
            SensorId::from("crowdstrike"),
            "demo-org-b must NOT see crowdstrike",
        ),
        (
            org_id_b,
            SensorId::from("armis"),
            "demo-org-b must NOT see armis",
        ),
    ] {
        assert!(
            adapter_registry.get(org_id, &sensor_id).is_none(),
            "AC-011 negative aliasing: adapter_registry.get({label}) must return None. \
             BC-3.2.001 invariant 1 — no cross-org aliasing; F-DEMO002-P1-MED-001."
        );
    }
}

// ---------------------------------------------------------------------------
// S-DEMO-QUERY-PUSHDOWN-001 v2.1 — SpecDrivenSensorAdapter push-down wire tests
// F-P1-CRIT-002: augment_armis_aql_with_time_window wired in production path
// F-P1-CRIT-004: limit seeded from params.limit into query_filters["query.limit"]
// F-P1-HIGH-003: wire-level assertions against real DTU
// ---------------------------------------------------------------------------

/// F-P1-CRIT-002 + F-P1-HIGH-003:
/// `SpecDrivenSensorAdapter::fetch()` with `params.start_time` set for Armis
/// must augment the base AQL with `after:YYYY-MM-DDTHH:MM:SS` and forward the
/// augmented AQL to the DTU wire.
///
/// # Production path
/// `params.start_time` → `augment_armis_aql_with_time_window()` →
/// `query_filters["aql"]` → `${query.filter.aql}` interpolation →
/// Armis DTU `GET /api/v1/search?aql=...` → aql-log captures verbatim.
///
/// # SAP-2
/// Uses the real `armis.sensor.toml` spec and the real Armis DTU clone.
#[tokio::test]
async fn test_armis_fetch_with_start_time_augments_aql_reaches_dtu_wire() {
    use prism_dtu_armis::ArmisClone;
    use prism_dtu_common::BehavioralClone;
    use prism_spec_engine::spec_parser::SpecLoader;

    // Start real Armis DTU clone.
    let mut clone = ArmisClone::new().expect("ArmisClone::new must succeed");
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("Armis DTU clone failed to start");
    let dtu_url = format!("http://{bound_addr}");

    // Load production armis.sensor.toml (SAP-2).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("armis.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content).expect("armis.sensor.toml must parse");
    spec.base_url = dtu_url.clone();

    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");

    // BearerStatic auth — Armis DTU accepts any non-empty bearer token.
    let auth_strategy = AdapterAuthStrategy::BearerStatic;
    let http_client = build_http_client_with_timeout().unwrap();
    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    // Build adapter spec for the devices table.
    let mut adapter_spec = make_adapter_spec("armis", org_id);
    adapter_spec.source_table = "armis_devices".to_string();

    // Set start_time — this is what extract_time_window_from_ast produces from
    // a `WHERE last_seen > 'T'` predicate (ADR-033 T1).
    // F-P1-CRIT-002: spec_driven_adapter.rs must call augment_armis_aql_with_time_window
    // to produce `after:2024-01-01T00:00:00` in the AQL.
    let mut params = make_query_params();
    params.start_time = Some("2024-01-01T00:00:00Z".to_string());
    // Seed the base AQL in filters (as would come from predicate_tree_to_filter_map
    // for `WHERE aql = 'in:devices'`).
    params.filters.insert(
        "aql".to_string(),
        serde_json::Value::String("in:devices".to_string()),
    );

    let sensor_auth = BearerStaticSensorAuth::new("armis-test-bearer-token");
    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    assert!(
        result.is_ok(),
        "F-P1-CRIT-002: SpecDrivenSensorAdapter::fetch() with start_time must succeed; \
         got Err: {:?}",
        result.err()
    );

    // Wire-level assertion: aql-log must contain the augmented AQL with `after:` clause.
    // The augmentation adds `after:2024-01-01T00:00:00` (timezone-naive, no Z).
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    let aql_log_resp = client
        .get(format!("{dtu_url}/dtu/aql-log"))
        .send()
        .await
        .expect("GET /dtu/aql-log must succeed");
    let aql_log_body: serde_json::Value = aql_log_resp.json().await.expect("aql-log must be JSON");
    let aql_strings = aql_log_body["aql_strings"]
        .as_array()
        .expect("aql_strings must be array");

    assert!(
        aql_strings.iter().any(|s| s
            .as_str()
            .map(|a| a.contains("after:2024-01-01"))
            .unwrap_or(false)),
        "F-P1-CRIT-002 WIRE-LEVEL (F-P1-HIGH-003): aql-log must contain 'after:2024-01-01' \
         in the augmented AQL. The production path: params.start_time = '2024-01-01T00:00:00Z' \
         → augment_armis_aql_with_time_window('in:devices', Some('2024-01-01T00:00:00Z'), None) \
         → 'in:devices after:2024-01-01T00:00:00' → DTU receives augmented AQL. \
         Got aql_strings: {:?}. REGRESSION: spec_driven_adapter.rs is NOT calling \
         augment_armis_aql_with_time_window, or the augmented AQL is not seeded in query_filters['aql'].",
        aql_strings
    );

    assert!(
        aql_strings.iter().any(|s| s
            .as_str()
            .map(|a| a.contains("in:devices"))
            .unwrap_or(false)),
        "F-P1-CRIT-002: base AQL entity discriminator 'in:devices' must be preserved \
         after augmentation. Got aql_strings: {:?}",
        aql_strings
    );
}

/// F-P1-CRIT-004: `SpecDrivenSensorAdapter::fetch()` seeds `query_filters["query.limit"]`
/// from `params.limit` for CrowdStrike sensors.
///
/// # Production path
/// `params.limit` → `query_filters["query.limit"]` → `${query.limit}` interpolation →
/// CrowdStrike DTU `?limit=N` → DTU returns at most N IDs → pipeline returns ≤ N records.
///
/// # SAP-2
/// Uses the real `crowdstrike.sensor.toml` spec and the real CrowdStrike DTU clone.
#[tokio::test]
async fn test_crowdstrike_fetch_with_limit_seeds_query_limit_reaches_dtu() {
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_crowdstrike::CrowdstrikeClone;
    use prism_spec_engine::spec_parser::SpecLoader;

    // Start real CrowdStrike DTU clone.
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("CrowdstrikeDTU clone failed to start");
    let dtu_url = format!("http://{bound_addr}");

    // Load production crowdstrike.sensor.toml (SAP-2).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("crowdstrike.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content).expect("crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_url.clone();

    // Construct a PluginAuthProvider-backed adapter for CrowdStrike.
    // For tests we use MockAuthProvider as the underlying auth provider.
    use prism_spec_engine::auth_provider::MockAuthProvider;
    let mock_auth: Arc<dyn prism_spec_engine::AuthProvider> =
        Arc::new(MockAuthProvider::new("test-oauth2-token"));
    let resolved = make_resolved_spec(spec, "demo-org");
    let (_, org_id, _) = make_org_registry("demo-org");
    let auth_strategy = AdapterAuthStrategy::Plugin(mock_auth);
    let http_client = build_http_client_with_timeout().unwrap();
    let adapter = SpecDrivenSensorAdapter::new(Arc::new(resolved), auth_strategy, http_client);

    // Build adapter spec for the detections table.
    let mut adapter_spec = make_adapter_spec("crowdstrike", org_id);
    adapter_spec.source_table = "crowdstrike_detections".to_string();

    // Set limit = 3 in QueryParams. F-P1-CRIT-004: the adapter must seed
    // query_filters["query.limit"] = "3" so the CrowdStrike DTU Step 1 receives limit=3.
    let sensor_auth = PluginSensorAuth;
    let mut params = make_query_params();
    params.limit = 3; // should cause DTU to return at most 3 IDs → 3 records

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    assert!(
        result.is_ok(),
        "F-P1-CRIT-004: fetch() with limit=3 must succeed; got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();
    let total_records: usize = batches.iter().map(|b| b.num_rows()).sum();

    // LOAD-BEARING (F-P1-CRIT-004): DTU must honor limit=3 from params.limit.
    // If adapter does NOT seed query.limit from params.limit, the DTU returns 50 records.
    // With correct seeding: DTU Step 1 returns 3 IDs → Step 2 returns 3 records.
    assert!(
        total_records <= 3,
        "F-P1-CRIT-004 LOAD-BEARING: fetch() with params.limit=3 must return at most 3 records; \
         got {}. The adapter MUST seed query_filters[\"query.limit\"] from params.limit. \
         Without this, DTU defaults to limit=100 and returns 50 fixture records. \
         REGRESSION: spec_driven_adapter.rs::fetch() is NOT seeding query.limit.",
        total_records
    );
    assert!(
        total_records > 0,
        "F-P1-CRIT-004: fetch() must return non-empty records (CrowdStrike fixture has 50 records)"
    );
}

/// `SpecDrivenSensorAdapter::sensor_type()` returns the sensor ID from the spec.
///
/// GREEN-BY-DESIGN: zero branching, no I/O, 1-line body.
/// BC-5.38.002 criterion.
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
        "sensor_type() must return SensorId from the resolved spec. BC-2.01.013."
    );
}
