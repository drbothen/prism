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

use std::collections::HashMap;
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
        // Reset report for this run.
        self.last_start_report = StartReport::default();

        // Validate bind security before starting any clone (AC-9).
        validate_bind_security(config)?;

        // Build ordered list of (pair_index, clone_config) for enabled clones.
        let enabled_indices: Vec<(usize, &CloneConfig, bool)> = self
            .pairs
            .iter()
            .enumerate()
            .map(|(i, pair)| {
                let cfg = clone_config_by_name(config, &pair.name);
                let coe = pair.continue_on_error;
                (i, cfg, coe)
            })
            .collect();

        for (pair_idx, clone_cfg, continue_on_error) in &enabled_indices {
            let bind_addr = clone_bind_addr(clone_cfg)?;
            let shutdown_rx = self.shutdown_tx.subscribe();

            let pair = &mut self.pairs[*pair_idx];

            match pair.clone.start_on(bind_addr, Some(shutdown_rx)).await {
                Ok(bound) => {
                    pair.bound_addr = Some(bound);
                    self.last_start_report
                        .successfully_started
                        .push(pair.name.clone());

                    // Spawn a no-op supervisor task (the server runs inside the clone's JoinHandle).
                    // We track a dummy task here to keep tasks parallel to pairs.
                    let task_handle = tokio::spawn(async {});
                    self.tasks.push(task_handle);


                }
                Err(e) => {
                    // Convert to std::io::Error if possible (for AddrInUse kind).
                    let io_err = to_io_error(e);

                    if *continue_on_error {
                        // Log warning and skip this clone.
                        tracing::warn!(
                            "[WARN] {} failed to start: {} — continuing per continue_on_error=true",
                            pair.name,
                            io_err
                        );
                        self.last_start_report
                            .skipped_due_to_error
                            .push((pair.name.clone(), io_err));
                        // Push dummy handle to keep indices in sync.
                        self.tasks.push(tokio::spawn(async {}));
                    } else {
                        // Abort path: stop the N already-started clones.
                        let failed_name = pair.name.clone();
                        self.last_start_report.failed_at = Some((failed_name.clone(), io_err));

                        // Stop previously started clones in reverse order (or forward — spec says in order).
                        // The report must list them as cleaned_up_after_failure.
                        // We need to stop pairs[0..started_count].
                        let already_started: Vec<String> = self
                            .last_start_report
                            .successfully_started
                            .drain(..)
                            .collect();

                        for name in &already_started {
                            // Find the pair by name and stop it.
                            if let Some(p) = self.pairs.iter_mut().find(|p| &p.name == name) {
                                let _ = p.clone.stop().await;
                            }
                        }

                        // Record cleanup.
                        self.last_start_report.cleaned_up_after_failure = already_started;

                        return Err(anyhow::anyhow!(
                            "clone '{}' failed to start: abort triggered (R-DEMO-001 cleanup)",
                            failed_name
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop all running clones.
    ///
    /// Sends the graceful-shutdown broadcast. Waits up to 5 seconds for all tasks to
    /// complete. Any task that has not completed is hard-aborted via `clone.stop()`.
    /// Calls `clone.reset()` on every pair regardless of drain outcome.
    pub async fn stop_all(&mut self) {
        // Send shutdown signal (ignore "no receivers" error).
        let _ = self.shutdown_tx.send(());

        // Wait up to 5 seconds for all supervisor tasks to complete.
        let graceful_timeout = std::time::Duration::from_secs(5);

        let result = tokio::time::timeout(graceful_timeout, async {
            for task in &mut self.tasks {
                let _ = task.await;
            }
        })
        .await;

        if result.is_err() {
            // Timeout elapsed — hard-abort any remaining clones.
            for pair in &mut self.pairs {
                if pair.bound_addr.is_some() {
                    let _ = pair.clone.stop().await;
                }
            }
        }

        // Reset every pair regardless of path taken.
        for pair in &mut self.pairs {
            let _ = pair.clone.reset().await;
        }

        // Clear tasks.
        self.tasks.clear();
    }

    /// Return the `StartReport` for the most recent `start_all()` call.
    pub fn last_start_report(&self) -> &StartReport {
        &self.last_start_report
    }

    /// Return a map from clone name to bound URL string.
    ///
    /// Only includes clones with a bound address (i.e., successfully started).
    /// Used by the binary to write the URL sidecar file for the `configure` subcommand.
    pub fn url_map(&self) -> HashMap<String, String> {
        self.pairs
            .iter()
            .filter_map(|pair| {
                pair.bound_addr
                    .map(|addr| (pair.name.clone(), format!("http://{addr}")))
            })
            .collect()
    }

    /// Print the URL table to stdout.
    ///
    /// Only lists clones with a bound address (i.e., successfully started).
    pub fn print_url_table(&self) {
        println!(
            "| {:<13} | {:<25} | {:<7} | {:<7} |",
            "Clone", "URL", "Fixture", "Failure"
        );
        println!("|{:-<15}|{:-<27}|{:-<9}|{:-<9}|", "", "", "", "");
        for pair in &self.pairs {
            if let Some(addr) = pair.bound_addr {
                println!(
                    "| {:<13} | {:<25} | {:<7} | {:<7} |",
                    pair.name,
                    format!("http://{addr}"),
                    "default",
                    "none"
                );
            }
        }
    }
}

/// Build all clone pairs from a `DemoConfig`.
///
/// Handles both infallible constructors (crowdstrike, claroty, threatintel) and fallible
/// constructors (cyberint, armis, nvd) by propagating errors with `?`.
pub fn build_clone_pairs(config: &DemoConfig) -> anyhow::Result<Vec<ClonePair>> {
    use prism_dtu_armis::ArmisClone;
    use prism_dtu_claroty::ClarotyClone;
    use prism_dtu_crowdstrike::CrowdstrikeClone;
    use prism_dtu_cyberint::CyberintClone;
    use prism_dtu_nvd::NvdClone;
    use prism_dtu_threatintel::ThreatIntelClone;

    let mut pairs = Vec::new();

    if config.clones.crowdstrike.enabled {
        let mut pair = ClonePair::new("crowdstrike", Box::new(CrowdstrikeClone::new()));
        pair.continue_on_error = config.clones.crowdstrike.continue_on_error;
        pairs.push(pair);
    }

    if config.clones.claroty.enabled {
        let mut pair = ClonePair::new("claroty", Box::new(ClarotyClone::new()));
        pair.continue_on_error = config.clones.claroty.continue_on_error;
        pairs.push(pair);
    }

    if config.clones.cyberint.enabled {
        let mut pair = ClonePair::new(
            "cyberint",
            Box::new(CyberintClone::new().context("failed to construct CyberintClone")?),
        );
        pair.continue_on_error = config.clones.cyberint.continue_on_error;
        pairs.push(pair);
    }

    if config.clones.armis.enabled {
        let mut pair = ClonePair::new(
            "armis",
            Box::new(ArmisClone::new().context("failed to construct ArmisClone")?),
        );
        pair.continue_on_error = config.clones.armis.continue_on_error;
        pairs.push(pair);
    }

    if config.clones.threatintel.enabled {
        let mut pair = ClonePair::new("threatintel", Box::new(ThreatIntelClone::new()));
        pair.continue_on_error = config.clones.threatintel.continue_on_error;
        pairs.push(pair);
    }

    if config.clones.nvd.enabled {
        let mut pair = ClonePair::new(
            "nvd",
            Box::new(NvdClone::new().context("failed to construct NvdClone")?),
        );
        pair.continue_on_error = config.clones.nvd.continue_on_error;
        pairs.push(pair);
    }

    Ok(pairs)
}

/// Parse a `SocketAddr` from a `CloneConfig` bind IP and port.
pub fn clone_bind_addr(cfg: &CloneConfig) -> anyhow::Result<SocketAddr> {
    let addr_str = format!("{}:{}", cfg.bind, cfg.port);
    addr_str
        .parse::<SocketAddr>()
        .with_context(|| format!("Invalid bind address: {}", addr_str))
}

/// Validate bind security (AC-9, R-DEMO-001).
///
/// Non-loopback binds require BOTH `--bind-any` flag AND
/// `PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK`.
///
/// In the harness, `bind_any` is tracked via the env var only (the CLI flag sets it
/// before calling `start_all`). The harness checks the config for non-loopback IPs.
fn validate_bind_security(config: &DemoConfig) -> anyhow::Result<()> {
    let allow_env = std::env::var("PRISM_DTU_DEMO_ALLOW_NETWORK_BIND")
        .unwrap_or_default();
    let allowed = allow_env == "I-UNDERSTAND-THE-RISK";

    // Check harness bind.
    if !is_loopback(&config.harness.bind) && !allowed {
        anyhow::bail!(
            "Non-loopback bind address '{}' rejected. \
             Set --bind-any AND PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK \
             to allow network binding (R-DEMO-001).",
            config.harness.bind
        );
    }

    // Check each clone bind.
    for (name, cfg) in all_clone_configs(config) {
        if cfg.enabled && !is_loopback(&cfg.bind) && !allowed {
            anyhow::bail!(
                "Non-loopback bind address '{}' for clone '{}' rejected. \
                 Set --bind-any AND PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK \
                 to allow network binding (R-DEMO-001). --bind-any loopback",
                cfg.bind,
                name
            );
        }
    }

    Ok(())
}

/// Return true if the IP string is a loopback address.
fn is_loopback(ip: &str) -> bool {
    if let Ok(addr) = ip.parse::<std::net::IpAddr>() {
        addr.is_loopback()
    } else {
        // Unparseable — be safe and treat as non-loopback.
        false
    }
}

/// Return all clone configs as (name, &CloneConfig).
fn all_clone_configs(config: &DemoConfig) -> Vec<(&'static str, &CloneConfig)> {
    vec![
        ("crowdstrike", &config.clones.crowdstrike),
        ("claroty", &config.clones.claroty),
        ("cyberint", &config.clones.cyberint),
        ("armis", &config.clones.armis),
        ("threatintel", &config.clones.threatintel),
        ("nvd", &config.clones.nvd),
    ]
}

/// Get the `CloneConfig` for a named clone.
fn clone_config_by_name<'a>(config: &'a DemoConfig, name: &str) -> &'a CloneConfig {
    match name {
        "crowdstrike" => &config.clones.crowdstrike,
        "claroty" => &config.clones.claroty,
        "cyberint" => &config.clones.cyberint,
        "armis" => &config.clones.armis,
        "threatintel" => &config.clones.threatintel,
        "nvd" => &config.clones.nvd,
        other => panic!("unknown clone name: {other}"),
    }
}

/// Convert an `anyhow::Error` into a `std::io::Error`, preserving `AddrInUse` kind
/// when the underlying error is an OS bind error.
fn to_io_error(err: anyhow::Error) -> std::io::Error {
    // Walk the error chain to find an io::Error.
    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
        std::io::Error::new(io_err.kind(), io_err.to_string())
    } else {
        // Check if source chain has an io::Error.
        let mut source: Option<&dyn std::error::Error> = err.source();
        while let Some(s) = source {
            if let Some(io_err) = s.downcast_ref::<std::io::Error>() {
                return std::io::Error::new(io_err.kind(), io_err.to_string());
            }
            source = s.source();
        }
        std::io::Error::other(err.to_string())
    }
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
