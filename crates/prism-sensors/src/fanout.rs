//! Cross-client fan-out orchestrator.
//!
//! `fan_out()` spawns one tokio task per `FanOutTarget`, collects all results
//! (successes and failures), and returns a `FanOutResult`. Partial failure is
//! the normal operating mode: as long as at least one target succeeds, the
//! query continues with partial results (BC-2.01.010).
//!
//! # Concurrency limits
//! Two semaphores work together (both MUST be held simultaneously by a task):
//! 1. **Fan-out semaphore** — 10 permits per `fan_out()` call; caps the
//!    number of concurrent sensor fetches within one query (BC-2.01.002).
//! 2. **Global HTTP semaphore** — 200 permits process-wide; caps total
//!    outbound HTTP connections (S-2.06 §Task 7, `crate::http`).
//!
//! Story: S-2.06 | BCs: BC-2.01.002, BC-2.01.010

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use prism_core::types::SensorType;
use tokio::sync::Semaphore;
use tracing::instrument;

use crate::{
    adapter::{QueryParams, SensorError, SensorSpec},
    auth::SensorAuth,
    registry::AdapterRegistry,
};

// ---------------------------------------------------------------------------
// FanOut concurrency constant
// ---------------------------------------------------------------------------

/// Maximum concurrent sensor fetches per `fan_out()` invocation.
///
/// This is distinct from the global HTTP semaphore (`HTTP_SEMAPHORE`).
/// A single task holds one fan-out permit AND one HTTP semaphore permit
/// simultaneously (AC-1, BC-2.01.002).
pub const MAX_FANOUT_CONCURRENCY: usize = 10;

// ---------------------------------------------------------------------------
// FanOutTarget
// ---------------------------------------------------------------------------

/// Identifies a single (client, sensor, source_table) fetch target.
#[derive(Debug, Clone)]
pub struct FanOutTarget {
    pub client_id: String,
    pub sensor_type: SensorType,
    pub spec: SensorSpec,
    pub params: QueryParams,
}

// ---------------------------------------------------------------------------
// RetryMetadata
// ---------------------------------------------------------------------------

/// Metadata describing the retry history for a failed target.
#[derive(Debug, Clone)]
pub struct RetryMetadata {
    /// Total fetch attempts including the initial attempt.
    pub attempts: u32,
    /// HTTP status code string from the last error (e.g., `"503"`, `"timeout"`).
    pub last_error_code: String,
    /// Whether the last error was classified as transient.
    pub is_transient: bool,
}

// ---------------------------------------------------------------------------
// FanOutError
// ---------------------------------------------------------------------------

/// A per-target failure record within a `FanOutResult`.
///
/// Even when a target fails, metadata about the failure is preserved so
/// callers can surface it in `sensor_errors` (BC-2.01.010, BC-2.01.002).
#[derive(Debug)]
pub struct FanOutError {
    pub client_id: String,
    pub sensor_type: SensorType,
    pub error: SensorError,
    pub retry_metadata: RetryMetadata,
}

impl std::fmt::Display for FanOutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FanOutError(client={}, sensor={}, attempts={}, transient={}): {}",
            self.client_id,
            self.sensor_type,
            self.retry_metadata.attempts,
            self.retry_metadata.is_transient,
            self.error,
        )
    }
}

// ---------------------------------------------------------------------------
// FanOutResult
// ---------------------------------------------------------------------------

/// The combined outcome of a `fan_out()` call.
///
/// `successes` holds all `RecordBatch`es from targets that completed without
/// error. `errors` holds one `FanOutError` per failed target. The result is
/// "partial" when both `successes` and `errors` are non-empty (AC-2).
#[derive(Debug, Default)]
pub struct FanOutResult {
    /// All `RecordBatch`es returned by successful targets.
    pub successes: Vec<RecordBatch>,
    /// Per-target error records for all failed targets.
    pub errors: Vec<FanOutError>,
}

// ---------------------------------------------------------------------------
// CredentialResolver
// ---------------------------------------------------------------------------

/// Resolves per-client, per-sensor auth credentials.
///
/// This trait is the abstraction point for the credential store (S-1.06).
/// `fan_out()` accepts `&dyn CredentialResolver` rather than a concrete store
/// so the query engine can provide the appropriate implementation.
///
/// Concrete implementation lives in S-2.07 (per-sensor auth resolution).
pub trait CredentialResolver: Send + Sync {
    /// Resolves the auth credential for `(client_id, sensor_type)`.
    ///
    /// Returns a boxed `dyn SensorAuth` on success. The concrete type is one
    /// of the four sealed `SensorAuth` subtypes; the resolver knows which type
    /// to return based on the sensor.
    fn resolve(
        &self,
        client_id: &str,
        sensor_type: SensorType,
    ) -> Result<Box<dyn SensorAuth>, SensorError>;
}

// ---------------------------------------------------------------------------
// fan_out
// ---------------------------------------------------------------------------

/// Fan out sensor fetches to all targets in parallel.
///
/// Spawns one tokio task per target. Each task:
/// 1. Acquires one permit from the fan-out semaphore (cap: `MAX_FANOUT_CONCURRENCY`).
/// 2. Acquires one permit from the global HTTP semaphore (cap: 200).
/// 3. Resolves credentials via `credentials.resolve(client_id, sensor_type)`.
/// 4. Calls `registry.get(sensor_type)?.fetch(spec, params, auth)`.
/// 5. Releases both permits on completion or error.
///
/// After all tasks complete via `join_all`, results are partitioned:
/// - At least one success → `Ok(FanOutResult { successes, errors })`
/// - All targets fail     → `Err(SensorError::AllTargetsFailed { errors })`
///
/// # Arguments
/// - `targets` — list of `(client_id, sensor_type, spec, params)` tuples.
/// - `registry` — shared adapter registry for adapter lookup.
/// - `credentials` — credential resolver for per-client auth.
///
/// # BC-2.01.002
/// Fan-out concurrency is capped at `MAX_FANOUT_CONCURRENCY` (10) per call.
///
/// # BC-2.01.010
/// Partial failure (some succeed, some fail) returns partial results + errors.
/// Only `AllTargetsFailed` is an `Err(...)` return.
///
/// Story: S-2.06 | BCs: BC-2.01.002, BC-2.01.010
#[instrument(skip_all, fields(target_count = targets.len()))]
pub async fn fan_out(
    targets: Vec<FanOutTarget>,
    _registry: Arc<AdapterRegistry>,
    _credentials: Arc<dyn CredentialResolver>,
) -> Result<FanOutResult, SensorError> {
    todo!(
        "AC-1 / BC-2.01.002: spawn one tokio task per target with fan-out semaphore \
         (cap {}) + global HTTP semaphore; collect via join_all; \
         partition successes and errors; return AllTargetsFailed if all fail",
        MAX_FANOUT_CONCURRENCY
    )
}

// ---------------------------------------------------------------------------
// Internal: per-target task
// ---------------------------------------------------------------------------

/// Executes a single fan-out task: resolves credentials, acquires permits, and
/// calls the appropriate `SensorAdapter::fetch()`.
///
/// Returns `Ok(Vec<RecordBatch>)` on success or `Err(FanOutError)` on failure.
/// The fan-out semaphore permit is passed in by the caller; the HTTP permit
/// is acquired inside this function (to keep the two distinct).
///
/// Story: S-2.06 | BC: BC-2.01.002
#[allow(dead_code)]
async fn execute_target(
    target: FanOutTarget,
    _registry: Arc<AdapterRegistry>,
    _credentials: Arc<dyn CredentialResolver>,
    _fanout_permit: tokio::sync::SemaphorePermit<'_>,
    _http_semaphore: Arc<Semaphore>,
) -> Result<Vec<RecordBatch>, FanOutError> {
    todo!(
        "AC-1 / BC-2.01.002: acquire HTTP semaphore permit; \
         resolve credentials; call adapter.fetch(); \
         log tracing span with client_id={} sensor_type={}",
        target.client_id,
        target.sensor_type
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Classifies a `SensorError` into `RetryMetadata` after all retry attempts
/// are exhausted (or no retry was attempted for non-transient errors).
pub fn error_to_retry_metadata(error: &SensorError, attempts: u32) -> RetryMetadata {
    let last_error_code = match error.http_status() {
        Some(code) => code.to_string(),
        None => match error {
            SensorError::Timeout { .. } => "timeout".to_string(),
            _ => "internal".to_string(),
        },
    };
    RetryMetadata {
        attempts,
        last_error_code,
        is_transient: error.is_transient(),
    }
}

// silence unused import warning for warn! macro path — used in todo impls
#[allow(unused_imports)]
use tracing::error;
