//! `CyberintClone` — `BehavioralClone` implementation for the Cyberint API DTU.
//!
//! Lifecycle:
//! 1. `CyberintClone::new()` — allocates state; loads fixtures from crate root.
//! 2. `start()` — binds an ephemeral TCP port, builds the axum router, spawns the server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — delegates to `state.reset()`.
//! 5. `configure()` — delegates to `state.apply_config()`.
//!
//! # ADR-002 Amendment #2 (TD-WV1-04)
//!
//! `start_on` accepts an optional `RustlsConfig` as its third argument.
//! When `Some(cfg)` and the `tls` feature is active, the clone binds via
//! `axum_server::bind_rustls` and serves HTTPS.  When `None`, plain axum HTTP
//! is used (backward-compatible default).

use std::{net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use axum::{
    routing::{get, patch, post},
    Router,
};
use prism_core::OrgId;
// NOTE: `post` is retained for `/api/v1/alerts` POST route (get_alerts also accepts POST)
// and `/api/v1/alerts/:id/close` (post_close_alert). Do not remove.
use prism_dtu_common::BehavioralClone;
use tokio::{sync::broadcast, task::JoinHandle};

use crate::{
    routes::{
        alerts::{get_alert_by_id, get_alerts, patch_alert_status, post_close_alert},
        dtu::{get_health, post_configure, post_reset},
        threats::get_threat_intel,
    },
    state::CyberintState,
    types::Alert,
};

/// L2-fidelity behavioral clone of the Cyberint API.
pub struct CyberintClone {
    /// Shared mutable state — public to allow test inspection of `generated_records`
    /// (fixture-gen Red Gate tests) and `instance_org_id` (org-isolation tests).
    pub state: Arc<CyberintState>,
    bound_addr: Option<SocketAddr>,
    server_handle: Option<JoinHandle<()>>,
    /// True when the server is currently bound via TLS (axum_server::bind_rustls).
    tls_active: bool,
    /// `axum_server::Handle` retained for graceful shutdown of TLS servers (MEDIUM-001).
    #[cfg(feature = "tls")]
    tls_handle: Option<axum_server::Handle>,
    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    admin_token: String,
    /// Internal broadcast sender for graceful HTTP shutdown (S-PERF-GATE-005).
    ///
    /// Set by `start_on` when `shutdown=None` (the direct `start()` path). Fired by
    /// `stop()` so the server task drains immediately instead of timing out on the
    /// 5s select! fallback. `None` when the harness-provided or TLS path is used.
    internal_shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
    /// Organisation this clone instance is bound to (BC-3.2.001).
    ///
    /// S-3.2.04 stub: set to a freshly-minted OrgId on `new()`. The implementation
    /// phase will accept this as a constructor parameter and thread it into route
    /// handlers via request-context extraction.
    #[allow(dead_code)]
    org_id: OrgId,
}

impl CyberintClone {
    /// Create a new `CyberintClone` with an initial access token registered in the allowlist.
    ///
    /// Equivalent to `CyberintClone::new()` followed by `register_access_token(token)`,
    /// but avoids needing a separate HTTP configure call or admin token.
    ///
    /// Used by `build_clone_pairs` when `initial_access_token` is set in the demo config
    /// (ADR-031 §D3-a; E2E test harness provision pattern).
    pub fn new_with_access_token(access_token: String) -> anyhow::Result<Self> {
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let alerts: Vec<Alert> = prism_dtu_common::load_fixture_as(crate_dir, "alerts")?;
        let alerts_page2: Vec<Alert> =
            prism_dtu_common::load_fixture_as(crate_dir, "alerts-page2")?;
        let threats: Vec<serde_json::Value> =
            prism_dtu_common::load_fixture_as(crate_dir, "threats")?;

        let admin_token = uuid::Uuid::new_v4().to_string();
        let org_id = OrgId::new();
        let state = Arc::new(
            CyberintState::with_org_id_and_admin_token(
                org_id,
                alerts,
                alerts_page2,
                threats,
                admin_token.clone(),
            )
            .with_demo_token(access_token),
        );
        Ok(Self {
            state,
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
            internal_shutdown_tx: None,
            org_id,
        })
    }

    /// Create a new `CyberintClone`. Loads fixtures from the crate root.
    pub fn new() -> anyhow::Result<Self> {
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let alerts: Vec<Alert> = prism_dtu_common::load_fixture_as(crate_dir, "alerts")?;
        let alerts_page2: Vec<Alert> =
            prism_dtu_common::load_fixture_as(crate_dir, "alerts-page2")?;
        let threats: Vec<serde_json::Value> =
            prism_dtu_common::load_fixture_as(crate_dir, "threats")?;

        let admin_token = uuid::Uuid::new_v4().to_string();
        // S-3.2.04 stub: mint a fresh OrgId for this clone instance.
        // The implementation phase will accept org_id as a constructor parameter.
        let org_id = OrgId::new();
        let state = Arc::new(CyberintState::with_org_id_and_admin_token(
            org_id,
            alerts,
            alerts_page2,
            threats,
            admin_token.clone(),
        ));
        Ok(Self {
            state,
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
            internal_shutdown_tx: None,
            org_id,
        })
    }

    // -----------------------------------------------------------------------
    // Story A: new_with_seed constructor stub (BC-2.06.018 / ADR-036 §2.3)
    // -----------------------------------------------------------------------

    /// Construct a `CyberintClone` with deterministic fixture data generated at
    /// construction time from `(seed, archetype, org_id)`.
    ///
    /// Gated `#[cfg(feature = "fixture-gen")]`.
    ///
    /// This constructor is **fallible** — mirrors `CyberintClone::new() -> anyhow::Result<Self>`.
    /// `build_clone_pairs` propagates the error via `?`.
    ///
    /// `CyberintClone::new()` is unchanged (backward-compatible, ADR-036 §2.5).
    ///
    /// ADR-036 v2.2: canonical 3-arg form — `archetype` is forwarded to `generate()`;
    /// NO hardcoded archetype inside this constructor.
    #[cfg(feature = "fixture-gen")]
    pub fn new_with_seed(
        seed: u64,
        archetype: prism_dtu_common::Archetype,
        org_id: prism_dtu_common::OrgId,
    ) -> anyhow::Result<Self> {
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
    #[cfg(feature = "fixture-gen")]
    pub fn new_with_seed_anchored(
        seed: u64,
        archetype: prism_dtu_common::Archetype,
        org_id: prism_dtu_common::OrgId,
        time_anchor: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<Self> {
        use crate::generator::generate;
        use prism_dtu_common::GenOpts;

        let opts = GenOpts {
            seed,
            time_anchor,
            ..GenOpts::default()
        };
        let fixture = generate(&org_id, archetype, &opts);

        // Load static fixtures (required for alert_fixture / alert_store initialization).
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let alerts: Vec<crate::types::Alert> =
            prism_dtu_common::load_fixture_as(crate_dir, "alerts")?;
        let alerts_page2: Vec<crate::types::Alert> =
            prism_dtu_common::load_fixture_as(crate_dir, "alerts-page2")?;
        let threats: Vec<serde_json::Value> =
            prism_dtu_common::load_fixture_as(crate_dir, "threats")?;

        let admin_token = uuid::Uuid::new_v4().to_string();
        // Use prism_core::OrgId::new() to get a fresh OrgId for the clone instance.
        let instance_org_id = OrgId::new();
        let mut state = CyberintState::with_org_id_and_admin_token(
            instance_org_id,
            alerts,
            alerts_page2,
            threats,
            admin_token.clone(),
        );
        state.generated_records = fixture.records;
        // Mark as seeded so route handlers use the generated path (even for DormantTenant
        // which produces 0 records). F-P6-HIGH-001 / ADR-036 v2.2.
        state.fixture_gen_seeded = true;

        Ok(Self {
            state: Arc::new(state),
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
            internal_shutdown_tx: None,
            org_id: instance_org_id,
        })
    }

    // -----------------------------------------------------------------------
    // Story B: new_with_scenario constructor (BC-2.06.019 / ADR-036 v2.3 §2.4)
    // -----------------------------------------------------------------------

    /// Construct a `CyberintClone` with the scenario timeline layer and catalog CVE injection.
    ///
    /// 6-arg fallible form per ADR-036 v2.3 §2.4 + BC-2.06.020 PC-8.
    /// Gated `#[cfg(feature = "fixture-gen")]` because `chrono::DateTime<Utc>` is only
    /// available under `fixture-gen` in this crate (dep:chrono gating in Cargo.toml).
    ///
    /// Internally calls `generate_with_catalog` so every CVE-surface record's `cve_id`
    /// is drawn from `catalog.device_cves` (cyclic assignment) instead of a synthetic
    /// `CVE-9999-*` baseline value. The RNG draw count is preserved (BC-3.4.001).
    ///
    /// Sets `state.timeline = Some(Arc::clone(&timeline))` so route handlers
    /// compute the current stage index and apply StageMask filtering per request.
    /// `get_alerts` implements the three-way composition: scenario path applies
    /// BC-2.06.019 PC-4 IOC-reference filtering; seeded path (no timeline) serves
    /// all generated alert records unchanged (BC-2.06.018). (BPRL-P2-01)
    ///
    /// BC-2.06.020 PC-8 + INV-CYBERINT-ALERT-CVE-CORRELATION-001: every `cve_id` on
    /// every CVE-surface record will be a member of `catalog.device_cves`, enabling
    /// the analyst pivot `enrich nvd(cve_id)` to resolve against the NVD registry for
    /// every CVE visible on the Cyberint surface.
    #[cfg(feature = "fixture-gen")]
    pub fn new_with_scenario(
        seed: u64,
        archetype: prism_dtu_common::Archetype,
        org_id: prism_dtu_common::OrgId,
        timeline: std::sync::Arc<prism_dtu_common::IncidentTimeline>,
        time_anchor: chrono::DateTime<chrono::Utc>,
        catalog: &prism_dtu_common::ScenarioEntityCatalog,
    ) -> anyhow::Result<Self> {
        use crate::generator::generate_with_scenario_iocs;
        use prism_dtu_common::GenOpts;

        // Generate fixture data with catalog IOC hashes stamped onto alert records (AC-002)
        // AND catalog CVEs on CVE-surface records (PC-8 / BC-2.06.020).
        // generate_with_scenario_iocs also stamps iocs[0].value with catalog_ioc_hashes[0]
        // onto CompromisedEndpoint alert-surface records, enabling the real-schema IOC filter
        // in routes/alerts.rs to apply StageMask projection (BC-2.06.019 PC-4).
        let opts = GenOpts {
            seed,
            time_anchor,
            ..GenOpts::default()
        };
        let fixture = generate_with_scenario_iocs(
            &org_id,
            archetype,
            &opts,
            &catalog.ioc_ips,
            &catalog.ioc_domains,
            &catalog.ioc_hashes,
            &catalog.device_cves,
        );

        // Load static fixtures (required for alert_fixture / alert_store initialization).
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let alerts: Vec<Alert> = prism_dtu_common::load_fixture_as(crate_dir, "alerts")?;
        let alerts_page2: Vec<Alert> =
            prism_dtu_common::load_fixture_as(crate_dir, "alerts-page2")?;
        let threats: Vec<serde_json::Value> =
            prism_dtu_common::load_fixture_as(crate_dir, "threats")?;

        let admin_token = uuid::Uuid::new_v4().to_string();
        let instance_org_id = OrgId::new();
        let mut state = CyberintState::with_org_id_and_admin_token(
            instance_org_id,
            alerts,
            alerts_page2,
            threats,
            admin_token.clone(),
        );
        state.generated_records = fixture.records;
        state.fixture_gen_seeded = true;

        // Attach the timeline so route handlers apply StageMask filtering.
        // ADR-036 v2.3 §2.3: Arc<IncidentTimeline> is read-only after construction.
        state.timeline = Some(Arc::clone(&timeline));

        Ok(Self {
            state: Arc::new(state),
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
            internal_shutdown_tx: None,
            org_id: instance_org_id,
        })
    }

    /// Return the base URL for this clone (e.g. `http://127.0.0.1:PORT`).
    ///
    /// Delegates to the trait's `base_url()` which checks `is_tls_active()`.
    pub fn base_url(&self) -> String {
        <Self as BehavioralClone>::base_url(self)
    }

    /// Return the authoritative `OrgId` for this clone instance (W3-FIX-SEC-001).
    ///
    /// Route handlers validate `X-Prism-Org-Id` against this value.
    /// Exposes the private `state.instance_org_id` to test helpers that need to
    /// construct matching org headers (e.g., `x_org_id_auth::start_clone_with_org`).
    pub fn instance_org_id(&self) -> OrgId {
        self.state.instance_org_id
    }

    fn build_router(&self) -> Router {
        // NOTE: POST /login route is intentionally ABSENT.
        // ADR-031 §D3-a rule 1: the real Cyberint API has no login step.
        // Auth is validated via `access_token` cookie on every data request.
        // AC-001 (S-DTU-CYBERINT-AUTH-FIDELITY-001): any request to POST /login returns 404.
        Router::new()
            // Alerts
            .route("/api/v1/alerts", get(get_alerts))
            .route("/api/v1/alerts", post(get_alerts))
            .route("/api/v1/alerts/:alert_id", get(get_alert_by_id))
            .route("/api/v1/alerts/:alert_id/status", patch(patch_alert_status))
            .route("/api/v1/alerts/:alert_id/close", post(post_close_alert))
            // Threat intel
            .route("/api/v1/threat-intel", get(get_threat_intel))
            // DTU internal
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .route("/dtu/health", get(get_health))
            .with_state(self.state.clone())
    }
}

#[async_trait]
impl BehavioralClone for CyberintClone {
    /// Start with an explicit bind address, optional graceful-shutdown receiver, and
    /// optional TLS configuration.
    async fn start_on(
        &mut self,
        bind: SocketAddr,
        shutdown: Option<broadcast::Receiver<()>>,
        #[cfg(feature = "tls")] tls: Option<Arc<axum_server::tls_rustls::RustlsConfig>>,
        #[cfg(not(feature = "tls"))] tls: Option<()>,
    ) -> anyhow::Result<SocketAddr> {
        let router = self.build_router();

        #[cfg(feature = "tls")]
        if let Some(rustls_cfg) = tls {
            let handle = axum_server::Handle::new();
            let handle_clone = handle.clone();
            let server_task = tokio::spawn(async move {
                // SAFETY: server task crash must surface immediately as a fatal error.
                #[allow(clippy::expect_used)]
                axum_server::bind_rustls(bind, (*rustls_cfg).clone())
                    .handle(handle_clone)
                    .serve(router.into_make_service())
                    .await
                    .expect("CyberintClone TLS server crashed");
            });
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("CyberintClone TLS server failed to start"))?;
            self.bound_addr = Some(addr);
            self.tls_active = true;
            self.server_handle = Some(server_task);
            // Retain handle so stop() can call graceful_shutdown() (MEDIUM-001 fix).
            self.tls_handle = Some(handle);
            return Ok(addr);
        }

        // Plain HTTP path.
        let _ = tls;
        let listener = tokio::net::TcpListener::bind(bind).await?;
        let addr = listener.local_addr()?;
        self.bound_addr = Some(addr);
        self.tls_active = false;

        if let Some(mut rx) = shutdown {
            // Harness-provided shutdown: keep the external broadcast receiver path unchanged.
            let handle = tokio::spawn(async move {
                let server = axum::serve(listener, router);
                let serve_future = server.with_graceful_shutdown(async move {
                    let _ = rx.recv().await;
                });
                // SAFETY: server task crash must surface immediately as a fatal error.
                #[allow(clippy::expect_used)]
                serve_future.await.expect("Cyberint DTU server error");
            });
            self.server_handle = Some(handle);
        } else {
            // Internal shutdown path (S-PERF-GATE-005): wire a broadcast channel so
            // stop() can signal the server task directly, completing in < 10ms for idle
            // clones instead of timing out on the 5s hard-abort fallback.
            let (handle, tx) = prism_dtu_common::server::spawn_with_internal_shutdown(
                listener,
                router,
                "Cyberint DTU server error",
            );
            self.server_handle = Some(handle);
            self.internal_shutdown_tx = Some(tx);
        }

        Ok(addr)
    }

    /// Stop the server: signal internal shutdown then await with a short fallback.
    ///
    /// # S-PERF-GATE-005 — graceful shutdown wiring
    ///
    /// All three paths (HTTP-internal, HTTP-harness, TLS) now resolve promptly:
    ///
    /// - **HTTP internal** (`shutdown=None` on start): fires `internal_shutdown_tx`,
    ///   the server task's `with_graceful_shutdown` future resolves, the select!
    ///   handle-done arm fires in < 10ms.  Fallback hard-abort at 250ms (safety only).
    /// - **HTTP harness** (`shutdown=Some(rx)` on start): the harness already sent
    ///   the broadcast before calling `stop()`; we wait up to 250ms before abort.
    /// - **TLS path**: fires `axum_server::Handle::graceful_shutdown(250ms)`, then
    ///   waits up to 250ms before abort.
    async fn stop(&mut self) -> anyhow::Result<()> {
        // TLS path: signal graceful shutdown via the retained axum_server::Handle.
        #[cfg(feature = "tls")]
        if let Some(h) = self.tls_handle.take() {
            h.graceful_shutdown(Some(std::time::Duration::from_millis(250)));
        }

        // All paths: fire internal sender (if present) and await with short fallback.
        if let Some(handle) = self.server_handle.take() {
            prism_dtu_common::server::graceful_stop(
                self.internal_shutdown_tx.take(),
                handle,
                std::time::Duration::from_millis(250),
            )
            .await;
        }

        self.tls_active = false;
        Ok(())
    }

    /// Reset all captured state to initial values.
    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    /// Reconfigure the stub at runtime (auth_mode, rate_limit_after, etc.).
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)
    }

    /// Return the `SocketAddr` the stub is bound to.
    fn bound_addr(&self) -> SocketAddr {
        // SAFETY: callers must call start() before bound_addr(); panic documents the programming error.
        #[allow(clippy::expect_used)]
        self.bound_addr
            .expect("CyberintClone::bound_addr() called before start()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }

    fn admin_token(&self) -> &str {
        &self.admin_token
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    // S-PERF-GATE-005 test names follow the factory naming convention for perf gate stories.
    #![allow(non_snake_case)]

    use prism_dtu_common::BehavioralClone;

    use super::CyberintClone;

    /// Red Gate test for S-PERF-GATE-005: stop() must complete in < 500ms for an idle
    /// clone started via the direct start() path (shutdown=None internal channel).
    ///
    /// BEFORE FIX: fails — stop() always hits the 5s select! timeout and hard-aborts,
    /// taking ~5.0s. The assertion fires because 5000ms >> 500ms.
    ///
    /// AFTER FIX: passes — stop() fires the internal shutdown sender, the server task
    /// completes its graceful-shutdown future immediately (no in-flight requests), the
    /// select! handle-done arm fires in < 10ms, total < 500ms.
    #[tokio::test]
    async fn test_PERF_GATE_005_stop_completes_promptly_for_idle_clone() {
        let mut clone = CyberintClone::new().expect("CyberintClone::new() must succeed");
        clone
            .start()
            .await
            .expect("CyberintClone::start() must succeed");

        let t = std::time::Instant::now();
        clone
            .stop()
            .await
            .expect("CyberintClone::stop() must succeed");
        let elapsed = t.elapsed();

        assert!(
            elapsed < std::time::Duration::from_millis(500),
            "S-PERF-GATE-005: idle clone.stop() must complete in <500ms \
             (graceful shutdown wired), not hit the ~5s hard-abort timeout; \
             elapsed {:?}",
            elapsed
        );
    }

    /// Red Gate variant for S-PERF-GATE-005: stop() after one completed request
    /// must also complete in < 500ms (no lingering connection delays teardown).
    ///
    /// BEFORE FIX: fails — stop() always hits the 5s select! timeout and hard-aborts
    /// regardless of whether a request was made (the root cause is the missing internal
    /// shutdown channel, not a lingering connection).
    ///
    /// AFTER FIX: passes — the connection is idle/closed at teardown; stop() fires
    /// the internal sender and the server drains in < 10ms.
    #[tokio::test]
    async fn test_PERF_GATE_005_stop_completes_promptly_after_one_request() {
        let mut clone = CyberintClone::new().expect("CyberintClone::new() must succeed");
        clone
            .start()
            .await
            .expect("CyberintClone::start() must succeed");

        // Make one completed request to the health endpoint (no auth required).
        // After this request completes the connection is idle — stop() must still
        // complete in < 500ms (the root cause is the missing internal shutdown channel,
        // not a lingering connection; both idle and post-request paths must be fast).
        let base_url = clone.base_url();
        // Use a 5s timeout so a stuck health request doesn't block the test indefinitely.
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("S-PERF-GATE-005: reqwest Client::builder must succeed");
        let _ = client
            .get(format!("{base_url}/dtu/health"))
            .send()
            .await
            .expect("S-PERF-GATE-005: health request must succeed before stop");

        let t = std::time::Instant::now();
        clone
            .stop()
            .await
            .expect("CyberintClone::stop() must succeed");
        let elapsed = t.elapsed();

        assert!(
            elapsed < std::time::Duration::from_millis(500),
            "S-PERF-GATE-005: stop() after one completed request must complete \
             in <500ms (no lingering connection delays teardown); elapsed {:?}",
            elapsed
        );
    }
}
