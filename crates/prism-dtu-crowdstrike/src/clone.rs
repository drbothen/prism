//! `CrowdstrikeClone` — implements [`BehavioralClone`] for the CrowdStrike Falcon API DTU.
//!
//! # ADR-002 Amendment #2 (TD-WV1-04)
//!
//! `start_on` accepts an optional `RustlsConfig` as its third argument.
//! When `Some(cfg)` and the `tls` feature is active, the clone binds via
//! `axum_server::bind_rustls` and serves HTTPS.  When `None`, plain axum HTTP
//! is used (backward-compatible default).

use std::{net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use prism_dtu_common::{BehavioralClone, StubConfig};
use tokio::{sync::broadcast, task::JoinHandle};

use crate::{routes::build_router, state::CrowdstrikeState};

/// L4-adversarial behavioral clone of the CrowdStrike Falcon API.
///
/// Maintains stateful write stores (containment, detection status) and a session-scoped
/// ID registry for two-step pagination. Supports configurable failure injection via the
/// shared `FailureLayer` from `prism-dtu-common`.
///
/// Binds to `127.0.0.1:0` (ephemeral port) on `start()`.
pub struct CrowdstrikeClone {
    pub config: StubConfig,
    pub state: Arc<CrowdstrikeState>,
    pub server_handle: Option<JoinHandle<()>>,
    pub bound_addr: Option<SocketAddr>,
    /// True when the server is currently bound via TLS (axum_server::bind_rustls).
    tls_active: bool,
    /// `axum_server::Handle` retained for graceful shutdown of TLS servers.
    /// Stored so `stop()` can call `handle.graceful_shutdown()` rather than
    /// relying on the broadcast signal (which is not wired to axum_server).
    #[cfg(feature = "tls")]
    tls_handle: Option<axum_server::Handle>,
    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    admin_token: String,
}

impl CrowdstrikeClone {
    /// Create a new clone with default `StubConfig` and empty state stores.
    pub fn new() -> Self {
        let admin_token = uuid::Uuid::new_v4().to_string();
        Self {
            config: StubConfig::default(),
            state: Arc::new(CrowdstrikeState::with_admin_token(admin_token.clone())),
            server_handle: None,
            bound_addr: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
        }
    }

    /// Create with explicit config.
    pub fn with_config(config: StubConfig) -> Self {
        let admin_token = uuid::Uuid::new_v4().to_string();
        Self {
            config,
            state: Arc::new(CrowdstrikeState::with_admin_token(admin_token.clone())),
            server_handle: None,
            bound_addr: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
        }
    }
}

impl Default for CrowdstrikeClone {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Story A: new_with_seed constructor stub (BC-2.06.018 / ADR-036 §2.3)
// ---------------------------------------------------------------------------

#[cfg(feature = "fixture-gen")]
impl CrowdstrikeClone {
    /// Construct a `CrowdstrikeClone` with deterministic fixture data generated at
    /// construction time from `(seed, archetype, org_id)`.
    ///
    /// Calls `generate(org_id, archetype, GenOpts { seed, ..GenOpts::default() })`
    /// under `#[cfg(feature = "fixture-gen")]`, stores the resulting records in
    /// `generated_devices` / `generated_detections` in state.
    ///
    /// Sets `state.fixture_gen_seeded = true`. Route handlers check this flag (not
    /// `generated_devices.is_empty()` / `generated_detections.is_empty()`) as the
    /// dual-path sentinel so that `Archetype::DormantTenant` (seeded=true, 0 records)
    /// serves EMPTY — it does NOT fall back to the static embedded JSON.
    /// F-P6-HIGH-001 / F-P10-HIGH-001 / ADR-036 v2.2.
    ///
    /// `CrowdstrikeClone::new()` is unchanged (backward-compatible, ADR-036 §2.5);
    /// it leaves `fixture_gen_seeded = false` and route handlers use the static fixture.
    ///
    /// ADR-036 §2.3: `new_with_seed` calls `generate()` ONCE at construction;
    /// route handlers MUST NOT call `generate()` per-request.
    ///
    /// ADR-036 v2.2: canonical 3-arg form — `archetype` is forwarded to `generate()`;
    /// NO hardcoded archetype inside this constructor.
    pub fn new_with_seed(
        seed: u64,
        archetype: prism_dtu_common::Archetype,
        org_id: prism_dtu_common::OrgId,
    ) -> Self {
        // Default anchor: fixed demo-era constant (review-2026-06-10 P1-01).
        Self::new_with_seed_anchored(
            seed,
            archetype,
            org_id,
            prism_dtu_common::demo_time_anchor(),
        )
    }

    /// `new_with_seed` with an explicit `time_anchor` for generated timestamps.
    ///
    /// Review-2026-06-10 P1-01: exposes the `GenOpts::time_anchor` input on the
    /// construction chain so callers can anchor generated data to a chosen era.
    /// Story B (S-DEMO-DTU-LIVE-SCENARIO-001-B) wires `scenario_start_secs` →
    /// `time_anchor`; this fix-burst deliberately does NOT wire
    /// `ScenarioConfig.scenario_start_secs` into generation (BC-2.06.019 scope).
    pub fn new_with_seed_anchored(
        seed: u64,
        archetype: prism_dtu_common::Archetype,
        org_id: prism_dtu_common::OrgId,
        time_anchor: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        use crate::generator::generate;
        use prism_dtu_common::GenOpts;

        let opts = GenOpts {
            seed,
            time_anchor,
            ..GenOpts::default()
        };
        let fixture = generate(org_id, archetype, opts);

        // Split records by _record_type discriminator (ADR-036 §2.3).
        // "device" records go to generated_devices; "detection" records go to
        // generated_detections. id_page / tombstone / oauth2_token records are
        // routing artifacts and not served by the dual-path handler.
        let mut generated_devices = Vec::new();
        let mut generated_detections = Vec::new();
        for record in fixture.records {
            match record.get("_record_type").and_then(|v| v.as_str()) {
                Some("device") => generated_devices.push(record),
                Some("detection") => generated_detections.push(record),
                _ => {} // id_page, tombstone, oauth2_token — not served via dual-path
            }
        }

        let admin_token = uuid::Uuid::new_v4().to_string();
        let mut state = CrowdstrikeState::with_admin_token(admin_token.clone());
        state.generated_devices = generated_devices;
        state.generated_detections = generated_detections;
        // Mark as seeded so route handlers use the generated path (even for DormantTenant
        // which produces 0 records). F-P6-HIGH-001 / ADR-036 v2.2.
        state.fixture_gen_seeded = true;

        Self {
            config: prism_dtu_common::StubConfig::default(),
            state: Arc::new(state),
            server_handle: None,
            bound_addr: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
        }
    }

    // -----------------------------------------------------------------------
    // Story B: new_with_scenario constructor (BC-2.06.019 / ADR-036 v2.3 §2.4)
    // -----------------------------------------------------------------------

    /// Construct a `CrowdstrikeClone` with the scenario timeline layer.
    ///
    /// 5-arg form per ADR-036 v2.3 §2.4. Internally calls
    /// `new_with_seed_anchored(seed, archetype, org_id, time_anchor)` (NOT the 3-arg
    /// `new_with_seed` which would produce stale timestamps for a June 2026 demo).
    ///
    /// Sets `state.timeline = Some(Arc::clone(&timeline))` so route handlers can
    /// compute the current stage and apply StageMask filtering.
    ///
    /// `time_anchor` is derived ONCE in `build_clone_pairs` from
    /// `scenario_start_epoch_secs` via `DateTime::from_timestamp`.
    pub fn new_with_scenario(
        seed: u64,
        archetype: prism_dtu_common::Archetype,
        org_id: prism_dtu_common::OrgId,
        timeline: std::sync::Arc<prism_dtu_common::IncidentTimeline>,
        time_anchor: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        // Call new_with_seed_anchored (NOT the 3-arg new_with_seed) to use the
        // caller-supplied time_anchor for era-coherent generated timestamps.
        // ADR-036 v2.3 §2.3: the 3-arg path anchors at demo_time_anchor() = 2026-01-01
        // which is stale for a June 2026 demo. Forbidden pattern per story spec.
        let mut clone = Self::new_with_seed_anchored(seed, archetype, org_id, time_anchor);
        // Attach the timeline so route handlers can compute the current stage index.
        // ADR-036 v2.3 §2.3: Arc<IncidentTimeline> is read-only after construction.
        // SAFETY: Arc::get_mut only fails if there are other references; we just
        // constructed the Arc in new_with_seed_anchored and have the sole reference.
        if let Some(state) = Arc::get_mut(&mut clone.state) {
            state.timeline = Some(Arc::clone(&timeline));
        }
        clone
    }
}

#[async_trait]
impl BehavioralClone for CrowdstrikeClone {
    /// Start with an explicit bind address, optional graceful-shutdown receiver, and
    /// optional TLS configuration.
    ///
    /// Returns the bound `SocketAddr`. Wires the shutdown receiver into
    /// `axum::serve(...).with_graceful_shutdown(...)` for graceful drain.
    ///
    /// When `tls` is `Some`, binds via `axum_server::bind_rustls` (HTTPS).
    /// When `None`, uses plain `axum::serve` (HTTP).
    async fn start_on(
        &mut self,
        bind: SocketAddr,
        shutdown: Option<broadcast::Receiver<()>>,
        #[cfg(feature = "tls")] tls: Option<Arc<axum_server::tls_rustls::RustlsConfig>>,
        #[cfg(not(feature = "tls"))] tls: Option<()>,
    ) -> anyhow::Result<SocketAddr> {
        // Propagate seed from StubConfig into RuntimeConfig so route handlers see it.
        {
            // SAFETY: mutex is only poisoned if a previous holder panicked while holding it,
            // which cannot happen in this single-threaded initialisation path.
            #[allow(clippy::expect_used)]
            let mut rc = self
                .state
                .runtime_config
                .lock()
                .expect("runtime_config poisoned");
            rc.seed = self.config.seed;
        }

        let router = build_router(
            Arc::clone(&self.state),
            self.config.failure_mode.clone(),
            self.config.latency_ms,
        );

        #[cfg(feature = "tls")]
        if let Some(rustls_cfg) = tls {
            // TLS path: bind via axum_server::bind_rustls.
            let handle = axum_server::Handle::new();
            let handle_clone = handle.clone();
            let server_task = tokio::spawn(async move {
                // SAFETY: server crash inside the task should propagate as a fatal error;
                // panic here is intentional and surfaces the root cause immediately.
                #[allow(clippy::expect_used)]
                axum_server::bind_rustls(bind, (*rustls_cfg).clone())
                    .handle(handle_clone)
                    .serve(router.into_make_service())
                    .await
                    .expect("CrowdstrikeClone TLS server crashed");
            });

            // Wait for the server to report its bound address.
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("CrowdstrikeClone TLS server failed to start"))?;

            self.bound_addr = Some(addr);
            self.tls_active = true;
            self.server_handle = Some(server_task);
            // Retain handle so stop() can call graceful_shutdown() (MEDIUM-001 fix).
            self.tls_handle = Some(handle);
            return Ok(addr);
        }

        // Plain HTTP path (also the no-tls feature path).
        let _ = tls; // consume no-tls Option<()> without warning
        let listener = tokio::net::TcpListener::bind(bind)
            .await
            .map_err(|e| anyhow::anyhow!("failed to bind listener on {bind}: {e}"))?;

        let addr = listener
            .local_addr()
            .map_err(|e| anyhow::anyhow!("failed to get local addr: {e}"))?;

        self.bound_addr = Some(addr);
        self.tls_active = false;

        let handle = tokio::spawn(async move {
            let server = axum::serve(listener, router);
            if let Some(mut rx) = shutdown {
                let serve_future = server.with_graceful_shutdown(async move {
                    let _ = rx.recv().await;
                });
                // SAFETY: server task panic is fatal; surfacing it immediately is correct.
                #[allow(clippy::expect_used)]
                serve_future.await.expect("CrowdstrikeClone server crashed");
            } else {
                // SAFETY: same as above — server task panic must surface immediately.
                #[allow(clippy::expect_used)]
                server.await.expect("CrowdstrikeClone server crashed");
            }
        });

        self.server_handle = Some(handle);
        Ok(addr)
    }

    /// Stop the server: graceful drain then hard-abort fallback for both TLS and HTTP.
    ///
    /// # TD-WV1-04-FU-001 — shutdown symmetry
    ///
    /// Both TLS and HTTP paths now use the same graceful-drain-then-abort pattern:
    ///
    /// - **TLS path**: signals `axum_server::Handle::graceful_shutdown(5s)` to begin
    ///   draining, then awaits the `JoinHandle` up to 5 s before hard-aborting.
    /// - **HTTP path**: the harness broadcast signal has already been sent before
    ///   `stop()` is called, so axum's `with_graceful_shutdown` future is already
    ///   resolving. We await the `JoinHandle` up to 5 s before hard-aborting —
    ///   matching the TLS drain window instead of the previous immediate abort.
    async fn stop(&mut self) -> anyhow::Result<()> {
        // TLS path: signal graceful shutdown via the retained axum_server::Handle.
        #[cfg(feature = "tls")]
        if let Some(h) = self.tls_handle.take() {
            h.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
        }

        // Both paths: attempt graceful drain; hard-abort after 5s.
        if let Some(mut handle) = self.server_handle.take() {
            tokio::select! {
                _ = &mut handle => {
                    // Server task completed within the drain window — clean shutdown.
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                    // Drain window expired — hard-abort the server task.
                    handle.abort();
                }
            }
        }

        self.tls_active = false;
        Ok(())
    }

    /// Reset all captured state: clears containment store, detection status store,
    /// and session registry. Does NOT change `RuntimeConfig` (auth_mode, seed, etc.).
    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    /// Reconfigure the stub at runtime.
    ///
    /// Accepts JSON such as `{"auth_mode": "reject"}`. Delegates to
    /// `CrowdstrikeState::apply_config`.
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)
    }

    fn bound_addr(&self) -> SocketAddr {
        // SAFETY: callers are required to call start() before bound_addr(); the expect
        // message documents the contract violation — this is a programming error, not runtime.
        #[allow(clippy::expect_used)]
        self.bound_addr
            .expect("CrowdstrikeClone::bound_addr called before start()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }

    fn admin_token(&self) -> &str {
        &self.admin_token
    }
}
