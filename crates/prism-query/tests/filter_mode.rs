//! Filter mode end-to-end execution tests for S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 Area D.
//!
//! BC-2.11.023 postcondition AC-011/AC-012: Filter mode must execute end-to-end via
//! `QueryEngine::execute`, not just parse. Tests drive `Ast::Filter` through the full
//! execution pipeline using a `QueryEngine` wired with a test sensor registry.
//!
//! Per SID-1 and F-PASS1-HIGH-1: these tests use a test-only stub adapter that returns
//! pre-seeded rows with a `severity` column. The tests seed rows where SOME match
//! `severity='HIGH'` and some do not, then assert the returned row count equals EXACTLY
//! the matching rows (negative control included). This makes the tests genuinely
//! load-bearing — a regression in filter-mode predicate application would fail them.
//!
//! Implementation note: BC-2.11.023 AC-011 / ADR-046 D4 requires Filter mode to lower
//! the predicate to DataFusion SQL (`SELECT * FROM <table> WHERE <predicate>`) and
//! execute it, not just return all rows. The `Ast::Filter` arm in `execute_against_session`
//! implements this (closed ENRICH-4-C deferred item).

// Test code — allow expect/unwrap per project pattern for integration tests.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    unused_imports
)]

use std::{collections::HashMap, sync::Arc};

use arrow::{
    array::{Array, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use async_trait::async_trait;
use prism_core::{OrgSlug, PrismError, SensorId};
use prism_query::{
    engine::{QueryEngine, QueryEngineConfig, QueryOptions},
    scoping::ClientRegistry,
};
use prism_sensors::{
    adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    AdapterRegistry,
};

// ─── Credential helpers ────────────────────────────────────────────────────────

/// Minimal no-op credential store.
struct NoopCs;

#[async_trait]
impl prism_credentials::CredentialStore for NoopCs {
    async fn get(
        &self,
        _tenant: &prism_core::OrgSlug,
        _sensor: &str,
        _name: &prism_credentials::namespace::CredentialName,
    ) -> Result<Option<secrecy::SecretString>, PrismError> {
        Ok(None)
    }
    async fn set(
        &self,
        _tenant: &prism_core::OrgSlug,
        _sensor: &str,
        _name: &prism_credentials::namespace::CredentialName,
        _value: secrecy::SecretString,
    ) -> Result<(), PrismError> {
        Ok(())
    }
    async fn delete(
        &self,
        _tenant: &prism_core::OrgSlug,
        _sensor: &str,
        _name: &prism_credentials::namespace::CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(false)
    }
    async fn list(
        &self,
        _tenant: &prism_core::OrgSlug,
    ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError> {
        Ok(vec![])
    }
    async fn exists(
        &self,
        _tenant: &prism_core::OrgSlug,
        _sensor: &str,
        _name: &prism_credentials::namespace::CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(false)
    }
}

/// Test stub that satisfies `SensorAuth`.
struct TestStubAuth;
impl SensorAuth for TestStubAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn auth_type_name(&self) -> &'static str {
        "test_stub"
    }
}

/// Stub credential resolver: always returns a no-op auth token.
struct StubCredentialResolver;
impl prism_sensors::CredentialResolver for StubCredentialResolver {
    fn resolve(
        &self,
        _client_id: &str,
        _sensor_id: SensorId,
    ) -> Result<Box<dyn SensorAuth>, SensorError> {
        Ok(Box::new(TestStubAuth))
    }
}

// ─── Stub sensor adapter ─────────────────────────────────────────────────────

/// Stub sensor adapter for filter-mode tests.
///
/// Returns a fixed set of rows with a `severity` column:
/// - 2 rows with severity = 'HIGH'
/// - 3 rows with severity = 'LOW'
///
/// This allows tests to assert that `severity = 'HIGH'` returns exactly 2 rows
/// and that `severity = 'LOW'` returns exactly 3 rows (negative control).
///
/// Sensor ID is configurable so the same struct can serve as "crowdstrike" or
/// any other sensor in different test scenarios.
struct SeverityStubAdapter {
    sensor_id: SensorId,
}

#[async_trait]
impl SensorAdapter for SeverityStubAdapter {
    fn sensor_type(&self) -> SensorId {
        self.sensor_id.clone()
    }

    fn sensor_name(&self) -> &'static str {
        "stub_sensor"
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<FetchOutput, SensorError> {
        // Schema: one `severity` column (Utf8).
        let schema = Arc::new(Schema::new(vec![Field::new(
            "severity",
            DataType::Utf8,
            false,
        )]));

        // Seed rows: 2 HIGH + 3 LOW = 5 total.
        // F-PASS1-HIGH-1: test asserts HIGH count=2 and total row count is verifiable.
        let severities: Vec<&str> = vec!["HIGH", "HIGH", "LOW", "LOW", "LOW"];
        let arr = Arc::new(StringArray::from(severities)) as _;
        let batch =
            RecordBatch::try_new(schema, vec![arr]).expect("stub severity batch must be valid");
        Ok(FetchOutput::new(vec![batch], false, false))
    }
}

// ─── Engine factory ───────────────────────────────────────────────────────────

/// Build a `QueryEngine` wired with `SeverityStubAdapter` registered under `sensor_id`.
///
/// Uses `StubCredentialResolver` so fan_out can reach `SeverityStubAdapter::fetch`
/// without failing on credential resolution (mirrors F-LP1-CRIT-2 pattern).
fn build_engine_with_severity_sensor(sensor_id: &str) -> QueryEngine {
    let sid = SensorId::try_from_str(sensor_id).expect("sensor_id must be valid");
    let mut registry = AdapterRegistry::new();
    // Register globally (OrgId::default()) — no per-org overlay needed for these tests.
    registry.register(
        prism_core::OrgId::default(),
        Arc::new(SeverityStubAdapter { sensor_id: sid }),
    );
    QueryEngine::new(
        Arc::new(registry),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
    .with_credential_resolver(Arc::new(StubCredentialResolver))
}

// ─── AC-011: BC-2.11.023 + BC-2.11.002 — filter mode simple predicate ────────

/// BC-2.11.023 AC-011 / BC-2.11.002: Filter mode end-to-end execution — bare predicate.
///
/// Calls `QueryEngine::execute("severity = 'HIGH'", ...)` against an engine wired with
/// `SeverityStubAdapter` (returns 2×HIGH + 3×LOW rows).
///
/// Asserts:
/// - The call returns `Ok(QueryResult)` (not Err).
/// - The total_available row count equals the MATCHING rows (2, not 5) — proving that
///   the filter predicate was applied by DataFusion, not just returned as-is.
///
/// Negative control: total_available must NOT equal 5 (which would mean no filtering).
///
/// Red Gate (closed): the engine's filter arm used to return ALL rows (no predicate
/// applied). This test detects that regression — if filter mode returns 5 rows instead
/// of 2, the assertion fails.
#[tokio::test]
async fn test_filter_mode_simple_predicate() {
    // "stub_sensor" is the sensor prefix; bare filter fans out to all registered sensors.
    // The stub adapter is registered under "stub" sensor id, but bare predicate fan-out
    // iterates all adapters in the registry.
    let engine = build_engine_with_severity_sensor("stub");
    let options = QueryOptions::default();

    // Bare filter predicate: severity = 'HIGH'.
    // Engine must parse as Ast::Filter, fan out to all sensors, apply WHERE severity='HIGH',
    // and return only matching rows.
    let result = engine.execute("severity = 'HIGH'", options).await;
    assert!(
        result.is_ok(),
        "BC-2.11.023 AC-011: filter-mode bare predicate must execute without error; \
         got: {:?}",
        result
    );

    let qr = result.unwrap();
    let total_rows: usize = qr.batches.iter().map(|b| b.num_rows()).sum();

    // The stub returns 2×HIGH + 3×LOW = 5 rows. After predicate application, only
    // 2 HIGH rows must be present. If total_rows == 5, filtering did not happen.
    assert_eq!(
        total_rows, 2,
        "BC-2.11.023 AC-011: filter predicate severity='HIGH' must return exactly 2 rows \
         (2×HIGH from stub); got {total_rows}. This would indicate filter-mode predicate \
         application is broken (returning all rows instead of filtered rows)."
    );

    // Negative control: assert we did NOT return the 3 LOW rows.
    assert_ne!(
        total_rows, 5,
        "BC-2.11.023 AC-011: negative control — total_rows must NOT be 5 (would mean \
         no filtering occurred)"
    );
    assert_ne!(
        total_rows, 3,
        "BC-2.11.023 AC-011: negative control — total_rows must NOT be 3 (would mean \
         wrong predicate applied)"
    );
}

/// BC-2.11.023 AC-011 / BC-2.11.002: Filter mode end-to-end execution — source-qualified.
///
/// Calls `QueryEngine::execute("crowdstrike.detections | severity = 'HIGH'", ...)`.
/// The engine is wired with `SeverityStubAdapter` registered under `"crowdstrike"`.
///
/// Asserts:
/// - The call returns `Ok(QueryResult)` (not Err).
/// - The total_available row count equals the MATCHING rows (2, not 5) — proving that
///   the filter predicate was applied.
///
/// Red Gate (closed): source-qualified filter mode used to return ALL rows from the
/// sensor without applying the predicate. This test detects that regression.
#[tokio::test]
async fn test_filter_mode_with_source() {
    let engine = build_engine_with_severity_sensor("crowdstrike");
    let options = QueryOptions::default();

    // Source-qualified filter: `sensor.table | predicate`.
    // Engine must parse as Ast::Filter (with source = crowdstrike.detections),
    // fan out to the crowdstrike adapter (returns 2×HIGH + 3×LOW), then apply
    // WHERE severity = 'HIGH' via DataFusion → return 2 rows.
    let result = engine
        .execute("crowdstrike.detections | severity = 'HIGH'", options)
        .await;
    assert!(
        result.is_ok(),
        "BC-2.11.023 AC-011: filter-mode source-qualified query must execute without error; \
         got: {:?}",
        result
    );

    let qr = result.unwrap();
    let total_rows: usize = qr.batches.iter().map(|b| b.num_rows()).sum();

    // Assert exactly 2 matching HIGH rows (not all 5 from stub).
    assert_eq!(
        total_rows, 2,
        "BC-2.11.023 AC-011: source-qualified filter severity='HIGH' must return exactly 2 \
         rows (2×HIGH from crowdstrike stub); got {total_rows}. This would indicate filter \
         predicate application is broken for source-qualified filter mode."
    );

    // Negative control: verify LOW rows were excluded.
    assert_ne!(
        total_rows, 5,
        "BC-2.11.023 AC-011: negative control — must NOT return all 5 rows (no filtering)"
    );
}
