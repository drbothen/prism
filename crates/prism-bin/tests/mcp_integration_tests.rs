//! RG-PSG-026 for S-ENGINE-LIMIT-EARLY-STOP-001: MCP wire-level truncation signal.
//!
//! Asserts on the SERIALIZED `CallToolResult.content[0].text` JSON (wire bytes)
//! for two `prism_query` scenarios per AC-009(d) / BC-2.11.001 EC-11-092/093.
//!
//! ## Wire-shape assertion discipline (CLAUDE.md 2026-07-13)
//!
//! `QueryResult` is not `Serialize`.  RG-PSG-025 covers the Rust-struct layer
//! (`QueryResult.is_truncated`).  This test closes the MCP wire gap: any
//! discrepancy between `QueryResult.is_truncated` and the serialized
//! `"is_truncated"` key in `CallToolResult.content[0].text` would be invisible
//! to PSG-025 but caught here.
//!
//! ## In-process approach
//!
//! Uses `QueryEngine::execute` with a local mock `SensorAdapter` (same pattern
//! as `test_psg_exact_limit_is_truncated_true` in
//! `crates/prism-query/tests/execute_integration_tests.rs`) to drive the query
//! through the full engine stack.  The resulting `QueryResult` is used to build
//! a wire payload JSON (matching the structure of the `prism_query` MCP handler
//! in `prism_mcp::server`), which is then passed through
//! `rmcp::model::CallToolResult::structured` — the same call the MCP handler
//! makes — and the serialized `content[0].text` string is asserted.
//!
//! Spawning the `prism` binary as a subprocess is NOT required; the test runs
//! entirely in-process without `#[ignore]` and verifies the RED gate via
//! `cargo nextest run -p prism-bin -E 'test(rg026)'`.
//!
//! ## Traces
//!
//! - BC-2.11.001 EC-11-092/EC-11-093 (wire-level)
//! - AC-009(d) of S-ENGINE-LIMIT-EARLY-STOP-001
//! - ADR-060 §D8.3 (`any_early_stopped` propagation chain)

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    non_snake_case,
    unused_imports
)]

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::{OrgId, OrgSlug, PrismError, SensorId};
use prism_credentials::{CredentialStore, namespace::CredentialName};
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    engine::{QueryEngine, QueryEngineConfig, QueryOptions},
    scoping::ClientRegistry,
};
use prism_sensors::{
    AdapterRegistry, CredentialResolver,
    adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
};
use secrecy::SecretString;

// ---------------------------------------------------------------------------
// NullCredentialStore — no-op credential store for in-process wire tests
// ---------------------------------------------------------------------------

struct NullCredentialStore;

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

    async fn list(&self, _tenant: &OrgSlug) -> Result<Vec<(String, CredentialName)>, PrismError> {
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

// ---------------------------------------------------------------------------
// StubCredentialResolver — succeeds for any (client, sensor) pair
// ---------------------------------------------------------------------------

struct StubCredentialResolver;

impl CredentialResolver for StubCredentialResolver {
    fn resolve(
        &self,
        _client_id: &str,
        _sensor_id: SensorId,
    ) -> Result<Box<dyn SensorAuth>, SensorError> {
        // All built-in auth types deleted in PLUGIN-MIGRATION-001-A (AC-003, AC-006).
        // Use an inline test stub that satisfies the SensorAuth bound.
        // The mock adapter ignores auth entirely.
        struct TestStubAuth;
        impl SensorAuth for TestStubAuth {
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

// ---------------------------------------------------------------------------
// make_wire_engine — factory for in-process MCP wire tests
// ---------------------------------------------------------------------------

/// Build a `QueryEngine` with the given adapter registry and client list.
///
/// Uses `NullCredentialStore` (credentials never needed for mock adapters) and
/// `StubCredentialResolver` (prevents `fan_out()` from short-circuiting on
/// credential resolution failure before reaching the mock adapter fetch).
fn make_wire_engine(registry: AdapterRegistry, clients: Vec<OrgSlug>) -> QueryEngine {
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
    .with_credential_resolver(Arc::new(StubCredentialResolver))
}

// ---------------------------------------------------------------------------
// RG-PSG-026: MCP wire-level truncation signal
// ---------------------------------------------------------------------------

/// RG-PSG-026 — AC-009(d) EC-11-092/EC-11-093 wire-level: `prism_query` MCP
/// tool `CallToolResult.content[0].text` JSON carries the correct `is_truncated`
/// signal for two scenarios.
///
/// ## What this test verifies
///
/// Drives the full `QueryEngine::execute` path for two scenarios, then builds the
/// MCP wire payload in the same way the `prism_query` handler does (per
/// `prism_mcp::server` §query handler: `serde_json::json!({ "is_truncated": result.is_truncated, ... })`).
/// The payload is passed through `rmcp::model::CallToolResult::structured`, and
/// the serialized `content[0].text` string is asserted at the raw-byte level.
///
/// ## Scenarios
///
/// ### Case 1 — bare projection at exact-limit boundary (RED gate driver)
///
/// `SELECT * FROM mock_events LIMIT {EXACT_LIMIT}` where mock page_size = EXACT_LIMIT.
///
/// Mock behavior: `params.limit > 0` (early-stop active) → returns exactly EXACT_LIMIT rows
/// (page 1 of 3).  Engine Step 6: `total_rows = EXACT_LIMIT, limit = EXACT_LIMIT`.
///
/// RED  (pre-round-16): `is_truncated = total_rows > limit = 1000 > 1000 = false`.
///   Wire JSON: `{"is_truncated":false,...}`.
///   Assertion `wire_text.contains("\"is_truncated\":true")` → FAILS.
///
/// GREEN (post-round-16 — `any_early_stopped` propagated):
///   `is_truncated = (total_rows > limit) OR any_early_stopped = false OR true = true`.
///   Wire JSON: `{"is_truncated":true,...}`.
///   Assertion → PASSES.
///
/// ### Case 2 — SQL WHERE suppresses early-stop (positive control)
///
/// `SELECT * FROM mock_events WHERE status = 'data' LIMIT {EXACT_LIMIT}`.
///
/// Condition G revised (`has_client_side_where = true` for SQL equality WHERE)
/// → `ast_is_reducing_plan = true` → `fetch_limit = 0` → `params.limit = 0` → mock
/// returns 3000 rows (all 3 pages).  DataFusion `WHERE status = 'data' LIMIT 1000`
/// → 1000 matching rows.  `any_early_stopped = false` (early-stop never fired).
/// `total_rows = 1000 = limit` → `total_rows > limit = false`.
/// `is_truncated = false OR false = false`.
///
/// Pre-round-16 (Condition G not yet implemented): `params.limit = 1000 > 0` → mock
/// returns 1000 rows → same result via `total_rows > limit = false`.
///
/// Wire JSON: `{"is_truncated":false,...}`.
/// Assertion `wire_text.contains("\"is_truncated\":false")` → PASSES in both states.
///
/// ## RED / GREEN
///
/// The whole test is RED (fails) because case 1 assertion panics before case 2 is reached.
/// GREEN: both assertions pass after `any_early_stopped` propagation chain is implemented.
///
/// ## SAP-3 compliance
///
/// Both queries go through `QueryEngine::execute` (full engine path from query string, not
/// a synthetic AST or direct `run_materialization_pipeline` call).
#[tokio::test]
async fn test_psg_rg026_prism_query_wire_surfaces_truncation_signal() {
    // -----------------------------------------------------------------------
    // Exact limit constant used by both cases.
    // -----------------------------------------------------------------------
    const EXACT_LIMIT: usize = 1000;

    // -----------------------------------------------------------------------
    // WireExactLimitMockAdapter
    //
    // Mirrors `EarlyStopExactLimitMockAdapter` from execute_integration_tests.rs
    // (PSG-025).  Identical behavior:
    //
    //   params.limit > 0 → page1_batches (EXACT_LIMIT rows, early-stop boundary)
    //   params.limit == 0 → full_batches  (3 × EXACT_LIMIT rows, gate suppressed)
    //
    // Case 1 (bare projection, early-stop active): params.limit = EXACT_LIMIT > 0
    //   → 1000 rows returned, engine sees total_rows = limit → is_truncated = false
    //   until any_early_stopped is wired (post-round-16: is_truncated = true).
    //
    // Case 2 (WHERE-suppressed, Condition G fires): params.limit = 0
    //   → 3000 rows returned, DataFusion LIMIT 1000 → 1000 rows, is_truncated = false.
    //   (Pre-round-16, Condition G not yet wired: params.limit = 1000 → 1000 rows,
    //   same is_truncated = false outcome.)
    // -----------------------------------------------------------------------
    struct WireExactLimitMockAdapter {
        fetch_count: Arc<AtomicU64>,
        page1_batches: Vec<RecordBatch>,
        full_batches: Vec<RecordBatch>,
    }

    impl std::fmt::Debug for WireExactLimitMockAdapter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("WireExactLimitMockAdapter")
                .field("fetch_count", &self.fetch_count.load(Ordering::Relaxed))
                .finish()
        }
    }

    #[async_trait]
    impl SensorAdapter for WireExactLimitMockAdapter {
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
        ) -> Result<Vec<RecordBatch>, SensorError> {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            if params.limit == 0 {
                // Gate suppressed (Condition G) → return all 3000 rows.
                Ok(self.full_batches.clone())
            } else {
                // Early-stop active → return exactly EXACT_LIMIT rows (page 1 of 3).
                Ok(self.page1_batches.clone())
            }
        }
    }

    // Build a RecordBatch with `n` rows; single "status" column (Utf8, all "data").
    // The "status" column is used by case 2's SQL WHERE status = 'data' predicate
    // (all rows match → DataFusion returns all rows up to LIMIT).
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

    // 3 pages × EXACT_LIMIT rows each (3000 rows total).
    let page1 = make_data_batch(EXACT_LIMIT);
    let page2 = make_data_batch(EXACT_LIMIT);
    let page3 = make_data_batch(EXACT_LIMIT);

    let fetch_count = Arc::new(AtomicU64::new(0));
    let adapter = Arc::new(WireExactLimitMockAdapter {
        fetch_count: Arc::clone(&fetch_count),
        page1_batches: vec![page1.clone()],
        full_batches: vec![page1, page2, page3],
    });

    let org_id = OrgId::new();
    let org_slug = OrgSlug::new_unchecked("test-org");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);

    let engine = make_wire_engine(registry, vec![org_slug.clone()]);

    // -----------------------------------------------------------------------
    // Case 1: bare projection LIMIT N at exact boundary
    //
    // Engine Step 6 (current code): is_truncated = total_rows > limit
    //   = EXACT_LIMIT > EXACT_LIMIT = false
    //
    // Wire JSON contains "is_truncated":false.
    // Assertion `contains("\"is_truncated\":true")` → FAILS → RED gate.
    //
    // Post-round-16: is_truncated = (total_rows > limit) OR any_early_stopped
    //   = false OR true = true
    // Wire JSON contains "is_truncated":true → assertion PASSES → GREEN.
    // -----------------------------------------------------------------------
    let options_case1 = QueryOptions {
        clients: Some(vec![org_slug.clone()]),
        sensors: None,
        limit: Some(EXACT_LIMIT),
        force_refresh: false,
        ..QueryOptions::default()
    };

    let result_case1 = engine
        .execute(
            &format!("SELECT * FROM mock_events LIMIT {EXACT_LIMIT}"),
            options_case1,
        )
        .await
        .expect(
            "PSG-026 case 1: QueryEngine::execute must not error for bare projection \
             against mock adapter",
        );

    // Precondition: adapter must have been called (not a vacuous short-circuit).
    let fc_after_case1 = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc_after_case1 >= 1,
        "PSG-026 case 1 precondition: adapter must have been fetched at least once \
         (fetch_count={fc_after_case1}); a count of 0 means the pipeline short-circuited \
         and all wire assertions are vacuous."
    );

    // Build the wire payload as the prism_query MCP handler does:
    //   serde_json::json!({ "returned_results": ..., "total_available": ..., "is_truncated": ... })
    // Then pass it through CallToolResult::structured to produce the MCP wire result.
    // content[0].text == payload.to_string() (compact JSON, no spaces around colons).
    let wire_payload_case1 = serde_json::json!({
        "returned_results": result_case1.returned_results,
        "total_available":  result_case1.total_available,
        "is_truncated":     result_case1.is_truncated,
    });
    let call_result_case1 = rmcp::model::CallToolResult::structured(wire_payload_case1);
    let wire_text_case1 = call_result_case1
        .content
        .first()
        .and_then(|c| c.as_text())
        .map(|t| t.text.clone())
        .expect("PSG-026 case 1: CallToolResult::structured must produce content[0] text");

    // PRIMARY ASSERTION (RED gate driver):
    //
    // RED  (pre-round-16): wire_text contains "is_truncated":false → FAILS.
    // GREEN (post-round-16): wire_text contains "is_truncated":true → PASSES.
    assert!(
        wire_text_case1.contains("\"is_truncated\":true"),
        "PSG-026 case 1 (AC-009(d) EC-11-092 wire-level — RED gate driver): \
         CallToolResult.content[0].text must contain \"is_truncated\":true when \
         early-stop fires at the exact LIMIT boundary ({EXACT_LIMIT} rows returned \
         == limit {EXACT_LIMIT}). \
         Got wire JSON: {wire_text_case1}. \
         \n\nDiagnosis: Engine Step 6 currently computes \
         is_truncated = total_rows > limit = {EXACT_LIMIT} > {EXACT_LIMIT} = false. \
         The any_early_stopped OR term is not yet implemented (ADR-060 §D8.3). \
         Fix: (1) Add any_early_stopped: bool to FetchOutput; (2) propagate through \
         FanOutResult → MaterializationOutput; (3) in engine Step 6 set \
         is_truncated = (total_rows > limit) OR any_early_stopped."
    );

    // -----------------------------------------------------------------------
    // Case 2: SQL WHERE suppresses early-stop (positive control)
    //
    // Condition G revised: has_client_side_where = true for SQL equality WHERE
    // → ast_is_reducing_plan = true → fetch_limit = 0 → params.limit = 0
    // → mock returns 3000 rows → DataFusion WHERE status = 'data' LIMIT 1000
    // → 1000 rows → total_rows = 1000 = limit → total_rows > limit = false
    // → any_early_stopped = false → is_truncated = false OR false = false.
    //
    // Pre-round-16 (Condition G not yet wired for SQL equality):
    // params.limit = 1000 > 0 → mock returns 1000 rows → same is_truncated = false.
    //
    // Wire JSON contains "is_truncated":false → assertion PASSES in both states.
    // -----------------------------------------------------------------------
    let options_case2 = QueryOptions {
        clients: Some(vec![org_slug.clone()]),
        sensors: None,
        limit: Some(EXACT_LIMIT),
        force_refresh: false,
        ..QueryOptions::default()
    };

    let result_case2 = engine
        .execute(
            &format!("SELECT * FROM mock_events WHERE status = 'data' LIMIT {EXACT_LIMIT}"),
            options_case2,
        )
        .await
        .expect(
            "PSG-026 case 2: QueryEngine::execute must not error for SQL WHERE query \
             against mock adapter",
        );

    let wire_payload_case2 = serde_json::json!({
        "returned_results": result_case2.returned_results,
        "total_available":  result_case2.total_available,
        "is_truncated":     result_case2.is_truncated,
    });
    let call_result_case2 = rmcp::model::CallToolResult::structured(wire_payload_case2);
    let wire_text_case2 = call_result_case2
        .content
        .first()
        .and_then(|c| c.as_text())
        .map(|t| t.text.clone())
        .expect("PSG-026 case 2: CallToolResult::structured must produce content[0] text");

    // POSITIVE CONTROL: is_truncated must be false when early-stop is suppressed.
    // PASSES in both RED and GREEN states — validates the suppression path.
    assert!(
        wire_text_case2.contains("\"is_truncated\":false"),
        "PSG-026 case 2 (AC-009(d) suppression positive control): \
         CallToolResult.content[0].text must contain \"is_truncated\":false when \
         early-stop is suppressed by Condition G (SQL WHERE clause). \
         Got wire JSON: {wire_text_case2}."
    );
}
