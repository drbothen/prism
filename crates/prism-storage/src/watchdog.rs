// S-2.02 — ResourceWatchdog: graduated memory levels, query termination, status reporting.
//
// Implements BC-2.15.006 (watchdog initialisation), BC-2.15.007 (query termination),
// and the `watchdog_status` tool data source (Task 7).
//
// Memory measurement uses the `sysinfo` crate (cross-platform).
// Do NOT call /proc/self/status directly (Architecture Compliance Rule).
//
// Concurrent query token registry uses `DashMap<QueryId, CancellationToken>` so that
// the Kill level cancels ALL registered tokens, not just the newest (EC-003 / AC-4).
//
// The `watchdog` Cargo feature enables the runtime dependencies (sysinfo, dashmap,
// tokio-util).  Type declarations are always compiled so downstream crates can name
// the types without enabling the feature.

use prism_core::PrismError;

use crate::denylist::DenylistEntry;

// ── QueryId ─────────────────────────────────────────────────────────────────

/// Opaque identifier for a running query registered with the watchdog.
///
/// A new `QueryId` is allocated per query execution via `QueryId::new()`.
/// Used as the key in the `DashMap<QueryId, CancellationToken>` registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QueryId(pub u64);

impl QueryId {
    /// Allocate a new monotonically increasing query ID.
    ///
    /// Uses an atomic counter so `new()` is safe to call from multiple threads.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        QueryId(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for QueryId {
    fn default() -> Self {
        Self::new()
    }
}

// ── WatchdogLevel ────────────────────────────────────────────────────────────

/// Graduated resource level returned by `ResourceWatchdog::current_level()`.
///
/// Level thresholds (fraction of `budget_bytes`, BC-2.15.006):
///
/// | Level    | Threshold       |
/// |----------|-----------------|
/// | Normal   | RSS < 70%       |
/// | Warn     | 70% ≤ RSS < 85% |
/// | Throttle | 85% ≤ RSS < 95% |
/// | Kill     | RSS ≥ 95%       |
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WatchdogLevel {
    /// RSS is below the warn threshold (< 70% of budget). Normal operation.
    Normal,
    /// RSS is at or above the warn threshold (≥ 70% of budget). Log warning.
    Warn,
    /// RSS is at or above the throttle threshold (≥ 85% of budget). Reduce parallelism.
    Throttle,
    /// RSS is at or above the kill threshold (≥ 95% of budget). Cancel all queries.
    Kill,
}

// ── WatchdogStatus ───────────────────────────────────────────────────────────

/// Snapshot of watchdog state returned by `get_watchdog_status()`.
///
/// Exposed via the `watchdog_status` MCP tool (gated behind the `runtime`
/// capability at the MCP tool layer — not enforced here per Dev Notes).
#[derive(Debug, Clone)]
pub struct WatchdogStatus {
    /// Current graduated resource level.
    pub level: WatchdogLevel,
    /// Configured memory budget in bytes (default 512 MiB).
    pub budget_bytes: usize,
    /// Current process RSS in bytes (read via `sysinfo`).
    pub current_bytes: usize,
    /// Current denylist entries (from the `watchdog` CF).
    pub denylist: Vec<DenylistEntry>,
}

// ── ResourceWatchdog ─────────────────────────────────────────────────────────

/// Resource watchdog for the Prism process.
///
/// Monitors process RSS and cancels registered query `CancellationToken`s when
/// the `Kill` threshold is reached.  Spawns a background tokio task that polls
/// every 500 ms (BC-2.15.007).
///
/// # Architecture Compliance
///
/// - Memory measurement via `sysinfo` (cross-platform; never `/proc/self/status`).
/// - Concurrent token registry via `DashMap<QueryId, CancellationToken>` so ALL
///   registered tokens are cancelled on Kill, not just the newest (EC-003).
/// - Uses `RocksStorageBackend` trait (not `RocksDbBackend` directly) for denylist
///   access — enables `InMemoryBackend` injection in tests.
pub struct ResourceWatchdog {
    /// Fraction of `budget_bytes` at which level transitions to `Warn` (0.70).
    pub warn_pct: f64,
    /// Fraction of `budget_bytes` at which level transitions to `Throttle` (0.85).
    pub throttle_pct: f64,
    /// Fraction of `budget_bytes` at which level transitions to `Kill` (0.95).
    pub kill_pct: f64,
    /// Configured process memory budget in bytes (default 512 MiB = 512 * 1024 * 1024).
    pub budget_bytes: usize,
}

impl ResourceWatchdog {
    /// Construct a `ResourceWatchdog` with the default Normal-level thresholds
    /// and the 512 MiB process budget.
    ///
    /// BC-2.15.006 postcondition: thresholds are set at construction; the
    /// watchdog cannot be disabled.
    pub fn new() -> Self {
        // AC-3: thresholds 0.70 / 0.85 / 0.95 with 512 MiB budget (BC-2.15.006)
        ResourceWatchdog {
            warn_pct: 0.70,
            throttle_pct: 0.85,
            kill_pct: 0.95,
            budget_bytes: 512 * 1024 * 1024,
        }
    }

    /// Read the current process RSS via `sysinfo` and return the graduated level.
    ///
    /// **Postcondition (BC-2.15.006):** returns `WatchdogLevel` based on RSS
    /// relative to `budget_bytes` using thresholds `warn_pct`, `throttle_pct`,
    /// `kill_pct`.
    ///
    /// AC-3: RSS at 86% → `WatchdogLevel::Throttle`.
    pub fn current_level(&self) -> WatchdogLevel {
        // BC-2.15.006 postcondition: read RSS via sysinfo (not /proc/self/status)
        // and map to WatchdogLevel based on thresholds
        todo!("BC-2.15.006 postcondition: read process RSS via sysinfo::System::new_all(), compare against budget_bytes * thresholds, return WatchdogLevel")
    }

    /// Register a query's `CancellationToken` with the watchdog.
    ///
    /// Called at query start.  The watchdog background task will cancel the
    /// token if `current_level()` reaches `Kill`.
    ///
    /// Returns the allocated `QueryId` used to deregister the token on
    /// query completion.
    pub fn register_query(
        &self,
        #[allow(unused_variables)] cancel_token: tokio_util::sync::CancellationToken,
    ) -> QueryId {
        // BC-2.15.007: register cancel_token in DashMap registry; return QueryId
        todo!("BC-2.15.007: insert cancel_token into DashMap<QueryId, CancellationToken> registry; return allocated QueryId")
    }

    /// Deregister a query's `CancellationToken` after the query completes
    /// (success or failure).
    pub fn deregister_query(&self, id: QueryId) {
        // BC-2.15.007: remove cancel_token from DashMap registry on query completion
        let _ = id;
        todo!("BC-2.15.007: remove QueryId from DashMap registry on query completion")
    }

    /// Check whether the current resource level requires killing the given query.
    ///
    /// If `current_level() == Kill`, cancels the token and returns
    /// `Err(PrismError::WatchdogKilled)` (E-WATCH-001, BC-2.15.007).
    ///
    /// AC-4: RSS at 96% → token cancelled → `Err(PrismError::WatchdogKilled)`.
    pub fn check_query(
        &self,
        _cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<(), PrismError> {
        // AC-4: if Kill level, cancel token and return Err(PrismError::WatchdogKilled)
        todo!("BC-2.15.007 postcondition: if current_level() == Kill, call _cancel_token.cancel() and return Err(PrismError::WatchdogKilled {{ budget_bytes: self.budget_bytes }})")
    }

    /// Return a snapshot of current watchdog state for the `watchdog_status` MCP tool.
    ///
    /// Reads current RSS, computes level, and collects denylist entries from the
    /// `watchdog` CF via the provided backend.
    ///
    /// Capability gating (`runtime`) is enforced by the MCP tool layer, not here.
    pub fn get_watchdog_status<B: crate::backend::RocksStorageBackend>(
        &self,
        _backend: &B,
    ) -> Result<WatchdogStatus, PrismError> {
        // Task 7: populate WatchdogStatus { level, budget_bytes, current_bytes, denylist }
        todo!("Task 7: call current_level(), read current RSS bytes via sysinfo, collect DenylistEntry list from watchdog CF via backend, return WatchdogStatus")
    }
}

impl Default for ResourceWatchdog {
    fn default() -> Self {
        Self::new()
    }
}
