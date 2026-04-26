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
//! # Table-type dispatch (S-2.08)
//! Before the existing fan-out logic, `dispatch_by_table_type()` inspects each
//! target's `TableSpec.table_type` and routes it to either the buffer scan path
//! (`EventStream`) or the live API fetch path (`PointInTime`). See
//! `crate::table_type::route_table_query()`.
//!
//! Story: S-2.06 | BCs: BC-2.01.002, BC-2.01.010
//! Story: S-2.08 | AC-2, AC-3, AC-5, AC-8

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
// dispatch_by_table_type (S-2.08)
// ---------------------------------------------------------------------------

/// Routes a single fan-out target through table-type dispatch before the
/// existing live-API fan-out logic.
///
/// # Routing Rules (AC-2, AC-3, AC-8)
/// - `TableType::PointInTime` → live API fetch (unchanged existing path)
/// - `TableType::EventStream` with buffered data → `EventBufferStore::scan_events`
/// - `TableType::EventStream` with no buffered data (cold start, AC-5) →
///   live API fetch once, write results to buffer, log INFO cold-start fallback
///
/// This is the single dispatch entry point used by all three PrismQL modes.
/// The result schema is identical regardless of which path was taken (AC-8).
///
/// # Stub
/// Full implementation depends on `EventBufferStore` and `TableSpec.table_type`
/// being wired together (S-2.08 implementation dispatch).
///
/// Story: S-2.08 | AC-2, AC-3, AC-5, AC-8
#[allow(dead_code)]
pub async fn dispatch_by_table_type(_target: &FanOutTarget) -> Result<FanOutResult, SensorError> {
    todo!("AC-2 / AC-3 / AC-5 / AC-8: implement table-type dispatch; check target TableSpec.table_type, route to EventBufferStore::scan_events or live API fetch, handle cold-start fallback with INFO log")
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
    registry: Arc<AdapterRegistry>,
    credentials: Arc<dyn CredentialResolver>,
) -> Result<FanOutResult, SensorError> {
    // Short-circuit for empty target list (returns empty FanOutResult — not AllTargetsFailed)
    if targets.is_empty() {
        return Ok(FanOutResult::default());
    }

    // Per-query fan-out semaphore caps concurrency at MAX_FANOUT_CONCURRENCY.
    let fanout_semaphore = Arc::new(Semaphore::new(MAX_FANOUT_CONCURRENCY));
    // Global HTTP semaphore must be initialized by the time fan_out() is called.
    crate::http::init_http_semaphore();

    // Spawn one task per target. Each task acquires fanout + HTTP permits.
    let tasks: Vec<_> = targets
        .into_iter()
        .map(|target| {
            let registry = Arc::clone(&registry);
            let credentials = Arc::clone(&credentials);
            let fanout_sem = Arc::clone(&fanout_semaphore);

            tokio::spawn(async move {
                // Acquire fan-out permit (owned, safe to move into task).
                // AcquireError fires only when the semaphore is explicitly closed,
                // which never happens here — we created it moments ago and hold
                // the only Arc. Map defensively to avoid `expect()`.
                let _fanout_permit = match fanout_sem.acquire_owned().await {
                    Ok(p) => p,
                    Err(_closed) => {
                        let e = SensorError::Internal {
                            detail: "fan-out semaphore closed unexpectedly".into(),
                        };
                        let retry_metadata = error_to_retry_metadata(&e, 1);
                        return Err(FanOutError {
                            client_id: target.client_id.clone(),
                            sensor_type: target.sensor_type,
                            error: e,
                            retry_metadata,
                        });
                    }
                };

                // Acquire global HTTP permit
                let _http_permit = match crate::http::acquire_http_permit().await {
                    Ok(p) => p,
                    Err(e) => {
                        let retry_metadata = error_to_retry_metadata(&e, 1);
                        let client_id = target.client_id.clone();
                        let sensor_type = target.sensor_type;
                        return Err(FanOutError {
                            client_id,
                            sensor_type,
                            error: e,
                            retry_metadata,
                        });
                    }
                };

                // Resolve credentials for this (client, sensor) pair
                let auth = match credentials.resolve(&target.client_id, target.sensor_type) {
                    Ok(a) => a,
                    Err(e) => {
                        let retry_metadata = error_to_retry_metadata(&e, 1);
                        return Err(FanOutError {
                            client_id: target.client_id.clone(),
                            sensor_type: target.sensor_type,
                            error: e,
                            retry_metadata,
                        });
                    }
                };

                // Look up the adapter for this sensor type
                let adapter = match registry.get(target.sensor_type) {
                    Some(a) => a,
                    None => {
                        let e = SensorError::AdapterNotFound {
                            sensor_type: target.sensor_type,
                        };
                        let retry_metadata = error_to_retry_metadata(&e, 1);
                        return Err(FanOutError {
                            client_id: target.client_id.clone(),
                            sensor_type: target.sensor_type,
                            error: e,
                            retry_metadata,
                        });
                    }
                };

                // Execute the fetch with a tracing span per AC-1
                let span = tracing::info_span!(
                    "fan_out_task",
                    client_id = %target.client_id,
                    sensor_type = %target.sensor_type,
                );
                let _enter = span.enter();

                match adapter
                    .fetch(&target.spec, &target.params, auth.as_ref())
                    .await
                {
                    Ok(batches) => Ok(batches),
                    Err(e) => {
                        let retry_metadata = error_to_retry_metadata(&e, 1);
                        Err(FanOutError {
                            client_id: target.client_id.clone(),
                            sensor_type: target.sensor_type,
                            error: e,
                            retry_metadata,
                        })
                    }
                }
            })
        })
        .collect();

    // Collect all task results (join_all does NOT short-circuit on failure)
    let outcomes = futures::future::join_all(tasks).await;

    let mut result = FanOutResult::default();

    for outcome in outcomes {
        match outcome {
            Ok(Ok(batches)) => result.successes.extend(batches),
            Ok(Err(fan_err)) => result.errors.push(fan_err),
            Err(join_err) => {
                // Task panicked — treat as internal error
                result.errors.push(FanOutError {
                    client_id: "<unknown>".into(),
                    sensor_type: prism_core::types::SensorType::CrowdStrike,
                    error: SensorError::Internal {
                        detail: format!("task panic: {join_err}"),
                    },
                    retry_metadata: RetryMetadata {
                        attempts: 1,
                        last_error_code: "internal".into(),
                        is_transient: false,
                    },
                });
            }
        }
    }

    // BC-2.01.010: all targets failed → Err(AllTargetsFailed)
    if result.successes.is_empty() && !result.errors.is_empty() {
        let count = result.errors.len();
        return Err(SensorError::AllTargetsFailed {
            count,
            errors: result.errors,
        });
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Internal: per-target task
// ---------------------------------------------------------------------------

/// Executes a single fan-out task: resolves credentials, acquires the HTTP
/// permit, and calls the appropriate `SensorAdapter::fetch()`.
///
/// Returns `Ok(Vec<RecordBatch>)` on success or `Err(FanOutError)` on failure.
/// The fan-out semaphore permit is passed in by the caller (already held);
/// the HTTP permit is acquired inside this function (to keep the two distinct).
///
/// Story: S-2.06 | BC: BC-2.01.002
#[allow(dead_code)]
async fn execute_target(
    target: FanOutTarget,
    registry: Arc<AdapterRegistry>,
    credentials: Arc<dyn CredentialResolver>,
    _fanout_permit: tokio::sync::SemaphorePermit<'_>,
    _http_semaphore: Arc<Semaphore>,
) -> Result<Vec<RecordBatch>, FanOutError> {
    // Acquire global HTTP permit (held until function returns)
    let _http_permit = match crate::http::acquire_http_permit().await {
        Ok(p) => p,
        Err(e) => {
            let retry_metadata = error_to_retry_metadata(&e, 1);
            return Err(FanOutError {
                client_id: target.client_id.clone(),
                sensor_type: target.sensor_type,
                error: e,
                retry_metadata,
            });
        }
    };

    // Resolve credentials
    let auth = match credentials.resolve(&target.client_id, target.sensor_type) {
        Ok(a) => a,
        Err(e) => {
            let retry_metadata = error_to_retry_metadata(&e, 1);
            return Err(FanOutError {
                client_id: target.client_id.clone(),
                sensor_type: target.sensor_type,
                error: e,
                retry_metadata,
            });
        }
    };

    // Look up the adapter
    let adapter = match registry.get(target.sensor_type) {
        Some(a) => a,
        None => {
            let e = SensorError::AdapterNotFound {
                sensor_type: target.sensor_type,
            };
            let retry_metadata = error_to_retry_metadata(&e, 1);
            return Err(FanOutError {
                client_id: target.client_id.clone(),
                sensor_type: target.sensor_type,
                error: e,
                retry_metadata,
            });
        }
    };

    // Fetch with a tracing span (AC-1: distinct client_id + sensor_type fields)
    let span = tracing::info_span!(
        "fan_out_task",
        client_id = %target.client_id,
        sensor_type = %target.sensor_type,
    );
    let _enter = span.enter();

    adapter
        .fetch(&target.spec, &target.params, auth.as_ref())
        .await
        .map_err(|e| {
            let retry_metadata = error_to_retry_metadata(&e, 1);
            FanOutError {
                client_id: target.client_id.clone(),
                sensor_type: target.sensor_type,
                error: e,
                retry_metadata,
            }
        })
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
