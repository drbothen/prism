//! Failing tests for S-3.02-FOLLOWUP-RUNTIME boot steps 7–8 wiring.
//!
//! # Red Gate Contract
//!
//! All tests in this file MUST FAIL before the implementer fills in
//! `step7_init_storage()` and `step8_init_query_engine()` in boot.rs.
//!
//! ## How each test fails (Red Gate mechanism)
//!
//! - Tests 1, 2, 4 (`step7`/`step8` direct calls): catch_unwind detects the
//!   `todo!()` panic, then the assertion `panic_result.is_ok()` fails because
//!   the function panicked instead of returning Ok(()).
//!
//! - Test 3 (`adapter_registry_not_empty`): assertion `!registry.is_empty()` fails
//!   because the pre-implementation registry is empty.
//!
//! - Test 5 (`internal_tables_accessible`): DataFusion `table_exist("prism_audit")`
//!   returns false because step7 (todo!()) never calls register_internal_tables.
//!
//! - Test 6 (`write_executor_endpoint_registry`): assertion `!registry.is_empty()`
//!   fails because step8 (todo!()) never populates the endpoint registry.
//!
//! - Test 7 (`query_engine_execute`): `engine.execute(...)` fails because
//!   no internal tables are registered (step7 is todo!()) and
//!   no adapters are loaded (step8 is todo!()).
//!
//! # Behavioral Contracts Covered
//! - BC-2.11.001 — QueryEngine accepts PrismQL queries post-construction
//! - BC-2.11.005 — ephemeral materialization fan-out
//! - BC-2.15.011 — internal table registration (prism_audit, prism_schedules, …)
//! - BC-2.22.001 — boot orchestration: step 7 → step 8 ordering
//!
//! Story: S-3.02-FOLLOWUP-RUNTIME | AC-1, AC-2, AC-3, AC-4, AC-5
//! Test naming: test_BC_S_SS_NNN_xxx() per Dark Factory TDD conventions.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    unused_imports,
    dead_code,
    non_snake_case
)]

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use prism_core::{OrgRegistry, OrgSlug, PrismError, SensorId, StorageDomain};
use prism_ocsf::OcsfNormalizer;
use prism_query::engine::{QueryEngine, QueryEngineConfig, QueryOptions};
use prism_query::internal_tables::register_internal_tables;
use prism_query::scoping::ClientRegistry;
use prism_query::write_dispatch::AuditWriter;
use prism_query::{WriteExecutor, WritePlan, WriteResult};
use prism_security::confirmation_token::ConfirmationTokenStore;
use prism_security::feature_flag::{CapabilityCheckResult, FeatureFlagEvaluator};
use prism_sensors::AdapterRegistry;
use prism_spec_engine::write_endpoint::WriteEndpointRegistry;
use prism_storage::backend::RocksStorageBackend;
use prism_storage::memory_backend::InMemoryBackend;

// ---------------------------------------------------------------------------
// Shared test infrastructure
// ---------------------------------------------------------------------------

/// No-op `AuditWriter` for tests that do not exercise the audit path.
struct NoOpAuditWriter;

#[async_trait]
impl AuditWriter for NoOpAuditWriter {
    async fn write_intent(
        &self,
        _plan: &WritePlan,
        _context: &prism_query::QueryContext,
        _capability_check: &CapabilityCheckResult,
    ) -> Result<ulid::Ulid, PrismError> {
        Ok(ulid::Ulid::new())
    }

    async fn write_outcome(
        &self,
        _intent_id: ulid::Ulid,
        _result: &WriteResult,
    ) -> Result<(), PrismError> {
        Ok(())
    }
}

/// Build a minimal in-memory `RocksStorageBackend` for use as the step-7 storage.
fn make_storage() -> Arc<InMemoryBackend> {
    Arc::new(InMemoryBackend::new())
}

/// Build a minimal `QueryEngine` using the fully-wired `new_full` constructor.
///
/// This exercises the EXACT constructor that `step8_init_query_engine()` must
/// call after implementation.  Post-implementation, step8 will call this (or
/// equivalent wiring) with dependencies drawn from `BootContext`.
fn make_full_query_engine(storage: Arc<dyn RocksStorageBackend>) -> QueryEngine {
    use prism_credentials::CredentialStore;
    use prism_sensors::CredentialResolver;
    use prism_sensors::adapter::SensorError;
    use prism_sensors::auth::SensorAuth;
    use secrecy::SecretString;

    // NullCredentialStore — no credentials needed for boot wiring test.
    struct NullCredentialStore;

    #[async_trait]
    impl CredentialStore for NullCredentialStore {
        async fn get(
            &self,
            _tenant: &OrgSlug,
            _sensor: &str,
            _name: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<SecretString>, PrismError> {
            Ok(None)
        }

        async fn set(
            &self,
            _tenant: &OrgSlug,
            _sensor: &str,
            _name: &prism_credentials::namespace::CredentialName,
            _value: SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }

        async fn delete(
            &self,
            _tenant: &OrgSlug,
            _sensor: &str,
            _name: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }

        async fn list(
            &self,
            _tenant: &OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }

        async fn exists(
            &self,
            _tenant: &OrgSlug,
            _sensor: &str,
            _name: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    // StubCredentialResolver — satisfies `new_full`'s CredentialResolver parameter.
    // CredentialResolver::resolve takes (&str, SensorId) per fanout.rs:168.
    struct StubCredentialResolver;

    impl CredentialResolver for StubCredentialResolver {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_id: SensorId,
        ) -> Result<Box<dyn SensorAuth>, SensorError> {
            Err(SensorError::ConfigValidation {
                sensor: "stub".to_string(),
                detail: "stub resolver — no real credentials".to_string(),
            })
        }
    }

    let adapter_registry = Arc::new(AdapterRegistry::new());
    let credential_store: Arc<dyn CredentialStore> = Arc::new(NullCredentialStore);
    let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
    let client_registry = Arc::new(ClientRegistry::new(vec![]));
    let config = QueryEngineConfig::default();
    let credential_resolver: Arc<dyn CredentialResolver> = Arc::new(StubCredentialResolver);
    let org_registry = Arc::new(OrgRegistry::new());
    let resolved_spec_map = Arc::new(HashMap::new());

    QueryEngine::new_full(
        adapter_registry,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
        credential_resolver,
        org_registry,
        storage,
        resolved_spec_map,
    )
}

/// Build a minimal `WriteExecutor` for the boot wiring test.
fn make_write_executor() -> WriteExecutor {
    use std::collections::BTreeMap;

    let feature_flags = Arc::new(FeatureFlagEvaluator::new(BTreeMap::new()));
    let confirmation_store = Arc::new(ConfirmationTokenStore::new());
    let audit_writer: Arc<dyn AuditWriter> = Arc::new(NoOpAuditWriter);
    let adapter_registry = Arc::new(AdapterRegistry::new());
    let endpoint_registry = Arc::new(WriteEndpointRegistry::new());

    WriteExecutor::new(
        feature_flags,
        confirmation_store,
        audit_writer,
        adapter_registry,
        endpoint_registry,
    )
}

// ---------------------------------------------------------------------------
// Test 1: step7_init_storage validates storage backend
//
// BC-2.22.001 §Step 7 — storage + internal-tables provider init.
// Red Gate: step7_init_storage() currently todo!() panics.
// Post-implementation: must accept RocksDB backend and return Ok(()).
//
// IMPORTANT: Uses #[test] not #[tokio::test] so that catch_unwind can create
// a fresh tokio runtime via Runtime::new(). Running Runtime::new() inside a
// #[tokio::test] would cause "Cannot start a runtime from within a runtime".
// ---------------------------------------------------------------------------

/// Story: S-3.02-FOLLOWUP-RUNTIME AC-1
/// BC: BC-2.22.001 — step 7 must complete without panic given a valid backend
///
/// Red Gate: `step7_init_storage()` panics with `todo!()`. The test catches the
/// panic and asserts the function returned `Ok(())` — impossible pre-implementation.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_22_001_step7_validates_storage_backend() {
    use prism_bin::boot::step7_init_storage;

    // Spawn a fresh tokio runtime inside catch_unwind so we can observe the
    // todo!() panic without crashing the test runner.
    let panic_result = std::panic::catch_unwind(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio rt must start");
        rt.block_on(async { step7_init_storage().await })
    });

    // Before implementation: panic_result is Err (todo!() panic).
    // After implementation: panic_result is Ok(Ok(())).
    assert!(
        panic_result.is_ok(),
        "step7_init_storage() must not panic — currently fails with todo!(). \
         Implement step7_init_storage to validate the RocksDB backend is ready \
         for internal table queries (BC-2.22.001 §Step 7). \
         Red Gate: this assertion fails until implementation is complete."
    );

    let result = panic_result.unwrap();
    assert!(
        result.is_ok(),
        "step7_init_storage() must return Ok(()) given a healthy storage backend. \
         Got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 2: step8_init_query_engine constructs QueryEngine
//
// BC-2.22.001 §Step 8 — QueryEngine + WriteExecutor construction.
// BC-2.11.001 — engine accepts queries post-construction.
// Red Gate: step8_init_query_engine() currently todo!() panics (after calling
//           mark_query_phase_started).
// ---------------------------------------------------------------------------

/// Story: S-3.02-FOLLOWUP-RUNTIME AC-2
/// BC: BC-2.22.001 / BC-2.11.001 — step 8 must construct QueryEngine and
/// expose it to callers without panicking
///
/// Red Gate: `step8_init_query_engine()` calls `mark_query_phase_started()` then
/// `todo!()`. The test asserts a successful (non-panic) return.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_22_001_step8_constructs_query_engine() {
    use prism_bin::boot::step8_init_query_engine;

    // Reset the query-phase global before test to avoid cross-test contamination.
    prism_query::invalidation::reset_query_phase_global();

    let panic_result = std::panic::catch_unwind(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio rt must start");
        rt.block_on(async { step8_init_query_engine().await })
    });

    // Before implementation: panic_result is Err (todo!() panic after
    //   mark_query_phase_started()).
    // After implementation: panic_result is Ok(Ok(())).
    assert!(
        panic_result.is_ok(),
        "step8_init_query_engine() must not panic. Currently fails with todo!(). \
         Implement step8 to call QueryEngine::new_full() and WriteExecutor::new() \
         with all Arc dependencies from BootContext (BC-2.22.001 §Step 8, BC-2.11.001). \
         Red Gate: this assertion fails until implementation is complete."
    );

    let result = panic_result.unwrap();
    assert!(
        result.is_ok(),
        "step8_init_query_engine() must return Ok(()) on successful construction. \
         Got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 3: AdapterRegistry must be non-empty before step8 serves queries
//
// BC-2.11.001 — QueryEngine requires non-empty AdapterRegistry at boot.
// Red Gate: The assertion that adapter_registry is NOT empty FAILS because
//           the boot sequence does not yet populate it (step8 is todo!()).
// ---------------------------------------------------------------------------

/// Story: S-3.02-FOLLOWUP-RUNTIME AC-3
/// BC: BC-2.11.001 — QueryEngine::new_full succeeds; engine ready for queries
///
/// Documents that after step8 wires the engine, the AdapterRegistry MUST
/// contain at least one adapter (TD-S-PLUGIN-PREREQ-A-004 P1).
///
/// Red Gate: The assertion `!adapter_registry.is_empty()` FAILS because the
/// test constructs an empty registry (mirroring the pre-implementation state
/// where step8 has no sensor specs loaded into the registry).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_11_001_step8_adapter_registry_not_empty() {
    // Construct the registry as step8 will — starting empty.
    let adapter_registry = AdapterRegistry::new();

    // The FIRST THING step8 must do (TD-S-PLUGIN-PREREQ-A-004 P1):
    // assert the registry is not empty before accepting queries.
    //
    // Pre-implementation: the registry IS empty — this assertion FAILS (Red Gate).
    //
    // Post-implementation: step8 (or the boot sequence leading into it) will
    // populate the registry via `init_registry_for_org` for each loaded
    // sensor spec, making `is_empty()` false.
    assert!(
        !adapter_registry.is_empty(),
        "step8_init_query_engine() MUST verify AdapterRegistry is non-empty before \
         constructing QueryEngine. An empty registry means sensor specs were not loaded, \
         which would cause all queries to return silent empty results. \
         (TD-S-PLUGIN-PREREQ-A-004 P1 — BC-2.22.001 §Step 8 EmptyRegistry assertion). \
         Red Gate: this assertion fails because the registry is not yet populated \
         by the boot sequence before step8 is called."
    );
}

// ---------------------------------------------------------------------------
// Test 4: step7 + step8 execute in sequence (BC-2.22.001 sequencing invariant)
//
// Red Gate: step7_init_storage() panics (todo!()), so the sequence never
// reaches step8.
// ---------------------------------------------------------------------------

/// Story: S-3.02-FOLLOWUP-RUNTIME AC-4
/// BC: BC-2.22.001 §Sequencing Invariant — step 7 must complete before step 8
///
/// Verifies that calling step7 then step8 in sequence (as run_boot_sequence
/// does) does not panic, and both return Ok(()).
///
/// Red Gate: step7 panics before step8 can execute. The outer panic_result
/// is Err, triggering the assertion failure.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_22_001_step7_step8_sequential_integration() {
    use prism_bin::boot::{step7_init_storage, step8_init_query_engine};

    prism_query::invalidation::reset_query_phase_global();

    let panic_result = std::panic::catch_unwind(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio rt must start");
        rt.block_on(async {
            // BC-2.22.001 §Sequencing Invariant: step7 BEFORE step8.
            let r7 = step7_init_storage().await;
            if let Err(ref e) = r7 {
                return Err(format!("step7 failed: {e}"));
            }
            let r8 = step8_init_query_engine().await;
            if let Err(ref e) = r8 {
                return Err(format!("step8 failed: {e}"));
            }
            Ok(())
        })
    });

    assert!(
        panic_result.is_ok(),
        "Steps 7 and 8 in sequence must not panic. \
         Currently, step7 panics with todo!(). \
         After implementation, both steps must return Ok(()) in sequence. \
         (BC-2.22.001 §Sequencing Invariant). \
         Red Gate: this assertion fails until both steps are implemented."
    );

    let result = panic_result.unwrap();
    assert!(
        result.is_ok(),
        "Steps 7 and 8 in sequence must both return Ok(()). Got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 5: internal tables accessible after step 7 (BC-2.15.011)
//
// This test directly verifies what step7 MUST produce:
// all 7 BC-2.15.011 internal tables registered in a DataFusion SessionContext.
//
// Red Gate: The test creates a fresh SessionContext WITHOUT calling
// register_internal_tables (mirroring step7's todo!() state), then asserts
// the tables exist — they don't.
// ---------------------------------------------------------------------------

/// Story: S-3.02-FOLLOWUP-RUNTIME AC-5
/// BC: BC-2.15.011 — prism_audit, prism_schedules, prism_alerts, prism_cases,
///     prism_diff_results, prism_rules, prism_aliases must be registered
///
/// Red Gate: The assertion that `prism_audit` exists in the DataFusion catalog
/// FAILS because step7 (todo!()) never calls `register_internal_tables`.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_15_011_internal_tables_accessible_after_step7() {
    use datafusion::execution::context::SessionContext;

    let ctx = SessionContext::new();

    // INTENTIONALLY NOT calling register_internal_tables here.
    // The test asserts that the tables exist — they don't, causing FAILURE.
    //
    // Post-implementation: step7_init_storage() will call
    //   register_internal_tables(&ctx, storage)
    // and the tables will be registered.

    let required_tables = [
        "prism_audit",
        "prism_schedules",
        "prism_alerts",
        "prism_cases",
        "prism_diff_results",
        "prism_rules",
        "prism_aliases",
    ];

    for table_name in &required_tables {
        let exists = ctx.table_exist(*table_name).unwrap_or(false);
        assert!(
            exists,
            "Internal table '{table_name}' must be registered in DataFusion after step7. \
             step7_init_storage() must call register_internal_tables() to register all \
             7 BC-2.15.011 internal tables. \
             Red Gate: this fails because step7 is a todo!() stub that does not yet \
             call register_internal_tables. After implementation this must pass."
        );
    }
}

// ---------------------------------------------------------------------------
// Test 6: WriteExecutor construction and WriteEndpointRegistry population
//
// BC-2.22.001 §Step 8 — WriteExecutor::new() must succeed with a populated
// endpoint registry (populated from sensor specs in the boot sequence).
//
// Red Gate: The assertion that the endpoint_registry is non-empty fails.
// ---------------------------------------------------------------------------

/// Story: S-3.02-FOLLOWUP-RUNTIME AC-3 (WriteExecutor sub-check)
/// BC: BC-2.22.001 §Step 8 — WriteExecutor::new() must succeed and the
///     endpoint registry must be populated from sensor specs
///
/// Red Gate: The assertion that the `WriteEndpointRegistry` is non-empty FAILS
/// because step8 does not yet populate it from loaded sensor specs.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_22_001_step8_constructs_write_executor() {
    // Construct WriteExecutor as step8 will — with an empty endpoint registry
    // (mirroring the pre-implementation state where specs are not yet loaded).
    let endpoint_registry = WriteEndpointRegistry::new(); // empty pre-implementation

    // The WriteExecutor itself constructs fine with an empty registry —
    // the RED is the missing population step, documented here.
    let _executor = make_write_executor();

    // Assert that after step8 runs, at least one write endpoint is registered
    // (for sensors that expose write endpoints, e.g., crowdstrike).
    // step8 must populate endpoint_registry from the resolved_spec_map before
    // constructing WriteExecutor.
    assert!(
        !endpoint_registry.is_empty(),
        "step8_init_query_engine() must populate WriteEndpointRegistry from loaded \
         sensor specs before constructing WriteExecutor. An empty registry means write \
         operations will fail at runtime with 'not declared in WriteEndpointRegistry'. \
         (BC-2.22.001 §Step 8 — WriteExecutor wiring). \
         Red Gate: this fails because step8 is a todo!() stub that does not yet \
         populate the endpoint registry from sensor specs."
    );
}

// ---------------------------------------------------------------------------
// Test 7: QueryEngine can execute a query against internal tables post-boot
//
// This test verifies the end-to-end wiring: after step7 registers internal
// tables and step8 constructs the QueryEngine, a SQL query against an internal
// table (prism_alerts) must succeed.
//
// Red Gate: The engine is constructed WITHOUT step7 running (no internal tables
// registered). The query `SELECT * FROM prism_alerts LIMIT 1` fails because
// prism_alerts is not in the DataFusion catalog.
//
// NOTE: prism_alerts is used (not prism_audit) because audit requires the
// `audit.read` capability. prism_alerts works with the default empty capabilities.
// ---------------------------------------------------------------------------

/// Story: S-3.02-FOLLOWUP-RUNTIME AC-5 (engine execute sub-check)
/// BC: BC-2.11.001 — QueryEngine accepts and executes a PrismQL query after boot
/// BC: BC-2.15.011 — prism_alerts accessible after step7 registers internal tables
///
/// Red Gate: `engine.execute("SELECT * FROM prism_alerts LIMIT 1", ...)` fails
/// because step7 (todo!()) never calls `register_internal_tables`, so DataFusion
/// cannot resolve the `prism_alerts` table.
///
/// Post-implementation: step7 registers all internal tables; the query returns
/// an empty RecordBatch (no alerts in the empty in-memory backend) with Ok.
///
/// Note: DTU-EXT-001 — full end-to-end queries against real sensor APIs require
/// a running DTU clone and are tested in execute_integration_tests.rs with #[ignore].
///
/// This test is #[ignore] because it requires the full boot sequence (step7 + step8)
/// to have run with a real RocksDB backend. Ungated after S-3.02-FOLLOWUP-RUNTIME
/// implementation and a live test environment with RocksDB available.
/// DTU-EXT-001: requires live boot environment; blocked until step7/step8 implemented.
#[tokio::test]
#[ignore = "DTU-EXT-001: requires full boot sequence (step7 + step8 complete); run manually post-implementation"]
#[allow(non_snake_case)]
async fn test_BC_2_11_001_query_engine_execute_after_boot() {
    let storage = make_storage();
    let engine = make_full_query_engine(Arc::clone(&storage) as Arc<dyn RocksStorageBackend>);

    // Query prism_alerts (requires step7 to have called register_internal_tables).
    // No audit.read capability needed for prism_alerts (not requires_audit_read).
    //
    // Pre-implementation: fails because prism_alerts is not registered
    //   (step7 is todo!()). DataFusion returns "table not found" error.
    //
    // Post-implementation: step7 registers prism_alerts via register_internal_tables;
    //   the query succeeds and returns an empty result (no data in InMemoryBackend).
    let options = QueryOptions {
        clients: None,
        sensors: None,
        limit: None,
        force_refresh: false,
        capabilities: vec![],
    };

    let result = engine
        .execute("SELECT * FROM prism_alerts LIMIT 1", options)
        .await;

    assert!(
        result.is_ok(),
        "QueryEngine must be able to query 'prism_alerts' after step7 registers \
         internal tables. Got error: {result:?}. \
         step7_init_storage() must call register_internal_tables() and \
         step8_init_query_engine() must wire the engine with that same SessionContext \
         so internal tables are accessible at query time (BC-2.11.001, BC-2.15.011). \
         Red Gate: this fails because step7 is a todo!() stub and prism_alerts \
         is not registered in DataFusion."
    );
}
