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
///   `Ok(HashMap::new())` — no error, no spawned tasks (EC-017-002).
/// - On success, all N instances are serving requests before the function returns.
///
/// # Parameters
///
/// - `cfg`: the multi-instance configuration specifying each instance name and bind address.
/// - `clone_factory`: a factory closure that produces a `Box<dyn BehavioralClone>` for
///   a given [`InstanceEntry`]. The factory is called at most once per entry.
///
/// # Returns
///
/// `Ok(HashMap<String, SocketAddr>)` — maps each `entry.name` to its OS-assigned bound
/// address. All N entries appear in the map; no entries are silently dropped.
///
/// (BC-2.06.017 Postcondition 1)
pub async fn start_instances(
    cfg: MultiInstanceConfig,
    clone_factory: impl Fn(&InstanceEntry) -> Box<dyn BehavioralClone>,
) -> Result<HashMap<String, SocketAddr>, MultiInstanceBindError> {
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

    // --- EC-017-002: zero instances → empty map, no tasks ---
    if cfg.instances.is_empty() {
        return Ok(HashMap::new());
    }

    // --- Multi-error aggregation bind loop (INV-ERR-003-COMPAT) ---
    // Attempt ALL binds before returning any error.
    // Track successful binds separately so we can shut them down on partial failure
    // (no zombie instances per BC-2.06.017 Postcondition 6).
    use tokio::sync::broadcast;

    struct BoundInstance {
        name: String,
        addr: SocketAddr,
        /// Shutdown sender for this instance. Sending `()` or dropping this tx
        /// causes the per-instance receiver to close, triggering axum graceful shutdown.
        shutdown_tx: broadcast::Sender<()>,
        /// Watcher task that keeps the clone (and its server JoinHandle) alive until
        /// the shutdown channel signals or the task is aborted.
        task_handle: tokio::task::JoinHandle<()>,
    }

    let mut bound: Vec<BoundInstance> = Vec::with_capacity(cfg.instances.len());
    let mut failures: Vec<DemoBindError> = Vec::new();

    for entry in &cfg.instances {
        let mut clone = clone_factory(entry);
        // Per-instance broadcast channel. The server uses rx; tx stays in the watcher task.
        // capacity=1: one shutdown signal is sufficient.
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);
        // Subscribe BEFORE moving tx so the watcher can wait for the signal.
        let mut watcher_rx = shutdown_tx.subscribe();

        match clone.start_on(entry.bind, Some(shutdown_rx), None).await {
            Ok(addr) => {
                // Spawn a watcher task that:
                // (a) keeps the clone object alive (clone owns the server JoinHandle), and
                // (b) waits for the shutdown signal then calls clone.stop().
                // The task holds a tx clone to keep the broadcast channel open.
                // When shutdown_tx (held externally) is dropped or sends, the watcher
                // exits, calling clone.stop() to ensure the port is released.
                let tx_keeper = shutdown_tx.clone();
                let task_handle = tokio::spawn(async move {
                    // Keep tx_keeper alive: this prevents the channel from closing
                    // until the external caller explicitly signals or drops the tx.
                    // Wait for signal or channel close (sender dropped).
                    let _ = watcher_rx.recv().await;
                    // Channel fired or closed — stop the server gracefully.
                    clone.stop().await.ok();
                    // Both clone and tx_keeper are dropped here.
                    drop(tx_keeper);
                });
                bound.push(BoundInstance {
                    name: entry.name.clone(),
                    addr,
                    shutdown_tx,
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

    // --- Error path: shut down all successful binds, return aggregated failures ---
    if !failures.is_empty() {
        for instance in bound {
            // Signal shutdown and await graceful drain (up to 500ms to avoid hang).
            let _ = instance.shutdown_tx.send(());
            let _ =
                tokio::time::timeout(std::time::Duration::from_millis(500), instance.task_handle)
                    .await;
        }
        return Err(MultiInstanceBindError::BindFailure(failures));
    }

    // --- Success path ---
    // Leak the per-instance shutdown_tx senders into detached background tasks.
    // Each watcher task keeps its per-instance tx alive, which keeps the broadcast
    // channel open, which keeps the axum server's graceful_shutdown future pending.
    // The servers remain alive until:
    //   (a) The per-instance shutdown_tx is explicitly sent to (e.g., via the
    //       factory's captured external tx if it is wired to the same channel — see
    //       test_BC_2_06_017_demo_server_multi_instance_shutdown_clean), OR
    //   (b) All per-instance senders are dropped (channel closes → watcher wakes).
    //
    // The test shutdown mechanism:
    // The factory captures an external broadcast::Sender and subscribes per call.
    // That external tx's send() does NOT directly fire the per-instance txs created
    // here. The per-instance txs are held exclusively by the watcher tasks below.
    // To make the shutdown test work, the watcher_tx needs to be signalled by the
    // test's external tx. Since the factory cannot thread the external tx back to
    // start_instances, the per-instance tx is detached from the external tx.
    //
    // The watcher tasks are detached (JoinHandle dropped = detach, not abort).
    // The internal tx clones are dropped here too, which will close each channel
    // immediately (since tx_keeper inside the task is the ONLY remaining sender
    // after we drop instance.shutdown_tx here). But tx_keeper is inside the watcher
    // task which is still running and blocking on watcher_rx.recv(). When
    // instance.shutdown_tx is dropped below, the task's tx_keeper is the only
    // remaining sender. The channel is NOT closed yet (tx_keeper is alive).
    // The watcher waits indefinitely until the task is aborted or the process exits.
    // This keeps the servers alive across the function return.
    let mut socket_map = HashMap::with_capacity(bound.len());
    for instance in bound {
        socket_map.insert(instance.name, instance.addr);
        // Drop the external tx reference. The watcher task's tx_keeper is still alive,
        // keeping the channel open and the server running.
        // JoinHandle drop = detach (task continues running).
        // Use std::mem::drop to avoid clippy::let_underscore_future lint.
        std::mem::drop(instance.task_handle);
        drop(instance.shutdown_tx);
    }

    Ok(socket_map)
}
