//! `ArmisClone` — `BehavioralClone` implementation for the Armis Centrix DTU.
//!
//! Lifecycle:
//! 1. `ArmisClone::new()` — allocates state; loads fixtures into registry.
//! 2. `start()` — binds an ephemeral TCP port, builds the axum router, spawns the server.
//! 3. `bound_addr()` / `base_url()` — exposes the server address to test clients.
//! 4. `reset()` — clears tag store + AQL log; fixtures remain loaded.
//! 5. `configure()` — applies JSON patch to runtime configuration (delegates to state).
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
    routing::{delete, get, post},
    Router,
};
use prism_core::OrgId;
use prism_dtu_common::{BehavioralClone, FailureLayer};
use tokio::{sync::broadcast, task::JoinHandle};

use crate::{
    routes::{
        alerts::get_alerts,
        devices::{get_device_activity, get_device_risk, get_or_post_devices, post_devices},
        dtu::{get_aql_log, get_health, post_configure, post_reset},
        search::get_search,
        tags::{delete_device_tag, post_device_tag},
    },
    state::ArmisState,
    types::{ActivityRecord, AlertRecord, DeviceRecord},
};

/// L2-fidelity behavioral clone of the Armis Centrix API.
pub struct ArmisClone {
    /// Shared mutable state — public to allow test inspection of `generated_records`
    /// (fixture-gen Red Gate tests) and `instance_org_id` (org-isolation tests).
    pub state: Arc<ArmisState>,
    bound_addr: Option<SocketAddr>,
    server_handle: Option<JoinHandle<()>>,
    /// True when the server is currently bound via TLS (axum_server::bind_rustls).
    tls_active: bool,
    /// `axum_server::Handle` retained for graceful shutdown of TLS servers (MEDIUM-001).
    #[cfg(feature = "tls")]
    tls_handle: Option<axum_server::Handle>,
    /// Admin shared-secret token for `POST /dtu/configure` (ADR-003 Amendment #5).
    admin_token: String,
}

impl ArmisClone {
    /// Create a new `ArmisClone`. Loads all fixtures from the crate root.
    ///
    /// Uses `prism_dtu_common::load_fixture_as` for deterministic fixture loading.
    pub fn new() -> anyhow::Result<Self> {
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let devices: Vec<DeviceRecord> = prism_dtu_common::load_fixture_as(crate_dir, "devices")?;
        let activity: Vec<ActivityRecord> =
            prism_dtu_common::load_fixture_as(crate_dir, "device-activity")?;
        let alerts: Vec<AlertRecord> = prism_dtu_common::load_fixture_as(crate_dir, "alerts")?;

        let admin_token = uuid::Uuid::new_v4().to_string();
        let state = Arc::new(ArmisState::with_admin_token(
            devices,
            activity,
            alerts,
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
        })
    }

    /// Create a new `ArmisClone` bound to a specific `instance_org_id`.
    ///
    /// Unlike `new()` (which uses `DTU_DEFAULT_INSTANCE_ORG_ID`), this constructor
    /// sets a real org identity so that the instance-identity guard (CR-012/SEC-P2-001)
    /// fires for requests with a mismatched or absent `X-Org-Id` header.
    ///
    /// Used by multi-tenant tests that need a real-org clone to verify the
    /// `instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID` guard path.
    ///
    /// (CR-012/SEC-P2-001; BC-3.5.002 precondition 3)
    pub fn new_with_org(instance_org_id: OrgId) -> anyhow::Result<Self> {
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let devices: Vec<DeviceRecord> = prism_dtu_common::load_fixture_as(crate_dir, "devices")?;
        let activity: Vec<ActivityRecord> =
            prism_dtu_common::load_fixture_as(crate_dir, "device-activity")?;
        let alerts: Vec<AlertRecord> = prism_dtu_common::load_fixture_as(crate_dir, "alerts")?;

        let admin_token = uuid::Uuid::new_v4().to_string();
        let state = Arc::new(ArmisState::with_admin_token_and_org(
            devices,
            activity,
            alerts,
            admin_token.clone(),
            instance_org_id,
        ));
        Ok(Self {
            state,
            bound_addr: None,
            server_handle: None,
            tls_active: false,
            #[cfg(feature = "tls")]
            tls_handle: None,
            admin_token,
        })
    }

    // -----------------------------------------------------------------------
    // Story A: new_with_seed constructor stub (BC-2.06.018 / ADR-036 §2.3)
    // -----------------------------------------------------------------------

    /// Construct an `ArmisClone` with deterministic fixture data generated at
    /// construction time from `(seed, archetype, org_id)`.
    ///
    /// Gated `#[cfg(feature = "fixture-gen")]` — not compiled for production binaries.
    ///
    /// Internally derives `org_slug` via `prism_dtu_common::org_slug_from_org_id(&org_id)`
    /// (ADR-036 §2.2 / ADR-036 v2.2). The `org_slug` is NO LONGER a constructor argument.
    ///
    /// Calls `generate(org_id, &org_slug, archetype, &GenOpts { seed, ..GenOpts::default() })`
    /// under `fixture-gen`, stores the resulting records in `generated_records` in state.
    ///
    /// This constructor is **fallible** — mirrors `ArmisClone::new() -> anyhow::Result<Self>`.
    /// `build_clone_pairs` propagates the error via `?`.
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

        // Derive org_slug internally from org_id bytes (ADR-036 §2.2).
        let org_slug = prism_dtu_common::org_slug_from_org_id(&org_id);

        let opts = GenOpts {
            seed,
            time_anchor,
            ..GenOpts::default()
        };
        let fixture = generate(org_id, &org_slug, archetype, &opts);

        // Load static fixtures (required by ArmisState; still used for activity and alerts).
        // We also need them to populate device_registry/devices_ordered for backward compat.
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let devices: Vec<crate::types::DeviceRecord> =
            prism_dtu_common::load_fixture_as(crate_dir, "devices")?;
        let activity: Vec<crate::types::ActivityRecord> =
            prism_dtu_common::load_fixture_as(crate_dir, "device-activity")?;
        let alerts: Vec<crate::types::AlertRecord> =
            prism_dtu_common::load_fixture_as(crate_dir, "alerts")?;

        let admin_token = uuid::Uuid::new_v4().to_string();
        let mut state =
            ArmisState::with_admin_token(devices, activity, alerts, admin_token.clone());
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
        })
    }

    // -----------------------------------------------------------------------
    // Story B: new_with_scenario constructor (BC-2.06.019 / ADR-036 v2.3 §2.4)
    // -----------------------------------------------------------------------

    /// Construct an `ArmisClone` with the scenario timeline layer.
    ///
    /// Gated `#[cfg(feature = "fixture-gen")]` because the signature references
    /// `Archetype`, `OrgId`, and `IncidentTimeline` which are only available
    /// under that feature (pre-existing gate omission fixed in B-P5-OBS-1).
    ///
    /// 5-arg form per ADR-036 v2.3 §2.4. Internally calls
    /// `new_with_seed_anchored(seed, archetype, org_id, time_anchor)` (NOT the 3-arg
    /// `new_with_seed` which anchors at demo_time_anchor() = 2026-01-01, producing
    /// stale timestamps for a June 2026 demo).
    ///
    /// Sets `state.timeline = Some(Arc::clone(&timeline))` so route handlers can
    /// compute the current stage index and apply StageMask filtering.
    #[cfg(feature = "fixture-gen")]
    pub fn new_with_scenario(
        seed: u64,
        archetype: prism_dtu_common::Archetype,
        org_id: prism_dtu_common::OrgId,
        timeline: std::sync::Arc<prism_dtu_common::IncidentTimeline>,
        time_anchor: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<Self> {
        // Call new_with_seed_anchored (NOT the 3-arg new_with_seed) to use the
        // caller-supplied time_anchor for era-coherent generated timestamps.
        // ADR-036 v2.3 §2.3 mandates this; the 3-arg path is FORBIDDEN here.
        let mut clone = Self::new_with_seed_anchored(seed, archetype, org_id, time_anchor)?;
        // Attach the timeline BEFORE any other reference can be taken.
        //
        // Structural threading (B-P5-OBS-1): use Arc::try_unwrap to reclaim the state
        // struct (refcount=1 immediately post-construction), set timeline, then re-wrap.
        // This is safe because new_with_seed_anchored just returned the only Arc clone;
        // no other thread can hold a reference at this point.
        //
        // Prefer try_unwrap over get_mut to avoid silent-drop risk: if a future refactor
        // creates a second Arc clone before this point, try_unwrap returns Err with the
        // original Arc, which we catch with expect() so the bug is loud.
        //
        // ADR-036 v2.3 §2.3: Arc<IncidentTimeline> is read-only after construction.
        let mut state = Arc::try_unwrap(clone.state).unwrap_or_else(|_| {
            panic!(
                "ArmisClone::new_with_scenario: Arc refcount must be 1 immediately after \
                 new_with_seed_anchored; a second Arc clone would indicate a refactor \
                 invariant violation (B-P5-OBS-1)"
            )
        });
        state.timeline = Some(Arc::clone(&timeline));
        clone.state = Arc::new(state);
        Ok(clone)
    }

    /// Return the base URL for the bound server (e.g. `"http://127.0.0.1:12345"`).
    ///
    /// Delegates to the trait's `base_url()` which checks `is_tls_active()`.
    ///
    /// Panics if `start()` has not been called.
    pub fn base_url(&self) -> String {
        <Self as BehavioralClone>::base_url(self)
    }

    /// Return the authoritative `OrgId` for this clone instance (W3-FIX-SEC-001).
    ///
    /// Route handlers validate `X-Org-Id` against this value.
    /// Exposes the private `state.instance_org_id` to test helpers that need to
    /// construct matching org headers (e.g., `x_org_id_auth::start_clone_with_org`).
    pub fn instance_org_id(&self) -> OrgId {
        self.state.instance_org_id
    }

    fn build_router(&self) -> Router {
        let failure_layer = FailureLayer::shared(Arc::clone(&self.state.failure_mode));

        // Vendor API routes — wrapped with FailureLayerShared so failure injection
        // applies only to the real API surface. DTU-internal routes MUST remain
        // reachable even when a failure mode is active (configure/reset must always work).
        let vendor_router = Router::new()
            // Primary AQL search endpoint (ADR-031 §D8-a — Gap-AR-001 closed by S-DEMO-ARMIS-AQL-001).
            // Real Armis Centrix API path: GET /api/v1/search?aql=<query>
            .route("/api/v1/search", get(get_search))
            // Direct endpoints retained for backward compatibility (AC-006).
            .route("/api/v1/devices", get(get_or_post_devices))
            .route("/api/v1/devices", post(post_devices))
            .route(
                "/api/v1/devices/:device_id/activity",
                get(get_device_activity),
            )
            .route("/api/v1/devices/:device_id/risk", get(get_device_risk))
            .route("/api/v1/alerts", get(get_alerts))
            .route("/api/v1/devices/:device_id/tags/", post(post_device_tag))
            .route(
                "/api/v1/devices/:device_id/tags/:tag_key",
                delete(delete_device_tag),
            )
            .layer(failure_layer);

        // DTU-internal routes — NOT wrapped by FailureLayer; always reachable.
        Router::new()
            .merge(vendor_router)
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .route("/dtu/health", get(get_health))
            // Armis-specific introspection: AQL capture log
            .route("/dtu/aql-log", get(get_aql_log))
            .with_state(self.state.clone())
    }
}

#[async_trait]
impl BehavioralClone for ArmisClone {
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
                let result = axum_server::bind_rustls(bind, (*rustls_cfg).clone())
                    .handle(handle_clone)
                    .serve(router.into_make_service())
                    .await;
                // SAFETY: server crash inside the task should propagate as a fatal error; surfacing it immediately is correct.
                #[allow(clippy::expect_used)]
                result.expect("ArmisClone TLS server crashed");
            });
            let addr = handle
                .listening()
                .await
                .ok_or_else(|| anyhow::anyhow!("ArmisClone TLS server failed to start"))?;
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

        let handle = tokio::spawn(async move {
            let server = axum::serve(listener, router);
            if let Some(mut rx) = shutdown {
                let result = server
                    .with_graceful_shutdown(async move {
                        let _ = rx.recv().await;
                    })
                    .await;
                // SAFETY: server task panic is fatal; surfacing it immediately is correct.
                #[allow(clippy::expect_used)]
                result.expect("Armis DTU server error");
            } else {
                let result = server.await;
                // SAFETY: same as above — server task panic must surface immediately.
                #[allow(clippy::expect_used)]
                result.expect("Armis DTU server error");
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

    /// Reset all captured state to initial values.
    ///
    /// Per ADR-002 §4: delegates to `self.state.reset()` with no additional logic.
    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }

    /// Reconfigure the stub at runtime.
    ///
    /// Per ADR-002 §4: delegates to `self.state.apply_config(&config)` — no inline
    /// JSON field inspection in `clone.rs`.
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)
    }

    /// Return the `SocketAddr` the stub is bound to.
    fn bound_addr(&self) -> SocketAddr {
        // SAFETY: callers are required to call start() before bound_addr(); the expect message documents the contract.
        #[allow(clippy::expect_used)]
        self.bound_addr
            .expect("ArmisClone::bound_addr() called before start()")
    }

    fn is_tls_active(&self) -> bool {
        self.tls_active
    }

    fn admin_token(&self) -> &str {
        &self.admin_token
    }
}
