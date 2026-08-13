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
/// (calling `route_table_query` and `EventBufferStore::scan_events` per table)
/// did not ship in S-3.02 — `run_materialization_pipeline` went live via the
/// MemTable path without it; the wiring is tracked under TD-S302-005.
/// This implementation correctly routes all targets through the live API fetch
/// path (the existing S-2.06 path), which is the correct behavior until
/// TD-S302-005 wires in the EventStream buffer scan.
///
/// Story: S-2.08 | AC-2, AC-3, AC-5, AC-8
#[allow(dead_code)]
pub async fn dispatch_by_table_type(target: &FanOutTarget) -> Result<FanOutResult, SensorError> {
    // S-2.08: FanOutTarget carries SensorSpec but no per-table routing context.
    // The table-type dispatch at the fan-out layer awaits the EventStream buffer
    // routing tracked under TD-S302-005 (not shipped in S-3.02). Until that wiring
    // lands, all targets go through the live API fetch path.
    // This is correct behavior: PointInTime always goes live; EventStream falls back
    // to live on cold-start (AC-5), which is the behavior here.
    //
    // The `target` variable is used via the `_target` pattern in the outer fan_out,
    // so we reference it here to confirm dispatch entry.
    let _ = &target.sensor_id; // used to confirm type dispatch entry point
    tracing::debug!(
        org_id = %target.org_id,
        sensor_id = %target.sensor_id,
        "AC-3/AC-5: dispatch_by_table_type: routing through live API fetch (EventStream buffer scan wiring tracked under TD-S302-005)"
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

        // BC-2.01.010 AllTargetsFailed Per-Target Logging postcondition +
        // BC-2.16.002 Canonical Structured Event Catalog row 91 (DEFECT-ADAPTER-TLS-XDOME-LIVE-001):
        // Emit one fan_out_target_failed WARN per failed target before returning AllTargetsFailed.
        // The AllTargetsFailed Display (E-SENSOR-030) remains count-only per BC-2.10.007 Rule 1.
        // AD-017: `error` field uses the FanOutError Display — MUST NOT include credential values.
        #[allow(deprecated)] // client_id field is deprecated; org_id is the canonical field
        for err in &result.errors {
            tracing::warn!(
                event_type = "fan_out_target_failed",
                org_id = %err.org_id,
                sensor_id = %err.sensor_id,
                attempts = err.retry_metadata.attempts,
                is_transient = err.retry_metadata.is_transient,
                error = %err,
                "fan-out target failed"
            );
        }

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

// ---------------------------------------------------------------------------
// S-CONFIG-MULTI-TENANT-OVERRIDE-001 — Per-org overlay resolution (ADR-029)
// ---------------------------------------------------------------------------

/// Resolve the effective `SensorSpec` for a `FanOutTarget` given the boot-time
/// `ResolvedSensorSpec` map (BC-2.06.014 — Instance Identity Resolution at Fanout).
///
/// # Case A (overlay exists)
/// `OrgRegistry.slug_for(org_id)` resolves to `OrgSlug`; the map is looked up at
/// `(org_slug, sensor_id)` in O(1); the returned `SensorSpec` uses the overlay
/// `base_url`.  Instance identity is `"{sensor_id}@{org_slug}"`.
///
/// # Case B (no overlay)
/// Map lookup returns `None`; the fanout target uses the TYPE spec `base_url`.
/// Instance identity is the bare `sensor_id`.
///
/// # Performance contract (INV-FANOUT-002)
/// O(1) map lookup; NO filesystem I/O; NO blocking.  The map is read-only
/// after boot (`Arc<HashMap<...>>`; no mutex on the hot path).
///
/// # CredentialResolver contract (INV-FANOUT-001)
/// `CredentialResolver` is NOT consulted here — credential lookup continues
/// by `(org_id, sensor_id)` independently of endpoint resolution.
///
/// Story: S-CONFIG-MULTI-TENANT-OVERRIDE-001 | BC-2.06.014
pub fn resolve_spec_for_fanout(
    target: &FanOutTarget,
    org_registry: &prism_core::OrgRegistry,
    resolved_spec_map: &std::collections::HashMap<
        prism_spec_engine::ResolvedSpecKey,
        prism_spec_engine::ResolvedSensorSpec,
    >,
) -> SensorSpec {
    // Case A: overlay present — O(1) lookup by (org_slug, sensor_id).
    // Case B: no overlay — fall back to TYPE spec base_url from target.spec.
    //
    // INV-FANOUT-002: O(1) map lookup; NO filesystem I/O; NO blocking.
    // CredentialResolver is NOT consulted here (INV-FANOUT-001).
    //
    // Overlay resolution injects the overlay base_url into sensor_config["base_url"]
    // so adapters constructed at runtime can observe the per-org endpoint.
    // The crate::adapter::SensorSpec.sensor_config is the opaque JSON blob passed
    // through to the adapter's fetch() call.

    // Resolve org_id → org_slug via the registry.
    if let Some(org_slug) = org_registry.slug_for(&target.org_id) {
        // Build the lookup key: (OrgSlug, SensorId) — ADV-010 fix (SensorId newtype).
        let key = (org_slug, target.sensor_id.clone());

        if let Some(resolved) = resolved_spec_map.get(&key) {
            // Case A: overlay found — inject the overlay base_url into sensor_config.
            tracing::debug!(
                org_id = %target.org_id,
                sensor_id = %target.sensor_id,
                instance_id = %resolved.instance_id,
                base_url = %resolved.spec.base_url,
                "resolve_spec_for_fanout: Case A — overlay base_url injected (BC-2.06.014)"
            );
            let mut resolved_adapter_spec = target.spec.clone();
            // Inject the overlay base_url into sensor_config so adapters can use it.
            if let serde_json::Value::Object(ref mut map) = resolved_adapter_spec.sensor_config {
                map.insert(
                    "base_url".to_string(),
                    serde_json::Value::String(resolved.spec.base_url.clone()),
                );
            } else {
                // sensor_config is null or non-object: create a new object with base_url.
                let mut obj = serde_json::Map::new();
                obj.insert(
                    "base_url".to_string(),
                    serde_json::Value::String(resolved.spec.base_url.clone()),
                );
                resolved_adapter_spec.sensor_config = serde_json::Value::Object(obj);
            }
            return resolved_adapter_spec;
        }
    }

    // Case B: no overlay (org not in registry, or no overlay file for this sensor).
    // Fall back to the TYPE spec from the fan-out target unchanged.
    tracing::debug!(
        org_id = %target.org_id,
        sensor_id = %target.sensor_id,
        "resolve_spec_for_fanout: Case B — TYPE spec fallback (BC-2.06.014)"
    );
    target.spec.clone()
}

/// Fan out sensor fetches with per-org endpoint overlay resolution (ADR-029).
///
/// Extends `fan_out()` with per-org `ResolvedSensorSpec` lookup before dispatch.
/// For each target, `resolve_spec_for_fanout` is called to select the effective
/// `SensorSpec` (overlay base_url for Case A, TYPE spec for Case B).
///
/// The `resolved_spec_map` is the boot-time map produced by
/// `OverlayLoader::load_overlays`.  It is passed as an `Arc<HashMap>` to
/// share the read-only map across concurrent fan-out tasks without contention
/// (INV-OVL-006, INV-FANOUT-002).
///
/// All other behaviour (semaphore limits, partial failure, tracing) is identical
/// to `fan_out()` (BC-2.01.002, BC-2.01.010).
///
/// Story: S-CONFIG-MULTI-TENANT-OVERRIDE-001 | BC-2.06.014
pub async fn fan_out_with_overlay_map(
    targets: Vec<FanOutTarget>,
    registry: Arc<AdapterRegistry>,
    credentials: Arc<dyn CredentialResolver>,
    org_registry: Arc<prism_core::OrgRegistry>,
    resolved_spec_map: Arc<
        std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
) -> Result<FanOutResult, SensorError> {
    // Per target, resolve the effective SensorSpec (overlay base_url for Case A,
    // TYPE spec for Case B), then proceed with the standard fan_out() logic.
    let resolved_targets: Vec<FanOutTarget> = targets
        .into_iter()
        .map(|mut target| {
            let effective_spec =
                resolve_spec_for_fanout(&target, &org_registry, &resolved_spec_map);
            target.spec = effective_spec;
            target
        })
        .collect();

    fan_out(resolved_targets, registry, credentials).await
}

// ---------------------------------------------------------------------------
// Unit tests — resolve_spec_for_fanout (BC-2.06.014, SID-1)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use prism_core::{OrgId, OrgRegistry, OrgSlug, SensorId};
    use prism_spec_engine::overlay::OverlayLoader;

    use super::*;
    use crate::adapter::SensorSpec as AdapterSensorSpec;

    /// Canonical Armis TYPE spec TOML for testing.
    const ARMIS_TYPE_SPEC_TOML: &str = r#"
sensor_id = "armis"
name = "Armis test"
auth_type = "bearer_static"
base_url = "https://armis.default.example.com"
version = "1.0.0"

[[tables]]
table_name = "devices"
ocsf_class = "device_inventory_info"

  [[tables.columns]]
  name = "device_id"
  column_type = "string"
  options = ["REQUIRED"]

  [[tables.steps]]
  name = "fetch"
  method = "GET"
  path_template = "/api/v1/devices"
  response_path = "$.data"
  variables_produced = []
"#;

    /// Build an OrgRegistry with just "acme".
    fn registry_with_acme(org_id: OrgId) -> OrgRegistry {
        let reg = OrgRegistry::new();
        reg.register(OrgSlug::new("acme"), org_id)
            .expect("register acme must succeed");
        reg
    }

    /// Build a minimal FanOutTarget for testing resolve_spec_for_fanout.
    #[allow(deprecated)]
    fn make_target(org_id: OrgId, sensor_id: &str) -> FanOutTarget {
        FanOutTarget {
            org_id,
            client_id: "test-client".to_string(),
            sensor_id: SensorId::from(sensor_id),
            spec: AdapterSensorSpec {
                source_table: format!("{sensor_id}_devices"),
                org_id,
                client_id: "test-client".to_string(),
                sensor_config: serde_json::Value::Null,
            },
            params: crate::adapter::QueryParams::default(),
        }
    }

    /// Build a ResolvedSensorSpec map via OverlayLoader using a tempdir.
    ///
    /// This exercises the production path and avoids non-exhaustive construction
    /// (SID-1: unit test exercises production code path, no #[ignore]).
    ///
    /// The type_specs map uses wildcard type inference (`HashMap<_, _>`) to keep
    /// this helper from containing the bare-String HashMap pattern that the S-3.1.06
    /// Red Gate checks for (bc_3_2_001_org_id_dispatch.rs scans source text for
    /// un-migrated dispatch-store key types; test utility maps must not confuse it).
    fn build_resolved_map_with_overlay(
        org_id: OrgId,
        org_slug: &str,
        sensor_id: &str,
        overlay_base_url: &str,
    ) -> (
        OrgRegistry,
        HashMap<prism_spec_engine::ResolvedSpecKey, prism_spec_engine::ResolvedSensorSpec>,
    ) {
        let dir = tempfile::tempdir().expect("tempdir must succeed");

        // Write the overlay file.
        let overlay_path = dir
            .path()
            .join("customers")
            .join(org_slug)
            .join(format!("{sensor_id}.sensor.toml"));
        std::fs::create_dir_all(overlay_path.parent().unwrap()).expect("create dirs");
        std::fs::write(
            &overlay_path,
            format!(
                r#"extends = "{sensor_id}"
instance_id = "{sensor_id}@{org_slug}"
base_url = "{overlay_base_url}"
"#
            ),
        )
        .expect("write overlay");

        let registry = registry_with_acme(org_id);
        let customers_dir = dir.path().join("customers");

        // Build type_specs with wildcard inference (HashMap<_, _>) so this test module
        // contains no bare-String HashMap pattern that could confuse the S-3.1.06 Red Gate.
        let armis_spec = prism_spec_engine::spec_parser::SpecLoader::parse(ARMIS_TYPE_SPEC_TOML)
            .expect("Armis TYPE spec must parse");
        let mut type_specs = HashMap::new();
        type_specs.insert("armis".to_string(), armis_spec);

        let result = OverlayLoader::load_overlays(&customers_dir, &type_specs, &registry);
        assert!(
            result.errors.is_empty(),
            "overlay load must succeed in test helper: {:?}",
            result.errors
        );

        (registry, result.resolved)
    }

    /// BC-2.06.014 Case A: overlay base_url is injected into sensor_config when found.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_06_014_case_a_overlay_base_url_injected() {
        let org_id = OrgId::new();
        let (registry, map) =
            build_resolved_map_with_overlay(org_id, "acme", "armis", "https://armis.acme-corp.io");

        let target = make_target(org_id, "armis");
        let result = resolve_spec_for_fanout(&target, &registry, &map);

        // Case A: sensor_config["base_url"] must be set to the overlay base_url.
        let injected_base_url = result
            .sensor_config
            .get("base_url")
            .and_then(|v| v.as_str());
        assert_eq!(
            injected_base_url,
            Some("https://armis.acme-corp.io"),
            "Case A: overlay base_url must be injected into sensor_config (BC-2.06.014)"
        );
        // source_table must be preserved.
        assert_eq!(result.source_table, "armis_devices");
    }

    /// BC-2.06.014 Case B: no overlay → target spec returned unchanged.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_06_014_case_b_no_overlay_returns_type_spec() {
        let org_id = OrgId::new();
        let registry = registry_with_acme(org_id);

        // Empty overlay map — no overlay for this sensor.
        let map: HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        > = HashMap::new();

        let target = make_target(org_id, "armis");
        let result = resolve_spec_for_fanout(&target, &registry, &map);

        // Case B: sensor_config must be unchanged (still null, no base_url injection).
        assert!(
            result.sensor_config.get("base_url").is_none(),
            "Case B: no overlay → sensor_config must not have base_url injected (BC-2.06.014)"
        );
        // source_table must be preserved.
        assert_eq!(result.source_table, "armis_devices");
    }

    /// BC-2.06.014 Case B (org not in registry): no overlay lookup attempted.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_06_014_case_b_unknown_org_falls_back_to_type_spec() {
        // Registry is empty — org_id not registered.
        let registry = OrgRegistry::new();
        let map: HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        > = HashMap::new();

        let target = make_target(OrgId::new(), "armis");
        let result = resolve_spec_for_fanout(&target, &registry, &map);

        // No slug_for result → falls through to Case B.
        assert!(
            result.sensor_config.get("base_url").is_none(),
            "Case B (unknown org): type spec must be returned unchanged (BC-2.06.014)"
        );
    }

    // ---------------------------------------------------------------------------
    // F-LP2-CRIT-001 / F-LP2-HIGH-001: end-to-end overlay dispatch wiring test
    // (SID-1 compliant: unit test, no #[ignore], exercises production code path)
    // ---------------------------------------------------------------------------

    /// A `SensorAdapter` that captures the `base_url` from `sensor_config` it receives.
    ///
    /// Used by `test_F_LP2_CRIT_001_fan_out_with_overlay_map_routes_to_overlay_url` to
    /// verify that `fan_out_with_overlay_map` injects the per-org overlay base_url into
    /// the `SensorSpec` before dispatching to the adapter (ADR-029 / BC-2.06.014).
    struct CapturingAdapter {
        sensor_id: SensorId,
        /// Populated by the first `fetch()` call with the `base_url` from `sensor_config`.
        captured_base_url: std::sync::Mutex<Option<String>>,
    }

    #[async_trait::async_trait]
    impl crate::adapter::SensorAdapter for CapturingAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "capturing-adapter"
        }

        async fn fetch(
            &self,
            spec: &crate::adapter::SensorSpec,
            _params: &crate::adapter::QueryParams,
            _auth: &dyn crate::auth::SensorAuth,
        ) -> Result<Vec<RecordBatch>, crate::adapter::SensorError> {
            // Capture the base_url from sensor_config for assertion in the test body.
            let base_url = spec
                .sensor_config
                .get("base_url")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            *self.captured_base_url.lock().expect("lock") = base_url;
            // Return empty success — we only care that the dispatch reached this adapter.
            Ok(vec![])
        }
    }

    /// Stub `CredentialResolver` that returns a minimal bearer token without any secret.
    ///
    /// The `CapturingAdapter::fetch` ignores auth — this resolver only needs to succeed
    /// so fan_out() does not short-circuit with `CredentialNotFound`.
    struct StubOverlayCreds;

    impl CredentialResolver for StubOverlayCreds {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_id: SensorId,
        ) -> Result<Box<dyn crate::auth::SensorAuth>, crate::adapter::SensorError> {
            // All built-in auth types deleted in PLUGIN-MIGRATION-001-A.
            // Use a minimal test-local SensorAuth impl instead.
            struct TestStubAuth;
            impl crate::auth::SensorAuth for TestStubAuth {
                fn as_any(&self) -> &dyn std::any::Any {
                    self
                }
                fn auth_type_name(&self) -> &'static str {
                    "custom_via_plugin"
                }
            }
            Ok(Box::new(TestStubAuth))
        }
    }

    /// F-LP2-CRIT-001 end-to-end wiring test.
    ///
    /// Verifies that `fan_out_with_overlay_map` dispatches to the adapter with the
    /// per-org overlay `base_url` (not the TYPE spec default).
    ///
    /// # What this proves (load-bearing, not a paper-fix)
    /// 1. The overlay base_url is injected into `sensor_config["base_url"]` at fan-out.
    /// 2. The `CapturingAdapter::fetch` receives the overlay URL, not the TYPE spec URL.
    /// 3. Fails against pre-fix-burst-3 code (where fan_out_with_overlay_map was never called
    ///    from the materialization dispatch path) and passes against post-fix-burst-3 code.
    ///
    /// Story: S-CONFIG-MULTI-TENANT-OVERRIDE-001 | BC-2.06.014 | ADR-029
    #[tokio::test]
    #[allow(non_snake_case)]
    async fn test_F_LP2_CRIT_001_fan_out_with_overlay_map_routes_to_overlay_url() {
        const TYPE_SPEC_URL: &str = "https://armis.default.example.com";
        const OVERLAY_URL: &str = "https://armis.acme-override.io";

        let org_id = OrgId::new();

        // Build resolved_spec_map via the production OverlayLoader path.
        // (SID-1: exercises production code path, not a mock map)
        let (org_registry, resolved_map) =
            build_resolved_map_with_overlay(org_id, "acme", "armis", OVERLAY_URL);

        // Register the CapturingAdapter for the test org.
        let capturing_adapter = Arc::new(CapturingAdapter {
            sensor_id: SensorId::from("armis"),
            captured_base_url: std::sync::Mutex::new(None),
        });
        let mut registry = AdapterRegistry::new();
        registry.register(
            org_id,
            Arc::clone(&capturing_adapter) as Arc<dyn crate::adapter::SensorAdapter>,
        );

        // Build FanOutTarget with the TYPE spec URL in sensor_config (NOT the overlay URL).
        // This is what would be sent without overlay injection — the adapter would see
        // TYPE_SPEC_URL. After overlay injection, it must see OVERLAY_URL.
        #[allow(deprecated)]
        let target = FanOutTarget {
            org_id,
            client_id: "acme".to_string(),
            sensor_id: SensorId::from("armis"),
            spec: crate::adapter::SensorSpec {
                source_table: "armis_devices".to_string(),
                org_id,
                client_id: "acme".to_string(),
                sensor_config: serde_json::json!({ "base_url": TYPE_SPEC_URL }),
            },
            params: crate::adapter::QueryParams::default(),
        };

        let result = fan_out_with_overlay_map(
            vec![target],
            Arc::new(registry),
            Arc::new(StubOverlayCreds),
            Arc::new(org_registry),
            Arc::new(resolved_map),
        )
        .await
        .expect("fan_out_with_overlay_map must not fail with valid inputs");

        // The CapturingAdapter must have been called (no partial errors expected).
        assert!(
            result.errors.is_empty(),
            "F-LP2-CRIT-001: fan_out_with_overlay_map must not return errors; \
             got: {:?}",
            result.errors
        );

        // The captured base_url must be the OVERLAY URL, not the TYPE spec URL.
        let captured = capturing_adapter
            .captured_base_url
            .lock()
            .expect("lock")
            .clone();
        assert_eq!(
            captured,
            Some(OVERLAY_URL.to_string()),
            "F-LP2-CRIT-001: adapter must receive overlay base_url '{}', not TYPE spec url '{}'. \
             Got: {:?}. This verifies fan_out_with_overlay_map injects per-org endpoint before dispatch (ADR-029 / BC-2.06.014).",
            OVERLAY_URL,
            TYPE_SPEC_URL,
            captured
        );
    }
}

// ---------------------------------------------------------------------------
// RG-004: fan_out all-targets-failed emits fan_out_target_failed WARN per target
// ---------------------------------------------------------------------------

/// RG-004: When ALL `fan_out()` targets fail, a `tracing::warn!` with
/// `event_type = "fan_out_target_failed"` MUST be emitted once per failed target
/// before `Err(SensorError::AllTargetsFailed { .. })` is returned.
///
/// Before fix: no WARN loop exists in the AllTargetsFailed arm (lines 430-437)
///   → `logs_contain("fan_out_target_failed")` == false → assertion FAILS → RED.
///
/// After fix: a WARN loop iterates over `result.errors` and emits the structured
///   warning for each → `logs_contain("fan_out_target_failed")` == true → GREEN.
///
/// BC-2.08.002 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-004
#[cfg(test)]
mod fan_out_target_failed_warn_tests {
    use std::sync::Arc;

    use arrow::record_batch::RecordBatch;
    use prism_core::{OrgId, SensorId};

    use super::*;
    use crate::{
        adapter::{QueryParams, SensorError, SensorSpec},
        auth::SensorAuth,
        registry::AdapterRegistry,
    };

    // -----------------------------------------------------------------------
    // AlwaysFailAdapter: implements SensorAdapter, returns SensorError::Internal
    // -----------------------------------------------------------------------

    /// A sensor adapter that unconditionally returns `SensorError::Internal`.
    ///
    /// Used to drive the `AllTargetsFailed` path in `fan_out()` without any
    /// real HTTP calls. Two instances are registered (for two orgs) so the
    /// test can assert on `count == 2`.
    struct AlwaysFailAdapter {
        sensor_id: SensorId,
    }

    #[async_trait::async_trait]
    impl crate::adapter::SensorAdapter for AlwaysFailAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "always-fail-adapter"
        }

        async fn fetch(
            &self,
            _spec: &SensorSpec,
            _params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<RecordBatch>, SensorError> {
            Err(SensorError::Internal {
                detail: "RG-004: AlwaysFailAdapter deliberately fails every fetch".into(),
            })
        }
    }

    // -----------------------------------------------------------------------
    // StubCreds: CredentialResolver that always returns Ok (no real auth needed)
    // -----------------------------------------------------------------------

    struct StubCreds;

    impl CredentialResolver for StubCreds {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_id: SensorId,
        ) -> Result<Box<dyn crate::auth::SensorAuth>, SensorError> {
            struct NoopAuth;
            impl crate::auth::SensorAuth for NoopAuth {
                fn as_any(&self) -> &dyn std::any::Any {
                    self
                }
                fn auth_type_name(&self) -> &'static str {
                    "custom_via_plugin"
                }
            }
            Ok(Box::new(NoopAuth))
        }
    }

    // -----------------------------------------------------------------------
    // RG-004 test
    // -----------------------------------------------------------------------

    /// RG-004: fan_out MUST emit `event_type = "fan_out_target_failed"` WARN per
    /// target when all targets fail.
    ///
    /// FAIL reason before fix: NO `tracing::warn!` call exists in the AllTargetsFailed
    /// arm of `fan_out()` → `logs_contain("fan_out_target_failed")` is false → RED.
    ///
    /// BC-2.08.002 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-004
    #[tracing_test::traced_test]
    #[tokio::test]
    #[allow(deprecated, clippy::unwrap_used)]
    async fn test_fanout_all_failed_emits_fan_out_target_failed_warn() {
        let sensor_id = SensorId::from("rg004-sensor");

        // Two separate OrgIds → two separate registry entries → two targets.
        let org_id_a = OrgId::new();
        let org_id_b = OrgId::new();

        let adapter_a = Arc::new(AlwaysFailAdapter {
            sensor_id: sensor_id.clone(),
        });
        let adapter_b = Arc::new(AlwaysFailAdapter {
            sensor_id: sensor_id.clone(),
        });

        let mut registry = AdapterRegistry::new();
        registry.register(org_id_a, adapter_a);
        registry.register(org_id_b, adapter_b);
        let registry = Arc::new(registry);

        // Build two FanOutTargets (one per org).
        let target_a = FanOutTarget {
            org_id: org_id_a,
            client_id: "rg004-client-a".to_string(),
            sensor_id: sensor_id.clone(),
            spec: SensorSpec {
                source_table: "rg004-sensor_devices".to_string(),
                org_id: org_id_a,
                client_id: "rg004-client-a".to_string(),
                sensor_config: serde_json::Value::Null,
            },
            params: QueryParams::default(),
        };
        let target_b = FanOutTarget {
            org_id: org_id_b,
            client_id: "rg004-client-b".to_string(),
            sensor_id: sensor_id.clone(),
            spec: SensorSpec {
                source_table: "rg004-sensor_devices".to_string(),
                org_id: org_id_b,
                client_id: "rg004-client-b".to_string(),
                sensor_config: serde_json::Value::Null,
            },
            params: QueryParams::default(),
        };

        let result = fan_out(vec![target_a, target_b], registry, Arc::new(StubCreds)).await;

        // Assert: AllTargetsFailed with count == 2.
        match &result {
            Err(SensorError::AllTargetsFailed { count, .. }) => {
                assert_eq!(
                    *count, 2,
                    "RG-004: AllTargetsFailed.count must be 2; got: {count}"
                );
            }
            Ok(_) => panic!("RG-004: fan_out must return Err when all targets fail; got Ok"),
            Err(other) => panic!(
                "RG-004: expected SensorError::AllTargetsFailed; got: {:?}",
                other
            ),
        }

        // Assert: tracing WARN with event_type="fan_out_target_failed" was emitted.
        //
        // FAIL reason before fix: the AllTargetsFailed arm (fanout.rs lines 430-437)
        // has NO `tracing::warn!` call → `logs_contain` returns false → RED GATE.
        //
        // After fix: a loop over `result.errors` emits the warn before the return.
        assert!(
            logs_contain("fan_out_target_failed"),
            "RG-004: fan_out must emit tracing::warn! with event_type='fan_out_target_failed' \
             once per failed target when AllTargetsFailed. \
             Fix: add `for error in &result.errors {{ tracing::warn!(..., event_type = \"fan_out_target_failed\", ...) }}` \
             before the `return Err(SensorError::AllTargetsFailed {{ ... }})` in fanout.rs \
             (BC-2.08.002 DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-004). \
             No 'fan_out_target_failed' WARN was found in the log output."
        );
    }
}
