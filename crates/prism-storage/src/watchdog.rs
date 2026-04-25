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
//
// ## MemoryProbe — test-driven design seam (introduced by test-writer, S-2.02 Red Gate)
//
// `ResourceWatchdog` accepts an `Arc<dyn MemoryProbe>` so tests can inject a static
// RSS value without spawning a real sysinfo read.  Production code uses `SysinfoProbe`.
// This seam does NOT change the public API surface — `ResourceWatchdog::new()` still
// constructs the default `SysinfoProbe`; `ResourceWatchdog::with_probe()` is the
// test-friendly constructor.  See `.factory/cycles/v1.0.0-greenfield/S-2.02/
// implementation/red-gate-log.md` for the design decision record.

use std::sync::Arc;

use dashmap::DashMap;
use prism_core::PrismError;
use tokio_util::sync::CancellationToken;

use crate::denylist::DenylistEntry;

// ── MemoryProbe — test-driven seam ───────────────────────────────────────────

/// Abstraction over process RSS measurement.
///
/// Production implementation: `SysinfoProbe` (reads from `sysinfo::System`).
/// Test implementation: `StaticProbe(bytes)` (returns a fixed value).
///
/// Introduced by the test-writer as a test-driven design seam so `ResourceWatchdog`
/// can be tested without reading real process RSS.  Design decision recorded in
/// `.factory/cycles/v1.0.0-greenfield/S-2.02/implementation/red-gate-log.md`.
pub trait MemoryProbe: Send + Sync {
    /// Return the current process RSS in bytes.
    fn current_rss_bytes(&self) -> usize;
}

/// Production `MemoryProbe`: reads current process RSS via the `sysinfo` crate.
///
/// Cross-platform — never reads `/proc/self/status` directly (Architecture
/// Compliance Rule: use `sysinfo` for cross-platform memory measurement).
pub struct SysinfoProbe;

impl MemoryProbe for SysinfoProbe {
    fn current_rss_bytes(&self) -> usize {
        use sysinfo::{Pid, System};
        let pid = Pid::from(std::process::id() as usize);
        let mut sys = System::new();
        sys.refresh_process(pid);
        sys.process(pid).map(|p| p.memory() as usize).unwrap_or(0)
    }
}

/// Test-only `MemoryProbe`: always returns the fixed byte count provided at
/// construction.
///
/// Used to exercise watchdog level thresholds and `check_query` without requiring
/// a real sysinfo read.
pub struct StaticProbe(pub usize);

impl MemoryProbe for StaticProbe {
    fn current_rss_bytes(&self) -> usize {
        self.0
    }
}

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
    /// Memory probe used to read current RSS (injectable for testing).
    pub probe: Arc<dyn MemoryProbe>,
    /// Concurrent registry of active query cancellation tokens.
    ///
    /// All tokens are cancelled when `Kill` level is reached (EC-003 / AC-4).
    tokens: DashMap<QueryId, CancellationToken>,
    /// Shutdown signal for the background monitor task.
    shutdown: CancellationToken,
}

impl ResourceWatchdog {
    /// Construct a `ResourceWatchdog` with the default Normal-level thresholds,
    /// the 512 MiB process budget, and the production `SysinfoProbe`.
    ///
    /// BC-2.15.006 postcondition: thresholds are set at construction; the
    /// watchdog cannot be disabled.
    pub fn new() -> Self {
        ResourceWatchdog {
            warn_pct: 0.70,
            throttle_pct: 0.85,
            kill_pct: 0.95,
            // 512 MB expressed in SI (10^6) bytes — consistent with how sysinfo
            // reports RSS and how the test constants are computed (512*1000*1000).
            budget_bytes: 512 * 1_000 * 1_000,
            probe: Arc::new(SysinfoProbe),
            tokens: DashMap::new(),
            shutdown: CancellationToken::new(),
        }
    }

    /// Construct a `ResourceWatchdog` with a custom `MemoryProbe`.
    ///
    /// Used in tests to inject `StaticProbe(bytes)` instead of reading real RSS.
    /// Production callers use `ResourceWatchdog::new()`.
    pub fn with_probe(probe: Arc<dyn MemoryProbe>) -> Self {
        ResourceWatchdog {
            warn_pct: 0.70,
            throttle_pct: 0.85,
            kill_pct: 0.95,
            budget_bytes: 512 * 1_000 * 1_000,
            probe,
            tokens: DashMap::new(),
            shutdown: CancellationToken::new(),
        }
    }

    /// Read the current process RSS via the injected `MemoryProbe` and return
    /// the graduated level.
    ///
    /// **Postcondition (BC-2.15.006):** returns `WatchdogLevel` based on RSS
    /// relative to `budget_bytes` using thresholds `warn_pct`, `throttle_pct`,
    /// `kill_pct`.
    ///
    /// AC-3: RSS at 86% → `WatchdogLevel::Throttle`.
    pub fn current_level(&self) -> WatchdogLevel {
        let rss = self.probe.current_rss_bytes();
        let budget = self.budget_bytes as f64;
        // Compute integer byte thresholds — avoids float ratio precision issues
        // (e.g. (budget * 0.70) as usize → test RSS of exactly that value must
        // return Warn, not Normal).
        let warn_threshold = (budget * self.warn_pct) as usize;
        let throttle_threshold = (budget * self.throttle_pct) as usize;
        let kill_threshold = (budget * self.kill_pct) as usize;

        if rss >= kill_threshold {
            WatchdogLevel::Kill
        } else if rss >= throttle_threshold {
            WatchdogLevel::Throttle
        } else if rss >= warn_threshold {
            WatchdogLevel::Warn
        } else {
            WatchdogLevel::Normal
        }
    }

    /// Register a query's `CancellationToken` with the watchdog.
    ///
    /// Called at query start.  The watchdog background task will cancel the
    /// token if `current_level()` reaches `Kill`.
    ///
    /// Returns the allocated `QueryId` used to deregister the token on
    /// query completion.
    pub fn register_query(&self, cancel_token: CancellationToken) -> QueryId {
        let id = QueryId::new();
        self.tokens.insert(id, cancel_token);
        id
    }

    /// Deregister a query's `CancellationToken` after the query completes
    /// (success or failure).
    pub fn deregister_query(&self, id: QueryId) {
        self.tokens.remove(&id);
    }

    /// Check whether the current resource level requires killing the given query.
    ///
    /// If `current_level() == Kill`, cancels the token and returns
    /// `Err(PrismError::WatchdogKilled)` (E-WATCHDOG-001, BC-2.15.007).
    ///
    /// AC-4: RSS at 96% → token cancelled → `Err(PrismError::WatchdogKilled)`.
    pub fn check_query(&self, cancel_token: CancellationToken) -> Result<(), PrismError> {
        if self.current_level() == WatchdogLevel::Kill {
            cancel_token.cancel();
            return Err(PrismError::WatchdogKilled {
                budget_bytes: self.budget_bytes,
            });
        }
        Ok(())
    }

    /// Spawn a background tokio task that polls every `poll_interval` and cancels
    /// all registered query tokens when `Kill` level is reached.
    ///
    /// Stops when the watchdog is dropped (shutdown signal via internal
    /// `CancellationToken`).
    pub fn spawn_monitor(
        self: Arc<Self>,
        poll_interval: std::time::Duration,
    ) -> tokio::task::JoinHandle<()> {
        let watchdog = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = watchdog.shutdown.cancelled() => break,
                    _ = tokio::time::sleep(poll_interval) => {
                        if watchdog.current_level() == WatchdogLevel::Kill {
                            for entry in watchdog.tokens.iter() {
                                entry.value().cancel();
                            }
                        }
                    }
                }
            }
        })
    }

    /// Return a snapshot of current watchdog state for the `watchdog_status` MCP tool.
    ///
    /// Reads current RSS, computes level, and collects denylist entries from the
    /// `watchdog` CF via the provided backend.
    ///
    /// Capability gating (`runtime`) is enforced by the MCP tool layer, not here.
    pub fn get_watchdog_status<B: crate::backend::RocksStorageBackend>(
        &self,
        backend: &B,
    ) -> Result<WatchdogStatus, PrismError> {
        let current_bytes = self.probe.current_rss_bytes();
        let level = self.current_level();

        // Collect denylist entries from the watchdog CF.
        use prism_core::StorageDomain;
        let entries = backend.scan(StorageDomain::Watchdog, b"denylist:")?;
        let mut denylist = Vec::with_capacity(entries.len());
        for (key, value) in entries {
            let key_str = String::from_utf8_lossy(&key);
            // Key format: denylist:{fingerprint}
            if let Some(fingerprint) = key_str.strip_prefix("denylist:") {
                let value_str = String::from_utf8_lossy(&value);
                // Value format: {failure_count}:{last_failure_ts}:{expiry_ts}
                let parts: Vec<&str> = value_str.splitn(3, ':').collect();
                if parts.len() == 3 {
                    let failure_count = parts[0].parse::<u32>().unwrap_or(0);
                    let last_failure_ts = parts[1].parse::<u64>().unwrap_or(0);
                    let expiry_ts = parts[2].parse::<u64>().unwrap_or(0);
                    denylist.push(DenylistEntry {
                        fingerprint: fingerprint.to_string(),
                        failure_count,
                        last_failure_ts,
                        expiry_ts,
                    });
                }
            }
        }

        Ok(WatchdogStatus {
            level,
            budget_bytes: self.budget_bytes,
            current_bytes,
            denylist,
        })
    }
}

impl Default for ResourceWatchdog {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ResourceWatchdog {
    fn drop(&mut self) {
        self.shutdown.cancel();
    }
}
