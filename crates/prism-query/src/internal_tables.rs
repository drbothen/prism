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
//! # Capability Gate — Two-Layer Defense-in-Depth (F-LP2-CRIT-1, BC-2.15.011)
//! Accessing `prism_audit` requires the `audit.read` capability. If denied,
//! `E-QUERY-011` (`PrismError::AuditTableAccessDenied`) is returned.
//!
//! **Layer 1 (pre-execution):** `engine.rs::check_internal_table_capabilities` recursively
//! walks the entire AST (including WHERE/HAVING subqueries like `IN (SELECT ... FROM prism_audit)`)
//! before any scan, returning `AuditTableAccessDenied` immediately.
//!
//! **Layer 2 (scan-time):** `RocksDbTableProvider::scan()` checks caller capabilities
//! stored at registration time. Even if Layer 1 is bypassed, `scan()` returns
//! `DataFusionError::Plan` with E-QUERY-011.
//!
//! **Layer 3 (descriptor-driven):** Both layers consult `InternalTableDescriptor.requires_audit_read`
//! (via `INTERNAL_TABLE_SPECS`), making the policy data-driven. Future tables flagged with
//! `requires_audit_read = true` are automatically gated without code changes.
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

// dead_code suppression removed — all items are now used (ADV-W3MT-P58-MED-002)

use std::{any::Any, sync::Arc};

use arrow::array::{BooleanArray, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use datafusion::{
    catalog::Session,
    datasource::memory::MemorySourceConfig,
    logical_expr::{Expr, TableProviderFilterPushDown, TableType as DfTableType},
    physical_plan::ExecutionPlan,
};
use prism_core::{PrismError, StorageDomain};
use prism_storage::RocksStorageBackend;

// Imported for scan-time capability gate (F-LP2-CRIT-1 Layer 2).
use crate::engine::Capability;

/// Maximum entries returned by an internal table scan before truncation.
///
/// BC-2.15.011 specifies 50,000 as the default soft limit for internal table scans.
/// This is distinct from the external sensor 10K hard cap (`MAX_MATERIALIZED_RECORDS`).
/// (ADV-W3MT-P58-CRIT-002)
pub const PRISM_MAX_INTERNAL_TABLE_SCAN: usize = 50_000;

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
/// - `schema`: Arrow schema for materialized RecordBatches (domain fields only)
/// - `requires_audit_read`: true for `prism_audit` table (BC-2.15.011)
///
/// # Virtual Fields (F-LP1-HIGH-4, BC-2.11.012)
/// The `full_schema()` method returns the complete schema including virtual fields
/// `_sensor`, `_client`, `_source_table`. `RocksDbTableProvider::schema()` exposes
/// this to DataFusion so query planning includes the virtual fields.
#[derive(Debug, Clone)]
pub struct InternalTableDescriptor {
    /// Fully-qualified table name as registered with DataFusion.
    pub table_name: String,
    /// RocksDB domain (column family prefix). Used by `RocksStorageBackend::scan`.
    pub domain: String,
    /// Arrow schema for the domain fields (without virtual fields).
    pub schema: SchemaRef,
    /// True if accessing this table requires the `audit.read` capability.
    pub requires_audit_read: bool,
}

impl InternalTableDescriptor {
    /// Return the full output schema: domain fields + virtual fields.
    ///
    /// DataFusion query planning uses this schema so that `SELECT *` includes
    /// `_sensor`, `_client`, `_source_table`, and `_meta_scan_truncated`.
    /// (F-LP1-HIGH-4, BC-2.11.012, ADV-W3MT-P59-HIGH-001)
    pub fn full_schema(&self) -> SchemaRef {
        let mut fields: Vec<Field> = self
            .schema
            .fields()
            .iter()
            .map(|f| f.as_ref().clone())
            .collect();
        fields.push(Field::new("_sensor", DataType::Utf8, false));
        fields.push(Field::new("_client", DataType::Utf8, false));
        fields.push(Field::new("_source_table", DataType::Utf8, false));
        fields.push(Field::new("_meta_scan_truncated", DataType::Boolean, false));
        Arc::new(Schema::new(fields))
    }
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
/// # Capability Gate — Two-Layer Enforcement (BC-2.15.011, F-LP2-CRIT-1)
/// Layer 1 (pre-execution): `engine.rs::check_internal_table_capabilities` walks the
/// full AST recursively (including WHERE/HAVING subqueries) before any scan begins.
/// Layer 2 (scan-time, this type): `scan()` checks `capabilities` against
/// `descriptor.requires_audit_read`. This is defense-in-depth — if Layer 1 is
/// bypassed (e.g., via direct DataFusion API usage), Layer 2 still denies access.
/// Both layers consult `descriptor.requires_audit_read` (Layer 3: descriptor-driven).
pub struct RocksDbTableProvider {
    /// Descriptor defining schema, domain, and capability requirements.
    pub(crate) descriptor: InternalTableDescriptor,
    /// RocksDB-backed storage backend.
    pub(crate) backend: Arc<dyn RocksStorageBackend>,
    /// Caller capabilities for scan-time gate (Layer 2, F-LP2-CRIT-1).
    ///
    /// Set at registration time via `new_with_capabilities`. Empty slice means
    /// no capabilities granted (deny for `requires_audit_read = true` tables).
    pub(crate) capabilities: Vec<Capability>,
}

impl std::fmt::Debug for RocksDbTableProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RocksDbTableProvider")
            .field("descriptor", &self.descriptor)
            .finish_non_exhaustive()
    }
}

impl RocksDbTableProvider {
    /// Construct a `RocksDbTableProvider` with no caller capabilities.
    ///
    /// The scan-time capability gate (Layer 2) will deny access to tables with
    /// `requires_audit_read = true`. Use `new_with_capabilities` to grant access.
    pub fn new(descriptor: InternalTableDescriptor, backend: Arc<dyn RocksStorageBackend>) -> Self {
        Self {
            descriptor,
            backend,
            capabilities: Vec::new(),
        }
    }

    /// Construct a `RocksDbTableProvider` with the caller's capabilities.
    ///
    /// The `capabilities` slice is stored and checked in `scan()` against
    /// `descriptor.requires_audit_read` (Layer 2 defense-in-depth for F-LP2-CRIT-1).
    ///
    /// # F-LP2-CRIT-1 Layer 2
    /// Passes the caller's `QueryOptions.capabilities` through to `scan()` so that
    /// even if the pre-execution gate (Layer 1) is bypassed, the scan itself denies
    /// access to `requires_audit_read = true` tables for callers without `AuditRead`.
    pub fn new_with_capabilities(
        descriptor: InternalTableDescriptor,
        backend: Arc<dyn RocksStorageBackend>,
        capabilities: Vec<Capability>,
    ) -> Self {
        Self {
            descriptor,
            backend,
            capabilities,
        }
    }
}

#[async_trait]
impl datafusion::datasource::TableProvider for RocksDbTableProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Return the full Arrow schema for this internal table.
    ///
    /// Includes both domain fields and virtual fields (`_sensor`, `_client`,
    /// `_source_table`) so DataFusion query planning includes the virtual fields
    /// in `SELECT *` expansions. (F-LP1-HIGH-4, BC-2.11.012)
    fn schema(&self) -> SchemaRef {
        self.descriptor.full_schema()
    }

    /// Return `TableType::Base` — internal tables are base tables (read from RocksDB).
    fn table_type(&self) -> DfTableType {
        DfTableType::Base
    }

    /// Scan the RocksDB column family for this domain.
    ///
    /// Reads all key-value pairs from the domain (using empty prefix to scan all),
    /// converts values to typed Arrow rows using the descriptor's schema, injects
    /// virtual fields (`_sensor`, `_client`, `_source_table`), and returns as
    /// `MemoryExec` plan. (F-LP1-HIGH-4, BC-2.11.012)
    ///
    /// # Scan-Time Capability Gate (F-LP2-CRIT-1 Layer 2)
    /// If `descriptor.requires_audit_read = true` and the caller's `capabilities`
    /// do not include `Capability::AuditRead`, returns `DataFusionError::Plan`
    /// with E-QUERY-011. This is defense-in-depth: catches bypasses of Layer 1.
    ///
    /// Scans are read-only — no `insert_into` is implemented (AD-022 write-safety).
    async fn scan(
        &self,
        _state: &dyn Session,
        projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        limit: Option<usize>,
    ) -> datafusion::error::Result<Arc<dyn ExecutionPlan>> {
        // F-LP2-CRIT-1 Layer 2: scan-time capability gate (defense-in-depth).
        // Checks descriptor.requires_audit_read against stored capabilities.
        // Layer 3: descriptor-driven — any table with requires_audit_read = true is gated.
        if self.descriptor.requires_audit_read
            && !self.capabilities.contains(&Capability::AuditRead)
        {
            return Err(datafusion::error::DataFusionError::Plan(
                "E-QUERY-011: Audit table requires audit.read capability. \
                 Grant via prism.toml [clients.{id}.capabilities]."
                    .to_string(),
            ));
        }

        // ADV-W3MT-P59-CRIT-001: prism_aliases is backed by AliasStore (TOML), NOT RocksDB.
        // `rocksdb_backed: false` — return empty batch pending AliasStore integration.
        // The domain string for prism_aliases is "aliases" but the CF is reserved; do NOT scan.
        if self.descriptor.table_name == "prism_aliases" {
            let schema = Arc::clone(&self.descriptor.schema);
            let empty = RecordBatch::new_empty(schema);
            let empty = inject_internal_virtual_fields(empty, &self.descriptor.table_name, false)?;
            let augmented_schema = empty.schema();
            let plan = MemorySourceConfig::try_new_exec(
                &[vec![empty]],
                augmented_schema,
                projection.cloned(),
            )?;
            return Ok(plan);
        }

        // Parse the StorageDomain from the descriptor's domain string.
        // We use an empty prefix scan to get all records in the domain.
        let domain_str = &self.descriptor.domain;
        let domain = parse_domain(domain_str).map_err(|e| {
            datafusion::error::DataFusionError::External(Box::new(std::io::Error::other(
                e.to_string(),
            )))
        })?;

        let kv_pairs = self.backend.scan(domain, b"").map_err(|e| {
            datafusion::error::DataFusionError::External(Box::new(std::io::Error::other(
                e.to_string(),
            )))
        })?;

        // Truncate at PRISM_MAX_INTERNAL_TABLE_SCAN or user-supplied limit, whichever is smaller.
        let cap = limit
            .map(|l| l.min(PRISM_MAX_INTERNAL_TABLE_SCAN))
            .unwrap_or(PRISM_MAX_INTERNAL_TABLE_SCAN);

        // Use domain-only schema for KV deserialization.
        // Virtual fields are added by inject_internal_virtual_fields below.
        // self.schema() returns the full schema (domain + virtual); we want domain-only here.
        let schema = Arc::clone(&self.descriptor.schema);
        let num_fields = schema.fields().len();

        // Build batches in chunks of 1000 rows.
        let chunk_size = 1000usize;
        let mut batches: Vec<RecordBatch> = Vec::new();

        // Collect all pairs first so we can detect truncation.
        // ADV-W3MT-P58-MED-001 / ADV-W3MT-P59-HIGH-001: BC-2.15.011 requires truncated scans
        // to be flagged via both tracing::warn! AND a _meta_scan_truncated Boolean column.
        let all_pairs: Vec<_> = kv_pairs.into_iter().collect();
        let total_pairs = all_pairs.len();
        let pairs_truncated: Vec<_> = all_pairs.into_iter().take(cap).collect();
        let scan_truncated = pairs_truncated.len() < total_pairs;
        if scan_truncated {
            tracing::warn!(
                table = %self.descriptor.table_name,
                total_rows = total_pairs,
                cap,
                "internal table scan truncated (BC-2.15.011 scan_truncated=true); \
                 increase PRISM_MAX_INTERNAL_TABLE_SCAN or add row-level filtering"
            );
        }

        for chunk in pairs_truncated.chunks(chunk_size) {
            let batch = build_batch_from_kv(schema.clone(), num_fields, chunk, domain)?;
            // F-LP1-HIGH-4: inject virtual fields into each batch.
            // BC-2.11.012: internal tables use _sensor = "prism", _client = "<system>",
            // _source_table = <table_name> (the bare domain table name).
            // ADV-W3MT-P59-HIGH-001: also inject _meta_scan_truncated column.
            let batch =
                inject_internal_virtual_fields(batch, &self.descriptor.table_name, scan_truncated)?;
            batches.push(batch);
        }

        // If no rows, produce a single empty batch (with virtual fields) to keep DataFusion happy.
        if batches.is_empty() {
            let empty = RecordBatch::new_empty(schema.clone());
            let empty =
                inject_internal_virtual_fields(empty, &self.descriptor.table_name, scan_truncated)?;
            batches.push(empty);
        }

        // The schema has been extended with virtual fields — use the augmented schema.
        let augmented_schema = batches[0].schema();
        let plan =
            MemorySourceConfig::try_new_exec(&[batches], augmented_schema, projection.cloned())?;

        Ok(plan)
    }

    /// Return `Inexact` for all filters — full scan, DataFusion applies filters post-scan.
    ///
    /// # Scope decision (F-LP3-OBS-2)
    /// Server-side filter pushdown into the RocksDB scan is intentionally deferred.
    /// `_filters` is unused because `Inexact` delegates all filter evaluation to
    /// DataFusion after the full scan completes.
    ///
    /// **Why deferred:** The `PRISM_MAX_INTERNAL_TABLE_SCAN=50_000` cap bounds
    /// the worst-case scan size, making full-scan acceptable for wave-4.
    /// (ADV-W3MT-P59-LOW-001: updated from stale 10_000 reference)
    /// True RocksDB seek-based filter pushdown (e.g., prefix scan on timestamp or
    /// audit_event_type) requires a stable `RocksStorageBackend::scan_range` API,
    /// which is out-of-scope until wave-5 schema stabilization.
    ///
    /// **Tracking:** Deferred as wave-5 follow-up: "Implement equality-filter
    /// pushdown for `RocksDbTableProvider` using RocksDB prefix seek to reduce
    /// full-CF scan overhead for large prism_audit tables."
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> datafusion::error::Result<Vec<TableProviderFilterPushDown>> {
        Ok(vec![TableProviderFilterPushDown::Inexact; filters.len()])
    }
}

// ---------------------------------------------------------------------------
// inject_internal_virtual_fields — add _sensor/_client/_source_table to internal-table batches
// ---------------------------------------------------------------------------

/// Inject virtual provenance fields into an internal-table RecordBatch.
///
/// Internal tables (backed by RocksDB) receive:
/// - `_sensor = "prism"` — distinguishes internal from external sensor data (BC-2.11.012).
/// - `_client = "<system>"` — internal tables are not per-org; they are system-wide.
/// - `_source_table = <table_name>` — the fully-qualified `prism_*` table name.
/// - `_meta_scan_truncated: Boolean` — true if the scan was truncated at PRISM_MAX_INTERNAL_TABLE_SCAN.
///   (ADV-W3MT-P59-HIGH-001, BC-2.15.011)
///
/// If the batch already contains any of these column names (schema drift or spoofing),
/// the existing column is overwritten with the canonical value.
///
/// # F-LP1-HIGH-4
/// This matches the virtual-field injection done for external sensor MemTables by
/// `crate::virtual_fields::inject_virtual_fields`. Internal tables must receive the
/// same treatment so they appear uniform to the query layer (BC-2.11.012 postcondition).
fn inject_internal_virtual_fields(
    batch: RecordBatch,
    table_name: &str,
    scan_truncated: bool,
) -> datafusion::error::Result<RecordBatch> {
    let num_rows = batch.num_rows();

    // Remove any pre-existing virtual columns to prevent schema drift / spoofing.
    let reserved: &[&str] = &[
        "_sensor",
        "_client",
        "_source_table",
        "_meta_scan_truncated",
    ];
    let keep_indices: Vec<usize> = batch
        .schema()
        .fields()
        .iter()
        .enumerate()
        .filter(|(_, f)| !reserved.contains(&f.name().as_str()))
        .map(|(i, _)| i)
        .collect();

    let base_batch = if keep_indices.len() == batch.num_columns() {
        batch
    } else {
        let new_columns: Vec<_> = keep_indices
            .iter()
            .map(|&i| batch.column(i).clone())
            .collect();
        let new_fields: Vec<arrow::datatypes::Field> = keep_indices
            .iter()
            .map(|&i| batch.schema().field(i).clone())
            .collect();
        let new_schema = std::sync::Arc::new(arrow::datatypes::Schema::new(new_fields));
        RecordBatch::try_new(new_schema, new_columns)
            .map_err(|e| datafusion::error::DataFusionError::ArrowError(Box::new(e), None))?
    };

    // Build virtual field arrays.
    let sensor_array: std::sync::Arc<dyn arrow::array::Array> =
        std::sync::Arc::new(StringArray::from(vec!["prism"; num_rows]));
    let client_array: std::sync::Arc<dyn arrow::array::Array> =
        std::sync::Arc::new(StringArray::from(vec!["<system>"; num_rows]));
    let table_array: std::sync::Arc<dyn arrow::array::Array> =
        std::sync::Arc::new(StringArray::from(vec![table_name; num_rows]));
    // ADV-W3MT-P59-HIGH-001: _meta_scan_truncated column (BC-2.15.011).
    let truncated_array: std::sync::Arc<dyn arrow::array::Array> =
        std::sync::Arc::new(BooleanArray::from(vec![scan_truncated; num_rows]));

    // Extend schema with virtual fields.
    let existing_schema = base_batch.schema();
    let mut new_fields: Vec<Field> = existing_schema
        .fields()
        .iter()
        .map(|f| f.as_ref().clone())
        .collect();
    new_fields.push(Field::new("_sensor", DataType::Utf8, false));
    new_fields.push(Field::new("_client", DataType::Utf8, false));
    new_fields.push(Field::new("_source_table", DataType::Utf8, false));
    new_fields.push(Field::new("_meta_scan_truncated", DataType::Boolean, false));
    let new_schema = std::sync::Arc::new(Schema::new(new_fields));

    // Build new column list.
    let mut new_columns: Vec<std::sync::Arc<dyn arrow::array::Array>> = (0..base_batch
        .num_columns())
        .map(|i| base_batch.column(i).clone())
        .collect();
    new_columns.push(sensor_array);
    new_columns.push(client_array);
    new_columns.push(table_array);
    new_columns.push(truncated_array);

    RecordBatch::try_new(new_schema, new_columns)
        .map_err(|e| datafusion::error::DataFusionError::ArrowError(Box::new(e), None))
}

// ---------------------------------------------------------------------------
// build_batch_from_kv — construct RecordBatch from raw KV pairs
// ---------------------------------------------------------------------------

/// Build an Arrow `RecordBatch` from raw key-value pairs, with domain-aware deserialization.
///
/// # Deserialization Strategy (F-LP1-HIGH-2, AD-012)
/// - `StorageDomain::AuditBuffer`: values are bincode 2.x encoded `AuditEntry` structs
///   (produced by `prism-storage::audit_buffer::append_audit_entry`). Deserialized via
///   `bincode::serde::decode_from_slice` and projected onto the audit schema columns.
///   Failed deserialization falls back to empty strings (graceful degradation — avoids
///   panicking on schema drift or partially-written entries).
/// - All other domains: raw bytes as best-effort UTF-8 for the first column, empty
///   strings for remaining columns. Full deserialization for those domains is deferred
///   to follow-up stories as domain types are stabilized.
///
/// # Schema Projection
/// Columns are filled to match `schema.fields()` order. Virtual fields are NOT included
/// here — they are appended by `inject_internal_virtual_fields()` after this function.
fn build_batch_from_kv(
    schema: SchemaRef,
    num_fields: usize,
    chunk: &[(Vec<u8>, Vec<u8>)],
    domain: StorageDomain,
) -> datafusion::error::Result<RecordBatch> {
    let n = chunk.len();
    let mut columns: Vec<Arc<dyn arrow::array::Array>> = Vec::with_capacity(num_fields);

    match domain {
        StorageDomain::AuditBuffer => {
            // F-LP1-HIGH-2 / ADV-W3MT-P59-CRIT-001: bincode 2.x deserialization of AuditEntry.
            // Authoritative schema (synced with prism-storage):
            //   { trace_id: Utf8, timestamp_ns: UInt64, operation: Utf8, client_id: Utf8,
            //     analyst_id: Utf8, outcome: Utf8, capability: Utf8 }
            // AuditEntry fields: { timestamp_ns: u64, trace_id: String, payload: BTreeMap<String,String> }
            // Projection:
            //   trace_id   <- e.trace_id
            //   timestamp_ns <- e.timestamp_ns (u64)
            //   operation  <- payload.get("operation").unwrap_or("")
            //   client_id  <- payload.get("client_id").unwrap_or("")
            //   analyst_id <- payload.get("analyst_id").unwrap_or("")
            //   outcome    <- payload.get("outcome").unwrap_or("")
            //   capability <- payload.get("capability").unwrap_or("")

            let mut trace_ids: Vec<String> = Vec::with_capacity(n);
            let mut timestamp_ns_vals: Vec<u64> = Vec::with_capacity(n);
            let mut operations: Vec<String> = Vec::with_capacity(n);
            let mut client_ids: Vec<String> = Vec::with_capacity(n);
            let mut analyst_ids: Vec<String> = Vec::with_capacity(n);
            let mut outcomes: Vec<String> = Vec::with_capacity(n);
            let mut capabilities: Vec<String> = Vec::with_capacity(n);

            for (_, value_bytes) in chunk {
                // ADV-W3MT-P58-LOW-001: config alignment.
                // `prism-storage::audit_buffer::append_audit_entry` encodes with
                // `bincode::config::standard()` (bincode 2.x default: little-endian,
                // variable-length integers, no byte limit). We decode with the same config
                // here. If the encoding config ever changes in prism-storage, update both
                // sides together. Failed deserialization gracefully falls back to defaults.
                let entry: Option<prism_storage::audit_buffer::AuditEntry> =
                    bincode::serde::decode_from_slice::<
                        prism_storage::audit_buffer::AuditEntry,
                        _,
                    >(value_bytes, bincode::config::standard())
                    .ok()
                    .map(|(e, _)| e);

                match entry {
                    Some(e) => {
                        trace_ids.push(e.trace_id.clone());
                        timestamp_ns_vals.push(e.timestamp_ns);
                        operations.push(e.payload.get("operation").cloned().unwrap_or_default());
                        client_ids.push(e.payload.get("client_id").cloned().unwrap_or_default());
                        analyst_ids.push(e.payload.get("analyst_id").cloned().unwrap_or_default());
                        outcomes.push(e.payload.get("outcome").cloned().unwrap_or_default());
                        capabilities.push(e.payload.get("capability").cloned().unwrap_or_default());
                    }
                    None => {
                        // Graceful degradation: failed deserialization → zero/empty defaults.
                        tracing::debug!(
                            "AuditBuffer: failed to deserialize entry via bincode 2.x; \
                             using defaults (graceful degradation)"
                        );
                        trace_ids.push(String::new());
                        timestamp_ns_vals.push(0u64);
                        operations.push(String::new());
                        client_ids.push(String::new());
                        analyst_ids.push(String::new());
                        outcomes.push(String::new());
                        capabilities.push(String::new());
                    }
                }
            }

            // Build columns in authoritative schema field order.
            let field_names: Vec<&str> =
                schema.fields().iter().map(|f| f.name().as_str()).collect();
            for field_name in &field_names {
                let arr: Arc<dyn arrow::array::Array> = match *field_name {
                    "trace_id" => Arc::new(StringArray::from(
                        trace_ids.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    )),
                    "timestamp_ns" => Arc::new(UInt64Array::from(timestamp_ns_vals.clone())),
                    "operation" => Arc::new(StringArray::from(
                        operations.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    )),
                    "client_id" => Arc::new(StringArray::from(
                        client_ids.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    )),
                    "analyst_id" => Arc::new(StringArray::from(
                        analyst_ids.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    )),
                    "outcome" => Arc::new(StringArray::from(
                        outcomes.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    )),
                    "capability" => Arc::new(StringArray::from(
                        capabilities.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    )),
                    _ => {
                        // Unknown field: empty string fallback.
                        Arc::new(StringArray::from(vec![""; n]))
                    }
                };
                columns.push(arr);
            }
        }

        _ => {
            // Other domains: raw bytes fallback — full deserialization deferred to
            // follow-up stories as domain types are stabilized (AD-012, story §Tasks step 6).
            // ADV-W3MT-P59-CRIT-001: handle UInt64 and Boolean columns with zero/false defaults.
            for (i, field) in schema.fields().iter().enumerate() {
                let arr: Arc<dyn arrow::array::Array> = match field.data_type() {
                    DataType::Utf8 => {
                        // For the first field, use raw value bytes as UTF-8 (best-effort).
                        // For other fields, use empty string.
                        if i == 0 {
                            let strings: Vec<&str> = chunk
                                .iter()
                                .map(|(_, v)| std::str::from_utf8(v).unwrap_or(""))
                                .collect();
                            Arc::new(StringArray::from(strings))
                        } else {
                            Arc::new(StringArray::from(vec![""; n]))
                        }
                    }
                    DataType::UInt64 => Arc::new(UInt64Array::from(vec![0u64; n])),
                    DataType::Boolean => Arc::new(BooleanArray::from(vec![false; n])),
                    DataType::Int32 => {
                        use arrow::array::Int32Array;
                        Arc::new(Int32Array::from(vec![0i32; n]))
                    }
                    _ => Arc::new(StringArray::from(vec![""; n])),
                };
                columns.push(arr);
            }
        }
    }

    // Verify column count matches schema field count before building batch.
    debug_assert_eq!(
        columns.len(),
        num_fields,
        "build_batch_from_kv: column count mismatch: expected {num_fields}, got {}",
        columns.len()
    );

    RecordBatch::try_new(schema, columns)
        .map_err(|e| datafusion::error::DataFusionError::ArrowError(Box::new(e), None))
}

// ---------------------------------------------------------------------------
// parse_domain — convert domain string to StorageDomain variant
// ---------------------------------------------------------------------------

/// Parse a domain string to the corresponding `StorageDomain`.
fn parse_domain(domain: &str) -> Result<StorageDomain, PrismError> {
    match domain {
        "audit_buffer" => Ok(StorageDomain::AuditBuffer),
        "alerts" => Ok(StorageDomain::Alerts),
        "cases" => Ok(StorageDomain::Cases),
        "schedules" => Ok(StorageDomain::Schedules),
        "diff_results" => Ok(StorageDomain::DiffResults),
        "event_buffer" => Ok(StorageDomain::EventBuffer),
        "detection_rules" => Ok(StorageDomain::DetectionRules),
        "detection_state" => Ok(StorageDomain::DetectionState),
        // ADV-W3MT-P58-HIGH-003: aliases domain added with prism_aliases table.
        "aliases" => Ok(StorageDomain::Aliases),
        other => Err(PrismError::QueryExecutionFailed {
            detail: format!("unknown storage domain: '{other}'"),
        }),
    }
}

// ---------------------------------------------------------------------------
// Internal table schema definitions
// ---------------------------------------------------------------------------

/// Return the Arrow schema for `prism_audit`.
///
/// Authoritative columns from `prism-storage::internal_tables::audit_columns()`.
/// (ADV-W3MT-P59-CRIT-001: synced with prism-storage authoritative INTERNAL_TABLES)
///
/// Schema: `{ trace_id: Utf8, timestamp_ns: UInt64, operation: Utf8, client_id: Utf8, analyst_id: Utf8, outcome: Utf8, capability: Utf8 }`
pub(crate) fn audit_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("trace_id", DataType::Utf8, false),
        Field::new("timestamp_ns", DataType::UInt64, false),
        Field::new("operation", DataType::Utf8, false),
        Field::new("client_id", DataType::Utf8, false),
        Field::new("analyst_id", DataType::Utf8, false),
        Field::new("outcome", DataType::Utf8, false),
        Field::new("capability", DataType::Utf8, false),
    ]))
}

/// Return the Arrow schema for `prism_alerts`.
///
/// Authoritative columns from `prism-storage::internal_tables::alerts_columns()`.
/// (ADV-W3MT-P59-CRIT-001: synced with prism-storage authoritative INTERNAL_TABLES)
///
/// Schema: `{ alert_id: Utf8, severity_id: UInt64, device_ip: Utf8, device_hostname: Utf8, client_id: Utf8, created_at: Utf8, rule_id: Utf8 }`
pub(crate) fn alerts_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("alert_id", DataType::Utf8, false),
        Field::new("severity_id", DataType::UInt64, false),
        Field::new("device_ip", DataType::Utf8, false),
        Field::new("device_hostname", DataType::Utf8, false),
        Field::new("client_id", DataType::Utf8, false),
        Field::new("created_at", DataType::Utf8, false),
        Field::new("rule_id", DataType::Utf8, false),
    ]))
}

/// Return the Arrow schema for `prism_cases`.
///
/// Authoritative columns from `prism-storage::internal_tables::cases_columns()`.
/// (ADV-W3MT-P59-CRIT-001: synced with prism-storage authoritative INTERNAL_TABLES)
///
/// Schema: `{ case_id: Utf8, title: Utf8, severity_id: UInt64, client_id: Utf8, created_at: Utf8, status: Utf8 }`
pub(crate) fn cases_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("case_id", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("severity_id", DataType::UInt64, false),
        Field::new("client_id", DataType::Utf8, false),
        Field::new("created_at", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
    ]))
}

/// Return the Arrow schema for `prism_schedules`.
///
/// Authoritative columns from `prism-storage::internal_tables::schedules_columns()`.
/// (ADV-W3MT-P59-CRIT-001: synced with prism-storage authoritative INTERNAL_TABLES)
///
/// Schema: `{ schedule_id: Utf8, name: Utf8, client_id: Utf8, query: Utf8, interval_secs: UInt64, last_run_at: Utf8 }`
pub(crate) fn schedules_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("schedule_id", DataType::Utf8, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("client_id", DataType::Utf8, false),
        Field::new("query", DataType::Utf8, false),
        Field::new("interval_secs", DataType::UInt64, false),
        Field::new("last_run_at", DataType::Utf8, false),
    ]))
}

/// Return the Arrow schema for `prism_diff_results`.
///
/// Authoritative columns from `prism-storage::internal_tables::diff_results_columns()`.
/// (ADV-W3MT-P59-CRIT-001: synced with prism-storage authoritative INTERNAL_TABLES)
///
/// Schema: `{ query_hash: Utf8, client_id: Utf8, previous_results_hash: Utf8, epoch: UInt64, counter: UInt64, last_diff_time: Utf8 }`
pub(crate) fn diff_results_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("query_hash", DataType::Utf8, false),
        Field::new("client_id", DataType::Utf8, false),
        Field::new("previous_results_hash", DataType::Utf8, false),
        Field::new("epoch", DataType::UInt64, false),
        Field::new("counter", DataType::UInt64, false),
        Field::new("last_diff_time", DataType::Utf8, false),
    ]))
}

/// Return the Arrow schema for `prism_rules`.
///
/// Authoritative columns from `prism-storage::internal_tables::rules_columns()`.
/// (ADV-W3MT-P59-CRIT-001: synced with prism-storage authoritative INTERNAL_TABLES)
///
/// Schema: `{ rule_id: Utf8, name: Utf8, client_id: Utf8, enabled: Boolean, created_at: Utf8 }`
pub(crate) fn rules_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("rule_id", DataType::Utf8, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("client_id", DataType::Utf8, false),
        Field::new("enabled", DataType::Boolean, false),
        Field::new("created_at", DataType::Utf8, false),
    ]))
}

/// Return the Arrow schema for `prism_aliases`.
///
/// Authoritative columns from `prism-storage::internal_tables::aliases_columns()`.
/// `prism_aliases` is backed by AliasStore (TOML), NOT RocksDB (`rocksdb_backed: false`).
/// The scan() method returns an empty batch for this table pending AliasStore integration.
/// (ADV-W3MT-P59-CRIT-001: synced with prism-storage authoritative INTERNAL_TABLES)
///
/// Schema: `{ alias_id: Utf8, alias: Utf8, expansion: Utf8, client_id: Utf8, created_at: Utf8 }`
pub(crate) fn aliases_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("alias_id", DataType::Utf8, false),
        Field::new("alias", DataType::Utf8, false),
        Field::new("expansion", DataType::Utf8, false),
        Field::new("client_id", DataType::Utf8, false),
        Field::new("created_at", DataType::Utf8, false),
    ]))
}

// ---------------------------------------------------------------------------
// register_internal_tables
// ---------------------------------------------------------------------------

/// Static table descriptor table for all internal Prism tables.
///
/// Each entry: `(table_name, domain, requires_audit_read)`.
/// Used by both `register_internal_tables_with_capabilities` and
/// `table_requires_audit_read` for descriptor-driven policy. (F-LP2-CRIT-1 Layer 3)
///
/// BC-2.15.011 requires at minimum 7 tables. (ADV-W3MT-P58-HIGH-003)
const INTERNAL_TABLE_SPECS: &[(&str, &str, bool)] = &[
    ("prism_audit", "audit_buffer", true),
    ("prism_alerts", "alerts", false),
    ("prism_cases", "cases", false),
    ("prism_schedules", "schedules", false),
    ("prism_diff_results", "diff_results", false),
    ("prism_rules", "detection_rules", false),
    ("prism_aliases", "aliases", false),
];

/// Check whether a named internal table requires the `audit.read` capability.
///
/// Consults `INTERNAL_TABLE_SPECS` (descriptor-driven policy, F-LP2-CRIT-1 Layer 3).
/// Returns `true` for `prism_audit` and any future table flagged as `requires_audit_read`.
/// Returns `false` for unknown table names (conservative: unknown tables are not gated here).
pub fn table_requires_audit_read(table_name: &str) -> bool {
    INTERNAL_TABLE_SPECS
        .iter()
        .find(|(name, _, _)| *name == table_name)
        .map(|(_, _, requires)| *requires)
        .unwrap_or(false)
}

/// Register all internal tables into the given `SessionContext`, forwarding
/// caller capabilities to each `RocksDbTableProvider` for scan-time enforcement.
///
/// # Two-Layer Enforcement (F-LP2-CRIT-1)
/// - Layer 1: pre-execution gate in `engine.rs::check_internal_table_capabilities`
///   (recursive AST walk before any scan).
/// - Layer 2 (this function): passes `capabilities` to `RocksDbTableProvider::new_with_capabilities`
///   so `scan()` enforces the gate even if Layer 1 is bypassed.
///
/// # BC-2.15.011
/// `prism_audit` requires `audit.read` capability. All tables with `requires_audit_read = true`
/// in `INTERNAL_TABLE_SPECS` are automatically gated.
///
/// # Idempotency (EC-003)
/// If called twice, the second call overwrites the first. No error is returned.
pub fn register_internal_tables_with_capabilities(
    ctx: &datafusion::execution::context::SessionContext,
    backend: Arc<dyn RocksStorageBackend>,
    capabilities: &[crate::engine::Capability],
) -> Result<(), PrismError> {
    // ADV-W3MT-P58-HIGH-003: schemas slice must align 1:1 with INTERNAL_TABLE_SPECS entries.
    // Order: audit, alerts, cases, schedules, diff_results, rules, aliases (7 total per BC-2.15.011).
    let schemas: &[SchemaRef] = &[
        audit_schema(),
        alerts_schema(),
        cases_schema(),
        schedules_schema(),
        diff_results_schema(),
        rules_schema(),
        aliases_schema(),
    ];

    for ((table_name, domain, requires_audit_read), schema) in
        INTERNAL_TABLE_SPECS.iter().zip(schemas.iter())
    {
        let descriptor = InternalTableDescriptor {
            table_name: table_name.to_string(),
            domain: domain.to_string(),
            schema: Arc::clone(schema),
            requires_audit_read: *requires_audit_read,
        };
        // F-LP2-CRIT-1 Layer 2: pass capabilities to provider for scan-time enforcement.
        let provider = Arc::new(RocksDbTableProvider::new_with_capabilities(
            descriptor,
            Arc::clone(&backend),
            capabilities.to_vec(),
        ));
        ctx.register_table(*table_name, provider).map_err(|e| {
            PrismError::QueryExecutionFailed {
                detail: format!("failed to register internal table '{table_name}': {e}"),
            }
        })?;
    }

    Ok(())
}

/// Register all internal tables into the given `SessionContext` with no capabilities.
///
/// Legacy API — preserved for callers that do not need capability-aware registration.
/// Equivalent to `register_internal_tables_with_capabilities(ctx, backend, &[])`.
///
/// # Capability Gate
/// Tables with `requires_audit_read = true` will deny scan access (Layer 2 gate).
/// Use `register_internal_tables_with_capabilities` to grant AuditRead access.
///
/// # Idempotency (EC-003)
/// If called twice, the second call overwrites the first. No error is returned.
pub fn register_internal_tables(
    ctx: &datafusion::execution::context::SessionContext,
    backend: Arc<dyn RocksStorageBackend>,
) -> Result<(), PrismError> {
    register_internal_tables_with_capabilities(ctx, backend, &[])
}
