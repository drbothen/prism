//! DataFusion `WriteCapableTableProvider` registration for write endpoints.
//!
//! Registers `WriteTableDescriptor`s exported by `prism-spec-engine` (S-1.13) as
//! DataFusion write-capable `TableProvider` implementations. All DataFusion
//! catalog registration happens here in S-3.07.
//!
//! `WriteCapableTableProvider` routes `insert_into()`, `update()`, and
//! `delete_from()` calls through `WriteExecutor::execute()` — enforcing the full
//! six-phase safety pipeline for SQL DML, identical to pipe-mode write calls
//! (BC-2.04.007, story AC-6).
//!
//! # DataFusion 53.1 API Note (Dev Notes)
//! The `TableProvider::insert_into`, `InsertOp` enum, and UPDATE/DELETE trait
//! surface in DataFusion 53.x are flagged as TDD-time verification gates in the
//! story spec. If the confirmed API differs from the shapes assumed here, a
//! W3-FIX-* story must be filed — this story's scope does NOT expand.
//!
//! Current stub uses `insert_into` with `InsertOp` as documented in DataFusion
//! 53.1 source. UPDATE and DELETE stubs use `todo!()` pending API confirmation.
//!
//! # Architecture Compliance
//! - `WriteCapableTableProvider::insert_into/update/delete_from` MUST all route
//!   through `WriteExecutor::execute()` — no shortcuts that bypass the safety
//!   pipeline (story §Architecture Compliance Rule 5).
//! - `WriteResult.failed_count > 0 && succeeded_count > 0`: return `E-QUERY-025`
//!   (partial failure) with the full `WriteResult`.
//! - prism-spec-engine MUST NOT be imported in DataFusion TableProvider methods —
//!   all spec resolution happens via `WriteEndpointSpec` passed at construction.
//!
//! Story: S-3.07 | BCs: BC-2.04.007, BC-2.04.008

// Stub module: all non-trivial bodies are todo!() pending implementation.
#![allow(dead_code, unused_variables)]

use std::any::Any;
use std::sync::Arc;

use arrow::datatypes::SchemaRef;
use async_trait::async_trait;
use datafusion::catalog::Session;
use datafusion::datasource::TableProvider;
use datafusion::error::DataFusionError;
use datafusion::logical_expr::TableType;
use datafusion::physical_plan::ExecutionPlan;
use datafusion::prelude::Expr;
use prism_core::PrismError;
use prism_spec_engine::write_endpoint::{WriteEndpointSpec, WriteTableDescriptor};

use crate::write_pipeline::WriteExecutor;

// ---------------------------------------------------------------------------
// WriteCapableTableProvider
// ---------------------------------------------------------------------------

/// DataFusion `TableProvider` implementation for write endpoints.
///
/// Created from a `WriteTableDescriptor` (exported by S-1.13 / prism-spec-engine).
/// Registered in the DataFusion catalog by `register_write_tables()` below.
///
/// Routes SQL DML through `WriteExecutor::execute()` to enforce the identical
/// six-phase safety pipeline as pipe-mode writes.
pub struct WriteCapableTableProvider {
    /// The write table descriptor from prism-spec-engine.
    pub(crate) descriptor: WriteTableDescriptor,
    /// The endpoint spec for this write table (resolved at registration time).
    pub(crate) endpoint_spec: WriteEndpointSpec,
    /// The write executor (shared across all table providers for this engine).
    pub(crate) executor: Arc<WriteExecutor>,
    /// Arrow schema for the write result row.
    ///
    /// A single-row schema encoding the `WriteResult` fields for DataFusion
    /// compatibility (story §Task 9).
    pub(crate) schema: SchemaRef,
}

// Manual Debug impl: WriteExecutor contains non-Debug fields (FeatureFlagEvaluator, etc.).
// Only the descriptor fields are printed — executor is shown as opaque.
impl std::fmt::Debug for WriteCapableTableProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteCapableTableProvider")
            .field("sql_table", &self.descriptor.sql_table)
            .field("verb", &self.descriptor.verb)
            .field("sensor", &self.descriptor.sensor)
            .finish_non_exhaustive()
    }
}

impl WriteCapableTableProvider {
    /// Construct a `WriteCapableTableProvider` from a descriptor and executor.
    pub fn new(
        descriptor: WriteTableDescriptor,
        endpoint_spec: WriteEndpointSpec,
        executor: Arc<WriteExecutor>,
    ) -> Self {
        // GREEN-BY-DESIGN self-check:
        // "If I include this real implementation, will the test for this function
        //  pass trivially without any implementer work?"
        // Answer: Yes — building the Arrow schema requires non-trivial field
        // construction logic. Replaced with todo!().
        todo!("S-3.07 — WriteCapableTableProvider::new: schema construction")
    }
}

#[async_trait]
impl TableProvider for WriteCapableTableProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn table_type(&self) -> TableType {
        // WIRING-EXEMPT: TableProvider::table_type() requires a concrete return value.
        // Write-only tables use the Base type (the only applicable DataFusion type).
        // No domain logic; pure constant return.
        TableType::Base
    }

    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
        // Write-only tables cannot be scanned (read from).
        // Return a DataFusion error rather than todo!() to avoid a panic
        // in the DataFusion planner when it probes all table types.
        // This is WIRING-EXEMPT: the error message is determined by the
        // TableProvider contract, not domain logic.
        Err(DataFusionError::Plan(format!(
            "table '{}' is write-only and cannot be queried via SELECT",
            self.descriptor.sql_table
        )))
    }

    async fn insert_into(
        &self,
        _state: &dyn Session,
        _input: Arc<dyn ExecutionPlan>,
        _insert_op: datafusion::logical_expr::dml::InsertOp,
    ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
        todo!("S-3.07 — WriteCapableTableProvider::insert_into: route through WriteExecutor")
    }

    async fn delete_from(
        &self,
        _state: &dyn Session,
        _filters: Vec<datafusion::prelude::Expr>,
    ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
        todo!("S-3.07 — WriteCapableTableProvider::delete_from: route through WriteExecutor")
    }

    async fn update(
        &self,
        _state: &dyn Session,
        _assignments: Vec<(String, datafusion::prelude::Expr)>,
        _filters: Vec<datafusion::prelude::Expr>,
    ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
        todo!("S-3.07 — WriteCapableTableProvider::update: route through WriteExecutor")
    }
}

// ---------------------------------------------------------------------------
// Catalog registration
// ---------------------------------------------------------------------------

/// Register all write endpoints from `descriptors` into the DataFusion catalog.
///
/// Called once at startup after `WriteEndpointRegistry::table_descriptors()`
/// returns the full set of exported descriptors from prism-spec-engine.
///
/// Each descriptor is wrapped in a `WriteCapableTableProvider` and registered
/// in the `SessionContext` catalog under its `sql_table` name.
pub async fn register_write_tables(
    descriptors: Vec<WriteTableDescriptor>,
    executor: Arc<WriteExecutor>,
    session: &datafusion::execution::context::SessionContext,
) -> Result<(), PrismError> {
    todo!("S-3.07 — register_write_tables: catalog registration for all write endpoints")
}
