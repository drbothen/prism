//! `TableType` enum — canonical classification of sensor table data-delivery model.
//!
//! This is the single authoritative definition of `TableType` for the entire
//! workspace. Both `prism-sensors` and `prism-spec-engine` import from here;
//! neither crate defines its own copy (S-2.08 Architecture Compliance Rule,
//! Defect 2 fix).
//!
//! Story: S-2.08 | AC-2, AC-3, AC-8

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// TableType
// ---------------------------------------------------------------------------

/// Classification of a sensor table's data-delivery model.
///
/// `PointInTime` is the default (backward-compatible with all pre-S-2.08 specs).
/// `EventStream` activates local RocksDB buffering via `EventBufferStore` and
/// a background `EventPoller` task.
///
/// # Canonical Home
/// This enum is defined **only** in `prism-core`. Both `prism-sensors` and
/// `prism-spec-engine` import `prism_core::TableType`; neither crate defines
/// its own copy (S-2.08 Architecture Compliance Rule, Defect 2 fix).
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
