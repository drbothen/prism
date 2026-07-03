//! DataFusion async UDF registration — wires plugin descriptors into SessionContext.
//!
//! Traces to: BC-2.19.001 postcondition — each field registers as a DataFusion scalar UDF.
//! AC-003 / S-DEMO-ENRICHMENT-PIVOT-001.
//!
//! # Tests in this file
//!
//! - `test_BC_2_19_001_plugin_udfs_registered_in_session_context`: Mock UDF sentinel + counter
//!   test verifying DataFusion async UDF machinery (uses `MockInfusionAsyncUdf`, not production).
//!
//! - `test_BC_2_19_001_real_infusion_async_udf_delegates_to_stub_source` (CRIT-2): REAL
//!   `InfusionAsyncUdf` backed by a stub `InfusionSource` that returns a sentinel. Proves the
//!   production `invoke_async_with_args` body runs and delegates to `descriptor.source.enrich_single`.
//!
//! - `test_BC_2_19_001_register_infusion_udfs_helper_compiles_and_registers`: Structural test
//!   verifying `register_infusion_udfs` produces a discoverable UDF in the SessionContext.
//!
//! # Production UDF implementation
//!
//! `InfusionAsyncUdf::invoke_async_with_args` is fully implemented — it reads the input
//! `ColumnarValue`, calls `descriptor.source.enrich_single` per row, and builds a `StringArray`
//! output. The REAL UDF test (`test_BC_2_19_001_real_infusion_async_udf_delegates_to_stub_source`)
//! exercises this body end-to-end.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    non_snake_case,
    dead_code,
    unused_imports
)]

use std::any::Any;
use std::hash::{Hash, Hasher};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use arrow::array::{Array, StringArray};
use arrow::datatypes::DataType;
use async_trait::async_trait;
use datafusion::common::not_impl_err;
use datafusion::error::Result as DataFusionResult;
use datafusion::execution::context::SessionContext;
use datafusion::logical_expr::async_udf::{AsyncScalarUDF, AsyncScalarUDFImpl};
use datafusion::logical_expr::{
    ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl, Signature, TypeSignature, Volatility,
};

// ---------------------------------------------------------------------------
// Mock async UDF with sentinel value + call counter (AC-003 anti-false-green)
// ---------------------------------------------------------------------------

const SENTINEL: &str = "CVE-PIVOT-TEST-SENTINEL";

/// Mock async UDF implementation for Red Gate anti-false-green test.
///
/// Returns the SENTINEL string `"CVE-PIVOT-TEST-SENTINEL"` from `invoke_async_with_args`.
/// Increments `call_count` on every async invocation — proves the async path was taken.
/// `invoke_with_args` returns `not_impl_err!` — the sync path cannot produce the sentinel.
struct MockInfusionAsyncUdf {
    call_count: Arc<AtomicUsize>,
}

impl std::fmt::Debug for MockInfusionAsyncUdf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockInfusionAsyncUdf").finish()
    }
}

// `ScalarUDFImpl` requires `DynEq + DynHash` (auto-impl for `Eq + Hash + Any`).
// `Arc<AtomicUsize>` doesn't implement `Hash`, so we key on the fixed name string.
impl PartialEq for MockInfusionAsyncUdf {
    fn eq(&self, _other: &Self) -> bool {
        // Only one instance of this mock per test; name is the identity key.
        true
    }
}
impl Eq for MockInfusionAsyncUdf {}
impl Hash for MockInfusionAsyncUdf {
    fn hash<H: Hasher>(&self, state: &mut H) {
        "mock_enrichment_udf".hash(state);
    }
}

// `AsyncScalarUDFImpl` is a supertrait of `ScalarUDFImpl` — both impl blocks are required.
// The `ScalarUDFImpl` methods (`as_any`, `name`, `signature`, `return_type`, `invoke_with_args`)
// live in the separate `ScalarUDFImpl` impl block below. The `AsyncScalarUDFImpl` impl block
// ONLY contains `invoke_async_with_args` (and optionally `ideal_batch_size`).
impl ScalarUDFImpl for MockInfusionAsyncUdf {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "mock_enrichment_udf"
    }

    fn signature(&self) -> &Signature {
        // SAFETY: Signature is used only for type-checking; this mock uses volatile for simplicity.
        // Leak to produce &'static; acceptable for test-only code.
        Box::leak(Box::new(Signature::new(
            TypeSignature::Exact(vec![DataType::Utf8]),
            Volatility::Volatile,
        )))
    }

    fn return_type(&self, _arg_types: &[DataType]) -> DataFusionResult<DataType> {
        Ok(DataType::Utf8)
    }

    /// Sync fallback — must NOT return the sentinel; must force async execution.
    fn invoke_with_args(&self, _args: ScalarFunctionArgs) -> DataFusionResult<ColumnarValue> {
        not_impl_err!(
            "MockInfusionAsyncUdf: use invoke_async_with_args (async context only). \
             This error appearing in the Red Gate test means DataFusion is not using the async path."
        )
    }
}

// `AsyncScalarUDFImpl` is declared with `#[async_trait]` in DataFusion 53.1.
// This impl block MUST be annotated with `#[async_trait]` to match the lifetime
// desugaring on the trait declaration.
#[async_trait]
impl AsyncScalarUDFImpl for MockInfusionAsyncUdf {
    /// Async enrichment call — returns the SENTINEL value for every row.
    ///
    /// Increments `call_count` to prove the async round-trip was taken.
    async fn invoke_async_with_args(
        &self,
        args: ScalarFunctionArgs,
    ) -> DataFusionResult<ColumnarValue> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        // Return the sentinel for each row in the input batch.
        let num_rows = args
            .args
            .first()
            .map(|col| match col {
                ColumnarValue::Array(arr) => arr.len(),
                ColumnarValue::Scalar(_) => 1,
            })
            .unwrap_or(1);

        let sentinel_array = Arc::new(StringArray::from(vec![SENTINEL; num_rows]));
        Ok(ColumnarValue::Array(sentinel_array))
    }
}

// ---------------------------------------------------------------------------
// Test 3 (AC-003): DataFusion async UDF registered in SessionContext and callable
// ---------------------------------------------------------------------------

/// Test BC-2.19.001: plugin-type UDF registered in DataFusion SessionContext + async execution.
///
/// Traces to: BC-2.19.001 postcondition — each field registers as a DataFusion scalar UDF.
/// AC-003 / S-DEMO-ENRICHMENT-PIVOT-001.
///
/// Red Gate failure: `InfusionAsyncUdf::invoke_async_with_args` panics with `todo!()`.
///
/// Anti-false-green: two-assertion guard — sentinel value + call counter > 0.
/// A no-op sync stub cannot satisfy both assertions simultaneously.
#[tokio::test]
async fn test_BC_2_19_001_plugin_udfs_registered_in_session_context() {
    use prism_query::memory::build_session_context;

    // Build a DataFusion SessionContext (same helper used by execute_inner).
    let ctx = build_session_context(prism_query::memory::QUERY_MEMORY_POOL_BYTES)
        .expect("BC-2.19.001: build_session_context must succeed for test setup");

    // Wire: register the mock async UDF (with sentinel + call counter).
    let call_count = Arc::new(AtomicUsize::new(0));
    let mock_udf = MockInfusionAsyncUdf {
        call_count: Arc::clone(&call_count),
    };

    let async_udf = AsyncScalarUDF::new(Arc::new(mock_udf));
    ctx.register_udf(async_udf.into_scalar_udf());

    // Register a MemTable with one row to drive the UDF.
    use arrow::array::StringArray;
    use arrow::datatypes::{Field, Schema};
    use arrow::record_batch::RecordBatch;
    use datafusion::datasource::MemTable;

    let schema = Arc::new(Schema::new(vec![Field::new(
        "ioc_value",
        DataType::Utf8,
        false,
    )]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec!["192.168.1.1"]))],
    )
    .expect("BC-2.19.001: test RecordBatch construction must succeed");

    let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
        .expect("BC-2.19.001: MemTable construction must succeed");

    ctx.register_table("test_events", Arc::new(table))
        .expect("BC-2.19.001: register_table must succeed");

    // Execute a DataFusion query that invokes the UDF (the async path).
    // FAILS RED: `InfusionAsyncUdf::invoke_async_with_args` panics with `todo!()`.
    // Once the real implementation is wired in the TDD green phase, this passes.
    let df = ctx
        .sql("SELECT mock_enrichment_udf(ioc_value) AS enriched_ioc FROM test_events")
        .await
        .expect("BC-2.19.001: UDF must be registered and SQL must parse");

    let batches = df
        .collect()
        .await
        .expect("BC-2.19.001: UDF async execution must succeed (FAILS RED: todo!() in stub)");

    // Assertion 1: the async call counter must be > 0.
    let count = call_count.load(Ordering::SeqCst);
    assert!(
        count > 0,
        "BC-2.19.001 AC-003 anti-false-green: call_count must be > 0 (async path must be taken). \
         Got count = {}. A stub returning a constant in invoke_with_args cannot reach this assertion.",
        count
    );

    // Assertion 2: the result must contain the sentinel value.
    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_rows, 1,
        "BC-2.19.001: query must return 1 row (1 input row -> 1 enriched output row)"
    );

    let enriched_col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("BC-2.19.001: enriched column must be a StringArray");

    let enriched_value = enriched_col.value(0);
    assert_eq!(
        enriched_value,
        SENTINEL,
        "BC-2.19.001 AC-003 anti-false-green: enriched value must be the sentinel '{}'. \
         Got '{}'. A no-op stub cannot produce this sentinel — only the async enrichment round-trip can.",
        SENTINEL,
        enriched_value
    );
}

// ---------------------------------------------------------------------------
// Test 3b (CRIT-2): Real InfusionAsyncUdf backed by stub InfusionSource — production UDF path
// ---------------------------------------------------------------------------

/// Test BC-2.19.001: REAL `InfusionAsyncUdf` backed by stub `InfusionSource` proves production body runs.
///
/// Traces to: BC-2.19.001 postcondition — each field registers as a DataFusion scalar UDF.
/// AC-003 / CRIT-2 / S-DEMO-ENRICHMENT-PIVOT-001.
///
/// # Anti-false-green (CRIT-2 fix)
///
/// Unlike `test_BC_2_19_001_plugin_udfs_registered_in_session_context` which uses a `MockInfusionAsyncUdf`
/// that bypasses production code, this test wires the REAL `InfusionAsyncUdf` backed by a STUB
/// `InfusionSource` that:
/// 1. Returns a known sentinel `"REAL-UDF-PIVOT-SENTINEL"` from `enrich_single`.
/// 2. Increments a call counter on every invocation.
///
/// The test then executes a DataFusion query through the REAL `invoke_async_with_args` body.
/// A sentinel that can ONLY originate from the production `InfusionAsyncUdf::invoke_async_with_args`
/// path proves:
/// - The real `invoke_async_with_args` body runs (not a mock bypass).
/// - The `descriptor.source.enrich_single` delegation actually fires.
/// - The counter > 0 proves the stub source was actually called.
///
/// This test would FAIL if `InfusionAsyncUdf::invoke_async_with_args` were still `todo!()`.
/// It would also FAIL if the UDF returns None (NullSource) instead of the sentinel.
const REAL_UDF_SENTINEL: &str = "REAL-UDF-PIVOT-SENTINEL";

/// Stub `InfusionSource` that returns a sentinel value and increments a counter.
///
/// Used in CRIT-2 test to verify the REAL production UDF path calls `enrich_single`.
struct SentinelInfusionSource {
    call_count: Arc<AtomicUsize>,
}

impl std::fmt::Debug for SentinelInfusionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SentinelInfusionSource").finish()
    }
}

impl prism_spec_engine::InfusionSource for SentinelInfusionSource {
    fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Some(serde_json::Value::String(REAL_UDF_SENTINEL.to_string()))
    }

    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
        inputs
            .iter()
            .map(|i| self.enrich_single(i, input_type))
            .collect()
    }
}

#[tokio::test]
async fn test_BC_2_19_001_real_infusion_async_udf_delegates_to_stub_source() {
    use arrow::array::StringArray;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use datafusion::datasource::MemTable;
    use datafusion::execution::FunctionRegistry;

    use prism_query::infusion_udf::{register_infusion_udfs, InfusionAsyncUdf};
    use prism_query::memory::build_session_context;
    use prism_spec_engine::{InfusionSource, InfusionUdfDescriptor};

    // Build the REAL InfusionAsyncUdf backed by the stub source.
    let call_count = Arc::new(AtomicUsize::new(0));
    let stub_source: Arc<dyn InfusionSource> = Arc::new(SentinelInfusionSource {
        call_count: Arc::clone(&call_count),
    });

    // Construct the descriptor directly — no registry needed for this wiring path.
    let descriptor = InfusionUdfDescriptor::new(
        "pivot_enrich",
        "ip",
        "string",
        "pivot_test",
        stub_source,
        None,
        3600,
        "",
    );

    // Build a SessionContext.
    let ctx = build_session_context(prism_query::memory::QUERY_MEMORY_POOL_BYTES)
        .expect("BC-2.19.001 CRIT-2: build_session_context must succeed");

    // Register using the REAL register_infusion_udfs → REAL InfusionAsyncUdf.
    register_infusion_udfs(&ctx, vec![descriptor])
        .expect("BC-2.19.001 CRIT-2: register_infusion_udfs must succeed");

    // Verify registration.
    let udf_names = ctx.udfs();
    assert!(
        udf_names.contains("pivot_enrich"),
        "BC-2.19.001 CRIT-2: 'pivot_enrich' UDF must be registered. Got: {:?}",
        udf_names
    );

    // Register a MemTable with one row to drive the UDF.
    let schema = Arc::new(Schema::new(vec![Field::new(
        "ioc_value",
        DataType::Utf8,
        false,
    )]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec!["10.0.0.1"]))],
    )
    .expect("BC-2.19.001 CRIT-2: RecordBatch construction must succeed");
    let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
        .expect("BC-2.19.001 CRIT-2: MemTable construction must succeed");
    ctx.register_table("pivot_events", Arc::new(table))
        .expect("BC-2.19.001 CRIT-2: register_table must succeed");

    // Execute a query that invokes the REAL InfusionAsyncUdf::invoke_async_with_args.
    let df = ctx
        .sql("SELECT pivot_enrich(ioc_value) AS enriched FROM pivot_events")
        .await
        .expect("BC-2.19.001 CRIT-2: SQL with real InfusionAsyncUdf must parse");

    let batches = df.collect().await.expect(
        "BC-2.19.001 CRIT-2: real InfusionAsyncUdf::invoke_async_with_args must succeed. \
             A todo!() in the UDF body would panic here.",
    );

    // Assertion 1: call counter > 0 — proves enrich_single was actually called.
    let count = call_count.load(Ordering::SeqCst);
    assert!(
        count > 0,
        "BC-2.19.001 CRIT-2: stub source call_count must be > 0 after query execution. \
         Got count = {}. \
         The production invoke_async_with_args body must call descriptor.source.enrich_single().",
        count
    );

    // Assertion 2: sentinel in output — proves the production path returns the enriched value.
    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_rows, 1,
        "BC-2.19.001 CRIT-2: query must return 1 row (1 input row → 1 enriched output row)"
    );

    let enriched_col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("BC-2.19.001 CRIT-2: enriched column must be a StringArray");

    let enriched_value = enriched_col.value(0);
    assert_eq!(
        enriched_value, REAL_UDF_SENTINEL,
        "BC-2.19.001 CRIT-2: enriched value must be the sentinel '{}'. \
         Got '{}'. \
         The real InfusionAsyncUdf delegates to enrich_single which returns the sentinel. \
         A NullSource wiring defect or mock bypass would not produce this sentinel.",
        REAL_UDF_SENTINEL, enriched_value
    );
}

// ---------------------------------------------------------------------------
// Companion test: register_infusion_udfs helper compiles and is callable
// ---------------------------------------------------------------------------

/// Verify `register_infusion_udfs` is callable with a valid descriptor list.
///
/// This test exercises the helper's signature and registration path.
#[test]
fn test_BC_2_19_001_register_infusion_udfs_helper_compiles_and_registers() {
    use prism_query::infusion_udf::register_infusion_udfs;
    use prism_query::memory::build_session_context;
    use prism_spec_engine::{
        InfusionField, InfusionRegistry, InfusionSpec, InfusionType, PluginConfig,
    };

    // Build a plugin-type InfusionSpec in-memory (same pattern as infusion_tests.rs).
    let fields = vec![InfusionField::new(
        "threat_score",
        "device_ip",
        "ip",
        "float",
    )];
    let mut spec = InfusionSpec::new(
        "threat_intel",
        "Threat Intel Plugin",
        InfusionType::Plugin,
        fields,
        "threat_intel.infusion.toml",
    );
    spec.plugin_config = Some(PluginConfig::new("plugins/threat_intel.prx"));

    // Load it into an InfusionRegistry to get InfusionUdfDescriptors.
    let registry = InfusionRegistry::new();
    let descriptors = registry
        .load_spec(spec)
        .expect("BC-2.19.001: plugin spec must load into registry");

    // Build a SessionContext (same as engine.rs execute path).
    let ctx = build_session_context(prism_query::memory::QUERY_MEMORY_POOL_BYTES)
        .expect("BC-2.19.001: build_session_context must succeed");

    // Call register_infusion_udfs — must succeed (registration only).
    // The UDF is registered with a NullSource (load_spec without runtime) — invoking it
    // in a query would return null for every row, but registration itself must succeed.
    register_infusion_udfs(&ctx, descriptors)
        .expect("BC-2.19.001: register_infusion_udfs must succeed for plugin-type descriptors");

    // Verify the UDF is discoverable in the SessionContext.
    // DataFusion 53.1: ctx.udfs() (via FunctionRegistry trait) returns HashSet<String> of names.
    use datafusion::execution::FunctionRegistry;
    let udf_names = ctx.udfs();
    assert!(
        udf_names.contains("threat_score"),
        "BC-2.19.001: 'threat_score' UDF must be registered in SessionContext after register_infusion_udfs. \
         Got registered UDF names: {:?}",
        udf_names
    );
}
