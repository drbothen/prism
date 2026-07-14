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
//!
//! # Per-Instance Request Counter (AC-006 / INV-ISOLATION-001)
//!
//! `ArmisState::request_counter` is an `AtomicU64` incremented by a tower
//! middleware layer injected at the outermost router position in `build_router()`.
//! The counter fires for EVERY HTTP request the clone receives — vendor API routes
//! AND DTU-internal routes alike — before FailureLayer or any route handler runs.
//!
//! Test code reads the counter via `clone.request_count()` (which delegates to
//! `Arc::clone(&clone.state).received_request_count()`) OR via `GET /dtu/request-count`
//! which returns the count as a plain decimal integer body.
//!
//! Both access paths are load-bearing for AC-006's isolation assertion:
//!
//! ```text
//! // After dispatching N requests to clone A only:
//! assert_eq!(clone_a.request_count(), N);   // server-side counter on A
//! assert_eq!(clone_b.request_count(), 0);   // zero requests reached B's router
//! ```
//!
//! See `ArmisState::received_request_count()` and `GET /dtu/request-count` route in
//! `routes/dtu.rs` for the accessible interfaces.

use std::{
    net::SocketAddr,
    sync::{atomic::Ordering, Arc},
};

use async_trait::async_trait;
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
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
        dtu::{get_aql_log, get_health, get_request_count, post_configure, post_reset},
        search::get_search,
        tags::{delete_device_tag, post_device_tag},
    },
    state::ArmisState,
    types::{ActivityRecord, AlertRecord, DeviceRecord},
};

/// Tower middleware that increments `ArmisState::request_counter` on every incoming request.
///
/// Injected at the outermost router position in `ArmisClone::build_router()` so it fires
/// before FailureLayer and all route handlers. This gives a LOAD-BEARING server-side count
/// of every HTTP request received by THIS clone instance's router.
///
/// Used by AC-006 / INV-ISOLATION-001 multi-tenant isolation tests:
/// `ArmisClone::request_count()` reads the same counter from outside the router.
///
/// (BC-2.06.017 AC-006 / INV-ISOLATION-001)
async fn count_request_middleware(
    state: axum::extract::State<Arc<ArmisState>>,
    request: Request,
    next: Next,
) -> Response {
    state.request_counter.fetch_add(1, Ordering::SeqCst);
    next.run(request).await
}

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
    /// Internal broadcast sender for graceful HTTP shutdown (S-PERF-GATE-005).
    ///
    /// Set by `start_on` when `shutdown=None` (the direct `start()` path). Fired by
    /// `stop()` so the server task drains immediately instead of timing out on the
    /// 5s select! fallback. `None` when the harness-provided or TLS path is used.
    internal_shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
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
            internal_shutdown_tx: None,
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
            internal_shutdown_tx: None,
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
            internal_shutdown_tx: None,
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
    /// 6-arg form per ADR-036 v2.3 §2.4 + F-PIVOT003-R2-002 (catalog wiring).
    /// Internally calls `generate_with_scenario_cves` so that CompromisedEndpoint
    /// device records carry `device_cves_first = catalog.device_cves[0]` — the
    /// anchor that makes the NVD pivot `enrich nvd(device_cves_first)` resolve
    /// at stage ≥ 4 (AC-008 / BC-2.06.019).
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
        catalog: &prism_dtu_common::ScenarioEntityCatalog,
    ) -> anyhow::Result<Self> {
        use crate::generator::generate_with_scenario_cves;
        use prism_dtu_common::GenOpts;

        // F-PIVOT003-R11C-002: generate ONCE with CVEs stamped in a single pass —
        // mirroring the CrowdStrike sibling pattern (see CrowdstrikeClone::new_with_scenario).
        // The previous approach called new_with_seed_anchored (generate #1) then
        // generate_with_scenario_cves (generate #2), discarding the first set of records.
        // Building state directly avoids the redundant generate() call.
        let org_slug = prism_dtu_common::org_slug_from_org_id(&org_id);
        let opts = GenOpts {
            seed,
            time_anchor,
            ..GenOpts::default()
        };
        let fixture_with_cves = generate_with_scenario_cves(
            org_id.clone(),
            &org_slug,
            archetype,
            &opts,
            &catalog.device_cves,
        );

        // Load static fixtures (required by ArmisState; still used for activity and alerts).
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
        state.generated_records = fixture_with_cves.records;
        // Mark as seeded so route handlers use the generated path (even for DormantTenant
        // which produces 0 records). F-P6-HIGH-001 / ADR-036 v2.2.
        state.fixture_gen_seeded = true;
        // Attach the timeline so route handlers can apply StageMask filtering.
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
        })
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

    /// Return the number of HTTP requests received by this clone's router since
    /// construction (or last `reset()`).
    ///
    /// This is the server-side per-instance request counter for AC-006 / INV-ISOLATION-001.
    /// The counter is incremented by `count_request_middleware` on EVERY request that reaches
    /// this instance's axum router — before FailureLayer and all route handlers run.
    ///
    /// # Multi-tenant isolation usage
    ///
    /// ```text
    /// // Dispatch 5 requests to clone_a (acme instance), 0 to clone_b (contoso instance).
    /// assert_eq!(clone_a.request_count(), 5);  // 5 requests received by A's router
    /// assert_eq!(clone_b.request_count(), 0);  // zero cross-tenant leakage
    /// ```
    ///
    /// Alternatively, read via HTTP: `GET http://{clone_addr}/dtu/request-count` returns
    /// the count as a plain decimal integer body (useful when the clone object is not
    /// directly accessible from the test scope).
    ///
    /// (BC-2.06.017 AC-006 / INV-ISOLATION-001)
    pub fn request_count(&self) -> u64 {
        self.state.received_request_count()
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
        // The per-instance request counter middleware is applied at the OUTERMOST layer
        // so it fires for EVERY request — vendor API + internal routes alike — before
        // FailureLayer and before any route handler (AC-006 / INV-ISOLATION-001).
        Router::new()
            .merge(vendor_router)
            .route("/dtu/configure", post(post_configure))
            .route("/dtu/reset", post(post_reset))
            .route("/dtu/health", get(get_health))
            // Armis-specific introspection: AQL capture log
            .route("/dtu/aql-log", get(get_aql_log))
            // Per-instance request counter endpoint (AC-006 / INV-ISOLATION-001).
            // Returns the count as a plain decimal integer body.
            // Both this endpoint AND ArmisClone::request_count() read the same AtomicU64.
            .route("/dtu/request-count", get(get_request_count))
            .with_state(self.state.clone())
            // Counting middleware: outermost layer — fires before all other middleware.
            // Increments ArmisState::request_counter on every HTTP request received.
            // Load-bearing for AC-006 / INV-ISOLATION-001 multi-tenant isolation proof.
            .layer(axum::middleware::from_fn_with_state(
                self.state.clone(),
                count_request_middleware,
            ))
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

        if let Some(mut rx) = shutdown {
            // Harness-provided shutdown: keep the external broadcast receiver path unchanged.
            let handle = tokio::spawn(async move {
                let server = axum::serve(listener, router);
                let serve_future = server.with_graceful_shutdown(async move {
                    let _ = rx.recv().await;
                });
                // SAFETY: server task panic is fatal; surfacing it immediately is correct.
                #[allow(clippy::expect_used)]
                serve_future.await.expect("Armis DTU server error");
            });
            self.server_handle = Some(handle);
        } else {
            // Internal shutdown path (S-PERF-GATE-005): wire a broadcast channel so
            // stop() can signal the server task directly, completing in < 10ms for idle
            // clones instead of timing out on the 5s hard-abort fallback.
            let (handle, tx) = prism_dtu_common::server::spawn_with_internal_shutdown(
                listener,
                router,
                "Armis DTU server error",
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
