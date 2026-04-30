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
