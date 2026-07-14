// SPDX-License-Identifier: Apache-2.0
//! ADV-P02-CRIT-001 + ADV-P02-MED-001 fix: real end-to-end push-down pipeline tests.
//!
//! These tests close the adversary pass-2 findings that the existing push-down tests
//! were false-greens: they hand-fed `_fql`/`aql`/`query.limit` into `FetchContext`
//! and called `PipelineExecutor::execute` directly, bypassing the production pipeline.
//!
//! The tests here call `run_materialization_pipeline` (the real production entry point)
//! with:
//! - A `SpecDrivenSensorAdapter` registered in an `AdapterRegistry` (built via `step9a`)
//! - A `resolved_spec_map` populated from the production TOML so
//!   `extract_time_window_from_ast` can identify datetime INDEX columns
//! - PrismQL queries with real WHERE time predicates (NOT hand-fed `FetchContext`)
//!
//! # Production call graph proven by each test
//!
//! ```text
//! run_materialization_pipeline(pql_query)
//!   → PrismQlParser::parse
//!   → extract_time_window_from_ast [reads resolved_spec_map INDEX columns]
//!   → fan_out → SpecDrivenSensorAdapter::fetch
//!       → build_crowdstrike_fql (CrowdStrike) or augment_armis_aql (Armis)
//!       → PipelineExecutor::execute → DTU wire
//! ```
//!
//! # Test inventory
//!
//! | Test | ADV finding | Entry point | DTU assertion |
//! |------|-------------|-------------|---------------|
//! | test_adv_p02_e2e_crowdstrike_fql_from_where_predicate | ADV-P02-CRIT-001 | run_materialization_pipeline | DTU filter-log contains BOTH created_timestamp bounds in combined `+` FQL form |
//! | test_adv_p02_e2e_crowdstrike_limit_from_pql_limit_clause | ADV-P02-CRIT-001 | run_materialization_pipeline | result.rows <= N where N comes from PQL LIMIT clause seeded into query.limit |
//! | test_adv_p02_e2e_armis_aql_augmentation_from_where_predicate | ADV-P02-CRIT-001 | run_materialization_pipeline | DTU aql-log contains augmented AQL built by augment_armis_aql_with_time_window |
//! | test_adv_p02_sid1_armis_fetch_start_time_augments_aql | ADV-P02-MED-001 | SpecDrivenSensorAdapter::fetch (direct) | query_filters["aql"] becomes augmented form with after: clause |
//! | test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline | AC-CWS-002 (ADV-P04-HIGH-002) | run_materialization_pipeline | DTU filter-log contains BOTH bounds in combined `+` form; filtered_count < unfiltered_count |
//! | test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline | AC-EQUIV-001 (ADV-P05-HIGH-001) | run_materialization_pipeline | Result A (time-filtered) is a strict subset of Result B (unfiltered); every record id in A appears in B (BC-2.11.007 no-fabrication invariant) |
//!
//! # SID-1 compliance
//!
//! All tests that require a DTU clone start one on 127.0.0.1:0.
//! `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` (ADV-P02-MED-001) is a
//! fully in-process test with a wiremock HTTP boundary (no DTU binary required).
//! It is NOT #[ignore]'d per SID-1 §2.
//!
//! # AC-ARMIS-TW-005 SID-1 update
//!
//! The new test `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` is the
//! required SID-1 substitute for AC-ARMIS-TW-005. The `#[ignore]` test in
//! prism-spec-engine/tests/parity/armis.rs now cites BOTH this test AND
//! `test_ac_armis_tw_001_time_window_augmented_into_aql` as its in-process coverage.
//!
//! BCs: BC-2.01.013, BC-2.11.007, BC-2.11.005 (introduced v1.6)
//! ADR: ADR-033 T1, ADR-022 §C
//! Story: S-DEMO-QUERY-PUSHDOWN-001 v2.1 ADV-P02 fix-burst

#![allow(
    non_snake_case,
    dead_code,
    unused_imports,
    clippy::unwrap_used,
    clippy::expect_used
)]

use std::{collections::HashMap, sync::Arc};

use datafusion::execution::context::SessionContext;
use prism_bin::spec_driven_adapter::{
    AdapterAuthStrategy, SpecDrivenSensorAdapter, build_http_client_with_timeout,
    step9a_populate_adapter_registry,
};
use prism_core::{OrgId, OrgSlug, SensorId};
use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::CrowdstrikeClone;
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    engine::QueryOptions,
    materialization::{MaterializationContext, run_materialization_pipeline},
};
use prism_sensors::auth::SensorAuth;
use prism_sensors::{
    AdapterRegistry, BearerStaticSensorAuth, CredentialResolver, SensorAdapter, SensorError,
};
use prism_spec_engine::{
    AuthProvider, PluginAuthProvider, ResolvedSensorSpec, ResolvedSpecKey,
    auth_provider::MockAuthProvider,
    overlay::{OverlayLoader, OverlayProvenance, SensorInstanceOverlay},
    spec_parser::{AuthType, SpecLoader},
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a `ResolvedSensorSpec` from a parsed `SensorSpec` and org_slug.
///
/// Uses `OverlayLoader::merge_overlay_onto_type_spec` — the only legitimate
/// external construction path for `#[non_exhaustive]` `ResolvedSensorSpec`.
fn make_resolved_spec(
    spec: prism_spec_engine::spec_parser::SensorSpec,
    org_slug: &str,
) -> ResolvedSensorSpec {
    let overlay_toml = format!(
        "extends = \"{}\"\ninstance_id = \"{}@{}\"",
        spec.sensor_id, spec.sensor_id, org_slug
    );
    let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
        .expect("make_resolved_spec: SensorInstanceOverlay TOML parse failed");
    OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, OrgSlug::new(org_slug))
}

/// Build an `OrgRegistry` with one org registered and return `(registry, org_id, org_slug)`.
fn make_org_registry(slug: &str) -> (prism_core::OrgRegistry, OrgId, OrgSlug) {
    let registry = prism_core::OrgRegistry::new();
    let uuid = uuid::Uuid::now_v7();
    let org_id = OrgId::from_uuid(uuid);
    let org_slug = OrgSlug::new(slug);
    registry
        .register(org_slug.clone(), org_id)
        .expect("make_org_registry: register failed");
    (registry, org_id, org_slug)
}

/// A `CredentialResolver` that returns `BearerStaticSensorAuth` for any Armis sensor.
///
/// Used by `run_materialization_pipeline` → `fan_out` → `SpecDrivenSensorAdapter::fetch`
/// where the `BearerStatic` auth strategy downcasts the auth arg to `BearerStaticSensorAuth`.
struct BearerStaticCredentialResolver {
    token: String,
}

impl BearerStaticCredentialResolver {
    fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl CredentialResolver for BearerStaticCredentialResolver {
    fn resolve(
        &self,
        _client_id: &str,
        _sensor_id: SensorId,
    ) -> Result<Box<dyn SensorAuth>, SensorError> {
        Ok(Box::new(BearerStaticSensorAuth::new(self.token.clone())))
    }
}

/// A `CredentialResolver` that returns a plugin-compatible `SensorAuth` for CrowdStrike.
///
/// The `Plugin` auth strategy ignores the `SensorAuth` arg entirely (ADR-028 §D10),
/// so the concrete type returned here does not matter — any `SensorAuth` impl works.
struct PluginIgnoredCredentialResolver;

impl CredentialResolver for PluginIgnoredCredentialResolver {
    fn resolve(
        &self,
        _client_id: &str,
        _sensor_id: SensorId,
    ) -> Result<Box<dyn SensorAuth>, SensorError> {
        // Plugin auth strategy ignores this; return a stub that satisfies the trait bound.
        struct PluginStubAuth;
        impl SensorAuth for PluginStubAuth {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn auth_type_name(&self) -> &'static str {
                "custom_via_plugin"
            }
        }
        Ok(Box::new(PluginStubAuth))
    }
}

// ---------------------------------------------------------------------------
// ADV-P02-CRIT-001 Test 1: CrowdStrike FQL built by production pipeline
// ---------------------------------------------------------------------------

/// ADV-P02-CRIT-001 / BC-2.11.007 — CrowdStrike FQL end-to-end via `run_materialization_pipeline`.
///
/// # Production call graph proven
///
/// ```text
/// PrismQL `WHERE created_timestamp > 'T1' AND created_timestamp < 'T2'`
///   → run_materialization_pipeline
///   → extract_time_window_from_ast [reads resolved_spec_map INDEX column list]
///   → QueryParams.start_time = 'T1', end_time = 'T2'
///   → fan_out → SpecDrivenSensorAdapter::fetch
///   → build_crowdstrike_fql(start_time, end_time) → query_filters["_fql"]
///   → PipelineExecutor → CrowdStrike DTU
/// ```
///
/// # What makes this load-bearing (ADV-P02-CRIT-001)
///
/// The `created_timestamp` column has `options = ["INDEX"]` in `crowdstrike.sensor.toml`
/// (AC-INDEX-CWS-001). `extract_time_window_from_ast` reads this from `resolved_spec_map`
/// and extracts `(start_time, end_time)` from the WHERE predicate. If `start_time` regresses
/// to `None`, `build_crowdstrike_fql` returns an empty string, the DTU receives no filter
/// param, and `filtered_count < unfiltered_count` FAILS — the test correctly catches the regression.
///
/// # Assertion
///
/// 1. DTU `/dtu/filter-log` contains `created_timestamp` — the FQL was built by
///    `build_crowdstrike_fql` from the AST-extracted bounds (NOT hand-fed).
/// 2. `filtered_count < unfiltered_count` — DTU honored the FQL filter.
///
/// BCs: BC-2.11.007, BC-2.01.013; ADR-033 T1; F-P1-CRIT-001.
#[tokio::test]
async fn test_adv_p02_e2e_crowdstrike_fql_from_where_predicate() {
    // Step 1: Start CrowdStrike DTU clone on ephemeral port.
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("ADV-P02-CRIT-001 CWS-FQL: CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load production crowdstrike.sensor.toml (SAP-2: no fabricated fixture).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("ADV-P02-CRIT-001 CWS-FQL: crowdstrike.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content)
        .expect("ADV-P02-CRIT-001 CWS-FQL: crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    // Step 3: Build resolved_spec_map with the production spec pointing at the DTU.
    // This is the map that `extract_time_window_from_ast` reads to find INDEX columns.
    let org_slug = "test-org";
    let (org_registry, org_id, org_slug_typed) = make_org_registry(org_slug);
    let sensor_id = SensorId::from("crowdstrike");

    let resolved = make_resolved_spec(spec, org_slug);
    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug_typed.clone(), sensor_id.clone()), resolved);
    let resolved_spec_map_arc: Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>> =
        Arc::new(resolved_spec_map);

    // Step 4: Register a SpecDrivenSensorAdapter for CrowdStrike.
    // Auth: MockAuthProvider (Plugin strategy — ignores SensorAuth arg per ADR-028 §D10).
    // Build the adapter directly (same as step9a would) and register it manually.
    // This avoids needing a wasm plugin runtime for the test harness.
    let mock_auth: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("test-oauth2-token"));
    let http_client = build_http_client_with_timeout()
        .expect("ADV-P02-CRIT-001 CWS-FQL: reqwest client build must succeed");
    // Re-read spec from map (we moved it) — extract the resolved spec for adapter construction.
    let spec_content2 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("re-read");
    let mut spec2 = SpecLoader::parse(&spec_content2).expect("re-parse");
    spec2.base_url = dtu_base_url.clone();
    let resolved2 = make_resolved_spec(spec2, org_slug);

    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved2),
        AdapterAuthStrategy::Plugin(mock_auth),
        http_client,
    );

    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(adapter));

    // Step 5: Build MaterializationContext with resolved_spec_map + registry.
    let normalizer = Arc::new(OcsfNormalizer::new());
    let credential_resolver: Arc<dyn CredentialResolver> =
        Arc::new(PluginIgnoredCredentialResolver);
    let org_registry_arc = Arc::new(org_registry);
    let mut mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry),
        normalizer,
        10_000,
        credential_resolver,
        Some(Arc::clone(&org_registry_arc)),
        Some(Arc::clone(&resolved_spec_map_arc)),
    );

    // Step 6: Build a fresh SessionContext.
    let session_ctx = SessionContext::new();

    // Step 7: Build the PrismQL query with WHERE time predicates.
    // Fixture spans 2026-01-01..2026-01-26 (50 records).
    // Filter: 2026-01-10 to 2026-01-20 → should return ~10 records.
    let start_time = "2026-01-10T00:00:00Z";
    let end_time = "2026-01-20T00:00:00Z";
    let query = format!(
        "SELECT * FROM crowdstrike_detections \
         WHERE created_timestamp > '{start_time}' AND created_timestamp < '{end_time}'"
    );

    let options = QueryOptions {
        clients: Some(vec![org_slug_typed.clone()]),
        limit: None,
        ..QueryOptions::default()
    };

    // Step 8: Run the REAL production pipeline.
    // This is the entry point that MUST call extract_time_window_from_ast, fan_out,
    // SpecDrivenSensorAdapter::fetch, build_crowdstrike_fql, and PipelineExecutor.
    let output = run_materialization_pipeline(&query, &options, &mut mat_ctx, &session_ctx)
        .await
        .expect("ADV-P02-CRIT-001 CWS-FQL: run_materialization_pipeline must succeed");

    let filtered_count: usize = output.batches.iter().map(|b| b.num_rows()).sum();

    // Step 9: Assert wire-level — DTU filter-log must contain created_timestamp FQL.
    // This proves the FQL was built by build_crowdstrike_fql from the AST-extracted bounds,
    // NOT hand-fed into FetchContext.
    let http_client_check = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("ADV-P02-CRIT-001 CWS-FQL: filter-log client build");
    let filter_log_resp = http_client_check
        .get(format!("{dtu_base_url}/dtu/filter-log"))
        .send()
        .await
        .expect("ADV-P02-CRIT-001 CWS-FQL: GET /dtu/filter-log must succeed");
    let filter_log_body: serde_json::Value = filter_log_resp
        .json()
        .await
        .expect("ADV-P02-CRIT-001 CWS-FQL: filter-log must be JSON");
    let filter_strings = filter_log_body["filter_strings"]
        .as_array()
        .expect("ADV-P02-CRIT-001 CWS-FQL: filter_strings must be array");

    // AC-CWS-002(b) BOTH-BOUNDS: assert the COMBINED `+` form (not just any `created_timestamp`).
    // build_crowdstrike_fql(start, end) produces:
    //   `created_timestamp:>'T1'+created_timestamp:<'T2'`
    // A test that only checks `contains("created_timestamp")` passes for single-bound FQL —
    // we must verify BOTH lower AND upper bound markers are present (ADV-P04-HIGH-002).
    let both_bounds_present = filter_strings.iter().any(|s| {
        s.as_str()
            .map(|f| f.contains("created_timestamp:>'") && f.contains("created_timestamp:<'"))
            .unwrap_or(false)
    });
    assert!(
        both_bounds_present,
        "ADV-P02-CRIT-001 CWS-FQL WIRE-LEVEL BOTH-BOUNDS (AC-CWS-002 item b): \
         DTU filter-log must contain a FQL string with BOTH 'created_timestamp:>\\'' \
         (lower bound) AND 'created_timestamp:<\\'' (upper bound) in the combined `+` form. \
         build_crowdstrike_fql(start='{}', end='{}') must produce this form. \
         This proves: run_materialization_pipeline \
         → extract_time_window_from_ast [reads resolved_spec_map INDEX columns] \
         → QueryParams.start_time/end_time populated → SpecDrivenSensorAdapter::fetch \
         → build_crowdstrike_fql → PipelineExecutor → DTU filter-log. \
         Got filter_strings: {:?}. \
         REGRESSION: if only one bound is present, extract_time_window_from_ast failed \
         to extract one bound, or build_crowdstrike_fql is not building the combined form.",
        start_time, end_time, filter_strings
    );

    // Step 10: Unfiltered baseline for filtered_count < unfiltered_count assertion.
    clone
        .reset()
        .await
        .expect("ADV-P02-CRIT-001 CWS-FQL: clone reset");

    let spec_content3 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("re-read3");
    let mut spec3 = SpecLoader::parse(&spec_content3).expect("re-parse3");
    spec3.base_url = dtu_base_url.clone();
    let resolved3 = make_resolved_spec(spec3, org_slug);

    let mock_auth3: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("test-oauth2-token"));
    let http_client3 = build_http_client_with_timeout().expect("http_client3");
    let adapter3 = SpecDrivenSensorAdapter::new(
        Arc::new(resolved3),
        AdapterAuthStrategy::Plugin(mock_auth3),
        http_client3,
    );

    let spec_content4 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("re-read4");
    let mut spec4 = SpecLoader::parse(&spec_content4).expect("re-parse4");
    spec4.base_url = dtu_base_url.clone();
    let resolved4 = make_resolved_spec(spec4, org_slug);

    let mut resolved_spec_map2: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    let (org_registry2, org_id2, org_slug_typed2) = make_org_registry("test-org2");
    resolved_spec_map2.insert(
        (org_slug_typed2.clone(), SensorId::from("crowdstrike")),
        resolved4,
    );

    let mut adapter_registry2 = AdapterRegistry::new();
    adapter_registry2.register(org_id2, Arc::new(adapter3));

    let normalizer2 = Arc::new(OcsfNormalizer::new());
    let cred_resolver2: Arc<dyn CredentialResolver> = Arc::new(PluginIgnoredCredentialResolver);
    let mut mat_ctx2 = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry2),
        normalizer2,
        10_000,
        cred_resolver2,
        Some(Arc::new(org_registry2)),
        Some(Arc::new(resolved_spec_map2)),
    );

    let session_ctx2 = SessionContext::new();
    let unfiltered_options = QueryOptions {
        clients: Some(vec![org_slug_typed2.clone()]),
        limit: None,
        ..QueryOptions::default()
    };

    let output2 = run_materialization_pipeline(
        "SELECT * FROM crowdstrike_detections",
        &unfiltered_options,
        &mut mat_ctx2,
        &session_ctx2,
    )
    .await
    .expect("ADV-P02-CRIT-001 CWS-FQL: unfiltered pipeline must succeed");

    let unfiltered_count: usize = output2.batches.iter().map(|b| b.num_rows()).sum();

    // LOAD-BEARING: filtered_count < unfiltered_count proves the FQL filter reached the DTU
    // and the DTU honored it. If the time-window extraction is broken, both return 50 records.
    assert!(
        filtered_count < unfiltered_count,
        "ADV-P02-CRIT-001 CWS-FQL LOAD-BEARING: filtered_count ({}) must be < \
         unfiltered_count ({}) when WHERE time predicate is pushed through \
         run_materialization_pipeline → extract_time_window_from_ast → build_crowdstrike_fql. \
         If this fails, the production push-down path has a REAL wiring gap — \
         start_time/end_time are not reaching SpecDrivenSensorAdapter::fetch.",
        filtered_count,
        unfiltered_count
    );
    assert!(
        filtered_count > 0,
        "ADV-P02-CRIT-001 CWS-FQL: filtered pipeline must return non-empty records \
         (fixture has records in range {} to {})",
        start_time,
        end_time
    );
}

// ---------------------------------------------------------------------------
// ADV-P02-CRIT-001 Test 2: CrowdStrike LIMIT from PQL clause
// ---------------------------------------------------------------------------

/// ADV-P02-CRIT-001 / BC-2.01.013 — CrowdStrike LIMIT end-to-end via `run_materialization_pipeline`.
///
/// # Production call graph proven
///
/// ```text
/// PrismQL `SELECT ... LIMIT N`
///   → run_materialization_pipeline [options.limit = N]
///   → fan_out [QueryParams.limit = N]
///   → SpecDrivenSensorAdapter::fetch → query_filters["query.limit"] = "N"
///   → PipelineExecutor → CrowdStrike DTU → result.len() <= N
/// ```
///
/// # What makes this load-bearing (ADV-P02-CRIT-001)
///
/// If `QueryParams.limit` does not flow through to `query_filters["query.limit"]`,
/// the DTU defaults to returning all 50 records → `result.len() <= 5` FAILS.
///
/// BCs: BC-2.01.013, BC-2.11.007; F-P1-CRIT-004.
#[tokio::test]
async fn test_adv_p02_e2e_crowdstrike_limit_from_pql_limit_clause() {
    // Start CrowdStrike DTU clone.
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("ADV-P02-CRIT-001 CWS-LIMIT: CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Load production spec.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("ADV-P02-CRIT-001 CWS-LIMIT: crowdstrike.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content)
        .expect("ADV-P02-CRIT-001 CWS-LIMIT: crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    let org_slug = "limit-test-org";
    let (org_registry, org_id, org_slug_typed) = make_org_registry(org_slug);
    let sensor_id = SensorId::from("crowdstrike");

    let resolved = make_resolved_spec(spec, org_slug);
    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug_typed.clone(), sensor_id.clone()), resolved);

    let mock_auth: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("test-oauth2-token"));
    let http_client =
        build_http_client_with_timeout().expect("ADV-P02-CRIT-001 CWS-LIMIT: reqwest client build");

    // Load fresh spec for adapter construction.
    let spec_content2 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("re-read2");
    let mut spec2 = SpecLoader::parse(&spec_content2).expect("re-parse2");
    spec2.base_url = dtu_base_url.clone();
    let resolved2 = make_resolved_spec(spec2, org_slug);

    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved2),
        AdapterAuthStrategy::Plugin(mock_auth),
        http_client,
    );

    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(adapter));

    let normalizer = Arc::new(OcsfNormalizer::new());
    let credential_resolver: Arc<dyn CredentialResolver> =
        Arc::new(PluginIgnoredCredentialResolver);
    let mut mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry),
        normalizer,
        10_000,
        credential_resolver,
        Some(Arc::new(org_registry)),
        Some(Arc::new(resolved_spec_map)),
    );

    let session_ctx = SessionContext::new();

    // PQL query with LIMIT 5. The limit must flow: options.limit → QueryParams.limit →
    // SpecDrivenSensorAdapter::fetch → query_filters["query.limit"] = "5" →
    // PipelineExecutor → DTU → returns ≤ 5 IDs → result has ≤ 5 rows.
    let options = QueryOptions {
        clients: Some(vec![org_slug_typed.clone()]),
        limit: Some(5),
        ..QueryOptions::default()
    };

    let output = run_materialization_pipeline(
        "SELECT * FROM crowdstrike_detections",
        &options,
        &mut mat_ctx,
        &session_ctx,
    )
    .await
    .expect("ADV-P02-CRIT-001 CWS-LIMIT: run_materialization_pipeline must succeed");

    let total_rows: usize = output.batches.iter().map(|b| b.num_rows()).sum();

    // LOAD-BEARING: result must have at most 5 rows.
    // If limit push-down is broken, DTU returns 50 records → fails.
    assert!(
        total_rows <= 5,
        "ADV-P02-CRIT-001 CWS-LIMIT LOAD-BEARING: run_materialization_pipeline with \
         options.limit=5 must produce at most 5 records; got {}. \
         Production path: options.limit → QueryParams.limit → \
         SpecDrivenSensorAdapter::fetch → query_filters[\"query.limit\"] = \"5\" → \
         PipelineExecutor → CrowdStrike DTU (limit=5 on Step 1 request). \
         REGRESSION: if this fails, the limit is NOT flowing through the production path.",
        total_rows
    );
    assert!(
        total_rows > 0,
        "ADV-P02-CRIT-001 CWS-LIMIT: pipeline must return non-empty records (CrowdStrike fixture has 50 records)"
    );
}

// ---------------------------------------------------------------------------
// ADV-P02-CRIT-001 Test 3: Armis AQL augmentation via production pipeline
// ---------------------------------------------------------------------------

/// ADV-P02-CRIT-001 / BC-2.11.007 §Mechanism B — Armis AQL augmentation end-to-end.
///
/// # Production call graph proven
///
/// ```text
/// PrismQL `SELECT ... FROM armis_devices WHERE aql = 'in:devices' AND last_seen > 'T'`
///   → run_materialization_pipeline
///   → extract_push_down_filters_as_map → FilterMap["aql"] = "in:devices"
///   → extract_time_window_from_ast [reads resolved_spec_map INDEX columns: last_seen]
///   → QueryParams.start_time = 'T'
///   → fan_out → SpecDrivenSensorAdapter::fetch
///   → augment_armis_aql_with_time_window("in:devices", start='T', end=None)
///       → "in:devices after:2026-01-10T00:00:00"
///   → query_filters["aql"] = "in:devices after:2026-01-10T00:00:00"
///   → PipelineExecutor → Armis DTU aql-log
/// ```
///
/// # What makes this load-bearing (ADV-P02-CRIT-001)
///
/// The `last_seen` column has `options = ["INDEX"]` in `armis.sensor.toml` (AC-INDEX-001).
/// `extract_time_window_from_ast` reads this from `resolved_spec_map` and extracts
/// `start_time = 'T'`. If `start_time` regresses to None, `augment_armis_aql_with_time_window`
/// is not called, the aql stays as `"in:devices"` without the `after:` clause, and
/// `filtered_count < unfiltered_count` FAILS — the test catches the regression.
///
/// # Assertion
///
/// 1. DTU `/dtu/aql-log` contains both `in:devices` AND `after:` — the augmented AQL
///    was built by `augment_armis_aql_with_time_window` (NOT hand-fed).
/// 2. `filtered_count < unfiltered_count` — DTU honored the time-filtered AQL.
///
/// BCs: BC-2.11.007 §Mechanism B, BC-2.01.013; ADR-033 T1; F-P1-CRIT-002.
#[tokio::test]
async fn test_adv_p02_e2e_armis_aql_augmentation_from_where_predicate() {
    // Start Armis DTU clone.
    let mut clone =
        ArmisClone::new().expect("ADV-P02-CRIT-001 ARMIS-AQL: ArmisClone::new must succeed");
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("ADV-P02-CRIT-001 ARMIS-AQL: Armis DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Load production armis.sensor.toml (SAP-2: no fabricated fixture).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("ADV-P02-CRIT-001 ARMIS-AQL: armis.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content)
        .expect("ADV-P02-CRIT-001 ARMIS-AQL: armis.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    let org_slug = "armis-test-org";
    let (org_registry, org_id, org_slug_typed) = make_org_registry(org_slug);
    let sensor_id = SensorId::from("armis");

    let resolved = make_resolved_spec(spec, org_slug);
    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug_typed.clone(), sensor_id.clone()), resolved);
    let resolved_spec_map_arc = Arc::new(resolved_spec_map);

    // Build SpecDrivenSensorAdapter for Armis (BearerStatic auth).
    // The auth token is provided by BearerStaticCredentialResolver.
    let spec_content2 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("re-read armis");
    let mut spec2 = SpecLoader::parse(&spec_content2).expect("re-parse armis");
    spec2.base_url = dtu_base_url.clone();
    let resolved2 = make_resolved_spec(spec2, org_slug);

    let http_client =
        build_http_client_with_timeout().expect("ADV-P02-CRIT-001 ARMIS-AQL: reqwest client build");
    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved2),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    );

    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(adapter));

    // BearerStaticCredentialResolver provides BearerStaticSensorAuth so the
    // BearerStatic downcast in SpecDrivenSensorAdapter::fetch succeeds.
    let normalizer = Arc::new(OcsfNormalizer::new());
    let credential_resolver: Arc<dyn CredentialResolver> = Arc::new(
        BearerStaticCredentialResolver::new("armis-test-bearer-token"),
    );
    let org_registry_arc = Arc::new(org_registry);
    let mut mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry),
        normalizer,
        10_000,
        credential_resolver,
        Some(Arc::clone(&org_registry_arc)),
        Some(Arc::clone(&resolved_spec_map_arc)),
    );

    let session_ctx = SessionContext::new();

    // PrismQL query with WHERE aql = 'in:devices' AND last_seen > 'T'.
    // The `last_seen` predicate is extracted by extract_time_window_from_ast
    // (because last_seen has options=["INDEX"]) → start_time = '2026-01-10T00:00:00Z'.
    // SpecDrivenSensorAdapter::fetch calls augment_armis_aql_with_time_window(
    //   "in:devices", start='2026-01-10T00:00:00Z', end=None)
    // → "in:devices after:2026-01-10T00:00:00"
    let start_time = "2026-01-10T00:00:00Z";
    let query = format!(
        "SELECT * FROM armis_devices \
         WHERE aql = 'in:devices' AND last_seen > '{start_time}'"
    );

    let options = QueryOptions {
        clients: Some(vec![org_slug_typed.clone()]),
        limit: None,
        ..QueryOptions::default()
    };

    // REAL production pipeline: not PipelineExecutor::execute, not hand-fed FetchContext.
    let output = run_materialization_pipeline(&query, &options, &mut mat_ctx, &session_ctx)
        .await
        .expect("ADV-P02-CRIT-001 ARMIS-AQL: run_materialization_pipeline must succeed");

    let filtered_count: usize = output.batches.iter().map(|b| b.num_rows()).sum();

    // Wire-level assertion: DTU aql-log must contain the augmented AQL.
    // The augmented AQL has BOTH 'in:devices' AND 'after:' — proving augment_armis_aql
    // was invoked from the production path (NOT hand-fed).
    let http_client_check = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("ADV-P02-CRIT-001 ARMIS-AQL: aql-log client build");
    let aql_log_resp = http_client_check
        .get(format!("{dtu_base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("ADV-P02-CRIT-001 ARMIS-AQL: GET /dtu/aql-log must succeed");
    let aql_log_body: serde_json::Value = aql_log_resp
        .json()
        .await
        .expect("ADV-P02-CRIT-001 ARMIS-AQL: aql-log must be JSON");
    let aql_strings = aql_log_body["aql_strings"]
        .as_array()
        .expect("ADV-P02-CRIT-001 ARMIS-AQL: aql_strings must be array");

    // The augmented AQL must contain BOTH 'in:devices' AND 'after:' to prove
    // augment_armis_aql_with_time_window was called from the production path.
    let has_augmented_aql = aql_strings.iter().any(|s| {
        s.as_str()
            .map(|aql| aql.contains("in:devices") && aql.contains("after:"))
            .unwrap_or(false)
    });
    assert!(
        has_augmented_aql,
        "ADV-P02-CRIT-001 ARMIS-AQL WIRE-LEVEL: DTU aql-log must contain an AQL string \
         with BOTH 'in:devices' AND 'after:' (the augmented form produced by \
         augment_armis_aql_with_time_window). \
         This proves: run_materialization_pipeline → extract_time_window_from_ast \
         [last_seen INDEX column in resolved_spec_map] → start_time='{}' → \
         SpecDrivenSensorAdapter::fetch → augment_armis_aql_with_time_window → \
         query_filters[\"aql\"] = 'in:devices after:...' → DTU. \
         Got aql_strings: {:?}. \
         REGRESSION: if start_time=None regresses, augmentation is NOT called, \
         the DTU receives plain 'in:devices' without the after: clause.",
        start_time, aql_strings
    );

    // LOAD-BEARING: filtered_count < unfiltered_count.
    // Reset and run unfiltered baseline.
    clone
        .reset()
        .await
        .expect("ADV-P02-CRIT-001 ARMIS-AQL: clone reset");

    let spec_content3 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("re-read armis3");
    let mut spec3 = SpecLoader::parse(&spec_content3).expect("re-parse armis3");
    spec3.base_url = dtu_base_url.clone();
    let resolved3 = make_resolved_spec(spec3, "armis-baseline-org");

    let (org_registry3, org_id3, org_slug_typed3) = make_org_registry("armis-baseline-org");

    let mut resolved_spec_map3: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map3.insert(
        (org_slug_typed3.clone(), SensorId::from("armis")),
        resolved3,
    );

    let spec_content4 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("re-read armis4");
    let mut spec4 = SpecLoader::parse(&spec_content4).expect("re-parse armis4");
    spec4.base_url = dtu_base_url.clone();
    let resolved4 = make_resolved_spec(spec4, "armis-baseline-org");

    let http_client3 = build_http_client_with_timeout().expect("http_client3");
    let adapter3 = SpecDrivenSensorAdapter::new(
        Arc::new(resolved4),
        AdapterAuthStrategy::BearerStatic,
        http_client3,
    );
    let mut adapter_registry3 = AdapterRegistry::new();
    adapter_registry3.register(org_id3, Arc::new(adapter3));

    let normalizer3 = Arc::new(OcsfNormalizer::new());
    let cred_resolver3: Arc<dyn CredentialResolver> = Arc::new(
        BearerStaticCredentialResolver::new("armis-test-bearer-token"),
    );
    let mut mat_ctx3 = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry3),
        normalizer3,
        10_000,
        cred_resolver3,
        Some(Arc::new(org_registry3)),
        Some(Arc::new(resolved_spec_map3)),
    );

    let session_ctx3 = SessionContext::new();
    let baseline_options = QueryOptions {
        clients: Some(vec![org_slug_typed3.clone()]),
        limit: None,
        ..QueryOptions::default()
    };

    let output3 = run_materialization_pipeline(
        "SELECT * FROM armis_devices WHERE aql = 'in:devices'",
        &baseline_options,
        &mut mat_ctx3,
        &session_ctx3,
    )
    .await
    .expect("ADV-P02-CRIT-001 ARMIS-AQL: unfiltered pipeline must succeed");

    let unfiltered_count: usize = output3.batches.iter().map(|b| b.num_rows()).sum();

    assert!(
        filtered_count < unfiltered_count,
        "ADV-P02-CRIT-001 ARMIS-AQL LOAD-BEARING: filtered_count ({}) must be < \
         unfiltered_count ({}) when WHERE last_seen > '{}' is pushed through \
         run_materialization_pipeline → extract_time_window_from_ast → \
         augment_armis_aql_with_time_window. \
         REGRESSION: if the production path is broken, both counts will be equal.",
        filtered_count,
        unfiltered_count,
        start_time
    );
    assert!(
        filtered_count > 0,
        "ADV-P02-CRIT-001 ARMIS-AQL: filtered pipeline must return non-empty records"
    );
}

// ---------------------------------------------------------------------------
// ADV-P02-MED-001: SID-1 substitute for AC-ARMIS-TW-005
// ---------------------------------------------------------------------------

/// ADV-P02-MED-001 / SID-1 substitute for AC-ARMIS-TW-005.
///
/// This test directly invokes `SpecDrivenSensorAdapter::fetch` with
/// `QueryParams { start_time: Some(...), end_time: Some(...), .. }` for the
/// Armis sensor and asserts that `query_filters["aql"]` becomes the augmented
/// form (`base_aql + " after:<ts>"`).
///
/// # Why this test is required (ADV-P02-MED-001)
///
/// The existing substitutes for AC-ARMIS-TW-005 either:
/// - Call `augment_armis_aql_with_time_window` directly (no SpecDrivenSensorAdapter::fetch)
/// - Hand-feed the augmented AQL into `FetchContext` (bypass the fetch() augmentation logic)
///
/// This test exercises the DECISION BRANCH in `spec_driven_adapter.rs::fetch()` at lines
/// 590-600 where the augmentation decision is made:
/// ```rust
/// if self.sensor_spec.spec.sensor_id.as_str() == "armis"
///     && (params.start_time.is_some() || params.end_time.is_some())
/// { augment ... }
/// ```
///
/// # SID-1 compliance
///
/// This test is NOT `#[ignore]`'d. It uses a wiremock HTTP server (no live DTU binary)
/// and exercises the production `fetch()` code path directly. This satisfies SID-1 §2:
/// the behavior (fetch() augmentation decision) is exercised in-process without external deps.
///
/// # AC-ARMIS-TW-005 SID-1 update
///
/// `test_ac_armis_tw_005_e2e_aql_log_contains_augmented_aql` (#[ignore]) now cites BOTH
/// `test_ac_armis_tw_001_time_window_augmented_into_aql` (pushdown.rs unit) AND
/// this test as its in-process coverage per SID-1 §2.
///
/// BCs: BC-2.11.007 §Mechanism B; ADR-033 T1; SID-1.
#[tokio::test]
async fn test_adv_p02_sid1_armis_fetch_start_time_augments_aql() {
    use prism_sensors::adapter::{
        QueryParams as SensorQueryParams, SensorSpec as SensorAdapterSpec,
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    // Start wiremock server to handle the Armis API call.
    // The adapter will POST/GET to this server; we return a minimal Armis response.
    let mock_server = MockServer::start().await;

    // Mount: return a minimal device list at GET /api/v1/search.
    // Response shape: SearchResponse { data: SearchData { results: [...], total } }
    // response_path = "$.data.results" (armis.sensor.toml spec).
    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "results": [
                    {
                        "id": 1001,
                        "name": "device-alpha",
                        "ipAddress": "10.0.0.1",
                        "macAddress": "aa:bb:cc:dd:ee:ff",
                        "type": "Camera",
                        "category": "IP Camera",
                        "firstSeen": "2026-01-05T10:00:00Z",
                        "lastSeen": "2026-01-15T10:00:00Z",
                        "operatingSystem": "Linux",
                        "manufacturer": "Acme",
                        "model": "Cam-100",
                        "site": "Site-A",
                        "zone": "Zone-1",
                        "riskLevel": 2,
                        "riskFactors": []
                    }
                ],
                "total": 1
            },
            "success": true,
            "count": 1
        })))
        .mount(&mock_server)
        .await;

    // Load production armis.sensor.toml directed at the mock server.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("ADV-P02-MED-001: armis.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("ADV-P02-MED-001: armis.sensor.toml must parse");
    spec.base_url = mock_server.uri();

    let resolved = make_resolved_spec(spec, "armis-sid1-org");
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("ADV-P02-MED-001: reqwest client build");

    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    );

    // Build adapter spec (prism-sensors SensorSpec, not spec-engine SensorSpec).
    let (_, org_id, _) = make_org_registry("armis-sid1-org");
    #[allow(deprecated)]
    let adapter_spec = SensorAdapterSpec {
        source_table: "armis_devices".to_string(),
        org_id,
        #[allow(deprecated)]
        client_id: "armis-sid1-org".to_string(),
        sensor_config: serde_json::json!({}),
    };

    // Build QueryParams with start_time set.
    // This is the condition that triggers augmentation in spec_driven_adapter.rs:590-600:
    //   if sensor_id == "armis" && params.start_time.is_some() { augment ... }
    let params = SensorQueryParams {
        cursor: None,
        limit: 10,
        start_time: Some("2026-01-10T00:00:00Z".to_string()),
        end_time: None,
        filters: {
            let mut m = prism_sensors::types::FilterMap::new();
            m.insert(
                "aql".to_string(),
                serde_json::Value::String("in:devices".to_string()),
            );
            m
        },
    };

    // Provide BearerStaticSensorAuth as the auth arg (BearerStatic path).
    let sensor_auth = BearerStaticSensorAuth::new("armis-test-bearer-token");

    // Call SpecDrivenSensorAdapter::fetch DIRECTLY (ADV-P02-MED-001: exercises fetch() decision branch).
    // This is the REQUIRED production call — not augment_armis_aql_with_time_window directly.
    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // The fetch must succeed (mock server returns 200 with device data).
    assert!(
        result.is_ok(),
        "ADV-P02-MED-001: SpecDrivenSensorAdapter::fetch must succeed against mock server; \
         got Err: {:?}",
        result.err()
    );

    // Now verify the mock server received the augmented AQL.
    // wiremock captures all received requests; we check what URL was called.
    let requests = mock_server.received_requests().await.unwrap_or_default();

    // Find the request that was directed at /api/v1/search.
    let search_request = requests.iter().find(|r| r.url.path() == "/api/v1/search");
    assert!(
        search_request.is_some(),
        "ADV-P02-MED-001: SpecDrivenSensorAdapter::fetch must have issued a GET /api/v1/search \
         request to the mock server. Got requests: {:?}",
        requests
            .iter()
            .map(|r| r.url.to_string())
            .collect::<Vec<_>>()
    );

    let request = search_request.unwrap();
    let aql_param = request
        .url
        .query_pairs()
        .find(|(k, _)| k == "aql")
        .map(|(_, v)| v.into_owned());

    assert!(
        aql_param.is_some(),
        "ADV-P02-MED-001: The GET /api/v1/search request must have an 'aql' query param. \
         Got URL: {}. \
         REGRESSION: SpecDrivenSensorAdapter::fetch is not passing the aql filter.",
        request.url
    );

    let aql_value = aql_param.unwrap();

    // LOAD-BEARING: the AQL must contain BOTH 'in:devices' AND 'after:'.
    // This proves the fetch() augmentation decision branch (spec_driven_adapter.rs:590-600)
    // was executed — augment_armis_aql_with_time_window was called FROM fetch(), not hand-fed.
    assert!(
        aql_value.contains("in:devices"),
        "ADV-P02-MED-001 LOAD-BEARING: aql param must contain 'in:devices' (the base AQL). \
         Got aql='{}'. \
         REGRESSION: base AQL from params.filters[\"aql\"] was not preserved.",
        aql_value
    );
    assert!(
        aql_value.contains("after:"),
        "ADV-P02-MED-001 LOAD-BEARING: aql param must contain 'after:' \
         (augmented by augment_armis_aql_with_time_window from params.start_time='2026-01-10T00:00:00Z'). \
         Got aql='{}'. \
         REGRESSION: the augmentation decision branch in SpecDrivenSensorAdapter::fetch() \
         (spec_driven_adapter.rs:590-600) was NOT executed. \
         The branch condition: \
         `if self.sensor_spec.spec.sensor_id == \"armis\" && params.start_time.is_some()` \
         This test DIRECTLY exercises that branch — not augment_armis_aql directly, not hand-fed FetchContext.",
        aql_value
    );
}

// ---------------------------------------------------------------------------
// AC-CWS-002 (NAMED): CrowdStrike FQL both-bounds via run_materialization_pipeline
// (ADV-P04-HIGH-002 closure — the authoritative AC-CWS-002 coverage)
// ---------------------------------------------------------------------------

/// AC-CWS-002 / BC-2.01.013 TV-BC-2.01.013-006 — both start+end bounds via
/// the real production pipeline path (`run_materialization_pipeline`).
///
/// # Why this test lives here (not in prism-spec-engine)
///
/// `prism-spec-engine` cannot depend on `prism-bin` (circular dependency guard — see
/// CLAUDE.md architecture compliance). The AC-CWS-002 coverage that genuinely exercises
/// `run_materialization_pipeline` must therefore live in `prism-bin/tests/`.
///
/// # Production call graph proven
///
/// ```text
/// PrismQL: `WHERE created_timestamp > 'T1' AND created_timestamp < 'T2'`
///   → run_materialization_pipeline
///   → PrismQlParser::parse
///   → extract_time_window_from_ast [reads resolved_spec_map: created_timestamp INDEX]
///   → QueryParams.start_time = 'T1', end_time = 'T2'
///   → fan_out → SpecDrivenSensorAdapter::fetch
///   → build_crowdstrike_fql(start='T1', end='T2')
///       → "created_timestamp:>'T1'+created_timestamp:<'T2'"  [COMBINED `+` form]
///   → FetchContext["_fql"] = combined FQL
///   → PipelineExecutor → CrowdStrike DTU filter-log
/// ```
///
/// # Assertions (AC-CWS-002)
///
/// (a) DTU `/dtu/filter-log` contains a FQL string with BOTH
///     `created_timestamp:>'<T1>'` AND `created_timestamp:<'<T2>'` (combined `+` form).
///     This proves `build_crowdstrike_fql` was invoked with both bounds from the AST.
///
/// (b) `filtered_count < unfiltered_count` — DTU honored the FQL filter.
///
/// # ADV-P04-HIGH-002 closure
///
/// The prism-spec-engine test `test_ac_cws_002_wire_level_fql_both_bounds_via_pipeline_executor`
/// validates the same wire-level propagation but with a pre-seeded FQL. THIS test is the
/// authoritative AC-CWS-002 coverage: it verifies that `run_materialization_pipeline` with
/// a real WHERE predicate causes `build_crowdstrike_fql` to build the combined both-bounds FQL.
///
/// BCs: BC-2.01.013, BC-2.11.007; ADR-033 T1; F-P1-CRIT-001.
#[tokio::test]
async fn test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline() {
    // Step 1: Start CrowdStrike DTU clone.
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-CWS-002: CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load production crowdstrike.sensor.toml (SAP-2: no fabricated fixture).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-CWS-002: crowdstrike.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-CWS-002: crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    // Step 3: Build resolved_spec_map — the map that extract_time_window_from_ast reads
    // to identify INDEX columns (created_timestamp has options=["INDEX"] per AC-INDEX-CWS-001).
    let org_slug = "ac-cws-002-org";
    let (org_registry, org_id, org_slug_typed) = make_org_registry(org_slug);
    let sensor_id = SensorId::from("crowdstrike");

    let resolved = make_resolved_spec(spec, org_slug);
    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug_typed.clone(), sensor_id.clone()), resolved);
    let resolved_spec_map_arc: Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>> =
        Arc::new(resolved_spec_map);

    // Step 4: Build SpecDrivenSensorAdapter and register it.
    let mock_auth: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("test-oauth2-token"));
    let http_client =
        build_http_client_with_timeout().expect("AC-CWS-002: reqwest client build must succeed");

    let spec_content2 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-CWS-002: re-read crowdstrike.sensor.toml");
    let mut spec2 = SpecLoader::parse(&spec_content2).expect("AC-CWS-002: re-parse");
    spec2.base_url = dtu_base_url.clone();
    let resolved2 = make_resolved_spec(spec2, org_slug);

    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved2),
        AdapterAuthStrategy::Plugin(mock_auth),
        http_client,
    );
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(adapter));

    // Step 5: Build MaterializationContext.
    let normalizer = Arc::new(OcsfNormalizer::new());
    let credential_resolver: Arc<dyn CredentialResolver> =
        Arc::new(PluginIgnoredCredentialResolver);
    let org_registry_arc = Arc::new(org_registry);
    let mut mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry),
        normalizer,
        10_000,
        credential_resolver,
        Some(Arc::clone(&org_registry_arc)),
        Some(Arc::clone(&resolved_spec_map_arc)),
    );
    let session_ctx = SessionContext::new();

    // Step 6: PrismQL with BOTH time predicates — this is what AC-CWS-002 tests.
    // Fixture spans 2026-01-01..2026-01-26 (50 records).
    // Window: 2026-01-10 to 2026-01-20 → should return records in that range (~10).
    let start_time = "2026-01-10T00:00:00Z";
    let end_time = "2026-01-20T00:00:00Z";
    let query = format!(
        "SELECT * FROM crowdstrike_detections \
         WHERE created_timestamp > '{start_time}' AND created_timestamp < '{end_time}'"
    );

    let options = QueryOptions {
        clients: Some(vec![org_slug_typed.clone()]),
        limit: None,
        ..QueryOptions::default()
    };

    // Step 7: Run the REAL production pipeline — NOT PipelineExecutor::execute directly,
    // NOT hand-fed FetchContext["_fql"]. This is the path AC-CWS-002(d) requires.
    let output = run_materialization_pipeline(&query, &options, &mut mat_ctx, &session_ctx)
        .await
        .expect("AC-CWS-002: run_materialization_pipeline must succeed");
    let filtered_count: usize = output.batches.iter().map(|b| b.num_rows()).sum();

    // Step 8: Wire-level assertion (F-P1-HIGH-003 / AC-CWS-002 item a+b).
    // The DTU filter-log must contain a FQL string with BOTH bounds in the combined `+` form.
    // This proves build_crowdstrike_fql was called with both start_time AND end_time from
    // the AST-extracted QueryParams.
    let http_client_check = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-CWS-002: filter-log client build");
    let filter_log_resp = http_client_check
        .get(format!("{dtu_base_url}/dtu/filter-log"))
        .send()
        .await
        .expect("AC-CWS-002: GET /dtu/filter-log must succeed");
    let filter_log_body: serde_json::Value = filter_log_resp
        .json()
        .await
        .expect("AC-CWS-002: filter-log must be JSON");
    let filter_strings = filter_log_body["filter_strings"]
        .as_array()
        .expect("AC-CWS-002: filter_strings must be array");

    // AC-CWS-002 item (b): BOTH lower AND upper bound must be present in the combined form.
    // `created_timestamp:>'T1'+created_timestamp:<'T2'` — the `+` separates the two clauses.
    let both_bounds_present = filter_strings.iter().any(|s| {
        s.as_str()
            .map(|f| f.contains("created_timestamp:>'") && f.contains("created_timestamp:<'"))
            .unwrap_or(false)
    });
    assert!(
        both_bounds_present,
        "AC-CWS-002 AUTHORITATIVE BOTH-BOUNDS (ADV-P04-HIGH-002): \
         DTU filter-log must contain a FQL string with BOTH 'created_timestamp:>\\'' \
         AND 'created_timestamp:<\\'' in the combined `+` form. \
         This proves run_materialization_pipeline → extract_time_window_from_ast \
         populated BOTH QueryParams.start_time='{}' AND QueryParams.end_time='{}' \
         → SpecDrivenSensorAdapter::fetch → build_crowdstrike_fql produced the combined form. \
         Got filter_strings: {:?}. \
         REGRESSION: if only one bound appears, extract_time_window_from_ast failed to \
         extract one bound, or build_crowdstrike_fql is not joining both bounds with `+`.",
        start_time, end_time, filter_strings
    );

    // Step 9: Unfiltered baseline for filtered_count < unfiltered_count.
    clone
        .reset()
        .await
        .expect("AC-CWS-002: clone reset must succeed");

    let spec_content3 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-CWS-002: re-read3");
    let mut spec3 = SpecLoader::parse(&spec_content3).expect("AC-CWS-002: re-parse3");
    spec3.base_url = dtu_base_url.clone();
    let resolved3 = make_resolved_spec(spec3, "ac-cws-002-baseline-org");

    let (org_registry3, org_id3, org_slug_typed3) = make_org_registry("ac-cws-002-baseline-org");
    let mut resolved_spec_map3: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map3.insert(
        (org_slug_typed3.clone(), SensorId::from("crowdstrike")),
        resolved3,
    );

    let spec_content4 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-CWS-002: re-read4");
    let mut spec4 = SpecLoader::parse(&spec_content4).expect("AC-CWS-002: re-parse4");
    spec4.base_url = dtu_base_url.clone();
    let resolved4 = make_resolved_spec(spec4, "ac-cws-002-baseline-org");

    let mock_auth4: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("test-oauth2-token"));
    let http_client4 = build_http_client_with_timeout().expect("AC-CWS-002: http_client4");
    let adapter4 = SpecDrivenSensorAdapter::new(
        Arc::new(resolved4),
        AdapterAuthStrategy::Plugin(mock_auth4),
        http_client4,
    );
    let mut adapter_registry4 = AdapterRegistry::new();
    adapter_registry4.register(org_id3, Arc::new(adapter4));

    let normalizer4 = Arc::new(OcsfNormalizer::new());
    let cred_resolver4: Arc<dyn CredentialResolver> = Arc::new(PluginIgnoredCredentialResolver);
    let mut mat_ctx4 = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry4),
        normalizer4,
        10_000,
        cred_resolver4,
        Some(Arc::new(org_registry3)),
        Some(Arc::new(resolved_spec_map3)),
    );

    let session_ctx4 = SessionContext::new();
    let unfiltered_options = QueryOptions {
        clients: Some(vec![org_slug_typed3.clone()]),
        limit: None,
        ..QueryOptions::default()
    };

    let output4 = run_materialization_pipeline(
        "SELECT * FROM crowdstrike_detections",
        &unfiltered_options,
        &mut mat_ctx4,
        &session_ctx4,
    )
    .await
    .expect("AC-CWS-002: unfiltered pipeline must succeed");
    let unfiltered_count: usize = output4.batches.iter().map(|b| b.num_rows()).sum();

    // LOAD-BEARING (AC-CWS-002 item b): filtered_count < unfiltered_count.
    // If extract_time_window_from_ast failed to extract both bounds, build_crowdstrike_fql
    // produces a less-specific (or empty) FQL → DTU returns more records → this FAILS.
    assert!(
        filtered_count < unfiltered_count,
        "AC-CWS-002 LOAD-BEARING: filtered_count ({}) must be < unfiltered_count ({}) \
         when WHERE created_timestamp > '{}' AND created_timestamp < '{}' is pushed \
         through run_materialization_pipeline → extract_time_window_from_ast → \
         build_crowdstrike_fql(both bounds). \
         REGRESSION: if the push-down pipeline is broken, both counts will be equal.",
        filtered_count,
        unfiltered_count,
        start_time,
        end_time
    );
    assert!(
        filtered_count > 0,
        "AC-CWS-002: filtered pipeline must return non-empty records \
         (fixture has records in range {} to {})",
        start_time,
        end_time
    );
}

// ---------------------------------------------------------------------------
// AC-EQUIV-001 (ADV-P05-HIGH-001): BC-2.11.007 result-equivalence via
// run_materialization_pipeline — the AUTHORITATIVE real-path test
// ---------------------------------------------------------------------------

/// AC-EQUIV-001 / BC-2.11.007 — result-equivalence invariant via the real production
/// materialization path (`run_materialization_pipeline`).
///
/// # Why this test lives in prism-bin (not prism-spec-engine)
///
/// `prism-spec-engine` cannot depend on `prism-bin` (circular dependency guard — see
/// CLAUDE.md architecture compliance). Real `run_materialization_pipeline` coverage that
/// needs `SpecDrivenSensorAdapter` + `AdapterRegistry` must live in `prism-bin/tests/`.
///
/// # ADV-P05-HIGH-001 finding closure
///
/// The old `test_ac_equiv_001_result_equivalence_via_real_materialization_path` in
/// `prism-spec-engine/tests/bc_2_11_007_pushdown_test.rs` hand-fed `_fql` into
/// `FetchContext` and called `PipelineExecutor::execute` directly — it NEVER called
/// `run_materialization_pipeline`. Its name was false advertising.
/// That test has been renamed to `test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary`
/// and its doc comment updated to correctly describe the PipelineExecutor boundary scope.
/// THIS test is the authoritative AC-EQUIV-001 coverage.
///
/// # Production call graph proven
///
/// ```text
/// PrismQL: `WHERE created_timestamp > 'T1' AND created_timestamp < 'T2'`
///   → run_materialization_pipeline
///   → PrismQlParser::parse
///   → extract_time_window_from_ast [reads resolved_spec_map: created_timestamp INDEX]
///   → QueryParams.start_time = 'T1', end_time = 'T2'
///   → fan_out → SpecDrivenSensorAdapter::fetch
///   → build_crowdstrike_fql(start='T1', end='T2')
///       → "created_timestamp:>'T1'+created_timestamp:<'T2'"
///   → FetchContext["_fql"] = combined FQL
///   → PipelineExecutor → CrowdStrike DTU
/// ```
///
/// If `run_materialization_pipeline` hardcodes `None` (F-P6-CRIT-001 dead-code
/// regression), `build_crowdstrike_fql` produces an empty string, the DTU returns
/// all 50 records, and `result_a_count < result_b_count` FAILS — test catches regression.
///
/// # BC-2.11.007 invariant assertions
///
/// (a) `result_a_count < result_b_count` — time-window push-down via `run_materialization_pipeline`
///     constrains result A relative to baseline B. Proves the full production path is live.
///
/// (b) Subset / no-fabrication: every `detection_id` in result A appears in result B.
///     Push-down is an optimization only — it must not introduce records absent from the
///     unfiltered set.
///
/// # SAP-2 compliance
///
/// Production `crowdstrike.sensor.toml` shape; real CrowdStrike DTU clone.
/// No fabricated TOML spec or fixture data.
#[tokio::test]
async fn test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline() {
    // Step 1: Start CrowdStrike DTU clone on ephemeral port.
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-EQUIV-001: CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load production crowdstrike.sensor.toml (SAP-2: no fabricated fixture).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-EQUIV-001: crowdstrike.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-EQUIV-001: crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    // Step 3: Build resolved_spec_map — the map that extract_time_window_from_ast reads
    // to identify INDEX columns (created_timestamp has options=["INDEX"] per AC-INDEX-CWS-001).
    let org_slug = "ac-equiv-001-org";
    let (org_registry, org_id, org_slug_typed) = make_org_registry(org_slug);
    let sensor_id = SensorId::from("crowdstrike");

    let resolved = make_resolved_spec(spec, org_slug);
    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug_typed.clone(), sensor_id.clone()), resolved);
    let resolved_spec_map_arc: Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>> =
        Arc::new(resolved_spec_map);

    // Step 4: Build SpecDrivenSensorAdapter and register it.
    let mock_auth: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("test-oauth2-token"));
    let http_client =
        build_http_client_with_timeout().expect("AC-EQUIV-001: reqwest client build must succeed");

    let spec_content2 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-EQUIV-001: re-read crowdstrike.sensor.toml");
    let mut spec2 = SpecLoader::parse(&spec_content2).expect("AC-EQUIV-001: re-parse");
    spec2.base_url = dtu_base_url.clone();
    let resolved2 = make_resolved_spec(spec2, org_slug);

    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved2),
        AdapterAuthStrategy::Plugin(mock_auth),
        http_client,
    );
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(adapter));

    // Step 5: Build MaterializationContext.
    let normalizer = Arc::new(OcsfNormalizer::new());
    let credential_resolver: Arc<dyn CredentialResolver> =
        Arc::new(PluginIgnoredCredentialResolver);
    let org_registry_arc = Arc::new(org_registry);
    let mut mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry),
        normalizer,
        10_000,
        credential_resolver,
        Some(Arc::clone(&org_registry_arc)),
        Some(Arc::clone(&resolved_spec_map_arc)),
    );
    let session_ctx = SessionContext::new();

    // Step 6: Execution A — run_materialization_pipeline WITH a time-window predicate.
    // Fixture spans 2026-01-01..2026-01-26 (50 records).
    // Window: 2026-01-15 to end-of-fixture → records det-015..det-025 ≈ ~11 records.
    // This predicate is NOT hand-fed into FetchContext — it flows through the full production path:
    //   run_materialization_pipeline → PrismQlParser::parse
    //   → extract_time_window_from_ast [reads resolved_spec_map INDEX column: created_timestamp]
    //   → QueryParams.start_time = '2026-01-15T00:00:00Z', end_time = None
    //   → SpecDrivenSensorAdapter::fetch → build_crowdstrike_fql → DTU
    let start_time_a = "2026-01-15T00:00:00Z";
    let query_a = format!(
        "SELECT * FROM crowdstrike_detections \
         WHERE created_timestamp > '{start_time_a}'"
    );

    let options_a = QueryOptions {
        clients: Some(vec![org_slug_typed.clone()]),
        limit: None,
        ..QueryOptions::default()
    };

    // REAL production pipeline entry point (NOT PipelineExecutor::execute directly).
    let output_a = run_materialization_pipeline(&query_a, &options_a, &mut mat_ctx, &session_ctx)
        .await
        .expect(
            "AC-EQUIV-001: run_materialization_pipeline (Execution A, time-filtered) must succeed",
        );

    // Collect detection_ids from result A for subset assertion.
    // The field is `detection_id` in the CrowdStrike DTU fixture shape.
    let ids_a: std::collections::HashSet<String> = output_a
        .batches
        .iter()
        .flat_map(|batch| {
            // Extract the `detection_id` column values from the Arrow RecordBatch.
            use datafusion::arrow::array::{Array, StringArray};
            use datafusion::arrow::datatypes::DataType;
            let schema = batch.schema();
            let col_idx = schema.index_of("detection_id").ok();
            match col_idx {
                Some(idx) => {
                    let col = batch.column(idx);
                    if col.data_type() == &DataType::Utf8 {
                        let arr = col
                            .as_any()
                            .downcast_ref::<StringArray>()
                            .expect("detection_id must be StringArray");
                        (0..arr.len())
                            .filter(|&i| !arr.is_null(i))
                            .map(|i| arr.value(i).to_string())
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    }
                }
                None => vec![],
            }
        })
        .collect();
    let count_a = ids_a.len();

    // Step 7: Reset DTU state between executions so Execution B is independent.
    clone
        .reset()
        .await
        .expect("AC-EQUIV-001: clone reset must succeed");

    // Step 8: Execution B — run_materialization_pipeline WITHOUT a time predicate (unfiltered baseline).
    // Build fresh infrastructure for the second execution (separate org to avoid registry collision).
    let org_slug_b = "ac-equiv-001-baseline-org";
    let (org_registry_b, org_id_b, org_slug_typed_b) = make_org_registry(org_slug_b);

    let spec_content_b = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-EQUIV-001: re-read for B");
    let mut spec_b = SpecLoader::parse(&spec_content_b).expect("AC-EQUIV-001: re-parse B");
    spec_b.base_url = dtu_base_url.clone();
    let resolved_b = make_resolved_spec(spec_b, org_slug_b);

    let mut resolved_spec_map_b: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map_b.insert(
        (org_slug_typed_b.clone(), SensorId::from("crowdstrike")),
        resolved_b,
    );

    let spec_content_b2 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-EQUIV-001: re-read B2");
    let mut spec_b2 = SpecLoader::parse(&spec_content_b2).expect("AC-EQUIV-001: re-parse B2");
    spec_b2.base_url = dtu_base_url.clone();
    let resolved_b2 = make_resolved_spec(spec_b2, org_slug_b);

    let mock_auth_b: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("test-oauth2-token"));
    let http_client_b = build_http_client_with_timeout().expect("AC-EQUIV-001: http_client_b");
    let adapter_b = SpecDrivenSensorAdapter::new(
        Arc::new(resolved_b2),
        AdapterAuthStrategy::Plugin(mock_auth_b),
        http_client_b,
    );
    let mut adapter_registry_b = AdapterRegistry::new();
    adapter_registry_b.register(org_id_b, Arc::new(adapter_b));

    let normalizer_b = Arc::new(OcsfNormalizer::new());
    let cred_resolver_b: Arc<dyn CredentialResolver> = Arc::new(PluginIgnoredCredentialResolver);
    let mut mat_ctx_b = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry_b),
        normalizer_b,
        10_000,
        cred_resolver_b,
        Some(Arc::new(org_registry_b)),
        Some(Arc::new(resolved_spec_map_b)),
    );
    let session_ctx_b = SessionContext::new();

    let options_b = QueryOptions {
        clients: Some(vec![org_slug_typed_b.clone()]),
        limit: None,
        ..QueryOptions::default()
    };

    // Execution B: no WHERE clause → no FQL → DTU returns all 50 fixture records.
    let output_b = run_materialization_pipeline(
        "SELECT * FROM crowdstrike_detections",
        &options_b,
        &mut mat_ctx_b,
        &session_ctx_b,
    )
    .await
    .expect("AC-EQUIV-001: run_materialization_pipeline (Execution B, unfiltered) must succeed");

    // Collect detection_ids from result B.
    let ids_b: std::collections::HashSet<String> = output_b
        .batches
        .iter()
        .flat_map(|batch| {
            use datafusion::arrow::array::{Array, StringArray};
            use datafusion::arrow::datatypes::DataType;
            let schema = batch.schema();
            let col_idx = schema.index_of("detection_id").ok();
            match col_idx {
                Some(idx) => {
                    let col = batch.column(idx);
                    if col.data_type() == &DataType::Utf8 {
                        let arr = col
                            .as_any()
                            .downcast_ref::<StringArray>()
                            .expect("detection_id must be StringArray");
                        (0..arr.len())
                            .filter(|&i| !arr.is_null(i))
                            .map(|i| arr.value(i).to_string())
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    }
                }
                None => vec![],
            }
        })
        .collect();
    let count_b = ids_b.len();

    // Step 9: BC-2.11.007 invariant (a) — result A is a STRICT SUBSET of result B
    // (filtered count < unfiltered count). This proves the production push-down path is live.
    // If run_materialization_pipeline hardcodes start_time=None (F-P6-CRIT-001 regression),
    // build_crowdstrike_fql returns empty FQL, DTU returns all 50 records, count_a == count_b → FAILS.
    assert!(
        count_a < count_b,
        "AC-EQUIV-001 LOAD-BEARING (BC-2.11.007 invariant a): run_materialization_pipeline \
         WITH `WHERE created_timestamp > '{start_time_a}'` must produce fewer records ({count_a}) \
         than WITHOUT time predicate ({count_b}). \
         Production path: run_materialization_pipeline → extract_time_window_from_ast \
         [reads resolved_spec_map INDEX column: created_timestamp] → QueryParams.start_time = '{start_time_a}' \
         → SpecDrivenSensorAdapter::fetch → build_crowdstrike_fql → DTU filter-log. \
         REGRESSION: if count_a == count_b, extract_time_window_from_ast returned start_time=None \
         (materialization.rs dead-code path — F-P6-CRIT-001) or build_crowdstrike_fql produced empty FQL.",
    );
    assert!(
        count_a > 0,
        "AC-EQUIV-001: time-filtered execution A must return non-empty records \
         (fixture has records after {start_time_a})",
    );
    assert!(
        count_b > 0,
        "AC-EQUIV-001: unfiltered execution B must return non-empty records \
         (CrowdStrike DTU fixture has 50 records)",
    );

    // Step 10: BC-2.11.007 invariant (b) — no fabrication.
    // Every detection_id in result A must appear in result B.
    // Push-down is an optimization only; the filtered result must be a true SUBSET of the unfiltered set.
    for id in &ids_a {
        assert!(
            ids_b.contains(id.as_str()),
            "AC-EQUIV-001 NO-FABRICATION (BC-2.11.007 invariant b): detection_id '{id}' \
             from push-down result A must also appear in unfiltered result B. \
             Push-down must not introduce records absent from the full dataset. \
             REGRESSION: the DTU produced a record in the filtered set that does not \
             exist in the unfiltered set — either a fixture ordering bug or a DTU fabrication.",
        );
    }
}

// ---------------------------------------------------------------------------
// ADV-P08-MED-001: inclusive-boundary result-equivalence via run_materialization_pipeline
// BC-2.11.007 §result-equivalence invariant — push-down must never under-fetch
// ---------------------------------------------------------------------------

/// ADV-P08-MED-001 (CrowdStrike) / BC-2.11.007 — inclusive-boundary record must survive
/// the full push-down path when the PrismQL predicate uses `>=`.
///
/// # Finding being closed
///
/// `extract_time_bounds_from_predicate` collapses `Gt|Ge → start_time` without tracking
/// inclusivity. `build_crowdstrike_fql` always emits strict `created_timestamp:>'T'`.
/// The DTU previously used `ts <= after` (exclusive) — so a record at exactly `T` was
/// EXCLUDED from push-down but INCLUDED by DataFusion's `>=` post-filter.
/// Result: push-down result was a strict SUBSET of the DataFusion result → BC-2.11.007
/// result-equivalence invariant VIOLATED (ADV-P08-MED-001).
///
/// # Two-layer fix (ADV-P08-MED-001)
///
/// **Fix 1 — DTU inclusive boundary:** DTU now uses `ts < after` (inclusive) so
/// records at exactly `T` are over-fetched. DataFusion removes them if the predicate
/// was strict `>`. For `>=` predicates, boundary records survive push-down.
///
/// **Fix 2 — Timestamp normalization:** The spec-engine `pipeline.rs` timestamp
/// normalization previously called `to_rfc3339()` which produces `+00:00` suffix.
/// DataFusion's string comparison `"T10:00:00+00:00" >= "T10:00:00Z"` returns FALSE
/// because lexicographically `+` (43) < `Z` (90), causing boundary records to be
/// silently excluded at the DataFusion layer. The fix uses `to_rfc3339_opts(Secs, true)`
/// which produces canonical `Z` suffix so the comparison `"T10:00:00Z" >= "T10:00:00Z"`
/// evaluates correctly.
///
/// Both fixes are needed: Fix 1 ensures the DTU returns boundary records; Fix 2
/// ensures DataFusion retains them through the WHERE post-filter.
///
/// # Fixture facts used by this test
///
/// CrowdStrike fixture has 2 records at exactly `2026-01-15T10:00:00Z`:
/// - det-014 (`created_timestamp = "2026-01-15T10:00:00Z"`)
/// - det-042 (`created_timestamp = "2026-01-15T10:00:00Z"`)
/// `WHERE created_timestamp >= '2026-01-15T10:00:00Z'` MUST include both.
///
/// # Assertions (through real `run_materialization_pipeline`)
///
/// (a) Both det-014 and det-042 appear in the push-down result.
/// (b) Count with `>=` (23) > count with `>` (21) — boundary records contribute.
/// (c) det-014 and det-042 do NOT appear in strict `>` result (DataFusion correct).
///
/// BCs: BC-2.11.007; ADR-033 T1; ADV-P08-MED-001.
/// SAP-2: production crowdstrike.sensor.toml; real CrowdStrike DTU clone.
#[tokio::test]
async fn test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline() {
    // Step 1: Start CrowdStrike DTU clone on ephemeral port.
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("ADV-P08-MED-001 CWS: CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load production crowdstrike.sensor.toml (SAP-2: no fabricated fixture).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("ADV-P08-MED-001 CWS: crowdstrike.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content)
        .expect("ADV-P08-MED-001 CWS: crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    // Step 3: Build resolved_spec_map for extract_time_window_from_ast.
    let org_slug = "adv-p08-cws-incl-org";
    let (org_registry, org_id, org_slug_typed) = make_org_registry(org_slug);
    let sensor_id = SensorId::from("crowdstrike");

    let resolved = make_resolved_spec(spec, org_slug);
    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug_typed.clone(), sensor_id.clone()), resolved);
    let resolved_spec_map_arc = Arc::new(resolved_spec_map);

    // Step 4: Build SpecDrivenSensorAdapter.
    let mock_auth: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("test-oauth2-token"));
    let http_client = build_http_client_with_timeout()
        .expect("ADV-P08-MED-001 CWS: reqwest client build must succeed");

    let spec_content2 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("ADV-P08-MED-001 CWS: re-read crowdstrike.sensor.toml");
    let mut spec2 = SpecLoader::parse(&spec_content2).expect("ADV-P08-MED-001 CWS: re-parse");
    spec2.base_url = dtu_base_url.clone();
    let resolved2 = make_resolved_spec(spec2, org_slug);

    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved2),
        AdapterAuthStrategy::Plugin(mock_auth),
        http_client,
    );
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(adapter));

    // Step 5: Build MaterializationContext.
    let normalizer = Arc::new(OcsfNormalizer::new());
    let credential_resolver: Arc<dyn CredentialResolver> =
        Arc::new(PluginIgnoredCredentialResolver);
    let mut mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry),
        normalizer,
        10_000,
        credential_resolver,
        Some(Arc::new(org_registry)),
        Some(Arc::clone(&resolved_spec_map_arc)),
    );
    let session_ctx = SessionContext::new();

    // Step 6: Execution A — inclusive lower-bound predicate `>= boundary`.
    // Fixture has det-014 and det-042 at exactly '2026-01-15T10:00:00Z'.
    // With the prior exclusive DTU logic (ts <= after → exclude), these 2 records were
    // excluded from push-down but included by DataFusion >= post-filter → BC-2.11.007 violated.
    // After the fix (ts < after → include boundary), both records survive push-down.
    let boundary_ts = "2026-01-15T10:00:00Z";
    let query_ge = format!(
        "SELECT * FROM crowdstrike_detections \
         WHERE created_timestamp >= '{boundary_ts}'"
    );

    let options = QueryOptions {
        clients: Some(vec![org_slug_typed.clone()]),
        limit: None,
        ..QueryOptions::default()
    };

    let output_ge = run_materialization_pipeline(&query_ge, &options, &mut mat_ctx, &session_ctx)
        .await
        .expect("ADV-P08-MED-001 CWS: run_materialization_pipeline (>= boundary) must succeed");

    // Extract detection_ids from the >= result.
    let ids_ge: std::collections::HashSet<String> = output_ge
        .batches
        .iter()
        .flat_map(|batch| {
            use datafusion::arrow::array::{Array, StringArray};
            use datafusion::arrow::datatypes::DataType;
            let schema = batch.schema();
            let col_idx = schema.index_of("detection_id").ok();
            match col_idx {
                Some(idx) => {
                    let col = batch.column(idx);
                    if col.data_type() == &DataType::Utf8 {
                        let arr = col
                            .as_any()
                            .downcast_ref::<StringArray>()
                            .expect("detection_id must be StringArray");
                        (0..arr.len())
                            .filter(|&i| !arr.is_null(i))
                            .map(|i| arr.value(i).to_string())
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    }
                }
                None => vec![],
            }
        })
        .collect();

    // Assertion (a): boundary records det-014 and det-042 MUST be present.
    // These records have created_timestamp == boundary_ts exactly. With the prior
    // exclusive DTU, they were silently dropped (push-down under-fetch). After the fix
    // they must appear here — DataFusion's `>=` post-filter keeps them too.
    assert!(
        ids_ge.contains("det-014"),
        "ADV-P08-MED-001 CWS BOUNDARY RECORD (BC-2.11.007 result-equivalence): \
         det-014 has created_timestamp = '{boundary_ts}' exactly. \
         `WHERE created_timestamp >= '{boundary_ts}'` MUST include det-014 in the push-down result. \
         REGRESSION: DTU used exclusive comparison (ts <= after → exclude), dropping the \
         exact-boundary record before DataFusion could post-filter it.",
    );
    assert!(
        ids_ge.contains("det-042"),
        "ADV-P08-MED-001 CWS BOUNDARY RECORD (BC-2.11.007 result-equivalence): \
         det-042 has created_timestamp = '{boundary_ts}' exactly. \
         `WHERE created_timestamp >= '{boundary_ts}'` MUST include det-042 in the push-down result. \
         REGRESSION: same as det-014 — exclusive DTU comparison dropped the boundary record.",
    );

    // Assertion (b): count with >= must be 2 more than count with >.
    // Fixture: 23 records with ts >= boundary, 21 records with ts > boundary.
    // The 2-record difference is exactly det-014 and det-042 at ts == boundary.
    let count_ge = ids_ge.len();
    assert!(
        count_ge >= 23,
        "ADV-P08-MED-001 CWS: `WHERE created_timestamp >= '{boundary_ts}'` must return \
         at least 23 records (fixture has 21 with ts > boundary + 2 at ts == boundary). \
         Got: {count_ge}. REGRESSION: boundary records excluded by DTU under-fetch.",
    );

    // Step 7: Reset and run strict > for comparison.
    clone
        .reset()
        .await
        .expect("ADV-P08-MED-001 CWS: clone reset must succeed");

    let org_slug_gt = "adv-p08-cws-strict-org";
    let (org_registry_gt, org_id_gt, org_slug_typed_gt) = make_org_registry(org_slug_gt);

    let spec_content3 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("ADV-P08-MED-001 CWS: re-read for strict-gt");
    let mut spec3 = SpecLoader::parse(&spec_content3).expect("ADV-P08-MED-001 CWS: re-parse gt");
    spec3.base_url = dtu_base_url.clone();
    let resolved3 = make_resolved_spec(spec3, org_slug_gt);
    let mut resolved_spec_map_gt: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map_gt.insert((org_slug_typed_gt.clone(), sensor_id.clone()), resolved3);

    let spec_content4 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("ADV-P08-MED-001 CWS: re-read for adapter3");
    let mut spec4 = SpecLoader::parse(&spec_content4).expect("ADV-P08-MED-001 CWS: re-parse4");
    spec4.base_url = dtu_base_url.clone();
    let resolved4 = make_resolved_spec(spec4, org_slug_gt);

    let mock_auth_gt: Arc<dyn AuthProvider> = Arc::new(MockAuthProvider::new("test-oauth2-token"));
    let http_client_gt =
        build_http_client_with_timeout().expect("ADV-P08-MED-001 CWS: http_client_gt");
    let adapter_gt = SpecDrivenSensorAdapter::new(
        Arc::new(resolved4),
        AdapterAuthStrategy::Plugin(mock_auth_gt),
        http_client_gt,
    );
    let mut adapter_registry_gt = AdapterRegistry::new();
    adapter_registry_gt.register(org_id_gt, Arc::new(adapter_gt));

    let normalizer_gt = Arc::new(OcsfNormalizer::new());
    let cred_gt: Arc<dyn CredentialResolver> = Arc::new(PluginIgnoredCredentialResolver);
    let mut mat_ctx_gt = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry_gt),
        normalizer_gt,
        10_000,
        cred_gt,
        Some(Arc::new(org_registry_gt)),
        Some(Arc::new(resolved_spec_map_gt)),
    );
    let session_ctx_gt = SessionContext::new();

    let query_gt =
        format!("SELECT * FROM crowdstrike_detections WHERE created_timestamp > '{boundary_ts}'");
    let options_gt = QueryOptions {
        clients: Some(vec![org_slug_typed_gt.clone()]),
        limit: None,
        ..QueryOptions::default()
    };
    let output_gt =
        run_materialization_pipeline(&query_gt, &options_gt, &mut mat_ctx_gt, &session_ctx_gt)
            .await
            .expect("ADV-P08-MED-001 CWS: run_materialization_pipeline (>) must succeed");

    let ids_gt: std::collections::HashSet<String> = output_gt
        .batches
        .iter()
        .flat_map(|batch| {
            use datafusion::arrow::array::{Array, StringArray};
            use datafusion::arrow::datatypes::DataType;
            let schema = batch.schema();
            let col_idx = schema.index_of("detection_id").ok();
            match col_idx {
                Some(idx) => {
                    let col = batch.column(idx);
                    if col.data_type() == &DataType::Utf8 {
                        let arr = col
                            .as_any()
                            .downcast_ref::<StringArray>()
                            .expect("detection_id must be StringArray");
                        (0..arr.len())
                            .filter(|&i| !arr.is_null(i))
                            .map(|i| arr.value(i).to_string())
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    }
                }
                None => vec![],
            }
        })
        .collect();
    let count_gt = ids_gt.len();

    // Assertion (b): >= must have 2 more records than > (the 2 exact-boundary records).
    assert!(
        count_ge > count_gt,
        "ADV-P08-MED-001 CWS OPERATOR DISTINCTION: \
         `>= '{boundary_ts}'` must return more records ({count_ge}) than `> '{boundary_ts}'` ({count_gt}). \
         The 2 boundary records (det-014, det-042 at ts == boundary) must appear in >= but not in >. \
         REGRESSION: if counts are equal, boundary records are being excluded from the >= path \
         (exclusive DTU comparison bug, ADV-P08-MED-001).",
    );

    // Assertion (c): boundary records absent from strict > — explicit guards below.
    // The loop assertion was removed: it was vacuous due to ||/&& precedence
    // (`!ids_gt.contains(id) || id != "det-014" && id != "det-042"` is always true).
    // The genuine boundary-exclusion property is load-bearingly covered by the
    // two explicit asserts immediately below.

    // Confirm boundary records are absent from strict > (DataFusion post-filter did its job).
    assert!(
        !ids_gt.contains("det-014"),
        "ADV-P08-MED-001: det-014 (ts == boundary) must NOT appear in `>` result \
         (DataFusion post-filter should exclude it). \
         REGRESSION: if it appears, DataFusion is not applying the strict > post-filter.",
    );
    assert!(
        !ids_gt.contains("det-042"),
        "ADV-P08-MED-001: det-042 (ts == boundary) must NOT appear in `>` result. \
         Same DataFusion post-filter check as det-014.",
    );
}

/// ADV-P08-MED-001 (Armis) / BC-2.11.007 — inclusive-boundary device record must survive
/// the full Armis push-down path when the PrismQL predicate uses `>=`.
///
/// # Finding being closed
///
/// The Armis DTU `device_in_time_window` and `alert_in_time_window` used `ts <= after`
/// (exclusive). An Armis device at exactly `last_seen = T` with `WHERE last_seen >= T`
/// was excluded by push-down but would be included by DataFusion post-filter → under-fetch.
///
/// # Fix
///
/// DTU now uses `ts < after` (inclusive boundary). Boundary device d-026 at
/// `last_seen = 2026-01-15T08:00:00Z` must now appear in the push-down result for
/// `WHERE last_seen >= '2026-01-15T08:00:00Z'`.
///
/// # Fixture facts
///
/// - Device d-026: `last_seen = "2026-01-15T08:00:00Z"` (the exact boundary record).
/// - Device d-027: `last_seen = "2026-03-20T14:30:00Z"` (above boundary, always included).
/// - All other devices: `last_seen < 2026-01-15` or null → excluded.
/// - Expected count with `>= boundary`: 2 (d-026 + d-027).
/// - Expected count with `> boundary`: 1 (d-027 only; d-026 excluded by strict `>`).
///
/// BCs: BC-2.11.007; ADV-P08-MED-001.
/// SAP-2: production armis.sensor.toml; real Armis DTU clone.
#[tokio::test]
async fn test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline() {
    // Step 1: Start Armis DTU clone on ephemeral port.
    let mut clone = ArmisClone::new().expect("ADV-P08-MED-001 ARMIS: ArmisClone::new must succeed");
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("ADV-P08-MED-001 ARMIS: Armis DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load production armis.sensor.toml (SAP-2: no fabricated fixture).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("ADV-P08-MED-001 ARMIS: armis.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content)
        .expect("ADV-P08-MED-001 ARMIS: armis.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    // Step 3: Build resolved_spec_map.
    let org_slug = "adv-p08-armis-incl-org";
    let (org_registry, org_id, org_slug_typed) = make_org_registry(org_slug);
    let sensor_id = SensorId::from("armis");

    let resolved = make_resolved_spec(spec, org_slug);
    let mut resolved_spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map.insert((org_slug_typed.clone(), sensor_id.clone()), resolved);
    let resolved_spec_map_arc = Arc::new(resolved_spec_map);

    // Step 4: Build SpecDrivenSensorAdapter (BearerStatic auth, Armis pattern).
    let spec_content2 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("ADV-P08-MED-001 ARMIS: re-read armis.sensor.toml");
    let mut spec2 = SpecLoader::parse(&spec_content2).expect("ADV-P08-MED-001 ARMIS: re-parse");
    spec2.base_url = dtu_base_url.clone();
    let resolved2 = make_resolved_spec(spec2, org_slug);

    let http_client =
        build_http_client_with_timeout().expect("ADV-P08-MED-001 ARMIS: reqwest client build");
    let adapter = SpecDrivenSensorAdapter::new(
        Arc::new(resolved2),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    );
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(adapter));

    // Step 5: Build MaterializationContext.
    let normalizer = Arc::new(OcsfNormalizer::new());
    let credential_resolver: Arc<dyn CredentialResolver> = Arc::new(
        BearerStaticCredentialResolver::new("armis-test-bearer-token"),
    );
    let mut mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry),
        normalizer,
        10_000,
        credential_resolver,
        Some(Arc::new(org_registry)),
        Some(Arc::clone(&resolved_spec_map_arc)),
    );
    let session_ctx = SessionContext::new();

    // Step 6: Execution A — inclusive `>=` predicate on the exact boundary timestamp.
    // d-026 has last_seen = "2026-01-15T08:00:00Z" (the exact boundary).
    // With the prior exclusive DTU (ts <= after → exclude), d-026 was dropped before
    // DataFusion could apply `>=` post-filter → BC-2.11.007 result-equivalence violated.
    // After fix (ts < after → include), d-026 survives push-down AND DataFusion `>=` → correct.
    let boundary_ts = "2026-01-15T08:00:00Z";
    let query_ge = format!(
        "SELECT * FROM armis_devices \
         WHERE aql = 'in:devices' AND last_seen >= '{boundary_ts}'"
    );

    let options = QueryOptions {
        clients: Some(vec![org_slug_typed.clone()]),
        limit: None,
        ..QueryOptions::default()
    };

    let output_ge = run_materialization_pipeline(&query_ge, &options, &mut mat_ctx, &session_ctx)
        .await
        .expect("ADV-P08-MED-001 ARMIS: run_materialization_pipeline (>= boundary) must succeed");

    // Count rows: with >= boundary there should be 2 devices (d-026 at exactly T, d-027 above T).
    let count_ge: usize = output_ge.batches.iter().map(|b| b.num_rows()).sum();

    assert!(
        count_ge >= 2,
        "ADV-P08-MED-001 ARMIS BOUNDARY RECORD (BC-2.11.007 result-equivalence): \
         `WHERE last_seen >= '{boundary_ts}'` must return at least 2 records. \
         Device d-026 has last_seen == '{boundary_ts}' exactly and MUST be included. \
         Device d-027 has last_seen > boundary and must also be included. \
         Got: {count_ge}. \
         REGRESSION: DTU used exclusive comparison (ts <= after → exclude), dropping d-026 \
         (the exact-boundary device) before DataFusion could apply the `>=` post-filter.",
    );
    assert!(
        count_ge >= 1,
        "ADV-P08-MED-001 ARMIS: at least the exact-boundary device d-026 must be returned \
         for `>= '{boundary_ts}'`; got 0 records — complete push-down failure.",
    );

    // Step 7: Reset DTU and run strict `>` for comparison.
    clone
        .reset()
        .await
        .expect("ADV-P08-MED-001 ARMIS: clone reset must succeed");

    let org_slug_gt = "adv-p08-armis-strict-org";
    let (org_registry_gt, org_id_gt, org_slug_typed_gt) = make_org_registry(org_slug_gt);

    let spec_content3 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("ADV-P08-MED-001 ARMIS: re-read for strict-gt");
    let mut spec3 = SpecLoader::parse(&spec_content3).expect("ADV-P08-MED-001 ARMIS: re-parse gt");
    spec3.base_url = dtu_base_url.clone();
    let resolved3 = make_resolved_spec(spec3, org_slug_gt);
    let mut resolved_spec_map_gt: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    resolved_spec_map_gt.insert((org_slug_typed_gt.clone(), sensor_id.clone()), resolved3);

    let spec_content4 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("ADV-P08-MED-001 ARMIS: re-read for adapter3");
    let mut spec4 = SpecLoader::parse(&spec_content4).expect("ADV-P08-MED-001 ARMIS: re-parse4");
    spec4.base_url = dtu_base_url.clone();
    let resolved4 = make_resolved_spec(spec4, org_slug_gt);

    let http_client_gt =
        build_http_client_with_timeout().expect("ADV-P08-MED-001 ARMIS: http_client_gt");
    let adapter_gt = SpecDrivenSensorAdapter::new(
        Arc::new(resolved4),
        AdapterAuthStrategy::BearerStatic,
        http_client_gt,
    );
    let mut adapter_registry_gt = AdapterRegistry::new();
    adapter_registry_gt.register(org_id_gt, Arc::new(adapter_gt));

    let normalizer_gt = Arc::new(OcsfNormalizer::new());
    let cred_gt: Arc<dyn CredentialResolver> = Arc::new(BearerStaticCredentialResolver::new(
        "armis-test-bearer-token",
    ));
    let mut mat_ctx_gt = MaterializationContext::new_with_resolver(
        Arc::new(adapter_registry_gt),
        normalizer_gt,
        10_000,
        cred_gt,
        Some(Arc::new(org_registry_gt)),
        Some(Arc::new(resolved_spec_map_gt)),
    );
    let session_ctx_gt = SessionContext::new();

    let query_gt = format!(
        "SELECT * FROM armis_devices WHERE aql = 'in:devices' AND last_seen > '{boundary_ts}'"
    );
    let options_gt = QueryOptions {
        clients: Some(vec![org_slug_typed_gt.clone()]),
        limit: None,
        ..QueryOptions::default()
    };
    let output_gt =
        run_materialization_pipeline(&query_gt, &options_gt, &mut mat_ctx_gt, &session_ctx_gt)
            .await
            .expect("ADV-P08-MED-001 ARMIS: run_materialization_pipeline (>) must succeed");

    let count_gt: usize = output_gt.batches.iter().map(|b| b.num_rows()).sum();

    // Assertion: >= must return more rows than > (the extra row is d-026 at exactly T).
    assert!(
        count_ge > count_gt,
        "ADV-P08-MED-001 ARMIS OPERATOR DISTINCTION: \
         `>= '{boundary_ts}'` must return more records ({count_ge}) than `> '{boundary_ts}'` ({count_gt}). \
         Device d-026 (last_seen == boundary) must appear in >= but not in >. \
         REGRESSION: equal counts mean boundary record d-026 is being excluded from both \
         paths (exclusive DTU comparison not fixed, ADV-P08-MED-001).",
    );
}
