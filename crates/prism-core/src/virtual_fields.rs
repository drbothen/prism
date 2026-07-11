//! VirtualField enum — pre-DataFusion queryable metadata columns (S-2.03).
//!
//! Virtual fields are injected into Arrow RecordBatches *before* DataFusion
//! execution, making them available in WHERE / GROUP BY / ORDER BY clauses.
//! They are distinct from decorator fields (which live in the `_meta` envelope
//! and are injected post-execution).
//!
//! Lives in prism-core (not prism-storage or prism-query) so that both layers
//! can reference the column names without creating circular dependencies.
//! (BC-2.15.009 — virtual field category)

use serde::{Deserialize, Serialize};

/// The three virtual provenance fields injected into internal-table
/// (RocksDB-backed) query result sets.
///
/// These columns are pre-injected before DataFusion execution so analysts can
/// reference them in `WHERE` / `GROUP BY` / `ORDER BY` clauses.  Sensor tables
/// receive **four** virtual fields — these three plus `_source_type` — via
/// `prism_query::virtual_fields::inject_virtual_fields` (BC-2.11.012 v1.9).
///
/// Each variant maps to a fixed, underscore-prefixed column name:
///
/// `VirtualField::Sensor.column_name()` returns `"_sensor"`,
/// `VirtualField::Client.column_name()` returns `"_client"`,
/// `VirtualField::SourceTable.column_name()` returns `"_source_table"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualField {
    /// `_sensor` — the sensor identifier (e.g., "crowdstrike", "armis", "prism").
    Sensor,
    /// `_client` — the client / tenant identifier (OrgSlug value).
    Client,
    /// `_source_table` — the specific table name (e.g., "crowdstrike_detections",
    /// "prism_alerts").
    SourceTable,
}

impl VirtualField {
    /// Returns the fixed Arrow column name for this virtual field.
    ///
    /// The mapping is static and must not change within a Prism release version;
    /// schema changes require a migration.
    pub fn column_name(&self) -> &'static str {
        match self {
            VirtualField::Sensor => "_sensor",
            VirtualField::Client => "_client",
            VirtualField::SourceTable => "_source_table",
        }
    }
}
