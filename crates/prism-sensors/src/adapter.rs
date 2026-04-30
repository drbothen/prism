//! SensorAdapter trait, SensorSpec, QueryParams, and SensorError.
//!
//! Defines the uniform async interface every sensor backend must implement.
//! All concrete adapters live in S-2.07; only the trait contract and shared
//! data types are defined here.
//!
//! # Architecture Compliance (S-2.06)
//! - `SensorAdapter` is object-safe: no generic methods; `&dyn SensorAuth` used
//!   in place of `impl SensorAuth` (BC-2.01.013).
//! - Trait is `Send + Sync + 'static` for use across tokio task boundaries.
//!
//! Story: S-2.06 | BCs: BC-2.01.013, BC-2.01.002

use std::fmt;

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::types::SensorType;
use prism_core::OrgId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::auth::SensorAuth;
use crate::types::FilterMap;

// ---------------------------------------------------------------------------
// SensorSpec
// ---------------------------------------------------------------------------

/// Identifies a single data-source feed within one sensor on one client.
///
/// Passed to `SensorAdapter::fetch()` to describe which table/feed to query.
/// The `sensor_config` field carries sensor-specific TOML-derived configuration
/// (e.g., base URL, page size) as an opaque JSON value.
///
/// # S-3.1.06 Stub: OrgId field added
/// `org_id` is the canonical per-org identity key (BC-3.2.001 precondition 3).
/// The legacy `client_id: String` field is retained for backward compat during
/// the Red Gate phase; it will be removed when S-3.1.06 implementation is
/// complete and all callers have migrated to `org_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorSpec {
    /// The logical table name (e.g., `"crowdstrike_alert"`, `"armis_device"`).
    pub source_table: String,
    /// Canonical organisation identity for this fetch (BC-3.2.001).
    ///
    /// This is the type-safe replacement for the legacy `client_id: String` field.
    /// All new code MUST use `org_id`; `client_id` is retained only for the
    /// duration of the S-3.1.06 Red Gate phase.
    ///
    /// Stub added by S-3.1.06 Stub Architect.  Implementation: S-3.1.06 Task 3.
    #[serde(default)]
    pub org_id: OrgId,
    /// The client (tenant) this fetch belongs to.
    ///
    /// # Deprecated
    /// Use `org_id` instead. Retained for backward compat during S-3.1.06
    /// Red Gate phase; will be removed in the implementation phase.
    #[deprecated(since = "0.2.0", note = "use `org_id: OrgId` instead (S-3.1.06)")]
    pub client_id: String,
    /// Sensor-specific configuration blob (from prism.toml or a sensor spec file).
    pub sensor_config: serde_json::Value,
}

// ---------------------------------------------------------------------------
// QueryParams
// ---------------------------------------------------------------------------

/// Parameters for a single sensor fetch: filter predicates, pagination cursor,
/// row limit, and time-range bounds.
///
/// Constructed by the query engine (S-3.02) from the PrismQL query plan.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryParams {
    /// Optional opaque cursor for paginated fetches (from prism-core CursorRegistry).
    pub cursor: Option<String>,
    /// Maximum number of rows to return in this fetch (0 = sensor default).
    pub limit: u64,
    /// Optional ISO-8601 start time for time-range pushdown.
    pub start_time: Option<String>,
    /// Optional ISO-8601 end time for time-range pushdown.
    pub end_time: Option<String>,
    /// Optional extra key/value filter predicates pushed down from the query planner.
    pub filters: FilterMap,
}

// ---------------------------------------------------------------------------
// SensorError
// ---------------------------------------------------------------------------

/// Errors returned by `SensorAdapter::fetch()` and retry/fan-out primitives.
///
/// Callers MUST distinguish transient errors (retryable) from permanent errors
/// (not retried) before deciding on retry strategy.
#[derive(Debug, Error)]
pub enum SensorError {
    /// Sensor API returned an unexpected HTTP status code.
    #[error("E-SENSOR-001: sensor {sensor} returned HTTP {status}: {body}")]
    HttpError {
        sensor: String,
        status: u16,
        body: String,
    },

    /// Sensor API call timed out.
    #[error("E-SENSOR-002: sensor {sensor} timed out after {elapsed_ms}ms")]
    Timeout { sensor: String, elapsed_ms: u64 },

    /// Sensor API returned a malformed or unrecognized response.
    #[error("E-SENSOR-003: sensor {sensor} response parse error: {detail}")]
    ResponseParse { sensor: String, detail: String },

    /// Sensor API rate-limited the request (HTTP 429).
    #[error("E-SENSOR-020: sensor {sensor} rate limited; retry after {retry_after_ms}ms")]
    RateLimited { sensor: String, retry_after_ms: u64 },

    /// No sensor adapter is registered for the requested sensor type.
    #[error("E-SENSOR-010: no adapter registered for sensor type {sensor_type}")]
    AdapterNotFound { sensor_type: SensorType },

    /// All fan-out targets failed — no partial results are available.
    ///
    /// Returned by `fan_out()` when every individual target returns an error
    /// and there are no successful `RecordBatch`es (BC-2.01.010, EC-001).
    #[error("E-SENSOR-030: all fan-out targets failed ({count} errors)")]
    AllTargetsFailed {
        count: usize,
        errors: Vec<crate::fanout::FanOutError>,
    },

    /// HTTP connection pool exhausted — semaphore acquisition timed out.
    ///
    /// Returned when a task cannot acquire a global HTTP semaphore permit
    /// within the 30-second timeout (S-2.06 §Task 7, EC-004).
    #[error("E-SENSOR-031: HTTP connection pool exhausted; semaphore acquisition timed out")]
    ConnectionPoolExhausted,

    /// Retry budget exhausted for a transient error.
    ///
    /// Returned by `retry_with_backoff()` after `max_attempts` are exceeded.
    #[error("E-SENSOR-032: retry budget exhausted for sensor {sensor} after {attempts} attempts")]
    RetryBudgetExhausted { sensor: String, attempts: u32 },

    /// Timestamp string could not be parsed by any supported format.
    ///
    /// Returned by `parse_timestamp()` when all format attempts fail (BC-2.01.006).
    #[error("E-SENSOR-040: unparseable timestamp: {raw:?}")]
    UnparseableTimestamp { raw: String },

    /// Internal error that does not fit any specific category.
    #[error("E-SENSOR-099: internal sensor error: {detail}")]
    Internal { detail: String },

    /// Spec-supplied AQL failed the allowlist validator (ADR-005, WGS-W2-001).
    ///
    /// Returned by `build_aql()` when `validate_aql()` rejects the AQL string
    /// before the HTTP call is issued.  The `detail` field includes the rejected
    /// AQL text and the validation failure reason (Q3 PO decision: including
    /// the AQL in the error aids debugging for trusted-analyst operators).
    ///
    /// # Note
    ///
    /// This error is **non-transient** — a structurally invalid AQL will not
    /// become valid on retry.  Callers MUST NOT retry on this variant.
    #[error("E-SENSOR-050: sensor {sensor} AQL config validation failed: {detail}")]
    ConfigValidation { sensor: String, detail: String },
}

impl SensorError {
    /// Returns `true` when the error is transient and may be retried.
    ///
    /// Non-transient errors (auth failures, not-found, bad-request) return
    /// `false` — retrying them is wasteful and masks bugs (BC-2.01.014).
    pub fn is_transient(&self) -> bool {
        match self {
            SensorError::HttpError { status, .. } => is_transient_status(*status),
            SensorError::Timeout { .. } => true,
            SensorError::RateLimited { .. } => true,
            SensorError::RetryBudgetExhausted { .. } => false,
            SensorError::AllTargetsFailed { .. } => false,
            SensorError::ConnectionPoolExhausted => false,
            SensorError::AdapterNotFound { .. } => false,
            SensorError::ResponseParse { .. } => false,
            SensorError::Internal { .. } => false,
            SensorError::UnparseableTimestamp { .. } => false,
            // ConfigValidation is a permanent error — retrying a structurally
            // invalid AQL will not produce a different result (ADR-005).
            SensorError::ConfigValidation { .. } => false,
        }
    }

    /// Extracts the HTTP status code if this is an `HttpError`, otherwise `None`.
    pub fn http_status(&self) -> Option<u16> {
        match self {
            SensorError::HttpError { status, .. } => Some(*status),
            SensorError::RateLimited { .. } => Some(429),
            _ => None,
        }
    }
}

/// Returns `true` for HTTP status codes treated as transient by the retry policy.
///
/// Transient codes: 429, 500, 502, 503, 504.
/// Non-transient 4xx (400, 401, 403, 404, …) are NOT retried (BC-2.01.014).
#[inline]
pub fn is_transient_status(status: u16) -> bool {
    matches!(status, 429 | 500 | 502 | 503 | 504)
}

// ---------------------------------------------------------------------------
// SensorAdapter trait
// ---------------------------------------------------------------------------

/// Uniform async interface for all sensor backends.
///
/// # Object Safety
/// This trait is object-safe. Do NOT add generic methods or associated types
/// that would break `dyn SensorAdapter` usage (BC-2.01.013).
///
/// # Concurrency
/// Implementations MUST be `Send + Sync + 'static` — adapters are shared
/// across tokio tasks via `Arc<dyn SensorAdapter>`.
///
/// Story: S-2.06 | BC: BC-2.01.013
#[async_trait]
pub trait SensorAdapter: Send + Sync + 'static {
    /// Returns the sensor type this adapter handles.
    ///
    /// Used by `AdapterRegistry` to key the adapter lookup table.
    fn sensor_type(&self) -> SensorType;

    /// Fetches a page of records from the sensor API.
    ///
    /// Implementations are responsible only for sensor-specific concerns:
    /// API call construction, response deserialization, and field extraction.
    /// Pagination cursor management and retry are handled by the shared
    /// infrastructure in `fanout.rs` and `retry.rs` (BC-2.01.013).
    ///
    /// # Arguments
    /// - `spec` — identifies which client/table to fetch from.
    /// - `params` — pagination cursor, row limit, filter predicates.
    /// - `auth` — sealed auth credential for this sensor; MUST NOT be logged.
    ///
    /// # Returns
    /// `Ok(Vec<RecordBatch>)` on success; one `RecordBatch` per API page
    /// fetched within this invocation. Empty vec indicates no more pages.
    async fn fetch(
        &self,
        spec: &SensorSpec,
        params: &QueryParams,
        auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError>;

    /// Returns a human-readable sensor name for use in tracing spans and error
    /// messages (e.g., `"crowdstrike"`, `"armis"`).
    fn sensor_name(&self) -> &'static str;
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

impl fmt::Display for SensorSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SensorSpec(org_id={}, table={})",
            self.org_id, self.source_table
        )
    }
}
