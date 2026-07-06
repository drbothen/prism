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
use prism_spec_engine::{
    InfusionLruCache, InfusionTier3Cache, InfusionUdfDescriptor, QueryScopedInfusionCache,
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
    /// DataFusion function signature (input: Utf8, output: Utf8 — simplified for stub).
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
        // Simplified: always returns Utf8 for the current implementation.
        // Full typed mapping of `descriptor.output_type` → Arrow DataType is deferred
        // to S-1.14-REDO (DRIFT-PIVOT-UDF-OUTPUT-TYPE-001); not in PIVOT-001 scope.
        // ADR-052: sensor datetime columns → Timestamp(Microsecond, Some("UTC")) (ADR-052).
        // ADR-051 (not yet implemented) will add a per-output_type branch here to bring
        // enrichment datetime fields to the same type.
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

        // Tier 1: per-call dedup cache (fresh per invoke_async_with_args invocation).
        // Ensures 500 rows with 30 unique IPs → 30 source calls, not 500 (AC-2 / INV-INFUSE-002).
        let mut tier1 = QueryScopedInfusionCache::new();

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
            // If the input starts with '[' and parses as Vec<String>, this is a wildcard
            // column value (e.g., from `$.iocs[*].value` → `["hash1","hash2"]`).
            // Enrich each element individually and return a JSON-list of results.
            // Elements that enrich to None are omitted from the output list.
            // Scalar path (no leading '[' or failed parse) is unchanged — backward compat.
            if input_str.starts_with('[') {
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
}
