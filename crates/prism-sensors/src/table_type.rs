//! `TableType` enum — classifies sensor tables as point-in-time or event-stream.
//!
//! Drives query routing in `fanout.rs`: `PointInTime` tables issue a live API
//! fetch on every query; `EventStream` tables serve results from the local
//! RocksDB `event_buffer` column family and are populated by a background
//! `EventPoller` task.
//!
//! Story: S-2.08 | AC-2, AC-3, AC-8

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// TableType
// ---------------------------------------------------------------------------

/// Classification of a sensor table's data-delivery model.
///
/// `PointInTime` is the default (backward-compatible). `EventStream` activates
/// local buffering via `EventBufferStore` and a background `EventPoller` task.
///
/// # GREEN-BY-DESIGN
/// The `as_str` and `Display` implementations are pure enum→string mappings
/// with no business logic; they are intentionally fully implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TableType {
    /// Fetch live from sensor API on every query (default).
    #[default]
    PointInTime,
    /// Buffer locally in RocksDB; serve reads from buffer; poll on schedule.
    EventStream,
}

impl TableType {
    /// Returns the canonical string representation used in sensor spec TOML files.
    ///
    /// GREEN-BY-DESIGN: pure enum→string mapping; no business logic.
    pub fn as_str(&self) -> &'static str {
        match self {
            TableType::PointInTime => "point_in_time",
            TableType::EventStream => "event_stream",
        }
    }
}

impl std::fmt::Display for TableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// TableTypeRouteDecision
// ---------------------------------------------------------------------------

/// Outcome of evaluating a table's type during query routing.
///
/// Produced by `route_table_query()` and consumed by the fanout dispatcher
/// to select the correct fetch path (AC-2, AC-3, AC-8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableTypeRouteDecision {
    /// Issue a live API fetch (PointInTime tables, or EventStream cold-start fallback).
    LiveFetch,
    /// Serve results from the local `event_buffer` RocksDB CF (EventStream tables).
    BufferScan,
    /// EventStream table with no buffered data — fall back to live fetch once
    /// and write results to the buffer.
    ColdStartFallback,
}

// ---------------------------------------------------------------------------
// route_table_query
// ---------------------------------------------------------------------------

/// Determines the routing path for a query against the given table type.
///
/// Returns `LiveFetch` for `PointInTime` tables and `ColdStartFallback` when
/// an `EventStream` table has no buffered data. Returns `BufferScan` when the
/// `EventStream` buffer is non-empty.
///
/// # AC-2, AC-3, AC-8
/// This function implements the transparent routing contract: callers receive
/// the same result schema regardless of which path was taken.
///
/// # Stub
/// The `has_buffer_data` predicate is injected by the caller; real buffer-
/// presence checks require `EventBufferStore` which is implemented separately.
pub fn route_table_query(_table_type: TableType, _has_buffer_data: bool) -> TableTypeRouteDecision {
    todo!("AC-2 / AC-3 / AC-8: implement table-type routing; PointInTime → LiveFetch, EventStream + has_data → BufferScan, EventStream + !has_data → ColdStartFallback")
}
