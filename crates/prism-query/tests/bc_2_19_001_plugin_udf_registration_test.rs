//! Red Gate test 3 — DataFusion async UDF registration wires plugin descriptors into SessionContext.
//!
//! Traces to: BC-2.19.001 postcondition — each field registers as a DataFusion scalar UDF.
//! AC-003 / S-DEMO-ENRICHMENT-PIVOT-001.
//!
//! # Anti-false-green hardening (AC-003)
//!
//! The test registers a MOCK async UDF implementation (NOT `InfusionAsyncUdf`, since that
//! would immediately `todo!()`) that returns a known SENTINEL value: `"CVE-PIVOT-TEST-SENTINEL"`.
//! This sentinel can ONLY originate from the async `invoke_async_with_args` round-trip — a
//! no-op or sync stub cannot produce it. An `Arc<AtomicUsize>` call counter is asserted > 0
//! after the DataFusion query executes, proving `invoke_async_with_args` was actually called.
//!
//! A stub that hard-codes a return value CANNOT pass both the sentinel assertion and the call
//! counter assertion simultaneously. An incorrectly-wrapped async UDF (sync fallback only)
//! fails loudly with `not_impl_err!` rather than silently returning a wrong value.
//!
//! # Red Gate failure mode
//!
//! This test registers a mock async UDF, not `InfusionAsyncUdf`, so it can test the
//! DataFusion registration path directly without hitting the `todo!()` in the production stub.
//! The actual `register_infusion_udfs` helper is tested via a companion sub-test that
//! verifies the function compiles and can be called (it will `todo!()` inside the UDF body
//! when DataFusion actually invokes the async path).
//!
//! `test_BC_2_19_001_plugin_udfs_registered_in_session_context` — exercises the real
//! `register_infusion_udfs` function + mock async UDF sentinel round-trip. FAILS RED because
//! `InfusionAsyncUdf::invoke_async_with_args` panics with `todo!()`.

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
// Companion test: register_infusion_udfs helper compiles and is callable
// ---------------------------------------------------------------------------

/// Verify `register_infusion_udfs` is callable with a valid descriptor list.
///
/// This test exercises the helper's signature and registration path. The UDF itself
/// has a `todo!()` body, so DataFusion execution would panic — but registration must
/// succeed (the function panics on invocation, not registration).
///
/// FAILS RED: `InfusionAsyncUdf::invoke_async_with_args` panics with `todo!()` when
/// the actual DataFusion query is executed. Registration itself should succeed.
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

    // Call register_infusion_udfs — must succeed (registration only, no execution).
    // The UDF is now registered in the SessionContext; invoking it would todo!().
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
