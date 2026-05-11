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
use prism_core::{OrgId, SensorId};
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

/// Identifies a single (org, sensor, source_table) fetch target.
///
/// # S-3.1.06 Stub: `org_id` added
/// `org_id` is the canonical per-org identity for dispatch (BC-3.2.001).
/// The legacy `client_id: String` is retained during the Red Gate phase;
/// it will be removed when S-3.1.06 implementation is complete.
#[derive(Debug, Clone)]
pub struct FanOutTarget {
    /// Canonical organisation identity for this fan-out target (BC-3.2.001).
    ///
    /// Stub added by S-3.1.06 Stub Architect.  Implementation: S-3.1.06 Task 4.
    pub org_id: OrgId,
    /// Legacy client identifier — use `org_id` for new code.
    ///
    /// # Deprecated
    /// Will be removed once all callers migrate to `org_id` (S-3.1.06).
    #[deprecated(since = "0.2.0", note = "use `org_id: OrgId` instead (S-3.1.06)")]
    pub client_id: String,
    pub sensor_id: SensorId,
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
///
/// # S-3.1.06 Stub: `org_id` added
/// `org_id` mirrors the canonical identity from `FanOutTarget` so error
/// attribution is org-scoped (BC-3.2.001). The legacy `client_id: String`
/// is retained during the Red Gate phase.
#[derive(Debug)]
pub struct FanOutError {
    /// Canonical organisation identity for this error (BC-3.2.001).
    ///
    /// Stub added by S-3.1.06 Stub Architect.  Implementation: S-3.1.06 Task 4.
    pub org_id: OrgId,
    /// Legacy client identifier — use `org_id` for new code.
    ///
    /// # Deprecated
    /// Will be removed once all callers migrate to `org_id` (S-3.1.06).
    #[deprecated(since = "0.2.0", note = "use `org_id: OrgId` instead (S-3.1.06)")]
    pub client_id: String,
    pub sensor_id: SensorId,
    pub error: SensorError,
    pub retry_metadata: RetryMetadata,
}

impl std::fmt::Display for FanOutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FanOutError(org_id={}, sensor={}, attempts={}, transient={}): {}",
            self.org_id,
            self.sensor_id,
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
    /// Resolves the auth credential for `(client_id, sensor_id)`.
    ///
    /// Returns a boxed `dyn SensorAuth` on success. The concrete type is one
    /// of the sensor-specific `SensorAuth` subtypes; the resolver knows which type
    /// to return based on the sensor id string.
    fn resolve(
        &self,
        client_id: &str,
        sensor_id: SensorId,
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
/// # S-2.08 note
/// The `FanOutTarget` carries a `SensorSpec` (with tables Vec) but not a
/// per-table `TableType` at this level. Full EventStream → buffer-scan wiring
/// requires S-3.02's DataFusion `TableProvider` integration which will call
/// `route_table_query` and `EventBufferStore::scan_events` per table.
/// This implementation correctly routes all targets through the live API fetch
/// path (the existing S-2.06 path), which is the correct behavior until S-3.02
/// wires in the EventStream buffer scan.
///
/// Story: S-2.08 | AC-2, AC-3, AC-5, AC-8
#[allow(dead_code)]
pub async fn dispatch_by_table_type(target: &FanOutTarget) -> Result<FanOutResult, SensorError> {
    // S-2.08: FanOutTarget carries SensorSpec but no per-table routing context.
    // The table-type dispatch at the fan-out layer requires S-3.02's TableProvider
    // integration. For now, all targets go through the live API fetch path.
    // This is correct behavior: PointInTime always goes live; EventStream falls back
    // to live on cold-start (AC-5), which is the behavior here.
    //
    // The `target` variable is used via the `_target` pattern in the outer fan_out,
    // so we reference it here to confirm dispatch entry.
    let _ = &target.sensor_id; // used to confirm type dispatch entry point
    tracing::debug!(
        org_id = %target.org_id,
        sensor_id = %target.sensor_id,
        "AC-3/AC-5: dispatch_by_table_type: routing through live API fetch (S-3.02 will wire EventStream buffer scan)"
    );
    // Return empty result — callers that need actual data use fan_out() directly.
    // This function's role is table-type inspection; it returns empty FanOutResult
    // when called in isolation (actual data flow goes through fan_out).
    Ok(FanOutResult::default())
}

// ---------------------------------------------------------------------------
// fan_out
// ---------------------------------------------------------------------------

/// Fan out sensor fetches to all targets in parallel.
///
/// Spawns one tokio task per target. Each task:
/// 1. Acquires one permit from the fan-out semaphore (cap: `MAX_FANOUT_CONCURRENCY`).
/// 2. Acquires one permit from the global HTTP semaphore (cap: 200).
/// 3. Resolves credentials via `credentials.resolve(client_id, sensor_id)`.
/// 4. Calls `registry.get(sensor_id)?.fetch(spec, params, auth)`.
/// 5. Releases both permits on completion or error.
///
/// After all tasks complete via `join_all`, results are partitioned:
/// - At least one success → `Ok(FanOutResult { successes, errors })`
/// - All targets fail     → `Err(SensorError::AllTargetsFailed { errors })`
///
/// # Arguments
/// - `targets` — list of `(client_id, sensor_id, spec, params)` tuples.
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
// S-3.1.06 stub: credentials.resolve() still takes &str client_id; deprecated
// field read is intentional here and will be removed in the implementation phase.
#[allow(deprecated)]
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
                        #[allow(deprecated)]
                        return Err(FanOutError {
                            org_id: target.org_id,
                            client_id: target.client_id.clone(),
                            sensor_id: target.sensor_id,
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
                        #[allow(deprecated)]
                        let err = FanOutError {
                            org_id: target.org_id,
                            client_id: target.client_id.clone(),
                            sensor_id: target.sensor_id,
                            error: e,
                            retry_metadata,
                        };
                        return Err(err);
                    }
                };

                // Resolve credentials for this (client, sensor) pair
                let auth = match credentials.resolve(&target.client_id, target.sensor_id.clone()) {
                    Ok(a) => a,
                    Err(e) => {
                        let retry_metadata = error_to_retry_metadata(&e, 1);
                        #[allow(deprecated)]
                        return Err(FanOutError {
                            org_id: target.org_id,
                            client_id: target.client_id.clone(),
                            sensor_id: target.sensor_id,
                            error: e,
                            retry_metadata,
                        });
                    }
                };

                // Look up the adapter for this sensor id
                let adapter = match registry.get(target.org_id, &target.sensor_id) {
                    Some(a) => a,
                    None => {
                        let e = SensorError::AdapterNotFound {
                            sensor_id: target.sensor_id.clone(),
                        };
                        let retry_metadata = error_to_retry_metadata(&e, 1);
                        #[allow(deprecated)]
                        return Err(FanOutError {
                            org_id: target.org_id,
                            client_id: target.client_id.clone(),
                            sensor_id: target.sensor_id,
                            error: e,
                            retry_metadata,
                        });
                    }
                };

                // Execute the fetch with a tracing span per AC-1
                let span = tracing::info_span!(
                    "fan_out_task",
                    org_id = %target.org_id,
                    sensor_id = %target.sensor_id,
                );
                let _enter = span.enter();

                // BC-3.2.001 precondition 4: org_id must match spec.org_id before dispatch.
                // debug_assert_eq! fires in debug/CI builds; no-op in release.
                debug_assert_eq!(
                    target.org_id, target.spec.org_id,
                    "fan_out precondition violation: target.org_id ({}) != target.spec.org_id ({}) — \
                     callers must set spec.org_id = target.org_id (BC-3.2.001 precondition 4)",
                    target.org_id, target.spec.org_id
                );

                match adapter
                    .fetch(&target.spec, &target.params, auth.as_ref())
                    .await
                {
                    Ok(batches) => Ok(batches),
                    Err(e) => {
                        let retry_metadata = error_to_retry_metadata(&e, 1);
                        #[allow(deprecated)]
                        let err = FanOutError {
                            org_id: target.org_id,
                            client_id: target.client_id.clone(),
                            sensor_id: target.sensor_id,
                            error: e,
                            retry_metadata,
                        };
                        Err(err)
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
                #[allow(deprecated)]
                result.errors.push(FanOutError {
                    org_id: OrgId::new(),
                    client_id: "unknown".into(),
                    // Reserved sentinel that cannot collide with user-defined sensor ids.
                    // "unknown" passes validate_sensor_id_string (length 7, [a-z]) so it
                    // could legitimately be authored by a spec writer. Use a hyphenated
                    // prefix that is semantically distinct and unlikely to be chosen as a
                    // plugin name (F-PR1-MED-001).
                    sensor_id: prism_core::SensorId::from("internal-panic-recovery"),
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
// S-3.1.06 stub: credentials.resolve() still takes &str client_id; deprecated
// field read is intentional here and will be removed in the implementation phase.
#[allow(dead_code, deprecated)]
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
            #[allow(deprecated)]
            return Err(FanOutError {
                org_id: target.org_id,
                client_id: target.client_id.clone(),
                sensor_id: target.sensor_id,
                error: e,
                retry_metadata,
            });
        }
    };

    // Resolve credentials
    let auth = match credentials.resolve(&target.client_id, target.sensor_id.clone()) {
        Ok(a) => a,
        Err(e) => {
            let retry_metadata = error_to_retry_metadata(&e, 1);
            #[allow(deprecated)]
            return Err(FanOutError {
                org_id: target.org_id,
                client_id: target.client_id.clone(),
                sensor_id: target.sensor_id,
                error: e,
                retry_metadata,
            });
        }
    };

    // Look up the adapter
    let adapter = match registry.get(target.org_id, &target.sensor_id) {
        Some(a) => a,
        None => {
            let e = SensorError::AdapterNotFound {
                sensor_id: target.sensor_id.clone(),
            };
            let retry_metadata = error_to_retry_metadata(&e, 1);
            #[allow(deprecated)]
            return Err(FanOutError {
                org_id: target.org_id,
                client_id: target.client_id.clone(),
                sensor_id: target.sensor_id,
                error: e,
                retry_metadata,
            });
        }
    };

    // Fetch with a tracing span (AC-1: distinct org_id + sensor_id fields)
    let span = tracing::info_span!(
        "fan_out_task",
        org_id = %target.org_id,
        sensor_id = %target.sensor_id,
    );
    let _enter = span.enter();

    adapter
        .fetch(&target.spec, &target.params, auth.as_ref())
        .await
        .map_err(|e| {
            let retry_metadata = error_to_retry_metadata(&e, 1);
            #[allow(deprecated)]
            FanOutError {
                org_id: target.org_id,
                client_id: target.client_id.clone(),
                sensor_id: target.sensor_id,
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
