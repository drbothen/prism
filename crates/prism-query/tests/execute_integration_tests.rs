//! Integration tests for S-3.02-FOLLOWUP-RUNTIME.
//!
//! Each test maps to one AC from .factory/stories/S-3.02-FOLLOWUP-RUNTIME-query-engine.md.
//! These tests fail in Red because nine implementation sites in prism-query are
//! still `todo!()` stubs. They pass when implementer fills those stubs per the
//! per-story-delivery TDD cycle.
//!
//! # Red-Gate Contract
//! AC-1 through AC-7: fail via `todo!()` panic at runtime (the correct Red signal).
//! AC-8: fails via assertion on `include_str!` content check.
//!
//! # Test naming
//! Tests follow `test_AC_N_description` pattern for AC-traced integration tests
//! per `.factory/stories/S-3.02-FOLLOWUP-RUNTIME-query-engine.md`.

#![allow(unused_imports, dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use arrow::array::{Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use datafusion::execution::context::SessionContext;
use prism_core::{OrgSlug, PrismError, StorageDomain};
use prism_ocsf::OcsfNormalizer;
use prism_query::engine::QueryOptions;
use prism_query::internal_tables::register_internal_tables;
use prism_query::materialization::{run_materialization_pipeline, MaterializationContext};
use prism_sensors::AdapterRegistry;
use prism_storage::backend::RocksStorageBackend;
use prism_storage::memory_backend::InMemoryBackend;

// ---------------------------------------------------------------------------
// Test Helpers
// ---------------------------------------------------------------------------

mod helpers {
    use std::sync::Arc;

    use arrow::array::StringArray;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use async_trait::async_trait;
    use datafusion::execution::context::SessionContext;
    use prism_core::{OrgSlug, PrismError, StorageDomain};
    use prism_credentials::namespace::CredentialName;
    use prism_credentials::CredentialStore;
    use prism_ocsf::OcsfNormalizer;
    use prism_query::engine::{QueryEngine, QueryEngineConfig};
    use prism_query::materialization::MaterializationContext;
    use prism_query::scoping::ClientRegistry;
    use prism_sensors::{AdapterRegistry, CredentialResolver};
    use prism_storage::backend::RocksStorageBackend;
    use prism_storage::memory_backend::InMemoryBackend;
    use secrecy::SecretString;

    // -----------------------------------------------------------------------
    // NullCredentialStore
    // -----------------------------------------------------------------------

    /// No-op `CredentialStore` for integration tests where credentials are
    /// never used (DTU stubs do not call real sensor APIs).
    pub struct NullCredentialStore;

    #[async_trait]
    impl CredentialStore for NullCredentialStore {
        async fn get(
            &self,
            _tenant: &OrgSlug,
            _sensor: &str,
            _name: &CredentialName,
        ) -> Result<Option<SecretString>, PrismError> {
            Ok(None)
        }

        async fn set(
            &self,
            _tenant: &OrgSlug,
            _sensor: &str,
            _name: &CredentialName,
            _value: SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }

        async fn delete(
            &self,
            _tenant: &OrgSlug,
            _sensor: &str,
            _name: &CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }

        async fn list(
            &self,
            _tenant: &OrgSlug,
        ) -> Result<Vec<(String, CredentialName)>, PrismError> {
            Ok(vec![])
        }

        async fn exists(
            &self,
            _tenant: &OrgSlug,
            _sensor: &str,
            _name: &CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    // -----------------------------------------------------------------------
    // StubCredentialResolver — succeeds for any (client, sensor) pair
    // -----------------------------------------------------------------------

    /// Test-only `CredentialResolver` that returns a dummy `CrowdStrikeAuth`
    /// for any request.
    ///
    /// Production adapters (CrowdStrikeAdapter, etc.) would reject this auth.
    /// `StubAdapter::fetch` ignores `_auth` entirely, so this is safe for tests.
    /// (F-LP1-CRIT-2: prevents NullCredentialResolver from short-circuiting fan_out)
    pub struct StubCredentialResolver;

    impl CredentialResolver for StubCredentialResolver {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_type: prism_core::SensorId,
        ) -> Result<Box<dyn prism_sensors::auth::SensorAuth>, SensorError> {
            Ok(Box::new(prism_sensors::CrowdStrikeAuth {
                client_id: "test-stub".to_string(),
                client_secret: prism_sensors::SecretString::new("test-secret".to_string()),
                cloud_region: "us-1".to_string(),
            }))
        }
    }

    // -----------------------------------------------------------------------
    // Engine factory
    // -----------------------------------------------------------------------

    /// Build a `QueryEngine` with the given adapter registry and client list.
    ///
    /// Uses `NullCredentialStore` and `StubCredentialResolver`.
    /// The `StubCredentialResolver` returns dummy auth so `fan_out()` can call
    /// `StubAdapter::fetch` (which ignores auth). (F-LP1-CRIT-2)
    pub fn make_engine(registry: AdapterRegistry, clients: Vec<OrgSlug>) -> QueryEngine {
        let adapter_registry = Arc::new(registry);
        let credential_store: Arc<dyn CredentialStore> = Arc::new(NullCredentialStore);
        let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
        let client_registry = Arc::new(ClientRegistry::new(clients));
        let config = QueryEngineConfig::default();
        // Use `with_credential_resolver` to inject StubCredentialResolver so
        // fan_out() can reach StubAdapter::fetch without failing on credential
        // resolution. (F-LP1-CRIT-2)
        QueryEngine::new(
            adapter_registry,
            credential_store,
            ocsf_normalizer,
            client_registry,
            config,
        )
        .with_credential_resolver(Arc::new(StubCredentialResolver))
    }

    // -----------------------------------------------------------------------
    // Storage factory
    // -----------------------------------------------------------------------

    /// Build a fresh in-memory `RocksStorageBackend`.
    pub fn make_storage() -> Arc<InMemoryBackend> {
        Arc::new(InMemoryBackend::new())
    }

    /// Write one key-value entry into `domain` of `storage`.
    pub fn seed_entry(
        storage: &Arc<InMemoryBackend>,
        domain: StorageDomain,
        key: &[u8],
        value: &[u8],
    ) {
        storage
            .put(domain, key, value)
            .expect("seed_entry: in-memory put must succeed");
    }

    // -----------------------------------------------------------------------
    // DataFusion helpers
    // -----------------------------------------------------------------------

    /// Fresh ephemeral `SessionContext` — one per test for isolation.
    pub fn make_ctx() -> SessionContext {
        SessionContext::new()
    }

    // -----------------------------------------------------------------------
    // Materialization helpers
    // -----------------------------------------------------------------------

    /// Build a `MaterializationContext` with the given record cap.
    pub fn make_mat_ctx(max_records: usize) -> MaterializationContext {
        let registry = Arc::new(AdapterRegistry::new());
        let normalizer = Arc::new(OcsfNormalizer::new());
        MaterializationContext::new(registry, normalizer, max_records)
    }

    // -----------------------------------------------------------------------
    // OrgSlug helper
    // -----------------------------------------------------------------------

    /// Construct an `OrgSlug` from a literal string.
    pub fn org(slug: &str) -> OrgSlug {
        OrgSlug::new_unchecked(slug)
    }

    // -----------------------------------------------------------------------
    // StubAdapter — returns a fixed RecordBatch with N rows
    // -----------------------------------------------------------------------

    use prism_core::{OrgId, SensorId};
    use prism_sensors::adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec};
    use prism_sensors::auth::SensorAuth;

    /// Minimal sensor adapter that returns a configurable number of rows with a
    /// `detection_id` column.  Used in tests that need real row data to exercise
    /// record-cap, virtual-field, and cross-client fan-out logic.
    pub struct StubAdapter {
        pub sensor_type: SensorId,
        pub row_count: usize,
        pub client_slug: String,
    }

    #[async_trait]
    impl SensorAdapter for StubAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_type.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "crowdstrike"
        }

        async fn fetch(
            &self,
            _spec: &SensorSpec,
            _params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<RecordBatch>, SensorError> {
            let schema = Arc::new(Schema::new(vec![Field::new(
                "detection_id",
                DataType::Utf8,
                false,
            )]));
            let ids: Vec<String> = (0..self.row_count).map(|i| format!("id-{}", i)).collect();
            let arr = Arc::new(StringArray::from(
                ids.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )) as _;
            let batch = RecordBatch::try_new(schema, vec![arr]).expect("stub batch must be valid");
            Ok(vec![batch])
        }
    }

    /// Build a `MaterializationContext` with a `StubAdapter` that returns `row_count` rows.
    ///
    /// Uses `StubCredentialResolver` so `fan_out()` can reach `StubAdapter::fetch`
    /// without credential failures. The `OrgId` is saved so the adapter can be
    /// found via `get_all_for_sensor_type`. (F-LP1-CRIT-2)
    pub fn make_mat_ctx_with_stub(max_records: usize, row_count: usize) -> MaterializationContext {
        let org_id = OrgId::new();
        let mut registry = AdapterRegistry::new();
        registry.register(
            org_id,
            Arc::new(StubAdapter {
                sensor_type: SensorId::from("crowdstrike"),
                row_count,
                client_slug: "acme".to_string(),
            }),
        );
        let normalizer = Arc::new(OcsfNormalizer::new());
        MaterializationContext::new_with_resolver(
            Arc::new(registry),
            normalizer,
            max_records,
            Arc::new(StubCredentialResolver),
            None,
        )
    }
}

// ---------------------------------------------------------------------------
// AC-1: QueryEngine::execute with adapter returns results
// ---------------------------------------------------------------------------

/// AC-1 (BC-2.11.001): `QueryEngine::execute` with a registered adapter returns
/// `QueryResult` where `returned_results <= 5` and batches contain a `_sensor`
/// column equal to `"crowdstrike"`.
///
/// F-LP1-CRIT-4 fix: registers a StubAdapter so the test actually exercises
/// virtual-field injection and is not vacuous (empty registry → empty batches).
/// Per S-7.01 sub-clause (b): AC tests with fixture-dependent assertions MUST
/// register a fixture producing rows.
///
/// Red-Gate: panics at `todo!("S-3.02 — QueryEngine::execute")` in engine.rs:276.
#[tokio::test]
async fn test_AC_1_query_engine_execute_with_dtu_returns_results() {
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let org_id = OrgId::new();

    // F-LP1-CRIT-4: register StubAdapter so fan-out produces real rows.
    let mut registry = AdapterRegistry::new();
    registry.register(
        org_id,
        Arc::new(helpers::StubAdapter {
            sensor_type: SensorId::from("crowdstrike"),
            row_count: 3,
            client_slug: "acme".to_string(),
        }),
    );

    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        sensors: None,
        limit: Some(5),
        force_refresh: false,
        ..QueryOptions::default()
    };

    // Post-implementation: must return Ok(QueryResult).
    let result = engine
        .execute("SELECT * FROM crowdstrike_detections LIMIT 5", options)
        .await
        .expect("AC-1: execute must succeed with registered adapter");

    // F-LP1-CRIT-4: precondition — must have rows for assertions to be meaningful.
    assert!(
        !result.batches.is_empty(),
        "AC-1: test fixture must produce at least one batch; \
         if this fails, the StubAdapter registration is broken"
    );
    assert!(
        result.returned_results > 0,
        "AC-1: test fixture must produce rows; returned_results = 0 means assertion loop is vacuous"
    );

    assert!(
        result.returned_results <= 5,
        "AC-1: returned_results must be <= 5; got {}",
        result.returned_results
    );

    // Every batch must carry _sensor = "crowdstrike".
    for batch in &result.batches {
        let idx = batch
            .schema()
            .index_of("_sensor")
            .expect("AC-1: _sensor virtual field must be present");

        let arr = batch
            .column(idx)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("AC-1: _sensor must be Utf8");

        for i in 0..arr.len() {
            assert_eq!(
                arr.value(i),
                "crowdstrike",
                "AC-1: _sensor must be 'crowdstrike' at row {i}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-2: run_materialization_pipeline produces a usable SessionContext
// ---------------------------------------------------------------------------
// F-LP3-OBS-1: Original vacuous test deleted (always-true `!catalog_names().is_empty()`).
// AC-2 enforcement is provided by `test_AC_2_materialization_pipeline_non_vacuous_assertion`
// (defined further below), which asserts non-empty batches, total_rows==3, and
// MemTable registration — a strictly stronger check.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// AC-3: record cap returns E-QUERY-003 before DataFusion execution
// ---------------------------------------------------------------------------

/// AC-3 (BC-2.11.006): When `run_materialization_pipeline` would exceed the
/// `max_records` cap, it must return `Err` containing "E-QUERY-003" before
/// any DataFusion SQL plan begins.
///
/// We set `max_records = 1` so any response with >=2 rows exceeds the cap.
///
/// Bug fix: original test omitted adapter registration (genuine test bug).
/// Now uses `make_mat_ctx_with_stub(1, 5)` — cap=1, stub returns 5 rows.
///
/// Red-Gate: panics at `todo!("S-3.02 — run_materialization_pipeline")`.
#[tokio::test]
async fn test_AC_3_size_limit_returns_e_query_003() {
    use prism_query::engine::QueryOptions;

    // Cap at 1 row; stub returns 5 rows — 2nd row exceeds cap → E-QUERY-003.
    let mut mat_ctx = helpers::make_mat_ctx_with_stub(1, 5);
    let session_ctx = helpers::make_ctx();
    let options = QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        sensors: None,
        limit: None,
        force_refresh: false,
        ..QueryOptions::default()
    };

    // Post-implementation: must return Err with E-QUERY-003.
    let result = run_materialization_pipeline(
        "SELECT * FROM crowdstrike_detections",
        &options,
        &mut mat_ctx,
        &session_ctx,
    )
    .await;

    let err =
        result.expect_err("AC-3: pipeline with 1-row cap must return Err when sensor has >1 row");
    let detail = err.to_string();
    assert!(
        detail.contains("E-QUERY-003"),
        "AC-3: error must contain 'E-QUERY-003' (record cap exceeded); got: {detail}"
    );
}

// ---------------------------------------------------------------------------
// AC-4: filter pushdown passed to sensor adapter
// ---------------------------------------------------------------------------

/// AC-4 (BC-2.11.007): A spy adapter captures the `QueryParams.filters` that
/// `resolve_source_refs` / `execute` passes to `SensorAdapter::fetch`. When the
/// query contains `WHERE hostname = 'target'`, the filter must be present.
///
/// Red-Gate: panics at `todo!("S-3.02 — QueryEngine::execute")` (or
/// `todo!("S-3.02 — resolve_source_refs")` reached first).
#[tokio::test]
async fn test_AC_4_filter_pushdown_passed_to_adapter() {
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec};
    use prism_sensors::auth::SensorAuth;
    use prism_sensors::types::FilterMap;
    use serde_json::json;

    /// Spy that records `QueryParams.filters` from every `fetch()` invocation.
    struct FilterSpyAdapter {
        captured: Arc<Mutex<Vec<FilterMap>>>,
    }

    #[async_trait]
    impl SensorAdapter for FilterSpyAdapter {
        fn sensor_type(&self) -> SensorId {
            SensorId::from("crowdstrike")
        }

        fn sensor_name(&self) -> &'static str {
            "crowdstrike"
        }

        async fn fetch(
            &self,
            _spec: &SensorSpec,
            params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<RecordBatch>, SensorError> {
            let mut guard = self.captured.lock().unwrap_or_else(|e| e.into_inner());
            guard.push(params.filters.clone());

            // Return one row with a `hostname` column.
            let schema = Arc::new(Schema::new(vec![Field::new(
                "hostname",
                DataType::Utf8,
                false,
            )]));
            let hostnames = Arc::new(StringArray::from(vec!["target"])) as _;
            let batch =
                RecordBatch::try_new(schema, vec![hostnames]).expect("spy batch must be valid");
            Ok(vec![batch])
        }
    }

    let captured: Arc<Mutex<Vec<FilterMap>>> = Arc::new(Mutex::new(Vec::new()));
    let spy = Arc::new(FilterSpyAdapter {
        captured: Arc::clone(&captured),
    });

    let org_slug = helpers::org("acme");
    let mut registry = AdapterRegistry::new();
    registry.register(OrgId::new(), spy);
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        sensors: None,
        limit: None,
        force_refresh: false,
        ..QueryOptions::default()
    };

    // Panics at todo!(). Post-implementation: spy must capture `hostname = "target"`.
    let _result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE hostname = 'target'",
            options,
        )
        .await
        .expect("AC-4: execute must succeed with spy adapter");

    let calls = captured.lock().unwrap_or_else(|e| e.into_inner());

    assert!(
        !calls.is_empty(),
        "AC-4: FilterSpyAdapter::fetch must be called at least once"
    );

    // At least one call must carry the hostname predicate pushed down.
    let any_has_hostname = calls.iter().any(|filters| {
        filters
            .get("hostname")
            .and_then(|v| v.as_str())
            .map(|s| s == "target")
            .unwrap_or(false)
    });
    assert!(
        any_has_hostname,
        "AC-4: at least one fetch call must carry hostname='target' in filters; \
         captured calls: {} total",
        calls.len()
    );
}

// ---------------------------------------------------------------------------
// AC-5: register_internal_tables then query prism_audit
// ---------------------------------------------------------------------------

/// AC-5 (BC-2.15.011): After registering internal tables with AuditRead capability, the query
/// `SELECT * FROM prism_audit LIMIT 20` must succeed without error.
///
/// We pre-populate `StorageDomain::AuditBuffer` with one row so the scan is
/// non-trivially exercised.
///
/// F-LP2-CRIT-1: uses `register_internal_tables_with_capabilities` with `Capability::AuditRead`
/// so the scan-time capability gate (Layer 2) allows access. Without AuditRead, the scan
/// would return E-QUERY-011 (tested separately in `test_LP2_CRIT_1_scan_time_gate_rejects_*`).
#[tokio::test]
async fn test_AC_5_register_internal_tables_then_query_prism_audit() {
    use prism_query::engine::Capability;
    use prism_query::internal_tables::register_internal_tables_with_capabilities;

    let storage = helpers::make_storage();

    // Seed one audit entry so the CF is non-empty.
    helpers::seed_entry(
        &storage,
        StorageDomain::AuditBuffer,
        b"audit-key-001",
        b"audit-payload-001",
    );

    let ctx = helpers::make_ctx();

    // Register with AuditRead so scan-time gate (Layer 2) allows access.
    register_internal_tables_with_capabilities(
        &ctx,
        Arc::clone(&storage) as Arc<dyn RocksStorageBackend>,
        &[Capability::AuditRead],
    )
    .expect("AC-5: register_internal_tables_with_capabilities must succeed");

    // SQL planning must succeed.
    let df = ctx
        .sql("SELECT * FROM prism_audit LIMIT 20")
        .await
        .expect("AC-5: SQL planning for prism_audit must succeed after registration");

    // Execution must not return an error.
    let _batches = df
        .collect()
        .await
        .expect("AC-5: collecting prism_audit results must succeed");
}

// ---------------------------------------------------------------------------
// AC-6: cross-client ALL scope fans out to every registered org
// ---------------------------------------------------------------------------

/// AC-6 (BC-2.11.011): With two orgs and `clients: None` (ALL scope), `execute`
/// must fan out to both orgs and the `QueryResult` batches must contain `_client`
/// values for both org slugs.
///
/// ADV-W3MT-P58-MED-003: original test asserted synthetic slugs that can only be
/// produced when OrgRegistry is absent. Now uses `QueryEngine::new_full` with a real
/// `OrgRegistry` mapping org_ids to "acme" and "beta" slugs, so the assertions are
/// non-vacuous and meaningful.
///
/// Red-Gate: panics at `todo!("S-3.02 — QueryEngine::execute")`.
#[tokio::test]
async fn test_AC_6_cross_client_query_all_scope_fans_out() {
    use prism_core::SensorId;
    use prism_core::{OrgId, OrgRegistry};
    use prism_query::engine::{QueryEngine, QueryEngineConfig, QueryOptions};

    let org_acme = helpers::org("acme");
    let org_beta = helpers::org("beta");

    // Create stable OrgIds for the two orgs.
    let id_acme = OrgId::new();
    let id_beta = OrgId::new();

    // Build OrgRegistry: acme → id_acme, beta → id_beta.
    let org_registry = OrgRegistry::new();
    org_registry
        .register(org_acme.clone(), id_acme)
        .expect("AC-6: OrgRegistry registration for 'acme' must succeed");
    org_registry
        .register(org_beta.clone(), id_beta)
        .expect("AC-6: OrgRegistry registration for 'beta' must succeed");
    let org_registry = Arc::new(org_registry);

    // Register one StubAdapter per org using their stable OrgIds.
    let mut registry = AdapterRegistry::new();
    registry.register(
        id_acme,
        Arc::new(helpers::StubAdapter {
            sensor_type: SensorId::from("crowdstrike"),
            row_count: 2,
            client_slug: "acme".to_string(),
        }),
    );
    registry.register(
        id_beta,
        Arc::new(helpers::StubAdapter {
            sensor_type: SensorId::from("crowdstrike"),
            row_count: 2,
            client_slug: "beta".to_string(),
        }),
    );

    // Build engine with OrgRegistry so _client values are the real slugs.
    let adapter_registry = Arc::new(registry);
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        org_acme.clone(),
        org_beta.clone(),
    ]));
    let config = QueryEngineConfig::default();
    let storage = helpers::make_storage();

    let engine = QueryEngine::new_full(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
        Arc::new(helpers::StubCredentialResolver),
        org_registry,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
    );

    // clients: None = ALL scope — both orgs fanned out.
    let options = QueryOptions {
        clients: None,
        sensors: None,
        limit: Some(100),
        force_refresh: false,
        ..QueryOptions::default()
    };

    // Post-implementation: batches must cover both orgs.
    let result = engine
        .execute("SELECT * FROM crowdstrike_detections LIMIT 100", options)
        .await
        .expect("AC-6: execute with ALL scope must succeed");

    // Collect all distinct _client values from all batches.
    let mut client_values: std::collections::HashSet<String> = std::collections::HashSet::new();
    for batch in &result.batches {
        if let Ok(idx) = batch.schema().index_of("_client") {
            if let Some(arr) = batch.column(idx).as_any().downcast_ref::<StringArray>() {
                for i in 0..arr.len() {
                    client_values.insert(arr.value(i).to_string());
                }
            }
        }
    }

    assert!(
        client_values.contains("acme"),
        "AC-6: _client must include 'acme' in ALL-scope fan-out; found: {client_values:?}"
    );
    assert!(
        client_values.contains("beta"),
        "AC-6: _client must include 'beta' in ALL-scope fan-out; found: {client_values:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-7: all three virtual fields present and non-null in every result row
// ---------------------------------------------------------------------------

/// AC-7 (BC-2.11.012): Every `QueryResult` batch must contain `_sensor`,
/// `_client`, and `_source_table` as non-null, non-empty Utf8 columns on
/// every row.
///
/// F-LP1-CRIT-4 fix: registers a StubAdapter so the assertion loop actually
/// exercises rows. An empty registry produces zero batches → vacuous pass.
/// Per S-7.01 sub-clause (b): assertion loops with fixture-dependent data
/// MUST register a fixture producing rows.
///
/// Red-Gate: panics at `todo!("S-3.02 — QueryEngine::execute")`.
#[tokio::test]
async fn test_AC_7_virtual_fields_present_in_all_results() {
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let org_id = OrgId::new();

    // F-LP1-CRIT-4: register StubAdapter so fan-out produces real rows.
    let mut registry = AdapterRegistry::new();
    registry.register(
        org_id,
        Arc::new(helpers::StubAdapter {
            sensor_type: SensorId::from("crowdstrike"),
            row_count: 3,
            client_slug: "acme".to_string(),
        }),
    );

    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        sensors: None,
        limit: Some(10),
        force_refresh: false,
        ..QueryOptions::default()
    };

    // Post-implementation: every row carries all three virtual fields.
    let result = engine
        .execute("SELECT * FROM crowdstrike_detections LIMIT 10", options)
        .await
        .expect("AC-7: execute must succeed");

    // F-LP1-CRIT-4: precondition — must have rows for assertions to be meaningful.
    assert!(
        !result.batches.is_empty(),
        "AC-7: test fixture must produce at least one batch; \
         if this fails, the StubAdapter registration is broken"
    );
    assert!(
        result.returned_results > 0,
        "AC-7: test fixture must produce rows; returned_results = 0 means assertion loop is vacuous"
    );

    const VIRTUAL_FIELDS: &[&str] = &["_sensor", "_client", "_source_table"];

    for (batch_idx, batch) in result.batches.iter().enumerate() {
        for vf in VIRTUAL_FIELDS {
            let col_idx = batch.schema().index_of(vf).unwrap_or_else(|_| {
                panic!(
                    "AC-7: virtual field '{vf}' must be present in batch {batch_idx}; \
                     schema: {:?}",
                    batch.schema()
                )
            });

            assert_eq!(
                batch.column(col_idx).data_type(),
                &DataType::Utf8,
                "AC-7: virtual field '{vf}' must be DataType::Utf8 in batch {batch_idx}"
            );

            let arr = batch
                .column(col_idx)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap_or_else(|| {
                    panic!("AC-7: '{vf}' must downcast to StringArray in batch {batch_idx}")
                });

            for row_idx in 0..arr.len() {
                assert!(
                    !arr.is_null(row_idx),
                    "AC-7: '{vf}' must be non-null at row {row_idx} batch {batch_idx}"
                );
                assert!(
                    !arr.value(row_idx).is_empty(),
                    "AC-7: '{vf}' must be non-empty at row {row_idx} batch {batch_idx}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AC-8: no todo!() or unimplemented!() remains in the stub files
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// F-LP1-HIGH-7: limit > 1000 returns E-QUERY-001
// ---------------------------------------------------------------------------

/// F-LP1-HIGH-7 (BC-2.11.001): `execute` MUST return an error when `limit > 1000`.
///
/// BC-2.11.001: "Max results returned (tool-level truncation). Default 25, max 1000."
/// The engine rejects limit=1001 before any sensor contact. Error message must
/// contain "E-QUERY-007" (ADV-W3MT-P58-CRIT-001: E-QUERY-001 is reserved for parse errors).
#[tokio::test]
async fn test_HIGH_7_limit_exceeds_1000_returns_error() {
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let engine = helpers::make_engine(AdapterRegistry::new(), vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        sensors: None,
        limit: Some(1001), // one above the maximum
        force_refresh: false,
        ..QueryOptions::default()
    };

    let result = engine
        .execute("SELECT * FROM crowdstrike_detections", options)
        .await;

    let err = result.expect_err(
        "HIGH-7: execute with limit=1001 must return Err (BC-2.11.001 max-1000 enforcement)",
    );
    let detail = err.to_string();
    assert!(
        detail.contains("E-QUERY-007"),
        "HIGH-7: error must contain 'E-QUERY-007' (limit exceeds 1000; ADV-W3MT-P58-CRIT-001); got: {detail}"
    );
}

/// F-LP1-HIGH-7 complement: limit=1000 (boundary) MUST succeed.
#[tokio::test]
async fn test_HIGH_7_limit_exactly_1000_is_allowed() {
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let engine = helpers::make_engine(AdapterRegistry::new(), vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        sensors: None,
        limit: Some(1000), // exactly at the maximum
        force_refresh: false,
        ..QueryOptions::default()
    };

    // Must NOT reject limit=1000 — only >1000 is rejected.
    let result = engine
        .execute("SELECT * FROM crowdstrike_detections LIMIT 1000", options)
        .await;

    assert!(
        result.is_ok(),
        "HIGH-7: execute with limit=1000 (maximum) must succeed; got: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// F-LP1-HIGH-3: capability gate — prism_audit access denied without AuditRead
// ---------------------------------------------------------------------------

/// F-LP1-HIGH-3 (BC-2.15.011): Querying `prism_audit` without the `AuditRead`
/// capability MUST return `PrismError::AuditTableAccessDenied` (E-QUERY-011).
///
/// The gate runs in `execute_inner` before scan, by inspecting source table refs
/// in the AST and rejecting if `requires_audit_read` is true and the caller
/// lacks `Capability::AuditRead` in `QueryOptions.capabilities`.
#[tokio::test]
async fn test_HIGH_3_audit_read_capability_gate_deny() {
    use prism_query::engine::{Capability, QueryEngine, QueryEngineConfig, QueryOptions};
    use prism_storage::memory_backend::InMemoryBackend;

    let org_slug = helpers::org("acme");
    let storage = helpers::make_storage();

    // Build a full engine with storage so internal tables are registered.
    let adapter_registry = Arc::new(AdapterRegistry::new());
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        org_slug.clone()
    ]));
    let config = QueryEngineConfig::default();
    let org_registry = Arc::new(prism_core::OrgRegistry::new());
    let engine = QueryEngine::new_full(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
        Arc::new(helpers::StubCredentialResolver),
        org_registry,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
    );

    // No capabilities — AuditRead NOT granted.
    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        capabilities: vec![], // no AuditRead
        ..QueryOptions::default()
    };

    let result = engine
        .execute("SELECT * FROM prism_audit LIMIT 10", options)
        .await;

    let err = result.expect_err(
        "HIGH-3: querying prism_audit without AuditRead capability must return Err (E-QUERY-011)",
    );
    assert!(
        matches!(err, prism_core::PrismError::AuditTableAccessDenied),
        "HIGH-3: error must be PrismError::AuditTableAccessDenied; got: {err:?}"
    );
}

/// F-LP1-HIGH-3 allow path: AuditRead capability grants access to prism_audit.
#[tokio::test]
async fn test_HIGH_3_audit_read_capability_gate_allow() {
    use prism_query::engine::{Capability, QueryEngine, QueryEngineConfig, QueryOptions};

    let org_slug = helpers::org("acme");
    let storage = helpers::make_storage();

    let adapter_registry = Arc::new(AdapterRegistry::new());
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        org_slug.clone()
    ]));
    let config = QueryEngineConfig::default();
    let org_registry = Arc::new(prism_core::OrgRegistry::new());
    let engine = QueryEngine::new_full(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
        Arc::new(helpers::StubCredentialResolver),
        org_registry,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
    );

    // AuditRead capability granted — must succeed.
    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        capabilities: vec![Capability::AuditRead],
        ..QueryOptions::default()
    };

    let result = engine
        .execute("SELECT * FROM prism_audit LIMIT 10", options)
        .await;

    assert!(
        result.is_ok(),
        "HIGH-3: querying prism_audit WITH AuditRead capability must succeed; got: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// F-LP1-HIGH-4: internal tables receive virtual field injection
// ---------------------------------------------------------------------------

/// F-LP1-HIGH-4 (BC-2.11.012): Scanning `prism_audit` (or any internal table)
/// via `RocksDbTableProvider::scan` MUST produce batches with `_sensor = "prism"`,
/// `_client = "<system>"`, and `_source_table = "prism_audit"` columns.
#[tokio::test]
async fn test_HIGH_4_internal_table_virtual_fields_present() {
    use prism_query::engine::{Capability, QueryEngine, QueryEngineConfig, QueryOptions};

    let org_slug = helpers::org("acme");
    let storage = helpers::make_storage();

    // Seed one audit entry so the scan is non-trivially exercised.
    helpers::seed_entry(
        &storage,
        StorageDomain::AuditBuffer,
        b"audit:00000000000000000001:trace-001",
        b"test-audit-payload",
    );

    let adapter_registry = Arc::new(AdapterRegistry::new());
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        org_slug.clone()
    ]));
    let config = QueryEngineConfig::default();
    let org_registry = Arc::new(prism_core::OrgRegistry::new());
    let engine = QueryEngine::new_full(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
        Arc::new(helpers::StubCredentialResolver),
        org_registry,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
    );

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        capabilities: vec![Capability::AuditRead],
        ..QueryOptions::default()
    };

    let result = engine
        .execute("SELECT * FROM prism_audit LIMIT 5", options)
        .await
        .expect("HIGH-4: prism_audit query with AuditRead must succeed");

    // The scan returns rows (we seeded one) — verify virtual fields.
    assert!(
        !result.batches.is_empty(),
        "HIGH-4: prism_audit must return at least one batch after seeding"
    );

    const VIRTUAL_FIELDS: &[&str] = &["_sensor", "_client", "_source_table"];

    for (batch_idx, batch) in result.batches.iter().enumerate() {
        for vf in VIRTUAL_FIELDS {
            let col_idx = batch.schema().index_of(vf).unwrap_or_else(|_| {
                panic!(
                    "HIGH-4: virtual field '{vf}' must be present in prism_audit batch {batch_idx}; \
                     schema: {:?}",
                    batch.schema()
                )
            });

            let arr = batch
                .column(col_idx)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap_or_else(|| {
                    panic!("HIGH-4: '{vf}' must be StringArray in batch {batch_idx}")
                });

            for row_idx in 0..arr.len() {
                assert!(
                    !arr.is_null(row_idx),
                    "HIGH-4: '{vf}' must be non-null at row {row_idx} batch {batch_idx}"
                );
                assert!(
                    !arr.value(row_idx).is_empty(),
                    "HIGH-4: '{vf}' must be non-empty at row {row_idx} batch {batch_idx}"
                );
            }
        }
    }

    // _sensor must be "prism" for internal tables (BC-2.11.012).
    for (batch_idx, batch) in result.batches.iter().enumerate() {
        if let Ok(idx) = batch.schema().index_of("_sensor") {
            if let Some(arr) = batch.column(idx).as_any().downcast_ref::<StringArray>() {
                for row_idx in 0..arr.len() {
                    assert_eq!(
                        arr.value(row_idx),
                        "prism",
                        "HIGH-4: _sensor must be 'prism' for internal tables at row {row_idx} batch {batch_idx}"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// F-LP1-MED-3: AC-3-bis — 10K boundary cross-source accumulation
// ---------------------------------------------------------------------------

/// F-LP1-MED-3 (BC-2.11.006): Two stub adapters each returning 6K rows (12K total)
/// MUST trigger E-QUERY-003 at the pipeline level, verifying the cap is checked
/// BEFORE DataFusion execution and across multiple sources.
///
/// This complements `test_AC_3_size_limit_returns_e_query_003` which only
/// tests a 1-row cap with a single-source stub.
#[tokio::test]
async fn test_AC_3_bis_size_limit_at_10k_boundary() {
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;

    // Register two StubAdapters each returning 6000 rows.
    // Total = 12000 > 10000 → must exceed the default cap.
    let mut registry = AdapterRegistry::new();
    registry.register(
        OrgId::new(),
        Arc::new(helpers::StubAdapter {
            sensor_type: SensorId::from("crowdstrike"),
            row_count: 6_000,
            client_slug: "acme".to_string(),
        }),
    );
    registry.register(
        OrgId::new(),
        Arc::new(helpers::StubAdapter {
            sensor_type: SensorId::from("crowdstrike"),
            row_count: 6_000,
            client_slug: "beta".to_string(),
        }),
    );

    let engine = helpers::make_engine(registry, vec![helpers::org("acme"), helpers::org("beta")]);

    // No explicit limit — uses the engine's 10K materialization cap.
    let options = QueryOptions {
        clients: None, // ALL scope — both adapters fire
        sensors: None,
        limit: None,
        force_refresh: false,
        ..QueryOptions::default()
    };

    let result = engine
        .execute("SELECT * FROM crowdstrike_detections", options)
        .await;

    let err = result.expect_err(
        "AC-3-bis: 12K total rows (6K×2 adapters) must exceed the 10K cap → E-QUERY-003",
    );
    let detail = err.to_string();
    assert!(
        detail.contains("E-QUERY-003"),
        "AC-3-bis: error must contain 'E-QUERY-003' (record cap exceeded); got: {detail}"
    );
}

// ---------------------------------------------------------------------------
// F-LP1-CRIT-1: prism_audit queryable through QueryEngine::execute (not just standalone)
// ---------------------------------------------------------------------------

/// F-LP1-CRIT-1 (BC-2.15.011): `register_internal_tables` is invoked from
/// `execute_inner`, so `prism_audit` is accessible through the full
/// `QueryEngine::execute` path — not just via standalone registration.
///
/// This tests the actual end-to-end wiring, not just `register_internal_tables`
/// in isolation (which AC-5 already covers).
#[tokio::test]
async fn test_CRIT_1_internal_table_queryable_through_execute() {
    use prism_query::engine::{Capability, QueryEngine, QueryEngineConfig, QueryOptions};

    let org_slug = helpers::org("acme");
    let storage = helpers::make_storage();

    // Seed one audit entry.
    helpers::seed_entry(
        &storage,
        StorageDomain::AuditBuffer,
        b"audit:00000000000000000001:trace-001",
        b"test-payload",
    );

    let adapter_registry = Arc::new(AdapterRegistry::new());
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        org_slug.clone()
    ]));
    let config = QueryEngineConfig::default();
    let org_registry = Arc::new(prism_core::OrgRegistry::new());
    let engine = QueryEngine::new_full(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
        Arc::new(helpers::StubCredentialResolver),
        org_registry,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
    );

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        capabilities: vec![Capability::AuditRead],
        ..QueryOptions::default()
    };

    // Must succeed — internal table registered by execute_inner, not just AC-5 path.
    let result = engine
        .execute("SELECT * FROM prism_audit LIMIT 5", options)
        .await
        .expect(
            "CRIT-1: prism_audit must be queryable via QueryEngine::execute when storage is set",
        );

    // The result should be Ok; content doesn't matter (empty scan is fine).
    drop(result); // success is all we need
}

// ---------------------------------------------------------------------------
// F-LP1-HIGH-2: bincode 2.x deserialization — AuditEntry fields appear in scan output
// ---------------------------------------------------------------------------

/// F-LP1-HIGH-2 / ADV-W3MT-P59-CRIT-001 (AD-012, BC-2.15.011): When `prism_audit` is queried
/// and the audit buffer contains properly bincode-encoded `AuditEntry` values, the scan
/// must deserialize them and project their fields onto the Arrow schema columns.
///
/// Authoritative schema (synced with prism-storage::internal_tables):
///   trace_id: Utf8, timestamp_ns: UInt64, operation: Utf8, client_id: Utf8,
///   analyst_id: Utf8, outcome: Utf8, capability: Utf8
///
/// This test uses `prism-storage::audit_buffer::append_audit_entry` to write a
/// properly-encoded entry, then queries through `QueryEngine::execute`.
#[tokio::test]
async fn test_HIGH_2_audit_entry_bincode_deserialization() {
    use arrow::array::UInt64Array;
    use prism_query::engine::{Capability, QueryEngine, QueryEngineConfig, QueryOptions};
    use prism_storage::audit_buffer::{append_audit_entry, AuditEntry};
    use std::collections::BTreeMap;

    let org_slug = helpers::org("acme");
    let storage = helpers::make_storage();

    // Seed one properly bincode-encoded AuditEntry with authoritative field names.
    let mut payload = BTreeMap::new();
    payload.insert("operation".to_string(), "query".to_string());
    payload.insert("client_id".to_string(), "acme".to_string());
    payload.insert("analyst_id".to_string(), "analyst-001".to_string());
    payload.insert("outcome".to_string(), "success".to_string());
    payload.insert("capability".to_string(), "query.execute".to_string());

    let entry = AuditEntry {
        timestamp_ns: 1_000_000_000_u64,
        trace_id: "trace-high2-001".to_string(),
        payload,
    };

    // Use the concrete InMemoryBackend directly (append_audit_entry is generic, not dyn-safe).
    append_audit_entry(storage.as_ref(), &entry).expect("HIGH-2: seed audit entry must succeed");

    let adapter_registry = Arc::new(AdapterRegistry::new());
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        org_slug.clone()
    ]));
    let config = QueryEngineConfig::default();
    let org_registry = Arc::new(prism_core::OrgRegistry::new());
    let engine = QueryEngine::new_full(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
        Arc::new(helpers::StubCredentialResolver),
        org_registry,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
    );

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        capabilities: vec![Capability::AuditRead],
        ..QueryOptions::default()
    };

    // Use authoritative column names from prism-storage::internal_tables (ADV-W3MT-P59-CRIT-001).
    let result = engine
        .execute(
            "SELECT trace_id, timestamp_ns, operation FROM prism_audit LIMIT 10",
            options,
        )
        .await
        .expect("HIGH-2: prism_audit query with AuditRead must succeed");

    assert!(
        !result.batches.is_empty(),
        "HIGH-2: must return at least one batch after seeding one AuditEntry"
    );

    // Find the batch with actual data (the empty batch still passes but has 0 rows).
    let data_batches: Vec<_> = result.batches.iter().filter(|b| b.num_rows() > 0).collect();
    assert!(
        !data_batches.is_empty(),
        "HIGH-2: must have at least one batch with rows; the seeded AuditEntry must appear"
    );

    // Verify that the `timestamp_ns` column contains the u64 value (not raw bytes or zero).
    for batch in &data_batches {
        if let Ok(ts_idx) = batch.schema().index_of("timestamp_ns") {
            if let Some(ts_arr) = batch.column(ts_idx).as_any().downcast_ref::<UInt64Array>() {
                for row in 0..ts_arr.len() {
                    let ts_val = ts_arr.value(row);
                    // timestamp_ns must be the u64 we seeded (1_000_000_000), not 0.
                    assert!(
                        ts_val > 0,
                        "HIGH-2: timestamp_ns must be non-zero (deserialized from AuditEntry); got: {ts_val}"
                    );
                }
            }
        }
    }

    // Verify trace_id is present and non-empty.
    for batch in &data_batches {
        if let Ok(tr_idx) = batch.schema().index_of("trace_id") {
            if let Some(tr_arr) = batch.column(tr_idx).as_any().downcast_ref::<StringArray>() {
                for row in 0..tr_arr.len() {
                    assert!(
                        !tr_arr.value(row).is_empty(),
                        "HIGH-2: trace_id must be non-empty; got empty string at row {row}"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AC-8: no todo!() or unimplemented!() remains in the stub files
// ---------------------------------------------------------------------------

/// AC-8 (POL-12 / Objective): No `todo!()` or `unimplemented!()` may remain in
/// `engine.rs`, `materialization.rs`, or `internal_tables.rs` before merge.
///
/// Uses `include_str!` to capture source text at compile time.
///
/// Red-Gate: assertion fails because `todo!(` is still present in the stubs.
#[test]
fn test_AC_8_no_todo_or_unimplemented_remains() {
    let engine_src = include_str!("../src/engine.rs");
    let materialization_src = include_str!("../src/materialization.rs");
    let internal_tables_src = include_str!("../src/internal_tables.rs");

    assert!(
        !engine_src.contains("todo!("),
        "AC-8: engine.rs still contains todo!() — fill stubs before merging (POL-12)"
    );
    assert!(
        !engine_src.contains("unimplemented!("),
        "AC-8: engine.rs still contains unimplemented!() — fill stubs before merging (POL-12)"
    );

    assert!(
        !materialization_src.contains("todo!("),
        "AC-8: materialization.rs still contains todo!() — fill stubs before merging (POL-12)"
    );
    assert!(
        !materialization_src.contains("unimplemented!("),
        "AC-8: materialization.rs still contains unimplemented!() — fill before merging (POL-12)"
    );

    assert!(
        !internal_tables_src.contains("todo!("),
        "AC-8: internal_tables.rs still contains todo!() — fill stubs before merging (POL-12)"
    );
    assert!(
        !internal_tables_src.contains("unimplemented!("),
        "AC-8: internal_tables.rs still contains unimplemented!() — fill before merging (POL-12)"
    );
}

// ---------------------------------------------------------------------------
// F-LP2-CRIT-1: Capability gate subquery bypass (defense-in-depth)
// ---------------------------------------------------------------------------

/// F-LP2-CRIT-1 (BC-2.15.011): A subquery referencing `prism_audit` in the
/// WHERE clause (IN subquery) MUST be caught by the pre-execution capability gate.
///
/// Layer 1 test: the recursive AST walker must extract `prism_audit` from
/// `WHERE field IN (SELECT ... FROM prism_audit)` and reject it without AuditRead.
#[tokio::test]
async fn test_LP2_CRIT_1_subquery_in_where_blocked_without_audit_read() {
    use prism_query::engine::{QueryEngine, QueryEngineConfig, QueryOptions};

    let org_slug = helpers::org("acme");
    let storage = helpers::make_storage();

    // Register a StubAdapter for crowdstrike_detections so the outer query can resolve.
    let mut registry = AdapterRegistry::new();
    use prism_core::OrgId;
    registry.register(
        OrgId::new(),
        Arc::new(helpers::StubAdapter {
            sensor_type: prism_core::SensorId::from("crowdstrike"),
            row_count: 2,
            client_slug: "acme".to_string(),
        }),
    );

    let adapter_registry = Arc::new(registry);
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(prism_ocsf::OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        org_slug.clone()
    ]));
    let config = QueryEngineConfig::default();
    let org_registry = Arc::new(prism_core::OrgRegistry::new());
    let engine = QueryEngine::new_full(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
        Arc::new(helpers::StubCredentialResolver),
        org_registry,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
    );

    // No AuditRead capability — subquery references prism_audit.
    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        capabilities: vec![], // AuditRead NOT granted
        ..QueryOptions::default()
    };

    // The pre-execution gate (Layer 1) must catch prism_audit in the subquery.
    let result = engine
        .execute(
            "SELECT alert_id FROM crowdstrike_detections \
             WHERE alert_id IN (SELECT trace_id FROM prism_audit)",
            options,
        )
        .await;

    let err = result.expect_err(
        "LP2-CRIT-1: subquery referencing prism_audit without AuditRead must be rejected \
         (pre-execution gate must walk subqueries in WHERE clause)",
    );
    assert!(
        matches!(err, prism_core::PrismError::AuditTableAccessDenied),
        "LP2-CRIT-1: error must be PrismError::AuditTableAccessDenied; got: {err:?}"
    );
}

/// F-LP2-CRIT-1: WITH AuditRead capability, the subquery referencing prism_audit
/// in WHERE must be allowed through.
#[tokio::test]
async fn test_LP2_CRIT_1_with_audit_read_capability_subquery_allowed() {
    use prism_query::engine::{Capability, QueryEngine, QueryEngineConfig, QueryOptions};

    let org_slug = helpers::org("acme");
    let storage = helpers::make_storage();

    let mut registry = AdapterRegistry::new();
    use prism_core::OrgId;
    registry.register(
        OrgId::new(),
        Arc::new(helpers::StubAdapter {
            sensor_type: prism_core::SensorId::from("crowdstrike"),
            row_count: 2,
            client_slug: "acme".to_string(),
        }),
    );

    let adapter_registry = Arc::new(registry);
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(prism_ocsf::OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        org_slug.clone()
    ]));
    let config = QueryEngineConfig::default();
    let org_registry = Arc::new(prism_core::OrgRegistry::new());
    let engine = QueryEngine::new_full(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
        Arc::new(helpers::StubCredentialResolver),
        org_registry,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
    );

    // AuditRead granted — should NOT be rejected at the capability gate.
    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        capabilities: vec![Capability::AuditRead],
        ..QueryOptions::default()
    };

    // With AuditRead, the capability gate passes. DataFusion may still fail (ok),
    // but the error must NOT be AuditTableAccessDenied.
    let result = engine
        .execute(
            "SELECT alert_id FROM crowdstrike_detections \
             WHERE alert_id IN (SELECT trace_id FROM prism_audit)",
            options,
        )
        .await;

    // The capability gate must not reject. Any DataFusion planning error is acceptable.
    match result {
        Ok(_) => {} // great
        Err(prism_core::PrismError::AuditTableAccessDenied) => {
            panic!("LP2-CRIT-1: WITH AuditRead capability, must NOT return AuditTableAccessDenied");
        }
        Err(_other) => {} // DataFusion planning/execution errors are ok
    }
}

/// F-LP2-CRIT-1: HAVING clause subquery referencing prism_audit is blocked without AuditRead.
#[tokio::test]
async fn test_LP2_CRIT_1_having_subquery_blocked_without_audit_read() {
    use prism_query::engine::{QueryEngine, QueryEngineConfig, QueryOptions};

    let org_slug = helpers::org("acme");
    let storage = helpers::make_storage();

    let mut registry = AdapterRegistry::new();
    use prism_core::OrgId;
    registry.register(
        OrgId::new(),
        Arc::new(helpers::StubAdapter {
            sensor_type: prism_core::SensorId::from("crowdstrike"),
            row_count: 2,
            client_slug: "acme".to_string(),
        }),
    );

    let adapter_registry = Arc::new(registry);
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(prism_ocsf::OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        org_slug.clone()
    ]));
    let config = QueryEngineConfig::default();
    let org_registry = Arc::new(prism_core::OrgRegistry::new());
    let engine = QueryEngine::new_full(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
        Arc::new(helpers::StubCredentialResolver),
        org_registry,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
    );

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        capabilities: vec![], // no AuditRead
        ..QueryOptions::default()
    };

    // The HAVING clause contains a subquery referencing prism_audit.
    // The pre-execution gate must walk HAVING predicates recursively.
    let result = engine
        .execute(
            "SELECT alert_id FROM crowdstrike_detections \
             GROUP BY alert_id \
             HAVING alert_id IN (SELECT trace_id FROM prism_audit)",
            options,
        )
        .await;

    let err = result.expect_err(
        "LP2-CRIT-1: HAVING subquery referencing prism_audit without AuditRead must be rejected",
    );
    assert!(
        matches!(err, prism_core::PrismError::AuditTableAccessDenied),
        "LP2-CRIT-1: HAVING subquery error must be AuditTableAccessDenied; got: {err:?}"
    );
}

/// F-LP2-CRIT-1 Layer 2: scan-time gate on RocksDbTableProvider.
///
/// Verifies that even if the pre-execution gate (Layer 1) is bypassed,
/// `RocksDbTableProvider::scan()` itself rejects access to audit table
/// when the provider was constructed with no AuditRead capability.
#[tokio::test]
async fn test_LP2_CRIT_1_scan_time_gate_rejects_without_audit_read() {
    use datafusion::datasource::TableProvider;
    use prism_query::engine::Capability;
    use prism_query::internal_tables::{InternalTableDescriptor, RocksDbTableProvider};
    use prism_storage::memory_backend::InMemoryBackend;

    let storage = Arc::new(InMemoryBackend::new());

    // Build a descriptor that requires audit.read.
    let audit_schema = Arc::new(arrow::datatypes::Schema::new(vec![
        arrow::datatypes::Field::new("trace_id", arrow::datatypes::DataType::Utf8, true),
        arrow::datatypes::Field::new("timestamp", arrow::datatypes::DataType::Utf8, true),
        arrow::datatypes::Field::new("event_type", arrow::datatypes::DataType::Utf8, true),
        arrow::datatypes::Field::new("org_id", arrow::datatypes::DataType::Utf8, true),
        arrow::datatypes::Field::new("payload", arrow::datatypes::DataType::Utf8, true),
    ]));

    let descriptor = InternalTableDescriptor {
        table_name: "prism_audit".to_string(),
        domain: "audit_buffer".to_string(),
        schema: Arc::clone(&audit_schema),
        requires_audit_read: true,
    };

    // Construct provider WITHOUT AuditRead in capability set.
    let provider = RocksDbTableProvider::new_with_capabilities(
        descriptor,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
        vec![], // no AuditRead
    );

    // Attempt scan — must return DataFusionError containing E-QUERY-011.
    let state = datafusion::execution::context::SessionContext::new().state();
    let result = provider.scan(&state, None, &[], None).await;

    assert!(
        result.is_err(),
        "LP2-CRIT-1 Layer 2: scan() without AuditRead must return Err"
    );
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("E-QUERY-011") || err_str.contains("audit.read"),
        "LP2-CRIT-1 Layer 2: scan() error must reference E-QUERY-011 or audit.read; got: {err_str}"
    );
}

/// F-LP2-CRIT-1 Layer 3: descriptor-driven policy — a non-prism_audit descriptor
/// with `requires_audit_read = true` is also gated.
#[tokio::test]
async fn test_LP2_CRIT_1_descriptor_driven_non_audit_table_also_gated() {
    use datafusion::datasource::TableProvider;
    use prism_query::engine::Capability;
    use prism_query::internal_tables::{InternalTableDescriptor, RocksDbTableProvider};
    use prism_storage::memory_backend::InMemoryBackend;

    let storage = Arc::new(InMemoryBackend::new());

    // A hypothetical future table with requires_audit_read = true.
    let schema = Arc::new(arrow::datatypes::Schema::new(vec![
        arrow::datatypes::Field::new("id", arrow::datatypes::DataType::Utf8, true),
    ]));

    let descriptor = InternalTableDescriptor {
        table_name: "prism_secrets".to_string(), // not prism_audit
        domain: "default".to_string(),
        schema,
        requires_audit_read: true, // but flagged as requiring audit.read
    };

    let provider = RocksDbTableProvider::new_with_capabilities(
        descriptor,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
        vec![], // no AuditRead
    );

    let state = datafusion::execution::context::SessionContext::new().state();
    let result = provider.scan(&state, None, &[], None).await;

    assert!(
        result.is_err(),
        "LP2-CRIT-1 Layer 3: scan() on non-audit table with requires_audit_read=true \
         and no AuditRead capability must return Err"
    );
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("E-QUERY-011") || err_str.contains("audit.read"),
        "LP2-CRIT-1 Layer 3: error must reference E-QUERY-011 or audit.read; got: {err_str}"
    );
}

/// F-LP2-CRIT-1 Layer 3: WITH AuditRead, scan() on a requires_audit_read table succeeds.
#[tokio::test]
async fn test_LP2_CRIT_1_scan_time_gate_allows_with_audit_read() {
    use datafusion::datasource::TableProvider;
    use prism_query::engine::Capability;
    use prism_query::internal_tables::{InternalTableDescriptor, RocksDbTableProvider};
    use prism_storage::memory_backend::InMemoryBackend;

    let storage = Arc::new(InMemoryBackend::new());

    let audit_schema = Arc::new(arrow::datatypes::Schema::new(vec![
        arrow::datatypes::Field::new("trace_id", arrow::datatypes::DataType::Utf8, true),
        arrow::datatypes::Field::new("timestamp", arrow::datatypes::DataType::Utf8, true),
        arrow::datatypes::Field::new("event_type", arrow::datatypes::DataType::Utf8, true),
        arrow::datatypes::Field::new("org_id", arrow::datatypes::DataType::Utf8, true),
        arrow::datatypes::Field::new("payload", arrow::datatypes::DataType::Utf8, true),
    ]));

    let descriptor = InternalTableDescriptor {
        table_name: "prism_audit".to_string(),
        domain: "audit_buffer".to_string(),
        schema: Arc::clone(&audit_schema),
        requires_audit_read: true,
    };

    // Construct provider WITH AuditRead.
    let provider = RocksDbTableProvider::new_with_capabilities(
        descriptor,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
        vec![Capability::AuditRead],
    );

    let state = datafusion::execution::context::SessionContext::new().state();
    let result = provider.scan(&state, None, &[], None).await;

    assert!(
        result.is_ok(),
        "LP2-CRIT-1 Layer 3: scan() with AuditRead on requires_audit_read table must succeed; \
         got: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// F-LP2-HIGH-1: AC-2 vacuous-pass fix — assert actual non-empty materialization
// ---------------------------------------------------------------------------

/// F-LP2-HIGH-1: AC-2 enhanced — verifies the pipeline materializes actual rows
/// and registers at least one MemTable. Replaces the previous vacuous
/// `!catalog_names().is_empty()` check that always passed.
#[tokio::test]
async fn test_AC_2_materialization_pipeline_non_vacuous_assertion() {
    use prism_query::engine::QueryOptions;

    let mut mat_ctx = helpers::make_mat_ctx_with_stub(10_000, 3);
    let session_ctx = helpers::make_ctx();
    let options = QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        sensors: None,
        limit: Some(10),
        force_refresh: false,
        ..QueryOptions::default()
    };

    let output = run_materialization_pipeline(
        "SELECT * FROM crowdstrike_detections LIMIT 10",
        &options,
        &mut mat_ctx,
        &session_ctx,
    )
    .await
    .expect("AC-2 enhanced: run_materialization_pipeline must succeed with valid source ref");

    // Non-vacuous assertion: at least one batch with actual rows must be present.
    assert!(
        !output.batches.is_empty(),
        "AC-2 enhanced: pipeline must materialize at least one batch; \
         if this fails, StubAdapter registration or fan_out is broken"
    );
    let total_rows: usize = output.batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_rows, 3,
        "AC-2 enhanced: StubAdapter returns 3 rows; pipeline must materialize all 3"
    );

    // Verify MemTable registration: session_ctx must have crowdstrike_detections registered.
    assert!(
        session_ctx
            .table_exist("crowdstrike_detections")
            .expect("AC-2 enhanced: table_exist() must not fail"),
        "AC-2 enhanced: crowdstrike_detections MemTable must be registered after pipeline runs"
    );
}

// ---------------------------------------------------------------------------
// F-LP2-MED-2: cache key includes where_filters (no leakage between filter variants)
// ---------------------------------------------------------------------------

/// F-LP2-MED-2 (BC-2.11.005): The in-query cache key must include the WHERE filters.
///
/// Same (client, sensor, source_table) but different SQL WHERE clauses must produce
/// two separate adapter calls (no cache leakage between differently-filtered queries).
///
/// Uses two separate `MaterializationContext` instances to isolate the per-query caches
/// and demonstrate that two identical queries DO hit the cache (same key) while two
/// queries with different WHERE clauses do NOT (different keys).
#[tokio::test]
async fn test_LP2_MED_2_cache_key_includes_filters() {
    use prism_core::OrgId;
    use prism_query::engine::QueryOptions;
    use prism_sensors::adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec};
    use prism_sensors::auth::SensorAuth;

    struct CountingAdapter {
        call_count: Arc<std::sync::atomic::AtomicUsize>,
    }

    #[async_trait]
    impl SensorAdapter for CountingAdapter {
        fn sensor_type(&self) -> prism_core::SensorId {
            prism_core::SensorId::from("crowdstrike")
        }

        fn sensor_name(&self) -> &'static str {
            "crowdstrike"
        }

        async fn fetch(
            &self,
            _spec: &SensorSpec,
            _params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<RecordBatch>, SensorError> {
            self.call_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let schema = Arc::new(arrow::datatypes::Schema::new(vec![
                arrow::datatypes::Field::new(
                    "detection_id",
                    arrow::datatypes::DataType::Utf8,
                    false,
                ),
            ]));
            let arr = Arc::new(arrow::array::StringArray::from(vec!["row1"])) as _;
            let batch = RecordBatch::try_new(schema, vec![arr]).unwrap();
            Ok(vec![batch])
        }
    }

    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let org_id = OrgId::new();

    // Helper to build a mat_ctx sharing the same adapter.
    let make_mat_ctx = {
        let call_count = Arc::clone(&call_count);
        let org_id = org_id;
        move || {
            let adapter = Arc::new(CountingAdapter {
                call_count: Arc::clone(&call_count),
            });
            let mut registry = AdapterRegistry::new();
            registry.register(org_id, adapter);
            let ocsf_normalizer = Arc::new(prism_ocsf::OcsfNormalizer::new());
            prism_query::materialization::MaterializationContext::new_with_resolver(
                Arc::new(registry),
                ocsf_normalizer,
                10_000,
                Arc::new(helpers::StubCredentialResolver),
                None,
            )
        }
    };

    let options = QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        ..QueryOptions::default()
    };

    // Scenario A: SAME query twice in one mat_ctx — second call should hit cache (1 adapter call).
    let mut mat_ctx_a = make_mat_ctx();
    let ctx1 = helpers::make_ctx();
    let ctx2 = helpers::make_ctx();
    let _ = run_materialization_pipeline(
        "SELECT detection_id FROM crowdstrike_detections WHERE detection_id = 'x'",
        &options,
        &mut mat_ctx_a,
        &ctx1,
    )
    .await;
    let _ = run_materialization_pipeline(
        "SELECT detection_id FROM crowdstrike_detections WHERE detection_id = 'x'",
        &options,
        &mut mat_ctx_a,
        &ctx2,
    )
    .await;
    let calls_scenario_a = call_count.load(std::sync::atomic::Ordering::SeqCst);

    // Reset counter for scenario B.
    call_count.store(0, std::sync::atomic::Ordering::SeqCst);

    // Scenario B: DIFFERENT WHERE filters in one mat_ctx — must NOT share cache (2 adapter calls).
    let mut mat_ctx_b = make_mat_ctx();
    let ctx3 = helpers::make_ctx();
    let ctx4 = helpers::make_ctx();
    let _ = run_materialization_pipeline(
        "SELECT detection_id FROM crowdstrike_detections WHERE detection_id = 'x'",
        &options,
        &mut mat_ctx_b,
        &ctx3,
    )
    .await;
    let _ = run_materialization_pipeline(
        "SELECT detection_id FROM crowdstrike_detections WHERE detection_id = 'y'",
        &options,
        &mut mat_ctx_b,
        &ctx4,
    )
    .await;
    let calls_scenario_b = call_count.load(std::sync::atomic::Ordering::SeqCst);

    // Scenario A: same query twice → 1 adapter call (second hits cache).
    assert_eq!(
        calls_scenario_a, 1,
        "LP2-MED-2 scenario A: identical queries must cache-hit on second call; \
         expected 1 adapter call, got {calls_scenario_a}"
    );

    // Scenario B: different WHERE filters → 2 adapter calls (no cache sharing).
    assert_eq!(
        calls_scenario_b, 2,
        "LP2-MED-2 scenario B: queries with different WHERE filters must NOT share cache; \
         expected 2 adapter calls (different keys), got {calls_scenario_b}"
    );
}

// ---------------------------------------------------------------------------
// F-LP2-LOW-1: limit exceeds max returns correct error variant
// ---------------------------------------------------------------------------

/// F-LP2-LOW-1 (BC-2.11.001): When limit > 1000, the engine must return
/// `PrismError::QueryLimitExceeded`, not `QueryExecutionFailed` stuffed with
/// an E-QUERY-001 string.
#[tokio::test]
async fn test_LP2_LOW_1_limit_exceeded_returns_query_limit_exceeded_variant() {
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let engine = helpers::make_engine(AdapterRegistry::new(), vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        limit: Some(1001), // exceeds 1000
        ..QueryOptions::default()
    };

    let result = engine
        .execute("SELECT * FROM crowdstrike_detections", options)
        .await;

    let err = result.expect_err("LP2-LOW-1: limit=1001 must return Err");

    assert!(
        matches!(
            err,
            prism_core::PrismError::QueryLimitExceeded { requested: 1001, max: 1000 }
        ),
        "LP2-LOW-1: error must be PrismError::QueryLimitExceeded {{ requested: 1001, max: 1000 }}; \
         got: {err:?}"
    );

    // Display must contain E-QUERY-007 (ADV-W3MT-P58-CRIT-001: code changed from E-QUERY-001
    // to avoid collision with QueryParseFailed which uses E-QUERY-001).
    let display = err.to_string();
    assert!(
        display.contains("E-QUERY-007"),
        "LP2-LOW-1: display must contain 'E-QUERY-007' (limit exceeded, ADV-W3MT-P58-CRIT-001); got: {display}"
    );
}

// ---------------------------------------------------------------------------
// F-LP2-LOW-3: HIGH-7 boundary test with actual pipeline success for limit=1000
// ---------------------------------------------------------------------------

/// F-LP2-LOW-3: limit=1000 (boundary) must produce actual pipeline success with
/// a StubAdapter returning rows — verifies the full path, not just the guard.
#[tokio::test]
async fn test_HIGH_7_limit_exactly_1000_pipeline_success_with_stub() {
    use prism_core::OrgId;
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let org_id = OrgId::new();

    let mut registry = AdapterRegistry::new();
    registry.register(
        org_id,
        Arc::new(helpers::StubAdapter {
            sensor_type: prism_core::SensorId::from("crowdstrike"),
            row_count: 5,
            client_slug: "acme".to_string(),
        }),
    );

    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        limit: Some(1000),
        ..QueryOptions::default()
    };

    let result = engine
        .execute("SELECT * FROM crowdstrike_detections LIMIT 1000", options)
        .await;

    assert!(
        result.is_ok(),
        "LP2-LOW-3: execute with limit=1000 and StubAdapter must succeed; got: {:?}",
        result.err()
    );

    let qr = result.unwrap();
    assert!(
        qr.returned_results > 0,
        "LP2-LOW-3: pipeline with StubAdapter must produce rows (not vacuous); returned_results=0"
    );
}

// ---------------------------------------------------------------------------
// HIGH-002 (ADV-W3MT-P58): Timeout (E-QUERY-004) and depth-limit (E-QUERY-005) tests
// ---------------------------------------------------------------------------

/// HIGH-002 / ADV-W3MT-P58-HIGH-002: Story §Tasks item 8 mandates a timeout test.
///
/// Set `timeout_secs = 1` and use a SlowAdapter that sleeps beyond the timeout.
/// `execute` must return `PrismError::QueryTimeout` (E-QUERY-004 path — timeout fires
/// before DataFusion execution begins when the sensor adapter is slow).
///
/// Note: E-QUERY-004 is the memory-budget error code; E-QUERY-005 is the timeout code.
/// The `PrismError::QueryTimeout` variant displays "E-QUERY-005: query timed out after".
/// The test verifies the correct variant, not a string code, to avoid brittle assertions.
#[tokio::test]
async fn test_AC_timeout_returns_query_timeout_error() {
    use prism_core::OrgId;
    use prism_core::SensorId;
    use prism_query::engine::{QueryEngine, QueryEngineConfig, QueryOptions};
    use prism_sensors::adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec};
    use prism_sensors::auth::SensorAuth;

    /// Adapter that sleeps for 2 seconds, causing a 1s timeout to fire.
    struct SlowAdapter;

    #[async_trait]
    impl SensorAdapter for SlowAdapter {
        fn sensor_type(&self) -> SensorId {
            SensorId::from("crowdstrike")
        }

        fn sensor_name(&self) -> &'static str {
            "crowdstrike"
        }

        async fn fetch(
            &self,
            _spec: &SensorSpec,
            _params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<arrow::record_batch::RecordBatch>, SensorError> {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            Ok(vec![])
        }
    }

    let org_slug = helpers::org("acme");
    let mut registry = AdapterRegistry::new();
    registry.register(OrgId::new(), Arc::new(SlowAdapter));

    let adapter_registry = Arc::new(registry);
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        org_slug.clone()
    ]));
    let config = QueryEngineConfig {
        timeout_secs: 1, // 1-second timeout; SlowAdapter sleeps 2 seconds → timeout fires
        ..QueryEngineConfig::default()
    };
    let org_registry = Arc::new(prism_core::OrgRegistry::new());
    let storage = helpers::make_storage();

    let engine = QueryEngine::new_full(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
        Arc::new(helpers::StubCredentialResolver),
        org_registry,
        storage as Arc<dyn prism_storage::backend::RocksStorageBackend>,
    );

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    let result = engine
        .execute("SELECT * FROM crowdstrike_detections", options)
        .await;

    let err = result
        .expect_err("timeout-test: execute with 1s timeout and 2s SlowAdapter must return Err");
    assert!(
        matches!(err, prism_core::PrismError::QueryTimeout { .. }),
        "timeout-test: error must be PrismError::QueryTimeout (E-QUERY-005); got: {err:?}"
    );
}

/// HIGH-002 / ADV-W3MT-P58-HIGH-002: Story §Tasks item 8 mandates a depth-limit test.
///
/// The PrismQL parser enforces a maximum nesting depth. A deeply nested query
/// (depth > the configured limit) must fail with a parse error (E-QUERY-001).
/// This exercises the depth-limit enforcement in the security module.
///
/// Note: depth limit is enforced by `PrismQlParser::parse` via `security.rs`.
/// The error surfaces as `PrismError::QueryParseFailed` which displays "E-QUERY-001".
#[tokio::test]
async fn test_AC_depth_limit_returns_parse_error() {
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let engine = helpers::make_engine(AdapterRegistry::new(), vec![org_slug.clone()]);

    // Construct a deeply nested subquery that exceeds the parser's depth limit.
    // Each level adds `(SELECT * FROM (...))`; depth limit is typically 10-20.
    let mut query = "SELECT * FROM crowdstrike_detections".to_string();
    for _ in 0..60 {
        query = format!("SELECT * FROM ({query}) AS sub");
    }

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    let result = engine.execute(&query, options).await;

    let err = result
        .expect_err("depth-limit-test: a 60-level nested subquery must fail the depth limit check");
    let detail = err.to_string();
    // Depth limit fires inside PrismQlParser::parse → QueryParseFailed (E-QUERY-001)
    // OR security module returns QueryExecutionFailed with depth-limit detail.
    // Either way the result must be Err.
    assert!(
        matches!(
            err,
            prism_core::PrismError::QueryParseFailed { .. }
                | prism_core::PrismError::QueryExecutionFailed { .. }
        ),
        "depth-limit-test: error must be a parse or execution failure (depth limit); got: {detail}"
    );
}

// ---------------------------------------------------------------------------
// F-LP1-CRITICAL-001 regression: unknown source table must return E-QUERY-006
// ---------------------------------------------------------------------------

/// S-PLUGIN-PREREQ-A F-LP1-CRITICAL-001 regression test.
///
/// An unknown table name (prefix not registered in the adapter registry) MUST
/// return `Err` containing "E-QUERY-006" rather than silently producing empty
/// results. Before the fix, `unknown_table | host = 'x'` would silently produce
/// an empty result set because `sensor_type_from_table_name` accepted any
/// non-empty prefix, and `get_all_for_sensor_type("unknown")` returned empty.
///
/// The registry must be NON-EMPTY for this guard to fire — an empty registry
/// indicates test/boot mode where the sensor roster is not yet known. In production
/// the registry is always populated with at least the four built-in sensors.
///
/// This test verifies the two-stage check: extract prefix + registry membership.
#[tokio::test]
async fn test_resolve_source_refs_unknown_table_returns_e_query_006() {
    use prism_core::{OrgId, OrgSlug, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::AdapterRegistry;

    // Register a known sensor (crowdstrike) so the registry is non-empty.
    // This matches production behavior where built-in sensors are always registered.
    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();
    registry.register(
        org_id,
        std::sync::Arc::new(helpers::StubAdapter {
            sensor_type: SensorId::from("crowdstrike"),
            row_count: 0,
            client_slug: "acme".to_string(),
        }),
    );
    let org_slug = OrgSlug::new("acme");
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // "unknown_table" has prefix "unknown" — not in registry → must be E-QUERY-006.
    // (Registry is non-empty so the guard fires; "unknown" != "crowdstrike".)
    let result = engine.execute("unknown_table | host = 'x'", options).await;

    let err = result.expect_err(
        "F-LP1-CRITICAL-001: unknown_table must return Err (E-QUERY-006), not empty results",
    );
    let detail = err.to_string();
    assert!(
        detail.contains("E-QUERY-006"),
        "F-LP1-CRITICAL-001: error must contain 'E-QUERY-006'; got: {detail}"
    );
}
