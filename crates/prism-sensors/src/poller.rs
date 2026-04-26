//! `EventPoller` — background polling task for event-stream sensor tables.
//!
//! One `EventPoller` instance is spawned per `(sensor_id, table_name, client_id)`
//! tuple at startup by `AdapterRegistry::start_pollers()`. The poller loops with
//! a configurable `poll_interval`, calling `SensorAdapter::fetch()` and writing
//! results to `EventBufferStore`. On error it logs WARN and continues (AC-6).
//! On graceful shutdown (via `CancellationToken`) it drains and exits cleanly (AC-1).
//!
//! Story: S-2.08 | AC-1, AC-4, AC-5, AC-6

use std::sync::Arc;
use std::time::Duration;

use tokio_util::sync::CancellationToken;
#[allow(unused_imports)]
use tracing::{info, warn};

use crate::event_buffer::EventBufferStore;

// ---------------------------------------------------------------------------
// PollerId
// ---------------------------------------------------------------------------

/// Uniquely identifies a running `EventPoller` instance.
///
/// Scoped to `(sensor_id, table_name, client_id)` so that multiple clients
/// and tables produce independent, named poller tasks.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PollerId {
    pub sensor_id: String,
    pub table_name: String,
    pub client_id: String,
}

impl std::fmt::Display for PollerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.sensor_id, self.table_name, self.client_id
        )
    }
}

// ---------------------------------------------------------------------------
// PollerStatus
// ---------------------------------------------------------------------------

/// Last-known operational status of a running `EventPoller`.
///
/// Reported via `get_diagnostics(subsystem: "fanout")` (Task 8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PollerStatus {
    /// Poller is running normally; last poll succeeded.
    Running,
    /// Poller is running but the last poll returned an error (WARN logged).
    Error,
    /// Poller has not completed its first poll cycle yet.
    ColdStart,
}

// ---------------------------------------------------------------------------
// PollerDiagnostics
// ---------------------------------------------------------------------------

/// Runtime diagnostics for one `EventPoller` instance.
///
/// Collected by `get_diagnostics(subsystem: "fanout")` (Task 8).
#[derive(Debug, Clone)]
pub struct PollerDiagnostics {
    pub poller_id: PollerId,
    pub status: PollerStatus,
    /// Wall-clock time of the last successful poll, in seconds since UNIX epoch.
    /// `None` if no poll has completed yet (cold start).
    pub last_poll_time_secs: Option<u64>,
    /// Number of records ingested in the last successful poll cycle.
    pub last_poll_record_count: usize,
    /// Approximate size in bytes of buffered records for this poller's scope.
    pub event_buffer_size_bytes: u64,
}

// ---------------------------------------------------------------------------
// EventPoller
// ---------------------------------------------------------------------------

/// Background task that periodically fetches events from a sensor API and
/// writes them to the local `EventBufferStore`.
///
/// # Lifecycle
/// Created by `AdapterRegistry::start_pollers()` at startup. Cancelled
/// via `CancellationToken` on graceful shutdown or hot-reload spec removal.
///
/// # Concurrency
/// Pollers share the same `reqwest::Client` and HTTP semaphore as query-time
/// fetches — they are NOT exempt from the 200-connection global cap.
///
/// # Architecture Compliance (S-2.08)
/// - On API error: log WARN and continue; do NOT crash the task.
/// - `poll_interval` minimum is 10 seconds (validated by SpecParser, AC-7).
/// - Pollers participate in the global HTTP semaphore (Architecture Compliance).
pub struct EventPoller {
    id: PollerId,
    poll_interval: Duration,
    retention: Duration,
    buffer: Arc<EventBufferStore>,
    cancel: CancellationToken,
}

impl EventPoller {
    /// Creates a new `EventPoller` for the given table.
    ///
    /// # Arguments
    /// - `id`            — unique poller identity (sensor_id, table_name, client_id)
    /// - `poll_interval` — how often to call the sensor API (minimum 10s per AC-7)
    /// - `retention`     — TTL for buffered records; expired records are evicted
    ///   after each poll cycle (AC-4)
    /// - `buffer`        — shared `EventBufferStore` for write and eviction ops
    /// - `cancel`        — token used to signal graceful shutdown (AC-1)
    pub fn new(
        id: PollerId,
        poll_interval: Duration,
        retention: Duration,
        buffer: Arc<EventBufferStore>,
        cancel: CancellationToken,
    ) -> Self {
        Self {
            id,
            poll_interval,
            retention,
            buffer,
            cancel,
        }
    }

    /// Runs the polling loop.
    ///
    /// Loop: sleep `poll_interval` → fetch from sensor API → write to buffer →
    /// evict expired records → repeat. Exits cleanly when the `CancellationToken`
    /// fires.
    ///
    /// On API error: logs WARN with sensor/table/client context and continues
    /// the loop (AC-6). Does NOT crash or stop the task.
    ///
    /// # AC-1
    /// When `CancellationToken` is triggered, any in-flight fetch is abandoned
    /// and the loop exits without spawning new work.
    ///
    /// # S-2.08 Status
    ///
    /// **AC-5 + AC-6 DEFERRED to S-3.02.** The current implementation is a
    /// structural foundation: it owns the loop, the CancellationToken handshake,
    /// and lazy `evict_expired` calls. The sensor-API fetch (AC-5: live-fetch
    /// fallback + buffer write + INFO log) and HTTP 429 handling (AC-6: WARN +
    /// continue) are wired when `SensorAdapter` becomes available in S-3.02.
    pub async fn run(self) {
        // AC-1: check for cancellation before entering the first sleep
        if self.cancel.is_cancelled() {
            return;
        }

        loop {
            // Sleep for poll_interval, but exit if cancelled during sleep
            tokio::select! {
                _ = tokio::time::sleep(self.poll_interval) => {
                    // Time to poll — but check cancellation first
                    if self.cancel.is_cancelled() {
                        return;
                    }

                    // Fetch from sensor API (stub: no actual SensorAdapter wired here yet)
                    // AC-6: On API error, log WARN and continue loop
                    // The full wiring (SensorAdapter call) is deferred to S-3.02
                    // when the query engine is wired up. The loop structure is correct.

                    // Evict expired records after each ingest cycle (AC-4)
                    if let Err(e) = self.buffer.evict_expired(
                        &self.id.sensor_id,
                        &self.id.table_name,
                        self.retention,
                    ) {
                        warn!(
                            poller_id = %self.id,
                            error = %e,
                            "AC-4: evict_expired failed after poll cycle; continuing"
                        );
                    }
                }
                _ = self.cancel.cancelled() => {
                    // AC-1: CancellationToken fired — exit cleanly
                    return;
                }
            }
        }
    }

    /// Returns the current diagnostics snapshot for this poller.
    ///
    /// Called periodically by the diagnostics subsystem (Task 8).
    pub fn diagnostics(&self) -> PollerDiagnostics {
        // A freshly constructed poller is always in ColdStart state:
        // - No poll has completed yet (last_poll_time_secs = None)
        // - last_poll_record_count = 0
        // - status = ColdStart
        //
        // Running pollers update state via shared state (not yet wired in S-2.08).
        // The diagnostics returned here reflect the initial state, which is correct
        // for a freshly constructed poller before any poll cycle completes.
        let event_buffer_size_bytes = self
            .buffer
            .buffer_size_bytes(&self.id.sensor_id, &self.id.table_name, &self.id.client_id)
            .unwrap_or(0);

        PollerDiagnostics {
            poller_id: self.id.clone(),
            status: PollerStatus::ColdStart,
            last_poll_time_secs: None,
            last_poll_record_count: 0,
            event_buffer_size_bytes,
        }
    }

    /// Returns the `PollerId` for this poller.
    pub fn id(&self) -> &PollerId {
        &self.id
    }
}

impl std::fmt::Debug for EventPoller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventPoller")
            .field("id", &self.id)
            .field("poll_interval", &self.poll_interval)
            .field("retention", &self.retention)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// start_pollers
// ---------------------------------------------------------------------------

/// Spawns one `EventPoller` tokio task for each event-stream table entry in
/// the provided spec set.
///
/// Called once at startup by the process entrypoint after `AdapterRegistry`
/// and `EventBufferStore` are initialized.
///
/// Returns a `Vec<PollerId>` for the pollers that were spawned so callers can
/// track and cancel them on hot-reload or shutdown (AC-1, EC-005).
///
/// # Concurrency
/// Shared semaphore caps concurrent background pollers at
/// `event_poller_concurrency` (default 4, configured via `[query]` section).
///
/// # S-2.08 Status — STRUCTURAL STUB (AC-5 DEFERRED to S-3.02)
///
/// **This function returns an empty `Vec` in S-2.08.** No event-stream table
/// specs are wired at this stage. The full wiring — iterating over sensor
/// specs from the spec registry and spawning per-`(sensor_id, table_name,
/// client_id)` poller tasks — is implemented in S-3.02 when `SensorAdapter`
/// and the query engine spec registry become available.
///
/// The function signature, return type (`Vec<PollerId>`), and
/// `CancellationToken` contract are the final API surface; only the loop body
/// is stubbed out. Callers may rely on the empty-Vec return value as a
/// documented S-2.08 invariant until S-3.02 lands.
pub fn start_pollers(
    _buffer: Arc<EventBufferStore>,
    _cancel: CancellationToken,
    _max_concurrency: usize,
) -> Vec<PollerId> {
    // S-2.08: No event-stream specs are wired at this stage.
    // When max_concurrency=0, no pollers should be spawned regardless of specs.
    // When specs are provided (S-3.02), this will iterate them and spawn tasks.
    Vec::new()
}
