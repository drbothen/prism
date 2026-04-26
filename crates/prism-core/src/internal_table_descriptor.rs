//! InternalTableDescriptor — registration metadata for RocksDB domains
//! exposed as PrismQL-queryable tables (S-2.03, BC-2.15.011).
//!
//! Each descriptor maps a PrismQL table name (e.g., `"prism_alerts"`) to the
//! backing `StorageDomain`, its column schema, and access-control flags.
//!
//! The Arrow schema for DataFusion is derived from `columns` inside prism-query
//! (S-3.02).  Column types are declared here using `types::InternalColumnType`
//! so this crate has no Arrow dependency.

use serde::{Deserialize, Serialize};

use crate::storage::StorageDomain;
use crate::types::ColumnType as InternalColumnType;

/// Registration metadata for a single internal PrismQL table.
///
/// Populated at compile time in `prism_storage::internal_tables::INTERNAL_TABLES`.
/// All fields are `'static` or owned so the static slice has no lifetime issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalTableDescriptor {
    /// PrismQL table name — underscore-delimited (dots are not valid identifiers).
    ///
    /// Examples: `"prism_alerts"`, `"prism_audit"`, `"prism_aliases"`.
    pub table_name: &'static str,

    /// The RocksDB `StorageDomain` backing this table.
    ///
    /// `None` for tables not backed directly by a RocksDB column family
    /// (e.g., tables backed by an alternative store).
    pub domain: Option<StorageDomain>,

    /// Ordered list of columns and their types.
    ///
    /// Uses `InternalColumnType` (a prism-core type with no Arrow dependency).
    /// prism-query (S-3.02) converts these to Arrow `DataType` values at
    /// query time.
    pub columns: Vec<(String, InternalColumnType)>,

    /// When `true`, the caller must have `audit.read = Allow` before this
    /// table can be scanned (BC-2.15.011 E-QUERY-011).
    pub requires_audit_read: bool,

    /// When `true`, the table is backed by RocksDB via `StorageBackend::scan()`.
    ///
    /// When `false`, the `TableProvider` in S-3.02 reads from an alternative
    /// store (e.g., the in-memory `AliasStore` loaded from `aliases.toml`).
    pub rocksdb_backed: bool,
}
