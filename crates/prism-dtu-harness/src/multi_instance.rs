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
//!   request counts (`Arc<AtomicUsize>`), not configure calls.
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
/// (BC-2.06.017 Postcondition 2)
#[non_exhaustive]
pub struct MultiInstanceHarness {
    // Fields suppressed until implementer populates them in `start()`.
    // All three are architect-locked (D-1075) and used by the implementation.
    #[allow(dead_code)]
    socket_map: HashMap<(String, String), SocketAddr>,
    shutdown_tx: broadcast::Sender<()>,
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
    /// for entry in entries.iter_mut() {
    ///     let bound = entry.clone.start_on(bind_addr, Some(shutdown_tx.subscribe()), None).await?;
    /// }
    /// ```
    ///
    /// (BC-2.06.017 Postcondition 2)
    pub async fn start(_entries: Vec<HarnessEntry>) -> Result<Self, HarnessError> {
        todo!(
            "S-DEMO-MULTI-TENANT-DTU-001: MultiInstanceHarness::start not yet implemented \
             (BC-2.06.017 Postcondition 2 — TDD Red Gate stub)"
        )
    }

    /// Return the per-`(org_slug, sensor_id)` socket address map.
    ///
    /// Keys are plain `(org_slug, sensor_id)` String pairs (U-004).
    /// Use this map with [`crate::overlay_wiring::write_overlay_temp_dir`] to
    /// construct per-org TOML overlay files for `SpecLoader::load_all`.
    ///
    /// (BC-2.06.017 Postcondition 2 — `socket_map()` returns `&HashMap<(String,String),SocketAddr>`)
    pub fn socket_map(&self) -> &HashMap<(String, String), SocketAddr> {
        todo!(
            "S-DEMO-MULTI-TENANT-DTU-001: MultiInstanceHarness::socket_map not yet implemented \
             (BC-2.06.017 Postcondition 2 — TDD Red Gate stub)"
        )
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
