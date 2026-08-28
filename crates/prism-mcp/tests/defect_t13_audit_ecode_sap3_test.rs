//! SAP-3 wire-level regression tests for DEFECT-T13-AUDIT-ECODE-EXPECTATIONS-001.
//!
//! The T13 pre-flight audit reported two false FAILs ([G4] and [H8]) caused by the
//! audit instrument reading the message-text regex scrape instead of the canonical
//! `structuredContent.error.code`.  These tests lock the wire-level behaviour that
//! the engine is **spec-correct** — they are regression coverage, NOT Red Gate tests.
//!
//! ## SAP-3 / SID-2 compliance
//!
//! Each test satisfies SAP-3 (spec-arm reachability) and SID-2 (composed-output
//! assertions) per CLAUDE.md §Standing Adversary Probes & Implementer Disciplines.
//!
//! - **[G4] coverage (two-layer)**:
//!   1. `test_sap3_sql_mode_ieq_e2e_routing` — **end-to-end routing test** (SAP-3
//!      primary): drives the IEQ SQL query through `QueryEngine::execute` (full
//!      parser → `run_materialization_pipeline` → `PrismError::QueryParseFailed`
//!      routing) and asserts the `detail` field is de-prefixed per ADR-048 §D.7.2.
//!      This catches a materialization regression that stops routing SQL-IEQ parse
//!      errors into `PrismError::QueryParseFailed`.
//!   2. `test_sap3_sql_mode_ieq_rejection_wire_shape` — **wire-shape defense-in-depth**:
//!      hand-builds `PrismError::QueryParseFailed` after calling `PrismQlParser::parse`
//!      directly, then drives it through `prism_error_to_structured_call_result` (the
//!      error-response constructor) and asserts on the SERIALISED JSON output.
//!
//! - **[H8] coverage (two-layer)**:
//!   1. `test_sap3_head_join_bare_unknown_col_plan_suspension` — **end-to-end
//!      planner test** (SAP-3 primary): drives a HEAD-JOIN + bare-unknown-col query
//!      through `QueryEngine::execute` (parser → plan gates) and asserts the
//!      plan-time E-QUERY-038 gate is SUSPENDED (fail-open per BC-2.11.016 §FP-001).
//!      This catches a planner regression where the suspension arm is removed.
//!   2. `test_sap3_head_join_bare_unknown_col_wire_shape` — **error-mapping
//!      defense-in-depth**: hand-builds `QueryExecutionFailed` and drives it through
//!      `prism_error_to_structured_call_result`, asserting the JSON code field is
//!      "E-QUERY-034" (not "E-QUERY-038").  Catches a mapping regression in
//!      `error_mapping.rs`; cannot catch a planner regression on its own.
//!
//! - **SID-2**: at least one assertion covers the FULL composed `content[].text` string.
//!
//! ## Wire-shape discipline
//!
//! Wire-shape assertions target the SERIALISED JSON bytes — the exact envelope the
//! LLM agent consumes — via `serde_json::to_string` on `structured_content`
//! (CLAUDE.md §Conventions wire-shape assertion discipline, 2026-07-13).
//!
//! # Test → defect mapping
//!
//! | Test | Defect check | BC |
//! |------|--------------|----|
//! | test_sap3_sql_mode_ieq_e2e_routing                 | [G4]  | BC-2.11.017 AC-003 / BC-2.11.024 (e2e routing — SAP-3 primary) |
//! | test_sap3_sql_mode_ieq_rejection_wire_shape        | [G4]  | BC-2.11.017 AC-003 / BC-2.11.024 (wire-shape defense-in-depth) |
//! | test_sap3_head_join_bare_unknown_col_plan_suspension | [H8] | BC-2.11.016 §FP-001 (planner gate) |
//! | test_sap3_head_join_bare_unknown_col_wire_shape    | [H8]  | BC-2.11.016 §FP-001 / BC-2.10.007 |

#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::sync::Arc;

use arrow::{
    array::StringArray,
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use async_trait::async_trait;
use prism_core::{column::ColumnType, error::PrismError, OrgId, OrgSlug, SensorId};
use prism_credentials::InMemoryCredentialStore;
use prism_mcp::error_mapping::prism_error_to_structured_call_result;
use prism_query::{
    engine::{QueryEngine, QueryEngineConfig, QueryOptions},
    scoping::ClientRegistry,
    table_registry::TableRegistry,
    PrismQlParser,
};
use prism_sensors::{
    adapter::FetchOutput, AdapterRegistry, CredentialResolver, QueryParams as SensorQueryParams,
    SensorAdapter, SensorAuth, SensorError, SensorSpec as SensorAdapterSpec,
};
use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec};

// ── Stub types for execution-capable JOIN fixture ─────────────────────────────
//
// Required by OPTION A (fix-burst #8 re-triage): to replicate the live [H8] outcome
// (Err(QueryExecutionFailed) for totally_unknown_col), the fixture must supply
// adapter data so DataFusion registers non-empty MemTables and validates column
// references at plan time.  With an empty AdapterRegistry, DataFusion receives 0
// rows and returns Ok(empty) without schema validation — the silent-swallow path
// the live [H8] audit correctly flags as FAIL-DEFECT (BC-2.11.016 §HEAD-JOIN
// SUSPENSION RULE: fail-open defers to execution-time DataFusion error, NOT 0-row success).

/// Stub auth token — ignored by `ReturnsOneRowAdapter::fetch`.
struct StubAuth;

impl SensorAuth for StubAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn auth_type_name(&self) -> &'static str {
        "custom_via_plugin"
    }
}

/// Credential resolver that always succeeds (returns `StubAuth`).
///
/// Required so `fan_out()` reaches the adapter boundary rather than
/// short-circuiting with a `CredentialNotFound` error.  The stub auth is
/// ignored by `ReturnsOneRowAdapter::fetch`.  Pattern from `normalized_pql.rs`
/// `AlwaysSucceedsCreds` (SID-1 compliance).
struct AlwaysSucceedsCreds;

impl CredentialResolver for AlwaysSucceedsCreds {
    fn resolve(
        &self,
        _client_id: &str,
        _sensor_id: SensorId,
    ) -> Result<Box<dyn SensorAuth>, SensorError> {
        Ok(Box::new(StubAuth))
    }
}

/// Stub sensor adapter that returns exactly one row with the given Arrow schema.
///
/// Every column is filled with the static string `"stub"` — sufficient for
/// DataFusion to register a non-empty MemTable and validate WHERE column
/// references.  The adapter ignores `_spec`, `_params`, and `_auth`.
struct ReturnsOneRowAdapter {
    sensor_id: SensorId,
    schema: Arc<Schema>,
}

#[async_trait]
impl SensorAdapter for ReturnsOneRowAdapter {
    fn sensor_type(&self) -> SensorId {
        self.sensor_id.clone()
    }

    fn sensor_name(&self) -> &'static str {
        "returns-one-row-stub"
    }

    async fn fetch(
        &self,
        _spec: &SensorAdapterSpec,
        _params: &SensorQueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<FetchOutput, SensorError> {
        let n_cols = self.schema.fields().len();
        let arrays: Vec<Arc<dyn arrow::array::Array>> = (0..n_cols)
            .map(|_| Arc::new(StringArray::from(vec!["stub"])) as Arc<dyn arrow::array::Array>)
            .collect();
        let batch = RecordBatch::try_new(Arc::clone(&self.schema), arrays)
            .expect("ReturnsOneRowAdapter: stub RecordBatch construction must not fail");
        Ok(FetchOutput::new(vec![batch], false))
    }
}

// ── Helper: extract content[0].text from a CallToolResult ────────────────────

fn content_text(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.as_str().to_owned()))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Fixture: QueryEngine with crowdstrike_alerts + secondary_table ───────────

/// Build an execution-capable single-tenant `QueryEngine` with two tables and
/// their matching stub adapters registered:
///   - `crowdstrike_alerts` (columns: `severity` String, `timestamp` String)
///   - `secondary_table`   (columns: `col` String, `id` String)
///
/// Each table is backed by a `ReturnsOneRowAdapter` that returns exactly one
/// schema-conforming row.  This makes the AdapterRegistry non-empty, which
/// causes `resolve_source_refs` to create FanOutTargets, which causes DataFusion
/// to receive real RecordBatch data and validate WHERE column references at plan
/// time.
///
/// Without real data (empty AdapterRegistry), `resolve_source_refs` silently
/// skips fan-out for sensors with no adapters (BC-2.11.011 EC-005), DataFusion
/// receives 0 rows, and returns Ok(empty) without schema validation — the
/// silent-swallow case that BC-2.11.016 §HEAD-JOIN SUSPENSION RULE flags as
/// FAIL-DEFECT.
///
/// OPTION A (fix-burst #8 re-triage): execution-capable fixture so the test
/// mirrors the live [H8] contract end-to-end.
///
/// Used by `test_sap3_head_join_bare_unknown_col_plan_suspension`.
fn make_join_engine() -> (QueryEngine, OrgSlug) {
    let org = OrgSlug::new("acme");

    // Deterministic OrgId (sentinel byte 0xA8 — JOIN fixture).
    // Pattern from bc_2_11_001_null_row_shape_test.rs and normalized_pql.rs.
    let org_id = OrgId::from_uuid(uuid::Uuid::from_bytes([
        0x01, 0x9f, 0x3a, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xA8,
    ]));

    // ── Primary: crowdstrike_alerts ───────────────────────────────────────────
    let cs_spec = SensorSpec::new(
        "crowdstrike",
        "CrowdStrike sensor",
        AuthType::ApiKey,
        "https://api.crowdstrike.com",
        vec![TableSpec::new_point_in_time(
            "alerts",
            "security_finding",
            vec![
                ColumnSpec::new("severity", ColumnType::String, None, vec![]),
                ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
            ],
            vec![],
        )],
        None,
        "1.0.0",
        Vec::new(),
    );

    // ── JOIN target: secondary_table ──────────────────────────────────────────
    // sensor_id must NOT contain underscores: `sensor_id_from_table_name` splits at
    // the first underscore to extract the sensor prefix.  "secondary" + table "table"
    // → full name "secondary_table" → prefix "secondary" ✓ (registered in AdapterRegistry).
    let jo_spec = SensorSpec::new(
        "secondary",
        "Some Other sensor",
        AuthType::ApiKey,
        "https://api.example.com",
        vec![TableSpec::new_point_in_time(
            "table",
            "some_category",
            vec![
                ColumnSpec::new("col", ColumnType::String, None, vec![]),
                ColumnSpec::new("id", ColumnType::String, None, vec![]),
            ],
            vec![],
        )],
        None,
        "1.0.0",
        Vec::new(),
    );

    let registry = Arc::new(TableRegistry::new());
    registry
        .register_sensor(&cs_spec)
        .expect("make_join_engine: register crowdstrike_alerts must not fail");
    registry
        .register_sensor(&jo_spec)
        .expect("make_join_engine: register secondary_table must not fail");

    // ── Execution-capable adapters: each returns 1 row with the declared schema ─
    // Arrow schema matches the declared columns (severity/timestamp for crowdstrike,
    // col/id for secondary).  `totally_unknown_col` is absent from BOTH schemas;
    // DataFusion fails at plan time with a schema error → Err(QueryExecutionFailed).
    let cs_schema = Arc::new(Schema::new(vec![
        Field::new("severity", DataType::Utf8, true),
        Field::new("timestamp", DataType::Utf8, true),
    ]));
    let jo_schema = Arc::new(Schema::new(vec![
        Field::new("col", DataType::Utf8, true),
        Field::new("id", DataType::Utf8, true),
    ]));
    let cs_adapter: Arc<dyn SensorAdapter> = Arc::new(ReturnsOneRowAdapter {
        sensor_id: SensorId::new("crowdstrike"),
        schema: cs_schema,
    });
    let jo_adapter: Arc<dyn SensorAdapter> = Arc::new(ReturnsOneRowAdapter {
        sensor_id: SensorId::new("secondary"),
        schema: jo_schema,
    });

    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, cs_adapter);
    adapter_registry.register(org_id, jo_adapter);

    let engine = QueryEngine::new_with_cache_config(
        Arc::new(adapter_registry),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        prism_query::cache::CacheConfig::default(),
    )
    // AlwaysSucceedsCreds: required so fan_out() reaches the adapter boundary
    // rather than short-circuiting on CredentialNotFound (SID-1 pattern).
    .with_credential_resolver(Arc::new(AlwaysSucceedsCreds))
    .with_table_registry(registry);

    (engine, org)
}

// ── Fixture: minimal QueryEngine (no sensor tables) ──────────────────────────

/// Build a `QueryEngine` with no sensor tables registered.
///
/// Sufficient for parse-error gate testing: `PrismError::QueryParseFailed` is
/// returned by `run_materialization_pipeline` before any table-registry lookup,
/// so no registered sensors are needed.
fn make_minimal_engine() -> QueryEngine {
    QueryEngine::new_with_cache_config(
        Arc::new(AdapterRegistry::new()),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        prism_query::cache::CacheConfig::default(),
    )
}

// ── Test A (SAP-3 primary): [G4] end-to-end SQL-IEQ routing ──────────────────

/// SAP-3 end-to-end routing test: `QueryEngine::execute` with a SQL-mode IEQ query
/// must return `Err(PrismError::QueryParseFailed)` with the detail de-prefixed per
/// ADR-048 §D.7.2 (no leading "E-QUERY-001: " in the `detail` field).
///
/// This is the **primary SAP-3 gate** for [G4]: it drives through
/// `execute` → `execute_inner` → `run_materialization_pipeline` (the
/// `QueryParseFailed` arm of the error-construction path in `materialization.rs`)
/// and proves that a materialization regression that stops routing SQL-IEQ parse
/// errors into `PrismError::QueryParseFailed` (e.g., returning a different variant,
/// or failing to strip the de-prefix) will be caught here.
///
/// The existing `test_sap3_sql_mode_ieq_rejection_wire_shape` is **defense-in-depth**
/// only (it hand-builds `QueryParseFailed` and does not drive through
/// `run_materialization_pipeline`); this test provides the load-bearing end-to-end coverage.
///
/// BC-2.11.017 AC-003 / BC-2.11.024 / ADR-048 §D.7.2.
#[tokio::test]
async fn test_sap3_sql_mode_ieq_e2e_routing() {
    let engine = make_minimal_engine();

    // SQL-mode IEQ query: must be rejected at parse time inside
    // run_materialization_pipeline (materialization.rs QueryParseFailed arm).
    let query = "SELECT severity, count(*) FROM cyberint_alerts \
                 WHERE severity IEQ 'high' GROUP BY severity";

    let result = engine
        .execute(
            query,
            QueryOptions {
                clients: None,
                ..QueryOptions::default()
            },
        )
        .await;

    // ── SAP-3 gate: must route to QueryParseFailed ────────────────────────────
    match result {
        Err(PrismError::QueryParseFailed {
            ref detail,
            offset: _,
            query: _,
        }) => {
            // ── De-prefix gate (ADR-048 §D.7.2): detail must NOT carry "E-QUERY-001: " ──
            // run_materialization_pipeline strips the "E-QUERY-001: " prefix before
            // injecting into detail (the `#[error]` template adds it back in the
            // Display form).  If this fails, the strip in run_materialization_pipeline
            // is broken (ADR-048 §D.7.2 de-prefix discipline).
            assert!(
                !detail.starts_with("E-QUERY-001: "),
                "SAP-3 [G4] e2e: QueryParseFailed.detail must NOT start with \
                 'E-QUERY-001: ' (ADR-048 §D.7.2 de-prefix discipline — \
                 run_materialization_pipeline strips the prefix before injection). \
                 Got detail: {:?}",
                detail.chars().take(120).collect::<String>()
            );

            // ── Pedagogical content gate ───────────────────────────────────────
            // The detail should contain the mode-boundary message fragment from
            // the SQL parser (BC-2.11.024 / sql_parser.rs SQL-mode rejection).
            assert!(
                detail.to_lowercase().contains("not supported in sql mode")
                    || detail.to_lowercase().contains("ieq"),
                "SAP-3 [G4] e2e: QueryParseFailed.detail must contain mode-boundary \
                 pedagogy ('not supported in SQL mode' or 'IEQ'). \
                 Got detail: {:?}",
                detail.chars().take(120).collect::<String>()
            );
        }
        Err(other) => {
            panic!(
                "SAP-3 [G4] e2e ROUTING RED GATE: QueryEngine::execute with SQL-mode IEQ \
                 must return Err(PrismError::QueryParseFailed); got Err({other:?}). \
                 This means run_materialization_pipeline (materialization.rs) did NOT \
                 route the IEQ parse error into QueryParseFailed. Check the \
                 QueryParseFailed arm of the error-construction path in \
                 run_materialization_pipeline (ADR-048 §D.7.2 / BC-2.11.024)."
            );
        }
        Ok(_) => {
            panic!(
                "SAP-3 [G4] e2e ROUTING RED GATE: QueryEngine::execute with SQL-mode IEQ \
                 must return Err; got Ok. The SQL parser must reject IEQ in SQL mode \
                 (BC-2.11.017 AC-003 / BC-2.11.024 SQL-Mode Rejection)."
            );
        }
    }
}

// ── Test A (defense-in-depth): [G4] SQL-mode IEQ rejection → wire-shape ──────

/// SAP-3 wire-shape defense-in-depth: SQL-mode IEQ rejection emits
/// `code == "E-QUERY-001"` in `structuredContent.error`, while `content[].text`
/// carries NO E-code (BC-2.10.007 message/suggestion split — the canonical code
/// lives in structured content only).
///
/// **Scope (defense-in-depth)**: this test hand-builds `PrismError::QueryParseFailed`
/// after calling `PrismQlParser::parse` directly, then drives it through
/// `prism_error_to_structured_call_result` (the error-response constructor).  It
/// does NOT drive through `run_materialization_pipeline`; a regression that stops
/// routing SQL-IEQ parse errors into `QueryParseFailed` would NOT be caught here.
/// That regression is caught by `test_sap3_sql_mode_ieq_e2e_routing` (the primary
/// SAP-3 gate for [G4]).
///
/// Defect: T13 audit check [G4] was a false FAIL because `parse_envelope` read the
/// regex-scraped code from message text ("PrismQL parse error: ...") which contains
/// no E-code, yielding "UNKNOWN".  The canonical code lives in
/// `structuredContent.error.code == "E-QUERY-001"` (via `ec_code_override` in
/// `error_mapping.rs`).
///
/// Coupling note (OBS-2 / ADR-048 §D.7.2 de-prefix discipline): the production
/// path (`run_materialization_pipeline` in `materialization.rs`) strips
/// `"E-QUERY-001: "` from the SQL parser's `ParseError.message` before injecting
/// it into `QueryParseFailed.detail`, preventing the prefix from doubling in the
/// Display template.  This test mimics that stripping so it exercises the same
/// wire shape the live audit sees.
/// The assertion at Step 2a below pins this coupling: if the SQL parser STOPS
/// prefixing messages with "E-QUERY-001: ", the assertion fires and alerts
/// maintainers that the de-prefix strip in `run_materialization_pipeline` is now
/// a no-op (and the ADR-048 §D.7.2 strip logic should be reviewed).
///
/// SID-2: the full composed `content[].text` is asserted (not only the `code` field).
///
/// BC-2.11.017 AC-003 / BC-2.11.024 / ADR-047.
#[test]
fn test_sap3_sql_mode_ieq_rejection_wire_shape() {
    // ── Step 1: confirm the parser rejects IEQ in SQL WHERE (SAP-3 reachability) ──
    let query = "SELECT severity, count(*) FROM cyberint_alerts WHERE severity IEQ 'high' GROUP BY severity";
    let parse_result = PrismQlParser::parse(query);
    assert!(
        parse_result.is_err(),
        "SAP-3 [G4]: IEQ in SQL WHERE must be rejected by PrismQlParser::parse; \
         got Ok (parser regression)"
    );
    let parse_errors = parse_result.unwrap_err();
    let first = parse_errors
        .first()
        .expect("SAP-3 [G4]: parse must return at least one error");

    // ── Step 2: build the MCP error via the production path ──────────────────
    // ADR-048 §D.7.2 / materialization.rs de-prefix discipline: production code strips
    // "E-QUERY-001: " from parse error messages before injecting into QueryParseFailed.detail
    // to prevent doubling by the `#[error]` template.  Mimic that here so the test
    // exercises the same wire shape the live audit sees.

    // 2a. COUPLING PIN (OBS-2): assert that the SQL parser DOES prefix messages with
    //     "E-QUERY-001: " so the strip_prefix below is load-bearing.  If this assertion
    //     fails, `sql_parser.rs` changed its error format and the ADR-048 §D.7.2
    //     de-prefix strip in `run_materialization_pipeline` (`materialization.rs`)
    //     should be revisited.
    assert!(
        first.message.starts_with("E-QUERY-001: "),
        "SAP-3 [G4] de-prefix coupling pin: SQL parser must emit ParseError.message \
         starting with 'E-QUERY-001: ' (ADR-048 §D.7.2 / sql_parser.rs BC-2.11.024 §SQL-Mode \
         Rejection). Got: {:?}. If this assertion fails, review the de-prefix strip in \
         materialization.rs and update the coupling here.",
        first.message.chars().take(80).collect::<String>()
    );

    let detail = first
        .message
        .strip_prefix("E-QUERY-001: ")
        .unwrap_or(&first.message)
        .to_string();
    let err = PrismError::QueryParseFailed {
        query: query.to_string(),
        offset: first.offset,
        detail,
    };
    let result = prism_error_to_structured_call_result(err);

    // ── Step 3: wire-level assertion on SERIALISED JSON ──────────────────────
    let sc = result
        .structured_content
        .as_ref()
        .expect("SAP-3 [G4]: structuredContent must be present (BC-2.10.007)");

    // Serialise to JSON — this is the exact envelope the LLM agent receives.
    let serialized =
        serde_json::to_string(sc).expect("SAP-3 [G4]: structured_content must serialise");

    // 3a. structuredContent.error.code MUST be "E-QUERY-001" (ec_code_override pin).
    let code = sc
        .get("error")
        .and_then(|e| e.get("code"))
        .and_then(|v| v.as_str())
        .expect("SAP-3 [G4]: structuredContent.error.code must be present");
    assert_eq!(
        code, "E-QUERY-001",
        "SAP-3 [G4]: structuredContent.error.code must be 'E-QUERY-001' (BC-2.11.017 AC-003 \
         ec_code_override); got {code:?}. \
         Serialised structuredContent: {serialized}"
    );

    // 3b. The message text must contain mode-boundary pedagogy naming the operator.
    //     The detail from parse_sql carries "not supported in SQL mode".
    assert!(
        serialized.contains("E-QUERY-001"),
        "SAP-3 [G4]: 'E-QUERY-001' must appear in serialised structuredContent (code field); \
         serialised: {serialized}"
    );

    // ── Step 4: content[].text wire assertions (SID-2 composed-output) ───────
    let text = content_text(&result);

    // 4a. content[].text MUST NOT contain "E-QUERY-001" — the E-code belongs in
    //     structuredContent only (BC-2.10.007 message/suggestion split).
    assert!(
        !text.contains("E-QUERY-001"),
        "SAP-3 [G4]: content[].text must NOT contain 'E-QUERY-001' — the E-code lives in \
         structuredContent.error.code, not in the human-readable content text \
         (BC-2.10.007 message/suggestion split). Got text: {text:?}"
    );

    // 4b. content[].text MUST contain the mode-boundary message fragment.
    //     The SQL parser emits "not supported in SQL mode" in the error detail.
    assert!(
        text.contains("not supported in SQL mode"),
        "SAP-3 [G4]: content[].text must contain 'not supported in SQL mode' \
         (pedagogical mode-boundary message per sql_parser.rs); got: {text:?}"
    );

    // 4c. SID-2: the full composed content[].text must contain "IEQ" (operator pedagogy).
    assert!(
        text.to_uppercase().contains("IEQ"),
        "SAP-3 [G4]: content[].text must name the IEQ operator (mode-boundary pedagogy \
         per BC-2.11.024 / ADR-047); got: {text:?}"
    );

    // 4d. SID-2: the full content[].text must start with "ERROR: [" (BC-2.10.007 format).
    assert!(
        text.starts_with("ERROR: ["),
        "SAP-3 [G4]: content[].text must start with 'ERROR: [' (BC-2.10.007 content_text \
         format 'ERROR: [{{category}}] - ...'). Got: {:?}",
        text.chars().take(60).collect::<String>()
    );
}

// ── Test B2: [H8] SAP-3 planner-level — HEAD-JOIN suspension gate ─────────────

/// SAP-3 planner-level test: a SQL query with a non-empty HEAD JOIN list and a
/// bare unqualified column reference absent from the FROM table MUST NOT fire
/// `PrismError::ColumnNotFound` (E-QUERY-038) at plan time.
///
/// This is the **primary SAP-3 coverage** for BC-2.11.016 §HEAD-JOIN SUSPENSION RULE:
/// it drives from the public `QueryEngine::execute` surface through the full
/// parser + planner path and asserts that `check_query_column_availability` suspends
/// the E-QUERY-038 gate (fail-open per FP-001) when `head_has_joins = true` and the
/// referenced column (`totally_unknown_col`) is absent from the FROM table schema.
///
/// A planner regression that removes or narrows the suspension arm would cause this
/// test to fail (E-QUERY-038 would surface as `Err(PrismError::ColumnNotFound)`),
/// whereas the synthetic Test B below (`test_sap3_head_join_bare_unknown_col_wire_shape`)
/// would continue to pass — demonstrating that this test is the load-bearing SAP-3 gate.
///
/// **Anti-vacuous-pass guards (PR-LEVEL OBS-1):** the test explicitly rejects
/// `Err(PrismError::QueryParseFailed)` (planner never reached — vacuous pass) and
/// `Err(PrismError::TableNotAvailable)` (column gate never reached — vacuous pass).
/// An `Ok(_)` result is also rejected as a silent-swallow signal (BC-2.11.016
/// §HEAD-JOIN SUSPENSION RULE / live T13 [H8] parity: 0-rows-no-error = FAIL-DEFECT).
///
/// Engine fixture: `make_join_engine()` — two tables registered in `TableRegistry`,
/// populated `AdapterRegistry` with `ReturnsOneRowAdapter` stubs (each returns 1
/// schema-conforming row per fetch) so DataFusion receives non-empty MemTables and
/// evaluates the WHERE schema at execution time. Single-tenant mode
/// (`resolved_spec_map = None`).
///
/// T13 live-audit coverage for [H8]: `scripts/t13-preflight-audit.py` check [H8]
/// exercises the full HEAD-JOIN execution path against a live DTU endpoint; this
/// in-process test provides continuous regression coverage without a live DTU.
///
/// BC-2.11.016 §HEAD-JOIN SUSPENSION RULE / §FP-001 / BC-2.10.007.
#[tokio::test]
async fn test_sap3_head_join_bare_unknown_col_plan_suspension() {
    let (engine, _org) = make_join_engine();

    // Query with HEAD JOIN + bare unqualified ref "totally_unknown_col" absent from
    // crowdstrike_alerts schema.  HEAD-JOIN SUSPENSION RULE: non-empty join list →
    // E-QUERY-038 gate MUST be suspended for this bare ref (fail-open per FP-001).
    let query = "SELECT severity FROM crowdstrike_alerts \
                 JOIN secondary_table ON crowdstrike_alerts.severity = secondary_table.id \
                 WHERE totally_unknown_col = 'foo'";

    let result = engine
        .execute(
            query,
            QueryOptions {
                clients: None,
                ..QueryOptions::default()
            },
        )
        .await;

    // ── Anti-vacuous-pass guard (PR-LEVEL OBS-1) ─────────────────────────────
    // If the query fails at PARSE time, the planner is never reached and a
    // suspension-arm regression is invisible — the test would pass vacuously.
    // Explicitly reject QueryParseFailed to prove reachability.
    if let Err(PrismError::QueryParseFailed {
        ref detail, offset, ..
    }) = result
    {
        panic!(
            "SAP-3 [H8] VACUOUS-PASS GUARD: query failed at PARSE time (offset={offset}), \
             planner never reached — suspension-arm regression would be invisible. \
             QueryParseFailed detail: {:?}. \
             If SQL syntax changed, update the query in this test; \
             if a grammar change caused this, investigate before loosening the guard.",
            detail.chars().take(200).collect::<String>()
        );
    }

    // ── Anti-vacuous-pass guard: TableNotAvailable (E-QUERY-037) ─────────────
    // TableNotAvailable fires BEFORE the column gate in execute_inner.  Unlike the
    // QueryParseFailed guard above (which is provably unreachable when the SQL is
    // syntactically valid), TableNotAvailable CAN fire if the query's table names
    // drift from the fixture's registered names — register_sensor(...).expect() only
    // proves registration succeeded, NOT that the query's table names resolve.
    // Symmetric runtime guard is required to catch that fixture/query drift.
    if let Err(PrismError::TableNotAvailable(ref details)) = result {
        panic!(
            "SAP-3 [H8] VACUOUS-PASS GUARD: TableNotAvailable (E-QUERY-037) fired before \
             column gate — column-availability gate was never reached, a suspension-arm \
             regression would be invisible. Details: {}. \
             If fixture table names drifted from the query table names, update one to match; \
             make_join_engine() must register both 'crowdstrike_alerts' and \
             'secondary_table', which must match the table names used in this test's query.",
            details
        );
    }

    // Pre-column-gate variants that are genuinely unreachable for this fixture/query:
    //
    // check_temporal_literals (E-QUERY-041 / E-QUERY-002): fires only when the query
    // contains a temporal literal (e.g., `timestamp > '2024-01-01T00:00:00Z'`).  Our
    // WHERE predicate is `totally_unknown_col = 'foo'` — a plain string comparison with
    // no temporal literal syntax.  check_temporal_literals returns Ok(()) immediately.
    //
    // AuditTableAccessDenied (E-QUERY-011): check_internal_table_capabilities fires
    // only for table names in INTERNAL_TABLE_DESCRIPTORS (e.g., `prism_audit`) that
    // require the AuditRead capability.  `crowdstrike_alerts` and `secondary_table`
    // are sensor tables not present in that descriptor set — this gate is structurally
    // unreachable from a query that names only sensor tables.
    //
    // resolve_clients / CapabilityDenied: resolve_clients executes AFTER the column
    // gate in execute_inner.  It cannot fire before the suspension arm is exercised.

    // ── SAP-3 gate assertion ─────────────────────────────────────────────────
    // Plan-time suspension MUST NOT surface ColumnNotFound for "totally_unknown_col".
    // The anti-vacuous-pass guards above ensure the planner was reached.
    // Expected outcome: Err(QueryExecutionFailed) — DataFusion schema error at
    // execution time because "totally_unknown_col" is absent from both table schemas.
    // BC-2.11.016 §HEAD-JOIN SUSPENSION RULE / §FP-001: fail-open means DataFusion
    // validates the WHERE column at execution time, not silent 0-row success.
    // Live [H8] parity: t13-preflight-audit.py [H8] check FAILs the `not ec and not rows`
    // branch (swallowed DataFusion schema error, FAIL-DEFECT). Ok(empty) here = T13 [H8] FAIL.

    // [H8] silent-swallow guard: Ok(_) with 0 rows means DataFusion skipped schema
    // validation — mirrors the live T13 [H8] FAIL condition (BC-2.11.016 §HEAD-JOIN
    // SUSPENSION RULE: fail-open must produce a DataFusion execution-time schema error,
    // NOT silent 0-row success).
    if result.is_ok() {
        panic!(
            "SAP-3 [H8] SILENT-SWALLOW: query returned Ok (empty or non-empty) for \
             WHERE totally_unknown_col = 'foo' in a JOIN query. \
             BC-2.11.016 §HEAD-JOIN SUSPENSION RULE (FP-001): fail-open must produce \
             a DataFusion execution-time schema error, NOT silent 0-row success. \
             Live T13 [H8] parity: t13-preflight-audit.py FAILs 0-rows-no-error as \
             swallowed DataFusion schema error (FAIL-DEFECT per BC-2.11.016 §HEAD-JOIN \
             SUSPENSION RULE). An Ok result here means the fixture has empty MemTables \
             (DataFusion skips schema validation on empty input) — verify that \
             make_join_engine() registers execution-capable adapters returning ≥1 row."
        );
    }

    if let Err(PrismError::ColumnNotFound(ref details)) = result {
        panic!(
            "SAP-3 [H8] PLANNER RED GATE: HEAD-JOIN suspension FAILED — E-QUERY-038 \
             fired for column '{}' in table '{}'. \
             BC-2.11.016 §HEAD-JOIN SUSPENSION RULE (FP-001): when `sql_query.joins` is \
             non-empty AND the reference is a bare unqualified ref absent from the FROM \
             schema, the E-QUERY-038 gate MUST NOT fire (fail-open). \
             Fix: verify `head_has_joins = !sql_query.joins.is_empty()` in \
             `check_query_column_availability` and that the suspension arm \
             `Err(PrismError::ColumnNotFound(_)) => {{}}` is reachable. \
             Available columns reported: {:?}",
            details.column, details.table, details.available_columns
        );
    }

    // Positive assertion: the execution path must terminate at QueryExecutionFailed
    // (DataFusion schema error for "totally_unknown_col" absent from both table schemas).
    assert!(
        matches!(result, Err(PrismError::QueryExecutionFailed { .. })),
        "SAP-3 [H8] EXECUTION OUTCOME: expected Err(QueryExecutionFailed) — DataFusion \
         schema error for 'totally_unknown_col' absent from both table schemas — \
         but got: {:?}. \
         BC-2.11.016 §HEAD-JOIN SUSPENSION RULE (FP-001): execution-time DataFusion schema \
         error is the spec-sanctioned outcome when the planner suspends E-QUERY-038 \
         (fail-open) and the column is genuinely absent from the execution schema.",
        result.as_ref().err()
    );

    // Detail-content assertion (OBS-1): "SQL planning error:" is a prism-owned constant
    // hardcoded at the sql.sql_planning_error construction site in materialization.rs
    // (`session_ctx.sql(&plan_pinned_sql).await.map_err(...)` — the DataFusion logical
    // planning step that validates WHERE column references against registered MemTable
    // schemas). Not DataFusion prose — immune to DataFusion version evolution.
    // `starts_with` (not `contains`) is required for correct site discrimination:
    // the sibling construction sites "filter SQL planning error:", "pipe SQL planning
    // error:", and "sqlpipe SQL planning error:" all CONTAIN the substring but do NOT
    // start with it (they carry a mode-prefix before "SQL planning error:").
    // starts_with excludes them. Full discrimination table:
    //   "virtual field injection failed:"  — fan-out path (different prefix)
    //   "<redacted; see server logs>"      — memory budget (map_datafusion_memory_error)
    //   "filter SQL planning error:"       — filter-mode (starts_with "filter …")
    //   "pipe SQL planning error:"         — pipe-mode (starts_with "pipe …")
    //   "sqlpipe SQL planning error:"      — sql-pipe-mode (starts_with "sqlpipe …")
    //   "SQL normalization failed:"        — normalize_for_datafusion None path
    // The column name "totally_unknown_col" is not present in the detail
    // (CWE-209 / BC-2.10.007 Rule-1 redaction); the site-specific prefix is the
    // stable behavioral anchor that uniquely identifies this execution site.
    if let Err(PrismError::QueryExecutionFailed { ref detail }) = result {
        assert!(
            detail.starts_with("SQL planning error:"),
            "SAP-3 [H8] DETAIL CONTENT: QueryExecutionFailed detail must start with \
             'SQL planning error:' — a prism-owned constant at the sql.sql_planning_error \
             site in materialization.rs `session_ctx.sql()`, not a virtual-field injection, \
             memory budget, filter/pipe/sqlpipe path (those carry a mode-prefix before the \
             phrase), or normalization failure. \
             Actual detail: {:?}",
            detail
        );
    }
}

// ── Test B: [H8] error-mapping defense-in-depth → E-QUERY-034, NOT E-QUERY-038 ─

/// SAP-3 error-mapping defense-in-depth: `QueryExecutionFailed` (the execution-time
/// variant that HEAD-JOIN fail-open produces at DataFusion execution) emits
/// `code == "E-QUERY-034"` in `structuredContent.error`, while `content[].text`
/// carries the redacted "Internal error" form with NO E-code (Rule-1 redaction,
/// BC-2.10.007).
///
/// **Scope**: this test covers the error-response constructor
/// (`prism_error_to_structured_call_result`) path only — it hand-builds a
/// `PrismError::QueryExecutionFailed` directly and does NOT drive through the
/// parser or planner.  A planner regression that changes the error variant fired
/// (e.g., E-QUERY-038 fires before reaching execution) would NOT be caught here;
/// that regression is caught by `test_sap3_head_join_bare_unknown_col_plan_suspension`
/// above (the primary SAP-3 gate for [H8]).
///
/// **Reachability rationale**: `QueryExecutionFailed` is produced by the DataFusion
/// execution path when a bare unqualified column is absent from ALL join sources at
/// run time (the column DID pass the plan-time suspension gate).  The T13 live-audit
/// [H8] check exercises this path against a live DTU endpoint; this in-process test
/// covers only the error-mapping half of the path without a live DTU.
///
/// Defect: T13 audit check [H8] was a false FAIL because `parse_envelope` read the
/// regex-scraped code from message text ("ERROR: [internal] - Internal error. ...")
/// which contains no E-code, yielding "UNKNOWN".  The canonical code lives in
/// `structuredContent.error.code == "E-QUERY-034"` (via `ec_code_override` in the
/// six-variant query-engine arm of `error_mapping.rs`).
///
/// SID-2: the full composed `content[].text` is asserted (not only the `code` field).
///
/// BC-2.11.016 §HEAD-JOIN SUSPENSION RULE / BC-2.10.007 §LOW-002.
#[test]
fn test_sap3_head_join_bare_unknown_col_wire_shape() {
    // ── Step 1: construct the production error that HEAD-JOIN fail-open yields ──
    // `QueryExecutionFailed` is the variant that `DataFusion` produces for schema
    // errors at execution time (e.g. unknown column in a cross-sensor JOIN).
    // error_mapping.rs pins it to `ec_code_override = Some("E-QUERY-034")`.
    let err = PrismError::QueryExecutionFailed {
        detail: "DataFusion plan execution: schema error: field 'totally_unknown_col' not found"
            .to_owned(),
    };
    let result = prism_error_to_structured_call_result(err);

    // ── Step 2: wire-level assertion on SERIALISED JSON ──────────────────────
    let sc = result
        .structured_content
        .as_ref()
        .expect("SAP-3 [H8]: structuredContent must be present (BC-2.10.007)");

    // Serialise to JSON — this is the exact envelope the LLM agent receives.
    let serialized =
        serde_json::to_string(sc).expect("SAP-3 [H8]: structured_content must serialise");

    // 2a. structuredContent.error.code MUST be "E-QUERY-034" (ec_code_override pin).
    //     NOT "E-QUERY-038" — E-QUERY-038 is for plan-time ColumnNotFound, not
    //     execution-time QueryExecutionFailed (BC-2.11.016 §HEAD-JOIN SUSPENSION RULE).
    let code = sc
        .get("error")
        .and_then(|e| e.get("code"))
        .and_then(|v| v.as_str())
        .expect("SAP-3 [H8]: structuredContent.error.code must be present");
    assert_eq!(
        code, "E-QUERY-034",
        "SAP-3 [H8]: structuredContent.error.code must be 'E-QUERY-034' for \
         QueryExecutionFailed (HEAD-JOIN fail-open; BC-2.11.016 §FP-001 / BC-2.10.007 §LOW-002); \
         got {code:?}. Serialised structuredContent: {serialized}"
    );

    // 2b. Must NOT be E-QUERY-038 — belt-and-suspenders: E-QUERY-038 is plan-time
    //     ColumnNotFound; HEAD-JOIN fail-open must not regress to it.
    assert_ne!(
        code, "E-QUERY-038",
        "SAP-3 [H8]: HEAD-JOIN fail-open must NOT produce E-QUERY-038 (plan-time \
         ColumnNotFound); that would violate BC-2.11.016 §HEAD-JOIN SUSPENSION RULE \
         which mandates fail-open to E-QUERY-034 or controlled rejection"
    );

    // 2c. E-QUERY-034 must appear in serialised structuredContent (code field).
    assert!(
        serialized.contains("E-QUERY-034"),
        "SAP-3 [H8]: 'E-QUERY-034' must appear in serialised structuredContent; \
         serialised: {serialized}"
    );

    // ── Step 3: content[].text wire assertions (SID-2 composed-output) ───────
    let text = content_text(&result);

    // 3a. content[].text MUST NOT contain "E-QUERY-034" — Rule-1 redaction (BC-2.10.007):
    //     the E-code belongs in structuredContent.error.code, not in the human-readable
    //     message text (prevents E-code leakage into LLM agent context).
    assert!(
        !text.contains("E-QUERY-034"),
        "SAP-3 [H8]: content[].text must NOT contain 'E-QUERY-034' (Rule-1 redaction, \
         BC-2.10.007 message/suggestion split); got text: {text:?}"
    );

    // 3b. content[].text MUST contain "Internal error" — the Rule-1 terse redaction form.
    assert!(
        text.contains("Internal error"),
        "SAP-3 [H8]: content[].text must contain 'Internal error' (Rule-1 redaction for \
         QueryExecutionFailed; BC-2.10.007 §LOW-002); got: {text:?}"
    );

    // 3c. SID-2: "audit log" must appear exactly ONCE in the full composed content[].text
    //     (no duplication between message and suggestion — BC-2.10.007 [H8b] invariant).
    let audit_log_count = text.to_lowercase().matches("audit log").count();
    assert_eq!(
        audit_log_count, 1,
        "SAP-3 [H8]: 'audit log' must appear exactly once in content[].text \
         (BC-2.10.007 [H8b] no-duplication invariant); found {audit_log_count} times. \
         Got text: {text:?}"
    );

    // 3d. SID-2: the full content[].text must start with "ERROR: [internal]" (BC-2.10.007).
    assert!(
        text.starts_with("ERROR: [internal]"),
        "SAP-3 [H8]: content[].text must start with 'ERROR: [internal]' for \
         QueryExecutionFailed (category='internal', BC-2.10.007 §LOW-002 / BC-2.10.007 \
         content_text format). Got: {:?}",
        text.chars().take(60).collect::<String>()
    );
}
