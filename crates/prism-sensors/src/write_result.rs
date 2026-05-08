//! Per-record write result types for `SensorAdapter::write()`.
//!
//! Defined in prism-sensors (not prism-query) because `SensorAdapter::write()`
//! returns these types, and prism-sensors cannot depend on prism-query.
//! prism-query re-exports these types in its `write_result` module.
//!
//! Story: S-3.07 | BC-2.04.007

use serde::{Deserialize, Serialize};

/// Per-record write status within a batch execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteStatus {
    /// Record write succeeded at the sensor API.
    Success,
    /// Record write failed at the sensor API; error detail in `RecordWriteResult.error`.
    Failed,
    /// Record was skipped (e.g., pre-filter eliminated it before dispatch).
    Skipped,
}

/// Outcome for a single record within a write batch.
///
/// Returned by `SensorAdapter::write()` as a `Vec<RecordWriteResult>` — one
/// entry per record in the input `RecordBatch`. Partial batch failure is normal;
/// failed records are NOT an error return from `write()` (story §Phase 5d).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordWriteResult {
    /// Identifier of this record (from `WriteEndpointSpec.record_id_field` column).
    pub record_id: String,
    /// Per-record write status.
    pub status: WriteStatus,
    /// Raw sensor API response for this record, if available.
    pub sensor_response: Option<serde_json::Value>,
    /// Error message if `status == Failed`.
    pub error: Option<String>,
}
