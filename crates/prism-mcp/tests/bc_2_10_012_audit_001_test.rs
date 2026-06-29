//! Red Gate test for S-DEMO-FIDELITY-REMEDIATION-001 AC-AUDIT-001 — BC-2.10.012 v1.5.
//!
//! Finding AUDIT-001: `build_tables_for_client` emits bare table names (`alerts`,
//! `devices`) instead of sensor-prefixed names (`crowdstrike_alerts`,
//! `crowdstrike_devices`). The `TableDescriptor.name` field (returned in
//! `prism_describe` response → `results.tables[*].name`) must be
//! `{sensor_id}_{table_name}` (e.g., `cyberint_alerts`), NOT `{table_name}`.
//!
//! Root cause: `build_tables_for_client` in `prism-mcp/src/tools/prism_describe.rs`:
//! - Multi-tenant path (line ~320): `name: table.table_name.clone()` → must be
//!   `name: format!("{sensor_id}_{}", table.table_name)`
//! - Single-tenant path (line ~368): `name: table.table_name.clone()` → must be
//!   `name: format!("{}_{}", sensor_spec.sensor_id, table.table_name)`
//!
//! `build_tables_for_client` is private — test routes through `handle_prism_describe`
//! which calls it internally and returns the result as a JSON SafetyEnvelope.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_bc_2_10_012_audit_001_sensor_prefixed_table_names | AUDIT-001 | BC-2.10.012 v1.5 |
//! | test_bc_2_10_012_audit_001_multi_tenant_sensor_prefixed_unique | AUDIT-001 (OBS-2) | BC-2.10.012 v1.5 |

use prism_core::column::ColumnType;
use prism_mcp::tools::prism_describe::handle_prism_describe;
use prism_spec_engine::{
    spec_parser::{ColumnSpec, SensorSpec, TableSpec},
    types::ConfigSnapshot,
    AuthType, ConfigManager,
};
use std::sync::{Arc, Mutex};

// ── Test fixture ──────────────────────────────────────────────────────────────

/// Build a `ConfigManager` with sensor `crowdstrike` having tables `alerts` and `devices`.
///
/// Uses the same pattern as `make_config_manager_acme_crowdstrike()` in mcp_prism_describe.rs.
/// The single-tenant path: `client_id == sensor_id == "crowdstrike"`.
fn make_config_manager_crowdstrike_two_tables() -> Arc<arc_swap::ArcSwap<ConfigManager>> {
    let alerts_table = TableSpec::new_point_in_time(
        "alerts",
        "security_finding",
        vec![ColumnSpec::new(
            "severity",
            ColumnType::String,
            Some("severity".to_string()),
            vec![],
        )],
        vec![],
    );

    let devices_table = TableSpec::new_point_in_time(
        "devices",
        "device_inventory_info",
        vec![ColumnSpec::new(
            "hostname",
            ColumnType::String,
            None,
            vec![],
        )],
        vec![],
    );

    let cs_spec = SensorSpec::new(
        "crowdstrike",
        "CrowdStrike Falcon sensor",
        AuthType::ApiKey,
        "https://api.crowdstrike.com",
        vec![alerts_table, devices_table],
        None,
        "1.0.0",
        vec![],
    );

    let mut sensor_specs = std::collections::HashMap::new();
    sensor_specs.insert("crowdstrike".to_string(), cs_spec);

    let snapshot = ConfigSnapshot {
        sensor_specs,
        ..ConfigSnapshot::empty()
    };
    Arc::new(arc_swap::ArcSwap::from_pointee(ConfigManager::new(
        snapshot,
    )))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// BC-2.10.012 v1.5 AUDIT-001 — Red Gate test.
///
/// `handle_prism_describe("crowdstrike", ...)` must return table descriptors with
/// `name` values set to `crowdstrike_alerts` and `crowdstrike_devices` (sensor-prefixed),
/// NOT bare `alerts` and `devices`.
///
/// # Red Gate failure
///
/// Current `build_tables_for_client` (single-tenant path, line ~368):
///   ```rust
///   TableDescriptor { name: table.table_name.clone(), ... }
///   ```
/// produces `name: "alerts"` instead of `name: "crowdstrike_alerts"`.
/// The test asserts the prefixed form is present and the bare form is absent.
///
/// # Why this matters
///
/// A `prism_describe` response with bare table names breaks the AI agent workflow:
/// agents build `FROM alerts | ...` queries (invalid PrismQL) instead of
/// `FROM crowdstrike_alerts | ...` (the correct form). This is the root cause of
/// the E-SENSOR-030 silent failures reported in the demo session evidence.
#[tokio::test]
async fn test_bc_2_10_012_audit_001_sensor_prefixed_table_names() {
    let config_manager = make_config_manager_crowdstrike_two_tables();

    let result = handle_prism_describe(
        "crowdstrike".to_string(),
        None,                  // single-tenant path: no query_engine
        Some(&config_manager), // sensor_spec source
        None,                  // no audit_writer (fail-open)
    )
    .await;

    let call_result = result.expect(
        "BC-2.10.012 AUDIT-001: handle_prism_describe must return Ok for valid client_id \
         'crowdstrike'; got Err",
    );

    assert!(
        !call_result.is_error.unwrap_or(false),
        "BC-2.10.012 AUDIT-001: prism_describe must not return is_error=true; \
         got: {:?}",
        call_result
    );

    // Extract JSON response.
    let content_text: String = call_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    let parsed: serde_json::Value = serde_json::from_str(&content_text).expect(
        "BC-2.10.012 AUDIT-001: prism_describe response must be valid JSON; \
         got non-JSON content",
    );

    // SafetyEnvelope: domain payload is under `results`.
    let results = parsed
        .get("results")
        .expect("BC-2.10.012 AUDIT-001: SafetyEnvelope response must have 'results' field");

    let tables = results
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AUDIT-001: response results must contain a 'tables' array");

    assert_eq!(
        tables.len(),
        2,
        "BC-2.10.012 AUDIT-001: expected 2 tables for crowdstrike; got {}. \
         Response: {}",
        tables.len(),
        content_text
    );

    // Collect the actual table names returned.
    let actual_names: Vec<&str> = tables
        .iter()
        .filter_map(|t| t.get("name").and_then(|v| v.as_str()))
        .collect();

    // ── Positive assertions: sensor-prefixed names must be present ────────────

    let expected_prefixed = ["crowdstrike_alerts", "crowdstrike_devices"];
    for name in &expected_prefixed {
        assert!(
            actual_names.contains(name),
            "BC-2.10.012 AUDIT-001: table 'name' must be sensor-prefixed '{}'. \
             Actual names: {:?}. Full response: {}",
            name,
            actual_names,
            content_text
        );
    }

    // ── Negative assertions: bare names must NOT appear ───────────────────────

    let bare_names = ["alerts", "devices"];
    for name in &bare_names {
        assert!(
            !actual_names.contains(name),
            "BC-2.10.012 AUDIT-001: bare table name '{}' must NOT appear in prism_describe \
             response — it causes AI agents to build invalid 'FROM {}' queries. \
             Actual names: {:?}",
            name,
            name,
            actual_names
        );
    }

    // ── example_query must reference the prefixed name ────────────────────────

    // After the fix, example_query must use the prefixed name (e.g., `FROM crowdstrike_alerts`).
    // This is also a guard that build_example_query is called with the corrected name.
    for table in tables {
        let name = table.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let example_query = table
            .get("example_query")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Only check tables that were successfully named with the prefix.
        if name.starts_with("crowdstrike_") {
            assert!(
                example_query.contains(name),
                "BC-2.10.012 AUDIT-001: example_query must reference sensor-prefixed name '{}'. \
                 Got example_query: {:?}",
                name,
                example_query
            );
        }
    }
}

/// OBS-2 (BC-2.10.012 AUDIT-001 multi-tenant path): `build_tables_for_client` via
/// `resolved_spec_map` must emit sensor-prefixed, UNIQUE `TableDescriptor.name` values
/// even when multiple sensors share the same underlying table names.
///
/// Scenario: org-c has 3 sensors (crowdstrike, claroty, armis), each with tables named
/// `alerts` and `devices`. Without sensor prefixing, the names would collide:
/// 3 × "alerts" and 3 × "devices" = 6 non-unique names. With correct prefixing they
/// become: crowdstrike_alerts, crowdstrike_devices, claroty_alerts, claroty_devices,
/// armis_alerts, armis_devices — 6 unique names.
///
/// This test drives `handle_prism_describe("org-c", ...)` via the MULTI-TENANT path
/// (wired `query_engine` with a `resolved_spec_map` containing 3 entries for org-c).
/// It asserts:
/// 1. All 6 expected sensor-prefixed names appear in the response.
/// 2. All 6 names are unique (no duplicates — collision-free prefixing).
/// 3. Bare names (`alerts`, `devices`) are absent (sensor prefix is mandatory).
#[tokio::test]
async fn test_bc_2_10_012_audit_001_multi_tenant_sensor_prefixed_unique() {
    use prism_core::{OrgId, OrgRegistry, OrgSlug, SensorId};
    use prism_credentials::InMemoryCredentialStore;
    use prism_query::engine::{QueryEngine, QueryEngineConfig};
    use prism_sensors::{
        registry::AdapterRegistry, CredentialResolver as SensorsCredentialResolver, SensorError,
    };
    use prism_spec_engine::{
        overlay::{OverlayLoader, SensorInstanceOverlay},
        spec_parser::ColumnSpec as CS,
    };
    use prism_storage::memory_backend::memory_backend_inner::InMemoryBackend;

    // ── Build resolved_spec_map for org-c with 3 sensors, each having alerts+devices ──

    let make_resolved = |sensor_id: &str, base_url: &str| {
        let spec = SensorSpec::new(
            sensor_id,
            format!("{sensor_id} sensor"),
            AuthType::ApiKey,
            base_url,
            vec![
                TableSpec::new_point_in_time(
                    "alerts",
                    "security_finding",
                    vec![CS::new("severity", ColumnType::String, None, vec![])],
                    vec![],
                ),
                TableSpec::new_point_in_time(
                    "devices",
                    "device_inventory_info",
                    vec![CS::new("hostname", ColumnType::String, None, vec![])],
                    vec![],
                ),
            ],
            None,
            "1.0.0",
            vec![],
        );
        let overlay_toml =
            format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@org-c\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("OBS-2 fixture: SensorInstanceOverlay TOML must parse");
        let org_slug = OrgSlug::new("org-c");
        let resolved =
            OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
        let sensor_id_typed = SensorId::new(sensor_id);
        let key = (org_slug, sensor_id_typed);
        (key, resolved)
    };

    let mut spec_map = std::collections::HashMap::new();
    let (k, v) = make_resolved("crowdstrike", "https://api.crowdstrike.com");
    spec_map.insert(k, v);
    let (k, v) = make_resolved("claroty", "https://api.claroty.com");
    spec_map.insert(k, v);
    let (k, v) = make_resolved("armis", "https://api.armis.com");
    spec_map.insert(k, v);
    let spec_map = Arc::new(spec_map);

    // ── Build OrgRegistry with org-c ──────────────────────────────────────────

    let org_reg = {
        let reg = OrgRegistry::new();
        reg.register(OrgSlug::new("org-c"), OrgId::new())
            .expect("OBS-2: OrgRegistry::register must succeed for 'org-c'");
        Arc::new(reg)
    };

    // ── Stub CredentialResolver (no real auth needed) ─────────────────────────

    struct StubCredResolver;
    impl SensorsCredentialResolver for StubCredResolver {
        fn resolve(
            &self,
            _client_id: &str,
            sensor_id: SensorId,
        ) -> Result<Box<dyn prism_sensors::SensorAuth>, SensorError> {
            Err(SensorError::Internal {
                detail: format!("StubCredResolver: no credential for {sensor_id:?} (OBS-2 stub)"),
            })
        }
    }

    let alias_store = Arc::new(Mutex::new(prism_query::alias_store::AliasStore::empty(
        std::path::Path::new("/tmp/test-prism-obs2"),
    )));

    let query_engine = Arc::new(QueryEngine::new_full(
        Arc::new(AdapterRegistry::new()),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        Arc::new(StubCredResolver),
        org_reg,
        Arc::new(InMemoryBackend::new()),
        spec_map,
        alias_store,
    ));

    // ── Call handle_prism_describe via the MULTI-TENANT path ──────────────────

    let result = handle_prism_describe(
        "org-c".to_string(),
        Some(&query_engine), // multi-tenant: resolved_spec_map from query_engine
        None,                // no config_manager — must use resolved_spec_map
        None,
    )
    .await;

    let call_result = result.expect(
        "BC-2.10.012 OBS-2: handle_prism_describe('org-c') with wired QueryEngine must return Ok",
    );

    assert!(
        !call_result.is_error.unwrap_or(false),
        "BC-2.10.012 OBS-2: prism_describe must not return is_error=true; got: {:?}",
        call_result
    );

    // ── Parse the table names from the JSON response ──────────────────────────

    let content_text: String = call_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    let parsed: serde_json::Value = serde_json::from_str(&content_text)
        .expect("BC-2.10.012 OBS-2: prism_describe response must be valid JSON");

    let results = parsed
        .get("results")
        .expect("BC-2.10.012 OBS-2: SafetyEnvelope response must have 'results' field");

    let tables = results
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 OBS-2: response results must contain a 'tables' array");

    let actual_names: Vec<&str> = tables
        .iter()
        .filter_map(|t| t.get("name").and_then(|v| v.as_str()))
        .collect();

    // ── 1. All 6 sensor-prefixed names must be present ────────────────────────

    let expected: &[&str] = &[
        "crowdstrike_alerts",
        "crowdstrike_devices",
        "claroty_alerts",
        "claroty_devices",
        "armis_alerts",
        "armis_devices",
    ];

    for expected_name in expected {
        assert!(
            actual_names.contains(expected_name),
            "BC-2.10.012 OBS-2 MULTI-TENANT: expected sensor-prefixed table name '{}' \
             to appear in prism_describe('org-c') response (multi-tenant resolved_spec_map path). \
             Actual names: {:?}. Full response: {}",
            expected_name,
            actual_names,
            content_text
        );
    }

    // ── 2. All names must be unique (no prefix-collision) ─────────────────────

    let unique_count = {
        let mut sorted = actual_names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        sorted.len()
    };

    assert_eq!(
        unique_count,
        actual_names.len(),
        "BC-2.10.012 OBS-2 MULTI-TENANT: all {} table names must be UNIQUE (sensor-prefixed \
         to avoid collision when multiple sensors share the same base table names). \
         Got {} names with {} unique. Duplicates detected in: {:?}. Full response: {}",
        actual_names.len(),
        actual_names.len(),
        unique_count,
        actual_names,
        content_text
    );

    // ── 3. Bare unprefixed names must NOT appear ──────────────────────────────

    let bare: &[&str] = &["alerts", "devices"];
    for bare_name in bare {
        assert!(
            !actual_names.contains(bare_name),
            "BC-2.10.012 OBS-2 MULTI-TENANT: bare table name '{}' must NOT appear — \
             all names must be sensor-prefixed. Found bare name in: {:?}",
            bare_name,
            actual_names
        );
    }
}

// ── AC-CAT2: Category-2 enrichment UDF discovery hints (BC-2.10.012 v1.7 §pql_hints) ──────────
//
// These 3 tests cover the Category-2 enrichment-discovery hint introduced by BC-2.10.012 v1.7
// (ADV-P208-P02-001 follow-on; human directive: implement AC-CAT2 in-scope on PR #208).
//
// Test vehicle: end-to-end via handle_prism_describe (SID-1: drives the real production path).
//   - Tests 1 and 2: assert pql_hints.len() == 3 (2 Cat-1 + 1 Cat-2).
//   - Test 3: assert pql_hints.len() == 1 (Cat-2 suppressed for N = 0 tables).
//
// Red Gate status:
//   - test_bc_2_10_012_cat2_enrichment_hint_with_udfs: ASSERTION-FAIL
//     (build_pql_hints currently emits 2 hints only; 4th param infusion_registry not yet added)
//   - test_bc_2_10_012_cat2_enrichment_absent_hint: ASSERTION-FAIL (same reason)
//   - test_bc_2_10_012_cat2_zero_table_no_category2: REGRESSION GUARD (passes vacuously
//     before implementation; guards against Cat-2 being emitted in the zero-table case)
//
// Implementation prerequisites (AC-CAT2 spec from story crates_touched §prism-mcp):
//   1. `InfusionUdfDescriptor` gains `pub input_field: String` (prism-spec-engine udf.rs + new()).
//   2. `udf_descriptors()` propagates `field.input_field.clone()` into each descriptor (mod.rs).
//   3. `build_pql_hints` gains 4th param `infusion_registry: Option<&prism_spec_engine::InfusionRegistry>`.
//   4. `handle_prism_describe` resolves
//      `let infusion_registry = query_engine.and_then(|qe| qe.infusion_registry());`
//      and passes `infusion_registry.as_deref()` as 4th arg to `build_pql_hints`.

/// Build the canonical Cat-2 fixture InfusionRegistry with 2 UDFs per BC-2.10.012 v1.7 EC-10-030:
///   - `nvd_cvss`     (input_field: `"device_cves_first"`)   — alphabetically first ('n' < 't')
///   - `threat_score` (input_field: `"ioc_value_singleton"`) — alphabetically second
///
/// After the AC-CAT2 fix: `InfusionUdfDescriptor` gains `pub input_field: String`; `udf_descriptors()`
/// propagates `field.input_field` into each descriptor; `build_pql_hints` uses `d.input_field` to
/// format each UDF entry as `"d.name(d.input_field)"`.
///
/// The `InfusionSpec::new()` sets `source: None` → `NullSource` for `LocalLookup` (no real data
/// file needed; enrichment lookup is not exercised by these tests). `pipe_stage: None` skips
/// `validate_pipe_stage_columns`. Both specs load successfully.
fn make_cat2_infusion_registry() -> Arc<prism_spec_engine::InfusionRegistry> {
    use prism_spec_engine::{InfusionField, InfusionRegistry, InfusionSpec, InfusionType};

    let registry = Arc::new(InfusionRegistry::new());

    // UDF 1: nvd_cvss — sorts before threat_score ('n' < 't') → becomes the example call.
    let nvd_spec = InfusionSpec::new(
        "nvd",
        "NVD CVE Lookup",
        InfusionType::LocalLookup,
        vec![InfusionField::new(
            "nvd_cvss",
            "device_cves_first",
            "string",
            "float",
        )],
        "/test/nvd.infusion.toml",
    );
    registry.load_spec(nvd_spec).expect(
        "AC-CAT2 fixture: nvd spec load must succeed (1 field, no duplicates, no pipe_stage)",
    );

    // UDF 2: threat_score — sorts after nvd_cvss alphabetically.
    let threat_spec = InfusionSpec::new(
        "threat_intel",
        "Threat Intelligence Lookup",
        InfusionType::LocalLookup,
        vec![InfusionField::new(
            "threat_score",
            "ioc_value_singleton",
            "string",
            "float",
        )],
        "/test/threat.infusion.toml",
    );
    registry
        .load_spec(threat_spec)
        .expect("AC-CAT2 fixture: threat_intel spec load must succeed");

    registry
}

/// BC-2.10.012 v1.7 AC-CAT2 (test 1 of 3) — Red Gate (assertion-fail).
///
/// When `infusion_registry` is `Some(reg)` with 2 non-empty UDFs AND N ≥ 1 tables, `pql_hints`
/// MUST have exactly 3 elements; `pql_hints[2]` MUST be the byte-exact enrichment-presence hint
/// with UDFs sorted alphabetically by name, each formatted as `name(input_field)`.
///
/// # Byte-exact expected string (BC-2.10.012 v1.7 §pql_hints Category-2, EC-10-030):
///
/// `"Enrichment available via pipe syntax: | enrich nvd_cvss(device_cves_first). Available UDFs
/// for this client: nvd_cvss(device_cves_first), threat_score(ioc_value_singleton)"`
///
/// # Wiring: how the infusion_registry reaches build_pql_hints
///
/// `query_engine` is wired via `.with_infusion_registry(registry)`. After the AC-CAT2 fix,
/// `handle_prism_describe` resolves:
/// `let infusion_registry = query_engine.and_then(|qe| qe.infusion_registry());`
/// and passes `infusion_registry.as_deref()` as the 4th arg to `build_pql_hints`.
///
/// `QueryEngine::new()` leaves `resolved_spec_map: None`, so `build_tables_for_client` falls
/// through to `config_manager` for table data — 2 crowdstrike tables (N = 2).
///
/// # Red Gate failure (assertion-fail)
///
/// Currently `build_pql_hints` emits exactly 2 Category-1 hints for non-empty tables
/// (`pql_hints.len() == 2`). `assert_eq!(pql_hints.len(), 3)` fails → Red Gate confirmed.
#[tokio::test]
async fn test_bc_2_10_012_cat2_enrichment_hint_with_udfs() {
    use prism_credentials::InMemoryCredentialStore;
    use prism_ocsf::OcsfNormalizer;
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig},
        scoping::ClientRegistry,
    };
    use prism_sensors::registry::AdapterRegistry;

    // InfusionRegistry with nvd_cvss and threat_score (2 UDFs per BC-2.10.012 EC-10-030).
    let infusion_registry = make_cat2_infusion_registry();

    // Minimal QueryEngine wired with infusion_registry; no resolved_spec_map.
    // handle_prism_describe falls through to config_manager for table data.
    let query_engine = Arc::new(
        QueryEngine::new(
            Arc::new(AdapterRegistry::new()),
            Arc::new(InMemoryCredentialStore::new()),
            Arc::new(OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
        )
        .with_infusion_registry(infusion_registry),
    );

    // config_manager with 2 crowdstrike tables (N = 2 ≥ 1 → Category-2 must be emitted).
    let config_manager = make_config_manager_crowdstrike_two_tables();

    let result = handle_prism_describe(
        "crowdstrike".to_string(),
        Some(&query_engine), // infusion_registry wired; resolved_spec_map = None
        Some(&config_manager), // N = 2 tables via single-tenant fallback
        None,
    )
    .await;

    let call_result = result.expect(
        "BC-2.10.012 AC-CAT2: handle_prism_describe must return Ok for valid 'crowdstrike'",
    );
    assert!(
        !call_result.is_error.unwrap_or(false),
        "BC-2.10.012 AC-CAT2: prism_describe must not return is_error=true; got: {:?}",
        call_result
    );

    let content_text: String = call_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    let parsed: serde_json::Value = serde_json::from_str(&content_text)
        .expect("BC-2.10.012 AC-CAT2: prism_describe response must be valid JSON");

    let results = parsed
        .get("results")
        .expect("BC-2.10.012 AC-CAT2: SafetyEnvelope response must have 'results' field");

    let pql_hints = results
        .get("pql_hints")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-CAT2: results must contain 'pql_hints' array");

    // ── RED GATE: currently pql_hints.len() == 2 (no Category-2 hint emitted yet) ──────────
    assert_eq!(
        pql_hints.len(),
        3,
        "BC-2.10.012 AC-CAT2: non-empty tables + UDFs registered → pql_hints MUST have \
         exactly 3 elements (2 Category-1 + 1 Category-2 enrichment-presence hint). \
         Currently fails because build_pql_hints does not yet accept infusion_registry (4th param). \
         Got {} elements: {:?}",
        pql_hints.len(),
        pql_hints
    );

    let hint2 = pql_hints[2]
        .as_str()
        .expect("BC-2.10.012 AC-CAT2: pql_hints[2] must be a JSON string");

    // Byte-exact assertion per BC-2.10.012 v1.7 §pql_hints Category-2, EC-10-030:
    // Sort order: nvd_cvss < threat_score ('n' < 't'); first sorted entry is the example call.
    // Entry format: "name(input_field)" — requires InfusionUdfDescriptor.input_field (not yet present).
    const EXPECTED_CAT2_WITH_UDFS: &str = concat!(
        "Enrichment available via pipe syntax: | enrich nvd_cvss(device_cves_first). ",
        "Available UDFs for this client: nvd_cvss(device_cves_first), threat_score(ioc_value_singleton)"
    );
    assert_eq!(
        hint2, EXPECTED_CAT2_WITH_UDFS,
        "BC-2.10.012 AC-CAT2: pql_hints[2] must be byte-exact enrichment hint with UDFs sorted \
         alphabetically as 'name(input_field)' pairs. Got: {:?}",
        hint2
    );
}

/// BC-2.10.012 v1.7 AC-CAT2 (test 2 of 3) — Red Gate (assertion-fail).
///
/// When `infusion_registry` is `None` (no enrichment configured) AND N ≥ 1 tables, `pql_hints`
/// MUST have exactly 3 elements; `pql_hints[2]` MUST be the byte-exact enrichment-absence hint.
///
/// # Byte-exact expected string (BC-2.10.012 v1.7 §pql_hints Category-2, EC-10-031):
///
/// `"No enrichment UDFs are registered for this client — enrichment is not available."`
///
/// # Test vehicle note
///
/// Uses `query_engine = None` (single-tenant path through config_manager). After the fix,
/// `handle_prism_describe` resolves `infusion_registry = None` (from `None` query_engine)
/// and passes it to `build_pql_hints` as the 4th parameter. This exercises the "None registry"
/// case per BC-2.10.012 v1.7: both `None` and empty registry emit the absence hint.
///
/// # Red Gate failure (assertion-fail)
///
/// Currently `build_pql_hints` emits exactly 2 Category-1 hints for non-empty tables.
/// `assert_eq!(pql_hints.len(), 3)` fails → Red Gate confirmed.
#[tokio::test]
async fn test_bc_2_10_012_cat2_enrichment_absent_hint() {
    // No query_engine → infusion_registry resolves to None after the fix.
    // N = 2 tables via config_manager (non-empty → Category-2 must be emitted with absence hint).
    let config_manager = make_config_manager_crowdstrike_two_tables();

    let result = handle_prism_describe(
        "crowdstrike".to_string(),
        None,                  // no query_engine → infusion_registry = None after fix
        Some(&config_manager), // N = 2 tables
        None,
    )
    .await;

    let call_result = result.expect(
        "BC-2.10.012 AC-CAT2 absent: handle_prism_describe must return Ok for valid 'crowdstrike'",
    );
    assert!(
        !call_result.is_error.unwrap_or(false),
        "BC-2.10.012 AC-CAT2 absent: prism_describe must not return is_error=true",
    );

    let content_text: String = call_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    let parsed: serde_json::Value = serde_json::from_str(&content_text)
        .expect("BC-2.10.012 AC-CAT2 absent: response must be valid JSON");

    let results = parsed
        .get("results")
        .expect("BC-2.10.012 AC-CAT2 absent: must have 'results' field");

    let pql_hints = results
        .get("pql_hints")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-CAT2 absent: must have 'pql_hints' array");

    // ── RED GATE: currently pql_hints.len() == 2 (no Category-2 hint emitted yet) ──────────
    assert_eq!(
        pql_hints.len(),
        3,
        "BC-2.10.012 AC-CAT2 absent: non-empty tables + no InfusionRegistry → pql_hints MUST \
         have 3 elements (2 Category-1 + 1 Category-2 absence hint). \
         Currently fails because build_pql_hints does not yet emit Category-2 for None registry. \
         Got {} elements: {:?}",
        pql_hints.len(),
        pql_hints
    );

    let hint2 = pql_hints[2]
        .as_str()
        .expect("BC-2.10.012 AC-CAT2 absent: pql_hints[2] must be a JSON string");

    // Byte-exact assertion per BC-2.10.012 v1.7 §pql_hints Category-2, EC-10-031:
    const EXPECTED_CAT2_ABSENT: &str =
        "No enrichment UDFs are registered for this client — enrichment is not available.";
    assert_eq!(
        hint2, EXPECTED_CAT2_ABSENT,
        "BC-2.10.012 AC-CAT2 absent: pql_hints[2] must be byte-exact absence hint. Got: {:?}",
        hint2
    );
}

/// BC-2.10.012 v1.7 AC-CAT2 (test 3 of 3) — Regression guard.
///
/// When UDFs ARE registered but N = 0 tables, Category-2 MUST be suppressed entirely.
/// `pql_hints` must have exactly 1 element (the zero-table Category-1 hint only).
///
/// # Regression guard semantics (not a pure Red Gate)
///
/// This test currently PASSES vacuously before the AC-CAT2 implementation: Category-2 does not
/// exist yet, so the zero-table path already produces 1 hint. It becomes load-bearing after the
/// fix: if the implementer accidentally emits Category-2 for the zero-table case,
/// `assert_eq!(pql_hints.len(), 1)` fails here.
///
/// # Setup
///
/// `client_id = "zero-tables-sensor"` has no entry in `config_manager`'s `sensor_specs`
/// (which only contains "crowdstrike"). `build_tables_for_client` returns `Vec::new()` (N = 0).
/// `query_engine` is wired with 2 UDFs to verify suppression holds even when UDFs are present.
#[tokio::test]
async fn test_bc_2_10_012_cat2_zero_table_no_category2() {
    use prism_credentials::InMemoryCredentialStore;
    use prism_ocsf::OcsfNormalizer;
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig},
        scoping::ClientRegistry,
    };
    use prism_sensors::registry::AdapterRegistry;

    // QueryEngine wired with 2 UDFs — but client "zero-tables-sensor" has no tables.
    let infusion_registry = make_cat2_infusion_registry();
    let query_engine = Arc::new(
        QueryEngine::new(
            Arc::new(AdapterRegistry::new()),
            Arc::new(InMemoryCredentialStore::new()),
            Arc::new(OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
        )
        .with_infusion_registry(infusion_registry),
    );

    // config_manager has "crowdstrike" tables but we request "zero-tables-sensor".
    // build_tables_for_client: resolved_spec_map = None → config_manager → no matching sensor
    // → returns Vec::new() (N = 0).
    let config_manager = make_config_manager_crowdstrike_two_tables();

    let result = handle_prism_describe(
        "zero-tables-sensor".to_string(), // no matching sensor spec → N = 0
        Some(&query_engine),
        Some(&config_manager),
        None,
    )
    .await;

    let call_result =
        result.expect("BC-2.10.012 AC-CAT2 zero-table: handle_prism_describe must return Ok");

    let content_text: String = call_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    let parsed: serde_json::Value = serde_json::from_str(&content_text)
        .expect("BC-2.10.012 AC-CAT2 zero-table: response must be valid JSON");

    let results = parsed
        .get("results")
        .expect("BC-2.10.012 AC-CAT2 zero-table: must have 'results' field");

    let pql_hints = results
        .get("pql_hints")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-CAT2 zero-table: must have 'pql_hints' array");

    // N = 0 → Category-2 is suppressed; exactly 1 zero-table hint (BC-2.10.012 §pql_hints Cat-2).
    // Load-bearing regression guard: after implementation, 2 here means Cat-2 leaked into
    // the zero-table path.
    assert_eq!(
        pql_hints.len(),
        1,
        "BC-2.10.012 AC-CAT2 zero-table: N=0 tables → Category-2 MUST be suppressed; \
         pql_hints must have exactly 1 element (zero-table Category-1 hint only). \
         Got {} elements: {:?}",
        pql_hints.len(),
        pql_hints
    );

    // Guard: the single hint must be the zero-table informational hint, NOT an enrichment hint.
    // Prevents Cat-2 from leaking into the zero-table path even when UDFs are registered.
    let hint0 = pql_hints[0]
        .as_str()
        .expect("BC-2.10.012 AC-CAT2 zero-table: pql_hints[0] must be a JSON string");
    assert!(
        !hint0.contains("Enrichment") && !hint0.contains("UDF"),
        "BC-2.10.012 AC-CAT2 zero-table: pql_hints[0] for zero-table case must be the \
         zero-table informational hint, NOT an enrichment hint. Got: {:?}",
        hint0
    );
}
