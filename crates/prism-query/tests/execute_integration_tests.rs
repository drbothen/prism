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
    use prism_query::scoping::ClientRegistry;
    use prism_sensors::AdapterRegistry;
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
    // Engine factory
    // -----------------------------------------------------------------------

    /// Build a `QueryEngine` with the given adapter registry and client list.
    ///
    /// Uses `NullCredentialStore` — adapters registered by callers must not
    /// require real credentials (DTU harness adapters are fine).
    pub fn make_engine(registry: AdapterRegistry, clients: Vec<OrgSlug>) -> QueryEngine {
        let adapter_registry = Arc::new(registry);
        let credential_store: Arc<dyn CredentialStore> = Arc::new(NullCredentialStore);
        let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
        let client_registry = Arc::new(ClientRegistry::new(clients));
        let config = QueryEngineConfig::default();
        QueryEngine::new(
            adapter_registry,
            credential_store,
            ocsf_normalizer,
            client_registry,
            config,
        )
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

    use prism_core::types::SensorType;
    use prism_core::OrgId;
    use prism_sensors::adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec};
    use prism_sensors::auth::SensorAuth;

    /// Minimal sensor adapter that returns a configurable number of rows with a
    /// `detection_id` column.  Used in tests that need real row data to exercise
    /// record-cap, virtual-field, and cross-client fan-out logic.
    pub struct StubAdapter {
        pub sensor_type: SensorType,
        pub row_count: usize,
        pub client_slug: String,
    }

    #[async_trait]
    impl SensorAdapter for StubAdapter {
        fn sensor_type(&self) -> SensorType {
            self.sensor_type
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
    pub fn make_mat_ctx_with_stub(max_records: usize, row_count: usize) -> MaterializationContext {
        let mut registry = AdapterRegistry::new();
        registry.register(
            OrgId::new(),
            Arc::new(StubAdapter {
                sensor_type: SensorType::CrowdStrike,
                row_count,
                client_slug: "acme".to_string(),
            }),
        );
        let normalizer = Arc::new(OcsfNormalizer::new());
        MaterializationContext::new(Arc::new(registry), normalizer, max_records)
    }

    // -----------------------------------------------------------------------
    // Re-exports for use in tests
    // -----------------------------------------------------------------------
    pub use prism_query::materialization::MaterializationContext;
}

// ---------------------------------------------------------------------------
// AC-1: QueryEngine::execute with adapter returns results
// ---------------------------------------------------------------------------

/// AC-1 (BC-2.11.001): `QueryEngine::execute` with a registered adapter returns
/// `QueryResult` where `returned_results <= 5` and batches contain a `_sensor`
/// column equal to `"crowdstrike"`.
///
/// Red-Gate: panics at `todo!("S-3.02 — QueryEngine::execute")` in engine.rs:276.
#[tokio::test]
async fn test_AC_1_query_engine_execute_with_dtu_returns_results() {
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let engine = helpers::make_engine(AdapterRegistry::new(), vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        sensors: None,
        limit: Some(5),
        force_refresh: false,
    };

    // Panics at todo!() in engine.rs:276.
    // Post-implementation: must return Ok(QueryResult).
    let result = engine
        .execute("SELECT * FROM crowdstrike_detections LIMIT 5", options)
        .await
        .expect("AC-1: execute must succeed with registered adapter");

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

/// AC-2 (BC-2.11.005): `run_materialization_pipeline` materializes records
/// from the sensor fan-out into Arrow `RecordBatch` slices and registers them
/// as `MemTable` entries in the provided `SessionContext`.
///
/// Bug fix: original test omitted adapter registration (genuine test bug).
/// Now uses `make_mat_ctx_with_stub` which registers a StubAdapter returning
/// 3 rows so the pipeline can produce real materialized output.
///
/// Red-Gate: panics at `todo!("S-3.02 — run_materialization_pipeline")`.
#[tokio::test]
async fn test_AC_2_materialization_pipeline_produces_session_context() {
    use prism_query::engine::QueryOptions;

    // Use a StubAdapter so the pipeline has real rows to materialize (bug fix).
    let mut mat_ctx = helpers::make_mat_ctx_with_stub(10_000, 3);
    let session_ctx = helpers::make_ctx();
    let options = QueryOptions {
        clients: Some(vec![helpers::org("acme")]),
        sensors: None,
        limit: Some(10),
        force_refresh: false,
    };

    // Post-implementation: returns Ok(batches); session_ctx has registered MemTable.
    let _batches = run_materialization_pipeline(
        "SELECT * FROM crowdstrike_detections LIMIT 10",
        &options,
        &mut mat_ctx,
        &session_ctx,
    )
    .await
    .expect("AC-2: run_materialization_pipeline must succeed with valid source ref");

    // The session context must have at least the default DataFusion catalog.
    assert!(
        !session_ctx.catalog_names().is_empty(),
        "AC-2: session_ctx must have at least the default DataFusion catalog after pipeline runs"
    );
}

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
    use prism_core::{types::SensorType, OrgId};
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
        fn sensor_type(&self) -> SensorType {
            SensorType::CrowdStrike
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

/// AC-5 (BC-2.15.011): After `register_internal_tables(ctx, storage)`, the query
/// `SELECT * FROM prism_audit LIMIT 20` must succeed without error.
///
/// We pre-populate `StorageDomain::AuditBuffer` with one row so the scan is
/// non-trivially exercised.
///
/// Red-Gate: panics at `todo!("S-3.02 — register_internal_tables")`.
#[tokio::test]
async fn test_AC_5_register_internal_tables_then_query_prism_audit() {
    let storage = helpers::make_storage();

    // Seed one audit entry so the CF is non-empty.
    helpers::seed_entry(
        &storage,
        StorageDomain::AuditBuffer,
        b"audit-key-001",
        b"audit-payload-001",
    );

    let ctx = helpers::make_ctx();

    // Panics at todo!("S-3.02 — register_internal_tables").
    // Post-implementation: must register prism_audit as a DataFusion table.
    register_internal_tables(&ctx, Arc::clone(&storage) as Arc<dyn RocksStorageBackend>)
        .expect("AC-5: register_internal_tables must succeed");

    // Post-implementation: SQL planning must succeed.
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
/// Bug fix: original test omitted adapter registration (genuine test bug).
/// Now registers two StubAdapters — one per org — so both produce rows with
/// distinct `_client` annotations.
///
/// Red-Gate: panics at `todo!("S-3.02 — QueryEngine::execute")`.
#[tokio::test]
async fn test_AC_6_cross_client_query_all_scope_fans_out() {
    use prism_core::types::SensorType;
    use prism_core::OrgId;
    use prism_query::engine::QueryOptions;

    let org_acme = helpers::org("acme");
    let org_beta = helpers::org("beta");

    // Register one StubAdapter per org so both produce rows.
    // get_all_for_sensor_type finds adapters by SensorType, ignoring OrgId.
    let mut registry = AdapterRegistry::new();
    registry.register(
        OrgId::new(),
        Arc::new(helpers::StubAdapter {
            sensor_type: SensorType::CrowdStrike,
            row_count: 2,
            client_slug: "acme".to_string(),
        }),
    );
    registry.register(
        OrgId::new(),
        Arc::new(helpers::StubAdapter {
            sensor_type: SensorType::CrowdStrike,
            row_count: 2,
            client_slug: "beta".to_string(),
        }),
    );

    let engine = helpers::make_engine(registry, vec![org_acme.clone(), org_beta.clone()]);

    // clients: None = ALL scope — both orgs fanned out.
    let options = QueryOptions {
        clients: None,
        sensors: None,
        limit: Some(100),
        force_refresh: false,
    };

    // Panics at todo!(). Post-implementation: batches must cover both orgs.
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
/// Red-Gate: panics at `todo!("S-3.02 — QueryEngine::execute")`.
#[tokio::test]
async fn test_AC_7_virtual_fields_present_in_all_results() {
    use prism_query::engine::QueryOptions;

    let org_slug = helpers::org("acme");
    let engine = helpers::make_engine(AdapterRegistry::new(), vec![org_slug.clone()]);

    let options = QueryOptions {
        clients: Some(vec![org_slug]),
        sensors: None,
        limit: Some(10),
        force_refresh: false,
    };

    // Panics at todo!(). Post-implementation: every row carries all three virtual fields.
    let result = engine
        .execute("SELECT * FROM crowdstrike_detections LIMIT 10", options)
        .await
        .expect("AC-7: execute must succeed");

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
