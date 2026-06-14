//! Multi-instance harness for `prism-dtu-harness`.
//!
//! Provides [`MultiInstanceHarness`] and [`HarnessEntry`] for starting N named
//! DTU clone instances at distinct socket addresses keyed by
//! `(org_slug, sensor_id)` plain string pairs.
//!
//! # Story anchor
//!
//! S-DEMO-MULTI-TENANT-DTU-001 (BC-2.06.017 Postconditions 2, 4, 6, 7)
//!
//! # Key design decisions (D-1075)
//!
//! - `socket_map` key is `(String, String)` — plain `(org_slug, sensor_id)` strings,
//!   NOT `(OrgSlug, SensorId)` newtypes (U-004: lightweight test-infra key, distinct
//!   from the production `OrgKey = (OrgId, DtuType)`).
//! - `shutdown_tx: broadcast::Sender<()>` — shared sender; per-instance `.subscribe()`
//!   issued at bind time so each clone receives the shutdown signal.
//! - `task_handles: Vec<JoinHandle<()>>` — one handle per started instance.
//! - `impl Drop` sends shutdown signal without explicit task abort; axum's
//!   `with_graceful_shutdown` drains in-flight requests (~5s). Matches existing
//!   `DemoHarness::drop` pattern.
//! - Admin-token map is OMITTED for this story: routing isolation is verified via
//!   the server-side per-instance request counter (Armis `GET /dtu/request-count`),
//!   not configure calls.
//! - Watcher task spawning is DEFERRED until after all bind operations succeed.
//!   On the ERROR path, `clone.stop().await` is called directly on each successfully-
//!   started clone — awaiting the real axum server `JoinHandle` before the error is
//!   returned (BC-2.06.017 Postcondition 6 / F-P1-MED-002).
//!   On the SUCCESS path, each clone is moved into a `tokio::spawn(async move { drop(clone); })`
//!   task. Because `ArmisClone` / `ClarotyClone` have no `Drop` impl, dropping the clone
//!   merely drops its `server_handle: Option<JoinHandle<()>>`, which DETACHES (does not
//!   abort) the internal axum server task. The server keeps running as a detached task;
//!   the watcher task itself completes immediately. Shutdown happens only when
//!   `shutdown_tx.send(())` (sent by `impl Drop` when the `MultiInstanceHarness` is dropped) wakes
//!   the axum graceful-shutdown receiver each server holds from `shutdown_tx.subscribe()`.
//!
//! # Perimeter constraint (BC-2.06.017 INV-PERIMETER-001)
//!
//! This module MUST NOT import `prism-spec-engine`, `prism-sensors`, or
//! `prism-query`. `src/` code only uses `Box<dyn BehavioralClone>` from
//! `prism-dtu-common`. Clone types (`ArmisClone`, `ClarotyClone`) are only
//! referenced in `tests/` via `[dev-dependencies]`.

use std::{collections::HashMap, net::SocketAddr};

use tokio::{sync::broadcast, task::JoinHandle};

use prism_dtu_common::BehavioralClone;

use crate::error::HarnessError;

/// A single named harness entry for [`MultiInstanceHarness::start`].
///
/// Each entry pairs an `(org_slug, sensor_id)` identity with a
/// `Box<dyn BehavioralClone>` instance. The `clone` field is consumed
/// mutably during `start` (U-001: `BehavioralClone::start_on` takes `&mut self`).
///
/// (BC-2.06.017 Postcondition 2)
#[non_exhaustive]
pub struct HarnessEntry {
    /// Org slug string — becomes the first element of the `socket_map` key.
    pub org_slug: String,
    /// Sensor ID string — becomes the second element of the `socket_map` key.
    pub sensor_id: String,
    /// The clone instance to start. Consumed mutably during `start_on` (U-001).
    ///
    /// `src/` code never names concrete clone types; only `tests/` does (U-002).
    pub clone: Box<dyn BehavioralClone>,
}

impl HarnessEntry {
    /// Construct a `HarnessEntry` with the given org slug, sensor ID, and clone.
    ///
    /// This is the canonical constructor for external callers (required because
    /// `#[non_exhaustive]` prevents struct-literal construction outside this crate).
    pub fn new(
        org_slug: impl Into<String>,
        sensor_id: impl Into<String>,
        clone: Box<dyn BehavioralClone>,
    ) -> Self {
        Self {
            org_slug: org_slug.into(),
            sensor_id: sensor_id.into(),
            clone,
        }
    }
}

/// Multi-instance harness managing N DTU clone instances started at distinct
/// ephemeral socket addresses, keyed by `(org_slug, sensor_id)` pairs.
///
/// # Field layout (D-1075 architect-locked)
///
/// - `socket_map`: `HashMap<(String, String), SocketAddr>` — keys are plain
///   `(org_slug, sensor_id)` string pairs (U-004; NOT `(OrgId, DtuType)` newtypes).
/// - `shutdown_tx`: `broadcast::Sender<()>` — shared sender; `.subscribe()` called
///   per instance at bind time.
/// - `task_handles`: `Vec<JoinHandle<()>>` — one per started instance (dropped
///   without explicit abort on `Drop`; axum drains in-flight requests gracefully).
///
/// # Drop behaviour (EC-017-005)
///
/// `impl Drop` sends `shutdown_tx.send(())` (best-effort; error ignored) then
/// drops `task_handles` without explicit abort. Axum `with_graceful_shutdown`
/// handles the ~5s drain on the clone side. Matches the existing `DemoHarness`
/// drop pattern.
///
/// # Error-path shutdown guarantee (BC-2.06.017 Postcondition 6 / F-P1-MED-002)
///
/// If any bind fails, `start` calls `clone.stop().await` on each successfully-started
/// clone — awaiting the real axum server `JoinHandle` — before returning the error.
/// This guarantees actual port release (no zombie instances), replacing the prior
/// pattern of a dummy watcher task + 100ms sleep which provided no such guarantee.
///
/// (BC-2.06.017 Postcondition 2)
#[non_exhaustive]
pub struct MultiInstanceHarness {
    socket_map: HashMap<(String, String), SocketAddr>,
    shutdown_tx: broadcast::Sender<()>,
    /// Watcher task handles — held to satisfy D-1075 locked field layout.
    ///
    /// Each handle corresponds to a `tokio::spawn(async move { drop(clone); })` task
    /// spawned on the SUCCESS path (AFTER all binds succeed). Because `ArmisClone` /
    /// `ClarotyClone` have no `Drop` impl, each watcher drops its clone and completes
    /// IMMEDIATELY — the internal axum server task was already DETACHED (dropping a
    /// `JoinHandle` detaches, never joins or aborts). These handles are therefore
    /// no-op completion handles; they do NOT provide lifetime management for the
    /// running server tasks.
    ///
    /// The servers remain alive as detached tokio tasks. Real shutdown happens only
    /// when `shutdown_tx.send(())` (sent by `impl Drop` when the harness is dropped)
    /// wakes the axum graceful-shutdown receiver each server holds from
    /// `shutdown_tx.subscribe()` at bind time.
    ///
    /// On the ERROR path, `clone.stop().await` is called directly — no watcher tasks
    /// are spawned. Dropped without explicit abort on `Drop` (EC-017-005).
    #[allow(dead_code)]
    task_handles: Vec<JoinHandle<()>>,
}

impl MultiInstanceHarness {
    /// Start N DTU clone instances at ephemeral addresses, returning a harness
    /// whose `socket_map` maps `(org_slug, sensor_id)` → `SocketAddr`.
    ///
    /// # Duplicate key check
    ///
    /// If two entries share the same `(org_slug, sensor_id)` pair, returns
    /// `Err(HarnessError::DuplicateKey { org_slug, sensor_id })` immediately
    /// before any clone instance is started (BC-2.06.017 Postcondition 7 / EC-017-003).
    ///
    /// # Bind errors (multi-error aggregation)
    ///
    /// If one or more clone `start_on` calls fail, all bind operations are attempted
    /// before returning `Err(HarnessError::BindFailure(failures))` (INV-ERR-003-COMPAT).
    /// Successfully-started instances are shut down before the error is returned.
    ///
    /// # Bind-loop call form (no-tls path, &mut self receiver — U-001)
    ///
    /// ```ignore
    /// for mut entry in entries {
    ///     let bound = entry.clone.start_on(bind_addr, Some(shutdown_tx.subscribe()), None).await?;
    /// }
    /// ```
    ///
    /// Clones are moved by value into the per-instance bound entry so that the error path
    /// can call `clone.stop().await` (deterministic port release, Postcondition 6); the
    /// `&mut self` receiver on `start_on` is satisfied via the moved `mut entry` binding.
    ///
    /// (BC-2.06.017 Postcondition 2)
    pub async fn start(entries: Vec<HarnessEntry>) -> Result<Self, HarnessError> {
        // --- Duplicate-key check (BC-2.06.017 Postcondition 7 / EC-017-003) ---
        // Must happen BEFORE any bind attempts.
        let mut seen_keys = std::collections::HashSet::new();
        for entry in &entries {
            let key = (entry.org_slug.clone(), entry.sensor_id.clone());
            if !seen_keys.insert(key) {
                return Err(HarnessError::DuplicateKey {
                    org_slug: entry.org_slug.clone(),
                    sensor_id: entry.sensor_id.clone(),
                });
            }
        }

        // Shared broadcast channel: capacity = number of entries (minimum 1).
        // Each clone subscribes once via shutdown_tx.subscribe() at bind time.
        // When impl Drop fires shutdown_tx.send(()), all clone receivers wake → graceful shutdown.
        let (shutdown_tx, _) = broadcast::channel::<()>(entries.len().max(1));

        // --- Multi-error aggregation bind loop (INV-ERR-003-COMPAT) ---
        // Attempt ALL binds before returning any error.
        //
        // Watcher task spawning is DEFERRED until after the bind loop so that:
        //   (a) On the ERROR path we hold the clone and call `clone.stop().await`
        //       — the real axum server JoinHandle inside the clone is awaited,
        //       guaranteeing actual port release before the error is returned.
        //       (BC-2.06.017 Postcondition 6 / F-P1-MED-002)
        //   (b) On the SUCCESS path we spawn the watcher task moving the clone in
        //       so it stays alive until `MultiInstanceHarness` is dropped.
        struct BoundEntry {
            key: (String, String),
            addr: SocketAddr,
            clone: Box<dyn BehavioralClone>,
        }

        let mut bound: Vec<BoundEntry> = Vec::with_capacity(entries.len());
        let mut failures: Vec<crate::error::BindError> = Vec::new();

        // OS-assigned ephemeral port on loopback — SocketAddr literal for zero port.
        // Using a const SocketAddr avoids parsing at runtime (no expect() needed).
        const EPHEMERAL_LOOPBACK: SocketAddr =
            SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 0);

        // Consume entries so we can move clones into BoundEntry or stop() them.
        for mut entry in entries {
            let shutdown_rx = shutdown_tx.subscribe();
            let key = (entry.org_slug.clone(), entry.sensor_id.clone());

            match entry
                .clone
                .start_on(EPHEMERAL_LOOPBACK, Some(shutdown_rx), None)
                .await
            {
                Ok(addr) => {
                    // Clone is alive and listening. Watcher task spawned on success path below.
                    bound.push(BoundEntry {
                        key,
                        addr,
                        clone: entry.clone,
                    });
                }
                Err(e) => {
                    let io_err = std::io::Error::other(e);
                    failures.push(crate::error::BindError {
                        org_slug: entry.org_slug,
                        sensor_id: entry.sensor_id,
                        source: io_err,
                    });
                }
            }
        }

        // --- Error path: stop each successfully-started clone, return aggregated failures ---
        // (BC-2.06.017 Postcondition 6 — no zombie instances on bind failure)
        if !failures.is_empty() {
            // Signal via the shared sender — wakes all graceful-shutdown receivers inside axum.
            let _ = shutdown_tx.send(());
            // Call stop() on each successfully-bound clone.
            // stop() awaits the real axum server JoinHandle (with 5s timeout + hard-abort fallback),
            // guaranteeing actual server termination and port release before we return the error.
            // This replaces the prior pattern of a dummy task + 100ms sleep, which provided
            // no actual guarantee of server termination. (F-P1-MED-002)
            for mut entry in bound {
                let _ = entry.clone.stop().await;
            }
            return Err(HarnessError::BindFailure(failures));
        }

        // --- Success path: spawn watcher tasks and build socket_map + task_handles ---
        // Each clone is moved into a `tokio::spawn(async move { drop(clone); })` task.
        // Because ArmisClone / ClarotyClone have no Drop impl, dropping the clone merely
        // drops its `server_handle: Option<JoinHandle<()>>`, which DETACHES (does not
        // abort) the internal axum server task. The server keeps running as a detached
        // tokio task; the watcher itself completes immediately.
        // The `task_handles` vec holds no-op completion handles (not lifetime handles).
        // Real shutdown happens ONLY when shutdown_tx.send(()) (from Drop) wakes the
        // axum graceful-shutdown receiver each server holds from shutdown_tx.subscribe().
        let mut socket_map = HashMap::with_capacity(bound.len());
        let mut task_handles: Vec<JoinHandle<()>> = Vec::with_capacity(bound.len());
        for entry in bound {
            let clone = entry.clone;
            // Spawn the watcher task AFTER confirming all binds succeeded.
            // Dropping the clone detaches (not aborts) its internal JoinHandle — the axum
            // server task continues running independently. The watcher finishes immediately.
            let task_handle = tokio::spawn(async move {
                // drop(clone) detaches the internal server JoinHandle.
                // The axum server keeps running until shutdown_tx.send(()) (from Drop)
                // wakes the graceful-shutdown receiver the server holds from
                // shutdown_tx.subscribe() at bind time.
                drop(clone);
            });
            socket_map.insert(entry.key, entry.addr);
            task_handles.push(task_handle);
        }

        Ok(Self {
            socket_map,
            shutdown_tx,
            task_handles,
        })
    }

    /// Return the per-`(org_slug, sensor_id)` socket address map.
    ///
    /// Keys are plain `(org_slug, sensor_id)` String pairs (U-004).
    /// Use this map with [`crate::overlay_wiring::write_overlay_temp_dir`] to
    /// construct per-org TOML overlay files for `SpecLoader::load_all`.
    ///
    /// (BC-2.06.017 Postcondition 2 — `socket_map()` returns `&HashMap<(String,String),SocketAddr>`)
    pub fn socket_map(&self) -> &HashMap<(String, String), SocketAddr> {
        &self.socket_map
    }
}

impl Drop for MultiInstanceHarness {
    /// Send the graceful-shutdown signal and drop task handles.
    ///
    /// The shutdown signal is sent best-effort (error ignored). Task handles are
    /// dropped WITHOUT explicit abort — axum's `with_graceful_shutdown` drains
    /// in-flight requests on the clone side (EC-017-005).
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
        // task_handles dropped here by RAII; no explicit abort.
    }
}
