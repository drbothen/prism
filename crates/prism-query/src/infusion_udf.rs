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
//! # Stub status (S-DEMO-ENRICHMENT-PIVOT-001)
//! `InfusionAsyncUdf::invoke_async_with_args` body is `todo!()` — Red Gate stub.
//! `register_infusion_udfs` is implemented — it registers UDFs calling the todo!() impl.
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
    /// Calls `self.descriptor.source.enrich_single(input, input_type)` for each
    /// row in the input batch and returns the enriched values as a `ColumnarValue`.
    ///
    /// # S-DEMO-ENRICHMENT-PIVOT-001 Red Gate stub
    /// Body is `todo!()` — implementation in this story's TDD green phase.
    async fn invoke_async_with_args(
        &self,
        _args: ScalarFunctionArgs,
    ) -> DataFusionResult<ColumnarValue> {
        todo!(
            "InfusionAsyncUdf::invoke_async_with_args — S-DEMO-ENRICHMENT-PIVOT-001 Red Gate: \
             implement by calling self.descriptor.source.enrich_single for each row"
        )
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
    for descriptor in descriptors {
        let udf_impl = InfusionAsyncUdf::new(descriptor);
        let async_udf = AsyncScalarUDF::new(Arc::new(udf_impl));
        ctx.register_udf(async_udf.into_scalar_udf());
    }
    Ok(())
}
