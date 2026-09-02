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
//! ## In-process approach (F-R16-P12-MED-001 fix)
//!
//! Dispatches through the REAL `PrismServer::query` tool handler — the same
//! code path that production MCP clients hit — using a local mock `SensorAdapter`
//! to force the exact-limit boundary and the temporal-WHERE-suppression scenarios.
//!
//! The handler builds the query payload, runs it through
//! `SafetyEnvelopeBuilder::wrap("query", DataSource::Multiple(...), payload, 1,
//! result.is_truncated, None, audit_warning)`, serializes the `ResponseEnvelope`
//! to JSON, and returns `CallToolResult::structured(envelope_val)`.  The test
//! asserts on `call_result.content[0].text` — the exact bytes the LLM agent
//! consumes — which contains the full serialized `ResponseEnvelope` JSON.
//!
//! The `is_truncated` key appears at `results.is_truncated` and
//! `structuredContent.results.is_truncated` within that envelope.  A regression
//! that drops or renames `is_truncated` in the envelope path would be caught here
//! but invisible to any test that bypasses `SafetyEnvelopeBuilder::wrap`.
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

extern crate toml;

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
use prism_mcp::server::{PrismServer, QueryToolParams};
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    engine::{QueryEngine, QueryEngineConfig},
    scoping::ClientRegistry,
};
use prism_sensors::{
    AdapterRegistry, CredentialResolver,
    adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
};
use rmcp::handler::server::wrapper::Parameters;
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
/// ## What this test verifies (F-R16-P12-MED-001 fix)
///
/// Dispatches through the REAL `PrismServer::query` tool handler using a local
/// mock `SensorAdapter`.  The handler runs the query through `QueryEngine::execute`,
/// builds the payload, wraps it via `SafetyEnvelopeBuilder::wrap`, serializes the
/// `ResponseEnvelope` to JSON, and returns `CallToolResult::structured(envelope_val)`.
///
/// The test asserts on `call_result.content[0].text` — `envelope_val.to_string()`,
/// the exact bytes the LLM agent consumes — which is the full serialized
/// `ResponseEnvelope`.  The `"is_truncated"` key appears at `results.is_truncated`
/// and `structuredContent.results.is_truncated` within that envelope.
///
/// A regression that drops or renames `is_truncated` in the envelope serialization
/// path (e.g., a bug in `SafetyEnvelopeBuilder::wrap` or in the `results` payload
/// construction) would be caught here but invisible to any test that bypasses
/// `SafetyEnvelopeBuilder::wrap` (the previous paper-gate defect class).
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
///   Wire envelope: `results.is_truncated = false`.
///   Assertion `wire_text.contains("\"is_truncated\":true")` → FAILS.
///
/// GREEN (post-round-16 — `any_early_stopped` propagated):
///   `is_truncated = (total_rows > limit) OR any_early_stopped = false OR true = true`.
///   Wire envelope: `results.is_truncated = true`.
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
/// Wire envelope: `results.is_truncated = false`.
/// Assertion `wire_text.contains("\"is_truncated\":false")` → PASSES in both states.
///
/// ## Test status
///
/// Both case 1 and case 2 assertions pass: `any_early_stopped` propagation is
/// implemented (post-round-16) and Condition G is active for SQL equality WHERE.
///
/// ## SAP-3 compliance
///
/// Both queries reach the PrismServer `query` tool handler end-to-end, driving
/// `QueryEngine::execute` from a query string (not a synthetic AST or direct
/// `run_materialization_pipeline` call).
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
        ) -> Result<FetchOutput, SensorError> {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            if params.limit == 0 {
                // Gate suppressed (Condition G) → return all 3000 rows.
                Ok(FetchOutput::new(self.full_batches.clone(), false, false))
            } else {
                // Early-stop active → return exactly EXACT_LIMIT rows (page 1 of 3).
                Ok(FetchOutput::new(self.page1_batches.clone(), true, false))
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

    let engine = make_wire_engine(registry, vec![org_slug]);

    // -----------------------------------------------------------------------
    // Build PrismServer with the engine wired.
    //
    // PrismServer::new() is the test-only constructor (no audit_writer, no
    // org_registry).  Audit emission with no AuditWriter returns Ok(None) per
    // emit_tool_audit's None branch — the query proceeds with audit_warning = None.
    //
    // With no OrgRegistry, resolve_org_id falls back to the first registered
    // adapter for sensor "mock" (Path 2 in materialization.rs) — which is the
    // WireExactLimitMockAdapter registered above.
    //
    // The "test-org" slug passes validate_client_ids (alphanumeric + dash, ≤ 64
    // chars) and is present in ClientRegistry, so resolve_clients succeeds.
    // -----------------------------------------------------------------------
    let server = PrismServer::new().with_query_engine(Arc::new(engine));

    // -----------------------------------------------------------------------
    // Case 1: bare projection LIMIT N at exact boundary
    //
    // params.limit = 1000 → build_query_options sets QueryOptions.limit = 1000
    // → early-stop active (fetch_limit = 1000 > 0) → mock returns 1000 rows.
    //
    // Engine Step 6 (post-round-16): is_truncated = (total_rows > limit) OR any_early_stopped
    //   = false OR true = true
    // Wire envelope results.is_truncated = true → assertion PASSES.
    // -----------------------------------------------------------------------
    // QueryToolParams is #[non_exhaustive] — use serde_json deserialization for
    // cross-crate construction (struct literal syntax is forbidden outside prism-mcp).
    let params_case1: QueryToolParams = serde_json::from_value(serde_json::json!({
        "query":         format!("SELECT * FROM mock_events LIMIT {EXACT_LIMIT}"),
        "clients":       ["test-org"],
        "limit":         EXACT_LIMIT as u32,
        "force_refresh": false
    }))
    .expect("PSG-026 case 1: QueryToolParams must deserialize from known-field JSON object");

    let call_result_case1 = server.query(Parameters(params_case1)).await.expect(
        "PSG-026 case 1: PrismServer::query must return Ok(CallToolResult) for bare \
             projection against mock adapter (Err here means an internal PrismError — \
             check query parse or engine wiring)",
    );

    // Precondition: adapter must have been called (not a vacuous short-circuit).
    let fc_after_case1 = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc_after_case1 >= 1,
        "PSG-026 case 1 precondition: adapter must have been fetched at least once \
         (fetch_count={fc_after_case1}); a count of 0 means the pipeline short-circuited \
         before reaching the mock adapter and all wire assertions are vacuous."
    );

    // Extract the ACTUAL wire bytes from the real SafetyEnvelopeBuilder serialization.
    //
    // content[0].text = serde_json::to_value(&ResponseEnvelope {
    //   _meta: { has_more: false, next_cursor: null, ... },  // ADR-060 §D8.7: always false/null
    //   results: { rows: [...], returned_results: N, total_available: N, is_truncated: X },
    //   content: [{ type: "text", text: "N results found" }],
    //   structuredContent: { results: { ... is_truncated: X ... } }
    // }).to_string()
    //
    // is_truncated appears in both results.is_truncated and
    // structuredContent.results.is_truncated within the serialized envelope.
    let wire_text_case1 = call_result_case1
        .content
        .first()
        .and_then(|c| c.as_text())
        .map(|t| t.text.clone())
        .expect(
            "PSG-026 case 1: CallToolResult::structured must produce a text content[0] item \
             (the serialized SafetyEnvelope JSON)",
        );

    // PRIMARY ASSERTION (RED gate driver):
    //
    // RED  (pre-round-16): envelope results.is_truncated = false
    //   → wire_text contains "is_truncated\":false → FAILS.
    // GREEN (post-round-16): envelope results.is_truncated = true
    //   → wire_text contains "is_truncated\":true → PASSES.
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
    // Wire envelope results.is_truncated = false → assertion PASSES in both states.
    // -----------------------------------------------------------------------
    let params_case2: QueryToolParams = serde_json::from_value(serde_json::json!({
        "query":         format!("SELECT * FROM mock_events WHERE status = 'data' LIMIT {EXACT_LIMIT}"),
        "clients":       ["test-org"],
        "limit":         EXACT_LIMIT as u32,
        "force_refresh": false
    }))
    .expect(
        "PSG-026 case 2: QueryToolParams must deserialize from known-field JSON object",
    );

    let call_result_case2 = server.query(Parameters(params_case2)).await.expect(
        "PSG-026 case 2: PrismServer::query must return Ok(CallToolResult) for SQL WHERE \
             query against mock adapter",
    );

    let wire_text_case2 = call_result_case2
        .content
        .first()
        .and_then(|c| c.as_text())
        .map(|t| t.text.clone())
        .expect("PSG-026 case 2: CallToolResult::structured must produce a text content[0] item");

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

// ---------------------------------------------------------------------------
// RG-PSG-028 wire-level: multi-sensor fan-out no-early-stop is NOT truncated
// (round-16 remediation)
// ADR-060 §D8.3 — any_early_stopped OR-aggregated across fan-out sensors
// BC-2.11.001 EC-11-092/EC-11-093 (wire level)
// ---------------------------------------------------------------------------

/// RG-PSG-028 wire-level — `CallToolResult.content[0].text` must carry
/// `"is_truncated":false` when 2 sensors fan out, both exhaust their full result
/// sets (25 rows each, total=50==limit), and neither early-stops.
///
/// ## What this test verifies (F-R16-P14-MED-001 fix)
///
/// Dispatches through the REAL `PrismServer::query` tool handler — the same code
/// path that production MCP clients hit.  The handler calls `QueryEngine::execute`,
/// builds the wire payload, and wraps it via `SafetyEnvelopeBuilder::wrap`, producing
/// a `ResponseEnvelope` that is serialized and returned as `CallToolResult`.
///
/// The test asserts on `call_result.content[0].text` — the exact bytes the LLM agent
/// consumes — which contains the full serialized `ResponseEnvelope` JSON including
/// `results.is_truncated` and `structuredContent.results.is_truncated`.
///
/// A regression that drops `is_truncated` from the envelope (e.g., a bug in
/// `SafetyEnvelopeBuilder::wrap` or the payload construction) is caught here but
/// invisible to any test that hand-builds a `serde_json::json!` payload and bypasses
/// the real handler (the previous paper-gate defect class closed by this fix).
///
/// ## Topology
///
/// 2-sensor fan-out, `options.limit = 50`:
/// - sensor1 (org-a): returns 25 rows (no early-stop)
/// - sensor2 (org-b): returns 25 rows (no early-stop)
/// - total = 50 == limit
///
/// ## RED Gate
///
/// Current heuristic: `total_fetched_rows=50 >= fetch_limit=50 → any_early_stopped=true`
/// → `is_truncated = (50>50) OR true = true`
/// → wire JSON contains `"is_truncated":true`
/// → assertion `contains("\"is_truncated\":false")` FAILS → RED GATE.
///
/// ## GREEN (post-round-16)
///
/// Per-sensor `FetchOutput.early_stopped=false` for both sensors
/// → `any_early_stopped = false OR false = false`
/// → `is_truncated = (50>50) OR false = false`
/// → wire JSON contains `"is_truncated":false` → assertion PASSES.
///
/// ## SAP-3 compliance
///
/// Query goes through `PrismServer::query` → `QueryEngine::execute` (full engine
/// path from query string, not a synthetic AST or direct `run_materialization_pipeline`
/// call).
///
/// ## Wire-shape assertion discipline (CLAUDE.md 2026-07-13)
///
/// Asserts on the SERIALIZED JSON output (the exact bytes the LLM agent consumes),
/// exercising the real `SafetyEnvelopeBuilder::wrap` path — not a hand-built
/// `serde_json::json!` payload that bypasses envelope construction.
///
/// ## HARNESS NOTE — why explicit OrgRegistry + clients:
///
/// UUID v7 uses a 48-bit millisecond timestamp in the high bits.  Two consecutive
/// OrgId::new() calls within the same millisecond produce UUIDs with identical
/// first-8 hex chars.  Without an OrgRegistry the pipeline synthesises client_id
/// as "org-{first8}" → both adapters share the same in-query cache key → second
/// adapter is served from cache (no live fan-out, total_fetched_rows = 25 not 50)
/// → heuristic 25 < 50 = false → is_truncated=false → wire assertion PASSES for
/// the WRONG reason (silent false negative, not a RED gate).
///
/// With OrgRegistry: "org-a" → org_id1, "org-b" → org_id2 → unique cache keys
/// → both adapters execute live fan-out → total_fetched_rows = 50 → heuristic
/// 50 >= 50 = TRUE (FALSE POSITIVE) → is_truncated=true → wire JSON contains
/// "is_truncated":true → assertion contains("\"is_truncated\":false") FAILS → RED.
#[tokio::test]
async fn test_psg_rg028_wire_multi_sensor_fanout_no_early_stop_is_not_truncated() {
    const LIMIT: usize = 50;

    // -----------------------------------------------------------------------
    // FanoutStubAdapter — returns exactly `row_count` rows unconditionally.
    //
    // Neither adapter early-stops: each returns its full result set.
    // This simulates two sensors that exhausted their data (25 rows each)
    // without hitting a pagination limit.
    //
    // fetch_count: Arc<AtomicU64> — non-vacuousness precondition; confirms the
    // handler dispatched to both adapters (not a cache hit or short-circuit).
    // -----------------------------------------------------------------------
    struct FanoutStubAdapter {
        fetch_count: Arc<AtomicU64>,
        batches: Vec<RecordBatch>,
    }

    impl std::fmt::Debug for FanoutStubAdapter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("FanoutStubAdapter")
                .field("fetch_count", &self.fetch_count.load(Ordering::Relaxed))
                .field("batch_count", &self.batches.len())
                .finish()
        }
    }

    #[async_trait]
    impl SensorAdapter for FanoutStubAdapter {
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
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            // Unconditionally return the pre-built batches — no early-stop behavior.
            Ok(FetchOutput::new(self.batches.clone(), false, false))
        }
    }

    // Build a RecordBatch with `n` rows; single "detection_id" column (Utf8).
    fn make_batch(n: usize) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "detection_id",
            DataType::Utf8,
            false,
        )]));
        let ids: Vec<String> = (0..n).map(|i| format!("id-{i}")).collect();
        let arr = Arc::new(StringArray::from(
            ids.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        )) as Arc<dyn arrow::array::Array>;
        RecordBatch::try_new(schema, vec![arr]).expect("PSG-028 wire: make_batch must succeed")
    }

    let fetch_count1 = Arc::new(AtomicU64::new(0));
    let fetch_count2 = Arc::new(AtomicU64::new(0));

    let org_id1 = OrgId::new();
    let org_id2 = OrgId::new();
    let mut registry = AdapterRegistry::new();
    registry.register(
        org_id1,
        Arc::new(FanoutStubAdapter {
            fetch_count: Arc::clone(&fetch_count1),
            batches: vec![make_batch(25)],
        }),
    );
    registry.register(
        org_id2,
        Arc::new(FanoutStubAdapter {
            fetch_count: Arc::clone(&fetch_count2),
            batches: vec![make_batch(25)],
        }),
    );

    let org_a = OrgSlug::new_unchecked("org-a");
    let org_b = OrgSlug::new_unchecked("org-b");

    let org_registry = prism_core::OrgRegistry::new();
    org_registry
        .register(org_a.clone(), org_id1)
        .expect("PSG-028 wire: register org-a must succeed");
    org_registry
        .register(org_b.clone(), org_id2)
        .expect("PSG-028 wire: register org-b must succeed");

    let engine =
        make_wire_engine(registry, vec![org_a, org_b]).with_org_registry(Arc::new(org_registry));

    // -----------------------------------------------------------------------
    // Build PrismServer with the engine wired — same pattern as RG-PSG-026.
    //
    // PrismServer::new() is the test-only constructor (no audit_writer).
    // No server-level OrgRegistry is needed for the query path: validate_client_ids
    // in the query handler is a FORMAT check only ([a-zA-Z0-9_-]{1,64}), not an
    // allowlist check — "org-a" and "org-b" satisfy it.  The engine's OrgRegistry
    // (wired above) handles client-slug → OrgId resolution during fan-out.
    //
    // Audit emission with no AuditWriter returns Ok(None) per emit_tool_audit's
    // None branch — the query proceeds with audit_warning = None.
    // -----------------------------------------------------------------------
    let server = PrismServer::new().with_query_engine(Arc::new(engine));

    // -----------------------------------------------------------------------
    // Build QueryToolParams via serde_json (QueryToolParams is #[non_exhaustive]
    // — struct literal syntax is forbidden outside prism-mcp; use serde_json
    // deserialization per the established PSG-026 pattern).
    // -----------------------------------------------------------------------
    let params: QueryToolParams = serde_json::from_value(serde_json::json!({
        "query":         format!("SELECT * FROM crowdstrike_detections LIMIT {LIMIT}"),
        "clients":       ["org-a", "org-b"],
        "limit":         LIMIT as u32,
        "force_refresh": false,
    }))
    .expect("PSG-028 wire: QueryToolParams must deserialize from known-field JSON object");

    let call_result = server.query(Parameters(params)).await.expect(
        "PSG-028 wire: PrismServer::query must return Ok(CallToolResult) for 2-sensor \
         fan-out against FanoutStubAdapters (Err here means an internal PrismError — \
         check query parse or engine wiring)",
    );

    // PRECONDITION: both adapters must have been fanned out to (non-vacuous check).
    // Verifies the pipeline dispatched live fetch calls to both adapters rather than
    // short-circuiting via cache or client-resolution failure.
    let fc1 = fetch_count1.load(Ordering::SeqCst);
    let fc2 = fetch_count2.load(Ordering::SeqCst);
    assert!(
        fc1 >= 1 && fc2 >= 1,
        "PSG-028 wire precondition: both adapters must have been fetched at least once \
         (fetch_count1={fc1}, fetch_count2={fc2}); a count of 0 means the pipeline \
         short-circuited before reaching that adapter and all wire assertions are vacuous."
    );

    // Extract the ACTUAL wire bytes from the real SafetyEnvelopeBuilder serialization.
    //
    // content[0].text = serde_json::to_value(&ResponseEnvelope {
    //   _meta: { has_more: false, next_cursor: null, ... },  // ADR-060 §D8.7: always false/null
    //   results: { rows: [...], returned_results: N, total_available: N, is_truncated: X },
    //   content: [{ type: "text", text: "N results found" }],
    //   structuredContent: { results: { ... is_truncated: X ... } }
    // }).to_string()
    //
    // is_truncated appears in both results.is_truncated and
    // structuredContent.results.is_truncated within the serialized envelope.
    let wire_text = call_result
        .content
        .first()
        .and_then(|c| c.as_text())
        .map(|t| t.text.clone())
        .expect(
            "PSG-028 wire: CallToolResult::structured must produce a text content[0] item \
             (the serialized SafetyEnvelope JSON)",
        );

    // PRIMARY ASSERTION (RED gate driver — wire level):
    //
    // RED  (current heuristic): total_fetched=50 >= fetch_limit=50 → any_early_stopped=TRUE
    //   → is_truncated=true → wire JSON contains "is_truncated":true
    //   → contains("\"is_truncated\":false") FAILS.
    //
    // GREEN (post-round-16): any_early_stopped=false → is_truncated=false
    //   → wire JSON contains "is_truncated":false → PASSES.
    assert!(
        wire_text.contains("\"is_truncated\":false"),
        "PSG-028 wire (ADR-060 §D8.3 — RED GATE — 2-sensor fan-out no-early-stop, wire level): \
         CallToolResult.content[0].text must contain \"is_truncated\":false when both sensors \
         fully exhausted their result sets (sensor1=25 rows, sensor2=25 rows, total=50==limit={LIMIT}, \
         any_early_stopped=false). \
         Got wire JSON: {wire_text}. \
         \n\nDiagnosis: The heuristic `total_fetched_rows >= fetch_limit` (50>=50=true) is a \
         FALSE POSITIVE at this boundary — it cannot distinguish sensor exhaustion from \
         early-stop pagination. Fix: implement per-sensor FetchOutput.early_stopped and \
         OR-aggregate into FanOutResult.any_early_stopped (ADR-060 §D8.3, \
         S-ENGINE-LIMIT-EARLY-STOP-001 Task 16). The LLM agent sees `\"is_truncated\":true` \
         and incorrectly signals incomplete data when the result set is actually complete.",
    );
}

// ---------------------------------------------------------------------------
// RG-SLUG-005 wire-level: cross-tenant isolation — collision-resistant cache keys
// SECURITY CRITICAL (AC-013 / CWE-284/340/OWASP-A01)
// ADR-061 D4 — in-query cache keys must be collision-resistant
// ---------------------------------------------------------------------------

/// RG-SLUG-005 wire-level — `CallToolResult.content[0].text` must contain
/// `"beta-001"` rows from tenant-beta when a bare-filter ALL-scope fan-out drives
/// two tenants whose org_ids share an identical first-8-hex prefix.
///
/// ## Wire-shape assertion discipline (CLAUDE.md 2026-07-13)
///
/// Dispatches through the REAL `PrismServer::query` tool handler — the same code
/// path that production MCP clients hit.  The handler runs the query through
/// `QueryEngine::execute`, serializes the `RecordBatch` rows via `arrow-json`, wraps
/// the payload in `SafetyEnvelopeBuilder::wrap`, and returns the serialized
/// `ResponseEnvelope` as `CallToolResult`.
///
/// The test asserts on `call_result.content[0].text` — the exact bytes the LLM
/// agent consumes — confirming that both tenants' rows appear in the wire output.
///
/// ## SECURITY CRITICAL: cross-tenant isolation (AC-013, CWE-284/340/OWASP-A01)
///
/// This is NOT merely a correctness test.  If the collision is present:
/// - Tenant-beta's adapter is NEVER called
/// - Tenant-alpha's cached rows are served in place of tenant-beta's rows
/// - The LLM agent receives rows from the wrong tenant without any error signal
/// - This is a cross-tenant data leakage / isolation failure at the wire level
///
/// ## Collision mechanism (RED)
///
/// Two `OrgId`s built from bytes with prefix `[0xde, 0xad, 0xbe, 0xef, ...]`
/// both stringify to `"deadbeef-..."` — first 8 chars are "deadbeef" for both.
/// Step 3b in `run_materialization_pipeline` (bare-filter ALL-scope path with
/// empty `ClientRegistry`) synthesizes slugs via `format!("org-{}", &org_id.to_string()[..8])`.
/// Both synthetic slugs = "org-deadbeef".
/// In-query cache key = `format!("{}:{}:...", client_id, sensor_id, ...)`.
/// With the same slug, adapter-A fetches first and its rows cache under
/// "org-deadbeef:crowdstrike:...".  Adapter-B's key is identical → cache HIT →
/// adapter-B NEVER called → "beta-001" rows absent from wire output.
///
/// `wire_text.contains("\"beta-001\"")` FAILS → RED GATE.
///
/// ## Correct behaviour (post-D2 fix — GREEN)
///
/// Step 3b consults `mat_ctx.org_registry`:
/// - org_id_A → slug "tenant-alpha" (from OrgRegistry)
/// - org_id_B → slug "tenant-beta"  (from OrgRegistry)
/// Distinct slugs → distinct in-query cache keys → adapter-B fetched independently
/// → "beta-001" rows present in wire output → assertion PASSES.
///
/// ## Non-vacuity precondition
///
/// `fetch_count_b >= 1` verifies adapter-B was actually dispatched (not just
/// short-circuited via cache).  In the RED state this precondition fires first
/// and directly names the root cause: adapter-B was never called.
///
/// ## Topology (mirrors PSG-026/PSG-028 pattern)
///
/// Empty ClientRegistry + `clients: None` → Step 3b enumerates `adapter_registry`.
/// OrgRegistry wired on the engine (not on the server) to satisfy Step 3b resolution
/// post-fix while keeping the MCP-server constructor minimal (no AuditWriter).
///
/// ## Traces
///
/// - AC-013 of S-ENGINE-LIMIT-EARLY-STOP-001
/// - ADR-061 D4 (collision-resistant in-query cache keys)
/// - BC-2.01.002 (multi-tenant fan-out isolation)
/// - CWE-284 / CWE-340 / OWASP A01
#[tokio::test]
async fn test_rg_slug_005_wire_cross_tenant_isolation_collision_resistant_cache_keys() {
    use prism_core::{OrgId, OrgRegistry};
    use uuid::Uuid;

    // Two OrgIds whose first-8-hex chars are identical ("deadbeef").
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

    // Collision precondition: both share the "deadbeef" first-8-hex prefix.
    assert_eq!(
        &org_id_a.to_string()[..8],
        "deadbeef",
        "SLUG-005 wire precondition: org_id_a first-8-hex must be 'deadbeef'"
    );
    assert_eq!(
        &org_id_b.to_string()[..8],
        "deadbeef",
        "SLUG-005 wire precondition: org_id_b first-8-hex must be 'deadbeef'"
    );

    // -----------------------------------------------------------------------
    // ProviderAdapter — returns a single row with the specified provider value.
    //
    // fetch_count: Arc<AtomicU64> — non-vacuousness gate; confirms the engine
    // dispatched a live fetch call rather than serving from the in-query cache.
    // -----------------------------------------------------------------------
    struct ProviderAdapter {
        sensor_id: prism_core::SensorId,
        provider_value: &'static str,
        fetch_count: Arc<AtomicU64>,
    }

    impl std::fmt::Debug for ProviderAdapter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ProviderAdapter")
                .field("provider_value", &self.provider_value)
                .finish()
        }
    }

    #[async_trait]
    impl SensorAdapter for ProviderAdapter {
        fn sensor_type(&self) -> prism_core::SensorId {
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
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            let schema = Arc::new(Schema::new(vec![Field::new(
                "provider",
                DataType::Utf8,
                false,
            )]));
            let arr = Arc::new(StringArray::from(vec![self.provider_value]))
                as Arc<dyn arrow::array::Array>;
            let batch = RecordBatch::try_new(schema, vec![arr])
                .expect("SLUG-005 wire: ProviderAdapter batch must build");
            Ok(FetchOutput::new(vec![batch], false, false))
        }
    }

    let fetch_count_a = Arc::new(AtomicU64::new(0));
    let fetch_count_b = Arc::new(AtomicU64::new(0));

    let mut registry = AdapterRegistry::new();
    registry.register(
        org_id_a,
        Arc::new(ProviderAdapter {
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            provider_value: "alpha-001",
            fetch_count: Arc::clone(&fetch_count_a),
        }),
    );
    registry.register(
        org_id_b,
        Arc::new(ProviderAdapter {
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            provider_value: "beta-001",
            fetch_count: Arc::clone(&fetch_count_b),
        }),
    );

    // OrgRegistry maps distinct slugs to each org_id.
    // Post-fix: Step 3b consults this registry instead of synthesizing from first-8-hex.
    let org_a = OrgSlug::new_unchecked("tenant-alpha");
    let org_b = OrgSlug::new_unchecked("tenant-beta");

    let org_registry = OrgRegistry::new();
    org_registry
        .register(org_a.clone(), org_id_a)
        .expect("SLUG-005 wire: register tenant-alpha must succeed");
    org_registry
        .register(org_b.clone(), org_id_b)
        .expect("SLUG-005 wire: register tenant-beta must succeed");

    // -----------------------------------------------------------------------
    // Engine setup: EMPTY ClientRegistry → Step 3b fires on bare-filter query.
    //
    // With a populated ClientRegistry ("tenant-alpha", "tenant-beta"):
    //   resolve_clients(None, registry) returns those slugs as an explicit list.
    //   The pipeline routes through resolve_source_refs with explicit clients,
    //   which correctly uses OrgRegistry — BYPASSING Step 3b entirely.
    //   The test would pass even with the collision bug (vacuous non-RED gate).
    //
    // With an EMPTY ClientRegistry:
    //   resolve_clients(None, empty) returns [].
    //   all_clients = [] → targets empty after Steps 1-3a → Step 3b fires.
    //   Step 3b iterates adapter_registry and synthesizes slugs from first-8-hex.
    //   Collision: both org_ids get "org-deadbeef" → same cache key → adapter-B
    //   served from adapter-A's cache → "beta-001" absent → RED GATE.
    // -----------------------------------------------------------------------
    let engine = make_wire_engine(registry, vec![]).with_org_registry(Arc::new(org_registry));

    // -----------------------------------------------------------------------
    // Build PrismServer with the engine wired — same pattern as RG-PSG-026/028.
    //
    // PrismServer::new() is the test-only constructor (no audit_writer).
    // Audit emission with no AuditWriter returns Ok(None) per emit_tool_audit's
    // None branch — the query proceeds with audit_warning = None.
    //
    // No server-level OrgRegistry needed: validate_client_ids is a FORMAT check
    // only ([a-zA-Z0-9_-]{1,64}), not an allowlist check.  The engine's
    // OrgRegistry (wired above) handles slug→OrgId resolution in Step 3b post-fix.
    // -----------------------------------------------------------------------
    let server = PrismServer::new().with_query_engine(Arc::new(engine));

    // -----------------------------------------------------------------------
    // Bare-filter query with clients: None → ALL-scope → Step 3b fires.
    //
    // QueryToolParams is #[non_exhaustive] — use serde_json deserialization for
    // cross-crate construction (struct literal syntax is forbidden outside prism-mcp).
    //
    // clients field OMITTED (None) so the pipeline does not receive an explicit
    // client list and must enumerate adapter_registry via Step 3b.
    // -----------------------------------------------------------------------
    let params: QueryToolParams = serde_json::from_value(serde_json::json!({
        "query":         "provider IS NOT NULL",
        "limit":         10_u32,
        "force_refresh": false
    }))
    .expect("SLUG-005 wire: QueryToolParams must deserialize from known-field JSON object");

    let call_result = server.query(Parameters(params)).await.expect(
        "SLUG-005 wire: PrismServer::query must return Ok(CallToolResult) for bare-filter \
         ALL-scope fan-out against ProviderAdapters (Err here means an internal PrismError — \
         check query parse or engine wiring)",
    );

    // NON-VACUITY PRECONDITION (RED state diagnostic):
    //
    // In the RED state (collision):
    //   adapter-B never called (fetch_count_b == 0) because Step 3b produces
    //   slug "org-deadbeef" for both org_ids → identical cache key → adapter-B
    //   served from adapter-A's cache entry → "beta-001" rows absent.
    //
    // In the GREEN state (fix):
    //   adapter-B called with slug "tenant-beta" → distinct cache key → live fetch
    //   → "beta-001" rows present.
    //
    // This precondition fires BEFORE the primary wire assertion and names the
    // isolation failure root cause explicitly (cache collision → missing fan-out).
    let fc_b = fetch_count_b.load(Ordering::SeqCst);
    assert!(
        fc_b >= 1,
        "SLUG-005 wire precondition (SECURITY CRITICAL — cross-tenant isolation): \
         adapter-B (tenant-beta / 'beta-001' rows) must have been dispatched at least once \
         (fetch_count_b={fc_b}). \
         A count of 0 means Step 3b produced slug 'org-deadbeef' for both org_ids \
         (UUID first-8-hex collision: 'deadbeef'), causing an in-query cache HIT for \
         adapter-B that served adapter-A's rows. \
         This is a cross-tenant data isolation failure (CWE-284/CWE-340/OWASP-A01): \
         tenant-beta's data is invisible to the LLM agent. \
         Fix: Step 3b must consult mat_ctx.org_registry when present (ADR-061 D2)."
    );

    // Extract the ACTUAL wire bytes from the real SafetyEnvelopeBuilder serialization.
    //
    // content[0].text = serde_json::to_value(&ResponseEnvelope {
    //   results: { rows: [{"provider": "alpha-001"}, {"provider": "beta-001"}], ... },
    //   content: [{ type: "text", text: "N results found" }],
    //   ...
    // }).to_string()
    let wire_text = call_result
        .content
        .first()
        .and_then(|c| c.as_text())
        .map(|t| t.text.clone())
        .expect(
            "SLUG-005 wire: CallToolResult::structured must produce a text content[0] item \
             (the serialized SafetyEnvelope JSON)",
        );

    // PRIMARY ASSERTION (RED gate driver — wire level, SECURITY CRITICAL):
    //
    // RED  (collision — current Step 3b):
    //   Slug "org-deadbeef" for both org_ids → cache collision → adapter-B never called
    //   → "beta-001" rows absent from wire output
    //   → wire_text does NOT contain "\"beta-001\"" → FAILS → RED GATE.
    //
    // GREEN (post-ADR-061 D2 fix):
    //   Step 3b resolves slug "tenant-beta" for org_id_b via OrgRegistry
    //   → distinct cache key → adapter-B fetched independently
    //   → "beta-001" rows present in wire output
    //   → wire_text CONTAINS "\"beta-001\"" → PASSES.
    assert!(
        wire_text.contains("\"beta-001\""),
        "SLUG-005 wire (ADR-061 D4 — SECURITY CRITICAL — cross-tenant wire isolation, RED GATE): \
         CallToolResult.content[0].text must contain 'beta-001' rows from tenant-beta adapter. \
         Got wire JSON: {wire_text}. \
         \n\nDiagnosis: Step 3b synthesized slug 'org-deadbeef' for BOTH org_ids \
         (UUID first-8-hex collision: bytes[0..4] = [0xde,0xad,0xbe,0xef]), causing an \
         in-query cache HIT for adapter-B that served adapter-A's 'alpha-001' rows. \
         Tenant-beta's data is completely invisible to the LLM agent — this is a \
         cross-tenant data leakage / isolation failure at the MCP wire level. \
         Fix: Step 3b must consult mat_ctx.org_registry when present (ADR-061 D2 / \
         S-ENGINE-LIMIT-EARLY-STOP-001 AC-013)."
    );

    // ISOLATION POSITIVE CONTROL: adapter-A (alpha-001) must also be present.
    // Verifies both tenants are represented in the wire output, not just beta.
    let fc_a = fetch_count_a.load(Ordering::SeqCst);
    assert!(
        fc_a >= 1,
        "SLUG-005 wire isolation control: adapter-A (tenant-alpha) fetch_count_a must be >= 1; \
         got {fc_a}. Adapter-A should always be dispatched first regardless of the collision fix."
    );
    assert!(
        wire_text.contains("\"alpha-001\""),
        "SLUG-005 wire isolation control: wire output must also contain 'alpha-001' rows from \
         tenant-alpha. Got: {wire_text}."
    );
}

// ---------------------------------------------------------------------------
// RG-PSG-040: MCP wire-level partial-final-page is_truncated=false
// ---------------------------------------------------------------------------

/// RG-PSG-040 — AC-014 (BC-2.11.001 EC-11-094) wire-level: `prism_query` MCP
/// `CallToolResult.content[0].text` JSON carries `"is_truncated":false` for a
/// `LIMIT 5` query where the sensor returns exactly 5 records on a page_size=1000
/// spec (PARTIAL final page — page_record_count=5 < page_size=1000).
///
/// ## What this test verifies (ADR-060 §D8.2 discriminator, wire surface)
///
/// Uses a REAL `SpecDrivenSensorAdapter` backed by wiremock so the pipeline
/// traverses `PipelineExecutor::execute_impl` — the production code path that
/// contains the partial-final-page discriminator bug.  A pure mock adapter that
/// hardcodes `any_early_stopped` would not automatically turn GREEN when
/// `pipeline.rs` is fixed; this test does.
///
/// ## Execution chain (RED state, pre-discriminator)
///
/// 1. Query `SELECT * FROM rg040sensor_items LIMIT 5` → fetch_limit = 5.
/// 2. `SpecDrivenSensorAdapter::fetch` → `PipelineExecutor::execute_impl`
///    with `early_stop_limit = Some(5)`, `page_size = 1000`.
/// 3. wiremock returns 5 records (PARTIAL: page_record_count=5 < page_size=1000).
/// 4. `all_records.len() (5) >= early_stop_limit (5)` → early-stop fires → BREAK.
/// 5. Current code: `early_stopped = true` (unconditional) → `any_early_stopped = true`.
/// 6. Engine Step 6: `is_truncated = (5 > 5) OR true = true`.
/// 7. Wire JSON: `"is_truncated":true`.
/// 8. Assertion `contains("\"is_truncated\":false")` → **FAILS** → RED gate.
///
/// ## Execution chain (GREEN state, post-discriminator, ADR-060 §D8.2)
///
/// 5. Fix: `early_stopped = (page_record_count(5) >= page_size(1000)) = false`.
/// 6. `any_early_stopped = false` → `is_truncated = (5 > 5) OR false = false`.
/// 7. Wire JSON: `"is_truncated":false`.
/// 8. Assertion → **PASSES**.
///
/// ## SAP-3 compliance
///
/// Reaches `PrismServer::query` end-to-end from a SQL query string, through
/// `SpecDrivenSensorAdapter` → `PipelineExecutor::execute_impl` (not a synthetic
/// AST or direct pipeline call).
///
/// ## Traces
///
/// - BC-2.11.001 EC-11-094 (PARTIAL-final-page wire arm)
/// - BC-2.16.002 EC-01-041 (PARTIAL-final-page pipeline arm)
/// - AC-014 of S-ENGINE-LIMIT-EARLY-STOP-001
/// - ADR-060 §D8.2
#[tokio::test]
async fn test_psg_rg040_partial_final_page_is_truncated_false_wire() {
    use prism_bin::spec_driven_adapter::{AdapterAuthStrategy, SpecDrivenSensorAdapter};
    use prism_core::column::ColumnType;
    use prism_sensors::BearerStaticSensorAuth;
    use prism_spec_engine::{
        overlay::{OverlayLoader, SensorInstanceOverlay},
        spec_parser::{
            AuthType, ColumnSpec, FetchStep, PaginationConfig, SensorSpec as PeSensorSpec,
            TableSpec,
        },
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    // -----------------------------------------------------------------------
    // BearerStaticCredentialResolver: resolves to BearerStaticSensorAuth.
    //
    // `SpecDrivenSensorAdapter` with `AdapterAuthStrategy::BearerStatic`
    // downcasts `&dyn SensorAuth` to `BearerStaticSensorAuth` in its fetch impl.
    // `StubCredentialResolver` (used elsewhere in this file) returns `TestStubAuth`,
    // which would fail that downcast with a SensorError::Internal.  A dedicated
    // resolver is required for BearerStatic adapters.
    // -----------------------------------------------------------------------
    struct BearerStaticCredentialResolver;
    impl CredentialResolver for BearerStaticCredentialResolver {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_id: SensorId,
        ) -> Result<Box<dyn SensorAuth>, SensorError> {
            Ok(Box::new(BearerStaticSensorAuth::new("rg040-test-token")))
        }
    }

    // -----------------------------------------------------------------------
    // wiremock: page_size=1000, returns exactly 5 records (PARTIAL page).
    // -----------------------------------------------------------------------
    let mock_server = MockServer::start().await;

    // Single data page: 5 records.
    //   PARTIAL: page_record_count=5 < page_size=1000.
    //   Cumulative: 5 >= early_stop_limit(5) → pipeline breaks after this page.
    let partial_page: Vec<serde_json::Value> = (0u32..5)
        .map(|i| serde_json::json!({"id": i.to_string()}))
        .collect();

    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": partial_page })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Terminal fallback (defensive — should NOT be reached; break fires after page 1).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "items": [] })))
        .mount(&mock_server)
        .await;

    // -----------------------------------------------------------------------
    // Build SpecDrivenSensorAdapter:
    //   sensor_id = "rg040sensor", table = "items", page_size = 1000.
    //
    // Query routing:
    //   `sensor_id_from_table_name("rg040sensor_items")` → "rg040sensor" ✓
    //   `queried_table_name = "rg040sensor_items".strip_prefix("rg040sensor_")` = "items" ✓
    // -----------------------------------------------------------------------
    let sensor_spec = PeSensorSpec::new(
        "rg040sensor",
        "RG-PSG-040 partial-final-page discriminator sensor",
        AuthType::BearerStatic,
        &mock_server.uri(),
        vec![TableSpec::new_point_in_time(
            "items",
            "security_finding",
            vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
            vec![FetchStep::new(
                "fetch_items",
                "GET",
                "/items",
                None,
                "$.items",
                None,
                vec![],
                None,
                Some(PaginationConfig::OffsetLimit { page_size: 1000 }),
            )],
        )],
        None,
        "1.0.0",
        vec![],
    );

    // Build ResolvedSensorSpec via OverlayLoader — only external construction path
    // (ResolvedSensorSpec is #[non_exhaustive]; struct literal syntax forbidden).
    let overlay_toml = format!(
        "extends = \"{}\"\ninstance_id = \"{}@{}\"",
        sensor_spec.sensor_id, sensor_spec.sensor_id, "rg040-org"
    );
    let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
        .expect("RG-PSG-040: SensorInstanceOverlay TOML parse must succeed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(
        &sensor_spec,
        &overlay,
        OrgSlug::new("rg040-org"),
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("RG-PSG-040: reqwest Client build must succeed");

    let adapter: Arc<dyn prism_sensors::SensorAdapter> = Arc::new(SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    ));

    // -----------------------------------------------------------------------
    // Register adapter and build engine with BearerStaticCredentialResolver.
    // -----------------------------------------------------------------------
    let org_id = OrgId::new();
    let org_slug = OrgSlug::new_unchecked("rg040-org");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);

    let adapter_arc = Arc::new(registry);
    let credential_store: Arc<dyn CredentialStore> = Arc::new(NullCredentialStore);
    let ocsf_normalizer = Arc::new(OcsfNormalizer::new());
    let client_registry = Arc::new(ClientRegistry::new(vec![org_slug]));
    let config = QueryEngineConfig::default();
    let engine = QueryEngine::new(
        adapter_arc,
        credential_store,
        ocsf_normalizer,
        client_registry,
        config,
    )
    .with_credential_resolver(Arc::new(BearerStaticCredentialResolver));

    let server = PrismServer::new().with_query_engine(Arc::new(engine));

    // -----------------------------------------------------------------------
    // Query: SELECT * FROM rg040sensor_items LIMIT 5
    //
    // params.limit = 5 → fetch_limit = 5 → early_stop_limit = Some(5).
    // wiremock returns 5 records (PARTIAL page, page_size=1000).
    // -----------------------------------------------------------------------
    let params: QueryToolParams = serde_json::from_value(serde_json::json!({
        "query":         "SELECT * FROM rg040sensor_items LIMIT 5",
        "clients":       ["rg040-org"],
        "limit":         5u32,
        "force_refresh": false
    }))
    .expect("RG-PSG-040: QueryToolParams must deserialize from known-field JSON object");

    let call_result = server.query(Parameters(params)).await.expect(
        "RG-PSG-040: PrismServer::query must return Ok(CallToolResult) for partial-page \
         LIMIT 5 query against SpecDrivenSensorAdapter (Err here means an internal \
         PrismError — check auth wiring or engine construction)",
    );

    // Precondition: wiremock must have been called (non-vacuous assertion).
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock: received_requests() must succeed");
    assert!(
        !received.is_empty(),
        "RG-PSG-040 precondition: wiremock must have received at least 1 HTTP request \
         (0 requests means the pipeline short-circuited before reaching the HTTP adapter \
         and all wire assertions are vacuous). Check auth wiring or sensor ID routing."
    );

    // Extract the ACTUAL wire bytes from the real SafetyEnvelopeBuilder serialization.
    let wire_text = call_result
        .content
        .first()
        .and_then(|c| c.as_text())
        .map(|t| t.text.clone())
        .expect(
            "RG-PSG-040: CallToolResult::structured must produce a text content[0] item \
             (the serialized SafetyEnvelope JSON)",
        );

    // PRIMARY ASSERTION (RED gate driver — AC-014 / ADR-060 §D8.2 / BC-2.11.001 EC-11-094):
    //
    // RED  (pre-discriminator, current pipeline.rs):
    //   `early_stopped = true` (unconditional when cumulative >= early_stop_limit)
    //   → FetchOutput.any_early_stopped = true
    //   → Engine Step 6: is_truncated = (total_rows(5) > limit(5)) OR true = true
    //   → wire JSON: "is_truncated":true
    //   → assertion `contains("\"is_truncated\":false")` → FAILS → RED gate.
    //
    // GREEN (post-discriminator, ADR-060 §D8.2):
    //   `early_stopped = (page_record_count(5) >= page_size(1000)) = false`
    //   → FetchOutput.any_early_stopped = false
    //   → Engine Step 6: is_truncated = (5 > 5) OR false = false
    //   → wire JSON: "is_truncated":false
    //   → assertion PASSES.
    assert!(
        wire_text.contains("\"is_truncated\":false"),
        "RG-PSG-040 (AC-014 RED gate — ADR-060 §D8.2 discriminator, wire-level): \
         CallToolResult.content[0].text must contain \"is_truncated\":false when the \
         final page is PARTIAL (page_record_count=5 < page_size=1000). \
         Got wire JSON: {wire_text}. \
         \n\nDiagnosis: Current pipeline.rs early-stop block sets `early_stopped = true` \
         unconditionally when `all_records.len() >= early_stop_limit`, regardless of \
         whether the final page was full or partial. \
         Fix: `early_stopped = (page_record_count >= page_size)` (ADR-060 §D8.2). \
         With 5 records on page_size=1000: `early_stopped = (5 >= 1000) = false` → \
         any_early_stopped=false → is_truncated = (total_rows > limit) OR false = false."
    );
}
