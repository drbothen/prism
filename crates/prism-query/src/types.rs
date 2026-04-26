//! `SensorQueryDescriptor` — lightweight query-routing descriptor for `inject_source_type`.
//!
//! This type carries only what the `_source_type` injection function needs:
//! the table's delivery model (`table_type`) and whether the current rows came
//! from the RocksDB buffer (`rows_from_buffer`).
//!
//! # Distinction from Other Descriptor Types (S-2.08 Architecture Compliance, Defect 1 fix)
//!
//! - `SensorQueryDescriptor` (this type, `prism-query`): describes sensor query
//!   routing for `_source_type` injection. Fields: `table_name`, `table_type`,
//!   `rows_from_buffer`. Passed to `inject_source_type` at query time.
//!
//! - `InternalTableDescriptor` (`prism-core`, S-2.03): describes internal RocksDB
//!   tables (alerts, audit, aliases). Fields: `table_name`, `domain`, `columns`,
//!   `requires_audit_read`, `rocksdb_backed`. Has no `table_type` field and no
//!   sensor routing context.
//!
//! - `SensorTableDescriptor` (`prism-spec-engine`): describes a sensor table for
//!   DataFusion registration. Fields: `table_name`, `columns`, `sensor_id`,
//!   `has_credentials`.
//!
//! These three types serve completely different consumers and MUST NOT be merged
//! or aliased (S-2.08 Architecture Compliance Rule).
//!
//! Story: S-2.08 | AC-9, AC-10, Task 7a

use prism_core::TableType;

// ---------------------------------------------------------------------------
// SensorQueryDescriptor
// ---------------------------------------------------------------------------

/// Lightweight descriptor passed to `inject_source_type` to describe the
/// query-time table context.
///
/// Carries only what the injection function needs: the table's delivery model
/// and whether the rows came from the buffer or a live fetch. Distinct from
/// `SensorTableDescriptor` (prism-spec-engine) and `InternalTableDescriptor`
/// (prism-core) — see module doc.
///
/// # AC-9, AC-10
/// `inject_source_type` uses `table_type` and `rows_from_buffer` to determine
/// whether each row receives `"_source_type": "buffered"` or `"live"`.
#[derive(Debug, Clone)]
pub struct SensorQueryDescriptor {
    /// Fully-qualified sensor table name (e.g., `"crowdstrike.process_events"`).
    pub table_name: String,
    /// Data-delivery model for this table (PointInTime or EventStream).
    pub table_type: TableType,
    /// `true` when the rows being injected came from the RocksDB buffer.
    /// `false` when they came from a live API fetch (including cold-start fallback).
    pub rows_from_buffer: bool,
}
