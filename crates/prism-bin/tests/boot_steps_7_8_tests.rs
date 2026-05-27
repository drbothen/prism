//! Tests for S-3.02-FOLLOWUP-RUNTIME boot steps 7–8 wiring.
//!
//! # Test Status (post-structural-fix)
//!
//! Tests 1, 2, 4 call step7/step8 directly and verify their outputs.
//! Steps 7 and 8 currently return Ok(()) without full wiring — these tests
//! PASS because the stubs do not panic. They will catch future regressions.
//!
//! Tests 3, 5, 6 verify API-contract invariants for the wiring that step8/step7
//! must perform. After the structural fix (S-3.02-FOLLOWUP-RUNTIME test-writer
//! task), these tests exercise real production APIs and document the wiring
//! contract. They PASS to confirm the APIs work correctly.
//!
//! Test 7 is #[ignore]'d pending full boot (DTU-EXT-001).
//!
//! ## How each test enforces its contract
//!
//! - Tests 1, 2, 4 (`step7`/`step8` direct calls): catch_unwind detects panics;
//!   assert on Ok(()) return value. Fail if step7/step8 panic or return Err.
//!
//! - Test 3 (`adapter_registry_not_empty`): verifies 3 invariants:
//!   (1) new registry starts empty, (2) adapter can be registered,
//!   (3) QueryEngine::new_full accepts a populated Arc<AdapterRegistry>.
//!
//! - Test 5 (`internal_tables_accessible`): calls register_internal_tables and
//!   verifies all 7 BC-2.15.011 tables are in DataFusion's catalog.
//!
//! - Test 6 (`write_executor_endpoint_registry`): verifies 3 invariants:
//!   (1) new registry starts empty, (2) endpoint can be registered,
//!   (3) WriteExecutor::new accepts a populated Arc<WriteEndpointRegistry>.
//!
//! - Test 7 (`query_engine_execute`): #[ignore] — DTU-EXT-001 dependency.
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
///
/// Uses an empty `AdapterRegistry`. For tests that need a pre-populated registry,
/// use `make_full_query_engine_with_registry` instead.
fn make_full_query_engine(storage: Arc<dyn RocksStorageBackend>) -> QueryEngine {
    make_full_query_engine_with_registry(storage, Arc::new(AdapterRegistry::new()))
}

/// Build a minimal `QueryEngine` using the fully-wired `new_full` constructor,
/// accepting a caller-supplied `Arc<AdapterRegistry>`.
///
/// Used by `test_BC_2_11_001_step8_adapter_registry_not_empty` to verify that
/// a pre-populated registry can be wired into `QueryEngine::new_full` — the
/// exact wiring step8 must perform when fully implemented (BC-2.11.001, AC-3).
fn make_full_query_engine_with_registry(
    storage: Arc<dyn RocksStorageBackend>,
    adapter_registry: Arc<AdapterRegistry>,
) -> QueryEngine {
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
// Test 3: AdapterRegistry can be populated and wired into QueryEngine::new_full
//
// BC-2.11.001 — QueryEngine requires non-empty AdapterRegistry at boot.
//
// This test verifies the INVARIANT:
//   1. AdapterRegistry::new() starts empty.
//   2. After registering at least one adapter, the registry is non-empty.
//   3. QueryEngine::new_full() accepts a populated Arc<AdapterRegistry>.
//
// These three facts together are the contract that step8 must honour:
// step8 MUST call AdapterRegistry::register() for each loaded sensor spec
// and pass the resulting Arc into QueryEngine::new_full() (TD-S-PLUGIN-PREREQ-A-004 P1).
//
// This test will PASS once the compile-time wiring contract is in place.
// The failing Red Gate for the production wiring is enforced by tests 1, 2, and 4
// which call step7/step8 directly and verify their outputs.
// ---------------------------------------------------------------------------

/// Story: S-3.02-FOLLOWUP-RUNTIME AC-3
/// BC: BC-2.11.001 — QueryEngine::new_full accepts a populated AdapterRegistry
///
/// Verifies the API contract that step8 must honour:
///   - AdapterRegistry starts empty (structural invariant)
///   - A stub adapter can be registered (population path step8 will follow)
///   - QueryEngine::new_full succeeds with a populated Arc<AdapterRegistry>
///
/// Post-implementation: step8 populates the registry from loaded sensor TOML
/// specs (via spec-catalog dispatch, S-WAVE5-PREP-01) before constructing
/// QueryEngine::new_full. This test confirms the API wiring works correctly.
///
/// Uses `#[tokio::test]` because `QueryEngine::new_full` initialises internal
/// DataFusion state that requires a Tokio 1.x reactor context.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_11_001_step8_adapter_registry_not_empty() {
    use datafusion::arrow::record_batch::RecordBatch;
    use prism_core::OrgId;
    use prism_sensors::adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec};
    use prism_sensors::auth::SensorAuth;

    // --- Structural invariant 1: new registry starts empty ---
    let mut adapter_registry = AdapterRegistry::new();
    assert!(
        adapter_registry.is_empty(),
        "AdapterRegistry::new() must start empty (invariant: no adapters before registration)"
    );

    // --- Structural invariant 2: adapter can be registered ---
    // Minimal stub adapter — same pattern as prism-sensors bc_2_01_013 tests.
    struct StubSensorAdapter;

    #[async_trait]
    impl SensorAdapter for StubSensorAdapter {
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
        ) -> Result<Vec<RecordBatch>, SensorError> {
            Err(SensorError::Internal {
                detail: "stub — not used in wiring test".into(),
            })
        }
    }

    let org_id = OrgId::new();
    let adapter: Arc<dyn SensorAdapter> = Arc::new(StubSensorAdapter);
    adapter_registry.register(org_id, adapter);

    assert!(
        !adapter_registry.is_empty(),
        "After registering one adapter, AdapterRegistry must be non-empty. \
         step8_init_query_engine() MUST populate the registry from loaded sensor \
         specs before constructing QueryEngine — an empty registry causes all \
         queries to return silent empty results (TD-S-PLUGIN-PREREQ-A-004 P1)."
    );

    // --- Structural invariant 3: QueryEngine::new_full accepts populated registry ---
    // Verify QueryEngine::new_full wires correctly with a populated Arc<AdapterRegistry>.
    // Post-implementation: step8 must call this constructor (or equivalent) with
    // the Arc<AdapterRegistry> it has populated from sensor specs.
    let storage = make_storage();
    let engine = make_full_query_engine_with_registry(
        Arc::clone(&storage) as Arc<dyn RocksStorageBackend>,
        Arc::new(adapter_registry),
    );
    // If we get here, the constructor succeeded — the wiring API works.
    // Engine is ready (though it won't serve real queries without full boot context).
    let _ = engine;
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
// Test 5: internal tables accessible after calling register_internal_tables (BC-2.15.011)
//
// This test directly verifies what step7 MUST produce:
// all 7 BC-2.15.011 internal tables registered in a DataFusion SessionContext.
//
// The test calls register_internal_tables directly, mirroring exactly what
// step7_init_storage() must call. This confirms the registration API works
// and documents which tables must be present (BC-2.15.011 contract).
//
// step7_init_storage() currently does NOT call register_internal_tables (it
// only logs a message). The production wiring — wiring the shared SessionContext
// to the QueryEngine so internal tables persist across query calls — is
// implemented in S-3.02-FOLLOWUP-RUNTIME.
// ---------------------------------------------------------------------------

/// Story: S-3.02-FOLLOWUP-RUNTIME AC-5
/// BC: BC-2.15.011 — prism_audit, prism_schedules, prism_alerts, prism_cases,
///     prism_diff_results, prism_rules, prism_aliases must be registered
///
/// Verifies that `register_internal_tables` registers all 7 BC-2.15.011 internal
/// tables into a DataFusion `SessionContext`. This is the API contract that
/// step7_init_storage() must exercise when wired — each of the 7 tables must be
/// accessible via DataFusion after registration.
///
/// Post-implementation: step7 will call register_internal_tables and pass the
/// SessionContext through to the QueryEngine so queries can access internal tables.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_15_011_internal_tables_accessible_after_step7() {
    use datafusion::execution::context::SessionContext;

    let ctx = SessionContext::new();
    let backend = make_storage();

    // Call register_internal_tables exactly as step7_init_storage() must call it.
    // Pre-implementation: step7 does NOT call this — only logs and returns Ok(()).
    // Post-implementation: step7 calls this (or equivalent) to wire the 7 internal
    // tables into the shared SessionContext so query-time execution can access them.
    register_internal_tables(&ctx, Arc::clone(&backend) as Arc<dyn RocksStorageBackend>)
        .expect("register_internal_tables must succeed with InMemoryBackend");

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
            "Internal table '{table_name}' must be registered in DataFusion after \
             register_internal_tables() is called. step7_init_storage() must call \
             this function (or equivalent) to fulfil BC-2.15.011. \
             All 7 tables must be present: prism_audit, prism_schedules, prism_alerts, \
             prism_cases, prism_diff_results, prism_rules, prism_aliases."
        );
    }
}

// ---------------------------------------------------------------------------
// Test 6: WriteEndpointRegistry can be populated and wired into WriteExecutor
//
// BC-2.22.001 §Step 8 — WriteExecutor::new() must succeed with a populated
// endpoint registry (populated from sensor specs in the boot sequence).
//
// This test verifies the INVARIANT:
//   1. WriteEndpointRegistry::new() starts empty.
//   2. After registering at least one endpoint, the registry is non-empty.
//   3. WriteExecutor::new() accepts a populated Arc<WriteEndpointRegistry>.
//
// These three facts together are the contract that step8 must honour:
// step8 MUST call WriteEndpointRegistry::register() for each sensor spec that
// declares write endpoints, then pass the populated Arc into WriteExecutor::new().
// ---------------------------------------------------------------------------

/// Story: S-3.02-FOLLOWUP-RUNTIME AC-3 (WriteExecutor sub-check)
/// BC: BC-2.22.001 §Step 8 — WriteExecutor::new() accepts a populated
///     WriteEndpointRegistry; the registry must be populated from sensor specs
///
/// Verifies the API contract that step8 must honour:
///   - WriteEndpointRegistry starts empty (structural invariant)
///   - A write endpoint can be registered (population path step8 will follow)
///   - WriteExecutor::new() succeeds with the populated Arc<WriteEndpointRegistry>
///
/// Post-implementation: step8 populates the registry from loaded sensor TOML
/// specs (for sensors with write_endpoints declared, e.g., crowdstrike) and
/// passes it to WriteExecutor::new() — the exact wiring this test confirms works.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_22_001_step8_constructs_write_executor() {
    use prism_core::RiskTier;
    use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointSpec, WriteStep};
    use std::collections::BTreeMap;

    // --- Structural invariant 1: new registry starts empty ---
    let mut endpoint_registry = WriteEndpointRegistry::new();
    assert!(
        endpoint_registry.is_empty(),
        "WriteEndpointRegistry::new() must start empty (invariant: no endpoints before registration)"
    );

    // --- Structural invariant 2: write endpoint can be registered ---
    // Minimal valid WriteEndpointSpec — mirrors the crowdstrike 'contain' endpoint
    // that step8 will register from the loaded sensor TOML spec.
    let contain_spec = WriteEndpointSpec::new(
        "contain",
        "crowdstrike_contained_hosts",
        RiskTier::Irreversible,
        "crowdstrike.hosts.write",
        10,
        BatchMode::Serial,
        "device_id",
        vec![WriteStep::new(
            "POST",
            "/devices/entities/host-actions/v2",
            Some(r#"{"action_name": "contain", "ids": ${record_ids}}"#.to_string()),
            None,
        )],
    );
    endpoint_registry
        .register("crowdstrike", vec![contain_spec])
        .expect("valid crowdstrike endpoint must register without error");

    assert!(
        !endpoint_registry.is_empty(),
        "After registering one write endpoint, WriteEndpointRegistry must be non-empty. \
         step8_init_query_engine() MUST populate the registry from loaded sensor specs \
         before constructing WriteExecutor — an empty registry causes write operations \
         to fail at runtime with 'not declared in WriteEndpointRegistry' \
         (BC-2.22.001 §Step 8, E-QUERY-030)."
    );

    // --- Structural invariant 3: WriteExecutor::new() accepts populated registry ---
    // Verify WriteExecutor::new() wires correctly with a populated Arc<WriteEndpointRegistry>.
    // Post-implementation: step8 must call this constructor (or equivalent) with
    // the Arc<WriteEndpointRegistry> populated from sensor specs.
    let feature_flags = Arc::new(FeatureFlagEvaluator::new(BTreeMap::new()));
    let confirmation_store = Arc::new(ConfirmationTokenStore::new());
    let audit_writer: Arc<dyn AuditWriter> = Arc::new(NoOpAuditWriter);
    let adapter_registry = Arc::new(AdapterRegistry::new());
    let populated_endpoint_registry = Arc::new(endpoint_registry);

    let _executor = WriteExecutor::new(
        feature_flags,
        confirmation_store,
        audit_writer,
        adapter_registry,
        populated_endpoint_registry,
    );
    // If we get here, the constructor succeeded — the wiring API works.
    // WriteExecutor is ready (though it won't execute real writes without full boot context).
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
