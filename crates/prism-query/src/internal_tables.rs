//! `internal_tables` — DataFusion `TableProvider` for internal RocksDB-backed tables.
//!
//! `RocksDbTableProvider` wraps an `InternalTableDescriptor` from `prism-storage`
//! (S-2.03) and implements the DataFusion `TableProvider` trait, enabling internal
//! Prism tables to participate in PrismQL queries alongside external sensor tables.
//!
//! # Internal Tables Registered (BC-2.15.011)
//! - `prism_schedules`   — scheduled queries
//! - `prism_alerts`      — alert records
//! - `prism_cases`       — case records
//! - `prism_diff_results` — diff detection results
//! - `prism_audit`       — audit log (requires `audit.read` capability)
//!
//! # Capability Gate
//! Accessing `prism_audit` requires the `audit.read` capability. If denied,
//! `E-QUERY-011` (`PrismError::AuditTableAccessDenied`) is returned. (BC-2.15.011)
//!
//! # Scan Truncation
//! Scans are truncated at `PRISM_MAX_INTERNAL_TABLE_SCAN` entries. The
//! `_meta.scan_truncated` flag is set in the result metadata when truncation
//! occurs. (BC-2.11.006)
//!
//! # BC References
//! - BC-2.11.005 — both external and internal tables materialized in SessionContext
//! - BC-2.11.001 — internal tables queryable via `prism.*` source names
//!
//! Story: S-3.02

// S-3.02 stub functions: dead_code suppressed for stub phase (BC-5.38.001).
#![allow(dead_code)]

use std::{any::Any, sync::Arc};

use arrow::datatypes::SchemaRef;
use async_trait::async_trait;
use datafusion::{
    catalog::Session,
    logical_expr::{Expr, TableProviderFilterPushDown, TableType as DfTableType},
    physical_plan::ExecutionPlan,
};
use prism_core::PrismError;
use prism_storage::RocksStorageBackend;

/// Maximum entries returned by an internal table scan before truncation.
///
/// Distinct from the external sensor 10K cap. (BC-2.11.006)
pub const PRISM_MAX_INTERNAL_TABLE_SCAN: usize = 10_000;

// ---------------------------------------------------------------------------
// InternalTableDescriptor (newtype shim)
// ---------------------------------------------------------------------------

/// Descriptor for an internal RocksDB-backed Prism table.
///
/// This stub mirrors the structure from S-2.03 (`prism-core::InternalTableDescriptor`).
/// S-3.02 connects this to `RocksDbTableProvider`.
///
/// # Fields
/// - `table_name`: e.g. `"prism_alerts"`
/// - `domain`: RocksDB column family name
/// - `schema`: Arrow schema for materialized RecordBatches
/// - `requires_audit_read`: true for `prism_audit` table (BC-2.15.011)
#[derive(Debug, Clone)]
pub struct InternalTableDescriptor {
    /// Fully-qualified table name as registered with DataFusion.
    pub table_name: String,
    /// RocksDB domain (column family prefix). Used by `RocksStorageBackend::scan`.
    pub domain: String,
    /// Arrow schema for the materialized RecordBatches.
    pub schema: SchemaRef,
    /// True if accessing this table requires the `audit.read` capability.
    pub requires_audit_read: bool,
}

// ---------------------------------------------------------------------------
// RocksDbTableProvider
// ---------------------------------------------------------------------------

/// DataFusion `TableProvider` backed by `RocksStorageBackend` (RocksDB).
///
/// Implements the DataFusion `TableProvider` trait so internal Prism tables
/// can be registered with a `SessionContext` and queried via SQL alongside
/// external sensor MemTables.
///
/// # Scan Semantics (BC-2.11.005)
/// `scan()` calls `RocksStorageBackend::scan(domain, prefix)`, deserializes rows
/// to Arrow RecordBatches using the descriptor's schema, and truncates at
/// `PRISM_MAX_INTERNAL_TABLE_SCAN` entries.
///
/// # Capability Gate (BC-2.15.011)
/// For `requires_audit_read = true`, `scan()` checks the `audit.read`
/// capability before proceeding. Denied access returns `E-QUERY-011`.
pub struct RocksDbTableProvider {
    /// Descriptor defining schema, domain, and capability requirements.
    pub(crate) descriptor: InternalTableDescriptor,
    /// RocksDB-backed storage backend.
    pub(crate) backend: Arc<dyn RocksStorageBackend>,
}

impl std::fmt::Debug for RocksDbTableProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RocksDbTableProvider")
            .field("descriptor", &self.descriptor)
            .finish_non_exhaustive()
    }
}

impl RocksDbTableProvider {
    /// Construct a `RocksDbTableProvider` for the given descriptor and backend.
    pub fn new(descriptor: InternalTableDescriptor, backend: Arc<dyn RocksStorageBackend>) -> Self {
        Self {
            descriptor,
            backend,
        }
    }
}

#[async_trait]
impl datafusion::datasource::TableProvider for RocksDbTableProvider {
    fn as_any(&self) -> &dyn Any {
        todo!("S-3.02 — RocksDbTableProvider::as_any")
    }

    fn schema(&self) -> SchemaRef {
        todo!("S-3.02 — RocksDbTableProvider::schema")
    }

    fn table_type(&self) -> DfTableType {
        todo!("S-3.02 — RocksDbTableProvider::table_type")
    }

    async fn scan(
        &self,
        _state: &dyn Session,
        _projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> datafusion::error::Result<Arc<dyn ExecutionPlan>> {
        todo!("S-3.02 — RocksDbTableProvider::scan")
    }

    fn supports_filters_pushdown(
        &self,
        _filters: &[&Expr],
    ) -> datafusion::error::Result<Vec<TableProviderFilterPushDown>> {
        todo!("S-3.02 — RocksDbTableProvider::supports_filters_pushdown")
    }
}

// ---------------------------------------------------------------------------
// register_internal_tables
// ---------------------------------------------------------------------------

/// Register all internal tables into the given `SessionContext`.
///
/// Called during `run_materialization_pipeline` before SQL plan execution.
/// Registers: `prism_schedules`, `prism_alerts`, `prism_cases`,
/// `prism_diff_results`, `prism_audit`. (BC-2.15.011)
///
/// # Capability Gate
/// `prism_audit` registration is deferred — the capability check occurs at
/// `scan()` time, not registration time. This allows the table to appear in
/// schema introspection regardless of capability.
pub fn register_internal_tables(
    _ctx: &datafusion::execution::context::SessionContext,
    _backend: Arc<dyn RocksStorageBackend>,
) -> Result<(), PrismError> {
    todo!("S-3.02 — register_internal_tables")
}
