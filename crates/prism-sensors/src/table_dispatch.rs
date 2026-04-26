//! `table_dispatch` — query routing logic for sensor table types.
//!
//! Consumes `prism_core::TableType` (the single canonical definition) to
//! produce a `TableTypeRouteDecision` that drives query fan-out dispatch.
//!
//! `TableType` is NOT redefined here — it is imported from `prism_core`.
//! This satisfies S-2.08 Architecture Compliance Rule (Defect 2 fix).
//!
//! Story: S-2.08 | AC-2, AC-3, AC-8

pub use prism_core::TableType;

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
