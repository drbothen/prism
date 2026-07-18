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
/// A zero-length `instances` vec is valid and returns `Ok(MultiInstanceServers)`
/// with an empty `socket_map()` — no error, no spawned tasks (BC-2.06.017 EC-017-002).
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
/// analogue of Postcondition 6's "no zombie instances on bind failure" guarantee) and
/// is consistent with EC-017-005 (both `MultiInstanceHarness` Drop and
/// `MultiInstanceServers` Drop use the same pattern).
///
/// # Watcher task vs error-path stop
///
/// On the **success path**, each clone is moved into a `tokio::spawn(async move { drop(clone); })`
/// task spawned AFTER all binds succeed. Because `ArmisClone` / `ClarotyClone` have no
/// `Drop` impl, dropping the clone merely drops its `server_handle: Option<JoinHandle<()>>`,
/// which DETACHES (does not abort) the internal axum server task. The server keeps running
/// as a detached tokio task; the watcher task itself completes immediately. The
/// `task_handles` field holds no-op completion handles, NOT lifetime handles.
/// Real shutdown happens only when `shutdown_tx.send(())` (from `Drop` or `shutdown()`)
/// wakes the axum graceful-shutdown receiver each server holds from `shutdown_tx.subscribe()`.
///
/// On the **error path** (see [`start_instances`]), no watcher tasks are spawned.
/// Instead, `clone.stop().await` is called directly on each successfully-started clone,
/// awaiting the real axum server `JoinHandle` before the error is returned.
/// (BC-2.06.017 Postcondition 6 / F-P1-MED-002)
///
/// (BC-2.06.017 Postcondition 1 v1.2 — D-1075-API-GAP-001)
#[non_exhaustive]
pub struct MultiInstanceServers {
    socket_map: HashMap<String, SocketAddr>,
    /// Per-instance admin tokens, keyed by `InstanceEntry::name`.
    ///
    /// Tokens are extracted from each `BoundInstance` via `clone.admin_token().to_string()`
    /// BEFORE the clone is moved into its detached watcher task (OWNERSHIP constraint:
    /// once moved into `tokio::spawn(async move { drop(clone); })`, the clone is unreachable).
    ///
    /// Used by `write_multi_admin_token_sidecar_to_path` (via `admin_token_map()`) so that
    /// `cmd_configure` can obtain per-clone tokens for the `X-Admin-Token` header
    /// (ADR-003 Amendment #5 / DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 T-02).
    token_map: HashMap<String, String>,
    /// Shared broadcast sender — owned EXCLUSIVELY by this handle.
    /// Each instance subscribed via `shutdown_tx.subscribe()` at bind time.
    /// Watcher tasks hold ONLY the clone object — NEVER an additional Sender
    /// clone (which would keep the channel open forever). Sending or dropping
    /// this sender wakes all axum graceful-shutdown receivers → drain → port release.
    shutdown_tx: broadcast::Sender<()>,
    /// Watcher task handles — retained to satisfy the D-1075 locked field layout.
    /// Each handle corresponds to a `tokio::spawn(async move { drop(clone); })` task.
    /// Because `ArmisClone` / `ClarotyClone` have no `Drop` impl, each watcher drops
    /// its clone and completes IMMEDIATELY — the internal axum server task was already
    /// DETACHED (dropping a `JoinHandle` detaches, never joins or aborts). These are
    /// no-op completion handles; they do NOT provide lifetime management for the servers.
    /// Real shutdown happens ONLY when `shutdown_tx.send(())` wakes the axum
    /// graceful-shutdown receiver each server holds from `shutdown_tx.subscribe()`.
    /// Dropped WITHOUT explicit abort on `Drop` (axum handles the graceful drain
    /// via the subscribed `shutdown_rx` inside each clone).
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

    /// Return the per-name admin token map.
    ///
    /// Each key is the `InstanceEntry::name` string; each value is the clone's
    /// admin token (extracted via `BehavioralClone::admin_token()` before the clone
    /// was moved into its detached watcher task).
    ///
    /// Used by `write_multi_admin_token_sidecar_to_path` in `multi_org_cmd.rs` to
    /// write the token sidecar so that `cmd_configure` can obtain per-clone tokens
    /// for the `X-Admin-Token` header (ADR-003 Amendment #5).
    ///
    /// (DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 T-02 / BC-2.06.017 Postcondition 1)
    pub fn admin_token_map(&self) -> &HashMap<String, String> {
        &self.token_map
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
            token_map: HashMap::new(),
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
    //
    // Watcher task spawning is DEFERRED until after the bind loop so that:
    //   (a) On the ERROR path we hold the clone and call `clone.stop().await`
    //       — the real axum server JoinHandle inside the clone is awaited,
    //       guaranteeing actual port release before the error is returned.
    //       (BC-2.06.017 Postcondition 6 / F-P1-MED-002)
    //   (b) On the SUCCESS path we spawn the watcher task as before, moving
    //       the clone in so it stays alive until `MultiInstanceServers` is dropped.
    struct BoundInstance {
        name: String,
        addr: SocketAddr,
        clone: Box<dyn BehavioralClone>,
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
                // Clone is alive and listening. Watcher task spawned on success path below.
                bound.push(BoundInstance {
                    name: entry.name.clone(),
                    addr,
                    clone,
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

    // --- Error path: stop each successfully-bound instance and return aggregated failures ---
    // (BC-2.06.017 Postcondition 6 — no zombie instances on bind failure)
    if !failures.is_empty() {
        // Signal via the shared sender — wakes all graceful-shutdown receivers inside axum.
        let _ = shutdown_tx.send(());
        // Call stop() on each successfully-bound clone.
        // stop() awaits the real axum server JoinHandle (with 250ms timeout + hard-abort fallback,
        // S-PERF-GATE-005), guaranteeing actual server termination and port release before we
        // return the error.
        // This replaces the prior watcher-task pattern which dropped clone immediately and
        // awaited a no-op task — providing no actual guarantee of server termination.
        // (F-P1-MED-002: BindFailure path determinism)
        for mut instance in bound {
            let _ = instance.clone.stop().await;
        }
        return Err(MultiInstanceBindError::BindFailure(failures));
    }

    // --- Success path: spawn watcher tasks and bundle into MultiInstanceServers handle ---
    // The shared shutdown_tx is owned EXCLUSIVELY by the returned handle.
    // Each clone already subscribed (via shutdown_tx.subscribe() above).
    // No extra Sender clones exist — the channel closes (waking all receivers)
    // when either shutdown() sends the signal or the handle is dropped.
    //
    // OWNERSHIP constraint (DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 T-02):
    // Admin tokens MUST be extracted from each BoundInstance BEFORE the clone is moved
    // into `tokio::spawn(async move { drop(clone); })`. Once moved, the clone is
    // unreachable from this call site — `clone.admin_token()` cannot be called after the move.
    // The token is a UUID v4 string; extracting it as `to_string()` here captures a copy
    // for the token_map without keeping any reference into the moved clone.
    //
    // Each clone is moved into a `tokio::spawn(async move { drop(clone); })` task.
    // Because ArmisClone / ClarotyClone have no Drop impl, drop(clone) merely drops
    // `server_handle: Option<JoinHandle<()>>`, which DETACHES (not aborts) the axum
    // server task. The server keeps running as a detached tokio task; the watcher
    // itself completes immediately. task_handles are no-op completion handles.
    // Real shutdown happens ONLY when shutdown_tx.send(()) (from Drop or shutdown())
    // wakes the axum graceful-shutdown receiver each server holds from subscribe().
    let mut socket_map = HashMap::with_capacity(bound.len());
    let mut token_map = HashMap::with_capacity(bound.len());
    let mut task_handles = Vec::with_capacity(bound.len());
    for instance in bound {
        // Extract admin token BEFORE moving clone into the watcher task (OWNERSHIP).
        // admin_token() returns &str; to_string() captures a copy before the move.
        let token = instance.clone.admin_token().to_string();
        let clone = instance.clone;
        // Spawn the watcher task AFTER confirming all binds succeeded.
        // drop(clone) detaches the internal JoinHandle — the axum server continues
        // running independently as a detached tokio task. This watcher finishes
        // immediately; shutdown is via shutdown_tx.send(()) only.
        let task_handle = tokio::spawn(async move {
            // drop(clone) drops server_handle, detaching (NOT aborting) the axum task.
            // The server stays alive until shutdown_tx.send(()) (from Drop or shutdown())
            // wakes the graceful-shutdown receiver from shutdown_tx.subscribe() at bind time.
            drop(clone);
        });
        socket_map.insert(instance.name.clone(), instance.addr);
        token_map.insert(instance.name, token);
        task_handles.push(task_handle);
    }

    Ok(MultiInstanceServers {
        socket_map,
        token_map,
        shutdown_tx,
        task_handles,
    })
}
