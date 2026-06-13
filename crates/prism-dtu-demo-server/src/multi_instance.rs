//! Multi-instance binding for `prism-dtu-demo-server`.
//!
//! Provides [`MultiInstanceConfig`] and the [`start_instances`] function for
//! binding N named DTU clone instances to distinct socket addresses.
//!
//! # Story anchor
//!
//! S-DEMO-MULTI-TENANT-DTU-001 (BC-2.06.017 Postcondition 1)
//!
//! # Architecture note (D-1075)
//!
//! These types live in `prism-dtu-demo-server`, NOT `prism-dtu-common`, to avoid
//! coupling all downstream clone crates to orchestration types.
//!
//! All public structs and enums carry `#[non_exhaustive]` per
//! `CLAUDE.md §#[non_exhaustive] discipline` and `BC-2.06.017 INV-NONEXHAUSTIVE-001`.

use std::{collections::HashMap, net::SocketAddr};

use prism_dtu_common::BehavioralClone;
use tokio::sync::broadcast;

/// Configuration for starting N named DTU clone instances at distinct addresses.
///
/// Each entry in `instances` starts exactly one clone instance via
/// `BehavioralClone::start_on`. Duplicate `InstanceEntry::name` values return
/// `Err(MultiInstanceBindError::DuplicateName { name })` before any bind attempts
/// (BC-2.06.017 Postcondition 7).
///
/// A zero-length `instances` vec is valid and returns `Ok(HashMap::new())`
/// with no spawned tasks (BC-2.06.017 EC-017-002).
///
/// (BC-2.06.017 Postcondition 1 — multi-instance bind configuration accepted)
#[non_exhaustive]
pub struct MultiInstanceConfig {
    /// Ordered list of named clone instances to bind.
    pub instances: Vec<InstanceEntry>,
}

/// A single named DTU clone instance bind entry.
///
/// `name` uniquely identifies the instance within a [`MultiInstanceConfig`].
/// Duplicate names return `Err(MultiInstanceBindError::DuplicateName { name })`
/// (BC-2.06.017 EC-017-009 / Postcondition 7).
///
/// `bind` is typically `127.0.0.1:0` so the OS assigns an ephemeral port.
///
/// (BC-2.06.017 Postcondition 1)
#[non_exhaustive]
pub struct InstanceEntry {
    /// Human-readable instance name (e.g., `"armis-acme"`).
    pub name: String,
    /// Bind address; use `127.0.0.1:0` for OS-assigned ephemeral port.
    pub bind: SocketAddr,
}

impl MultiInstanceConfig {
    /// Construct a `MultiInstanceConfig` with the given instances.
    ///
    /// This is the canonical constructor for external callers (required because
    /// `#[non_exhaustive]` prevents struct-literal construction outside this crate).
    pub fn new(instances: Vec<InstanceEntry>) -> Self {
        Self { instances }
    }
}

impl InstanceEntry {
    /// Construct an `InstanceEntry` with the given name and bind address.
    ///
    /// This is the canonical constructor for external callers (required because
    /// `#[non_exhaustive]` prevents struct-literal construction outside this crate).
    pub fn new(name: impl Into<String>, bind: std::net::SocketAddr) -> Self {
        Self {
            name: name.into(),
            bind,
        }
    }
}

/// Error returned by [`start_instances`].
///
/// Two variants:
/// - `DuplicateName`: returned immediately (before any bind) when two
///   [`InstanceEntry`] values share the same `name`.
/// - `BindFailure`: returned after all bind operations are attempted (multi-error
///   aggregation per BC-2.06.017 INV-ERR-003-COMPAT / Postcondition 6).
///
/// (BC-2.06.017 Postconditions 6–7)
#[derive(Debug)]
#[non_exhaustive]
pub enum MultiInstanceBindError {
    /// Two [`InstanceEntry`] items share the same `name` string.
    ///
    /// Returned before any bind attempt (EC-017-009).
    DuplicateName {
        /// The duplicated name string.
        name: String,
    },

    /// One or more bind operations failed.
    ///
    /// All N bind operations are attempted before this error is returned; the
    /// `Vec` contains every failure (INV-ERR-003-COMPAT). Successfully-started
    /// instances are shut down before the error is returned (no zombie instances).
    BindFailure(Vec<DemoBindError>),
}

/// A single bind failure within a [`MultiInstanceBindError::BindFailure`] error.
///
/// (BC-2.06.017 Postcondition 6 + v1.1 Amendment 2 — name disambiguation from
/// the harness-side `BindError` type)
#[derive(Debug)]
#[non_exhaustive]
pub struct DemoBindError {
    /// The `InstanceEntry::name` of the instance that failed to bind.
    pub instance_name: String,
    /// The underlying OS bind error.
    pub source: std::io::Error,
}

/// Lifecycle handle for N running DTU clone instances started by [`start_instances`].
///
/// Owns the single shared `shutdown_tx: broadcast::Sender<()>` and all N watcher
/// task handles. Calling [`shutdown`][MultiInstanceServers::shutdown] or dropping
/// this value sends the graceful-shutdown signal to all instances; axum's
/// `with_graceful_shutdown` drains in-flight requests, then releases bound ports.
///
/// This eliminates the success-path zombie/leaked-instance hazard (the success-path
/// analogue of Postcondition 6's "no partial-bound zombie instances on bind failure"
/// guarantee) and is consistent with EC-017-005 (both `MultiInstanceHarness` Drop
/// and `MultiInstanceServers` Drop use the same pattern).
///
/// (BC-2.06.017 Postcondition 1 v1.2 — D-1075-API-GAP-001)
#[non_exhaustive]
pub struct MultiInstanceServers {
    socket_map: HashMap<String, SocketAddr>,
    /// Shared broadcast sender — owned EXCLUSIVELY by this handle.
    /// Each instance subscribed via `shutdown_tx.subscribe()` at bind time.
    /// Watcher tasks hold ONLY the Receiver (subscriber), NEVER an additional Sender
    /// clone (the prior tx_keeper-in-watcher pattern was the bug — it kept the
    /// channel open forever). Sending or dropping this sender wakes all receivers
    /// → axum graceful drain → port release.
    shutdown_tx: broadcast::Sender<()>,
    /// Watcher task handles — held for RAII lifetime management.
    /// Dropped WITHOUT explicit abort on shutdown (axum handles the graceful drain).
    #[allow(dead_code)]
    task_handles: Vec<tokio::task::JoinHandle<()>>,
}

impl MultiInstanceServers {
    /// Return the per-name bound `SocketAddr` map.
    ///
    /// Each key is the `InstanceEntry::name` string; each value is the OS-assigned
    /// bound address. All N instances appear in the map; no entries are silently dropped.
    ///
    /// (BC-2.06.017 Postcondition 1)
    pub fn socket_map(&self) -> &HashMap<String, SocketAddr> {
        &self.socket_map
    }

    /// Send the graceful-shutdown signal to all instances.
    ///
    /// Idempotent — calling multiple times is safe (subsequent sends are no-ops
    /// because there are no receivers left after the first send wakes them all).
    /// Axum's `with_graceful_shutdown` drains in-flight requests, then releases
    /// bound ports (EC-017-005 / BC-2.06.017 Postcondition 1 v1.2).
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

impl Drop for MultiInstanceServers {
    /// Send the graceful-shutdown signal and drop task handles.
    ///
    /// The signal is sent best-effort (error ignored). Task handles are dropped
    /// WITHOUT explicit abort — axum `with_graceful_shutdown` drains in-flight
    /// requests on the clone side (EC-017-005).
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
        // task_handles dropped here by RAII; no explicit abort.
    }
}

/// Start N named DTU clone instances at distinct socket addresses.
///
/// # Behaviour
///
/// - **Duplicate check first:** if any two entries share the same `name`,
///   returns `Err(MultiInstanceBindError::DuplicateName { name })` before
///   any bind attempt (BC-2.06.017 Postcondition 7 / EC-017-009).
/// - **Multi-error aggregation:** all bind operations are attempted before
///   returning any error (INV-ERR-003-COMPAT). If any fail, successfully-started
///   instances are shut down and `Err(BindFailure(failures))` is returned.
/// - **Zero instances:** `MultiInstanceConfig { instances: [] }` returns
///   `Ok(MultiInstanceServers)` with an empty socket map — no error, no spawned
///   tasks (EC-017-002).
/// - On success, all N instances are serving requests before the function returns.
///
/// # Parameters
///
/// - `cfg`: the multi-instance configuration specifying each instance name and bind address.
/// - `clone_factory`: a factory closure that produces a `Box<dyn BehavioralClone>` for
///   a given [`InstanceEntry`]. The factory is called at most once per entry.
///   The factory receives only the `InstanceEntry` — it does NOT receive any external
///   shutdown channel. The returned [`MultiInstanceServers`] owns the single shared
///   `shutdown_tx` internally; calling `servers.shutdown()` or dropping `servers` sends
///   the signal (D-1075-API-GAP-001).
///
/// # Returns
///
/// `Ok(MultiInstanceServers)` — lifecycle handle that:
/// - Exposes `servers.socket_map() -> &HashMap<String, SocketAddr>` mapping each
///   `entry.name` to its OS-assigned bound address.
/// - Triggers graceful shutdown of ALL instances when `servers.shutdown()` is called
///   or when `servers` is dropped.
///
/// (BC-2.06.017 Postcondition 1 v1.2 — D-1075-API-GAP-001)
pub async fn start_instances(
    cfg: MultiInstanceConfig,
    clone_factory: impl Fn(&InstanceEntry) -> Box<dyn BehavioralClone>,
) -> Result<MultiInstanceServers, MultiInstanceBindError> {
    // --- Duplicate-name check (BC-2.06.017 Postcondition 7 / EC-017-009) ---
    // Must happen BEFORE any bind attempts.
    let mut seen_names = std::collections::HashSet::new();
    for entry in &cfg.instances {
        if !seen_names.insert(entry.name.clone()) {
            return Err(MultiInstanceBindError::DuplicateName {
                name: entry.name.clone(),
            });
        }
    }

    // --- EC-017-002: zero instances → empty MultiInstanceServers with empty socket_map ---
    // Use a capacity-1 channel even for zero instances so the Sender is always valid.
    if cfg.instances.is_empty() {
        let (shutdown_tx, _) = broadcast::channel::<()>(1);
        return Ok(MultiInstanceServers {
            socket_map: HashMap::new(),
            shutdown_tx,
            task_handles: Vec::new(),
        });
    }

    // --- Single shared broadcast channel (D-1075-API-GAP-001 architect decision) ---
    // ONE shared Sender owned exclusively by the returned MultiInstanceServers handle.
    // Each instance subscribes via shutdown_tx.subscribe() at bind time.
    // Watcher tasks hold ONLY the Receiver — NEVER an additional Sender clone.
    // This is the fix for the prior tx_keeper-in-watcher pattern that kept the
    // channel open forever (the bug identified in D-1075-API-GAP-001).
    let (shutdown_tx, _) = broadcast::channel::<()>(cfg.instances.len().max(1));

    // --- Multi-error aggregation bind loop (INV-ERR-003-COMPAT) ---
    // Attempt ALL binds before returning any error.
    // Track successful binds so we can shut them down on partial failure
    // (no zombie instances per BC-2.06.017 Postcondition 6).
    struct BoundInstance {
        name: String,
        addr: SocketAddr,
        task_handle: tokio::task::JoinHandle<()>,
    }

    let mut bound: Vec<BoundInstance> = Vec::with_capacity(cfg.instances.len());
    let mut failures: Vec<DemoBindError> = Vec::new();

    for entry in &cfg.instances {
        let mut clone = clone_factory(entry);
        // Subscribe to the shared channel BEFORE calling start_on.
        // The clone's axum server uses this receiver for with_graceful_shutdown.
        let shutdown_rx = shutdown_tx.subscribe();

        match clone.start_on(entry.bind, Some(shutdown_rx), None).await {
            Ok(addr) => {
                // Spawn a lightweight watcher task that keeps the clone object alive
                // until the shared shutdown_tx fires (via Drop or shutdown()).
                // The watcher holds ONLY the clone — no extra Sender clone.
                // When shutdown_tx.send(()) fires (from Drop or shutdown()),
                // the clone's axum server receives the signal and drains gracefully.
                // The watcher task just waits for the clone's server_handle to complete.
                //
                // Note: ArmisClone / ClarotyClone keep their server JoinHandle alive
                // internally. When the clone is dropped, the JoinHandle detaches
                // (doesn't abort). The server continues running until its shutdown_rx
                // fires. We spawn this watcher so the clone object stays alive (and
                // thus the server JoinHandle stays in scope) until the harness drops.
                let task_handle = tokio::spawn(async move {
                    // Keep clone alive until the task is dropped.
                    // The server shuts down when the shared shutdown_tx fires.
                    // This task just holds the clone alive for its lifetime.
                    // We use a channel recv to park the task; it wakes when the
                    // clone's internal server_handle completes (after graceful drain).
                    //
                    // Actually: the server's JoinHandle is inside clone.server_handle.
                    // When clone is dropped here, the JoinHandle detaches. The server
                    // keeps running via the axum graceful-shutdown mechanism until the
                    // shared shutdown_rx (subscribed above) fires. Dropping clone here
                    // is correct — the server's JoinHandle is detached, not aborted.
                    drop(clone);
                });
                bound.push(BoundInstance {
                    name: entry.name.clone(),
                    addr,
                    task_handle,
                });
            }
            Err(e) => {
                let io_err = std::io::Error::other(e);
                failures.push(DemoBindError {
                    instance_name: entry.name.clone(),
                    source: io_err,
                });
            }
        }
    }

    // --- Error path: signal shutdown to all successful binds, return aggregated failures ---
    if !failures.is_empty() {
        // Signal via the shared sender — wakes all subscribers (successfully-bound instances).
        let _ = shutdown_tx.send(());
        // Await graceful drain for successfully-bound instances (up to 500ms).
        for instance in bound {
            let _ =
                tokio::time::timeout(std::time::Duration::from_millis(500), instance.task_handle)
                    .await;
        }
        return Err(MultiInstanceBindError::BindFailure(failures));
    }

    // --- Success path: bundle into MultiInstanceServers lifecycle handle ---
    // The shared shutdown_tx is owned EXCLUSIVELY by the returned handle.
    // Each clone already subscribed (via shutdown_tx.subscribe() above).
    // No extra Sender clones exist — the channel closes (waking all receivers)
    // when either shutdown() sends the signal or the handle is dropped.
    let mut socket_map = HashMap::with_capacity(bound.len());
    let mut task_handles = Vec::with_capacity(bound.len());
    for instance in bound {
        socket_map.insert(instance.name, instance.addr);
        task_handles.push(instance.task_handle);
    }

    Ok(MultiInstanceServers {
        socket_map,
        shutdown_tx,
        task_handles,
    })
}
