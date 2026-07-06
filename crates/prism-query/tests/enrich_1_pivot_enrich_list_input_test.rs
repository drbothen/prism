//! ENRICH-1 Red Gate — `pivot_enrich` UDF handles JSON-list string input.
//!
//! The design document (Decision 2 §pivot_enrich UDF input contract) requires:
//! - Scalar string input: `"10.0.0.1"` → enrich directly (existing behavior).
//! - JSON-list string input: `'["hash1","hash2"]'` → parse as JSON array,
//!   enrich each element, return a JSON-list of enriched results.
//!
//! These tests drive the ENRICH-1 change to `InfusionAsyncUdf::invoke_async_with_args`
//! in prism-query/src/infusion_udf.rs.
//!
//! All tests in this file MUST FAIL before implementation (Red Gate).
//! Tests MUST PASS after implementation with zero regressions.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use datafusion::arrow::array::StringArray;
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;

use prism_query::infusion_udf::register_infusion_udfs;
use prism_query::memory::build_session_context;
use prism_spec_engine::{InfusionSource, InfusionUdfDescriptor};

// ---------------------------------------------------------------------------
// Sentinel stub source
// ---------------------------------------------------------------------------

/// Stub InfusionSource that returns `"enriched:<input>"` so we can verify
/// each element of a list was enriched independently.
struct PrefixInfusionSource {
    call_count: Arc<AtomicUsize>,
}

impl std::fmt::Debug for PrefixInfusionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrefixInfusionSource").finish()
    }
}

impl InfusionSource for PrefixInfusionSource {
    fn enrich_single(&self, input_value: &str, _input_type: &str) -> Option<serde_json::Value> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Some(serde_json::Value::String(format!("enriched:{input_value}")))
    }

    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
        inputs
            .iter()
            .map(|s| self.enrich_single(s, input_type))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Test 1: Scalar input still works (backward compat must not regress).
// This test should ALREADY pass before ENRICH-1, and must continue to pass after.
// ---------------------------------------------------------------------------

/// ENRICH-1 backward-compat: scalar `"10.0.0.1"` input still works.
#[tokio::test]
async fn test_ENRICH_1_pivot_enrich_scalar_input_backward_compat() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let stub_source: Arc<dyn InfusionSource> = Arc::new(PrefixInfusionSource {
        call_count: Arc::clone(&call_count),
    });

    let descriptor = InfusionUdfDescriptor::new(
        "pivot_enrich",
        "ip",
        "string",
        "test_enrich",
        stub_source,
        None,
        3600,
        "",
    );

    let ctx = build_session_context(prism_query::memory::QUERY_MEMORY_POOL_BYTES)
        .expect("build_session_context must succeed");

    register_infusion_udfs(&ctx, vec![descriptor]).expect("register_infusion_udfs must succeed");

    let schema = Arc::new(Schema::new(vec![Field::new(
        "ip_col",
        DataType::Utf8,
        false,
    )]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec!["10.0.0.1"]))],
    )
    .expect("RecordBatch must succeed");
    let table =
        MemTable::try_new(Arc::clone(&schema), vec![vec![batch]]).expect("MemTable must succeed");
    ctx.register_table("ip_events", Arc::new(table))
        .expect("register_table must succeed");

    let df = ctx
        .sql("SELECT pivot_enrich(ip_col) AS enriched FROM ip_events")
        .await
        .expect("SQL must parse");

    let batches = df.collect().await.expect("query must succeed");

    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(total_rows, 1, "scalar input must produce 1 output row");

    let enriched_col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("enriched must be StringArray");

    let val = enriched_col.value(0);
    assert_eq!(
        val, "enriched:10.0.0.1",
        "ENRICH-1 backward-compat: scalar enrichment must produce 'enriched:10.0.0.1'; got '{val}'"
    );
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        1,
        "ENRICH-1 backward-compat: scalar input must call enrich_single exactly once"
    );
}

// ---------------------------------------------------------------------------
// Test 2: JSON-list string input `'["hash1","hash2"]'` → enriches each element,
//          returns a JSON-list of enriched results.
// FAILS RED: UDF currently treats the whole string as a scalar ("enriched:[\"hash1\",\"hash2\"]").
// After ENRICH-1: returns `'["enriched:hash1","enriched:hash2"]'`.
// ---------------------------------------------------------------------------

/// ENRICH-1: JSON-list string input enriches each element and returns a JSON-list.
///
/// ADR-051 D4 (S-DEMO-ENRICHMENT-TYPED-OUTPUT-001): ENRICH-1 list-dispatch is gated
/// to `output_type = "json"` only. Updated from `"string"` to `"json"` to align with
/// the Scalar-Input rule (integer/boolean/etc. UDFs receive raw scalar, not list-dispatch).
#[tokio::test]
async fn test_ENRICH_1_pivot_enrich_json_list_input_enriches_each_element() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let stub_source: Arc<dyn InfusionSource> = Arc::new(PrefixInfusionSource {
        call_count: Arc::clone(&call_count),
    });

    let descriptor = InfusionUdfDescriptor::new(
        "pivot_enrich",
        "hash",
        "json", // ADR-051 D4: ENRICH-1 list-dispatch only for json output_type
        "test_enrich",
        stub_source,
        None,
        3600,
        "",
    );

    let ctx = build_session_context(prism_query::memory::QUERY_MEMORY_POOL_BYTES)
        .expect("build_session_context must succeed");

    register_infusion_udfs(&ctx, vec![descriptor]).expect("register_infusion_udfs must succeed");

    // The column value is a JSON-list string as produced by ColumnMapper for wildcard paths.
    let schema = Arc::new(Schema::new(vec![Field::new(
        "ioc_col",
        DataType::Utf8,
        false,
    )]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec![r#"["hash1","hash2"]"#]))],
    )
    .expect("RecordBatch must succeed");
    let table =
        MemTable::try_new(Arc::clone(&schema), vec![vec![batch]]).expect("MemTable must succeed");
    ctx.register_table("ioc_events", Arc::new(table))
        .expect("register_table must succeed");

    let df = ctx
        .sql("SELECT pivot_enrich(ioc_col) AS enriched FROM ioc_events")
        .await
        .expect("SQL must parse");

    let batches = df
        .collect()
        .await
        .expect("ENRICH-1: UDF with JSON-list input must succeed");

    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_rows, 1,
        "ENRICH-1: JSON-list input must produce 1 output row (no explosion)"
    );

    let enriched_col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("enriched must be StringArray");

    let val = enriched_col.value(0);

    // Must be a JSON-list string with each element enriched.
    let parsed: serde_json::Value = serde_json::from_str(val).unwrap_or_else(|_| {
        panic!("ENRICH-1: UDF output for JSON-list input must be a valid JSON string; got '{val}'")
    });

    let arr = parsed
        .as_array()
        .unwrap_or_else(|| panic!("ENRICH-1: UDF output must be a JSON array; got '{val}'"));

    assert_eq!(
        arr.len(),
        2,
        "ENRICH-1: enriched list must have 2 elements; got {:?}",
        arr
    );
    assert_eq!(
        arr[0].as_str().unwrap_or(""),
        "enriched:hash1",
        "ENRICH-1: first enriched element must be 'enriched:hash1'; got {:?}",
        arr[0]
    );
    assert_eq!(
        arr[1].as_str().unwrap_or(""),
        "enriched:hash2",
        "ENRICH-1: second enriched element must be 'enriched:hash2'; got {:?}",
        arr[1]
    );

    // Must have called enrich_single twice (once per element).
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        2,
        "ENRICH-1: JSON-list input must call enrich_single once per element (2 calls); got {}",
        call_count.load(Ordering::SeqCst)
    );
}

// ---------------------------------------------------------------------------
// Test 3: JSON-list with one element → returns a JSON-list with one enriched element.
// FAILS RED: UDF currently treats input as scalar.
// ---------------------------------------------------------------------------

/// ENRICH-1: single-element JSON-list `'["hash1"]'` → `'["enriched:hash1"]'`.
///
/// ADR-051 D4 (S-DEMO-ENRICHMENT-TYPED-OUTPUT-001): ENRICH-1 list-dispatch is gated
/// to `output_type = "json"` only. Updated from `"string"` to `"json"`.
#[tokio::test]
async fn test_ENRICH_1_pivot_enrich_single_element_json_list() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let stub_source: Arc<dyn InfusionSource> = Arc::new(PrefixInfusionSource {
        call_count: Arc::clone(&call_count),
    });

    let descriptor = InfusionUdfDescriptor::new(
        "pivot_enrich",
        "hash",
        "json", // ADR-051 D4: ENRICH-1 list-dispatch only for json output_type
        "test_enrich",
        stub_source,
        None,
        3600,
        "",
    );

    let ctx = build_session_context(prism_query::memory::QUERY_MEMORY_POOL_BYTES)
        .expect("build_session_context must succeed");

    register_infusion_udfs(&ctx, vec![descriptor]).expect("register_infusion_udfs must succeed");

    let schema = Arc::new(Schema::new(vec![Field::new(
        "ioc_col",
        DataType::Utf8,
        false,
    )]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec![r#"["hash1"]"#]))],
    )
    .expect("RecordBatch must succeed");
    let table =
        MemTable::try_new(Arc::clone(&schema), vec![vec![batch]]).expect("MemTable must succeed");
    ctx.register_table("ioc_events2", Arc::new(table))
        .expect("register_table must succeed");

    let df = ctx
        .sql("SELECT pivot_enrich(ioc_col) AS enriched FROM ioc_events2")
        .await
        .expect("SQL must parse");

    let batches = df.collect().await.expect("UDF must succeed");
    let enriched_col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("enriched must be StringArray");

    let val = enriched_col.value(0);
    let parsed: serde_json::Value = serde_json::from_str(val)
        .unwrap_or_else(|_| panic!("ENRICH-1: single-element output must be JSON; got '{val}'"));
    let arr = parsed.as_array().unwrap_or_else(|| {
        panic!("ENRICH-1: single-element output must be JSON array; got '{val}'")
    });
    assert_eq!(
        arr.len(),
        1,
        "ENRICH-1: single-element output must have 1 element"
    );
    assert_eq!(arr[0].as_str().unwrap_or(""), "enriched:hash1");
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        1,
        "must call enrich_single exactly once"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Malformed JSON starting with `[` but not valid JSON → scalar path fallback.
// The design says "On failure (malformed JSON), fall through to scalar path."
// FAILS RED: UDF doesn't attempt JSON parse before ENRICH-1.
// ---------------------------------------------------------------------------

/// ENRICH-1: malformed JSON like `"[not_valid_json"` falls through to scalar path.
#[tokio::test]
async fn test_ENRICH_1_pivot_enrich_malformed_json_falls_through_to_scalar() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let stub_source: Arc<dyn InfusionSource> = Arc::new(PrefixInfusionSource {
        call_count: Arc::clone(&call_count),
    });

    let descriptor = InfusionUdfDescriptor::new(
        "pivot_enrich",
        "string",
        "string",
        "test_enrich",
        stub_source,
        None,
        3600,
        "",
    );

    let ctx = build_session_context(prism_query::memory::QUERY_MEMORY_POOL_BYTES)
        .expect("build_session_context must succeed");

    register_infusion_udfs(&ctx, vec![descriptor]).expect("register_infusion_udfs must succeed");

    let schema = Arc::new(Schema::new(vec![Field::new(
        "ioc_col",
        DataType::Utf8,
        false,
    )]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec!["[not_valid_json"]))],
    )
    .expect("RecordBatch must succeed");
    let table =
        MemTable::try_new(Arc::clone(&schema), vec![vec![batch]]).expect("MemTable must succeed");
    ctx.register_table("ioc_events3", Arc::new(table))
        .expect("register_table must succeed");

    let df = ctx
        .sql("SELECT pivot_enrich(ioc_col) AS enriched FROM ioc_events3")
        .await
        .expect("SQL must parse");

    let batches = df
        .collect()
        .await
        .expect("UDF must not panic on malformed JSON");

    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(total_rows, 1, "ENRICH-1: malformed JSON must produce 1 row");

    let enriched_col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("enriched must be StringArray");

    let val = enriched_col.value(0);
    assert_eq!(
        val, "enriched:[not_valid_json",
        "ENRICH-1: malformed JSON must fall through to scalar path; got '{val}'"
    );
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        1,
        "ENRICH-1: scalar fallback must call enrich_single once"
    );
}
