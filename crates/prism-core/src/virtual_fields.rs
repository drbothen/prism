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

/// The three virtual fields injected into every query result set.
///
/// Each variant maps to a fixed, underscore-prefixed column name that analysts
/// can reference in PrismQL predicates.
///
/// `VirtualField::Sensor.column_name()` returns `"_sensor"`,
/// `VirtualField::Client.column_name()` returns `"_client"`,
/// `VirtualField::SourceTable.column_name()` returns `"_source_table"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualField {
    /// `_sensor` — the sensor identifier (e.g., "crowdstrike", "armis", "prism").
    Sensor,
    /// `_client` — the client / tenant identifier (TenantId value).
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
