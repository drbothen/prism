//! `DemoHarness` — multi-clone boot, supervisor tasks, and URL table.
//!
//! Owns clone objects by-value in a `Vec<ClonePair>`. Provides:
//! - `start_all()`: starts all enabled clones, populates `StartReport`
//! - `stop_all()`: sends graceful shutdown signal, hard-aborts after 5s
//! - `print_url_table()`: prints the URL table to stdout
//! - `last_start_report()`: returns `&StartReport` for the most recent `start_all()`
//!
//! # Ownership Model (ADR-002 Amendment §H1)
//!
//! The harness owns clone objects by-value in `Vec<ClonePair>`. There is no `Mutex`
//! wrapping — each clone is accessed sequentially during startup (no concurrent `&mut`
//! borrows). This satisfies the workspace `await_holding_lock = "deny"` lint.
//!
//! # Shutdown (ADR-002 Amendment §H2)
//!
//! Graceful drain via `shutdown_tx` broadcast; hard-abort via `JoinHandle::abort()`
//! inside `clone.stop()` after a 5-second timeout.

use std::net::SocketAddr;

use anyhow::Context;
use prism_dtu_common::BehavioralClone;
use tokio::task::JoinHandle;

use crate::config::{CloneConfig, DemoConfig};

/// A clone name + instance + bound address, held by value in the harness.
pub struct ClonePair {
    /// Human-readable clone name (e.g. `"crowdstrike"`).
    pub name: String,
    /// The clone instance owned by this pair.
    pub clone: Box<dyn BehavioralClone>,
    /// Set after `start_on()` returns successfully; `None` if not yet started.
    pub bound_addr: Option<SocketAddr>,
    /// When `true`: a bind failure logs WARN and the harness skips this clone.
    /// When `false` (default): a bind failure aborts startup (AC-11 cleanup path).
    pub continue_on_error: bool,
}

impl ClonePair {
    /// Construct a new `ClonePair` from a named clone instance.
    pub fn new(name: impl Into<String>, clone: Box<dyn BehavioralClone>) -> Self {
        Self {
            name: name.into(),
            clone,
            bound_addr: None,
            continue_on_error: false,
        }
    }
}

/// Describes the outcome of the most recent `DemoHarness::start_all()` call.
///
/// Exactly one of the following conditions holds after `start_all()` returns:
///
/// - **All success**: `successfully_started.len() == 6`, all other vecs/fields empty/None.
/// - **Abort** (`continue_on_error=false`, one clone failed): `failed_at.is_some()`,
///   `cleaned_up_after_failure` has the rolled-back clones, `skipped_due_to_error.is_empty()`.
/// - **Partial success** (`continue_on_error=true`, ≥1 clone failed): `skipped_due_to_error`
///   has the failures, `successfully_started` has the survivors, `failed_at.is_none()`,
///   `cleaned_up_after_failure.is_empty()` (no rollback in continue mode).
///
/// Used by tests (AC-11, AC-12, AC-13) to observe partial-startup cleanup behavior.
#[derive(Debug, Default)]
pub struct StartReport {
    /// Names of clones that bound successfully and are now serving.
    pub successfully_started: Vec<String>,
    /// Names of clones that were started and then stopped during partial-startup cleanup
    /// (abort path only — `continue_on_error=false`).
    pub cleaned_up_after_failure: Vec<String>,
    /// Set when `continue_on_error=false` and a clone failed — the harness aborted and
    /// rolled back `cleaned_up_after_failure`. Always `None` under `continue_on_error=true`.
    pub failed_at: Option<(String, std::io::Error)>,
    /// Clones that failed to bind and were skipped (only under `continue_on_error=true`).
    /// Always empty under `continue_on_error=false`.
    pub skipped_due_to_error: Vec<(String, std::io::Error)>,
}

/// Multi-clone demo harness.
///
/// Manages the lifecycle of all enabled DTU clone instances:
/// start, supervise, shutdown.
pub struct DemoHarness {
    /// Clone pairs owned by value; indexed by clone position.
    pub pairs: Vec<ClonePair>,
    /// Supervisor task handles; parallel index to `pairs`.
    tasks: Vec<JoinHandle<()>>,
    /// Broadcast sender for the graceful-shutdown signal.
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
    /// Populated by `start_all()`; describes the most recent startup outcome.
    last_start_report: StartReport,
}

impl DemoHarness {
    /// Create a new harness from a list of clone pairs.
    pub fn new(pairs: Vec<ClonePair>) -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        Self {
            pairs,
            tasks: Vec::new(),
            shutdown_tx,
            last_start_report: StartReport::default(),
        }
    }

    /// Start all enabled clone pairs.
    ///
    /// On success all pairs are bound and serving. On failure (when `continue_on_error=false`),
    /// the already-started clones are stopped and `Err` is returned.
    pub async fn start_all(&mut self, config: &DemoConfig) -> anyhow::Result<()> {
        todo!(
            "DemoHarness::start_all() not yet implemented — \
             implement in S-6.20 Phase 2"
        )
    }

    /// Stop all running clones.
    ///
    /// Sends the graceful-shutdown broadcast. Waits up to 5 seconds for all tasks to
    /// complete. Any task that has not completed is hard-aborted via `clone.stop()`.
    /// Calls `clone.reset()` on every pair regardless of drain outcome.
    pub async fn stop_all(&mut self) {
        todo!(
            "DemoHarness::stop_all() not yet implemented — \
             implement in S-6.20 Phase 2"
        )
    }

    /// Return the `StartReport` for the most recent `start_all()` call.
    pub fn last_start_report(&self) -> &StartReport {
        &self.last_start_report
    }

    /// Print the URL table to stdout.
    ///
    /// Only lists clones with a bound address (i.e., successfully started).
    pub fn print_url_table(&self) {
        todo!(
            "DemoHarness::print_url_table() not yet implemented — \
             implement in S-6.20 Phase 2"
        )
    }
}

/// Build all clone pairs from a `DemoConfig`.
///
/// Handles both infallible constructors (crowdstrike, claroty, threatintel) and fallible
/// constructors (cyberint, armis, nvd) by propagating errors with `?`.
pub fn build_clone_pairs(config: &DemoConfig) -> anyhow::Result<Vec<ClonePair>> {
    todo!(
        "build_clone_pairs() not yet implemented — \
         implement in S-6.20 Phase 2"
    )
}

/// Parse a `SocketAddr` from a `CloneConfig` bind IP and port.
pub fn clone_bind_addr(cfg: &CloneConfig) -> anyhow::Result<SocketAddr> {
    let addr_str = format!("{}:{}", cfg.bind, cfg.port);
    addr_str
        .parse::<SocketAddr>()
        .with_context(|| format!("Invalid bind address: {}", addr_str))
}

/// Test utilities.
///
/// Always compiled (this crate is test/demo infrastructure only; it never
/// links into production binaries). Integration tests access this module via
/// `prism_dtu_demo_server::harness::test_utils`.
pub mod test_utils {
    use std::net::SocketAddr;

    /// Assert that the given `SocketAddr` is no longer bound within `timeout`.
    ///
    /// Used by AC-11 tests to verify no listener leak after partial-startup failure.
    pub async fn assert_port_released(addr: SocketAddr, timeout: std::time::Duration) {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if tokio::net::TcpListener::bind(addr).await.is_ok() {
                return;
            }
            if tokio::time::Instant::now() >= deadline {
                panic!("port {} still bound after {:?}", addr, timeout);
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
}
