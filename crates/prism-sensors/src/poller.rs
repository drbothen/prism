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
    // Stub: all fields are consumed by `run()` once implemented.
    #[allow(dead_code)]
    poll_interval: Duration,
    #[allow(dead_code)]
    retention: Duration,
    #[allow(dead_code)]
    buffer: Arc<EventBufferStore>,
    #[allow(dead_code)]
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
    pub async fn run(self) {
        todo!("AC-1 / AC-4 / AC-6: implement poll loop: sleep poll_interval, fetch from SensorAdapter, write to EventBufferStore, evict_expired, handle API errors with WARN log and loop continuation; poller_id={}", self.id)
    }

    /// Returns the current diagnostics snapshot for this poller.
    ///
    /// Called periodically by the diagnostics subsystem (Task 8).
    pub fn diagnostics(&self) -> PollerDiagnostics {
        todo!("Task-8: return PollerDiagnostics snapshot with status, last_poll_time_secs, last_poll_record_count, event_buffer_size_bytes; poller_id={}", self.id)
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
pub fn start_pollers(
    _buffer: Arc<EventBufferStore>,
    _cancel: CancellationToken,
    max_concurrency: usize,
) -> Vec<PollerId> {
    todo!("AC-1: implement poller spawning from spec entries; iterate event-stream tables, spawn EventPoller tasks, return PollerId list; max_concurrency={max_concurrency}")
}
