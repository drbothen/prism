//! Infusion enrichment UDF registration for DataFusion `SessionContext`.
//!
//! Consumes `InfusionUdfDescriptor` values exported by `prism-spec-engine`
//! (BC-2.19.001) and registers each as a DataFusion `AsyncScalarUDF` so that
//! analyst queries using `| enrich infusion(field)` resolve against plugin-backed
//! DTU HTTP services via the WASM runtime.
//!
//! # DataFusion 53.1.0 async UDF registration (RESEARCH CONFIRMED)
//! `ctx.register_udf(AsyncScalarUDF::new(Arc::new(impl)).into_scalar_udf())` ALONE
//! is sufficient — `AsyncFuncExec` is built into `DefaultPhysicalPlanner`.
//! No analyzer/optimizer/physical-optimizer rule required.
//!
//! HALLUCINATED SYMBOLS (DO NOT USE — do not exist in DF 53.1):
//! - `AsyncFunctionRule` — does not exist
//! - `enable_async_udf` config flag — does not exist
//! - `concurrent_async_udf_tasks` option — does not exist
//! - `GLOBAL_ASYNC_UDF_SEMAPHORE` — does not exist
//!
//! # Architecture Compliance
//! - `prism-spec-engine` MUST NOT depend on `prism-query` — dependency is one-way:
//!   `prism-query` imports `InfusionUdfDescriptor` from `prism-spec-engine`.
//! - New `event_type` tracing emissions require a BC-2.16.002 catalog row (SAP-1).
//! - `invoke_with_args` MUST return `not_impl_err!(...)` to force the async path.
//!
//! # Implementation status (S-DEMO-ENRICHMENT-PIVOT-001 — GREEN)
//! `InfusionAsyncUdf::invoke_async_with_args` is fully implemented: reads input `ColumnarValue`,
//! calls `descriptor.source.enrich_single` per row, and returns a `StringArray` output.
//! `register_infusion_udfs` registers `InfusionAsyncUdf` instances per descriptor.
//!
//! # async_trait requirement
//! `AsyncScalarUDFImpl` is declared with `#[async_trait]` in DataFusion 53.1.
//! Implementors MUST annotate their impl block with `#[async_trait]` to match
//! the lifetime desugaring produced by the macro on the trait declaration.
//!
//! Story: S-DEMO-ENRICHMENT-PIVOT-001

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_trait::async_trait;
use datafusion::arrow::datatypes::DataType;
use datafusion::error::Result as DataFusionResult;
use datafusion::execution::context::SessionContext;
use datafusion::logical_expr::async_udf::{AsyncScalarUDF, AsyncScalarUDFImpl};
use datafusion::logical_expr::{
    ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl, Signature, TypeSignature, Volatility,
};
use prism_spec_engine::InfusionUdfDescriptor;

// ---------------------------------------------------------------------------
// InfusionAsyncUdf — AsyncScalarUDFImpl wrapper for an infusion descriptor
// ---------------------------------------------------------------------------

/// DataFusion async scalar UDF implementation backed by an `InfusionUdfDescriptor`.
///
/// Registered per field (INV-INFUSE-001 / BC-2.19.001): each `[[infusion.fields]]`
/// entry produces one `InfusionAsyncUdf` instance registered in the `SessionContext`.
///
/// `invoke_async_with_args` performs the actual enrichment call via the descriptor's
/// `InfusionSource`. `invoke_with_args` returns `not_impl_err!` to force the async path —
/// this is the correct pattern; an incorrectly-wrapped UDF fails loudly.
///
/// # PartialEq / Eq / Hash keying
/// `ScalarUDFImpl` requires `DynEq + DynHash` (auto-impl'd for `Eq + Hash + Any` types).
/// We key equality and hashing on the UDF name, which is globally unique within a
/// `SessionContext` (DataFusion enforces uniqueness at registration time).
///
/// `#[non_exhaustive]`: forward-compat per CLAUDE.md §Conventions.
#[non_exhaustive]
#[derive(Debug)]
pub struct InfusionAsyncUdf {
    /// The infusion UDF descriptor exported by `prism-spec-engine`.
    descriptor: InfusionUdfDescriptor,
    /// DataFusion function signature (input: Utf8, output: Utf8 — simplified for stub).
    signature: Signature,
    /// Function name — stored separately so `ScalarUDFImpl::name` can return `&str`.
    name: String,
}

impl PartialEq for InfusionAsyncUdf {
    fn eq(&self, other: &Self) -> bool {
        // UDF names are unique within a SessionContext; equality is keyed on name.
        self.name == other.name
    }
}

impl Eq for InfusionAsyncUdf {}

impl Hash for InfusionAsyncUdf {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash keyed on name to be consistent with PartialEq.
        self.name.hash(state);
    }
}

impl InfusionAsyncUdf {
    /// Construct an `InfusionAsyncUdf` from an `InfusionUdfDescriptor`.
    pub fn new(descriptor: InfusionUdfDescriptor) -> Self {
        // Simplified signature: one Utf8 input → Utf8 output.
        // The full implementation will map `descriptor.input_type` / `descriptor.output_type`
        // to the canonical Arrow DataType (S-DEMO-ENRICHMENT-PIVOT-001 TDD green phase).
        let signature = Signature::new(
            TypeSignature::Exact(vec![DataType::Utf8]),
            Volatility::Volatile,
        );
        let name = descriptor.name.clone();
        Self {
            descriptor,
            signature,
            name,
        }
    }
}

// `ScalarUDFImpl` is the base trait that `AsyncScalarUDFImpl` extends.
// Both must be implemented — `AsyncScalarUDF::new(Arc::new(impl))` wraps
// them into a `ScalarUDF` via `into_scalar_udf()`.
impl ScalarUDFImpl for InfusionAsyncUdf {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> DataFusionResult<DataType> {
        // Simplified: always returns Utf8 for the stub.
        // The full implementation maps `descriptor.output_type` to a DataType.
        Ok(DataType::Utf8)
    }

    /// Synchronous fallback — MUST return `not_impl_err!` to force the async execution path.
    ///
    /// Per AC-003: a stub returning a constant here would produce a `not_impl_err!` failure
    /// on the sync path, ensuring the Red Gate test cannot pass vacuously.
    fn invoke_with_args(&self, _args: ScalarFunctionArgs) -> DataFusionResult<ColumnarValue> {
        datafusion::common::not_impl_err!(
            "InfusionAsyncUdf '{}': use invoke_async_with_args (async context only). \
             Sync execution path is unsupported for plugin-backed enrichment UDFs.",
            self.descriptor.name
        )
    }
}

// `AsyncScalarUDFImpl` is declared with `#[async_trait]` in DataFusion 53.1.
// This impl block MUST also be annotated with `#[async_trait]` to match
// the lifetime desugaring from the macro on the trait declaration.
// Without it, the compiler reports "lifetimes do not match method in trait".
#[async_trait]
impl AsyncScalarUDFImpl for InfusionAsyncUdf {
    /// Async enrichment call — the production execution path.
    ///
    /// For each row in the input batch, calls `self.descriptor.source.enrich_single(input, input_type)`
    /// and returns the enriched values as a `StringArray` wrapped in `ColumnarValue::Array`.
    ///
    /// If the source returns `None` for a row (plugin failure, no enrichment available),
    /// the row's output is `null` in the output array.
    ///
    /// If the source returns `Some(Value)`, the JSON value is serialized to a string.
    /// For plain strings in the JSON value (the common case for plugin enrichment results),
    /// the string is returned unwrapped to avoid double-quoting.
    ///
    /// # Input argument
    /// Expects exactly one `Utf8` column as the first argument (enforced by the `Signature`).
    /// `ColumnarValue::Scalar` inputs are treated as a single-element batch.
    async fn invoke_async_with_args(
        &self,
        args: ScalarFunctionArgs,
    ) -> DataFusionResult<ColumnarValue> {
        use datafusion::arrow::array::{Array, StringArray};
        use datafusion::common::ScalarValue;

        // Extract the input column — must be the first arg.
        let input_col = args.args.first().ok_or_else(|| {
            datafusion::error::DataFusionError::Execution(format!(
                "InfusionAsyncUdf '{}': expected 1 argument, got 0",
                self.descriptor.name
            ))
        })?;

        // Materialise the input as a list of (index, value) pairs.
        // For scalar inputs, expand to a single element.
        let inputs: Vec<Option<String>> = match input_col {
            ColumnarValue::Array(arr) => {
                let str_arr = arr.as_any().downcast_ref::<StringArray>().ok_or_else(|| {
                    datafusion::error::DataFusionError::Execution(format!(
                        "InfusionAsyncUdf '{}': input column must be Utf8, got {:?}",
                        self.descriptor.name,
                        arr.data_type()
                    ))
                })?;
                (0..str_arr.len())
                    .map(|i| {
                        if str_arr.is_null(i) {
                            None
                        } else {
                            Some(str_arr.value(i).to_owned())
                        }
                    })
                    .collect()
            }
            ColumnarValue::Scalar(scalar) => {
                // Single scalar — produce a single-element list.
                let value = match scalar {
                    ScalarValue::Utf8(opt) => opt.clone(),
                    ScalarValue::LargeUtf8(opt) => opt.clone(),
                    ScalarValue::Null => None,
                    other => {
                        return Err(datafusion::error::DataFusionError::Execution(format!(
                            "InfusionAsyncUdf '{}': scalar input must be Utf8 or Null, got {:?}",
                            self.descriptor.name,
                            other.data_type()
                        )));
                    }
                };
                vec![value]
            }
        };

        // Enrich each row via the plugin source.
        // NULL input rows short-circuit to NULL output without dispatching to the source
        // (a NULL IOC has no enrichment key — calling enrich_single("", ...) is incorrect).
        let enriched: Vec<Option<String>> = inputs
            .iter()
            .map(|opt_input| {
                let input_str = opt_input.as_deref()?;
                let result = self
                    .descriptor
                    .source
                    .enrich_single(input_str, &self.descriptor.input_type);
                result.map(|json_val| {
                    // If the plugin returns a plain JSON string, unwrap it to avoid
                    // double-quoting ("value" → value). Other JSON shapes are serialized as-is.
                    match json_val {
                        serde_json::Value::String(s) => s,
                        other => other.to_string(),
                    }
                })
            })
            .collect();

        // Build the output StringArray (nulls where enrichment returned None).
        let output = StringArray::from(
            enriched
                .iter()
                .map(|opt| opt.as_deref())
                .collect::<Vec<_>>(),
        );
        Ok(ColumnarValue::Array(Arc::new(output)))
    }
}

// ---------------------------------------------------------------------------
// register_infusion_udfs — wire descriptors into SessionContext
// ---------------------------------------------------------------------------

/// Register all infusion UDF descriptors as DataFusion async scalar UDFs.
///
/// Called at both `SessionContext` construction sites in `engine.rs`:
/// - the `execute` path (`execute_inner`)
/// - the `execute_scheduled` path
///
/// Each `InfusionUdfDescriptor` from `registry.udf_descriptors()` becomes one
/// `AsyncScalarUDF` registered via `ctx.register_udf(...)`.
///
/// # DataFusion 53.1 async UDF wiring (RESEARCH CONFIRMED)
/// `ctx.register_udf(AsyncScalarUDF::new(Arc::new(impl)).into_scalar_udf())` alone
/// is sufficient. `DefaultPhysicalPlanner` handles async UDFs natively.
/// No `AsyncFunctionRule` or config flag needed.
///
/// # Merge-coordination note (S-3.13)
/// `engine.rs` is also touched by S-3.13 (capability-discovery block, TableRegistry).
/// This function is the minimal call site — `register_infusion_udfs(&ctx, descriptors)` —
/// to minimize rebase conflict surface with S-3.13.
pub fn register_infusion_udfs(
    ctx: &SessionContext,
    descriptors: Vec<InfusionUdfDescriptor>,
) -> datafusion::error::Result<()> {
    // Detect duplicate UDF names before registration (registration-time defense-in-depth guard).
    // DataFusion's `register_udf` silently overwrites duplicates, which would cause
    // the last-registered UDF for a given name to win — a silent misconfiguration.
    //
    // Taxonomy: E-INFUSE-002 — registration-time defense-in-depth variant.
    //   File-load-time variant (validate_spec_against / DuplicateUdfName{path1,path2}):
    //     "Duplicate UDF name '{udf_name}' in '{path2}' — already registered from '{path1}'."
    //   This call site has no file paths — it keys on infusion_id instead. The message
    //   explicitly cites udf_name + infusion_id to satisfy the taxonomy's identity requirements.
    // E-INFUSE-007 is reserved for DataFusion `register_udf`/`register_table_function`
    // CALL failures (i.e., when DataFusion itself rejects the registration at runtime).
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for descriptor in descriptors {
        if !seen_names.insert(descriptor.name.clone()) {
            return Err(datafusion::error::DataFusionError::Execution(format!(
                "E-INFUSE-002: Duplicate UDF name '{}' from infusion '{}' — a UDF with this name \
                 was already registered from a prior infusion spec. Each infusion field must \
                 produce a unique UDF name within the DataFusion SessionContext.",
                descriptor.name, descriptor.infusion_id,
            )));
        }
        let udf_impl = InfusionAsyncUdf::new(descriptor);
        let async_udf = AsyncScalarUDF::new(Arc::new(udf_impl));
        ctx.register_udf(async_udf.into_scalar_udf());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use datafusion::execution::context::SessionContext;
    use prism_spec_engine::{InfusionSource, InfusionUdfDescriptor};

    use super::register_infusion_udfs;

    // ── test helpers ────────────────────────────────────────────────────────

    /// Stub `InfusionSource` that counts calls to `enrich_single`.
    #[derive(Debug)]
    struct CountingSource {
        call_count: Arc<AtomicUsize>,
        return_value: Option<serde_json::Value>,
    }

    impl CountingSource {
        fn new_returning(val: &str) -> (Arc<AtomicUsize>, Arc<dyn InfusionSource>) {
            let counter = Arc::new(AtomicUsize::new(0));
            let src = Arc::new(CountingSource {
                call_count: Arc::clone(&counter),
                return_value: Some(serde_json::Value::String(val.to_string())),
            });
            (counter, src)
        }
    }

    impl InfusionSource for CountingSource {
        fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            self.return_value.clone()
        }

        fn enrich_batch(
            &self,
            inputs: &[String],
            input_type: &str,
        ) -> Vec<Option<serde_json::Value>> {
            inputs
                .iter()
                .map(|i| self.enrich_single(i, input_type))
                .collect()
        }
    }

    fn make_descriptor(
        name: &str,
        infusion_id: &str,
        source: Arc<dyn InfusionSource>,
    ) -> InfusionUdfDescriptor {
        InfusionUdfDescriptor {
            name: name.to_string(),
            input_type: "ip".to_string(),
            output_type: "string".to_string(),
            infusion_id: infusion_id.to_string(),
            source,
            source_column: None,
        }
    }

    // ── Finding 1 tests ─────────────────────────────────────────────────────

    /// Happy-path: two descriptors with DISTINCT names register successfully.
    #[test]
    fn test_register_infusion_udfs_distinct_names_ok() {
        let ctx = SessionContext::new();
        let (_, src_a) = CountingSource::new_returning("a");
        let (_, src_b) = CountingSource::new_returning("b");
        let descriptors = vec![
            make_descriptor("geoip_country", "geoip", src_a),
            make_descriptor("asset_owner", "asset", src_b),
        ];
        let result = register_infusion_udfs(&ctx, descriptors);
        assert!(
            result.is_ok(),
            "distinct UDF names must register without error; got: {:?}",
            result
        );
    }

    /// E-INFUSE-002: duplicate UDF name emits error with real infusion_id and E-INFUSE-002 code.
    ///
    /// Taxonomy source-of-truth (error-taxonomy.md §INFUSE):
    ///   E-INFUSE-002 — "Duplicate UDF name '{udf_name}' in '{path2}' — already registered
    ///                   from '{path1}'." — spec-load-time collision BEFORE DataFusion registration.
    ///   E-INFUSE-007 — runtime `register_udf` call failure from DataFusion itself.
    ///
    /// Verifies:
    /// - `register_infusion_udfs` returns `Err` for duplicate names.
    /// - The error message contains `E-INFUSE-002`.
    /// - The error message does NOT contain `E-INFUSE-007` (that code is for DataFusion
    ///   registration failures, not pre-registration duplicate-name guard).
    /// - The error message contains the real `infusion_id` of the colliding spec.
    #[test]
    fn test_register_infusion_udfs_duplicate_name_emits_e_infuse_002_with_infusion_id() {
        let ctx = SessionContext::new();
        let (_, src_a) = CountingSource::new_returning("sentinel-a");
        let (_, src_b) = CountingSource::new_returning("sentinel-b");

        // Both descriptors share name "geoip_country" but belong to different infusion specs.
        // The second duplicate should trigger E-INFUSE-002 citing the *second* descriptor's
        // infusion_id (the one that caused the collision).
        let dup_infusion_id = "geoip_v2";
        let descriptors = vec![
            make_descriptor("geoip_country", "geoip_v1", src_a),
            make_descriptor("geoip_country", dup_infusion_id, src_b),
        ];
        let result = register_infusion_udfs(&ctx, descriptors);

        assert!(
            result.is_err(),
            "duplicate UDF name must return Err; got Ok"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("E-INFUSE-002"),
            "error must contain 'E-INFUSE-002' taxonomy code (duplicate UDF name at spec-load time, \
             NOT E-INFUSE-007 which is for DataFusion registration call failures); got: {err_msg}"
        );
        assert!(
            !err_msg.contains("E-INFUSE-007"),
            "error must NOT contain 'E-INFUSE-007' (that code is for runtime register_udf failures); \
             got: {err_msg}"
        );
        assert!(
            err_msg.contains(dup_infusion_id),
            "error must contain the real infusion_id '{}' of the colliding spec; got: {err_msg}",
            dup_infusion_id
        );
        // OBS-3: registration-time E-INFUSE-002 variant must cite both the udf_name AND the
        // infusion_id. The file-load-time variant uses {path1}/{path2}; at registration time
        // no file paths are available so infusion_id is the identity anchor instead.
        assert!(
            err_msg.contains("geoip_country"),
            "error must contain the udf_name 'geoip_country' (registration-time E-INFUSE-002 \
             variant must cite udf_name per OBS-3); got: {err_msg}"
        );
    }

    // ── Finding 2 tests ─────────────────────────────────────────────────────

    /// NULL-input rows must map to NULL output without invoking `enrich_single`.
    ///
    /// Drives `InfusionAsyncUdf::invoke_async_with_args` via a DataFusion SQL query
    /// against a table with one NULL row and one non-NULL row. Asserts:
    /// - `enrich_single` call_count == 1 (only the non-NULL row dispatches).
    /// - The NULL row produces a NULL output value.
    /// - The non-NULL row produces the sentinel enrichment value.
    #[tokio::test]
    async fn test_null_input_row_short_circuits_to_null_without_calling_enrich_single() {
        use datafusion::arrow::array::StringArray;
        use datafusion::arrow::datatypes::{DataType, Field, Schema};
        use datafusion::arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;

        let ctx = SessionContext::new();
        const SENTINEL: &str = "NULL-TEST-SENTINEL";
        let (call_count, src) = CountingSource::new_returning(SENTINEL);

        let descriptor = make_descriptor("null_test_udf", "null_test_infusion", src);
        register_infusion_udfs(&ctx, vec![descriptor]).expect("registration must succeed");

        // Table: two rows — row 0 is NULL, row 1 is "10.0.0.1".
        let schema = Arc::new(Schema::new(vec![Field::new("ioc", DataType::Utf8, true)]));
        let arr = StringArray::from(vec![None, Some("10.0.0.1")]);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("RecordBatch construction must succeed");
        let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
            .expect("MemTable construction must succeed");
        ctx.register_table("null_test_events", Arc::new(table))
            .expect("register_table must succeed");

        let df = ctx
            .sql("SELECT null_test_udf(ioc) AS enriched FROM null_test_events")
            .await
            .expect("SQL must parse");
        let batches = df.collect().await.expect("query must execute");

        // Verify enrich_single was called exactly once (for the non-NULL row).
        let count = call_count.load(Ordering::SeqCst);
        assert_eq!(
            count, 1,
            "enrich_single call_count must be 1 (NULL row must NOT dispatch); got {count}"
        );

        // Verify output: 2 rows total.
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 2, "must have 2 output rows; got {total_rows}");

        // Verify row 0 is NULL and row 1 contains the sentinel.
        use datafusion::arrow::array::Array;
        let output_col = batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("output column must be StringArray");

        assert!(
            output_col.is_null(0),
            "NULL input row must produce NULL output; got: {:?}",
            output_col.value(0)
        );
        assert!(
            !output_col.is_null(1),
            "non-NULL input row must produce non-NULL output"
        );
        assert_eq!(
            output_col.value(1),
            SENTINEL,
            "non-NULL row must produce the sentinel enrichment value"
        );
    }
}
