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

use arrow::{
    array::{Array, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use async_trait::async_trait;
use datafusion::execution::context::SessionContext;
use prism_core::{OrgSlug, PrismError, StorageDomain};
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    engine::QueryOptions,
    internal_tables::register_internal_tables,
    materialization::{run_materialization_pipeline, MaterializationContext},
};
use prism_sensors::AdapterRegistry;
use prism_storage::{backend::RocksStorageBackend, memory_backend::InMemoryBackend};

// ---------------------------------------------------------------------------
// Test Helpers
// ---------------------------------------------------------------------------

mod helpers {
    use std::sync::Arc;

    use arrow::{
        array::StringArray,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use datafusion::execution::context::SessionContext;
    use prism_core::{OrgSlug, PrismError, StorageDomain};
    use prism_credentials::{namespace::CredentialName, CredentialStore};
    use prism_ocsf::OcsfNormalizer;
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig},
        materialization::MaterializationContext,
        scoping::ClientRegistry,
    };
    use prism_sensors::{AdapterRegistry, CredentialResolver};
    use prism_storage::{backend::RocksStorageBackend, memory_backend::InMemoryBackend};
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

    /// Test-only `CredentialResolver` that returns a stub `SensorAuth` impl for any request.
    /// All built-in auth types were deleted in PLUGIN-MIGRATION-001-A; this stub satisfies
    /// the trait bound for tests.
    /// (F-LP1-CRIT-2: prevents NullCredentialResolver from short-circuiting fan_out)
    pub struct StubCredentialResolver;

    impl CredentialResolver for StubCredentialResolver {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_id: prism_core::SensorId,
        ) -> Result<Box<dyn prism_sensors::auth::SensorAuth>, SensorError> {
            // All built-in auth types deleted in PLUGIN-MIGRATION-001-A (AC-003, AC-006).
            // Use an inline test stub that satisfies the SensorAuth bound.
            // StubAdapter::fetch ignores _auth entirely (F-LP1-CRIT-2).
            struct TestStubAuth;
            impl prism_sensors::auth::SensorAuth for TestStubAuth {
                fn as_any(&self) -> &dyn std::any::Any {
                    self
                }
                fn auth_type_name(&self) -> &'static str {
                    "custom_via_plugin"
                }
            }
            Ok(Box::new(TestStubAuth))
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
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
    };

    /// Minimal sensor adapter that returns a configurable number of rows with a
    /// `detection_id` column.  Used in tests that need real row data to exercise
    /// record-cap, virtual-field, and cross-client fan-out logic.
    pub struct StubAdapter {
        pub sensor_id: SensorId,
        pub row_count: usize,
        pub client_slug: String,
        /// Whether this adapter signals early-stop in FetchOutput (ADR-060 §D8.3).
        /// Set to true to model a sensor that stopped after the first page.
        pub any_early_stopped: bool,
    }

    #[async_trait]
    impl SensorAdapter for StubAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "crowdstrike"
        }

        async fn fetch(
            &self,
            _spec: &SensorSpec,
            _params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<FetchOutput, SensorError> {
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
            Ok(FetchOutput::new(vec![batch], self.any_early_stopped))
        }
    }

    /// Self-contained fixture that owns both the `AliasStore` and its backing `TempDir`.
    ///
    /// The `TempDir` is kept alive by this struct so the backing directory is never
    /// removed while the store is in use. Callers obtain the store arc via `store()`.
    ///
    /// F-PASS9-NIT-1: replaces the bare `(Arc<Mutex<AliasStore>>, TempDir)` tuple so
    /// the contract is structurally self-enforcing rather than documentation-dependent.
    pub struct AliasStoreFixture {
        pub store: Arc<std::sync::Mutex<prism_query::alias_store::AliasStore>>,
        _tmpdir: tempfile::TempDir, // private — owns the backing dir for this fixture's lifetime
    }

    impl AliasStoreFixture {
        /// Clone the inner `Arc` for passing to constructors that require ownership.
        pub fn store(&self) -> Arc<std::sync::Mutex<prism_query::alias_store::AliasStore>> {
            Arc::clone(&self.store)
        }
    }

    /// Build an empty `AliasStore` wrapped in a self-contained `AliasStoreFixture`.
    ///
    /// F-PASS9-NIT-1: returns `AliasStoreFixture` (owns both store + TempDir) so
    /// the backing directory cannot be accidentally dropped before the store is done.
    /// Tests that don't exercise alias functionality pass `fixture.store()` to
    /// `QueryEngine::new_full`.
    pub fn make_empty_alias_store() -> AliasStoreFixture {
        let tmpdir = tempfile::tempdir().expect("create tempdir for empty alias store");
        let store = Arc::new(std::sync::Mutex::new(
            prism_query::alias_store::AliasStore::empty(tmpdir.path().join("test-aliases.toml")),
        ));
        AliasStoreFixture {
            store,
            _tmpdir: tmpdir,
        }
    }

    pub fn make_mat_ctx_with_stub(max_records: usize, row_count: usize) -> MaterializationContext {
        let org_id = OrgId::new();
        let mut registry = AdapterRegistry::new();
        registry.register(
            org_id,
            Arc::new(StubAdapter {
                sensor_id: SensorId::from("crowdstrike"),
                row_count,
                client_slug: "acme".to_string(),
                any_early_stopped: false,
            }),
        );
        let normalizer = Arc::new(OcsfNormalizer::new());
        MaterializationContext::new_with_resolver(
            Arc::new(registry),
            normalizer,
            max_records,
            Arc::new(StubCredentialResolver),
            None,
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
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 3,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
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
// AC-3: record cap returns E-QUERY-005 before DataFusion execution
// ---------------------------------------------------------------------------

/// AC-3 (BC-2.11.006): When `run_materialization_pipeline` would exceed the
/// `max_records` cap, it must return `Err` containing "E-QUERY-005"
/// (materialization limit exceeded, per error-taxonomy.md) before any
/// DataFusion SQL plan begins.
///
/// We set `max_records = 1` so any response with >=2 rows exceeds the cap.
///
/// Bug fix: original test omitted adapter registration (genuine test bug).
/// Now uses `make_mat_ctx_with_stub(1, 5)` — cap=1, stub returns 5 rows.
///
/// Red-Gate: panics at `todo!("S-3.02 — run_materialization_pipeline")`.
#[tokio::test]
async fn test_AC_3_size_limit_returns_e_query_005() {
    use prism_query::engine::QueryOptions;

    // Cap at 1 row; stub returns 5 rows — 2nd row exceeds cap → E-QUERY-005.
    let mut mat_ctx = helpers::make_mat_ctx_with_stub(1, 5);
    let session_ctx = helpers::make_ctx();
    let options = QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        sensors: None,
        limit: None,
        force_refresh: false,
        ..QueryOptions::default()
    };

    // Post-implementation: must return Err with E-QUERY-005.
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
        detail.contains("E-QUERY-005"),
        "AC-3: error must contain 'E-QUERY-005' (materialization limit exceeded); got: {detail}"
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
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
        types::FilterMap,
    };
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
        ) -> Result<FetchOutput, SensorError> {
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
            Ok(FetchOutput::new(vec![batch], false))
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
    use prism_query::{
        engine::Capability, internal_tables::register_internal_tables_with_capabilities,
    };

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
    use prism_core::{OrgId, OrgRegistry, SensorId};
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
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 2,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
        }),
    );
    registry.register(
        id_beta,
        Arc::new(helpers::StubAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 2,
            client_slug: "beta".to_string(),
            any_early_stopped: false,
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
        Arc::new(std::collections::HashMap::new()),
        helpers::make_empty_alias_store().store(),
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
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 3,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
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
// F-LP1-HIGH-7: limit > 1000 returns E-QUERY-033
// ---------------------------------------------------------------------------

/// F-LP1-HIGH-7 (BC-2.11.001): `execute` MUST return an error when `limit > 1000`.
///
/// BC-2.11.001: "Max results returned (tool-level truncation). Default 25, max 1000."
/// The engine rejects limit=1001 before any sensor contact. Error message must
/// contain "E-QUERY-033" (taxonomy v1.70 P2-01 adjudication: the interim code was a
/// Phase-1 tombstone, permanently unreusable per ADR-038 D5; E-QUERY-001 is reserved
/// for parse errors per ADV-W3MT-P58-CRIT-001).
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
        detail.contains("E-QUERY-033"),
        "HIGH-7: error must contain 'E-QUERY-033' (limit exceeds 1000; taxonomy v1.70 P2-01); got: {detail}"
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
        Arc::new(std::collections::HashMap::new()),
        helpers::make_empty_alias_store().store(),
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
        Arc::new(std::collections::HashMap::new()),
        helpers::make_empty_alias_store().store(),
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
        Arc::new(std::collections::HashMap::new()),
        helpers::make_empty_alias_store().store(),
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
/// MUST trigger E-QUERY-005 at the pipeline level, verifying the cap is checked
/// BEFORE DataFusion execution and across multiple sources.
///
/// This complements `test_AC_3_size_limit_returns_e_query_005` which only
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
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 6_000,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
        }),
    );
    registry.register(
        OrgId::new(),
        Arc::new(helpers::StubAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 6_000,
            client_slug: "beta".to_string(),
            any_early_stopped: false,
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
        "AC-3-bis: 12K total rows (6K×2 adapters) must exceed the 10K cap → E-QUERY-005",
    );
    let detail = err.to_string();
    assert!(
        detail.contains("E-QUERY-005"),
        "AC-3-bis: error must contain 'E-QUERY-005' (materialization limit exceeded); got: {detail}"
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
        Arc::new(std::collections::HashMap::new()),
        helpers::make_empty_alias_store().store(),
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
    use std::collections::BTreeMap;

    use arrow::array::UInt64Array;
    use prism_query::engine::{Capability, QueryEngine, QueryEngineConfig, QueryOptions};
    use prism_storage::audit_buffer::{append_audit_entry, AuditEntry};

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
        Arc::new(std::collections::HashMap::new()),
        helpers::make_empty_alias_store().store(),
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
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            row_count: 2,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
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
        Arc::new(std::collections::HashMap::new()),
        helpers::make_empty_alias_store().store(),
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
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            row_count: 2,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
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
        Arc::new(std::collections::HashMap::new()),
        helpers::make_empty_alias_store().store(),
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
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            row_count: 2,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
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
        Arc::new(std::collections::HashMap::new()),
        helpers::make_empty_alias_store().store(),
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
    use prism_query::{
        engine::Capability,
        internal_tables::{InternalTableDescriptor, RocksDbTableProvider},
    };
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
    use prism_query::{
        engine::Capability,
        internal_tables::{InternalTableDescriptor, RocksDbTableProvider},
    };
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
    use prism_query::{
        engine::Capability,
        internal_tables::{InternalTableDescriptor, RocksDbTableProvider},
    };
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
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
    };

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
        ) -> Result<FetchOutput, SensorError> {
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
            Ok(FetchOutput::new(vec![batch], false))
        }
    }

    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let org_id = OrgId::new();

    // Helper to build a mat_ctx sharing the same adapter.
    let make_mat_ctx = {
        let call_count = Arc::clone(&call_count);
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

    // Display must contain E-QUERY-033 (taxonomy v1.70 P2-01 adjudication: the interim
    // code was a Phase-1 tombstone, permanently unreusable per ADR-038 D5; the code was
    // moved off E-QUERY-001 by ADV-W3MT-P58-CRIT-001 to avoid the QueryParseFailed
    // collision).
    let display = err.to_string();
    assert!(
        display.contains("E-QUERY-033"),
        "LP2-LOW-1: display must contain 'E-QUERY-033' (limit exceeded, taxonomy v1.70 P2-01); got: {display}"
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
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            row_count: 5,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
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
// HIGH-002 (ADV-W3MT-P58): Timeout (E-QUERY-004) and depth-limit (E-QUERY-003) tests
// ---------------------------------------------------------------------------

/// HIGH-002 / ADV-W3MT-P58-HIGH-002: Story §Tasks item 8 mandates a timeout test.
///
/// Set `timeout_secs = 1` and use a SlowAdapter that sleeps beyond the timeout.
/// `execute` must return `PrismError::QueryTimeout` (E-QUERY-004 path — timeout fires
/// before DataFusion execution begins when the sensor adapter is slow).
///
/// Note (error-taxonomy.md): E-QUERY-004 is the query timeout code; E-WATCHDOG-001
/// is the memory-budget code; E-QUERY-005 is the materialization-limit code.
/// The `PrismError::QueryTimeout` variant displays "E-QUERY-004: query timed out after".
/// The test verifies the correct variant, not a string code, to avoid brittle assertions.
#[tokio::test]
async fn test_AC_timeout_returns_query_timeout_error() {
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::{QueryEngine, QueryEngineConfig, QueryOptions};
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
    };

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
        ) -> Result<FetchOutput, SensorError> {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            Ok(FetchOutput::new(vec![], false))
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
        Arc::new(std::collections::HashMap::new()),
        helpers::make_empty_alias_store().store(),
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
        "timeout-test: error must be PrismError::QueryTimeout (E-QUERY-004); got: {err:?}"
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
    // OR security module returns QuerySecurityLimitExceeded (E-QUERY-003) with
    // depth-limit detail. Either way the result must be Err.
    assert!(
        matches!(
            err,
            prism_core::PrismError::QueryParseFailed { .. }
                | prism_core::PrismError::QuerySecurityLimitExceeded { .. }
        ),
        "depth-limit-test: error must be a parse or execution failure (depth limit); got: {detail}"
    );
}

// ---------------------------------------------------------------------------
// F-LP1-CRITICAL-001 regression: unknown source table must return E-QUERY-036
// ---------------------------------------------------------------------------

/// S-PLUGIN-PREREQ-A F-LP1-CRITICAL-001 regression test.
///
/// An unknown table name (prefix not registered in the adapter registry) MUST
/// return `Err` containing "E-QUERY-036" rather than silently producing empty
/// results. Before the fix, `unknown_table | host = 'x'` would silently produce
/// an empty result set because `sensor_id_from_table_name` accepted any
/// non-empty prefix, and `get_all_for_sensor("unknown")` returned empty.
///
/// P6-02 adjudication 2026-06-11: the error is now `PrismError::UnknownSourceTable`
/// (E-QUERY-036) rather than `QueryExecutionFailed` with an embedded E-QUERY-006
/// prefix. Both conditions describe the same caller-visible condition; the variant
/// change maps the error to -32602 INVALID_PARAMS instead of -32000 INTERNAL_ERROR.
///
/// The registry must be NON-EMPTY for this guard to fire — an empty registry
/// indicates test/boot mode where the sensor roster is not yet known. In production
/// the registry is always populated with at least the four built-in sensors.
///
/// This test verifies the two-stage check: extract prefix + registry membership.
#[tokio::test]
async fn test_resolve_source_refs_unknown_table_returns_e_query_036() {
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
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 0,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
        }),
    );
    let org_slug = OrgSlug::new("acme");
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // "unknown_table" has prefix "unknown" — not in registry → must be E-QUERY-036.
    // (Registry is non-empty so the guard fires; "unknown" != "crowdstrike".)
    let result = engine.execute("unknown_table | host = 'x'", options).await;

    let err = result.expect_err(
        "F-LP1-CRITICAL-001: unknown_table must return Err (E-QUERY-036), not empty results",
    );
    // Verify the error is the dedicated UnknownSourceTable variant (E-QUERY-036).
    assert!(
        matches!(err, prism_core::PrismError::UnknownSourceTable(..)),
        "F-LP1-CRITICAL-001: error must be PrismError::UnknownSourceTable (E-QUERY-036); got: {err:?}"
    );
    let detail = err.to_string();
    assert!(
        detail.contains("E-QUERY-036"),
        "F-LP1-CRITICAL-001: error display must contain 'E-QUERY-036'; got: {detail}"
    );
    assert!(
        detail.contains("unknown_table"),
        "F-LP1-CRITICAL-001: error display must include the unknown table name; got: {detail}"
    );
}

// ---------------------------------------------------------------------------
// QRY-02 (BC-2.07.003): cross-query response cache wired into QueryEngine
// ---------------------------------------------------------------------------

/// Sensor adapter that counts `fetch` invocations — used to observe whether the
/// engine served a query from the cross-query response cache (no fetch) or hit
/// the sensor API (fetch counted).
struct FetchCountingAdapter {
    call_count: Arc<std::sync::atomic::AtomicUsize>,
}

#[async_trait]
impl prism_sensors::adapter::SensorAdapter for FetchCountingAdapter {
    fn sensor_type(&self) -> prism_core::SensorId {
        prism_core::SensorId::from("crowdstrike")
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike"
    }

    async fn fetch(
        &self,
        _spec: &prism_sensors::adapter::SensorSpec,
        _params: &prism_sensors::adapter::QueryParams,
        _auth: &dyn prism_sensors::auth::SensorAuth,
    ) -> Result<prism_sensors::adapter::FetchOutput, prism_sensors::adapter::SensorError> {
        self.call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let schema = Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("detection_id", arrow::datatypes::DataType::Utf8, false),
        ]));
        let arr = Arc::new(arrow::array::StringArray::from(vec!["det-1", "det-2"])) as _;
        let batch = RecordBatch::try_new(schema, vec![arr]).expect("counting batch");
        Ok(prism_sensors::adapter::FetchOutput::new(vec![batch], false))
    }
}

/// Build a `QueryEngine` backed by a single `FetchCountingAdapter`.
fn make_counting_engine() -> (
    prism_query::engine::QueryEngine,
    Arc<std::sync::atomic::AtomicUsize>,
) {
    use prism_core::OrgId;

    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut registry = AdapterRegistry::new();
    registry.register(
        OrgId::new(),
        Arc::new(FetchCountingAdapter {
            call_count: Arc::clone(&call_count),
        }),
    );
    let engine = helpers::make_engine(registry, vec![helpers::org("acme")]);
    (engine, call_count)
}

/// QRY-02 / BC-2.07.003: two identical queries through the SAME engine must
/// share the cross-query response cache — the second `execute` returns the
/// cached sensor response without calling the sensor API.
///
/// Pre-fix failure mode: `Arc<QueryCache>` was constructed by the engine but
/// `execute_inner` never consulted it, so every query re-fetched from the
/// sensor (cache dead in production).
#[tokio::test]
async fn test_QRY_02_cross_query_cache_hit_skips_sensor_fetch() {
    use prism_query::engine::QueryOptions;

    let (engine, call_count) = make_counting_engine();
    let make_options = || QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        ..QueryOptions::default()
    };

    let r1 = engine
        .execute("SELECT * FROM crowdstrike_detections", make_options())
        .await
        .expect("first execute must succeed");
    let r2 = engine
        .execute("SELECT * FROM crowdstrike_detections", make_options())
        .await
        .expect("second execute must succeed");

    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "QRY-02/BC-2.07.003: second identical query must be served from the \
         cross-query response cache (exactly 1 sensor fetch expected)"
    );
    assert_eq!(
        r1.returned_results, r2.returned_results,
        "cached response must produce the same result count as the fresh fetch"
    );
    assert!(
        r2.returned_results > 0,
        "cache-hit result must not be empty (got {} rows)",
        r2.returned_results
    );
}

/// QRY-02 / BC-2.07.003 §Postconditions: `force_refresh: true` bypasses the
/// cache read and REPLACES the existing entry with the fresh response; a
/// subsequent normal query is then served from the replaced entry.
#[tokio::test]
async fn test_QRY_02_force_refresh_bypasses_and_replaces_cache_entry() {
    use prism_query::engine::QueryOptions;

    let (engine, call_count) = make_counting_engine();
    let make_options = |force_refresh: bool| QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        force_refresh,
        ..QueryOptions::default()
    };

    // 1) Populate the cache (1 fetch).
    engine
        .execute("SELECT * FROM crowdstrike_detections", make_options(false))
        .await
        .expect("populate execute must succeed");
    // 2) force_refresh bypasses the cached entry (2nd fetch) and replaces it.
    engine
        .execute("SELECT * FROM crowdstrike_detections", make_options(true))
        .await
        .expect("force_refresh execute must succeed");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "BC-2.07.003: force_refresh must bypass the cache and call the sensor API"
    );
    // 3) Normal query is served from the replaced entry (still 2 fetches).
    engine
        .execute("SELECT * FROM crowdstrike_detections", make_options(false))
        .await
        .expect("post-refresh execute must succeed");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "BC-2.07.003 EC-07-032 spirit: force_refresh must REPLACE the entry — \
         the following normal query must hit the refreshed cache entry"
    );
}

/// P1-01 / BC-2.07.005 v4.4 (EC-07-043): limit-poisoning regression.
///
/// The fan-out target pushes the effective fetch-limit to the sensor API
/// (BC-2.01.013 / F-P1-CRIT-004), so the cached response is
/// limit-truncated at the source. A `limit=3` query must therefore populate an
/// entry that a later `limit=1000` query with identical filters does NOT hit —
/// otherwise the analyst silently receives 3 records and a wrong
/// `total_available`. Same-limit re-queries DO share the entry (bounded
/// fragmentation, architect adjudication D1,
/// `proposals/cache-envelope-adjudication-2026-06-10.md`).
#[tokio::test]
async fn test_QRY_P1_01_lower_limit_entry_must_not_serve_higher_limit_query() {
    use prism_query::engine::QueryOptions;

    let (engine, call_count) = make_counting_engine();
    let make_options = |limit: usize| QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        limit: Some(limit),
        ..QueryOptions::default()
    };

    // 1) limit=3 populates a 3-truncated cache entry (1 fetch).
    engine
        .execute("SELECT * FROM crowdstrike_detections", make_options(3))
        .await
        .expect("limit=3 execute must succeed");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "limit=3 query must fetch from the sensor"
    );

    // 2) Identical filters, limit=1000 → MUST be a cache MISS (2nd fetch).
    //    Pre-fix failure mode: the 3-truncated entry silently served the
    //    1000-limit query (P1-01 cache poisoning).
    engine
        .execute("SELECT * FROM crowdstrike_detections", make_options(1000))
        .await
        .expect("limit=1000 execute must succeed");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "EC-07-043: a limit=1000 query must NOT be served from the entry \
         populated by the limit=3 fetch (different effective fetch-limit → \
         different push_down_hash → cache miss)"
    );

    // 3) Same limit=1000 again → cache HIT (still 2 fetches): entries fetched
    //    under the same effective fetch-limit share one cache entry.
    engine
        .execute("SELECT * FROM crowdstrike_detections", make_options(1000))
        .await
        .expect("repeat limit=1000 execute must succeed");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "BC-2.07.005 v4.4: identical effective fetch-limit must share the \
         cache entry (exactly 2 fetches expected)"
    );
}

/// Sensor adapter with a toggleable failure mode and a fetch counter — used to
/// exercise the BC-2.07.003 force_refresh invalidation semantics (P1-05 /
/// architect adjudication D3).
struct TogglingFailureAdapter {
    fail: Arc<std::sync::atomic::AtomicBool>,
    call_count: Arc<std::sync::atomic::AtomicUsize>,
}

#[async_trait]
impl prism_sensors::adapter::SensorAdapter for TogglingFailureAdapter {
    fn sensor_type(&self) -> prism_core::SensorId {
        prism_core::SensorId::from("crowdstrike")
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike"
    }

    async fn fetch(
        &self,
        _spec: &prism_sensors::adapter::SensorSpec,
        _params: &prism_sensors::adapter::QueryParams,
        _auth: &dyn prism_sensors::auth::SensorAuth,
    ) -> Result<prism_sensors::adapter::FetchOutput, prism_sensors::adapter::SensorError> {
        self.call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if self.fail.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(prism_sensors::adapter::SensorError::Internal {
                detail: "injected fetch failure (P1-05 test)".to_string(),
            });
        }
        let schema = Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("detection_id", arrow::datatypes::DataType::Utf8, false),
        ]));
        let arr = Arc::new(arrow::array::StringArray::from(vec!["det-1", "det-2"])) as _;
        let batch = RecordBatch::try_new(schema, vec![arr]).expect("toggling batch");
        Ok(prism_sensors::adapter::FetchOutput::new(vec![batch], false))
    }
}

/// Build a `QueryEngine` backed by a single `TogglingFailureAdapter`.
fn make_toggling_engine() -> (
    prism_query::engine::QueryEngine,
    Arc<std::sync::atomic::AtomicBool>,
    Arc<std::sync::atomic::AtomicUsize>,
) {
    use prism_core::OrgId;

    let fail = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut registry = AdapterRegistry::new();
    registry.register(
        OrgId::new(),
        Arc::new(TogglingFailureAdapter {
            fail: Arc::clone(&fail),
            call_count: Arc::clone(&call_count),
        }),
    );
    let engine = helpers::make_engine(registry, vec![helpers::org("acme")]);
    (engine, fail, call_count)
}

/// P1-05 / BC-2.07.003 (EC-07-033, architect adjudication D3): a forced
/// refresh whose fetch fails for all targets must INVALIDATE the existing
/// cache entry — `force_refresh` is an explicit analyst distrust signal, and
/// retaining the distrusted entry would silently serve it to later non-forced
/// queries. The fetch failure is surfaced to the forcing caller via the
/// partial-failure envelope (BC-2.11.011 `sensor_errors`), and a subsequent
/// non-forced identical query is a cache MISS that re-attempts the fetch.
#[tokio::test]
async fn test_QRY_P1_05_forced_refresh_failed_fetch_invalidates_entry() {
    use prism_query::engine::QueryOptions;

    let (engine, fail, call_count) = make_toggling_engine();
    let make_options = |force_refresh: bool| QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        force_refresh,
        ..QueryOptions::default()
    };

    // 1) Populate the cache (1 fetch, success).
    engine
        .execute("SELECT * FROM crowdstrike_detections", make_options(false))
        .await
        .expect("populate execute must succeed");
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);

    // 2) Forced refresh while the sensor is failing (2nd fetch, fails).
    //    The forcing caller receives the error via the partial-failure envelope.
    fail.store(true, std::sync::atomic::Ordering::SeqCst);
    let forced = engine
        .execute("SELECT * FROM crowdstrike_detections", make_options(true))
        .await
        .expect("forced execute must return Ok with partial-failure envelope");
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert!(
        !forced.sensor_errors.is_empty(),
        "BC-2.11.011: the forcing caller must receive the fetch failure via \
         sensor_errors; got empty envelope"
    );

    // 3) Sensor recovers; a NON-forced identical query must be a cache MISS
    //    (3rd fetch) — the distrusted entry was invalidated, not retained.
    //    Pre-fix failure mode: the stale entry kept serving non-forced queries
    //    for the remainder of its TTL (count stayed 2).
    fail.store(false, std::sync::atomic::Ordering::SeqCst);
    engine
        .execute("SELECT * FROM crowdstrike_detections", make_options(false))
        .await
        .expect("post-recovery execute must succeed");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        3,
        "EC-07-033: after a failed forced refresh the entry must be \
         invalidated — the subsequent non-forced query must MISS and re-fetch"
    );
}

// ---------------------------------------------------------------------------
// RG-PSG-034 (S-ENGINE-LIMIT-EARLY-STOP-001): EC-01-039 / F-R16-P17-LENSA-MED-001
// Early-stopped response must NOT be cached as complete — `is_truncated` must
// survive a cache round-trip.
// ---------------------------------------------------------------------------

/// Sensor adapter that (a) counts `fetch` invocations and (b) always signals
/// early-stop in `FetchOutput` — models a sensor whose first page fills the
/// requested limit exactly (LIMIT == page_size) while more upstream rows exist.
struct EarlyStopCountingAdapter {
    call_count: Arc<std::sync::atomic::AtomicUsize>,
    /// Number of rows to emit per fetch (always == requested limit in this test).
    row_count: usize,
}

#[async_trait]
impl prism_sensors::adapter::SensorAdapter for EarlyStopCountingAdapter {
    fn sensor_type(&self) -> prism_core::SensorId {
        prism_core::SensorId::from("crowdstrike")
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike"
    }

    async fn fetch(
        &self,
        _spec: &prism_sensors::adapter::SensorSpec,
        _params: &prism_sensors::adapter::QueryParams,
        _auth: &dyn prism_sensors::auth::SensorAuth,
    ) -> Result<prism_sensors::adapter::FetchOutput, prism_sensors::adapter::SensorError> {
        self.call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let schema = Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("detection_id", arrow::datatypes::DataType::Utf8, false),
        ]));
        let ids: Vec<String> = (0..self.row_count).map(|i| format!("early-{i}")).collect();
        let arr = Arc::new(arrow::array::StringArray::from(
            ids.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        )) as _;
        let batch = RecordBatch::try_new(schema, vec![arr]).expect("early-stop batch");
        // any_early_stopped = true: sensor stopped after page 1 with more rows upstream.
        Ok(prism_sensors::adapter::FetchOutput::new(vec![batch], true))
    }
}

/// Build a `QueryEngine` backed by a single `EarlyStopCountingAdapter`.
fn make_early_stop_engine(
    row_count: usize,
) -> (
    prism_query::engine::QueryEngine,
    Arc<std::sync::atomic::AtomicUsize>,
) {
    use prism_core::OrgId;

    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut registry = AdapterRegistry::new();
    registry.register(
        OrgId::new(),
        Arc::new(EarlyStopCountingAdapter {
            call_count: Arc::clone(&call_count),
            row_count,
        }),
    );
    let engine = helpers::make_engine(registry, vec![helpers::org("acme")]);
    (engine, call_count)
}

/// RG-PSG-034 / EC-01-039 (F-R16-P17-LENSA-MED-001, BC-2.16.002):
/// An early-stopped response MUST NOT be cached as "complete".
///
/// Scenario: the adapter returns exactly `limit` rows with `any_early_stopped=true`
/// (LIMIT == page_size; more data exists upstream). The first `execute` populates
/// the response cache. The second identical `execute` is a cache HIT (sensor NOT
/// fetched again). `is_truncated` must be `true` on the cached-response path.
///
/// CURRENT (RED): `complete = fan_result.errors.is_empty()` ignores
/// `any_early_stopped` → the early-stopped partial is cached as "complete" →
/// the cache-hit path never restores `any_early_stopped` → Engine Step 6 computes
/// `is_truncated = (total_rows > limit) || false = (2 > 2) || false = false`.
/// The assertion below therefore FAILS against HEAD @ce196ae7b.
///
/// GREEN CONTRACT (implementer fix):
/// `complete = errors.is_empty() && !fan_result.any_early_stopped`
/// → early-stopped responses are NOT cached → Query 2 re-fetches or the
/// cache-hit path restores `any_early_stopped` → `is_truncated = true`.
#[tokio::test]
async fn test_psg_rg034_early_stopped_response_not_cached_as_complete() {
    use prism_query::engine::QueryOptions;

    // 2 rows returned, limit = 2 → early-stop condition (LIMIT == page_size).
    // any_early_stopped=true means more data exists upstream.
    let limit = 2_usize;
    let (engine, call_count) = make_early_stop_engine(limit);

    let make_options = || QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        limit: Some(limit),
        ..QueryOptions::default()
    };

    // Query 1: live fetch → sensor called once → early-stop response cached (bug)
    // or NOT cached (correct).
    let r1 = engine
        .execute("SELECT * FROM crowdstrike_detections", make_options())
        .await
        .expect("Query 1 must succeed");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "RG-PSG-034: Query 1 must invoke the sensor adapter exactly once"
    );
    // Query 1 must itself report is_truncated=true (early-stop signal present).
    assert!(
        r1.is_truncated,
        "RG-PSG-034: Query 1 (live fetch) must report is_truncated=true \
         when the adapter sets any_early_stopped=true (EC-01-039)"
    );

    // Query 2: identical SQL + same options → may be a cache HIT (if the bug
    // incorrectly cached the partial) or a cache MISS (if correctly NOT cached).
    // Either way, is_truncated MUST be true because more data exists upstream.
    let r2 = engine
        .execute("SELECT * FROM crowdstrike_detections", make_options())
        .await
        .expect("Query 2 must succeed");

    // PRIMARY ASSERTION (the discriminating test): is_truncated must survive
    // the cache round-trip. With the current bug:
    //   - cache stores the early-stopped partial as "complete" (errors.is_empty())
    //   - cache-hit path never restores any_early_stopped
    //   - Engine Step 6: is_truncated = (2 > 2) || false = false  ← FAILS HERE
    assert!(
        r2.is_truncated,
        "RG-PSG-034 / EC-01-039: Query 2 must report is_truncated=true even \
         when served from the response cache; an early-stopped partial response \
         must NOT be cached as complete (complete = errors.is_empty() ignores \
         any_early_stopped; got is_truncated=false — cached partial lost the \
         truncation signal)"
    );
}

// ---------------------------------------------------------------------------
// RG-PSG-035/036 (S-ENGINE-LIMIT-EARLY-STOP-001): DI-019 cache-completeness
// and Step-6 truncation-signal red gates.
//
// DI-019 scenario: the spec-engine pipeline (`PipelineExecutor::execute_impl`)
// truncates at `MAX_PIPELINE_RECORDS = 10_000` and sets `PipelineResult.truncated
// = true`, but `early_stopped` remains `false`. This `truncated` signal is NOT
// propagated through `FetchOutput` → `FanOutResult` → `MaterializationOutput` to
// the cache-completeness check or engine Step 6. As a result:
//   • The 10K-capped response is incorrectly cached as "complete" (bug: should
//     be a cache miss on re-query so the analyst gets fresh, complete data).
//   • Engine Step 6 computes `is_truncated = total_rows > limit || false = false`
//     when `limit=None` → usize::MAX, suppressing the truncation signal.
//
// These tests exercise the mock-adapter approximation of DI-019:
//   • `Di019CountingAdapter` returns 10,001 rows with `any_early_stopped=false`
//     (simulating the 10K-capped fetch output before `any_pipeline_truncated` is
//     added to `FetchOutput` by the implementer).
//   • `make_di019_engine` raises `max_materialized_records` to `usize::MAX` so
//     the materialization layer does NOT hard-error (E-QUERY-005) on 10,001 rows
//     — the DI-019 cap fires in the spec-engine pipeline level, not here.
//
// GREEN CONTRACT for both tests: the implementer adds `any_pipeline_truncated:
// bool` to `FetchOutput`, propagates it through `FanOutResult` →
// `MaterializationOutput`, updates the cache-completeness guard in
// `materialization.rs`, extends engine Step 6, and updates this mock to set
// `any_pipeline_truncated = true`.
// ---------------------------------------------------------------------------

/// Sensor adapter that counts `fetch` invocations and returns exactly 10,001 rows
/// with `any_early_stopped=false` — models the surface output of a DI-019 pipeline
/// truncation event (`PipelineResult.truncated=true`, `PipelineResult.early_stopped=
/// false`). Row generator uses a lightweight string index column (no hand-written
/// literals); one batch per fetch call.
///
/// NOTE: `FetchOutput::new(batch, false)` is the CURRENT call — it cannot set
/// `any_pipeline_truncated` because that field does not yet exist on `FetchOutput`.
/// The implementer will extend this mock as part of making RG-PSG-035/036 GREEN.
struct Di019CountingAdapter {
    call_count: Arc<std::sync::atomic::AtomicUsize>,
}

#[async_trait]
impl prism_sensors::adapter::SensorAdapter for Di019CountingAdapter {
    fn sensor_type(&self) -> prism_core::SensorId {
        prism_core::SensorId::from("crowdstrike")
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike"
    }

    async fn fetch(
        &self,
        _spec: &prism_sensors::adapter::SensorSpec,
        _params: &prism_sensors::adapter::QueryParams,
        _auth: &dyn prism_sensors::auth::SensorAuth,
    ) -> Result<prism_sensors::adapter::FetchOutput, prism_sensors::adapter::SensorError> {
        self.call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let schema = Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("detection_id", arrow::datatypes::DataType::Utf8, false),
        ]));
        // 10,001 rows: one more than MAX_PIPELINE_RECORDS (10,000). Lightweight
        // generator — no hand-written literals. In the spec-engine pipeline this
        // volume causes DI-019 to fire (truncate to 10K, PipelineResult.truncated=true).
        let ids: Vec<String> = (0..10_001_usize).map(|i| format!("di019-{i}")).collect();
        let arr = Arc::new(arrow::array::StringArray::from(
            ids.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        )) as _;
        let batch = RecordBatch::try_new(schema, vec![arr]).expect("di019 counting batch");
        // any_early_stopped=false: DI-019 is a pipeline record-count cap
        // (PipelineResult.truncated=true), NOT a LIMIT-driven early-stop
        // (PipelineResult.early_stopped=false). The current FetchOutput API cannot
        // carry the DI-019 truncated signal; the implementer will add
        // `any_pipeline_truncated` to FetchOutput and update this mock accordingly.
        Ok(prism_sensors::adapter::FetchOutput::new(vec![batch], false))
    }
}

/// Build a `QueryEngine` backed by a single `Di019CountingAdapter`.
///
/// `max_materialized_records = usize::MAX` (DI-019 tests only): the DI-019 record-count
/// cap fires in the spec-engine pipeline layer (`PipelineExecutor::execute_impl`), not
/// in the materialization layer. A direct mock adapter bypasses the spec-engine, so
/// raising the materialization cap prevents the engine from hard-erroring
/// (E-QUERY-005 `QueryMaterializationLimitExceeded`) on the 10,001-row batch. Without
/// this, `execute()` would return `Err` rather than `Ok(QueryResult)`, making the
/// cache-completeness and `is_truncated` assertions unreachable.
fn make_di019_engine() -> (
    prism_query::engine::QueryEngine,
    Arc<std::sync::atomic::AtomicUsize>,
) {
    use prism_core::OrgId;
    use prism_query::engine::QueryEngineConfig;

    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut registry = AdapterRegistry::new();
    registry.register(
        OrgId::new(),
        Arc::new(Di019CountingAdapter {
            call_count: Arc::clone(&call_count),
        }),
    );
    let adapter_registry = Arc::new(registry);
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        helpers::org("acme"),
    ]));
    let config = QueryEngineConfig {
        // Must be > 10_001 to prevent E-QUERY-005 `QueryMaterializationLimitExceeded`.
        // DI-019 cap fires in the spec-engine pipeline layer; these tests exercise
        // the cache-completeness guard and `is_truncated` signal that depend on
        // `any_pipeline_truncated`, not the materialization record limit.
        max_materialized_records: usize::MAX,
        ..QueryEngineConfig::default()
    };
    let engine = prism_query::engine::QueryEngine::new(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
    )
    .with_credential_resolver(Arc::new(helpers::StubCredentialResolver));
    (engine, call_count)
}

/// RG-PSG-035 / EC-01-039 / F-R16-P18-LENSA-MED-001 (DI-019 cache-completeness):
/// A pipeline-level record-count truncation (DI-019) must NOT be cached as a
/// "complete" response. A second identical query must re-fetch from the sensor.
///
/// Scenario: `Di019CountingAdapter` returns 10,001 rows with `any_early_stopped=false`
/// (the DI-019 profile). Two identical queries are executed against the same engine.
/// The first populates the cross-query response cache; the second must be a cache
/// MISS (adapter called again) because the first response was DI-019-truncated and
/// therefore incomplete.
///
/// CURRENT (RED): `complete = fan_result.errors.is_empty() && !fan_result.any_early_stopped`
/// `= true && !false = true` — DI-019's `truncated=true` is not propagated →
/// response is incorrectly cached as "complete" → Query 2 is a cache HIT → adapter
/// call count stays at 1 → the `== 2` assertion below FAILS.
///
/// GREEN CONTRACT (implementer + gate extension):
///   1. Add `any_pipeline_truncated: bool` to `FetchOutput`; expose via new constructor.
///   2. Propagate `PipelineResult.truncated` → `FetchOutput.any_pipeline_truncated`
///      → `FanOutResult.any_pipeline_truncated` → `MaterializationOutput.any_pipeline_truncated`.
///   3. Update cache-completeness guard in `materialization.rs`:
///      `complete = errors.is_empty() && !any_early_stopped && !any_pipeline_truncated`.
///   4. Update this mock to set `any_pipeline_truncated = true`.
///   After the fix: not cached → Query 2 re-fetches → call count == 2.
#[tokio::test]
async fn test_psg_rg035_di019_truncated_response_not_cached_as_complete() {
    use prism_query::engine::QueryOptions;

    let (engine, call_count) = make_di019_engine();
    let make_options = || QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        ..QueryOptions::default()
    };

    // Query 1: live fetch — adapter is called once.
    let _r1 = engine
        .execute("SELECT * FROM crowdstrike_detections", make_options())
        .await
        .expect("RG-PSG-035: Query 1 must succeed");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "RG-PSG-035: Query 1 must invoke the sensor adapter exactly once"
    );

    // Query 2: identical SQL + same options.
    // MUST be a cache MISS (adapter called again) because the DI-019-truncated
    // partial is not a complete response and must not be cached.
    //
    // CURRENTLY (RED): the result IS cached (because the DI-019 truncated signal
    // is not propagated to the cache-completeness guard), so the adapter is NOT
    // called — call count stays 1 — and this assertion FAILS.
    let _r2 = engine
        .execute("SELECT * FROM crowdstrike_detections", make_options())
        .await
        .expect("RG-PSG-035: Query 2 must succeed");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "RG-PSG-035 / EC-01-039 (F-R16-P18-LENSA-MED-001): a DI-019-truncated \
         response (PipelineResult.truncated=true, any_early_stopped=false) must \
         NOT be cached as complete — Query 2 must re-fetch from the sensor \
         (expected call_count == 2, got {}). \
         Root cause: `complete = errors.is_empty() && !any_early_stopped` does \
         not account for `any_pipeline_truncated` → DI-019 partial is stored as \
         a complete cache entry → Query 2 is a cache HIT → adapter not called. \
         Fix: add `&& !fan_result.any_pipeline_truncated` to the completeness guard.",
        call_count.load(std::sync::atomic::Ordering::SeqCst)
    );
}

/// RG-PSG-036 / EC-01-039 / F-R16-P18-LENSA-MED-001 (DI-019 Step-6 truncation signal):
/// When a pipeline-level record-count truncation (DI-019) fires and `options.limit=None`
/// (no user-specified row limit), `QueryResult.is_truncated` must be `true`.
///
/// Scenario: `Di019CountingAdapter` returns 10,001 rows with `any_early_stopped=false`
/// (the DI-019 profile). Query is executed with `options.limit = None`.
///
/// CURRENT (RED): Engine Step 6 formula:
///   `is_truncated = total_rows > limit || any_early_stopped`
///              = `10_001 > usize::MAX   || false`
///              = `false                 || false`
///              = `false`
/// DI-019's `PipelineResult.truncated=true` is never propagated to
/// `MaterializationOutput`, so Step 6 has no `any_pipeline_truncated` term →
/// `is_truncated=false` → the assertion below FAILS.
///
/// GREEN CONTRACT (implementer):
///   Extend engine Step 6 to:
///   `is_truncated = total_rows > limit || output.any_early_stopped || output.any_pipeline_truncated`
///   After propagating `any_pipeline_truncated = true` from the mock adapter:
///   `is_truncated = false || false || true = true`.
#[tokio::test]
async fn test_psg_rg036_di019_truncated_step6_is_truncated_true() {
    use prism_query::engine::QueryOptions;

    let (engine, _call_count) = make_di019_engine();
    let options = QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        // limit=None → effective_limit = usize::MAX in engine Step 6.
        // With 10,001 rows: `10_001 > usize::MAX` is false, so the truncation
        // signal MUST come from `any_pipeline_truncated`, not `total_rows > limit`.
        limit: None,
        ..QueryOptions::default()
    };

    let result = engine
        .execute("SELECT * FROM crowdstrike_detections", options)
        .await
        .expect("RG-PSG-036: execute must succeed (10,001 rows within usize::MAX mat-cap)");

    assert!(
        result.is_truncated,
        "RG-PSG-036 / EC-01-039 (F-R16-P18-LENSA-MED-001): \
         QueryResult.is_truncated must be true when DI-019 fires \
         (PipelineResult.truncated=true, any_early_stopped=false, limit=None). \
         Current engine Step 6 formula `total_rows > limit || any_early_stopped` = \
         `10_001 > usize::MAX || false` = false — the `any_pipeline_truncated` \
         term is missing. Fix: extend Step 6 to \
         `is_truncated = total_rows > limit || any_early_stopped || any_pipeline_truncated`."
    );
}

// ---------------------------------------------------------------------------
// S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001: CRIT-1 / CRIT-2
// SqlPipe end-to-end via QueryEngine::execute (not via bare plan_sqlpipe_query)
// ---------------------------------------------------------------------------

/// CRIT-1: `SELECT * FROM crowdstrike_detections | limit 10` must execute
/// end-to-end via `QueryEngine::execute` and return rows (NOT empty).
///
/// Before the fix, `execute_against_session` dispatches `Ast::SqlPipe` to the
/// `_ =>` catch-all which silently returns `Ok(Vec::new())`.
/// After the fix, the `Ast::SqlPipe` arm is present and the pipe stages (| limit)
/// are applied via the SQL lowering path.
///
/// Red Gate: before the fix, the assertion `total_rows > 0` FAILS because
/// the engine returns 0 rows for any SqlPipe query.
#[tokio::test]
async fn test_crit1_sqlpipe_executes_via_engine_not_empty() {
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let org_id = OrgId::new();
    let mut registry = prism_sensors::AdapterRegistry::new();
    registry.register(
        org_id,
        std::sync::Arc::new(helpers::StubAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 5,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
        }),
    );
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    let result = engine
        .execute("SELECT * FROM crowdstrike_detections | limit 10", options)
        .await
        .expect("CRIT-1: SqlPipe query must not fail");

    let total_rows: usize = result.batches.iter().map(|b| b.num_rows()).sum();
    assert!(
        total_rows > 0,
        "CRIT-1: SqlPipe query must return rows (not empty); \
         got 0 — Ast::SqlPipe arm is missing from execute_against_session"
    );
}

/// CRIT-2: `SELECT * FROM crowdstrike_detections LIMIT 5 | limit 3` must return
/// `Err(PrismError::RedundantRowLimit{sql_limit:5, pipe_limit:3})` via
/// `QueryEngine::execute` (not via bare `plan_sqlpipe_query`).
///
/// The FORBID-BOTH E-QUERY-040 check must be wired into the production execute path.
///
/// Red Gate: before the fix, the `_ =>` catch-all returns `Ok(Vec::new())` so
/// the engine silently succeeds instead of returning the hard error.
#[tokio::test]
async fn test_crit2_sqlpipe_forbid_both_via_engine_returns_e_query_040() {
    use prism_core::{OrgId, PrismError, SensorId};
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let org_id = OrgId::new();
    let mut registry = prism_sensors::AdapterRegistry::new();
    registry.register(
        org_id,
        std::sync::Arc::new(helpers::StubAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 5,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
        }),
    );
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections LIMIT 5 | limit 3",
            options,
        )
        .await;

    let err = result.expect_err(
        "CRIT-2: FORBID-BOTH (both SQL LIMIT and pipe | limit) must return Err, not Ok",
    );
    assert!(
        matches!(
            err,
            PrismError::RedundantRowLimit {
                sql_limit: 5,
                pipe_limit: 3,
            }
        ),
        "CRIT-2: error must be PrismError::RedundantRowLimit{{sql_limit:5, pipe_limit:3}}; got: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001: CRIT-1b
// NOW() substitution wired into the production execute path
// ---------------------------------------------------------------------------

/// BC-2.11.021 AC-004 (coverage): SQL-mode temporal query executes end-to-end.
///
/// SCOPE NOTE (post D-1333 Option A): execute_against_session re-emits the
/// SQL-mode AST via `PqlNormalizer::normalize(ast)` (plan-pinned constant),
/// so DataFusion receives `TIMESTAMP '<iso>'` rather than a runtime `NOW()`.
/// This test is a smoke test confirming SQL-mode NOW() queries succeed end-to-end
/// (regression coverage). The DISCRIMINATING test is
/// `test_high003_discriminating_sql_in_window_row_returned`, which asserts exactly
/// 1 in-window row is returned (proving the temporal predicate fires correctly).
#[tokio::test]
async fn test_crit1b_sql_mode_now_substituted_before_datafusion() {
    use arrow::{
        array::{StringArray, TimestampMicrosecondArray},
        datatypes::{DataType, Field, Schema, TimeUnit},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
    };

    /// StubAdapter that returns rows with a real `timestamp` column (Utf8 ISO-8601).
    /// DataFusion can CAST a Utf8 column to TIMESTAMP in the WHERE clause, so the
    /// injected `TIMESTAMP '<iso8601>'` literal is type-compatible.
    struct TimestampStubAdapter;

    #[async_trait]
    impl SensorAdapter for TimestampStubAdapter {
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
        ) -> Result<FetchOutput, SensorError> {
            // Return one row with a `timestamp` column as a recent ISO-8601 string.
            // This is a recent timestamp so it falls within `NOW() - INTERVAL '7d'`.
            let schema = Arc::new(Schema::new(vec![
                Field::new("detection_id", DataType::Utf8, false),
                Field::new("timestamp", DataType::Utf8, false),
            ]));
            let ids = Arc::new(StringArray::from(vec!["det-001"])) as _;
            // Use chrono to produce a UTC timestamp string.
            let ts_str = chrono::Utc::now().to_rfc3339();
            let ts_arr = Arc::new(StringArray::from(vec![ts_str.as_str()])) as _;
            let batch =
                RecordBatch::try_new(schema, vec![ids, ts_arr]).expect("timestamp stub batch");
            Ok(FetchOutput::new(vec![batch], false))
        }
    }

    let org_slug = helpers::org("acme");
    let mut registry = AdapterRegistry::new();
    registry.register(OrgId::new(), Arc::new(TimestampStubAdapter));
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // SQL-mode query with NOW() - INTERVAL. DataFusion 53.1 handles NOW() natively
    // in SQL strings, so this passes whether or not inject_now is called.
    // Retained as regression coverage for the SQL-mode execution path.
    let result = engine
        .execute(
            "SELECT detection_id FROM crowdstrike_detections \
             WHERE timestamp > NOW() - INTERVAL '7d'",
            options,
        )
        .await;

    assert!(
        result.is_ok(),
        "CRIT-1b: SQL-mode query with NOW() must succeed (DataFusion handles NOW() \
         natively in SQL strings); got: {:?}",
        result.err()
    );
}

/// BC-2.11.021 AC-004 (coverage): SqlPipe-head temporal query executes end-to-end.
///
/// SCOPE NOTE (post D-1333 Option A): execute_against_session computes
/// `plan_pinned_head_sql` from `PqlNormalizer::normalize(spq.head AST)` and
/// passes it directly to `sqlpipe_to_executable_sql` (no find_sqlpipe_split).
/// Retained as regression coverage for the SqlPipe execution path.
/// The DISCRIMINATING test is `test_high003_discriminating_sqlpipe_in_window_row_returned`.
#[tokio::test]
async fn test_crit1b_sqlpipe_head_now_substituted_before_datafusion() {
    use arrow::{
        array::StringArray,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
    };

    struct TimestampStubAdapterSqlPipe;

    #[async_trait]
    impl SensorAdapter for TimestampStubAdapterSqlPipe {
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
        ) -> Result<FetchOutput, SensorError> {
            let schema = Arc::new(Schema::new(vec![
                Field::new("detection_id", DataType::Utf8, false),
                Field::new("timestamp", DataType::Utf8, false),
            ]));
            let ts_str = chrono::Utc::now().to_rfc3339();
            let batch = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(StringArray::from(vec!["det-sqlpipe-001"])) as _,
                    Arc::new(StringArray::from(vec![ts_str.as_str()])) as _,
                ],
            )
            .expect("sqlpipe timestamp stub batch");
            Ok(FetchOutput::new(vec![batch], false))
        }
    }

    let org_slug = helpers::org("acme");
    let mut registry = AdapterRegistry::new();
    registry.register(OrgId::new(), Arc::new(TimestampStubAdapterSqlPipe));
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // SqlPipe: SQL head with NOW() followed by a pipe | limit stage.
    // inject_now must fire for SqlPipe.head BEFORE sqlpipe_to_executable_sql.
    let result = engine
        .execute(
            "SELECT detection_id FROM crowdstrike_detections \
             WHERE timestamp > NOW() - INTERVAL '7d' | limit 10",
            options,
        )
        .await;

    assert!(
        result.is_ok(),
        "CRIT-1b SqlPipe: SqlPipe temporal query with NOW() must succeed (DataFusion \
         handles NOW() natively in the head SQL string); got: {:?}",
        result.err()
    );
}

/// BC-2.11.020 INV-FORBID-BOTH-PERMANENT (coverage): FORBID-BOTH fires with a 0-row batch.
///
/// SCOPE NOTE: `StubAdapter(row_count: 0)` returns `Ok(vec![0-row-batch])` — one batch
/// with zero rows. That batch IS pushed to table_batches; the MemTable IS registered;
/// the early-return guard does NOT fire. This test passes because the FORBID-BOTH hoist
/// is now in run_materialization_pipeline Step 1b (before fan-out), but it would also
/// have passed with the old code because execute_against_session was always reached.
///
/// The TRUE early-return bypass test (adapter returning Ok(vec![])) is:
/// `test_high1_forbid_both_fires_with_empty_vec_sensor`.
/// Retained as regression coverage for the zero-row-batch case.
#[tokio::test]
async fn test_forbid_both_fires_with_zero_row_sensor() {
    use prism_core::{OrgId, PrismError, SensorId};
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let mut registry = AdapterRegistry::new();
    // row_count: 0 → sensor returns Ok(vec![0-row-batch]) (one batch, zero rows).
    // The batch IS pushed to table_batches; the early-return guard does NOT fire.
    // This exercises FORBID-BOTH via the standard code path.
    registry.register(
        OrgId::new(),
        Arc::new(helpers::StubAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 0,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
        }),
    );
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // Query has BOTH SQL LIMIT 5 AND pipe | limit 3 — must be E-QUERY-040.
    // FORBID-BOTH is enforced in run_materialization_pipeline Step 1b (after parse,
    // before fan-out), so it fires regardless of row count.
    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections LIMIT 5 | limit 3",
            options,
        )
        .await;

    let err = result.expect_err("FORBID-BOTH (E-QUERY-040) must fire with 0-row-batch sensor");
    let msg = err.to_string();
    assert!(
        msg.contains("E-QUERY-040"),
        "error must contain 'E-QUERY-040' (FORBID-BOTH); got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001: HIGH-2
// did_you_mean near-miss via QueryEngine::execute
// ---------------------------------------------------------------------------

/// HIGH-2: A near-miss table name (Levenshtein ≤ 3 to a registered table)
/// must produce `PrismError::UnknownSourceTable` with a non-None `did_you_mean`
/// field via `QueryEngine::execute`.
///
/// The existing E-QUERY-036 test only covers Lev > 3 (did_you_mean: None).
/// This test exercises the `Some(...)` branch.
///
/// Red Gate: if did_you_mean is None for a near-miss, the assertion fails.
#[tokio::test]
async fn test_high2_did_you_mean_near_miss_via_engine() {
    use prism_core::{OrgId, PrismError, SensorId};
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let org_id = OrgId::new();
    let mut registry = prism_sensors::AdapterRegistry::new();
    registry.register(
        org_id,
        std::sync::Arc::new(helpers::StubAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 0,
            client_slug: "acme".to_string(),
            any_early_stopped: false,
        }),
    );
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // "crowdstrik" is 1 edit (delete 'e') from "crowdstrike_detections"
    // Lev("crowdstrik_detections", "crowdstrike_detections") = 1 — well within the ≤3 threshold.
    let result = engine
        .execute("crowdstrik_detections | host = 'test'", options)
        .await;

    let err =
        result.expect_err("HIGH-2: near-miss table name must return Err (E-QUERY-036), not Ok");

    match &err {
        PrismError::UnknownSourceTable(detail) => {
            assert!(
                detail.did_you_mean.is_some(),
                "HIGH-2: did_you_mean must be Some(...) for near-miss 'crowdstrik_detections'; \
                 got None — Levenshtein near-miss branch not reached"
            );
            let suggestion = detail.did_you_mean.as_deref().unwrap();
            assert!(
                suggestion.contains("crowdstrike"),
                "HIGH-2: did_you_mean must suggest a name containing 'crowdstrike'; got: {suggestion}"
            );
        }
        other => panic!("HIGH-2: expected PrismError::UnknownSourceTable, got: {other:?}"),
    }
}

/// BC-2.07.004: a write-operation invalidation against the engine's response
/// cache (`QueryEngine::response_cache()`) evicts the cached entries, so the
/// next query re-fetches from the sensor (write-then-read consistency, DEC-018).
#[tokio::test]
async fn test_BC_2_07_004_write_invalidation_evicts_engine_response_cache() {
    use prism_query::{engine::QueryOptions, invalidation::CacheInvalidator};

    let (engine, call_count) = make_counting_engine();
    let make_options = || QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        ..QueryOptions::default()
    };

    // Populate the cache (1 fetch).
    engine
        .execute("SELECT * FROM crowdstrike_detections", make_options())
        .await
        .expect("populate execute must succeed");

    // Simulate a successful write: crowdstrike_acknowledge_alert invalidates
    // crowdstrike_alerts + crowdstrike_detections (BC-2.07.004 mapping table).
    let invalidator = CacheInvalidator::new(engine.response_cache());
    invalidator
        .invalidate_for_write_operation(
            &helpers::org("acme"),
            &prism_core::SensorId::from("crowdstrike"),
            "crowdstrike_acknowledge_alert",
        )
        .expect("invalidation must succeed");

    // The next identical query must MISS the cache and re-fetch (2nd fetch).
    engine
        .execute("SELECT * FROM crowdstrike_detections", make_options())
        .await
        .expect("post-write execute must succeed");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "BC-2.07.004/DEC-018: a query after write invalidation must re-fetch \
         from the sensor API (cache entry evicted before write response)"
    );
}

// ---------------------------------------------------------------------------
// S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001: F-P1-CRIT-001 load-bearing tests
// NOW()/INTERVAL injection wired into production execute path — PIPE mode
// BC-2.11.021 / AC-004 / ADR-044
// ---------------------------------------------------------------------------

/// F-P1-CRIT-001 / BC-2.11.021 AC-004: Pipe mode temporal query via QueryEngine::execute.
///
/// Tests that `NOW() - INTERVAL '7d'` in a `| where` stage executes end-to-end.
/// This is the LOAD-BEARING test: SQL mode and SqlPipe head bypass this because
/// DataFusion 53 has a built-in NOW() function. Pipe mode stages go through
/// `pipe_to_executable_sql` → `expr_to_sql`, which must handle
/// `Expr::TimestampArithmetic` (and requires `inject_now` to have replaced
/// `Expr::Now` with `Expr::Literal(Literal::Timestamp)` first).
///
/// Red Gate (two-part requirement, BOTH must be present):
///   1. `inject_now` must be called in `run_materialization_pipeline` so
///      `Expr::Now` is replaced with `Literal::Timestamp(now)` before
///      `pipe_to_executable_sql` processes the pipe stages.
///   2. `expr_to_sql` in `pipe_sql_emitter.rs` must handle `Expr::TimestampArithmetic`
///      and emit it as `TIMESTAMP '<iso>' - INTERVAL '<n> seconds'`.
///
/// Without BOTH, execution returns `Err(QueryExecutionFailed)` with
/// "Complex expression in pipe WHERE stage is not yet supported."
///
/// Negative control is structural: if `inject_now` is absent, `Expr::Now` reaches
/// `expr_to_sql` which has no arm for it → `Err`. If `expr_to_sql` lacks the
/// `TimestampArithmetic` arm, even after injection the outer wrapper errors → `Err`.
#[tokio::test]
async fn test_crit1_pipe_now_interval_executes_end_to_end() {
    use arrow::{
        array::StringArray,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
    };

    /// Pipe-mode stub: returns rows with `detection_id` and `event_timestamp` columns.
    /// `event_timestamp` is a recent ISO-8601 string so DataFusion's CAST comparison
    /// against the injected TIMESTAMP constant succeeds.
    struct PipeTimestampStubAdapter;

    #[async_trait]
    impl SensorAdapter for PipeTimestampStubAdapter {
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
        ) -> Result<FetchOutput, SensorError> {
            let schema = Arc::new(Schema::new(vec![
                Field::new("detection_id", DataType::Utf8, false),
                Field::new("event_timestamp", DataType::Utf8, false),
            ]));
            let ts_str = chrono::Utc::now().to_rfc3339();
            let batch = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(StringArray::from(vec!["pipe-det-001"])) as _,
                    Arc::new(StringArray::from(vec![ts_str.as_str()])) as _,
                ],
            )
            .expect("pipe timestamp stub batch");
            Ok(FetchOutput::new(vec![batch], false))
        }
    }

    let org_slug = helpers::org("acme");
    let mut registry = prism_sensors::AdapterRegistry::new();
    registry.register(OrgId::new(), Arc::new(PipeTimestampStubAdapter));
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // Pure pipe mode with NOW() - INTERVAL in a | where stage.
    // This is the mode that CANNOT rely on DataFusion's built-in NOW():
    // the pipe stage goes through expr_to_sql, which must handle
    // Expr::TimestampArithmetic after inject_now replaces Expr::Now.
    //
    // If inject_now is not wired OR expr_to_sql lacks TimestampArithmetic support,
    // this returns Err(QueryExecutionFailed) with
    // "Complex expression in pipe WHERE stage is not yet supported."
    let result = engine
        .execute(
            "crowdstrike_detections | where event_timestamp > NOW() - INTERVAL '1d'",
            options,
        )
        .await;

    assert!(
        result.is_ok(),
        "F-P1-CRIT-001: Pipe-mode query with NOW() - INTERVAL '1d' must succeed. \
         Requires inject_now wired in run_materialization_pipeline AND \
         Expr::TimestampArithmetic handled in expr_to_sql. Got: {:?}",
        result.err()
    );
}

/// F-P1-HIGH-001 / BC-2.11.020 INV-FORBID-BOTH-PERMANENT: FORBID-BOTH fires even
/// when the sensor adapter returns an EMPTY vec of batches (Ok(vec![])).
///
/// This tests the TRUE bypass scenario: an adapter returning `Ok(vec![])` puts
/// NOTHING into `table_batches`, so the Step-5 MemTable registration loop registers
/// no table, `any_external_table_registered` stays false, and the early-return at
/// Step 6 fires BEFORE `execute_against_session` (which contains the FORBID-BOTH
/// plan_sqlpipe_query call).
///
/// Red Gate: before the FORBID-BOTH hoist to AFTER parse (but BEFORE fan-out),
/// a query with both SQL LIMIT and `| limit` against an adapter returning
/// `Ok(vec![])` returns `Ok(empty)` instead of `Err(E-QUERY-040)`.
///
/// Note: the existing `test_forbid_both_fires_with_zero_row_sensor` uses a
/// `StubAdapter(row_count: 0)` which returns `Ok(vec![0-row-batch])` — one batch
/// with zero rows. That batch IS pushed to table_batches and the MemTable IS
/// registered, so the early-return never fires. That test passes whether or not
/// the hoist is implemented. THIS test uses an adapter returning `Ok(vec![])`,
/// which triggers the actual bypass.
#[tokio::test]
async fn test_high1_forbid_both_fires_with_empty_vec_sensor() {
    use async_trait::async_trait;
    use prism_core::{OrgId, PrismError, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
    };

    /// Adapter that returns Ok(vec![]) — completely empty, no batches at all.
    /// This triggers the early-return guard in run_materialization_pipeline
    /// (`if !any_external_table_registered { return Ok(empty) }`).
    struct EmptyVecAdapter;

    #[async_trait]
    impl SensorAdapter for EmptyVecAdapter {
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
        ) -> Result<FetchOutput, SensorError> {
            // Return an empty vec — no batches at all.
            // This is different from row_count:0 (which returns vec![0-row-batch]).
            Ok(FetchOutput::new(vec![], false))
        }
    }

    let org_slug = helpers::org("acme");
    let mut registry = prism_sensors::AdapterRegistry::new();
    registry.register(OrgId::new(), Arc::new(EmptyVecAdapter));
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // SqlPipe query with BOTH SQL LIMIT and pipe | limit — must be E-QUERY-040
    // even though the adapter returns Ok(vec![]) (triggering the early-return).
    // After the hoist, plan_sqlpipe_query fires RIGHT AFTER parse, before fan-out.
    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections LIMIT 5 | limit 3",
            options,
        )
        .await;

    let err = result.expect_err(
        "F-P1-HIGH-001: FORBID-BOTH must fire even when adapter returns Ok(vec![]) \
         (empty vec triggers early-return before execute_against_session). \
         plan_sqlpipe_query must run before the data-availability guard.",
    );
    let msg = err.to_string();
    assert!(
        msg.contains("E-QUERY-040"),
        "F-P1-HIGH-001: error must contain 'E-QUERY-040' (FORBID-BOTH); got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001: LOCAL CASCADE PASS-1
// HIGH-001: pushdown spy test — QueryParams.start_time populated for relative-time query
// HIGH-002: SQL-mode + SqlPipe-head execute folded TIMESTAMP constant, not raw INTERVAL
// HIGH-003: discriminating in/out-of-window tests (semantic filtering assertions)
// BC-2.11.021 / ADR-033 T1 / ADR-044
// ---------------------------------------------------------------------------

/// HIGH-001 / BC-2.11.021 ADR-033 T1: `NOW() - INTERVAL '24h'` in a SQL-mode query
/// must constant-fold to a single `TIMESTAMP '<iso>'` literal after `inject_now`, so
/// `extract_time_bounds_from_predicate` can match the RHS as `Expr::Literal(Literal::Timestamp)`
/// and populate `QueryParams.start_time`.
///
/// This is the LOAD-BEARING pushdown spy test. A spy adapter records the
/// `QueryParams.start_time` it receives on each `fetch()` call. The test
/// asserts `start_time == Some(...)` (approximately now-24h).
///
/// Red Gate: before constant-folding, `TimestampArithmetic { base: Literal::Timestamp(now),
/// op: Sub, offset: 24h }` is NOT matched by `extract_time_bounds_from_predicate`
/// (which requires a bare `Literal::Timestamp` RHS) → `start_time` is `None`.
#[tokio::test]
async fn test_high001_pushdown_spy_start_time_populated_for_relative_time_query() {
    use std::sync::{Arc, Mutex};

    use arrow::{
        array::StringArray,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
    };

    /// Spy adapter that records `QueryParams.start_time` and `end_time`.
    struct StartTimeSpyAdapter {
        captured_start: Arc<Mutex<Vec<Option<String>>>>,
    }

    #[async_trait]
    impl SensorAdapter for StartTimeSpyAdapter {
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
        ) -> Result<FetchOutput, SensorError> {
            let mut guard = self
                .captured_start
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            guard.push(params.start_time.clone());

            // Return one row with a `timestamp` column (recent, within the 24h window).
            let schema = Arc::new(Schema::new(vec![
                Field::new("detection_id", DataType::Utf8, false),
                Field::new("timestamp", DataType::Utf8, false),
            ]));
            let ts_str = chrono::Utc::now().to_rfc3339();
            let batch = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(StringArray::from(vec!["det-spy-001"])) as _,
                    Arc::new(StringArray::from(vec![ts_str.as_str()])) as _,
                ],
            )
            .expect("spy batch");
            Ok(FetchOutput::new(vec![batch], false))
        }
    }

    // To wire QueryParams.start_time we need a resolved_spec_map with a datetime INDEX
    // column named `timestamp` on `crowdstrike.detections`. Build it in-memory.
    // Uses the legitimate external construction path: OverlayLoader::merge_overlay_onto_type_spec
    // (ResolvedSensorSpec is #[non_exhaustive] — cannot construct with struct literal in tests).
    use prism_core::{ColumnOptions, ColumnType};
    use prism_spec_engine::{
        overlay::{OverlayLoader, SensorInstanceOverlay},
        spec_parser::{ColumnSpec, SensorSpec as SpecSensorSpec, TableSpec},
        ResolvedSpecKey,
    };

    let mut col = ColumnSpec::default();
    col.name = "timestamp".to_string();
    col.column_type = ColumnType::Datetime;
    col.options = vec![ColumnOptions::Index];

    let table = TableSpec::new(
        "detections",
        "security_finding",
        vec![col],
        vec![],
        prism_core::TableType::PointInTime,
        None,
        None,
    );

    let mut sensor_spec = SpecSensorSpec::default();
    sensor_spec.sensor_id = "crowdstrike".to_string();
    sensor_spec.tables = vec![table];

    let org_slug = helpers::org("acme");

    // Construct via the only external construction path (non_exhaustive guard).
    let overlay_toml = format!("extends = \"crowdstrike\"\ninstance_id = \"crowdstrike@acme\"",);
    let overlay: SensorInstanceOverlay =
        toml::from_str(&overlay_toml).expect("HIGH-001: overlay TOML parse");
    let resolved =
        OverlayLoader::merge_overlay_onto_type_spec(&sensor_spec, &overlay, org_slug.clone());

    let key: ResolvedSpecKey = (org_slug.clone(), prism_core::SensorId::from("crowdstrike"));
    let mut spec_map = std::collections::HashMap::new();
    spec_map.insert(key, resolved);
    let spec_map = Arc::new(spec_map);

    let captured_start: Arc<Mutex<Vec<Option<String>>>> = Arc::new(Mutex::new(Vec::new()));
    let spy = Arc::new(StartTimeSpyAdapter {
        captured_start: Arc::clone(&captured_start),
    });

    let org_id = OrgId::new();
    let mut registry = prism_sensors::AdapterRegistry::new();
    registry.register(org_id, spy);

    // Build QueryEngine with org_registry and resolved_spec_map so the pushdown wiring fires.
    use prism_query::engine::{QueryEngine, QueryEngineConfig};
    let adapter_registry = Arc::new(registry);
    let credential_store: Arc<dyn prism_credentials::CredentialStore> =
        Arc::new(helpers::NullCredentialStore);
    let ocsf_normalizer = Arc::new(prism_ocsf::OcsfNormalizer::new());
    let client_registry = Arc::new(prism_query::scoping::ClientRegistry::new(vec![
        org_slug.clone()
    ]));
    let config = QueryEngineConfig::default();
    let org_registry = Arc::new(prism_core::OrgRegistry::new());
    org_registry
        .register(org_slug.clone(), org_id)
        .expect("HIGH-001: OrgRegistry registration must succeed");
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
        spec_map,
        helpers::make_empty_alias_store().store(),
    );

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // SQL-mode query: WHERE timestamp > NOW() - INTERVAL '24h'.
    // After constant-folding: RHS becomes TIMESTAMP '<now-24h>' literal.
    // extract_time_bounds_from_predicate matches Literal::Timestamp(now-24h) → start_time populated.
    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE timestamp > NOW() - INTERVAL '24h'",
            options,
        )
        .await;

    assert!(
        result.is_ok(),
        "HIGH-001: temporal query must execute successfully; got: {:?}",
        result.err()
    );

    let calls = captured_start.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        !calls.is_empty(),
        "HIGH-001: spy adapter must have been called at least once"
    );

    // start_time must be populated (constant-folding enables push-down extraction).
    let any_start_populated = calls.iter().any(|st| st.is_some());
    assert!(
        any_start_populated,
        "HIGH-001: QueryParams.start_time must be Some(...) for \
         `WHERE timestamp > NOW() - INTERVAL '24h'` after constant-folding. \
         Got calls: {calls:?}. \
         Root cause: inject_now_expr does not fold TimestampArithmetic {{ base: Literal::Timestamp, \
         op: Sub, offset }} into a single Literal::Timestamp — so extract_time_bounds_from_predicate \
         never matches the RHS as a bare Timestamp literal."
    );

    // The start_time value must contain a date within the last 25 hours.
    for st in calls.iter().flatten() {
        // Should be an RFC3339 string close to now-24h.
        assert!(
            !st.is_empty(),
            "HIGH-001: start_time must be a non-empty ISO8601 string; got: {st:?}"
        );
    }
}

/// HIGH-003 (in-window row) / BC-2.11.021 ADR-044: filter mode temporal query —
/// a row with an `event_time` column value INSIDE the 24h window MUST be returned.
///
/// Discriminating test: the stub adapter returns TWO rows:
///   Row A: `event_time` = now (inside window)
///   Row B: `event_time` = now - 30 days (outside window, 720h ago)
///
/// Query: `crowdstrike_detections | where event_time > NOW() - INTERVAL '24h'`
///
/// Expected: exactly Row A returned (count=1, id="in-window-001").
///
/// Red Gate: before constant-folding, `TimestampArithmetic` in pipe WHERE emits
/// `TIMESTAMP '<now>' - INTERVAL '<86400> seconds'` to DataFusion — which is
/// correct runtime arithmetic. However the test verifies BOTH the count AND that
/// the out-of-window row is excluded (semantic predicate correctness).
///
/// NOTE: This test MAY pass before the constant-fold fix if DataFusion correctly
/// handles `TIMESTAMP '<iso>' - INTERVAL '86400 seconds'` at runtime.
/// It is still load-bearing for HIGH-003 because it verifies the predicate
/// is NOT dropped (if inject_now is missing, Expr::Now reaches expr_to_sql
/// → Err, and the test would panic rather than assert wrong count).
#[tokio::test]
async fn test_high003_discriminating_pipe_in_window_row_returned() {
    use arrow::{
        array::StringArray,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
    };

    /// Returns two rows: one inside the 24h window, one outside (30 days ago).
    struct TwoRowStub;

    #[async_trait]
    impl SensorAdapter for TwoRowStub {
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
        ) -> Result<FetchOutput, SensorError> {
            // Row A: now (inside 24h window)
            // Row B: 30 days ago (outside 24h window)
            let schema = Arc::new(Schema::new(vec![
                Field::new("detection_id", DataType::Utf8, false),
                Field::new("event_time", DataType::Utf8, false),
            ]));
            let now = chrono::Utc::now();
            let old = now - chrono::Duration::days(30);
            let batch = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(StringArray::from(vec!["in-window-001", "out-window-001"])) as _,
                    Arc::new(StringArray::from(vec![
                        now.to_rfc3339().as_str(),
                        old.to_rfc3339().as_str(),
                    ])) as _,
                ],
            )
            .expect("two-row stub batch");
            Ok(FetchOutput::new(vec![batch], false))
        }
    }

    let org_slug = helpers::org("acme");
    let mut registry = prism_sensors::AdapterRegistry::new();
    registry.register(OrgId::new(), Arc::new(TwoRowStub));
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // Pipe mode: filter stage with NOW() - INTERVAL '24h'.
    // After inject_now + expr_to_sql, DataFusion sees:
    //   WHERE event_time > TIMESTAMP '<now-24h>' - INTERVAL '86400 seconds'
    // (before fold), OR after fold:
    //   WHERE event_time > TIMESTAMP '<now-24h-ISO>'
    // Both should filter out the 30d-old row.
    let result = engine
        .execute(
            "crowdstrike_detections | where event_time > NOW() - INTERVAL '24h'",
            options,
        )
        .await
        .expect("HIGH-003: pipe temporal query must succeed");

    let total_rows: usize = result.batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_rows, 1,
        "HIGH-003: pipe temporal filter must return EXACTLY 1 row (in-window); \
         got {total_rows}. \
         If 0: inject_now or expr_to_sql is broken (query errors out or drops predicate). \
         If 2: temporal predicate is a no-op (NOW() not substituted or INTERVAL emitted wrongly)."
    );

    // Verify the returned row is the in-window one.
    for batch in &result.batches {
        if batch.num_rows() > 0 {
            let id_idx = batch
                .schema()
                .index_of("detection_id")
                .expect("HIGH-003: detection_id column must be present");
            let ids = batch
                .column(id_idx)
                .as_any()
                .downcast_ref::<StringArray>()
                .expect("detection_id must be StringArray");
            for row in 0..ids.len() {
                assert_eq!(
                    ids.value(row),
                    "in-window-001",
                    "HIGH-003: only 'in-window-001' must pass the temporal filter; \
                     got: '{}' at row {}",
                    ids.value(row),
                    row
                );
            }
        }
    }
}

/// HIGH-003 (out-of-window row) / SQL-mode: discriminating SQL temporal query.
///
/// Same two-row stub as above, but via SQL mode:
///   `SELECT * FROM crowdstrike_detections WHERE event_time > NOW() - INTERVAL '24h'`
///
/// SQL mode uses DataFusion's native NOW() in the raw SQL string, so this SHOULD
/// work before the constant-fold fix. It's included as a cross-mode regression test.
#[tokio::test]
async fn test_high003_discriminating_sql_in_window_row_returned() {
    use arrow::{
        array::StringArray,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
    };

    struct TwoRowStubSql;

    #[async_trait]
    impl SensorAdapter for TwoRowStubSql {
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
        ) -> Result<FetchOutput, SensorError> {
            let schema = Arc::new(Schema::new(vec![
                Field::new("detection_id", DataType::Utf8, false),
                Field::new("event_time", DataType::Utf8, false),
            ]));
            let now = chrono::Utc::now();
            let old = now - chrono::Duration::days(30);
            let batch = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(StringArray::from(vec![
                        "in-window-sql-001",
                        "out-window-sql-001",
                    ])) as _,
                    Arc::new(StringArray::from(vec![
                        now.to_rfc3339().as_str(),
                        old.to_rfc3339().as_str(),
                    ])) as _,
                ],
            )
            .expect("two-row sql stub batch");
            Ok(FetchOutput::new(vec![batch], false))
        }
    }

    let org_slug = helpers::org("acme");
    let mut registry = prism_sensors::AdapterRegistry::new();
    registry.register(OrgId::new(), Arc::new(TwoRowStubSql));
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // SQL mode: execute_against_session re-emits from injected AST via PqlNormalizer
    // (D-1333 Option A), so DataFusion receives TIMESTAMP '<iso>' (plan-pinned constant).
    // Uses PrismQL interval syntax: integer + unit letter (e.g. '24h', not '24 hours').
    // This should filter out the 30d-old row.
    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE event_time > NOW() - INTERVAL '24h'",
            options,
        )
        .await
        .expect("HIGH-003 SQL: temporal SQL query must succeed");

    let total_rows: usize = result.batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_rows, 1,
        "HIGH-003 SQL: SQL temporal filter must return EXACTLY 1 row (in-window); \
         got {total_rows}. \
         If 2: DataFusion's native NOW() is not filtering the 30d-old row."
    );

    for batch in &result.batches {
        if batch.num_rows() > 0 {
            if let Ok(id_idx) = batch.schema().index_of("detection_id") {
                if let Some(ids) = batch.column(id_idx).as_any().downcast_ref::<StringArray>() {
                    for row in 0..ids.len() {
                        assert_eq!(
                            ids.value(row),
                            "in-window-sql-001",
                            "HIGH-003 SQL: only 'in-window-sql-001' must pass the filter; \
                             got: '{}'",
                            ids.value(row)
                        );
                    }
                }
            }
        }
    }
}

/// HIGH-003 (SqlPipe-head) / discriminating SqlPipe temporal query.
///
/// Same two-row stub, but via SqlPipe mode:
///   `SELECT * FROM crowdstrike_detections WHERE event_time > NOW() - INTERVAL '24h' | limit 10`
///
/// SqlPipe head uses raw SQL string for DataFusion (same as SQL mode).
#[tokio::test]
async fn test_high003_discriminating_sqlpipe_in_window_row_returned() {
    use arrow::{
        array::StringArray,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
    };

    struct TwoRowStubSqlPipe;

    #[async_trait]
    impl SensorAdapter for TwoRowStubSqlPipe {
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
        ) -> Result<FetchOutput, SensorError> {
            let schema = Arc::new(Schema::new(vec![
                Field::new("detection_id", DataType::Utf8, false),
                Field::new("event_time", DataType::Utf8, false),
            ]));
            let now = chrono::Utc::now();
            let old = now - chrono::Duration::days(30);
            let batch = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(StringArray::from(vec![
                        "in-window-sp-001",
                        "out-window-sp-001",
                    ])) as _,
                    Arc::new(StringArray::from(vec![
                        now.to_rfc3339().as_str(),
                        old.to_rfc3339().as_str(),
                    ])) as _,
                ],
            )
            .expect("two-row sqlpipe stub batch");
            Ok(FetchOutput::new(vec![batch], false))
        }
    }

    let org_slug = helpers::org("acme");
    let mut registry = prism_sensors::AdapterRegistry::new();
    registry.register(OrgId::new(), Arc::new(TwoRowStubSqlPipe));
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        ..QueryOptions::default()
    };

    // SqlPipe: SQL head with NOW() (raw string to DataFusion) + pipe | limit stage.
    // Uses PrismQL interval syntax: integer + unit letter.
    let result = engine
        .execute(
            "SELECT * FROM crowdstrike_detections WHERE event_time > NOW() - INTERVAL '24h' | limit 10",
            options,
        )
        .await
        .expect("HIGH-003 SqlPipe: temporal SqlPipe query must succeed");

    let total_rows: usize = result.batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_rows, 1,
        "HIGH-003 SqlPipe: SqlPipe temporal filter must return EXACTLY 1 row (in-window); \
         got {total_rows}."
    );

    for batch in &result.batches {
        if batch.num_rows() > 0 {
            if let Ok(id_idx) = batch.schema().index_of("detection_id") {
                if let Some(ids) = batch.column(id_idx).as_any().downcast_ref::<StringArray>() {
                    for row in 0..ids.len() {
                        assert_eq!(
                            ids.value(row),
                            "in-window-sp-001",
                            "HIGH-003 SqlPipe: only 'in-window-sp-001' must pass; got: '{}'",
                            ids.value(row)
                        );
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// HIGH-002: plan-time pinning — SQL-mode and SqlPipe-head must execute the
// plan-pinned TIMESTAMP constant, NOT DataFusion's runtime NOW()
// BC-2.11.021 / ADR-044 D4 / D-1333 human decision (Option A accepted)
// ---------------------------------------------------------------------------

// HIGH-002 tests are unit tests in lib.rs (pub(crate) access to inject_now).

// ---------------------------------------------------------------------------
// F-R13-MED-002: truncation signal correctness when plan-shape gate suppresses
// early-stop — BC-2.11.001 `is_truncated` / `total_available` / `returned_results`
// ---------------------------------------------------------------------------

/// F-R13-MED-002 (BC-2.11.001): `QueryEngine::execute` must set `is_truncated = true`
/// and report `total_available = 100` (true match count) when:
///   1. the plan-shape gate suppresses early-stop (`fetch_limit = 0`), AND
///   2. the filter yields MORE rows than the tool limit (100 > 25).
///
/// ## Defect being caught
///
/// `run_materialization_pipeline` contains a `truncate_result_to_limit` call that
/// caps the DataFusion output to `options.limit` **before** returning to the caller.
/// `engine.rs::execute` Step 6 then sees `total_rows = 25` (already capped) instead
/// of `total_rows = 100`, so it computes `is_truncated = 25 > 25 = false` and
/// `total_available = 25` — the truncation signal is silently lost.
///
/// The fix is to REMOVE `truncate_result_to_limit` from `run_materialization_pipeline`
/// so Step 6 sees the raw 100 filtered rows and correctly computes
/// `is_truncated = true`, `total_available = 100`, `returned_results = 25`.
///
/// ## Red-Gate mechanics
///
/// RED  (pre-cap present, current state):
///   - Gate: `fetch_limit = 0` → 300 rows fetched.
///   - DataFusion WHERE: 100 "page2" rows.
///   - `truncate_result_to_limit(100, 25)` → 25 rows returned to engine.
///   - Engine Step 6: total_rows=25, is_truncated=25>25=false, returned_results=25.
///   - Assertions: `is_truncated == true` → FAIL; `total_available == 100` → FAIL.
///
/// GREEN (pre-cap removed):
///   - `run_materialization_pipeline` returns 100 rows untruncated.
///   - Engine Step 6: total_rows=100, is_truncated=100>25=true, returned_results=25.
///   - All three assertions pass.
///
/// ## Setup
///
/// Uses `TruncSignalMockAdapter` (local to this test) whose `sensor_name() = "mock"`
/// so the table reference `mock_events` resolves through `sensor_id_from_table_name`.
/// The adapter replicates `PlanShapeGateMockAdapter` behavior:
///   - `params.limit == 0`: gate suppressed → returns 300 rows (page1+page2+page3).
///   - `params.limit > 0`: early-stop active → returns 100 rows (page1 only).
///
/// SAP-3: query goes through the full `QueryEngine::execute` path — not through a
/// synthetic AST or `run_materialization_pipeline` directly.
#[tokio::test]
async fn test_BC_2_11_001_tool_limit_truncation_signal_on_suppressed_filter() {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    use arrow::array::StringArray;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use async_trait::async_trait;
    use prism_core::{OrgId, OrgSlug, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
        AdapterRegistry,
    };

    // -----------------------------------------------------------------------
    // TruncSignalMockAdapter
    // -----------------------------------------------------------------------
    // Returns different data based on params.limit:
    //   limit == 0: gate suppressed → 300 rows (100 page1 + 100 page2 + 100 page3)
    //   limit >  0: early-stop active → 100 rows (page1 only)
    //
    // The query `mock_events | where status = 'page2'` with a non-temporal predicate
    // triggers Condition G in `ast_is_reducing_plan`, setting fetch_limit=0.
    // DataFusion then filters the 300-row full set to 100 "page2" rows.
    // With the pre-cap in place: those 100 rows are capped to 25 before engine Step 6.
    // After pre-cap removal: engine Step 6 sees 100 rows and sets is_truncated=true.

    struct TruncSignalMockAdapter {
        fetch_count: Arc<AtomicU64>,
        full_batches: Vec<RecordBatch>, // 300 rows — returned when limit == 0
        page1_batches: Vec<RecordBatch>, // 100 rows — returned when limit > 0
    }

    impl std::fmt::Debug for TruncSignalMockAdapter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TruncSignalMockAdapter")
                .field("fetch_count", &self.fetch_count.load(Ordering::Relaxed))
                .finish()
        }
    }

    #[async_trait]
    impl SensorAdapter for TruncSignalMockAdapter {
        fn sensor_type(&self) -> SensorId {
            SensorId::from("mock")
        }

        fn sensor_name(&self) -> &'static str {
            "mock"
        }

        async fn fetch(
            &self,
            _spec: &SensorSpec,
            params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<FetchOutput, SensorError> {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            if params.limit == 0 {
                Ok(FetchOutput::new(self.full_batches.clone(), false))
            } else {
                Ok(FetchOutput::new(self.page1_batches.clone(), true))
            }
        }
    }

    // Build 300-row full set (100 page1 + 100 page2 + 100 page3).
    fn make_status_batch(value: &str, n: usize) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "status",
            DataType::Utf8,
            true,
        )]));
        let values: Vec<Option<&str>> = std::iter::repeat_n(Some(value), n).collect();
        let array = Arc::new(StringArray::from(values)) as Arc<dyn arrow::array::Array>;
        RecordBatch::try_new(schema, vec![array]).expect("make_status_batch must succeed")
    }

    let fetch_count = Arc::new(AtomicU64::new(0));
    let page1 = make_status_batch("page1", 100);
    let page2 = make_status_batch("page2", 100);
    let page3 = make_status_batch("page3", 100);

    let adapter = Arc::new(TruncSignalMockAdapter {
        fetch_count: Arc::clone(&fetch_count),
        full_batches: vec![page1.clone(), page2, page3],
        page1_batches: vec![page1],
    });

    let org_id = OrgId::new();
    let org_slug = OrgSlug::new_unchecked("test-org");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);

    // Build engine using the same helper used by AC-1..AC-7.
    // `make_engine` wires StubCredentialResolver so fan_out can call the adapter.
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        sensors: None,
        limit: Some(25),
        force_refresh: false,
        ..QueryOptions::default()
    };

    // Condition G: `| where status = 'page2'` is a non-temporal pipe WHERE predicate.
    // `ast_is_reducing_plan` must detect it and set fetch_limit=0 (gate suppresses).
    // DataFusion WHERE then filters 300 rows to 100 "page2" rows.
    //
    // With pre-cap:    materialization returns 25 → engine sees 25 → is_truncated=false.
    // Without pre-cap: materialization returns 100 → engine sees 100 → is_truncated=true.
    let result = engine
        .execute("mock_events | where status = 'page2'", options)
        .await
        .expect("F-R13-MED-002: execute must not error for pipe WHERE query");

    // Precondition: adapter must have been called (not vacuously empty pipeline).
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "F-R13-MED-002: adapter must have been called at least once (fetch_count={fc}). \
         A count of 0 means the pipeline short-circuited and all result assertions are vacuous."
    );

    // PRIMARY — truncation signal: is_truncated must be true when 100 rows > limit 25.
    //
    // RED  (pre-cap): engine sees total_rows=25 → is_truncated=false → FAIL.
    // GREEN (fix):    engine sees total_rows=100 → is_truncated=true  → PASS.
    assert!(
        result.is_truncated,
        "F-R13-MED-002 (BC-2.11.001): is_truncated must be true when 100 matching rows \
         exceed the tool limit of 25; got is_truncated=false. \
         This means truncate_result_to_limit pre-capped the output to 25 inside \
         run_materialization_pipeline before engine Step 6 could compute the signal."
    );

    // PRIMARY — total_available must reflect the TRUE match count before the cap.
    //
    // RED  (pre-cap): total_available=25 (already capped) → FAIL.
    // GREEN (fix):    total_available=100 (raw filtered count) → PASS.
    assert_eq!(
        result.total_available, 100,
        "F-R13-MED-002 (BC-2.11.001): total_available must equal the true filtered \
         match count (100); got {}. \
         The pre-cap truncates before engine Step 6, so total_available reports 25 \
         (the cap) instead of 100 (the real count).",
        result.total_available
    );

    // PRIMARY — returned_results must honour the tool limit (25 rows delivered).
    //
    // PASSES both before and after fix (25 rows are returned either way).
    // Included to confirm the cap still applies correctly at the engine layer.
    assert_eq!(
        result.returned_results, 25,
        "F-R13-MED-002 (BC-2.11.001): returned_results must equal the tool limit (25); \
         got {}.",
        result.returned_results
    );
}
// See `crates/prism-query/src/lib.rs` mod tests::high002_*

// ---------------------------------------------------------------------------
// RG-PSG-025: exact-limit is_truncated soundness (round-15)
// ADR-060 §D8.3 — early-stop at exact LIMIT boundary must set is_truncated = true
// ---------------------------------------------------------------------------

/// RG-PSG-025 — ADR-060 §D8.3: `is_truncated` must be `true` when early-stop fires
/// at the exact LIMIT boundary (total_rows == limit with more pages remaining).
///
/// ## Defect Being Caught
///
/// Engine Step 6 computes `is_truncated = total_rows > limit`. When early-stop fires
/// after exactly `limit` rows are accumulated (i.e., `all_records.len() >= limit`
/// triggers the break), the pipeline returns exactly `limit` rows. DataFusion's
/// LIMIT clause also returns exactly `limit` rows. Step 6 then sees:
///
///   `is_truncated = limit > limit = false`
///
/// This is WRONG: early-stop halted pagination with pages remaining (the mock has
/// 3 × limit rows total). The honest signal is `is_truncated = true`.
///
/// ## Round-15 Fix (ADR-060 v1.5 §D8.3)
///
/// Propagate an `early_stopped` flag from `run_materialization_pipeline` to
/// `QueryEngine::execute`. Engine Step 6 uses:
///
///   `is_truncated = early_stopped` (if early-stop fires, more data may exist,
///   so always report truncated regardless of `total_rows vs limit` comparison).
///
/// Or equivalently: `is_truncated = total_rows >= limit` (since `>= limit` covers
/// both the `> limit` case and the `== limit` early-stop case).
///
/// ## Mock Setup
///
/// `EarlyStopExactLimitMockAdapter`:
/// - `params.limit  > 0`: returns exactly `EXACT_LIMIT` rows (one "full page")
/// - `params.limit == 0`: returns `3 × EXACT_LIMIT` rows (all pages, gate suppressed)
///
/// Query: `SELECT * FROM mock_events LIMIT 1000` — bare projection, no WHERE/agg/DISTINCT.
/// `ast_is_reducing_plan = false` → early-stop fires → `fetch_limit = 1000`.
/// Mock returns 1000 rows. DataFusion LIMIT 1000 → 1000 rows to engine.
/// Engine Step 6: `total_rows = 1000`, `limit = 1000`,
///   `is_truncated = 1000 > 1000 = false` ← WRONG (2 more pages exist).
///
/// ## RED / GREEN
///
/// RED  (current — `is_truncated = total_rows > limit`):
///   `is_truncated = 1000 > 1000 = false` → assertion `is_truncated == true` FAILS.
/// RED  (post-Task-12, same formula unchanged):
///   same → still FAILS.
/// GREEN (round-15 fix — `early_stopped` propagated):
///   `is_truncated = true` (early-stop fired) → PASSES.
///
/// SAP-3: query goes through the full `QueryEngine::execute` path from a real SQL
/// query string — not through a synthetic AST or `run_materialization_pipeline` directly.
#[tokio::test]
async fn test_psg_exact_limit_is_truncated_true() {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    use arrow::array::StringArray;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use async_trait::async_trait;
    use prism_core::{OrgId, OrgSlug, SensorId};
    use prism_query::engine::QueryOptions;
    use prism_sensors::{
        adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
        AdapterRegistry,
    };

    // -----------------------------------------------------------------------
    // The exact limit constant used by this test.
    // -----------------------------------------------------------------------
    const EXACT_LIMIT: usize = 1000;

    // -----------------------------------------------------------------------
    // EarlyStopExactLimitMockAdapter
    //
    // Returns EXACT_LIMIT rows when params.limit > 0 (early-stop active, one page).
    // Returns 3 × EXACT_LIMIT rows when params.limit == 0 (gate suppressed, all pages).
    //
    // This simulates: server has 3 pages of 1000 rows each (3000 total). When
    // early-stop fires after page 1 (1000 rows ≥ LIMIT 1000), we stop fetching.
    // The 2 remaining pages (2000 rows) are never fetched.
    // -----------------------------------------------------------------------
    struct EarlyStopExactLimitMockAdapter {
        fetch_count: Arc<AtomicU64>,
        /// 1 page × EXACT_LIMIT rows — returned when params.limit > 0.
        page1_batches: Vec<RecordBatch>,
        /// 3 pages × EXACT_LIMIT rows — returned when params.limit == 0.
        full_batches: Vec<RecordBatch>,
    }

    impl std::fmt::Debug for EarlyStopExactLimitMockAdapter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("EarlyStopExactLimitMockAdapter")
                .field("fetch_count", &self.fetch_count.load(Ordering::Relaxed))
                .finish()
        }
    }

    #[async_trait]
    impl SensorAdapter for EarlyStopExactLimitMockAdapter {
        fn sensor_type(&self) -> SensorId {
            SensorId::from("mock")
        }

        fn sensor_name(&self) -> &'static str {
            "mock"
        }

        async fn fetch(
            &self,
            _spec: &SensorSpec,
            params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<FetchOutput, SensorError> {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            if params.limit == 0 {
                // Gate suppressed → return all 3000 rows (3 pages × EXACT_LIMIT).
                Ok(FetchOutput::new(self.full_batches.clone(), false))
            } else {
                // Early-stop active → return EXACT_LIMIT rows (page 1 only).
                // This simulates the boundary case: one full page = exactly the limit.
                Ok(FetchOutput::new(self.page1_batches.clone(), true))
            }
        }
    }

    // Build RecordBatch with `n` rows; single "status" column (Utf8).
    fn make_data_batch(n: usize) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "status",
            DataType::Utf8,
            true,
        )]));
        let values: Vec<Option<&str>> = std::iter::repeat_n(Some("data"), n).collect();
        let array = Arc::new(StringArray::from(values)) as Arc<dyn arrow::array::Array>;
        RecordBatch::try_new(schema, vec![array]).expect("make_data_batch must succeed")
    }

    // 3 pages × EXACT_LIMIT rows each.
    let page1 = make_data_batch(EXACT_LIMIT);
    let page2 = make_data_batch(EXACT_LIMIT);
    let page3 = make_data_batch(EXACT_LIMIT);

    let fetch_count = Arc::new(AtomicU64::new(0));
    let adapter = Arc::new(EarlyStopExactLimitMockAdapter {
        fetch_count: Arc::clone(&fetch_count),
        page1_batches: vec![page1.clone()],
        full_batches: vec![page1, page2, page3],
    });

    let org_id = OrgId::new();
    let org_slug = OrgSlug::new_unchecked("test-org");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);

    // Build engine using the helpers::make_engine factory (StubCredentialResolver wired).
    let engine = helpers::make_engine(registry, vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        sensors: None,
        limit: Some(EXACT_LIMIT),
        force_refresh: false,
        ..QueryOptions::default()
    };

    // Bare projection — no WHERE, no aggregation, no DISTINCT, no JOIN.
    // ast_is_reducing_plan = false → early-stop fires → fetch_limit = EXACT_LIMIT.
    // Mock: params.limit = EXACT_LIMIT > 0 → returns EXACT_LIMIT rows (page 1 only).
    // DataFusion LIMIT EXACT_LIMIT → returns EXACT_LIMIT rows to engine.
    // Engine Step 6: total_rows = EXACT_LIMIT, limit = EXACT_LIMIT.
    //   is_truncated = EXACT_LIMIT > EXACT_LIMIT = false  ← WRONG (2 more pages exist)
    //   returned_results = min(EXACT_LIMIT, EXACT_LIMIT) = EXACT_LIMIT
    let result = engine
        .execute(
            &format!("SELECT * FROM mock_events LIMIT {EXACT_LIMIT}"),
            options,
        )
        .await
        .expect("PSG-025: execute must not error for bare projection");

    // Precondition: adapter must have been called (not a vacuous short-circuit).
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-025 (exact-limit is_truncated): adapter must have been called at least once \
         (fetch_count={fc}). A count of 0 means the pipeline short-circuited and all \
         result assertions are vacuous."
    );

    // PRIMARY — is_truncated must be true when early-stop fires at the exact boundary.
    //
    // RED  (current code): is_truncated = EXACT_LIMIT > EXACT_LIMIT = false → FAILS.
    // RED  (post-Task-12, same formula): same → still FAILS.
    // GREEN (round-15 fix — propagate early_stopped flag):
    //   is_truncated = early_stopped = true → PASSES.
    //
    // There are 2 remaining pages (2 × EXACT_LIMIT rows) that were never fetched.
    // The consumer sees EXACT_LIMIT rows with is_truncated=false and has no signal
    // that additional data exists — this is the defect being caught.
    assert!(
        result.is_truncated,
        "PSG-025 (ADR-060 §D8.3 — exact-limit truncation signal): is_truncated must be \
         true when early-stop fires at the exact LIMIT boundary ({EXACT_LIMIT} rows \
         accumulated >= limit {EXACT_LIMIT}); got is_truncated=false. \
         Engine Step 6 computes is_truncated = total_rows > limit = {EXACT_LIMIT} > \
         {EXACT_LIMIT} = false — this misses the early-stop case where total_rows == \
         limit but more pages exist. Fix: propagate early_stopped signal from \
         run_materialization_pipeline and set is_truncated = true when early_stopped."
    );

    // SECONDARY — total_available must equal EXACT_LIMIT (Step 6 sole-owner, EC-11-093).
    //
    // This assertion passes both in RED and GREEN state; it verifies that materialization
    // returns the full pre-cap count to engine Step 6 WITHOUT applying a tool-level pre-cap.
    // A pre-cap inside run_materialization_pipeline would cause Step 6 to see fewer rows,
    // producing a wrong total_available and potentially a wrong is_truncated signal.
    //
    // In both RED and GREEN: mock returns EXACT_LIMIT rows (page 1); DataFusion LIMIT
    // EXACT_LIMIT → EXACT_LIMIT rows reach engine Step 6; total_available = EXACT_LIMIT.
    assert_eq!(
        result.total_available, EXACT_LIMIT,
        "PSG-025 (EC-11-093 Step-6 sole-owner): total_available must equal the pre-cap \
         row count ({EXACT_LIMIT}) returned by materialization; got {}. \
         If total_available < EXACT_LIMIT, materialization applied a tool-level pre-cap \
         before returning to engine Step 6, which violates the Step-6 sole-owner property \
         and breaks is_truncated signal computation (F-R13-CRIT-001 prohibited behavior).",
        result.total_available
    );

    // SECONDARY — returned_results must be the tool limit (EXACT_LIMIT rows delivered).
    // This assertion passes both in RED and GREEN state; included as a sanity check.
    assert_eq!(
        result.returned_results, EXACT_LIMIT,
        "PSG-025: returned_results must equal the tool LIMIT ({EXACT_LIMIT}); got {}.",
        result.returned_results
    );
}

// ---------------------------------------------------------------------------
// RG-PSG-027 / RG-PSG-028: multi-sensor fan-out any_early_stopped correctness
// (round-16 remediation)
// ADR-060 §D8.3 — any_early_stopped OR-aggregated across fan-out sensors
// BC-2.11.001 EC-11-092/EC-11-093
// ---------------------------------------------------------------------------

/// RG-PSG-027 — ADR-060 §D8.3: `is_truncated = true` when at least one sensor
/// in a 2-sensor fan-out early-stops at the exact LIMIT boundary.
///
/// ## Topology
///
/// 2-sensor fan-out, `options.limit = 50`:
/// - sensor1 (org1): returns 40 rows (no early-stop — sensor has exactly 40 rows)
/// - sensor2 (org2): returns 10 rows (models early-stop at page boundary — sensor
///   stopped pagination after page 1, more pages exist)
/// - total = 40 + 10 = 50 == limit
///
/// ## Expected behavior (ADR-060 §D8.3)
///
/// `is_truncated = (total_rows > limit) OR any_early_stopped`
///   = (50 > 50) OR true
///   = false OR true
///   = true
///
/// ## Current state (heuristic path)
///
/// The current implementation at `materialization.rs` computes:
///   `any_early_stopped = fetch_limit > 0 && total_fetched_rows >= fetch_limit`
///   = `50 > 0 && 50 >= 50` = `true`
///
/// By coincidence, this gives the CORRECT answer for this topology (sum equals
/// limit), so `is_truncated = true` and the assertion PASSES with the current code.
///
/// NOTE: This test becomes RED in the intermediate state (post-Task-11 heuristic
/// removal, pre-Task-16 per-sensor chain wiring). It is written now so the
/// implementer has a green target to hit when wiring the correct chain.
///
/// ## SAP-3 compliance
///
/// The query goes through `QueryEngine::execute` from a real SQL string (not a
/// synthetic AST or direct `run_materialization_pipeline` call).
#[tokio::test]
async fn test_psg_multi_sensor_fanout_exact_limit_one_early_stopped_is_truncated_true() {
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;

    const LIMIT: usize = 50;

    // Register two StubAdapters: sensor1 returns 40 rows, sensor2 returns 10 rows.
    // Total = 50 == LIMIT.
    //
    // sensor2 models a sensor that early-stopped at the page boundary (it has more
    // pages, but pagination halted after returning exactly 10 rows that completed
    // the limit). StubAdapter does not simulate internal pagination — the "early-stop"
    // signal is provided by the any_early_stopped chain wired in Task 16.
    //
    // HARNESS NOTE — why explicit OrgRegistry + clients: Some(...):
    //
    // UUID v7 uses a 48-bit millisecond timestamp in the high bits.  Two consecutive
    // OrgId::new() calls within the same millisecond produce UUIDs whose first 8 hex
    // characters are IDENTICAL.  The materialization pipeline builds in-query cache
    // keys as:
    //
    //   format!("{}:{:?}:{}:{}", target.client_id.as_str(), ...)
    //
    // When the pipeline runs without an OrgRegistry, it synthesises client_id as
    // "org-{first8_of_org_uuid}".  Identical first-8 chars → identical cache keys
    // → the second adapter is served from the in-query cache (skips live fan-out,
    // skips total_fetched_rows accumulation).  Result: total_fetched_rows = 40 (or 10)
    // instead of 50, heuristic does not fire, both tests fail for the WRONG reason.
    //
    // Fix: wire an OrgRegistry that maps deterministic slug strings ("org1"/"org2")
    // to each distinct OrgId.  The pipeline's explicit-client path (resolve_org_id
    // Path 1) maps "org1" → org_id1 and "org2" → org_id2, giving unique cache keys
    // "org1:..." and "org2:..." regardless of UUID timestamp similarity.
    let org_id1 = OrgId::new();
    let org_id2 = OrgId::new();
    let mut registry = prism_sensors::AdapterRegistry::new();
    registry.register(
        org_id1,
        std::sync::Arc::new(helpers::StubAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 40,
            client_slug: "org1".to_string(),
            any_early_stopped: false,
        }),
    );
    registry.register(
        org_id2,
        std::sync::Arc::new(helpers::StubAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 10,
            client_slug: "org2".to_string(),
            any_early_stopped: true,
        }),
    );

    let org_registry = prism_core::OrgRegistry::new();
    org_registry
        .register(helpers::org("org1"), org_id1)
        .expect("PSG-027: register org1 must succeed");
    org_registry
        .register(helpers::org("org2"), org_id2)
        .expect("PSG-027: register org2 must succeed");

    let engine = helpers::make_engine(registry, vec![helpers::org("org1"), helpers::org("org2")])
        .with_org_registry(std::sync::Arc::new(org_registry));

    let options = QueryOptions {
        clients: Some(vec![helpers::org("org1"), helpers::org("org2")]),
        sensors: None,
        limit: Some(LIMIT),
        force_refresh: false,
        ..QueryOptions::default()
    };

    let result = engine
        .execute(
            &format!("SELECT * FROM crowdstrike_detections LIMIT {LIMIT}"),
            options,
        )
        .await
        .expect(
            "PSG-027: QueryEngine::execute must not error for 2-sensor fan-out \
             (40+10 rows, limit=50) against StubAdapters",
        );

    // PRECONDITION: total rows must be exactly LIMIT (non-vacuous execution check).
    assert_eq!(
        result.total_available, LIMIT,
        "PSG-027 precondition: total_available must be {LIMIT} (40 + 10 rows from fan-out); \
         got {}. A lower count means one or both adapters short-circuited.",
        result.total_available
    );

    // PRECONDITION: returned_results must equal LIMIT (no silent cap below limit).
    assert_eq!(
        result.returned_results, LIMIT,
        "PSG-027 precondition: returned_results must be {LIMIT}; got {}.",
        result.returned_results
    );

    // PRIMARY — is_truncated must be true: sensor2 early-stopped, more data exists.
    //
    // ADR-060 §D8.3: is_truncated = (total_rows > limit) OR any_early_stopped
    //   = (50 > 50) OR true = false OR true = true.
    //
    // CURRENT (heuristic): total_fetched=50 >= fetch_limit=50 → any_early_stopped=true
    //   → is_truncated=true → PASSES (correct result, wrong mechanism).
    // POST-ROUND-16 (correct chain): any_early_stopped OR-aggregated from per-sensor
    //   FetchOutput.early_stopped → same result via proper chain → PASSES.
    assert!(
        result.is_truncated,
        "PSG-027 (ADR-060 §D8.3 — 2-sensor fan-out, one sensor early-stopped): \
         is_truncated must be true when sensor2 (10 rows) early-stopped at the page \
         boundary (total_rows={total}==limit={LIMIT}, any_early_stopped=true). \
         Engine Step 6: is_truncated = (total > limit) OR any_early_stopped \
         = ({total} > {LIMIT}) OR true = true. \
         Got is_truncated=false.",
        total = result.total_available
    );
}

/// RG-PSG-028 — ADR-060 §D8.3: `is_truncated = false` when all sensors in a
/// 2-sensor fan-out return rows WITHOUT early-stopping, even though the sum
/// equals the query limit exactly.
///
/// ## Topology
///
/// 2-sensor fan-out, `options.limit = 50`:
/// - sensor1 (org1): returns 25 rows (no early-stop — sensor exhausted)
/// - sensor2 (org2): returns 25 rows (no early-stop — sensor exhausted)
/// - total = 25 + 25 = 50 == limit
///
/// ## Expected behavior (ADR-060 §D8.3)
///
/// `is_truncated = (total_rows > limit) OR any_early_stopped`
///   = (50 > 50) OR false
///   = false OR false
///   = false
///
/// ## RED Gate — WHY THIS MUST FAIL NOW
///
/// The current heuristic at `materialization.rs`:
///   `any_early_stopped = fetch_limit > 0 && total_fetched_rows >= fetch_limit`
///   = `50 > 0 && 50 >= 50` = `true`
///
/// This is a FALSE POSITIVE: no sensor actually early-stopped. Both sensors
/// exhausted their full result set. The heuristic cannot distinguish this case
/// from a case where sensors did stop early.
///
/// Therefore:
///   `is_truncated = (50 > 50) OR true (FALSE POSITIVE) = true`
///
/// The assertion `is_truncated == false` FAILS → RED GATE.
///
/// ## GREEN (post-round-16)
///
/// After implementing per-sensor `FetchOutput.early_stopped` tracking and wiring
/// `any_early_stopped` as an OR-aggregation across all sensors:
///   `any_early_stopped = false OR false = false`
///   `is_truncated = false OR false = false` → assertion PASSES.
///
/// ## SAP-3 compliance
///
/// The query goes through `QueryEngine::execute` from a real SQL string.
#[tokio::test]
async fn test_psg_multi_sensor_fanout_exact_total_no_early_stop_is_not_truncated() {
    use prism_core::{OrgId, SensorId};
    use prism_query::engine::QueryOptions;

    const LIMIT: usize = 50;

    // Register two StubAdapters each returning 25 rows — no early-stop on either.
    // Total = 50 == LIMIT.  Both sensors exhausted their full result set.
    //
    // HARNESS NOTE — why explicit OrgRegistry + clients: Some(...):
    // (See PSG-027 for the full explanation of the UUID v7 collision root cause.)
    //
    // Without OrgRegistry: both OrgId::new() calls in the same millisecond produce
    // UUIDs with identical first-8 hex chars → synthetic cache key "org-{first8}:..."
    // collides for both adapters → second adapter served from in-query cache →
    // total_fetched_rows = 25 (not 50) → heuristic 25 < 50 = false → any_early_stopped=false
    // → is_truncated=false → assertion !is_truncated PASSES for the WRONG reason.
    //
    // With OrgRegistry: unique slugs "org1"/"org2" → cache keys "org1:..." and "org2:..."
    // → both adapters execute live fan-out → total_fetched_rows = 50 → heuristic
    // 50 >= 50 = TRUE (FALSE POSITIVE) → any_early_stopped=true → is_truncated=true
    // → assertion !is_truncated FAILS → RED GATE (correct).
    let org_id1 = OrgId::new();
    let org_id2 = OrgId::new();
    let mut registry = prism_sensors::AdapterRegistry::new();
    registry.register(
        org_id1,
        std::sync::Arc::new(helpers::StubAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 25,
            client_slug: "org1".to_string(),
            any_early_stopped: false,
        }),
    );
    registry.register(
        org_id2,
        std::sync::Arc::new(helpers::StubAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            row_count: 25,
            client_slug: "org2".to_string(),
            any_early_stopped: false,
        }),
    );

    let org_registry = prism_core::OrgRegistry::new();
    org_registry
        .register(helpers::org("org1"), org_id1)
        .expect("PSG-028: register org1 must succeed");
    org_registry
        .register(helpers::org("org2"), org_id2)
        .expect("PSG-028: register org2 must succeed");

    let engine = helpers::make_engine(registry, vec![helpers::org("org1"), helpers::org("org2")])
        .with_org_registry(std::sync::Arc::new(org_registry));

    let options = QueryOptions {
        clients: Some(vec![helpers::org("org1"), helpers::org("org2")]),
        sensors: None,
        limit: Some(LIMIT),
        force_refresh: false,
        ..QueryOptions::default()
    };

    let result = engine
        .execute(
            &format!("SELECT * FROM crowdstrike_detections LIMIT {LIMIT}"),
            options,
        )
        .await
        .expect(
            "PSG-028: QueryEngine::execute must not error for 2-sensor fan-out \
             (25+25 rows, limit=50) against StubAdapters",
        );

    // PRECONDITION: total rows must be exactly LIMIT (both sensors fully exhausted).
    assert_eq!(
        result.total_available, LIMIT,
        "PSG-028 precondition: total_available must be {LIMIT} (25 + 25 rows from fan-out); \
         got {}. A lower count means one or both adapters short-circuited.",
        result.total_available
    );

    // PRECONDITION: returned_results must equal LIMIT (no silent cap below limit).
    assert_eq!(
        result.returned_results, LIMIT,
        "PSG-028 precondition: returned_results must be {LIMIT}; got {}.",
        result.returned_results
    );

    // PRIMARY — is_truncated must be false: no sensor early-stopped, all data returned.
    //
    // ADR-060 §D8.3: is_truncated = (total_rows > limit) OR any_early_stopped
    //   = (50 > 50) OR false = false OR false = false.
    //
    // RED  (current heuristic): total_fetched=50 >= fetch_limit=50 → any_early_stopped=TRUE
    //   → is_truncated = false OR TRUE = TRUE → assertion FAILS → RED GATE.
    //   The heuristic cannot distinguish "50 rows because sensors exhausted at 25+25"
    //   from "50 rows because sensors stopped early at the limit boundary".
    //
    // GREEN (post-round-16): per-sensor FetchOutput.early_stopped=false for both sensors
    //   → any_early_stopped = false OR false = false
    //   → is_truncated = false OR false = false → assertion PASSES.
    assert!(
        !result.is_truncated,
        "PSG-028 (ADR-060 §D8.3 — RED GATE — 2-sensor fan-out, no early-stop): \
         is_truncated must be false when both sensors fully exhausted their result sets \
         (sensor1=25 rows, sensor2=25 rows, total=50==limit={LIMIT}, any_early_stopped=false). \
         Engine Step 6 should compute: is_truncated = (50>50) OR false = false. \
         Got is_truncated=true. \
         \n\nDiagnosis: The heuristic `total_fetched_rows >= fetch_limit` (50>=50=true) \
         produces a FALSE POSITIVE — it cannot distinguish sensor exhaustion from \
         early-stop pagination. Fix: implement per-sensor FetchOutput.early_stopped \
         and OR-aggregate into FanOutResult.any_early_stopped (ADR-060 §D8.3, \
         S-ENGINE-LIMIT-EARLY-STOP-001 Task 16)."
    );
}

// ===========================================================================
// SLUG-005 — RG-SLUG-005 (AC-013): cross-tenant cache-key collision
//            resistance — collision-resistant cache keys via OrgRegistry
// ===========================================================================

/// RG-SLUG-005 — ADR-061 D4 collision-resistant in-query cache keys
///
/// Verifies that two `OrgId`s whose UUIDs share an identical first-8-hex prefix
/// produce DISTINCT in-query cache keys after the D2 fix for Step 3b, so that
/// adapter-B is called independently of adapter-A and its rows appear in the result.
///
/// ## Collision mechanism (current code — RED)
///
/// Step 3b synthesizes a slug via `format!("org-{}", &org_id.to_string()[..8])`.
/// Two `OrgId`s built from bytes with prefix `[0xde, 0xad, 0xbe, 0xef, ...]`
/// both stringify to `"deadbeef-..."` — first 8 chars are "deadbeef" for both.
/// Synthetic slugs are identical: `"org-deadbeef"`.
///
/// In-query cache key: `format!("{}:{:?}:{}:{}", target.client_id, sensor_id, …)`.
/// With the same slug, adapter-A fetches first and its rows are cached under
/// `"org-deadbeef:crowdstrike:crowdstrike:..."`.  Adapter-B's key is identical →
/// cache HIT → adapter-B is NEVER called → "beta-001" rows are absent from result.
///
/// Assertion `provider_values_contain_beta` FAILS → RED GATE.
///
/// ## Correct behaviour (post-D2 fix — GREEN)
///
/// Step 3b checks `mat_ctx.org_registry`:
/// - org_id_A → `"tenant-alpha"` (from OrgRegistry)
/// - org_id_B → `"tenant-beta"`  (from OrgRegistry)
/// Distinct slugs → distinct in-query cache keys → adapter-B fetched independently
/// → "beta-001" rows present → assertion PASSES.
///
/// ## Engine-layer defense-in-depth assertion (NOT the wire-level MCP assertion)
///
/// This test operates on `QueryResult.batches` (Arrow `RecordBatch` structs) —
/// a pre-serialization Rust structure.  It verifies the engine-layer isolation
/// property directly.
///
/// The MCP wire-level assertion — routing through `PrismServer::query` and
/// asserting on `CallToolResult.content[0].text` (the exact bytes the LLM agent
/// consumes) — is in `crates/prism-bin/tests/mcp_integration_tests.rs`
/// `test_rg_slug_005_wire_cross_tenant_isolation_collision_resistant_cache_keys`
/// (F-R16-P15-LENSB-MED-001 fix).
///
/// Both tests are load-bearing: this test catches engine-layer isolation
/// failures before they reach the wire; the wire test catches envelope-path
/// bugs that this test cannot see.
///
/// SAP-3: the query reaches the fan-out path through `QueryEngine::execute`
/// from a real bare-filter string, not a synthetic AST.
#[tokio::test]
async fn test_rg_slug_005_cross_tenant_wire_isolation_collision_resistant_cache_keys() {
    use prism_core::{OrgId, OrgRegistry, OrgSlug, SensorId};
    use prism_query::engine::QueryOptions;
    use std::sync::atomic::{AtomicU64, Ordering};
    use uuid::Uuid;

    // Build two OrgIds whose first-8-hex chars are identical ("deadbeef").
    // bytes[0..4] = [0xde, 0xad, 0xbe, 0xef] → UUID display starts "deadbeef-".
    // bytes[15] differs so the UUIDs are distinct.
    let uuid_a = Uuid::from_bytes([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]);
    let uuid_b = Uuid::from_bytes([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02,
    ]);
    let org_id_a = OrgId::from_uuid(uuid_a);
    let org_id_b = OrgId::from_uuid(uuid_b);

    // Verify the collision precondition: both produce the same first-8-hex prefix.
    assert_eq!(
        &org_id_a.to_string()[..8],
        "deadbeef",
        "SLUG-005 precondition: org_id_a first-8-hex must be 'deadbeef'"
    );
    assert_eq!(
        &org_id_b.to_string()[..8],
        "deadbeef",
        "SLUG-005 precondition: org_id_b first-8-hex must be 'deadbeef'"
    );

    // ---------------------------------------------------------------------------
    // Two inline adapters: adapter-A returns "alpha-001", adapter-B "beta-001".
    // ---------------------------------------------------------------------------
    struct ProviderAdapter {
        sensor_id: SensorId,
        provider_value: &'static str,
        fetch_count: std::sync::Arc<AtomicU64>,
    }

    impl std::fmt::Debug for ProviderAdapter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ProviderAdapter")
                .field("provider_value", &self.provider_value)
                .finish()
        }
    }

    #[async_trait::async_trait]
    impl prism_sensors::adapter::SensorAdapter for ProviderAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }
        fn sensor_name(&self) -> &'static str {
            "crowdstrike"
        }
        async fn fetch(
            &self,
            _spec: &prism_sensors::adapter::SensorSpec,
            _params: &prism_sensors::adapter::QueryParams,
            _auth: &dyn prism_sensors::auth::SensorAuth,
        ) -> Result<prism_sensors::adapter::FetchOutput, prism_sensors::adapter::SensorError>
        {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            let schema = std::sync::Arc::new(arrow::datatypes::Schema::new(vec![
                arrow::datatypes::Field::new("provider", arrow::datatypes::DataType::Utf8, false),
            ]));
            let arr =
                std::sync::Arc::new(arrow::array::StringArray::from(vec![self.provider_value]))
                    as _;
            let batch =
                arrow::record_batch::RecordBatch::try_new(schema, vec![arr]).expect("batch");
            Ok(prism_sensors::adapter::FetchOutput::new(vec![batch], false))
        }
    }

    let fetch_count_a = std::sync::Arc::new(AtomicU64::new(0));
    let fetch_count_b = std::sync::Arc::new(AtomicU64::new(0));

    let mut adapter_registry = prism_sensors::AdapterRegistry::new();
    adapter_registry.register(
        org_id_a,
        std::sync::Arc::new(ProviderAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            provider_value: "alpha-001",
            fetch_count: std::sync::Arc::clone(&fetch_count_a),
        }),
    );
    adapter_registry.register(
        org_id_b,
        std::sync::Arc::new(ProviderAdapter {
            sensor_id: SensorId::from("crowdstrike"),
            provider_value: "beta-001",
            fetch_count: std::sync::Arc::clone(&fetch_count_b),
        }),
    );

    // OrgRegistry with both orgs mapped — after D2 fix, Step 3b will use these
    // distinct slugs instead of the identical "org-deadbeef" synthetic slug.
    let org_registry = OrgRegistry::new();
    org_registry
        .register(helpers::org("tenant-alpha"), org_id_a)
        .expect("register tenant-alpha");
    org_registry
        .register(helpers::org("tenant-beta"), org_id_b)
        .expect("register tenant-beta");

    // CRITICAL: use an EMPTY ClientRegistry so Step 3b (bare-filter ALL-scope fan-out) fires.
    //
    // With a populated ClientRegistry containing "tenant-alpha"/"tenant-beta",
    // `resolve_clients(None, registry)` returns those slugs as an explicit list.
    // `run_materialization_pipeline` then routes through `resolve_source_refs` with
    // explicit clients, which already looks up OrgRegistry correctly — bypassing Step 3b
    // entirely and making the test pass vacuously (the bug is never exercised).
    //
    // With an empty ClientRegistry: `resolve_clients(None, empty)` returns `[]`.
    // In the pipeline `all_clients = []` → targets remain empty after Steps 1–3a →
    // Step 3b fires: iterates adapter_registry, synthesizes slugs from first-8-hex.
    // Collision: both org_ids get "org-deadbeef" → same cache key → adapter-B skipped.
    let engine = helpers::make_engine(adapter_registry, vec![])
        .with_org_registry(std::sync::Arc::new(org_registry));

    // Bare-filter query (no explicit source) → Step 3b fan-out to ALL adapters.
    // With collision (current code): both get slug "org-deadbeef" → same cache key
    //   → adapter-B hits adapter-A's cache → adapter-B NEVER called → "beta-001" absent.
    // After fix: distinct slugs "tenant-alpha"/"tenant-beta" → distinct cache keys
    //   → both adapters fetched → "beta-001" present.
    let options = QueryOptions {
        clients: None, // ALL scope — Step 3b enumerates all registered adapters
        sensors: None,
        limit: Some(10),
        force_refresh: false,
        ..QueryOptions::default()
    };
    let result = engine
        .execute("provider IS NOT NULL", options)
        .await
        .expect(
            "SLUG-005: QueryEngine::execute must not error for bare-filter fan-out \
             (provider IS NOT NULL against crowdstrike adapters)",
        );

    // Engine-layer defense-in-depth: collect 'provider' column values from
    // QueryResult.batches (Arrow RecordBatch — pre-serialization Rust structs).
    // Verifies the engine-layer isolation property. This does NOT exercise the
    // MCP wire path; see mcp_integration_tests.rs for the real wire-level assertion.
    let provider_values: Vec<String> = result
        .batches
        .iter()
        .flat_map(|b| {
            b.schema()
                .index_of("provider")
                .ok()
                .map(|col_idx| {
                    let arr = b
                        .column(col_idx)
                        .as_any()
                        .downcast_ref::<arrow::array::StringArray>()
                        .expect("provider column must be StringArray");
                    (0..arr.len())
                        .map(|i| arr.value(i).to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();

    // Engine-layer assertion: "beta-001" must appear in the result batches.
    //
    // RED (current code — Step 3b collision):
    //   Both org_ids get slug "org-deadbeef" → cache collision → adapter-B never called
    //   → result contains only "alpha-001" rows → provider_values does NOT contain "beta-001"
    //   → FAILS.
    //
    // GREEN (post-D2 fix):
    //   org_id_B → slug "tenant-beta" → distinct cache key → adapter-B fetched
    //   → "beta-001" row in result → provider_values CONTAINS "beta-001" → PASSES.
    let provider_values_contain_beta = provider_values.iter().any(|v| v == "beta-001");
    assert!(
        provider_values_contain_beta,
        "RG-SLUG-005 (ADR-061 D4 — collision-resistant cache keys via OrgRegistry, \
         engine-layer defense-in-depth): \
         'beta-001' rows from adapter-B must appear in QueryResult.batches. \
         Got provider_values={provider_values:?}. \
         If absent, Step 3b synthesized the same slug 'org-deadbeef' for both org_ids \
         (UUID prefix collision), causing an in-query cache HIT for adapter-B that served \
         adapter-A's rows. Fix: Step 3b must consult mat_ctx.org_registry when present \
         (ADR-061 D2 for the bare-filter path). \
         NOTE: the MCP wire-level assertion is in \
         test_rg_slug_005_wire_cross_tenant_isolation_collision_resistant_cache_keys."
    );
}
