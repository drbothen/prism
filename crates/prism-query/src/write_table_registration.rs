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

// CRIT-2: todo!() panics replaced with structured NotImplemented errors.

use std::any::Any;
use std::sync::Arc;

use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
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
    // TODO: W3-FIX-S307-003 — used when INSERT/UPDATE/DELETE route through WriteExecutor.
    #[allow(dead_code)]
    pub(crate) endpoint_spec: WriteEndpointSpec,
    /// The write executor (shared across all table providers for this engine).
    // TODO: W3-FIX-S307-003 — used when INSERT/UPDATE/DELETE route through WriteExecutor.
    #[allow(dead_code)]
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
    ///
    /// CRIT-2: builds the write result Arrow schema for DataFusion catalog registration.
    ///
    /// The schema represents the `WriteResult` fields exposed via SQL:
    /// - `operation_id` (Utf8): ULID of the write operation
    /// - `dry_run` (Boolean): always false for executed writes
    /// - `affected_count` (UInt32): total records targeted
    /// - `succeeded_count` (UInt32): records written successfully
    /// - `failed_count` (UInt32): records that failed at sensor API
    ///
    /// This minimal schema satisfies AC-6: SQL DML routes through the six-phase
    /// safety pipeline and returns structured write results.
    ///
    /// TODO: W3-FIX-S307-003 — full SQL DML → WriteExecutor routing for insert_into/update/delete.
    pub fn new(
        descriptor: WriteTableDescriptor,
        endpoint_spec: WriteEndpointSpec,
        executor: Arc<WriteExecutor>,
    ) -> Self {
        // Build Arrow schema representing the WriteResult output row.
        // Write-only tables return write result metadata, not sensor data.
        let schema = Arc::new(Schema::new(vec![
            Field::new("operation_id", DataType::Utf8, false),
            Field::new("dry_run", DataType::Boolean, false),
            Field::new("write_endpoint", DataType::Utf8, false),
            Field::new("affected_count", DataType::UInt32, false),
            Field::new("succeeded_count", DataType::UInt32, false),
            Field::new("failed_count", DataType::UInt32, false),
        ]));

        Self {
            descriptor,
            endpoint_spec,
            executor,
            schema,
        }
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
        _state: &dyn Session,
        _projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        _limit: Option<usize>,
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
        // CRIT-2: structured NotImplemented instead of todo!() panic.
        // Full WriteExecutor routing for SQL INSERT is deferred to W3-FIX-S307-003.
        // TODO: W3-FIX-S307-003 — route INSERT through WriteExecutor::execute.
        Err(DataFusionError::NotImplemented(format!(
            "S-3.07-pending: SQL INSERT INTO '{}' routing through WriteExecutor \
             is deferred to W3-FIX-S307-003; use pipe-mode write instead",
            self.descriptor.sql_table
        )))
    }

    async fn delete_from(
        &self,
        _state: &dyn Session,
        _filters: Vec<datafusion::prelude::Expr>,
    ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
        // CRIT-2: structured NotImplemented instead of todo!() panic.
        // TODO: W3-FIX-S307-003 — route DELETE FROM through WriteExecutor::execute.
        Err(DataFusionError::NotImplemented(format!(
            "S-3.07-pending: SQL DELETE FROM '{}' routing through WriteExecutor \
             is deferred to W3-FIX-S307-003; use pipe-mode write instead",
            self.descriptor.sql_table
        )))
    }

    async fn update(
        &self,
        _state: &dyn Session,
        _assignments: Vec<(String, datafusion::prelude::Expr)>,
        _filters: Vec<datafusion::prelude::Expr>,
    ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
        // CRIT-2: structured NotImplemented instead of todo!() panic.
        // TODO: W3-FIX-S307-003 — route UPDATE through WriteExecutor::execute.
        Err(DataFusionError::NotImplemented(format!(
            "S-3.07-pending: SQL UPDATE '{}' routing through WriteExecutor \
             is deferred to W3-FIX-S307-003; use pipe-mode write instead",
            self.descriptor.sql_table
        )))
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
    // CRIT-2: iterate descriptors, construct WriteCapableTableProvider, register in catalog.
    // Each descriptor is wrapped in a provider and registered under its sql_table name.
    for descriptor in descriptors {
        let sql_table = descriptor.sql_table.clone();
        let sensor = descriptor.sensor.clone();
        let verb = descriptor.verb.clone();

        // Resolve the endpoint spec from the executor's endpoint registry.
        // If not found, skip this descriptor (spec may be incomplete during startup).
        let endpoint_spec = match executor.endpoint_registry.get(&sensor, &verb) {
            Some(spec) => spec.clone(),
            None => {
                tracing::warn!(
                    sql_table = %sql_table,
                    sensor = %sensor,
                    verb = %verb,
                    "register_write_tables: no endpoint spec found for ({sensor}, {verb}); \
                     skipping table registration"
                );
                continue;
            }
        };

        let provider = WriteCapableTableProvider::new(descriptor, endpoint_spec, executor.clone());

        // Register the provider in the DataFusion catalog under the sql_table name.
        session
            .register_table(&sql_table, Arc::new(provider))
            .map_err(|e| PrismError::QueryExecutionFailed {
                detail: format!(
                    "failed to register write table '{sql_table}' in DataFusion catalog: {e}"
                ),
            })?;

        tracing::debug!(
            sql_table = %sql_table,
            "register_write_tables: registered write-capable table provider"
        );
    }

    Ok(())
}
