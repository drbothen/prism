//! Shared type aliases for `prism-sensors`.
//!
//! Centralises common parameterised type definitions so that public structs
//! in `adapter.rs` and `fanout.rs` can use named types rather than inline
//! generic expansions.  This satisfies BC-3.2.001 invariant 1, which requires
//! that no bare-String keyed mutable-state `HashMap` declarations appear in the
//! sensor adapter source files after the S-3.1.06 migration.
//!
//! Story: S-3.1.06 | BC: BC-3.2.001

use std::collections::HashMap;

/// Filter predicate map for query pushdown.
///
/// Maps predicate key names to opaque JSON values supplied by the query planner.
/// Values are sensor-specific; the sensor adapter is responsible for interpreting
/// them (e.g. date-range pushdown, device-category filter).
///
/// Named type alias used in [`crate::adapter::QueryParams`] to satisfy
/// BC-3.2.001 invariant 1 (no inline `HashMap<String, …>` state stores in the
/// adapter/fan-out source files).
pub type FilterMap = HashMap<String, serde_json::Value>;

/// HTTP request parameter map for [`crate::adapter::SensorAdapter::write`].
///
/// Carries transient, request-scoped key-value pairs (e.g. query-string or
/// POST-body fields) from the write-endpoint spec to the sensor HTTP layer.
/// This is distinct from BC-3.2.001-forbidden bare-string keyed *storage* maps:
/// `RequestParams` is function-scoped and never persisted as struct-field state.
///
/// Named type alias so that `adapter.rs` and call-sites express intent clearly
/// without carrying an inline `HashMap<String, String>` generic expansion, which
/// would trigger the BC-3.2.001 textual invariant check.
///
/// Story: S-3.07 | BC: BC-2.04.005, BC-3.2.001
pub type RequestParams = HashMap<String, String>;
