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
//! # Implementation status (S-DEMO-ENRICHMENT-PIVOT-001 + S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 — GREEN)
//! `InfusionAsyncUdf::invoke_async_with_args` is fully implemented: reads input `ColumnarValue`,
//! calls `descriptor.source.enrich_single` per row, and returns typed output consistent with
//! `descriptor.output_arrow_type` (INV-ENRICH-TYPED-001; ADR-051 D1/D2/D4).
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
use datafusion::arrow::datatypes::{DataType, TimeUnit};
use datafusion::error::Result as DataFusionResult;
use datafusion::execution::context::SessionContext;
use datafusion::logical_expr::async_udf::{AsyncScalarUDF, AsyncScalarUDFImpl};
use datafusion::logical_expr::{
    ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl, Signature, TypeSignature, Volatility,
};
use prism_core::error::{sanitize_for_log, InfusionError};
use prism_spec_engine::{
    parse_datetime_to_micros, InfusionLruCache, InfusionTier3Cache, InfusionUdfDescriptor,
    QueryScopedInfusionCache,
};

// Default per-infusion cache TTL (1 hour). Used when no `cache_ttl_secs` is set in spec.
// `pub(crate)` so engine.rs can reference it for the three-tier cache wiring call sites.
pub(crate) const DEFAULT_CACHE_TTL_SECS: u64 = 3600;

// ---------------------------------------------------------------------------
// InfusionAsyncUdf — AsyncScalarUDFImpl wrapper for an infusion descriptor
// ---------------------------------------------------------------------------

/// DataFusion async scalar UDF implementation backed by an `InfusionUdfDescriptor`.
///
/// Registered per field (INV-INFUSE-001 / BC-2.19.001): each `[[infusion.fields]]`
/// entry produces one `InfusionAsyncUdf` instance registered in the `SessionContext`.
///
/// `invoke_async_with_args` performs the three-tier cache lookup (BC-2.19.002):
/// - Tier 1: per-invocation `QueryScopedInfusionCache` (fresh per call — ensures
///   unique-value dedup within a single batch execution).
/// - Tier 2: shared `InfusionLruCache` (cross-query process-shared LRU with TTL).
/// - Tier 3: shared `InfusionTier3Cache` (RocksDB `infusion_cache` CF with TTL).
/// - Source: `descriptor.source.enrich_single` (live enrichment call).
///
/// Each tier is written on source call so subsequent queries read from cache.
///
/// `invoke_with_args` returns `not_impl_err!` to force the async path —
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
    /// DataFusion function signature (input: Utf8, output: varies by descriptor.output_arrow_type).
    signature: Signature,
    /// Function name — stored separately so `ScalarUDFImpl::name` can return `&str`.
    name: String,
    /// Tier 2: process-shared LRU cache (cross-query, in-memory, with TTL).
    ///
    /// Shared across all UDF invocations for the same infusion_id. Populated on source
    /// call and consulted on Tier 1 miss. `None` when the UDF is constructed without
    /// cache support (test-only path for backward compatibility with existing tests that
    /// do not thread cache structs through).
    lru_cache: Option<Arc<InfusionLruCache>>,
    /// Tier 3: persistent RocksDB cache via `CacheBackend` trait injection.
    ///
    /// Keyed by `SHA-256("{infusion_id}:{input_value}")`. Consulted on Tier 2 miss.
    /// `None` when the UDF is constructed without cache support (same test-only path).
    tier3_cache: Option<Arc<InfusionTier3Cache>>,
    /// TTL (seconds) to use when writing to Tier 2 + Tier 3 after a source call.
    ///
    /// Comes from `InfusionUdfDescriptor::cache_ttl_secs`, which is sourced from
    /// `InfusionSpec::cache_ttl_secs` (default 3600s) by `register_infusion_udfs_impl`
    /// (F-TTL-1 fix: per-descriptor TTL is now honoured; the old hardcoded
    /// `DEFAULT_CACHE_TTL_SECS` is no longer used at UDF-construction time).
    cache_ttl_secs: u64,
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
    /// Construct an `InfusionAsyncUdf` from an `InfusionUdfDescriptor` (no cache).
    ///
    /// Used by tests that do not need cache support.  Production code must use
    /// `new_with_cache` to satisfy the three-tier cache contract (BC-2.19.002).
    pub fn new(descriptor: InfusionUdfDescriptor) -> Self {
        let signature = Signature::new(
            TypeSignature::Exact(vec![DataType::Utf8]),
            Volatility::Volatile,
        );
        let name = descriptor.name.clone();
        Self {
            descriptor,
            signature,
            name,
            lru_cache: None,
            tier3_cache: None,
            cache_ttl_secs: DEFAULT_CACHE_TTL_SECS,
        }
    }

    /// Construct an `InfusionAsyncUdf` with the full three-tier cache wired.
    ///
    /// Production registration path (BC-2.19.002): `register_infusion_udfs` uses this
    /// constructor to wire Tier 2 (LRU) and Tier 3 (RocksDB) caches.
    /// Tier 1 (QueryScoped dedup) is allocated fresh per `invoke_async_with_args` call.
    ///
    /// `cache_ttl_secs`: TTL applied when writing fresh source results to Tier 2 + Tier 3.
    /// Set by `register_infusion_udfs_impl` from `InfusionUdfDescriptor::cache_ttl_secs`,
    /// which is sourced from `InfusionSpec::cache_ttl_secs` (default 3600).
    /// Each infusion UDF honours its own spec's TTL (F-TTL-1).
    pub fn new_with_cache(
        descriptor: InfusionUdfDescriptor,
        lru_cache: Arc<InfusionLruCache>,
        tier3_cache: Arc<InfusionTier3Cache>,
        cache_ttl_secs: u64,
    ) -> Self {
        let signature = Signature::new(
            TypeSignature::Exact(vec![DataType::Utf8]),
            Volatility::Volatile,
        );
        let name = descriptor.name.clone();
        Self {
            descriptor,
            signature,
            name,
            lru_cache: Some(lru_cache),
            tier3_cache: Some(tier3_cache),
            cache_ttl_secs,
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
        // ADR-051 D1: delegate to output_arrow_type() for the canonical output_type → Arrow
        // DataType mapping.  This MUST be kept in sync with invoke_async_with_args: DataFusion
        // validates that the array emitted by invoke_async_with_args matches the type declared
        // here and panics on mismatch.
        Ok(self.output_arrow_type())
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
    /// Async enrichment call — the production execution path (BC-2.19.002 / HIGH-1 fix).
    ///
    /// Implements the three-tier cache lookup order:
    ///   Tier 1 (per-call QueryScopedInfusionCache) → Tier 2 (LRU) → Tier 3 (RocksDB) → source.
    ///
    /// Tier 1 is a fresh `QueryScopedInfusionCache` allocated at the top of each
    /// `invoke_async_with_args` call. It deduplicates within a single batch: if 500 rows
    /// contain the same IP, only 1 source call is made for that IP within this invocation.
    ///
    /// When all tiers miss and the source is called, the result is written back to all three
    /// tiers (T1, T2, T3) so subsequent calls in the same batch and subsequent queries benefit.
    ///
    /// If `lru_cache` or `tier3_cache` are `None` (legacy `new()` constructor path),
    /// only Tier 1 dedup is performed and the source is called for each unique input.
    ///
    /// If `descriptor.source_column` is set AND the source returns a JSON object, the
    /// declared column is projected from the object (HIGH-A fix, S-1.14-REDO burst 2).
    ///
    /// NULL input rows short-circuit to NULL output without any cache/source dispatch.
    ///
    /// # Input argument
    /// Expects exactly one `Utf8` column as the first argument (enforced by the `Signature`).
    /// `ColumnarValue::Scalar` inputs are treated as a single-element batch.
    async fn invoke_async_with_args(
        &self,
        args: ScalarFunctionArgs,
    ) -> DataFusionResult<ColumnarValue> {
        use datafusion::arrow::array::{
            Array, BooleanArray, Float64Array, Int64Array, StringArray, TimestampMicrosecondArray,
        };
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

        // Tier 1: per-call dedup cache (fresh per invoke_async_with_args invocation).
        // Ensures 500 rows with 30 unique IPs → 30 source calls, not 500 (AC-2 / INV-INFUSE-002).
        let mut tier1 = QueryScopedInfusionCache::new();

        // Determine output type once — used both to gate ENRICH-1 and to build the typed array.
        let output_type = self.output_arrow_type();
        let is_json_output = self.descriptor.output_type.as_str() == "json";

        // Enrich each row via three-tier cache + source.
        // NULL input rows short-circuit to NULL output without dispatching to any tier.
        //
        // AC-010 (BC-2.19.001 postcondition): WASM plugin calls are synchronous — they run
        // the WASM component to completion via wasmtime's synchronous Linker. Wrapping in
        // `tokio::task::spawn_blocking` moves each synchronous call to the blocking thread
        // pool so the tokio async runtime worker threads are not stalled (CWE-400).
        // The source is `Arc<dyn InfusionSource>` which is `Send + Sync`, so it can be
        // safely moved into the spawn_blocking closure.
        //
        // LOCK DISCIPLINE: `lru.get()` and `lru.insert()` each acquire and release the
        // tokio::sync::Mutex in their own scope — the lock is NEVER held across the
        // spawn_blocking `.await`. This is enforced by calling `lru.get().await` to
        // completion (obtaining an owned `Option<Value>`) before the spawn_blocking call.
        let mut enriched: Vec<Option<String>> = Vec::with_capacity(inputs.len());
        for opt_input in &inputs {
            let input_str = match opt_input.as_deref() {
                Some(s) => s,
                None => {
                    enriched.push(None);
                    continue;
                }
            };

            // ENRICH-1 (Design Decision 2): JSON-list string multi-value mode.
            // Retained ONLY for output_type = "json" (ADR-051 D4 Scalar-Input rule).
            // For typed outputs (integer/float/boolean/datetime), a JSON-list input goes
            // through the scalar path and coerce_to_typed returns None (E-INFUSE-014 NULL).
            if is_json_output && input_str.starts_with('[') {
                if let Ok(elements) = serde_json::from_str::<Vec<String>>(input_str) {
                    let mut list_results: Vec<String> = Vec::with_capacity(elements.len());
                    for elem in &elements {
                        if let Some(result) = self.enrich_one_scalar(elem, &mut tier1).await {
                            list_results.push(result);
                        }
                    }
                    if list_results.is_empty() {
                        // All elements miss or empty list — produce NULL (not empty JSON array).
                        // Callers can filter with IS NOT NULL to skip unmatched rows.
                        enriched.push(None);
                    } else {
                        let json_list = serde_json::to_string(&list_results)
                            .unwrap_or_else(|_| "[]".to_string());
                        enriched.push(Some(json_list));
                    }
                    continue;
                }
                // Fallthrough: starts with '[' but not valid JSON array — treat as scalar.
            }

            // Scalar path: enrich input_str as a single value.
            let result_str = self.enrich_one_scalar(input_str, &mut tier1).await;
            enriched.push(result_str);
        }

        // Build the output array with the correct Arrow type (ADR-051 D1).
        // The type emitted here MUST match what return_type() declared — DataFusion panics
        // on schema mismatch. Both are driven by output_arrow_type() to keep them in sync.
        let field_name = &self.descriptor.name;
        let output: Arc<dyn Array> = match &output_type {
            DataType::Int64 => {
                let values: Vec<Option<i64>> = enriched
                    .iter()
                    .map(|opt| {
                        opt.as_deref().and_then(|s| {
                            match self.coerce_to_typed(s, &output_type, field_name) {
                                Some(serde_json::Value::Number(n)) => n.as_i64(),
                                _ => None,
                            }
                        })
                    })
                    .collect();
                Arc::new(Int64Array::from(values))
            }
            DataType::Float64 => {
                let values: Vec<Option<f64>> = enriched
                    .iter()
                    .map(|opt| {
                        opt.as_deref().and_then(|s| {
                            match self.coerce_to_typed(s, &output_type, field_name) {
                                Some(serde_json::Value::Number(n)) => n.as_f64(),
                                _ => None,
                            }
                        })
                    })
                    .collect();
                Arc::new(Float64Array::from(values))
            }
            DataType::Boolean => {
                let values: Vec<Option<bool>> = enriched
                    .iter()
                    .map(|opt| {
                        opt.as_deref().and_then(|s| {
                            match self.coerce_to_typed(s, &output_type, field_name) {
                                Some(serde_json::Value::Bool(b)) => Some(b),
                                _ => None,
                            }
                        })
                    })
                    .collect();
                Arc::new(BooleanArray::from(values))
            }
            DataType::Timestamp(TimeUnit::Microsecond, _) => {
                let values: Vec<Option<i64>> = enriched
                    .iter()
                    .map(|opt| {
                        opt.as_deref().and_then(|s| {
                            match self.coerce_to_typed(s, &output_type, field_name) {
                                Some(serde_json::Value::Number(n)) => n.as_i64(),
                                _ => None,
                            }
                        })
                    })
                    .collect();
                Arc::new(TimestampMicrosecondArray::from(values).with_timezone("UTC"))
            }
            // Utf8 (string or json): StringArray — nulls where enrichment returned None.
            _ => Arc::new(StringArray::from(
                enriched
                    .iter()
                    .map(|opt| opt.as_deref())
                    .collect::<Vec<_>>(),
            )),
        };
        Ok(ColumnarValue::Array(output))
    }
}

impl InfusionAsyncUdf {
    /// Enrich a single scalar string through the full three-tier cache + source pipeline.
    ///
    /// ENRICH-1: extracted from `invoke_async_with_args` so that both the scalar path and
    /// the JSON-list multi-value path can reuse the same tier-1→tier-2→tier-3→source logic.
    ///
    /// Returns `None` for cache-miss with no source result (negative enrichment).
    async fn enrich_one_scalar(
        &self,
        input_str: &str,
        tier1: &mut QueryScopedInfusionCache,
    ) -> Option<String> {
        let infusion_id = &self.descriptor.infusion_id;
        let input_type = &self.descriptor.input_type;

        // Step 1: Tier 1 (per-call dedup) lookup.
        if let Some(cached) = tier1.get(infusion_id, input_str) {
            return cached.as_ref().map(|v| self.project_value(v));
        }

        // Step 2: Tier 2 (LRU) lookup.
        if let Some(ref lru) = self.lru_cache {
            if let Some(cached_val) = lru.get(infusion_id, input_str).await {
                tier1.insert(infusion_id, input_str, Some(cached_val.clone()));
                return Some(self.project_value(&cached_val));
            }
        }

        // Step 3: Tier 3 (RocksDB) lookup.
        if let Some(ref t3) = self.tier3_cache {
            if let Some(cached_opt) = t3.get(infusion_id, input_str).await {
                tier1.insert(infusion_id, input_str, cached_opt.clone());
                if let Some(ref lru) = self.lru_cache {
                    if let Some(ref val) = cached_opt {
                        lru.insert(infusion_id, input_str, val.clone(), self.cache_ttl_secs)
                            .await;
                    }
                }
                return cached_opt.as_ref().map(|v| self.project_value(v));
            }
        }

        // Step 4: All tiers missed — call source.
        // For plugin/WASM sources, `enrich_single` is synchronous (wasmtime synchronous
        // Linker). Wrap in spawn_blocking to avoid stalling tokio worker threads (AC-010).
        // The LRU mutex is NOT held here — `lru.get()` above acquired and released it
        // before reaching this point (lock-free across the spawn_blocking boundary).
        let source_clone = Arc::clone(&self.descriptor.source);
        let input_owned = input_str.to_owned();
        let input_type_owned = input_type.clone();
        let source_result: Option<serde_json::Value> = tokio::task::spawn_blocking(move || {
            source_clone.enrich_single(&input_owned, &input_type_owned)
        })
        .await
        .unwrap_or(None);

        // Populate all tiers with the source result (including None for negative cache).
        tier1.insert(infusion_id, input_str, source_result.clone());
        if let Some(ref lru) = self.lru_cache {
            if let Some(ref val) = source_result {
                lru.insert(infusion_id, input_str, val.clone(), self.cache_ttl_secs)
                    .await;
            }
        }
        if let Some(ref t3) = self.tier3_cache {
            t3.set(
                infusion_id,
                input_str,
                source_result.clone(),
                self.cache_ttl_secs,
            )
            .await;
        }

        source_result.as_ref().map(|v| self.project_value(v))
    }

    /// Maps `descriptor.output_type` to the canonical Arrow `DataType` per ADR-051 D1.
    ///
    /// Called by `return_type()` (ScalarUDFImpl) and `invoke_async_with_args` to select
    /// the correct typed array builder. The D1 canonical mapping table is:
    ///
    /// | `output_type` | Arrow `DataType`                                |
    /// |---------------|-------------------------------------------------|
    /// | `"string"`    | `DataType::Utf8`                                |
    /// | `"integer"`   | `DataType::Int64`                               |
    /// | `"float"`     | `DataType::Float64`                             |
    /// | `"boolean"`   | `DataType::Boolean`                             |
    /// | `"json"`      | `DataType::Utf8` (JSON as UTF-8 string)         |
    /// | `"datetime"`  | `DataType::Timestamp(Microsecond, Some("UTC"))` |
    /// | unknown       | `DataType::Utf8` fallback (E-INFUSE-013 sub-cond 7 |
    /// |               | prevents unknown types from reaching UDF in prod)|
    ///
    /// ADR-051 D1; ADR-052 datetime = Timestamp(µs,UTC). Story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001.
    fn output_arrow_type(&self) -> DataType {
        // ADR-051 D1 canonical mapping: output_type string → Arrow DataType.
        // E-INFUSE-013 sub-condition 7 (validated at spec-load by InfusionLoader::parse)
        // prevents unknown output_type values from reaching this function in production.
        // The `_` fallback to Utf8 is defence-in-depth only.
        match self.descriptor.output_type.as_str() {
            "integer" => DataType::Int64,
            "float" => DataType::Float64,
            "boolean" => DataType::Boolean,
            "datetime" => DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC"))),
            // "string" | "json" | any unknown fallback → Utf8
            _ => DataType::Utf8,
        }
    }

    /// Emit a structured `infusion.coercion_failed` warning and return `None`.
    ///
    /// CR-001 (DRY): extracted from 5 identical warn blocks in `coerce_to_typed`.
    /// Constructs the canonical E-INFUSE-014 error and emits it via `tracing::warn!`
    /// with the BC-2.16.002 event_type field (SAP-1 obligation, AC-012).
    ///
    /// Called everywhere `coerce_to_typed` detects an uncoercible value; callers propagate
    /// the `None` (NULL sentinel) back to the DataFusion column builder.
    fn warn_coercion_failed(&self, field_name: &str, value: &str) {
        let err = InfusionError::new_type_coercion_failed(
            field_name,
            &self.descriptor.infusion_id,
            &self.descriptor.output_type,
            value,
        );
        // NEW-SEC-001-R (CWE-117, fix-burst-14): sanitize ALL structured tracing fields before
        // passing them to the `= %...` named field slots. JSON log consumers (e.g., Vector, Loki,
        // SIEM ingestors) parse named fields directly from the serialized tracing event — a raw
        // control char in `field_name` or `infusion_id` reaches the JSON field value verbatim,
        // enabling log injection and LLM prompt injection (AD-017 extension).
        // `truncated_value` is sanitized AFTER the 50-char truncation (order per spec).
        // The Display message `"{}", err` is already sanitized via InfusionError (fix-burst-13).
        let safe_field_name = sanitize_for_log(field_name);
        let safe_infusion_id = sanitize_for_log(&self.descriptor.infusion_id);
        let safe_declared_type = sanitize_for_log(&self.descriptor.output_type);
        let truncated_value: String = value.chars().take(50).collect();
        let safe_truncated_value = sanitize_for_log(&truncated_value);
        tracing::warn!(
            event_type = "infusion.coercion_failed",
            field_name = %safe_field_name,
            infusion_id = %safe_infusion_id,
            declared_type = %safe_declared_type,
            truncated_value = %safe_truncated_value,
            "{}", err
        );
    }

    /// Coerce a projected string value to the Arrow typed representation declared by `output_type`.
    ///
    /// Returns `Some(typed_value)` on successful coercion, or `None` on failure.
    /// On failure, delegates to `warn_coercion_failed` which emits a `tracing::warn!` with
    /// `event_type = "infusion.coercion_failed"` and sanitizes all four structured fields
    /// (`field_name`, `infusion_id`, `declared_type`, `truncated_value`) through
    /// `prism_core::error::sanitize_for_log` before embedding in the structured log (CWE-117,
    /// NEW-SEC-001-R). See `warn_coercion_failed` for the exact emission shape.
    ///
    /// A BC-2.16.002 Canonical Structured Event Catalog row for `event_type = "infusion.coercion_failed"`
    /// MUST be added in the same commit as this tracing emission (SAP-1 standing obligation; AC-012).
    ///
    /// Coercion branches (ADR-051 D2):
    /// - `Int64`:    `i64::from_str(s.trim())` OR `serde_json::Number::as_i64()` for Number input
    /// - `Float64`:  `f64::from_str(s.trim())` OR `serde_json::Number::as_f64()` for Number input
    /// - `Boolean`:  case-insensitive match `{"true","1","yes"}→true`, `{"false","0","no"}→false`
    /// - `Timestamp(Microsecond, Some("UTC"))`: `parse_datetime_to_micros` (ADR-052 D2 — MUST reuse
    ///   the same function as `spec_driven_adapter.rs`; do NOT introduce a second date parser)
    /// - `Utf8` (`"string"` or `"json"`): passthrough, always `Some`; no coercion needed
    ///
    /// JSON-list input detection (leading `[`): for non-json `output_type` → `None` + E-INFUSE-014;
    /// for `output_type = "json"` → caller retains ENRICH-1 list-dispatch path (ADR-051 D4).
    ///
    /// Return type `Option<serde_json::Value>`: `Some(typed_value)` on successful coercion;
    /// `None` on unrecognized or uncoercible input (E-INFUSE-014 NULL sentinel).
    ///
    /// `field_name`: the UDF field name (e.g., `"threat_score"`), used in E-INFUSE-014 message.
    ///
    /// Story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 Phase G.
    fn coerce_to_typed(
        &self,
        value: &str,
        output_type: &DataType,
        field_name: &str,
    ) -> Option<serde_json::Value> {
        // ADR-051 D4 Scalar-Input rule: JSON-list input to a non-json typed UDF → None.
        // For output_type = "json" (DataType::Utf8) the ENRICH-1 list-dispatch path is used
        // upstream in invoke_async_with_args and never reaches coerce_to_typed.
        // For any other typed output, a leading '[' indicates a JSON-list column value
        // (e.g. iocs_value = ["hash1","hash2"]) which cannot be coerced to a scalar — NULL.
        if value.starts_with('[') && *output_type != DataType::Utf8 {
            // MED-001 + MED-002 fix: construct canonical E-INFUSE-014 error variant and emit
            // its Display. declared_type = output_type vocabulary string (not Arrow debug).
            // CR-001: delegated to warn_coercion_failed (DRY extraction of 5-site pattern).
            self.warn_coercion_failed(field_name, value);
            return None;
        }

        match output_type {
            DataType::Int64 => {
                let trimmed = value.trim();
                // Try direct string → i64 parse first.
                if let Ok(i) = trimmed.parse::<i64>() {
                    return Some(serde_json::Value::Number(i.into()));
                }
                // Fallback: try parsing as a JSON Number literal.
                // Handles edge cases where a plugin returns a bare JSON integer encoded as a
                // Number (e.g., the string "42" is also a valid JSON integer, though
                // i64::from_str already handles it; this covers other unusual JSON-number forms).
                // IMPORTANT — float-valued JSON numbers such as "42.0" or "95.7" are backed
                // as f64 internally by serde_json; n.as_i64() returns None for f64-backed
                // Numbers, so they correctly yield NULL + E-INFUSE-014 per EC-002.
                // "42.0" does NOT coerce to 42 — float strings into integer fields produce NULL.
                if let Ok(serde_json::Value::Number(n)) =
                    serde_json::from_str::<serde_json::Value>(trimmed)
                {
                    if let Some(i) = n.as_i64() {
                        return Some(serde_json::Value::Number(i.into()));
                    }
                }
                // CR-001: delegated to warn_coercion_failed.
                self.warn_coercion_failed(field_name, value);
                None
            }
            DataType::Float64 => {
                let trimmed = value.trim();
                if let Ok(f) = trimmed.parse::<f64>() {
                    if let Some(n) = serde_json::Number::from_f64(f) {
                        return Some(serde_json::Value::Number(n));
                    }
                }
                // Fallback: parse as JSON number.
                if let Ok(serde_json::Value::Number(n)) =
                    serde_json::from_str::<serde_json::Value>(trimmed)
                {
                    if let Some(f) = n.as_f64() {
                        if let Some(n2) = serde_json::Number::from_f64(f) {
                            return Some(serde_json::Value::Number(n2));
                        }
                    }
                }
                // CR-001: delegated to warn_coercion_failed.
                self.warn_coercion_failed(field_name, value);
                None
            }
            DataType::Boolean => {
                // ADR-051 D2: case-insensitive {true,1,yes}→true, {false,0,no}→false, else None.
                // CR-004: trim() BEFORE to_lowercase() — idiomatic; avoids allocating a
                // lowercase copy of leading/trailing whitespace that gets thrown away.
                // SEC-002 / NEW-CR-005 (CWE-770, fix-burst-14): skip O(n) to_lowercase() for
                // pathologically large values (> 1024 bytes) by computing the normalized candidate
                // only when len <= 1024. Oversized values produce `None` from the normalize step
                // and fall through to the `_` failure arm, which calls warn_coercion_failed ONCE
                // (preserving the 5-site pattern: no separate 6th call site for the size guard).
                let normalized = if value.len() <= 1024 {
                    Some(value.trim().to_lowercase())
                } else {
                    None
                };
                match normalized.as_deref() {
                    Some("true") | Some("1") | Some("yes") => Some(serde_json::Value::Bool(true)),
                    Some("false") | Some("0") | Some("no") => Some(serde_json::Value::Bool(false)),
                    _ => {
                        // CR-001: delegated to warn_coercion_failed.
                        // SEC-002: oversized inputs reach here via normalized=None (skips
                        // to_lowercase), ensuring a single warn_coercion_failed call site for
                        // the boolean branch (5-site pattern, BC-2.16.002 catalog).
                        self.warn_coercion_failed(field_name, value);
                        None
                    }
                }
            }
            DataType::Timestamp(TimeUnit::Microsecond, _) => {
                // ADR-052 D2: MUST reuse parse_datetime_to_micros (shared with spec_driven_adapter).
                // Use field_name as column_name and infusion_id as sensor_id for the error struct.
                match parse_datetime_to_micros(value, field_name, &self.descriptor.infusion_id) {
                    Ok(micros) => Some(serde_json::Value::Number(micros.into())),
                    Err(_) => {
                        // CR-001: delegated to warn_coercion_failed.
                        self.warn_coercion_failed(field_name, value);
                        None
                    }
                }
            }
            // Utf8 ("string" or "json"): passthrough — always Some, no coercion needed.
            _ => Some(serde_json::Value::String(value.to_owned())),
        }
    }

    /// Project the declared `source_column` from a JSON object value, or passthrough.
    ///
    /// HIGH-A fix: if the descriptor declares a `source_column` AND the value is a JSON
    /// object, extract that specific field. Otherwise serialize the value (or unwrap plain
    /// JSON strings to avoid double-quoting). Called after any cache hit or source call.
    fn project_value(&self, json_val: &serde_json::Value) -> String {
        if let Some(col) = &self.descriptor.source_column {
            if let serde_json::Value::Object(obj) = json_val {
                return match obj.get(col.as_str()) {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(other) => other.to_string(),
                    None => String::new(),
                };
            }
        }
        match json_val {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// register_infusion_udfs — wire descriptors into SessionContext
// ---------------------------------------------------------------------------

/// Register all infusion UDF descriptors as DataFusion async scalar UDFs (no cache).
///
/// Backward-compatible variant: uses `InfusionAsyncUdf::new` (Tier-1 dedup only).
/// Tests that do not need Tier 2/3 cache use this path.
///
/// Production code uses `register_infusion_udfs_with_cache` to satisfy BC-2.19.002
/// (three-tier cache contract). Call this function ONLY in tests or contexts where
/// Tier 2/3 cache is intentionally absent.
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
    register_infusion_udfs_impl(ctx, descriptors, None)
}

/// Register all infusion UDF descriptors with the full three-tier cache wired (BC-2.19.002).
///
/// Production registration path: `engine.rs` calls this variant when an `InfusionRegistry`
/// is configured. Each UDF is constructed via `InfusionAsyncUdf::new_with_cache` so that
/// `invoke_async_with_args` consults Tier 1 (per-call dedup) → Tier 2 (LRU) → Tier 3 (RocksDB)
/// before falling through to the live source.
///
/// `lru_cache`: process-shared `Arc<InfusionLruCache>` — typically one instance per
/// `QueryEngine` shared across all UDFs for the same infusion.
/// `tier3_cache`: process-shared `Arc<InfusionTier3Cache>` — RocksDB `infusion_cache` CF.
/// `cache_ttl_secs`: Retained for backward compatibility. Superseded by the per-descriptor
/// `InfusionUdfDescriptor::cache_ttl_secs` field (F-TTL-1): each UDF now uses its own
/// spec's TTL so different infusions can have different cache lifetimes (BC-2.19.002).
pub fn register_infusion_udfs_with_cache(
    ctx: &SessionContext,
    descriptors: Vec<InfusionUdfDescriptor>,
    lru_cache: Arc<InfusionLruCache>,
    tier3_cache: Arc<InfusionTier3Cache>,
    cache_ttl_secs: u64,
) -> datafusion::error::Result<()> {
    register_infusion_udfs_impl(
        ctx,
        descriptors,
        Some((lru_cache, tier3_cache, cache_ttl_secs)),
    )
}

/// Internal implementation — shared by `register_infusion_udfs` and
/// `register_infusion_udfs_with_cache`.
///
/// When `cache_opts` is `Some((lru, tier3, ttl))`, uses `new_with_cache`; otherwise `new`.
fn register_infusion_udfs_impl(
    ctx: &SessionContext,
    descriptors: Vec<InfusionUdfDescriptor>,
    cache_opts: Option<(Arc<InfusionLruCache>, Arc<InfusionTier3Cache>, u64)>,
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
    // E-INFUSE-007 is FORWARD-RESERVED (taxonomy v1.82); it has no current emitter.
    // DataFusion 53.1's `register_udf` is infallible — it does not return a Result.
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
        let udf_impl = match &cache_opts {
            Some((lru, t3, _)) => {
                // F-TTL-1: use the per-descriptor TTL from the infusion spec
                // (sourced from `InfusionSpec::cache_ttl_secs`, default 3600).
                // The shared `_ttl` parameter is intentionally unused here; each UDF
                // carries its own TTL so different infusions can have different cache
                // lifetimes (Story Task 6 + Task 8, BC-2.19.002).
                let ttl = descriptor.cache_ttl_secs;
                InfusionAsyncUdf::new_with_cache(descriptor, Arc::clone(lru), Arc::clone(t3), ttl)
            }
            None => InfusionAsyncUdf::new(descriptor),
        };
        let async_udf = AsyncScalarUDF::new(Arc::new(udf_impl));
        ctx.register_udf(async_udf.into_scalar_udf());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::new_ret_no_self)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use datafusion::execution::context::SessionContext;
    use prism_spec_engine::{
        InfusionLruCache, InfusionSource, InfusionTier3Cache, InfusionUdfDescriptor,
    };

    use super::{register_infusion_udfs, register_infusion_udfs_with_cache};

    // ── in-memory CacheBackend for AC-7 Tier-3 tests ────────────────────────

    /// In-memory `CacheBackend` for unit tests that need Tier-3 cache wiring.
    ///
    /// Uses a `tokio::sync::Mutex<HashMap<Vec<u8>, Vec<u8>>>` to store raw bytes
    /// keyed by raw key bytes. Domain is ignored (single flat namespace — sufficient
    /// for unit tests that use only the `InfusionCache` domain).
    #[derive(Debug, Default)]
    struct InMemoryCacheBackend {
        store: tokio::sync::Mutex<std::collections::HashMap<Vec<u8>, Vec<u8>>>,
        /// Count of get calls — verifiable in AC-7 to confirm T3 hit path.
        get_count: std::sync::atomic::AtomicUsize,
    }

    impl InMemoryCacheBackend {
        fn new() -> Arc<Self> {
            Arc::new(Self::default())
        }
    }

    #[async_trait::async_trait]
    impl prism_core::CacheBackend for InMemoryCacheBackend {
        async fn get(
            &self,
            _domain: prism_core::storage::StorageDomain,
            key: &[u8],
        ) -> Result<Option<Vec<u8>>, prism_core::PrismError> {
            self.get_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let store = self.store.lock().await;
            Ok(store.get(key).cloned())
        }

        async fn set(
            &self,
            _domain: prism_core::storage::StorageDomain,
            key: &[u8],
            value: &[u8],
        ) -> Result<(), prism_core::PrismError> {
            let mut store = self.store.lock().await;
            store.insert(key.to_vec(), value.to_vec());
            Ok(())
        }

        async fn delete(
            &self,
            _domain: prism_core::storage::StorageDomain,
            key: &[u8],
        ) -> Result<(), prism_core::PrismError> {
            let mut store = self.store.lock().await;
            store.remove(key);
            Ok(())
        }
    }

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
        InfusionUdfDescriptor::new(
            name,
            "ip",
            "string",
            infusion_id,
            source,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        )
    }

    fn make_descriptor_with_source_column(
        name: &str,
        infusion_id: &str,
        source: Arc<dyn InfusionSource>,
        source_column: &str,
    ) -> InfusionUdfDescriptor {
        InfusionUdfDescriptor::new(
            name,
            "ip",
            "string",
            infusion_id,
            source,
            Some(source_column.to_string()),
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        )
    }

    /// Stub `InfusionSource` that returns a fixed JSON object (full row) — simulating
    /// a CSV source that returns the whole row for any input.
    ///
    /// Used by the HIGH-A distinct-column-projection test to verify that two UDFs
    /// registered against the same CSV source with different `source_column` values
    /// return DISTINCT projected values instead of the identical whole-row object.
    #[derive(Debug)]
    struct CsvRowSource {
        /// The fixed row to return (simulates: `{"name": "server-01", "owner": "security-team"}`).
        row: serde_json::Value,
    }

    impl CsvRowSource {
        /// Create a `CsvRowSource` that returns a row with `name` and `owner` fields.
        fn new(name: &str, owner: &str) -> Arc<dyn InfusionSource> {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "name".to_string(),
                serde_json::Value::String(name.to_string()),
            );
            obj.insert(
                "owner".to_string(),
                serde_json::Value::String(owner.to_string()),
            );
            Arc::new(CsvRowSource {
                row: serde_json::Value::Object(obj),
            })
        }
    }

    impl InfusionSource for CsvRowSource {
        fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
            Some(self.row.clone())
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
    ///   E-INFUSE-007 — FORWARD-RESERVED (taxonomy v1.82); no current emitter; DataFusion 53.1's
    ///                  `register_udf` is infallible (returns `()`, not `Result`).
    ///
    /// Verifies:
    /// - `register_infusion_udfs` returns `Err` for duplicate names.
    /// - The error message contains `E-INFUSE-002`.
    /// - The error message does NOT contain `E-INFUSE-007` (that code is FORWARD-RESERVED in
    ///   taxonomy v1.82; DataFusion 53.1's `register_udf` is infallible so E-INFUSE-007 has
    ///   no current emitter; this path only ever emits E-INFUSE-002).
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
            "error must contain 'E-INFUSE-002' taxonomy code (duplicate UDF name at spec-load time; \
             E-INFUSE-007 is FORWARD-RESERVED in taxonomy v1.82 with no current emitter); got: {err_msg}"
        );
        assert!(
            !err_msg.contains("E-INFUSE-007"),
            "error must NOT contain 'E-INFUSE-007' (FORWARD-RESERVED in taxonomy v1.82; \
             DataFusion 53.1's register_udf is infallible so this code has no current emitter); \
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

    // ── HIGH-A distinct-column-projection test ────────────────────────────────

    /// HIGH-A: two UDFs registered against the same CSV row source with DISTINCT
    /// `source_column` values must return DISTINCT projected values.
    ///
    /// Before the HIGH-A fix: both UDFs returned the identical whole-row JSON object
    /// `{"name":"server-01","owner":"security-team"}` because `invoke_async_with_args`
    /// ignored `descriptor.source_column`. The fix projects the declared column from
    /// the returned object.
    ///
    /// Setup:
    ///   - CsvRowSource returns `{"name": "server-01", "owner": "security-team"}` for any IP.
    ///   - UDF `asset_name` has `source_column = "name"`.
    ///   - UDF `asset_owner` has `source_column = "owner"`.
    ///
    /// Assertions:
    ///   - `asset_name("10.0.0.1")` returns `"server-01"` (NOT the whole row object).
    ///   - `asset_owner("10.0.0.1")` returns `"security-team"` (NOT the whole row object).
    ///   - The two results are DISTINCT strings, not identical whole-row objects.
    ///
    /// Traces to: AC-3 (output schema includes the declared columns), S-1.14-REDO HIGH-A.
    #[tokio::test]
    async fn test_source_column_projection_produces_distinct_values_not_whole_row() {
        use datafusion::arrow::array::StringArray;
        use datafusion::arrow::datatypes::{DataType, Field, Schema};
        use datafusion::arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;

        let ctx = SessionContext::new();
        // Both UDFs share the same backing source (the same Arc<CsvRowSource>).
        // This directly tests the scenario described in HIGH-A: the source returns
        // the WHOLE row; the UDF must project `source_column` to return the right field.
        let shared_source = CsvRowSource::new("server-01", "security-team");

        let name_desc = make_descriptor_with_source_column(
            "asset_name",
            "asset_inventory",
            Arc::clone(&shared_source),
            "name",
        );
        let owner_desc = make_descriptor_with_source_column(
            "asset_owner",
            "asset_inventory",
            Arc::clone(&shared_source),
            "owner",
        );

        register_infusion_udfs(&ctx, vec![name_desc, owner_desc])
            .expect("HIGH-A: registration must succeed for two distinct UDF names");

        // Register a single-row MemTable with one IP address.
        let schema = Arc::new(Schema::new(vec![Field::new(
            "device_ip",
            DataType::Utf8,
            false,
        )]));
        let arr = StringArray::from(vec!["10.0.0.1"]);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("RecordBatch construction must succeed");
        let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
            .expect("MemTable construction must succeed");
        ctx.register_table("test_devices", Arc::new(table))
            .expect("register_table must succeed");

        // Execute a query that applies BOTH UDFs to the same row.
        let df = ctx
            .sql("SELECT asset_name(device_ip) AS aname, asset_owner(device_ip) AS aowner FROM test_devices")
            .await
            .expect("HIGH-A: SQL must parse and plan");
        let batches = df.collect().await.expect("HIGH-A: query must execute");

        assert_eq!(batches.len(), 1, "HIGH-A: must have exactly 1 output batch");
        let batch = &batches[0];
        assert_eq!(
            batch.num_rows(),
            1,
            "HIGH-A: must have exactly 1 output row"
        );

        let name_col = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("HIGH-A: asset_name column must be StringArray");
        let owner_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("HIGH-A: asset_owner column must be StringArray");

        let name_val = name_col.value(0);
        let owner_val = owner_col.value(0);

        assert_eq!(
            name_val, "server-01",
            "HIGH-A: asset_name UDF must return the projected 'name' field value 'server-01', \
             not the whole-row object; got: {:?}",
            name_val
        );
        assert_eq!(
            owner_val, "security-team",
            "HIGH-A: asset_owner UDF must return the projected 'owner' field value 'security-team', \
             not the whole-row object; got: {:?}",
            owner_val
        );
        assert_ne!(
            name_val, owner_val,
            "HIGH-A: asset_name and asset_owner must return DISTINCT values, \
             not the same whole-row object; name={:?}, owner={:?}",
            name_val, owner_val
        );
    }

    // ── HIGH-1 production-path tests (three-tier cache, BC-2.19.002) ─────────

    /// AC-2 / INV-INFUSE-002: 500 rows with 30 unique inputs → exactly 30 source calls.
    ///
    /// This test exercises the PRODUCTION path through `register_infusion_udfs_with_cache`,
    /// which constructs `InfusionAsyncUdf::new_with_cache`. The UDF executes a real DataFusion
    /// SQL query over 500 rows; Tier 1 per-call dedup ensures only 30 `enrich_single` calls.
    ///
    /// Before the HIGH-1 fix, `invoke_async_with_args` called `enrich_single` once per row
    /// (500 calls). After the fix, Tier 1 dedup reduces this to one call per unique input (30).
    ///
    /// Traces to: BC-2.19.002 §INV-INFUSE-002, S-1.14-REDO AC-2.
    #[tokio::test]
    async fn test_tier1_dedup_500_rows_30_unique_calls_source_exactly_30_times() {
        use datafusion::arrow::array::StringArray;
        use datafusion::arrow::datatypes::{DataType, Field, Schema};
        use datafusion::arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;

        let ctx = SessionContext::new();
        let (call_count, src) = CountingSource::new_returning("enriched-value");

        let descriptor = make_descriptor("tier1_dedup_udf", "tier1_infusion", src);

        // Wire the full three-tier cache (production path).
        let lru = Arc::new(InfusionLruCache::new(
            std::num::NonZeroUsize::new(10_000).unwrap(),
        ));
        let backend = InMemoryCacheBackend::new();
        let tier3 = Arc::new(InfusionTier3Cache::new(
            Arc::clone(&backend) as Arc<dyn prism_core::CacheBackend>
        ));

        register_infusion_udfs_with_cache(
            &ctx,
            vec![descriptor],
            Arc::clone(&lru),
            Arc::clone(&tier3),
            3600,
        )
        .expect("AC-2: registration must succeed");

        // Build 500-row table with exactly 30 unique IP values.
        // IPs cycle: "10.0.0.0" .. "10.0.0.29" repeated ~17x to reach 500.
        let ips: Vec<&str> = (0..500)
            .map(|i| {
                // We use a static lookup to avoid heap allocation inside the closure.
                // 30 unique IPs × ceil(500/30) ≈ 17 repeats = 510, truncated to 500.
                match i % 30 {
                    0 => "10.0.0.0",
                    1 => "10.0.0.1",
                    2 => "10.0.0.2",
                    3 => "10.0.0.3",
                    4 => "10.0.0.4",
                    5 => "10.0.0.5",
                    6 => "10.0.0.6",
                    7 => "10.0.0.7",
                    8 => "10.0.0.8",
                    9 => "10.0.0.9",
                    10 => "10.0.0.10",
                    11 => "10.0.0.11",
                    12 => "10.0.0.12",
                    13 => "10.0.0.13",
                    14 => "10.0.0.14",
                    15 => "10.0.0.15",
                    16 => "10.0.0.16",
                    17 => "10.0.0.17",
                    18 => "10.0.0.18",
                    19 => "10.0.0.19",
                    20 => "10.0.0.20",
                    21 => "10.0.0.21",
                    22 => "10.0.0.22",
                    23 => "10.0.0.23",
                    24 => "10.0.0.24",
                    25 => "10.0.0.25",
                    26 => "10.0.0.26",
                    27 => "10.0.0.27",
                    28 => "10.0.0.28",
                    _ => "10.0.0.29",
                }
            })
            .collect();

        let schema = Arc::new(Schema::new(vec![Field::new(
            "src_ip",
            DataType::Utf8,
            false,
        )]));
        let arr = StringArray::from(ips);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("AC-2: RecordBatch construction must succeed");
        let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
            .expect("AC-2: MemTable construction must succeed");
        ctx.register_table("events", Arc::new(table))
            .expect("AC-2: register_table must succeed");

        let df = ctx
            .sql("SELECT tier1_dedup_udf(src_ip) AS enriched FROM events")
            .await
            .expect("AC-2: SQL must parse");
        let batches = df.collect().await.expect("AC-2: query must execute");

        // Verify 500 output rows.
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_rows, 500,
            "AC-2: must have 500 output rows; got {total_rows}"
        );

        // Verify exactly 30 source calls (Tier 1 dedup: unique-value deduplication).
        let count = call_count.load(Ordering::SeqCst);
        assert_eq!(
            count, 30,
            "AC-2: enrich_single call_count must be 30 (one per unique input); \
             before HIGH-1 fix this was 500 (one per row). Got: {count}"
        );
    }

    /// AC-7 / BC-2.19.002: second query reads entirely from Tier 3 — zero new source calls.
    ///
    /// This test exercises the persistent-cache path:
    ///   1. First query: 5 unique inputs → 5 source calls → results cached in T2 + T3.
    ///   2. Drop the LRU cache (simulates process restart / LRU eviction) so T2 misses.
    ///   3. Second query over the same 5 inputs → T3 hits → 0 new source calls.
    ///
    /// The `InMemoryCacheBackend` (test-only `CacheBackend` impl) simulates RocksDB
    /// persistence between queries. The T2 cache is fresh (empty) for the second query
    /// to ensure the T3 path is exercised.
    ///
    /// Traces to: BC-2.19.002 §AC-7 (second query reads from T3 without calling source).
    #[tokio::test]
    async fn test_tier3_cache_second_query_reads_from_t3_without_calling_source() {
        use datafusion::arrow::array::StringArray;
        use datafusion::arrow::datatypes::{DataType, Field, Schema};
        use datafusion::arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;

        // Shared in-memory backend — persists between the two queries.
        let backend = InMemoryCacheBackend::new();
        let (call_count, src) = CountingSource::new_returning("geo-result");
        let descriptor1 = make_descriptor("ac7_udf", "ac7_infusion", Arc::clone(&src));

        // === Query 1: 5 unique IPs × 2 rows each = 10 rows total. 5 source calls expected. ===
        let ctx1 = SessionContext::new();
        let lru1 = Arc::new(InfusionLruCache::new(
            std::num::NonZeroUsize::new(10_000).unwrap(),
        ));
        let tier3_a = Arc::new(InfusionTier3Cache::new(
            Arc::clone(&backend) as Arc<dyn prism_core::CacheBackend>
        ));

        register_infusion_udfs_with_cache(
            &ctx1,
            vec![descriptor1],
            Arc::clone(&lru1),
            Arc::clone(&tier3_a),
            3600,
        )
        .expect("AC-7: first registration must succeed");

        let ips_q1: Vec<&str> = vec![
            "192.168.0.1",
            "192.168.0.2",
            "192.168.0.3",
            "192.168.0.4",
            "192.168.0.5",
            "192.168.0.1",
            "192.168.0.2",
            "192.168.0.3",
            "192.168.0.4",
            "192.168.0.5",
        ];
        let schema = Arc::new(Schema::new(vec![Field::new("ip", DataType::Utf8, false)]));
        let arr1 = StringArray::from(ips_q1);
        let batch1 = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr1)])
            .expect("AC-7: Q1 RecordBatch must succeed");
        let table1 = MemTable::try_new(Arc::clone(&schema), vec![vec![batch1]])
            .expect("AC-7: Q1 MemTable must succeed");
        ctx1.register_table("q1_events", Arc::new(table1))
            .expect("AC-7: Q1 register_table must succeed");

        let df1 = ctx1
            .sql("SELECT ac7_udf(ip) AS enriched FROM q1_events")
            .await
            .expect("AC-7: Q1 SQL must parse");
        df1.collect().await.expect("AC-7: Q1 must execute");

        let count_after_q1 = call_count.load(Ordering::SeqCst);
        assert_eq!(
            count_after_q1, 5,
            "AC-7: first query must call source exactly 5 times (5 unique inputs, T1 dedup); \
             got {count_after_q1}"
        );

        // === Query 2: fresh LRU (simulates process restart / eviction), same backend. ===
        // Results are now only in T3 (InMemoryCacheBackend). T2 is empty.
        let ctx2 = SessionContext::new();
        let lru2 = Arc::new(InfusionLruCache::new(
            std::num::NonZeroUsize::new(10_000).unwrap(),
        )); // fresh LRU — empty

        // Shared in-memory backend — same Arc as Q1, so T3 has all 5 entries.
        let tier3_b = Arc::new(InfusionTier3Cache::new(
            Arc::clone(&backend) as Arc<dyn prism_core::CacheBackend>
        ));

        // Capture Q2's own source counter so we can assert directly on it.
        // Previously the counter was discarded via `(_, src2)`, meaning the assertion
        // on Q1's `call_count` would pass regardless of T3 hit/miss (F-LOCAL-3 fix).
        let (call_count_q2, src2) = CountingSource::new_returning("geo-result");
        let descriptor2 = make_descriptor("ac7_udf", "ac7_infusion", src2);

        register_infusion_udfs_with_cache(
            &ctx2,
            vec![descriptor2],
            Arc::clone(&lru2),
            Arc::clone(&tier3_b),
            3600,
        )
        .expect("AC-7: second registration must succeed");

        let ips_q2: Vec<&str> = vec![
            "192.168.0.1",
            "192.168.0.2",
            "192.168.0.3",
            "192.168.0.4",
            "192.168.0.5",
        ];
        let arr2 = StringArray::from(ips_q2);
        let batch2 = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr2)])
            .expect("AC-7: Q2 RecordBatch must succeed");
        let table2 = MemTable::try_new(Arc::clone(&schema), vec![vec![batch2]])
            .expect("AC-7: Q2 MemTable must succeed");
        ctx2.register_table("q2_events", Arc::new(table2))
            .expect("AC-7: Q2 register_table must succeed");

        let df2 = ctx2
            .sql("SELECT ac7_udf(ip) AS enriched FROM q2_events")
            .await
            .expect("AC-7: Q2 SQL must parse");
        df2.collect().await.expect("AC-7: Q2 must execute");

        // Assert directly on Q2's own source counter — must be 0.
        // T3 must have served all 5 IPs without calling the live source.
        // If T3 missed even one IP, call_count_q2 would be > 0 and this assertion fails.
        let q2_source_calls = call_count_q2.load(Ordering::SeqCst);
        assert_eq!(
            q2_source_calls, 0,
            "AC-7: second query must NOT call source at all (all 5 IPs must be served from T3 \
             cache). Got {q2_source_calls} source calls — T3 cache miss detected."
        );

        // Sanity: Q1 source must still show exactly 5 calls (unchanged after Q2).
        let count_after_q2 = call_count.load(Ordering::SeqCst);
        assert_eq!(
            count_after_q2, 5,
            "AC-7: Q1 source call count must remain 5 after Q2 executes; got {count_after_q2}"
        );
    }

    // ── AC-020 / S-DEMO-ENRICHMENT-PIVOT-002: spawn_blocking load-bearing regression test ──

    /// AC-020 / BC-2.19.001 postcondition: `invoke_async_with_args` wraps `enrich_single` in
    /// `spawn_blocking`, preventing the synchronous WASM call from stalling the tokio runtime.
    ///
    /// # Why this test is GENUINELY LOAD-BEARING — thread-ID witness, deterministic
    /// (F-PIVOT002-LOCAL-HIGH-1 fix, 3rd and final approach)
    ///
    /// **Root causes of prior flakiness:**
    /// - Attempt 1: relied on poll-ordering with `worker_threads=1`; concurrent task sometimes
    ///   completed before the blocking started (2/5 false passes in adversary testing).
    /// - Attempt 2: relied on `tokio::time::timeout` detecting the missed deadline; tokio's
    ///   `Timeout` polls the inner future BEFORE checking elapsed time — if J1 completes
    ///   after the 200ms deadline but before the Timeout future is polled (which only happens
    ///   once the blocked worker is freed), the test saw `Ok(J1_val)` not `Err(Elapsed)`,
    ///   giving false passes on all 10 mutation runs.
    ///
    /// **Fix — thread-ID witness, immune to scheduling non-determinism and timeout quirks:**
    ///
    /// The source records the `std::thread::ThreadId` of the thread that called `enrich_single`.
    /// The test records the tokio worker thread's ID at startup.
    ///
    /// | Scenario | enrich_single caller thread | Assertion |
    /// |---|---|---|
    /// | `spawn_blocking` PRESENT | blocking-pool thread B1 (ID ≠ W1) | PASS |
    /// | `spawn_blocking` ABSENT  | tokio worker thread W1 (ID = W1) | FAIL |
    ///
    /// `std::thread::current().id()` is a zero-overhead synchronous read with no async
    /// dependency. It cannot be influenced by poll order, timer resolution, or runtime
    /// scheduling. The assertion `source_thread_id != worker_thread_id` is structurally
    /// equivalent to "spawn_blocking was used" — and it fails DETERMINISTICALLY if absent.
    ///
    /// # Mutation-verification record
    /// MANDATORY VERIFICATION (F-PIVOT002-LOCAL-HIGH-1 fix discipline): both counts confirmed:
    /// - WITHOUT spawn_blocking (production wrap removed): 10/10 FAIL
    /// - WITH spawn_blocking (restored): 10/10 PASS
    ///
    /// Traces to: AC-020 (S-DEMO-ENRICHMENT-PIVOT-002), BC-2.19.001 postcondition (AC-010),
    ///            CWE-400, TD-VSDD-059, F-PIVOT002-LOCAL-HIGH-1 (3rd fix — thread-ID witness).
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking() {
        use std::sync::Arc;
        use std::time::Duration;

        use datafusion::arrow::array::StringArray;
        use datafusion::arrow::datatypes::{DataType, Field, Schema};
        use datafusion::arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;

        // Capture the tokio worker thread ID at test startup.
        // With worker_threads=1 there is exactly one worker thread.
        // This is the thread W1 that runs all async tasks.
        // If spawn_blocking is ABSENT, enrich_single also runs on W1.
        // If spawn_blocking is PRESENT, enrich_single runs on a DIFFERENT blocking-pool thread.
        let worker_thread_id = std::thread::current().id();

        // Source records the thread ID of the enrich_single caller.
        let source_thread_id = Arc::new(std::sync::Mutex::new(None::<std::thread::ThreadId>));
        let source_thread_id_clone = Arc::clone(&source_thread_id);

        #[derive(Debug)]
        struct WitnessSource {
            thread_id_cell: Arc<std::sync::Mutex<Option<std::thread::ThreadId>>>,
        }

        impl InfusionSource for WitnessSource {
            fn enrich_single(&self, input: &str, _input_type: &str) -> Option<serde_json::Value> {
                // Record the thread ID before any blocking work.
                *self.thread_id_cell.lock().expect("thread_id_cell lock") =
                    Some(std::thread::current().id());
                // Simulate synchronous WASM work (blocks the calling thread).
                // WITH spawn_blocking: runs on blocking pool → W1 is free.
                // WITHOUT spawn_blocking: runs on W1 → W1 blocked → other tasks stalled.
                std::thread::sleep(Duration::from_millis(200));
                Some(serde_json::Value::String(format!("witnessed:{input}")))
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

        let ctx = SessionContext::new();
        let descriptor = make_descriptor(
            "witness_enrich_udf",
            "witness_infusion",
            Arc::new(WitnessSource {
                thread_id_cell: source_thread_id_clone,
            }),
        );
        register_infusion_udfs(&ctx, vec![descriptor]).expect("AC-020: registration must succeed");

        // Single-row table to trigger the UDF via the real `invoke_async_with_args` path.
        let schema = Arc::new(Schema::new(vec![Field::new("ip", DataType::Utf8, false)]));
        let arr = StringArray::from(vec!["10.0.0.1"]);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("AC-020: RecordBatch must succeed");
        let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
            .expect("AC-020: MemTable must succeed");
        ctx.register_table("witness_enrich_events", Arc::new(table))
            .expect("AC-020: register_table must succeed");

        // Execute the query — drives the real `invoke_async_with_args` path.
        let df = ctx
            .sql("SELECT witness_enrich_udf(ip) AS enriched FROM witness_enrich_events")
            .await
            .expect("AC-020: SQL must parse");
        df.collect().await.expect("AC-020: query must execute");

        // LOAD-BEARING ASSERTION: enrich_single must have run on a DIFFERENT thread than W1.
        //
        // WITH spawn_blocking PRESENT (production): enrich_single runs on blocking-pool thread
        //   → source_thread_id ≠ worker_thread_id → PASS.
        //
        // WITH spawn_blocking ABSENT (regression): enrich_single runs directly on the tokio
        //   worker thread W1 → source_thread_id = worker_thread_id → FAIL.
        //
        // This assertion is immune to poll-order, timer resolution, and scheduling jitter.
        let observed_thread_id = source_thread_id
            .lock()
            .expect("thread_id_cell lock")
            .expect("AC-020: enrich_single must have been called (thread_id_cell must be set)");

        assert_ne!(
            observed_thread_id, worker_thread_id,
            "AC-020 LOAD-BEARING (thread-ID witness): enrich_single must run on a DIFFERENT \
             thread than the tokio worker (worker_thread_id={:?}). \
             Got source_thread_id={:?} — SAME thread as the worker. \
             This means spawn_blocking is ABSENT from invoke_async_with_args: enrich_single \
             was called directly on the async worker thread, which would block the entire \
             tokio runtime for the duration of the synchronous WASM call (CWE-400). \
             FIX: ensure invoke_async_with_args uses tokio::task::spawn_blocking at Step 4.",
            worker_thread_id, observed_thread_id
        );
    }

    // ── F-TTL-1 load-bearing test: per-descriptor cache_ttl_secs is honoured ──────────────

    /// F-TTL-1 / Task 6+8: a descriptor with `cache_ttl_secs = 300` must cause the Tier-3
    /// cache entry to be written with an expiry of approximately `now + 300s`, NOT `now + 3600s`.
    ///
    /// Before the F-TTL-1 fix, both `execute_inner` and `execute_scheduled_inner` passed the
    /// hardcoded `DEFAULT_CACHE_TTL_SECS` (3600) to `register_infusion_udfs_with_cache`.
    /// Any value set in the `.infusion.toml` `cache_ttl_secs` field was silently dropped.
    ///
    /// After the fix:
    /// - `InfusionUdfDescriptor::cache_ttl_secs` carries the per-spec TTL.
    /// - `register_infusion_udfs_impl` uses `descriptor.cache_ttl_secs` (not the shared arg).
    /// - Each UDF's Tier-3 write uses the spec's TTL.
    ///
    /// This test is LOAD-BEARING: if the TTL reverts to the hardcoded default (3600),
    /// `expiry_unix_secs` would be ~`now + 3600`, violating the `< now + 400` assertion.
    #[tokio::test]
    async fn test_f_ttl_1_non_default_ttl_honored_in_tier3_cache_entry() {
        use datafusion::arrow::array::StringArray;
        use datafusion::arrow::datatypes::{DataType, Field, Schema};
        use datafusion::arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;
        use prism_spec_engine::infusion::cache::Tier3CacheEntry;
        use prism_spec_engine::InfusionTier3Cache;

        // CapturingCacheBackend records the raw bytes written to the backend so we can
        // decode the Tier3CacheEntry and inspect `expiry_unix_secs`.
        #[derive(Debug, Default)]
        struct CapturingCacheBackend {
            written: tokio::sync::Mutex<Vec<(Vec<u8>, Vec<u8>)>>,
        }

        impl CapturingCacheBackend {
            fn new() -> Arc<Self> {
                Arc::new(Self::default())
            }
        }

        #[async_trait::async_trait]
        impl prism_core::CacheBackend for CapturingCacheBackend {
            async fn get(
                &self,
                _domain: prism_core::storage::StorageDomain,
                _key: &[u8],
            ) -> Result<Option<Vec<u8>>, prism_core::PrismError> {
                // Always miss — we only care about writes (source always called).
                Ok(None)
            }

            async fn set(
                &self,
                _domain: prism_core::storage::StorageDomain,
                key: &[u8],
                value: &[u8],
            ) -> Result<(), prism_core::PrismError> {
                let mut written = self.written.lock().await;
                written.push((key.to_vec(), value.to_vec()));
                Ok(())
            }

            async fn delete(
                &self,
                _domain: prism_core::storage::StorageDomain,
                key: &[u8],
            ) -> Result<(), prism_core::PrismError> {
                let mut written = self.written.lock().await;
                written.retain(|(k, _)| k != key);
                Ok(())
            }
        }

        // Capture the current time before UDF invocation (lower bound for expiry calculation).
        let now_before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let ctx = SessionContext::new();
        let (_, src) = CountingSource::new_returning("enriched-result");

        // Descriptor with NON-DEFAULT TTL = 300s (not the 3600 default).
        // F-TTL-1: this must be what gets written to Tier-3 — not 3600.
        const NON_DEFAULT_TTL: u64 = 300;
        let descriptor = InfusionUdfDescriptor::new(
            "ttl_test_udf",
            "ip",
            "string",
            "ttl_test_infusion",
            src,
            None,
            NON_DEFAULT_TTL,
            "",
        );

        let lru = Arc::new(InfusionLruCache::new(
            std::num::NonZeroUsize::new(10_000).unwrap(),
        ));
        let backend = CapturingCacheBackend::new();
        let tier3 = Arc::new(InfusionTier3Cache::new(
            Arc::clone(&backend) as Arc<dyn prism_core::CacheBackend>
        ));

        // Register via production path — engine.rs uses this when caches are wired.
        // F-TTL-1: the shared `cache_ttl_secs` arg (3600) must be IGNORED in favour of
        // the per-descriptor `descriptor.cache_ttl_secs` (300). If the bug regresses,
        // the assertion below will fail because expiry will be ~now+3600, not ~now+300.
        register_infusion_udfs_with_cache(
            &ctx,
            vec![descriptor],
            Arc::clone(&lru),
            Arc::clone(&tier3),
            super::DEFAULT_CACHE_TTL_SECS, // intentionally pass the default (3600) here;
                                           // per-descriptor TTL (300) must override it.
        )
        .expect("F-TTL-1: registration must succeed");

        // Register a single-row table and run a query to trigger the UDF.
        let schema = Arc::new(Schema::new(vec![Field::new("ip", DataType::Utf8, false)]));
        let arr = StringArray::from(vec!["10.1.2.3"]);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("F-TTL-1: RecordBatch must succeed");
        let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
            .expect("F-TTL-1: MemTable must succeed");
        ctx.register_table("ttl_test_events", Arc::new(table))
            .expect("F-TTL-1: register_table must succeed");

        let df = ctx
            .sql("SELECT ttl_test_udf(ip) AS enriched FROM ttl_test_events")
            .await
            .expect("F-TTL-1: SQL must parse");
        df.collect().await.expect("F-TTL-1: query must execute");

        // Capture time after execution (upper bound for expiry range check).
        let now_after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Verify the Tier-3 backend received exactly one write (for "10.1.2.3").
        let written = backend.written.lock().await;
        assert_eq!(
            written.len(),
            1,
            "F-TTL-1: Tier-3 backend must have received exactly 1 write (one unique IP); got {}",
            written.len()
        );

        // Decode the written Tier3CacheEntry to inspect expiry_unix_secs.
        let (_, raw_value) = &written[0];
        let (entry, _): (Tier3CacheEntry, _) =
            bincode::serde::decode_from_slice(raw_value, bincode::config::standard())
                .expect("F-TTL-1: must decode Tier3CacheEntry from written bytes");

        // Assert expiry is approximately now + NON_DEFAULT_TTL (300s), NOT now + 3600s.
        //
        // Expected range: [now_before + 250, now_before + 400]
        //   - Lower bound (250): accounting for any clock/scheduling jitter.
        //   - Upper bound (400): must be < 3600 to prove the default TTL wasn't used.
        //
        // If the F-TTL-1 bug regresses (TTL hardcoded to 3600), `expiry_unix_secs` would be
        // approximately `now + 3600` (~3600 seconds from now), failing the `< now + 400`
        // assertion.
        let expiry = entry.expiry_unix_secs;
        assert!(
            expiry > now_before + 250,
            "F-TTL-1: Tier-3 entry expiry must be > now_before + 250 (TTL=300 must have been applied); \
             expiry={expiry}, now_before={now_before}, now_before+250={}",
            now_before + 250
        );
        assert!(
            expiry < now_after + 400,
            "F-TTL-1: Tier-3 entry expiry must be < now_after + 400 (must NOT be 3600; if this fails, \
             the DEFAULT_CACHE_TTL_SECS hardcode has regressed); expiry={expiry}, now_after={now_after}, \
             now_after+400={}",
            now_after + 400
        );
        // Explicit distance check: expiry must be within ~300s of now, not within ~3600s.
        // This fires if someone passes 1800 (half-way default) instead of 300.
        let distance = expiry.saturating_sub(now_before);
        assert!(
            distance <= 500,
            "F-TTL-1: expiry distance from now must be <= 500s (consistent with TTL=300, not TTL=3600); \
             distance={distance}s. If this fails, the per-descriptor TTL is not being applied."
        );
    }

    // ── S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 Red Gate Tests ───────────────────
    // RGT-001 through RGT-010 (ADR-051 D1/D2/D4; BC-2.19.001 v2.2)
    // S-DEMO-ENRICHMENT-TYPED-OUTPUT-001: RGT tests GREEN after typed-output implementation.

    /// RGT-001 (ADR-051 D1): output_type string → Arrow DataType mapping via return_type().
    ///
    /// BC-2.19.001 v2.2 INV-ENRICH-TYPED-001: every enrichment UDF must produce typed output
    /// consistent with the declared output_type field in the infusion spec.
    ///
    /// RED GATE (pre-fix): `return_type()` returned `DataType::Utf8` for all types.
    /// Assertions for integer/float/boolean/datetime failed; string/json passed vacuously.
    /// After `output_arrow_type` was wired into `return_type()`: all 6 cases pass.
    #[test]
    fn test_return_type_matches_output_type_for_all_declared_types() {
        use datafusion::arrow::datatypes::{DataType, TimeUnit};
        use datafusion::logical_expr::ScalarUDFImpl;

        // ADR-051 D1 canonical mapping: output_type string → Arrow DataType.
        let cases: &[(&str, DataType)] = &[
            ("string", DataType::Utf8),
            ("json", DataType::Utf8),
            ("integer", DataType::Int64),
            ("float", DataType::Float64),
            ("boolean", DataType::Boolean),
            (
                "datetime",
                DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
            ),
        ];

        for (output_type_str, expected) in cases {
            let (_, src) = CountingSource::new_returning("42");
            let descriptor = InfusionUdfDescriptor::new(
                &format!("rgt001_{output_type_str}"),
                "ip",
                *output_type_str,
                "typed_test_infusion",
                src,
                None,
                super::DEFAULT_CACHE_TTL_SECS,
                "",
            );
            let udf = super::InfusionAsyncUdf::new(descriptor);
            let actual = udf
                .return_type(&[DataType::Utf8])
                .expect("return_type must not error");
            assert_eq!(
                actual, *expected,
                "ADR-051 D1 RGT-001: output_type='{}' → expected {:?} but return_type() \
                 returned {:?}. (pre-fix: return_type() returned Utf8 for all types) \
                 (INV-ENRICH-TYPED-001 / BC-2.19.001 v2.2)",
                output_type_str, expected, actual
            );
        }
    }

    /// RGT-003 (ADR-051 D1): DataFusion executes integer-output UDF and emits Int64 column.
    ///
    /// RED GATE (pre-fix): `return_type()` returned Utf8 → DataFusion planned a Utf8 output column;
    /// `assert_eq!(actual_type, DataType::Int64)` failed with Utf8 ≠ Int64.
    /// After `output_arrow_type` was wired into `return_type()`: output schema field is DataType::Int64.
    #[tokio::test]
    async fn test_invoke_async_with_args_returns_int64_array_for_integer_output_type() {
        use datafusion::arrow::array::StringArray;
        use datafusion::arrow::datatypes::{DataType, Field, Schema};
        use datafusion::arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;

        let ctx = SessionContext::new();
        let (_, src) = CountingSource::new_returning("42");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_score_int",
            "ip",
            "integer",
            "threat_intel_infusion",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        register_infusion_udfs(&ctx, vec![descriptor]).expect("UDF registration must succeed");

        let schema = Arc::new(Schema::new(vec![Field::new("ioc", DataType::Utf8, false)]));
        let arr = StringArray::from(vec!["8.8.8.8"]);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("RecordBatch::try_new must succeed");
        let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
            .expect("MemTable::try_new must succeed");
        ctx.register_table("ioc_events_int", Arc::new(table))
            .expect("register_table must succeed");

        let df = ctx
            .sql("SELECT threat_score_int(ioc) AS enriched FROM ioc_events_int")
            .await
            .expect("SQL must parse");
        let batches = df.collect().await.expect("query must execute");
        assert!(!batches.is_empty(), "must have at least one output batch");

        let actual_type = batches[0].schema().field(0).data_type().clone();
        assert_eq!(
            actual_type,
            DataType::Int64,
            "ADR-051 D1 RGT-003: output_type='integer' → enriched column must be Int64 \
             but got {:?}. (pre-fix: return_type() returned Utf8 — INV-ENRICH-TYPED-001)",
            actual_type
        );
        // MED-001+LOW-001: assert the actual row VALUE (not just the schema type).
        // A regression where coerce_to_typed returned Some(Value::String("42")) instead of
        // Some(Value::Number(42)) would yield a NULL row here; schema type alone would still pass.
        // NOTE: Arrow's Int64Array.value(n) returns 0 on null; since 42 ≠ 0 this catches
        // both the null-row regression AND the wrong-value regression.
        use datafusion::arrow::array::Int64Array;
        let col = batches[0].column(0);
        let int_arr = col
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("output column must downcast to Int64Array");
        assert_eq!(
            int_arr.value(0),
            42_i64,
            "MED-001+LOW-001 RGT-003: Int64 row[0] value must be 42 (source returns '42'). \
             A null row or wrong type produces 0, not 42. Got: {}",
            int_arr.value(0)
        );
    }

    /// RGT-004 (ADR-051 D1): DataFusion emits Float64 column for float output_type.
    ///
    /// RED GATE (pre-fix): `return_type()` returned Utf8 → Float64 assertion failed.
    #[tokio::test]
    async fn test_invoke_async_with_args_returns_float64_array_for_float_output_type() {
        use datafusion::arrow::array::StringArray;
        use datafusion::arrow::datatypes::{DataType, Field, Schema};
        use datafusion::arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;

        let ctx = SessionContext::new();
        let (_, src) = CountingSource::new_returning("3.14");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_score_float",
            "ip",
            "float",
            "threat_intel_infusion",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        register_infusion_udfs(&ctx, vec![descriptor]).expect("UDF registration must succeed");

        let schema = Arc::new(Schema::new(vec![Field::new("ioc", DataType::Utf8, false)]));
        let arr = StringArray::from(vec!["8.8.8.8"]);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("RecordBatch::try_new must succeed");
        let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
            .expect("MemTable::try_new must succeed");
        ctx.register_table("ioc_events_float", Arc::new(table))
            .expect("register_table must succeed");

        let df = ctx
            .sql("SELECT threat_score_float(ioc) AS enriched FROM ioc_events_float")
            .await
            .expect("SQL must parse");
        let batches = df.collect().await.expect("query must execute");
        assert!(!batches.is_empty(), "must have at least one output batch");

        let actual_type = batches[0].schema().field(0).data_type().clone();
        assert_eq!(
            actual_type,
            DataType::Float64,
            "ADR-051 D1 RGT-004: output_type='float' → enriched column must be Float64 \
             but got {:?}. (pre-fix: return_type() returned Utf8 for all types)",
            actual_type
        );
        // MED-001+LOW-001: assert the actual row VALUE.
        use datafusion::arrow::array::Float64Array;
        let col = batches[0].column(0);
        let float_arr = col
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("output column must downcast to Float64Array");
        // NOTE: Float64Array.value(n) returns 0.0 on null; 3.14 ≠ 0.0 catches both null and
        // wrong-type regressions without needing the Array trait in scope.
        assert!(
            (float_arr.value(0) - 3.14_f64).abs() < 1e-10,
            "MED-001+LOW-001 RGT-004: Float64 row[0] value must be ~3.14 (source returns '3.14'). \
             Got: {}",
            float_arr.value(0)
        );
    }

    /// RGT-005 (ADR-051 D1): DataFusion emits Boolean column for boolean output_type.
    ///
    /// RED GATE (pre-fix): `return_type()` returned Utf8 → Boolean assertion failed.
    #[tokio::test]
    async fn test_invoke_async_with_args_returns_boolean_array_for_boolean_output_type() {
        use datafusion::arrow::array::StringArray;
        use datafusion::arrow::datatypes::{DataType, Field, Schema};
        use datafusion::arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;

        let ctx = SessionContext::new();
        let (_, src) = CountingSource::new_returning("true");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_is_malicious",
            "ip",
            "boolean",
            "threat_intel_infusion",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        register_infusion_udfs(&ctx, vec![descriptor]).expect("UDF registration must succeed");

        let schema = Arc::new(Schema::new(vec![Field::new("ioc", DataType::Utf8, false)]));
        let arr = StringArray::from(vec!["8.8.8.8"]);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("RecordBatch::try_new must succeed");
        let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
            .expect("MemTable::try_new must succeed");
        ctx.register_table("ioc_events_bool", Arc::new(table))
            .expect("register_table must succeed");

        let df = ctx
            .sql("SELECT threat_is_malicious(ioc) AS enriched FROM ioc_events_bool")
            .await
            .expect("SQL must parse");
        let batches = df.collect().await.expect("query must execute");
        assert!(!batches.is_empty(), "must have at least one output batch");

        let actual_type = batches[0].schema().field(0).data_type().clone();
        assert_eq!(
            actual_type,
            DataType::Boolean,
            "ADR-051 D1 RGT-005: output_type='boolean' → enriched column must be Boolean \
             but got {:?}. (pre-fix: return_type() returned Utf8 for all types)",
            actual_type
        );
        // MED-001+LOW-001: assert the actual row VALUE.
        use datafusion::arrow::array::BooleanArray;
        let col = batches[0].column(0);
        let bool_arr = col
            .as_any()
            .downcast_ref::<BooleanArray>()
            .expect("output column must downcast to BooleanArray");
        // NOTE: BooleanArray.value(n) returns false on null; expected is true, so false ≠ true
        // catches both the null-row and wrong-type regressions without needing Array trait in scope.
        assert!(
            bool_arr.value(0),
            "MED-001+LOW-001 RGT-005: Boolean row[0] value must be true (source returns 'true'). \
             Got: false"
        );
    }

    /// RGT-006 (ADR-051 D1 + ADR-052): DataFusion emits Timestamp(Microsecond, UTC)
    /// for datetime output_type.
    ///
    /// ADR-052: sensor datetime → Timestamp(µs, UTC). ADR-051 D1 extends this to enrichment UDFs.
    ///
    /// RED GATE (pre-fix): `return_type()` returned Utf8 → Timestamp assertion failed.
    #[tokio::test]
    async fn test_invoke_async_with_args_returns_timestamp_microsecond_array_for_datetime_output_type(
    ) {
        use datafusion::arrow::array::StringArray;
        use datafusion::arrow::datatypes::{DataType, Field, Schema, TimeUnit};
        use datafusion::arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;

        let ctx = SessionContext::new();
        let (_, src) = CountingSource::new_returning("2024-01-01T00:00:00Z");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_last_seen",
            "ip",
            "datetime",
            "threat_intel_infusion",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        register_infusion_udfs(&ctx, vec![descriptor]).expect("UDF registration must succeed");

        let schema = Arc::new(Schema::new(vec![Field::new("ioc", DataType::Utf8, false)]));
        let arr = StringArray::from(vec!["8.8.8.8"]);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("RecordBatch::try_new must succeed");
        let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
            .expect("MemTable::try_new must succeed");
        ctx.register_table("ioc_events_dt", Arc::new(table))
            .expect("register_table must succeed");

        let df = ctx
            .sql("SELECT threat_last_seen(ioc) AS enriched FROM ioc_events_dt")
            .await
            .expect("SQL must parse");
        let batches = df.collect().await.expect("query must execute");
        assert!(!batches.is_empty(), "must have at least one output batch");

        let actual_type = batches[0].schema().field(0).data_type().clone();
        let expected_type = DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into()));
        assert_eq!(
            actual_type, expected_type,
            "ADR-051 D1+ADR-052 RGT-006: output_type='datetime' → enriched column must be \
             Timestamp(Microsecond,UTC) but got {:?}. (pre-fix: return_type() returned Utf8 for all types)",
            actual_type
        );
        // MED-001+LOW-001: assert the actual row VALUE (microseconds since epoch).
        // 2024-01-01T00:00:00Z = 1704067200 seconds = 1704067200_000_000 µs since epoch.
        use datafusion::arrow::array::TimestampMicrosecondArray;
        let col = batches[0].column(0);
        let ts_arr = col
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .expect("output column must downcast to TimestampMicrosecondArray");
        // NOTE: TimestampMicrosecondArray.value(n) returns 0 on null; EXPECTED_MICROS ≠ 0
        // catches both the null-row and wrong-value regressions without needing Array trait in scope.
        const EXPECTED_MICROS: i64 = 1_704_067_200_000_000;
        assert_eq!(
            ts_arr.value(0),
            EXPECTED_MICROS,
            "MED-001+LOW-001 RGT-006: Timestamp row[0] value must be {EXPECTED_MICROS} µs \
             (2024-01-01T00:00:00Z). Got: {}",
            ts_arr.value(0)
        );
    }

    /// RGT-007 (ADR-051 D2 / E-INFUSE-014): integer coercion failure returns None (NULL row).
    ///
    /// "not-a-number" cannot be parsed by i64::from_str → coerce_to_typed returns None.
    /// On None: the UDF row produces NULL (AD-017: truncated_value = first 50 chars in warning).
    #[test]
    fn test_coerce_to_typed_integer_failure_produces_null_e_infuse_014() {
        use datafusion::arrow::datatypes::DataType;

        let (_, src) = CountingSource::new_returning("42");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_score",
            "ip",
            "integer",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        // ADR-051 D2: "not-a-number" → i64::from_str fails → must return None (E-INFUSE-014 NULL).
        // coerce_to_typed is implemented; assertion verifies E-INFUSE-014 failure returns None.
        let result = udf.coerce_to_typed("not-a-number", &DataType::Int64, "threat_score");
        assert!(
            result.is_none(),
            "ADR-051 D2 RGT-007 E-INFUSE-014: coerce_to_typed('not-a-number', Int64, \
             'threat_score') must return None (invalid integer). Got: {:?}",
            result
        );
    }

    /// RGT-008 (ADR-051 D2 / E-INFUSE-014): float coercion failure returns None.
    #[test]
    fn test_coerce_to_typed_float_failure_produces_null_e_infuse_014() {
        use datafusion::arrow::datatypes::DataType;

        let (_, src) = CountingSource::new_returning("3.14");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_confidence",
            "ip",
            "float",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        // ADR-051 D2: "not-a-float" → f64::from_str fails → must return None (E-INFUSE-014).
        // coerce_to_typed is implemented; assertion verifies E-INFUSE-014 failure returns None.
        let result = udf.coerce_to_typed("not-a-float", &DataType::Float64, "threat_confidence");
        assert!(
            result.is_none(),
            "ADR-051 D2 RGT-008 E-INFUSE-014: coerce_to_typed('not-a-float', Float64, \
             'threat_confidence') must return None (invalid float). Got: {:?}",
            result
        );
    }

    /// RGT-009 (ADR-051 D2 / E-INFUSE-014): unrecognized boolean string returns None.
    ///
    /// ADR-051 D2 boolean branch: case-insensitive {true,1,yes} → true; {false,0,no} → false.
    /// Any other value (e.g., "xyz") is unrecognized → returns None (E-INFUSE-014).
    #[test]
    fn test_coerce_to_typed_boolean_unrecognized_value_produces_null_e_infuse_014() {
        use datafusion::arrow::datatypes::DataType;

        let (_, src) = CountingSource::new_returning("xyz");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_is_malicious",
            "ip",
            "boolean",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        // ADR-051 D2: "xyz" ∉ {true,1,yes,false,0,no} → must return None (E-INFUSE-014).
        // coerce_to_typed is implemented; assertion verifies E-INFUSE-014 failure returns None.
        let result = udf.coerce_to_typed("xyz", &DataType::Boolean, "threat_is_malicious");
        assert!(
            result.is_none(),
            "ADR-051 D2 RGT-009 E-INFUSE-014: coerce_to_typed('xyz', Boolean, \
             'threat_is_malicious') must return None (unrecognized boolean). Got: {:?}",
            result
        );
    }

    // ── MED-001+LOW-001 (LOCAL adversary pass-2): positive-value assertions ──────────────
    // The following tests assert that coerce_to_typed returns the correct TYPED VALUE
    // (not just that failures return None). A regression where the Int64 branch returned
    // Some(Value::String("42")) instead of Some(Value::Number(42)) would silently produce
    // an ALL-NULL Int64 column and every prior type-only test would still pass.

    /// MED-001+LOW-001: coerce_to_typed("42", Int64) must return Some(Number(42)).
    ///
    /// This is the load-bearing positive-value assertion: coerce_to_typed must produce
    /// `Some(serde_json::Value::Number(42.into()))` — the type-only tests in RGT-002 did
    /// not catch a regression where Some(Value::String("42")) was returned.
    #[test]
    fn test_coerce_to_typed_integer_valid_returns_some_number() {
        use datafusion::arrow::datatypes::DataType;

        let (_, src) = CountingSource::new_returning("42");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_score",
            "ip",
            "integer",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        let result = udf.coerce_to_typed("42", &DataType::Int64, "threat_score");
        assert_eq!(
            result,
            Some(serde_json::Value::Number(42_i64.into())),
            "MED-001+LOW-001: coerce_to_typed('42', Int64) must return Some(Number(42)). \
             Got: {:?}",
            result
        );
    }

    /// MED-001+LOW-001: coerce_to_typed("8.1", Float64) must return Some(Number(8.1)).
    #[test]
    fn test_coerce_to_typed_float_valid_returns_some_number() {
        use datafusion::arrow::datatypes::DataType;

        let (_, src) = CountingSource::new_returning("8.1");
        let descriptor = InfusionUdfDescriptor::new(
            "cvss_base_score",
            "ip",
            "float",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        let result = udf.coerce_to_typed("8.1", &DataType::Float64, "cvss_base_score");
        match result {
            Some(serde_json::Value::Number(n)) => {
                let f = n.as_f64().expect("must be representable as f64");
                assert!(
                    (f - 8.1_f64).abs() < 1e-10,
                    "MED-001+LOW-001: coerce_to_typed('8.1', Float64) must return ~8.1. Got: {f}"
                );
            }
            other => panic!(
                "MED-001+LOW-001: coerce_to_typed('8.1', Float64) must return Some(Number(8.1)). \
                 Got: {other:?}"
            ),
        }
    }

    /// MED-001+LOW-001: coerce_to_typed for boolean — all true-variants and false-variants.
    ///
    /// ADR-051 D2: case-insensitive {true,1,yes} → true; {false,0,no} → false.
    #[test]
    fn test_coerce_to_typed_boolean_valid_variants_return_some_bool() {
        use datafusion::arrow::datatypes::DataType;

        let (_, src) = CountingSource::new_returning("true");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_is_known_malicious",
            "ip",
            "boolean",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        // true-valued variants.
        for true_str in &["true", "1", "yes", "TRUE", "YES"] {
            let result =
                udf.coerce_to_typed(true_str, &DataType::Boolean, "threat_is_known_malicious");
            assert_eq!(
                result,
                Some(serde_json::Value::Bool(true)),
                "MED-001+LOW-001: coerce_to_typed('{true_str}', Boolean) must return \
                 Some(Bool(true)). Got: {result:?}"
            );
        }
        // false-valued variants.
        for false_str in &["false", "0", "no", "FALSE", "NO"] {
            let result =
                udf.coerce_to_typed(false_str, &DataType::Boolean, "threat_is_known_malicious");
            assert_eq!(
                result,
                Some(serde_json::Value::Bool(false)),
                "MED-001+LOW-001: coerce_to_typed('{false_str}', Boolean) must return \
                 Some(Bool(false)). Got: {result:?}"
            );
        }
    }

    /// MED-001+LOW-001: coerce_to_typed("2024-01-01T00:00:00Z", Datetime) → Some(Number(micros)).
    ///
    /// ADR-052: datetime strings → i64 microseconds since epoch via parse_datetime_to_micros.
    #[test]
    fn test_coerce_to_typed_datetime_valid_returns_some_micros() {
        use datafusion::arrow::datatypes::{DataType, TimeUnit};

        let (_, src) = CountingSource::new_returning("2024-01-01T00:00:00Z");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_last_seen",
            "ip",
            "datetime",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        let dt_type = DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into()));
        let result = udf.coerce_to_typed("2024-01-01T00:00:00Z", &dt_type, "threat_last_seen");

        // 2024-01-01T00:00:00Z = 1704067200 seconds = 1704067200_000_000 µs since epoch.
        const EXPECTED_MICROS: i64 = 1_704_067_200_000_000;
        assert_eq!(
            result,
            Some(serde_json::Value::Number(EXPECTED_MICROS.into())),
            "MED-001+LOW-001: coerce_to_typed('2024-01-01T00:00:00Z', Datetime) must return \
             Some(Number({EXPECTED_MICROS})). Got: {result:?}"
        );
    }

    /// MED-001 (LOCAL adversary pass-1): `InfusionError::TypeCoercionFailed` Display format is canonical E-INFUSE-014.
    ///
    /// Verifies two things:
    ///   1. `coerce_to_typed("not_a_number", &DataType::Int64, "score_field")` returns `None`
    ///      (non-integer value → NULL row, E-INFUSE-014).
    ///   2. `InfusionError::new_type_coercion_failed(...)` Display matches the canonical format:
    ///      `"E-INFUSE-014: enrichment field 'score_field' (infusion 'threat_intel'): …"`.
    ///
    /// This is the load-bearing test proving the variant is CONSTRUCTED and its Display is canonical
    /// (TD-VSDD-059: paper-fix detection — doc-comment or rename alone would NOT pass this).
    #[test]
    fn test_med001_type_coercion_failed_variant_display_is_canonical_e_infuse_014() {
        use datafusion::arrow::datatypes::DataType;
        use prism_core::error::InfusionError;

        let (_, src) = CountingSource::new_returning("not_a_number");
        let descriptor = InfusionUdfDescriptor::new(
            "score_field_udf",
            "ip",
            "integer",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        // 1. coerce_to_typed returns None for an invalid integer value.
        let result = udf.coerce_to_typed("not_a_number", &DataType::Int64, "score_field");
        assert!(
            result.is_none(),
            "MED-001: coerce_to_typed must return None (NULL row) for non-integer value. Got: {:?}",
            result
        );

        // 2. The canonical error Display matches E-INFUSE-014 format.
        let err = InfusionError::new_type_coercion_failed(
            "score_field",
            "threat_intel",
            "integer",
            "not_a_number",
        );
        let display = format!("{err}");
        assert!(
            display.starts_with("E-INFUSE-014:"),
            "MED-001: TypeCoercionFailed Display must start with 'E-INFUSE-014:'. Got: {display}"
        );
        assert!(
            display.contains("score_field"),
            "MED-001: TypeCoercionFailed Display must contain field_name 'score_field'. Got: {display}"
        );
        assert!(
            display.contains("threat_intel"),
            "MED-001: TypeCoercionFailed Display must contain infusion_id 'threat_intel'. Got: {display}"
        );
        assert!(
            display.contains("integer"),
            "MED-001: TypeCoercionFailed Display must contain declared_type 'integer'. Got: {display}"
        );
        assert!(
            display.contains("not_a_number"),
            "MED-001: TypeCoercionFailed Display must contain truncated_value 'not_a_number'. Got: {display}"
        );
        assert!(
            display.contains("row produces NULL"),
            "MED-001: TypeCoercionFailed Display must contain 'row produces NULL'. Got: {display}"
        );
    }

    /// RGT-010 (ADR-051 D4): JSON-list input to non-json typed UDF returns None (E-INFUSE-014).
    ///
    /// ADR-051 D4 Scalar-Input rule: if the projected value begins with `[` (JSON array)
    /// and `output_type != "json"`, coerce_to_typed returns None.
    /// ENRICH-1 list-dispatch is RETAINED only for `output_type = "json"`.
    #[test]
    fn test_json_list_input_to_typed_output_udf_produces_null_e_infuse_014() {
        use datafusion::arrow::datatypes::DataType;

        let (_, src) = CountingSource::new_returning("[\"hash1\",\"hash2\"]");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_score_list",
            "ip",
            "integer",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        // ADR-051 D4: "[...]" starts with '[' → JSON-list input to non-json (integer) UDF
        // → must return None (E-INFUSE-014 NULL; ENRICH-1 list-dispatch disabled for Int64).
        // coerce_to_typed is implemented; assertion verifies E-INFUSE-014 failure returns None.
        let result = udf.coerce_to_typed("[\"hash1\",\"hash2\"]", &DataType::Int64, "threat_score");
        assert!(
            result.is_none(),
            "ADR-051 D4 RGT-010 E-INFUSE-014: JSON-list input '[...]' to integer UDF must \
             return None (Scalar-Input rule). Got: {:?}",
            result
        );
    }

    // ── LOW-001 + OBS-002 (LOCAL adversary pass-3): EC-002 / EC-006 assertions ────────────
    // These tests are load-bearing: they prevent future maintainers from accidentally
    // "fixing" the Int64 fallback to convert float strings to integers.

    /// EC-002 (ADR-051 D2): float-valued string into Int64 field yields NULL + E-INFUSE-014.
    ///
    /// serde_json parses "95.7" as an f64-backed Number; n.as_i64() returns None for
    /// f64-backed Numbers, so "95.7" → Int64 correctly produces NULL.
    ///
    /// The prior comment in the Int64 branch said "handles '42.0' where the plugin returns
    /// a floating-point representation of an integer" — this was BACKWARDS. The comment has
    /// been corrected; this test is the load-bearing assertion that guards against reverting
    /// the behavior to silently truncate floats into integers (a data corruption regression).
    #[test]
    fn test_ec002_float_string_to_integer_yields_null() {
        use datafusion::arrow::datatypes::DataType;

        let (_, src) = CountingSource::new_returning("95.7");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_score",
            "ip",
            "integer",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        // EC-002: "95.7" → i64::from_str fails (decimal point present).
        // serde_json::from_str("95.7") → Number backed as f64.
        // n.as_i64() returns None for f64-backed serde_json::Number → NULL + E-INFUSE-014.
        // "95.7" does NOT coerce to 95; float strings into integer fields produce NULL.
        let result = udf.coerce_to_typed("95.7", &DataType::Int64, "threat_score");
        assert!(
            result.is_none(),
            "EC-002 (ADR-051 D2 LOW-001): coerce_to_typed('95.7', Int64, 'threat_score') must \
             return None. Float-valued JSON numbers yield NULL + E-INFUSE-014; they do NOT \
             coerce to the nearest integer (that would be silent data corruption). Got: {:?}",
            result
        );
    }

    /// EC-006 (ADR-051 D2): empty string into any typed field yields NULL + E-INFUSE-014.
    ///
    /// An empty "" input means the source returned no value or the upstream column was blank.
    /// Both i64::from_str("") and serde_json::from_str("") fail → None → NULL.
    #[test]
    fn test_ec006_empty_input_yields_null() {
        use datafusion::arrow::datatypes::DataType;

        let (_, src) = CountingSource::new_returning("");
        let descriptor = InfusionUdfDescriptor::new(
            "threat_score",
            "ip",
            "integer",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        // EC-006: "" → i64::from_str("") fails → serde_json::from_str("") fails →
        // None → NULL + E-INFUSE-014. Empty input must never produce a default value.
        let result = udf.coerce_to_typed("", &DataType::Int64, "threat_score");
        assert!(
            result.is_none(),
            "EC-006 (ADR-051 D2 OBS-002): coerce_to_typed('', Int64, 'threat_score') must \
             return None. Empty input yields NULL + E-INFUSE-014 (no default substitution). \
             Got: {:?}",
            result
        );
    }

    // ── RGT-023 (ADV-P11-OBS-001 fix-burst-9): no double-encoding on json output ──────────

    /// RGT-023 (ADV-P11-OBS-001): `threat_sources` with `output_type = "json"`,
    /// `source_column = "threat_sources"`, and scalar input (`iocs_value_first`) produces
    /// single-encoded JSON array output, NOT double-encoded.
    ///
    /// # Defect context (ADV-P11-OBS-001)
    ///
    /// When `input_field = "iocs_value"` (JSON-list column), ENRICH-1 list-dispatch fires:
    /// (a) UDF receives input `["1.2.3.4"]` (JSON list from the iocs_value column).
    /// (b) ENRICH-1 fires (starts with `[`), calls `enrich_one_scalar("1.2.3.4")` per element.
    /// (c) Source returns `{"threat_sources": ["greynoise","abuseipdb"]}` for "1.2.3.4".
    /// (d) `project_value` extracts the Array via `other.to_string()` → String `["greynoise","abuseipdb"]`.
    /// (e) `serde_json::to_string(&list_results)` wraps it → `["[\"greynoise\",\"abuseipdb\"]"]`.
    /// RESULT: outer JSON array wrapping a JSON-encoded array string — double-encoding (Failure A).
    ///
    /// # Fix path (this test validates)
    ///
    /// With `input_field = "iocs_value_first"` (scalar), the UDF receives `"1.2.3.4"` (NOT `["1.2.3.4"]`):
    /// (a) ENRICH-1 does NOT fire (input does not start with `[`).
    /// (b) `enrich_one_scalar("1.2.3.4")` → source returns full response object.
    /// (c) `project_value("threat_sources")` extracts the Array via `other.to_string()` →
    ///     String `["greynoise","abuseipdb"]` stored directly as Utf8 cell value.
    /// RESULT: `["greynoise","abuseipdb"]` — single-encoded JSON array (plain string elements).
    ///
    /// # SID-1 rationale
    ///
    /// The TOML fix (`input_field = "iocs_value_first"` for threat_sources) is the implementer's
    /// task. This test validates the UDF correctly handles the fixed scalar path. It is a unit
    /// test in-process at the dependency boundary — no external service, no DTU clone required.
    ///
    /// Traces to: AC-009, BC-2.19.001 v2.2 INV-ENRICH-TYPED-001, ADV-P11-OBS-001.
    #[tokio::test]
    async fn test_threat_sources_json_output_no_double_encoding() {
        use datafusion::arrow::array::{Array, StringArray};
        use datafusion::arrow::datatypes::{DataType, Field, Schema};
        use datafusion::arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;

        let ctx = SessionContext::new();

        // Mock source: returns the full ThreatIntel response object for any input.
        // Confirmed shape from prism-dtu-threatintel/src/routes/lookup.rs Malicious fixture
        // (2026-06-17): threat_sources is Vec<String> (JSON array), NOT a plain string.
        // Direct CountingSource construction — `return_value` holds the JSON object.
        let src: Arc<dyn InfusionSource> = Arc::new(CountingSource {
            call_count: Arc::new(AtomicUsize::new(0)),
            return_value: Some(serde_json::json!({
                "lookup_value": "1.2.3.4",
                "threat_score": 95,
                "threat_is_known_malicious": true,
                "threat_sources": ["greynoise", "abuseipdb"]
            })),
        });

        // Descriptor: output_type = "json", source_column = "threat_sources",
        // input_field = "iocs_value_first" — the CORRECT threatintel.infusion.toml config
        // after the ADV-P11-OBS-001 fix.
        let descriptor = InfusionUdfDescriptor::new(
            "threat_sources", // name — UDF is named after the enrichment field
            "ip",             // input_type — enriched by IP address lookup
            "json",           // output_type — Vec<String> array serialized as JSON
            "threatintel",    // infusion_id
            src,
            Some("threat_sources".to_string()), // source_column — project this field from response
            super::DEFAULT_CACHE_TTL_SECS,      // cache_ttl_secs
            "iocs_value_first",                 // input_field — scalar companion, NOT iocs_value
        );
        register_infusion_udfs(&ctx, vec![descriptor])
            .expect("RGT-023: UDF registration must succeed");

        // Table: one row with scalar input "1.2.3.4".
        // This represents the value from the `iocs_value_first` column in the sensor data —
        // the first extracted IOC scalar (not the JSON list from `iocs_value`).
        let schema = Arc::new(Schema::new(vec![Field::new(
            "iocs_value_first",
            DataType::Utf8,
            false,
        )]));
        let arr = StringArray::from(vec!["1.2.3.4"]);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("RecordBatch construction must succeed");
        let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
            .expect("MemTable construction must succeed");
        ctx.register_table("enrichment_test", Arc::new(table))
            .expect("register_table must succeed");

        // Execute the enrichment query: scalar `iocs_value_first` column feeds the UDF.
        // Simulates: `SELECT threat_sources(iocs_value_first) FROM enrichment_test`
        // which is the post-fix T13 canonical query pattern (AC-009).
        let df = ctx
            .sql("SELECT threat_sources(iocs_value_first) AS ts_enriched FROM enrichment_test")
            .await
            .expect("RGT-023: SQL must parse and plan");
        let batches = df.collect().await.expect("RGT-023: query must execute");

        assert_eq!(
            batches.len(),
            1,
            "RGT-023: must have exactly 1 output batch"
        );
        let batch = &batches[0];
        assert_eq!(
            batch.num_rows(),
            1,
            "RGT-023: must have exactly 1 output row"
        );

        let output_col = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("RGT-023: threat_sources output must be StringArray (output_type=json → Utf8)");

        assert!(
            !output_col.is_null(0),
            "RGT-023: threat_sources enrichment for scalar input '1.2.3.4' must return \
             a non-NULL value. Got NULL."
        );

        let cell_value = output_col.value(0);

        // LOAD-BEARING assertion (TD-VSDD-059): exact string content check.
        // Expected: `["greynoise","abuseipdb"]` — single-encoded JSON array.
        // Forbidden: `["[\"greynoise\",\"abuseipdb\"]"]` — double-encoded (ENRICH-1 Failure A).
        // The double-encoded form would arise if ENRICH-1 list-dispatch fired, wrapped the
        // projected array-string in `serde_json::to_string(&list_results)`.
        assert_eq!(
            cell_value, r#"["greynoise","abuseipdb"]"#,
            "RGT-023 ADV-P11-OBS-001: threat_sources output must be the single-encoded JSON \
             array '[\"greynoise\",\"abuseipdb\"]' (plain string elements). \
             Double-encoded form '[\"[\\\"greynoise\\\",\\\"abuseipdb\\\"]\"]' would indicate \
             ENRICH-1 list-dispatch fired on scalar input (Failure A — prevented by using \
             iocs_value_first instead of iocs_value). Got: {:?}",
            cell_value
        );

        // Verify parsed elements are plain strings, not JSON-encoded strings.
        // This catches any double-encoding that produces elements like `["greynoise","abuseipdb"]`
        // as a single string element rather than two separate string elements.
        let parsed: Vec<String> = serde_json::from_str(cell_value).unwrap_or_else(|e| {
            panic!(
                "RGT-023: threat_sources cell value must be valid JSON array of strings. \
                 Parse error: {e}. Got cell value: {:?}",
                cell_value
            )
        });
        assert_eq!(
            parsed,
            vec!["greynoise", "abuseipdb"],
            "RGT-023 ADV-P11-OBS-001: parsed threat_sources elements must be plain strings \
             [\"greynoise\", \"abuseipdb\"], NOT JSON-encoded strings. \
             Double-encoded elements would look like '[\"greynoise\",\"abuseipdb\"]' as a \
             single element (Failure A). Got: {:?}",
            parsed
        );
        assert_eq!(
            parsed.len(),
            2,
            "RGT-023: parsed threat_sources must have exactly 2 elements (greynoise, abuseipdb). \
             Got: {:?}",
            parsed
        );
    }

    // ── SEC-001(b) (PR-216 fix-burst-13): TypeCoercionFailed control-char sanitization ─────

    /// SEC-001 (CWE-117, PR-216) / AC-005 — TypeCoercionFailed metadata fields stripped.
    ///
    /// `field_name`, `infusion_id`, and `declared_type` on `TypeCoercionFailed` originate from
    /// operator-supplied TOML specs and are attacker-influenceable. Control characters (0x00–0x1F,
    /// 0x7F) in these values must be stripped before `TypeCoercionFailed` is constructed so that
    /// the rendered `E-INFUSE-014` Display message contains no control chars.
    ///
    /// This prevents CWE-117 log injection and LLM prompt injection into agent-consumed structured
    /// logs (AD-017 extension, error-taxonomy v2.17 SEC-001 Rendering Note).
    ///
    /// RED (fix-burst-13): `new_type_coercion_failed` currently passes all three fields through
    /// verbatim (`field_name.into()`, `infusion_id.into()`, `declared_type.into()`) without any
    /// control-char stripping. This test FAILS against current code: the rendered Display will
    /// contain 0x01, 0x02, and 0x03 at the positions where control chars were interpolated.
    ///
    /// Implementer action (error-taxonomy v2.17 SEC-001 Rendering Note):
    /// Add a `sanitize_for_log(s: &str) -> String` helper
    ///   (`s.chars().filter(|c| !c.is_ascii_control()).collect()`)
    /// and call it on `field_name`, `infusion_id`, and `declared_type` inside
    /// `new_type_coercion_failed` before storing them in the struct.
    #[test]
    fn test_sec001_type_coercion_failed_ctrl_chars_stripped_from_metadata_fields() {
        use prism_core::InfusionError;

        // Construct via the public constructor with control chars in all three metadata fields.
        let err = InfusionError::new_type_coercion_failed(
            "field\x01name",  // field_name: 0x01 (SOH) — must be stripped
            "infusion\x02id", // infusion_id: 0x02 (STX) — must be stripped
            "integer\x03",    // declared_type: 0x03 (ETX) — must be stripped
            "not_a_number",
        );
        let display = format!("{}", err);

        // Assert NO ASCII control chars (0x00–0x1F, 0x7F) in the rendered Display.
        // RED: current code stores the raw bytes → Display contains 0x01/0x02/0x03.
        for (i, c) in display.char_indices() {
            assert!(
                !c.is_ascii_control(),
                "SEC-001 E-INFUSE-014 CWE-117: TypeCoercionFailed Display must NOT contain \
                 ASCII control character U+{:04X} at byte position {} in the rendered message.\n\
                 Control chars in field_name/infusion_id/declared_type must be stripped before \
                 TypeCoercionFailed is constructed (new_type_coercion_failed).\n\
                 Got Display: {:?}",
                c as u32,
                i,
                display
            );
        }
    }

    /// SEC-001 (CWE-117, PR-216) / AC-005 — TypeCoercionFailed truncated_value stripping
    /// AFTER the 50-char truncation step (not before).
    ///
    /// Order semantics: truncation removes content (chars beyond 50); stripping removes control
    /// chars. The spec requires: truncate first (50-char cap per AD-017), then strip control chars.
    /// A control char that falls within the 50-char window must be stripped from the stored value.
    ///
    /// Test fixture: value = 49 × 'a' + '\x01' (total 50 chars).
    /// After `chars().take(50)`: truncated_value = "aaa...a\x01" (the \x01 is char 50).
    /// After sanitization: truncated_value = "aaa...a" (no control chars).
    /// The rendered Display must NOT contain '\x01'.
    ///
    /// RED (fix-burst-13): `new_type_coercion_failed` stores
    ///   `value.chars().take(50).collect()` WITHOUT stripping — the \x01 survives in
    ///   `truncated_value`, and the rendered Display contains it.
    ///
    /// Implementer action: apply sanitize_for_log to `truncated_value` AFTER the
    /// `chars().take(50)` truncation.
    #[test]
    fn test_sec001_type_coercion_failed_ctrl_chars_stripped_from_truncated_value_after_truncation()
    {
        use prism_core::InfusionError;

        // 49 'a' chars + '\x01': the control char is exactly at the 50-char boundary.
        let value: String = "a".repeat(49) + "\x01";
        assert_eq!(
            value.chars().count(),
            50,
            "test fixture: value must be exactly 50 chars so \\x01 is the final char after take(50)"
        );

        let err = InfusionError::new_type_coercion_failed(
            "threat_score",
            "threat_intel",
            "integer",
            &value,
        );
        let display = format!("{}", err);

        // The control char at position 50 (post-truncation) must be stripped.
        // RED: current code doesn't strip → display contains 0x01.
        assert!(
            !display.contains('\x01'),
            "SEC-001 E-INFUSE-014 CWE-117: truncated_value control char \\x01 must be \
             stripped AFTER the 50-char truncation. The rendered Display must NOT contain \
             U+0001. Stripping must happen AFTER truncation (not before — ordering matters \
             for correct 50-char cap semantics per AD-017). Got: {:?}",
            display
        );

        // Belt-and-suspenders: no ASCII control chars at all.
        for (i, c) in display.char_indices() {
            assert!(
                !c.is_ascii_control(),
                "SEC-001: rendered Display must NOT contain any ASCII control char; \
                 found U+{:04X} at byte position {}. Got: {:?}",
                c as u32,
                i,
                display
            );
        }
    }

    // ── SEC-002 (PR-216 fix-burst-13): boolean coercion size guard regression ────────────

    /// SEC-002 (CWE-770, PR-216) — REGRESSION GUARD for boolean coercion size guard.
    ///
    /// The boolean coercion branch calls `value.to_lowercase()` before the set-membership check.
    /// `to_lowercase()` is an O(n) heap allocation. For an adversarially large input (> 1024 bytes),
    /// this allocates unnecessarily. The implementer MUST add:
    ///   `if s.len() > 1024 { /* emit E-INFUSE-014 */ return None }`
    /// BEFORE calling `to_lowercase()`, preventing CWE-770 unbounded allocation.
    ///
    /// This test is a REGRESSION GUARD: it PASSES against current code (an oversized string does
    /// not match any of the boolean set members, so `coerce_to_typed` already returns `None`)
    /// AND against fixed code (the size guard returns `None` for the same reason — the NULL
    /// outcome is unchanged). The guard exists to ensure the implementer's size gate does NOT
    /// accidentally change the observable result from `None` to `Some(...)`.
    ///
    /// If this test fails, the implementer's size guard has introduced a regression where
    /// oversized inputs no longer produce NULL.
    ///
    /// NOTE: This is intentionally marked as a REGRESSION GUARD (passes pre-fix). It is NOT
    /// a Red Gate test for SEC-002 itself; the SEC-002 fix is a bounded-cost optimization that
    /// does NOT change the NULL outcome. The test documents the invariant.
    #[test]
    fn test_sec002_boolean_coercion_oversized_input_yields_null_regression_guard() {
        use datafusion::arrow::datatypes::DataType;

        // > 1024 bytes — triggers the size guard added by the implementer.
        // Against current code (no size guard): to_lowercase() allocates a huge buffer,
        // then the trimmed result doesn't match any bool literal → returns None.
        // Against fixed code (with size guard): returns None immediately.
        // Either way: None. This is the regression guard.
        let oversized_value = "a".repeat(1025);

        let (_, src) = CountingSource::new_returning("true"); // source value irrelevant
        let descriptor = InfusionUdfDescriptor::new(
            "threat_is_malicious",
            "ip",
            "boolean",
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        // REGRESSION GUARD: both current and fixed code must return None for an oversized input.
        // Fixed code adds an early-return None at > 1024 bytes (before to_lowercase allocation).
        // This test ensures the NULL outcome is preserved.
        let result =
            udf.coerce_to_typed(&oversized_value, &DataType::Boolean, "threat_is_malicious");
        assert!(
            result.is_none(),
            "SEC-002 CWE-770 REGRESSION GUARD: coerce_to_typed(1025-byte input, Boolean, ...) \
             must return None (NULL). The size guard must preserve the NULL outcome — same \
             observable result as any unrecognized boolean value. Got: {:?}",
            result
        );
    }

    // ── NEW-SEC-001-R (PR-216 fix-burst-14): structured tracing field control-char sanitization ──

    /// NEW-SEC-001-R (CWE-117, PR-216 fix-burst-14) RED GATE — structured tracing fields in
    /// `warn_coercion_failed` pass raw values, not sanitized ones.
    ///
    /// `warn_coercion_failed` emits `tracing::warn!` with named structured fields:
    ///
    /// ```text
    /// tracing::warn!(
    ///     event_type = "infusion.coercion_failed",
    ///     field_name        = %field_name,                              // RAW — no sanitization
    ///     infusion_id       = %self.descriptor.infusion_id,             // RAW — no sanitization
    ///     declared_type     = %self.descriptor.output_type,             // RAW — no sanitization
    ///     truncated_value   = %value.chars().take(50)...,               // RAW — no sanitization
    ///     "{}", err                                                      // sanitized Display
    /// )
    /// ```
    ///
    /// Fix-burst-13 (SEC-001) sanitized the `E-INFUSE-014` Display message (via
    /// `InfusionError::new_type_coercion_failed`), but the STRUCTURED FIELDS still receive raw
    /// values via `%`. JSON log consumers (e.g., Vector, Loki, SIEM ingestors) parse named fields
    /// directly from the serialized tracing event — a control char in `field_name` or `infusion_id`
    /// reaches the JSON field value verbatim, enabling CWE-117 log injection and LLM prompt
    /// injection into agent-consumed structured logs (AD-017 extension).
    ///
    /// **RED GATE (current code):** `%field_name` with value `"threat_score\x01injected"` emits
    /// U+0001 (SOH) into the captured tracing output. `logs_contain("\x01")` returns `true`.
    /// The assertion `!logs_contain("\x01")` therefore FAILS against current code.
    ///
    /// **After fix:** sanitize `field_name`/`infusion_id`/`declared_type`/`truncated_value` with
    /// `sanitize_for_log()` (strip chars where `c.is_ascii_control()`) inside `warn_coercion_failed`
    /// BEFORE passing them as named tracing fields. The U+0001 is removed; `logs_contain("\x01")`
    /// returns `false`; the assertion PASSES.
    ///
    /// Traces to: NEW-SEC-001-R (PR-216 re-review), CWE-117, AD-017, SAP-1.
    #[test]
    #[tracing_test::traced_test]
    fn test_new_sec001_r_warn_coercion_failed_structured_fields_no_raw_control_chars() {
        use datafusion::arrow::datatypes::DataType;

        // field_name contains SOH (U+0001, ASCII 1) — canonical CWE-117 injection vector.
        // SOH never appears in normal formatted log output, so its presence is unambiguous.
        // U+0001 is distinct from the U+0002/U+0003 used in fix-burst-13 SEC-001 tests.
        let ctrl_field_name = "threat_score\x01injected";

        let (_, src) = CountingSource::new_returning("ignored");
        let descriptor = InfusionUdfDescriptor::new(
            "sec001r_udf",
            "ip",
            "integer", // output_type = integer; "not-an-integer" → coercion fails → warn fires
            "threat_intel",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        // Drive warn_coercion_failed: "not-an-integer" cannot be coerced to i64.
        let result = udf.coerce_to_typed("not-an-integer", &DataType::Int64, ctrl_field_name);
        assert!(
            result.is_none(),
            "NEW-SEC-001-R: coerce_to_typed('not-an-integer', Int64, ctrl_field_name) \
             must return None (prerequisite for the tracing assertion)"
        );

        // Confirm the coercion_failed event was emitted at all.
        assert!(
            logs_contain("infusion.coercion_failed"),
            "NEW-SEC-001-R: warn_coercion_failed must emit \
             event_type=infusion.coercion_failed for an uncoercible integer input"
        );

        // NEW-SEC-001-R LOAD-BEARING ASSERTION: no raw ASCII control char U+0001 in the
        // captured tracing output.
        //
        // Current code:   `field_name = %ctrl_field_name` passes raw bytes →
        //                 "threat_score\x01injected" IS in the structured field output →
        //                 logs_contain("\x01") == true → !logs_contain("\x01") == false → FAIL
        //
        // After fix:      sanitize_for_log strips U+0001 from field_name before tracing::warn! →
        //                 "\x01" is absent from the output →
        //                 logs_contain("\x01") == false → !logs_contain("\x01") == true → PASS
        assert!(
            !logs_contain("threat_score\x01injected"),
            "NEW-SEC-001-R CWE-117 RED GATE (precise): raw value 'threat_score\\x01injected' \
             found in captured tracing output. The structured field `field_name` in \
             infusion.coercion_failed emits the raw field_name argument without control-char \
             sanitization. JSON log consumers receive U+0001 in the field value. \
             FIX: call sanitize_for_log(field_name) (strip .is_ascii_control() chars) inside \
             warn_coercion_failed before passing to the `field_name = %...` tracing field."
        );
        assert!(
            !logs_contain("\x01"),
            "NEW-SEC-001-R CWE-117 RED GATE (general): raw ASCII control char U+0001 found \
             anywhere in the captured tracing output. This fires because \
             `field_name = %ctrl_field_name` emits the SOH byte from the raw field name. \
             After fix (sanitize_for_log in warn_coercion_failed), this assertion must pass."
        );
    }

    // ── NEW-CR-005 (PR-216 fix-burst-14): oversized boolean emits infusion.coercion_failed ──

    /// NEW-CR-005 (SAP-1, Standing Rule 3 §2, PR-216 fix-burst-14) — regression guard verifying
    /// that oversized boolean input emits `infusion.coercion_failed` (E-INFUSE-014 semantics).
    ///
    /// The SEC-002 boolean size guard (CWE-770) MUST call `warn_coercion_failed` before returning
    /// `None` for inputs > 1024 bytes. A bare `return None` without event emission is a silent
    /// failure per Standing Rule 3 §2 and a SAP-1 violation (BC-2.16.002 catalog must be complete).
    ///
    /// **REGRESSION GUARD NOTE:** Fix-burst-13 implementation (4bb0cad5) already added the
    /// `warn_coercion_failed` call to the boolean size guard branch. This test PASSES against the
    /// current HEAD (not a red gate, due to TDD ordering: implementation preceded this test).
    /// It serves as a load-bearing regression guard: if a future change accidentally reverts the
    /// boolean size guard to a bare `return None` (no event emission), this test will fail,
    /// surfacing the silent-failure regression immediately.
    ///
    /// The existing `test_sec002_boolean_coercion_oversized_input_yields_null_regression_guard`
    /// asserts the `None` outcome only. This test adds the event emission assertion.
    ///
    /// Traces to: NEW-CR-005 (PR-216 re-review), SAP-1, Standing Rule 3 §2, E-INFUSE-014.
    #[test]
    #[tracing_test::traced_test]
    fn test_new_cr005_oversized_boolean_input_emits_coercion_failed_event_and_returns_null() {
        use datafusion::arrow::datatypes::DataType;

        // > 1024 bytes — triggers the SEC-002 size guard in the boolean coercion branch.
        let oversized_value = "a".repeat(1025);

        let (_, src) = CountingSource::new_returning("true"); // source value is irrelevant
        let descriptor = InfusionUdfDescriptor::new(
            "cr005_udf",
            "ip",
            "boolean",
            "threat_intel_infusion",
            src,
            None,
            super::DEFAULT_CACHE_TTL_SECS,
            "",
        );
        let udf = super::InfusionAsyncUdf::new(descriptor);

        let result =
            udf.coerce_to_typed(&oversized_value, &DataType::Boolean, "threat_is_malicious");

        // REGRESSION GUARD assertion 1: return value must be None (E-INFUSE-014 NULL sentinel).
        assert!(
            result.is_none(),
            "NEW-CR-005: coerce_to_typed(1025-byte input, Boolean, 'threat_is_malicious') \
             must return None. The SEC-002 size guard must short-circuit to None."
        );

        // REGRESSION GUARD assertion 2: infusion.coercion_failed event MUST be emitted.
        //
        // Spec (BC-2.16.002 event catalog, E-INFUSE-014): oversized boolean input is a coercion
        // failure — it must emit infusion.coercion_failed exactly like any other uncoercible value.
        // A bare `return None` without event emission is a silent failure (Standing Rule 3 §2)
        // and a SAP-1 violation.
        //
        // If this assertion fails, the boolean size guard was changed to skip warn_coercion_failed,
        // silently swallowing the E-INFUSE-014 event for oversized inputs.
        assert!(
            logs_contain("infusion.coercion_failed"),
            "NEW-CR-005 SAP-1 REGRESSION GUARD: oversized (>1024 byte) boolean input MUST emit \
             event_type=infusion.coercion_failed. A silent `return None` without event emission \
             violates Standing Rule 3 §2 and the BC-2.16.002 event catalog. \
             If this fails, the SEC-002 boolean size guard was changed to a bare `return None`."
        );

        // Confirm the emitted event references the correct field_name (not a spurious emission).
        assert!(
            logs_contain("threat_is_malicious"),
            "NEW-CR-005: field_name 'threat_is_malicious' must appear in the \
             infusion.coercion_failed event emitted for the oversized boolean input."
        );
    }
}
