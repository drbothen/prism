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
