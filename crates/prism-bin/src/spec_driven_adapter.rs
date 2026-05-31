// SPDX-License-Identifier: Apache-2.0
//! SpecDrivenSensorAdapter — bridges PipelineExecutor to AdapterRegistry (S-DEMO-001).
//!
//! Closes GAP-002-A: wires sensor TOML specs loaded at boot into the `AdapterRegistry`
//! so that `fan_out()` in prism-query can resolve `(org_id, sensor_id) → Arc<dyn SensorAdapter>`
//! and return live Arrow data from the DTU clones.
//!
//! # Architecture Compliance
//!
//! - MUST live in `prism-bin` (NOT `prism-sensors`) per ADR-023 §D3 Forbidden Dependencies.
//!   `prism-sensors` MUST NOT import `prism-spec-engine`; only `prism-bin` imports both.
//! - `BearerStaticAuthProvider` lives here (NOT in `prism-spec-engine`) because it bridges
//!   `SensorAuth` (prism-sensors) ↔ `AuthProvider` (prism-spec-engine). Only prism-bin imports both.
//! - `StaticCookieAuthProvider` lives in `prism-spec-engine/src/auth_provider.rs`.
//!
//! # Auth strategies (OQ-1 Resolution)
//!
//! `AdapterAuthStrategy` is held at construction time:
//! - `Plugin(Arc<dyn AuthProvider>)` — CrowdStrike: held PluginAuthProvider, ignores SensorAuth arg.
//! - `BearerStatic` — Armis/Claroty: token extracted from SensorAuth arg at fetch() call time.
//! - `StaticCookie(Arc<dyn AuthProvider>)` — Cyberint: held StaticCookieAuthProvider (NO HTTP calls
//!   at acquire_token; reads api_key from credential store). Renamed from CookieLogin (v1.1/v1.2)
//!   per ADR-031 §D3 / DEMO-001 v1.3. NOT `CookieLogin` — that required a login step.
//!
//! # reqwest::Client timeout
//!
//! `SpecDrivenSensorAdapter::new` constructs the HTTP client with `.timeout(Duration::from_secs(30))`
//! per CLAUDE.md conventions (TD-S-PLUGIN-PREREQ-B-005 closure requirement).
//!
//! BCs: BC-2.01.013, BC-2.11.005, BC-2.06.014, BC-2.22.001
//! Story: S-DEMO-001 v1.3

use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::{OrgId, SensorId};
use prism_sensors::{
    SensorAdapter,
    adapter::{QueryParams, SensorError, SensorSpec},
    auth::SensorAuth,
};
use prism_spec_engine::{
    AuthProvider, AuthToken, ResolvedSensorSpec,
    error::SpecEngineError,
    pipeline::{FetchContext, PipelineExecutor},
    spec_parser::SensorSpec as SpecEngineSensorSpec,
};

// ---------------------------------------------------------------------------
// AdapterAuthStrategy — auth path selected at construction time (OQ-1)
// ---------------------------------------------------------------------------

/// Auth strategy held by a `SpecDrivenSensorAdapter` instance.
///
/// Selected once at boot step 9A construction time based on the sensor spec's `auth_type`:
/// - `Plugin` — CrowdStrike: held `PluginAuthProvider` from step 7.5b; ignores `SensorAuth` arg.
/// - `BearerStatic` — Armis/Claroty: token extracted from `SensorAuth::BearerStatic` arg per-fetch.
/// - `StaticCookie` — Cyberint: held `StaticCookieAuthProvider` (no HTTP calls at acquire_token).
///   NOTE: v1.1/v1.2 called this `CookieLogin`. Renamed `StaticCookie` per ADR-031 §D3 (S-DEMO-001 v1.3).
///   The old `CookieLogin` variant made HTTP calls to `POST /login` — WRONG under ADR-031 DTU=true-DTU.
///
/// BC-2.01.013 postcondition 4; OQ-1 Resolution (S-DEMO-001 §OQ-1).
#[derive(Clone)]
pub enum AdapterAuthStrategy {
    /// CrowdStrike: WASM plugin auth via held `Arc<dyn AuthProvider>` (`PluginAuthProvider`).
    /// The `SensorAuth` argument to `fetch()` is ignored for plugin-authed sensors (ADR-028 §D10).
    Plugin(Arc<dyn AuthProvider>),
    /// Armis / Claroty: `bearer_static` auth type. Token is extracted from the `SensorAuth`
    /// argument at each `fetch()` call (per-fetch `BearerStaticAuthProvider` construction).
    /// No held auth state — the token lives in the credential store and arrives at fetch time.
    BearerStatic,
    /// Cyberint: `cookie_roundtrip` auth type. Held `StaticCookieAuthProvider` reads the API
    /// key from the credential store at `acquire_token()` time with NO HTTP call (ADR-031 §D1-b).
    /// The token is injected by `PipelineExecutor::build_request` as `Cookie: access_token={token}`
    /// (NOT `cyberint_session` — that was the pre-ADR-031 DTU model; permanently superseded).
    ///
    /// ADR-031 §D3-b; S-DEMO-001 v1.3.
    StaticCookie(Arc<dyn AuthProvider>),
}

// ---------------------------------------------------------------------------
// BearerStaticAuthProvider — thin AuthProvider wrapper for bearer_static sensors
// ---------------------------------------------------------------------------

/// Thin `AuthProvider` wrapper for `bearer_static` sensors (Armis, Claroty).
///
/// Lives in `prism-bin` (NOT `prism-spec-engine`) because it bridges `SensorAuth` from
/// `prism-sensors` ↔ `AuthProvider` from `prism-spec-engine`. Only `prism-bin` imports both
/// crates (ADR-023 §Permitted Patterns).
///
/// Constructed **per-fetch** from the `SensorAuth::BearerStatic { token }` argument at
/// `SpecDrivenSensorAdapter::fetch()` call time (OQ-1 Resolution — Option A per-fetch).
///
/// The token is not held at construction time of `SpecDrivenSensorAdapter` — it arrives
/// at query time via the `SensorAuth` argument (ADR-022 §C wiring; AD-017 credential safety).
///
/// BC-2.01.013 postcondition 4; ADR-023 §Permitted Patterns; OQ-1.
pub struct BearerStaticAuthProvider {
    /// Bearer token string.
    ///
    /// AD-017: this field holds the bearer token for the duration of a single fetch() call.
    /// The token is NOT stored at SpecDrivenSensorAdapter construction time.
    token: String,
}

impl BearerStaticAuthProvider {
    /// Construct a `BearerStaticAuthProvider` from a bearer token string.
    ///
    /// Called per-fetch from `SpecDrivenSensorAdapter::fetch()` when `auth_strategy` is
    /// `AdapterAuthStrategy::BearerStatic`. Token comes from the `SensorAuth` argument.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl AuthProvider for BearerStaticAuthProvider {
    /// Return the bearer token as an `AuthToken`.
    ///
    /// Zero I/O. No branching. Returns the held token wrapped in `AuthToken::new`.
    /// `PipelineExecutor::build_request` injects it as `Authorization: Bearer {token}`
    /// for `BearerStatic` sensors.
    ///
    /// WIRING-EXEMPT: this is a pure delegation to `AuthToken::new(token)` — no logic,
    /// no I/O, no branching. The box-pin wrapper is required for dyn-compatibility per
    /// `AuthProvider` trait contract (BC-2.16.002 object-safety).
    fn acquire_token<'a>(
        &'a self,
        _spec: &'a SpecEngineSensorSpec,
        _client_id: &'a prism_core::OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
        let token = self.token.clone();
        Box::pin(async move { Ok(AuthToken::new(token)) })
    }
}

// ---------------------------------------------------------------------------
// SpecDrivenSensorAdapter
// ---------------------------------------------------------------------------

/// Spec-driven `SensorAdapter` that delegates to `PipelineExecutor::execute()`.
///
/// Registered in `AdapterRegistry` at boot step 9A for every `(OrgId, SensorId)` pair
/// present in the resolved spec catalog. Closes GAP-002-A — enables `fan_out()` to
/// return live Arrow RecordBatches from the DTU clones instead of `AdapterNotFound`.
///
/// ## Auth dispatch (BC-2.01.013 postcondition 4)
///
/// - `Plugin` strategy: passes the held `Arc<PluginAuthProvider>`; ignores `SensorAuth` arg
///   (ADR-028 §D10 — plugin-authed sensors provide their own auth at the plugin level).
/// - `BearerStatic` strategy: constructs `BearerStaticAuthProvider` per-call from the
///   `SensorAuth` argument's token field (OQ-1 Resolution — per-fetch construction).
/// - `StaticCookie` strategy: uses the held `StaticCookieAuthProvider`; ignores `SensorAuth` arg.
///   `acquire_token()` reads the API key from the credential store with NO HTTP call (ADR-031 §D1-b).
///
/// ## reqwest::Client
///
/// Constructed via `reqwest::Client::builder().timeout(Duration::from_secs(30)).build()`
/// per CLAUDE.md conventions (TD-S-PLUGIN-PREREQ-B-005).
///
/// ## OCSF normalization
///
/// The `PipelineExecutor` does NOT return Arrow `RecordBatch` — it returns `PipelineResult`
/// (raw JSON records). `SpecDrivenSensorAdapter::fetch()` maps those records through
/// `ColumnMapper` + `OcsfNormalizer` to produce `Vec<RecordBatch>` (BC-2.11.005 postcondition).
///
/// BCs: BC-2.01.013, BC-2.06.014, BC-2.11.005; Story: S-DEMO-001 v1.3.
// S-DEMO-001 stub: fields are read in fetch() and sensor_type() — both implemented in this story.
// `dead_code` warning is suppressed because fetch() body is todo!() during the Red Gate phase.
#[allow(dead_code)]
pub struct SpecDrivenSensorAdapter {
    /// The resolved sensor spec (with per-org overlay applied).
    ///
    /// Contains the TYPE spec merged with per-org scalar overrides (base_url, timeout).
    /// Immutable after construction — shared via Arc for O(1) dispatch (INV-FANOUT-002).
    sensor_spec: Arc<ResolvedSensorSpec>,

    /// Auth strategy for this adapter (held at construction time, not per-fetch).
    ///
    /// OQ-1 Resolution: BearerStatic extracts the token from the SensorAuth arg at fetch time;
    /// Plugin and StaticCookie hold their providers here.
    auth_strategy: AdapterAuthStrategy,

    /// HTTP client with 30-second timeout (TD-S-PLUGIN-PREREQ-B-005).
    ///
    /// Constructed via `reqwest::Client::builder().timeout(Duration::from_secs(30)).build()`
    /// in `SpecDrivenSensorAdapter::new()`. MUST NOT be a global singleton (ADR-022 §C).
    http_client: reqwest::Client,
}

impl SpecDrivenSensorAdapter {
    /// Construct a `SpecDrivenSensorAdapter`.
    ///
    /// # Parameters
    ///
    /// - `sensor_spec` — The resolved sensor spec (TYPE + per-org overlay). Shared via Arc.
    /// - `auth_strategy` — Auth strategy selected based on `sensor_spec.spec.auth_type`.
    /// - `http_client` — `reqwest::Client` with 30s timeout (TD-S-PLUGIN-PREREQ-B-005).
    ///   MUST be constructed with `.timeout(Duration::from_secs(30))`.
    ///
    /// Production callers pass a client constructed by `build_http_client_with_timeout()`.
    /// Tests inject a client directed at a mock HTTP server.
    ///
    /// BC-2.01.013; ADR-022 §C wiring; OQ-1 Resolution.
    pub fn new(
        sensor_spec: Arc<ResolvedSensorSpec>,
        auth_strategy: AdapterAuthStrategy,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            sensor_spec,
            auth_strategy,
            http_client,
        }
    }
}

#[async_trait]
impl SensorAdapter for SpecDrivenSensorAdapter {
    /// Returns the sensor ID for this adapter.
    ///
    /// Used by `AdapterRegistry::register()` to key the adapter.
    ///
    /// GREEN-BY-DESIGN: zero branching, no I/O, no non-trivial helpers, 1 line.
    /// Criteria: (1) no `if`/`match`/`?`/`unwrap`, (2) no I/O, (3) only type constructor,
    /// (4) 1 line body. BC-5.38.002 — this test passes trivially and that is correct-by-construction.
    fn sensor_type(&self) -> SensorId {
        SensorId::from(self.sensor_spec.spec.sensor_id.as_str())
    }

    /// Returns a human-readable sensor name for tracing spans and error messages.
    ///
    /// Returns `"spec_driven"` as a static discriminator for this adapter type.
    /// Per-sensor names are available via `sensor_type()` (returns the SensorId).
    ///
    /// GREEN-BY-DESIGN: zero branching, no I/O, no non-trivial helpers, 1 line.
    fn sensor_name(&self) -> &'static str {
        "spec_driven"
    }

    /// Fetch sensor data by delegating to `PipelineExecutor::execute()`.
    ///
    /// Dispatches authentication by `auth_strategy`:
    /// - `Plugin`: passes the held `Arc<PluginAuthProvider>`; ignores `auth` arg (ADR-028 §D10).
    /// - `BearerStatic`: extracts bearer token from `auth: &dyn SensorAuth`, constructs
    ///   `BearerStaticAuthProvider` per-call, passes to `PipelineExecutor::execute()`.
    /// - `StaticCookie`: uses the held `StaticCookieAuthProvider` (NO HTTP calls at
    ///   acquire_token; `build_request` injects `Cookie: access_token={token}` per ADR-031 §D3-b).
    ///
    /// Maps `PipelineResult` (raw JSON) → `Vec<RecordBatch>` via OCSF normalization (BC-2.11.005).
    ///
    /// On double-401: propagates `SpecEngineError::AuthRefreshFailed` → `SensorError::Internal`
    /// (BC-2.01.013 error case; AC-012).
    ///
    /// # BC-5.38.001 Red Gate
    ///
    /// This function is NON-TRIVIAL (auth dispatch, async I/O, OCSF normalization, error mapping).
    /// Body is `todo!()` — implementer must write the real auth dispatch and pipeline delegation.
    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        todo!(
            "S-DEMO-001: implement SpecDrivenSensorAdapter::fetch — dispatch by auth_strategy, \
             call PipelineExecutor::execute with appropriate AuthProvider, map PipelineResult \
             to Vec<RecordBatch> via OCSF normalization (BC-2.01.013 postcondition 4)"
        )
    }
}

// ---------------------------------------------------------------------------
// build_http_client_with_timeout — production reqwest::Client factory
// ---------------------------------------------------------------------------

/// Construct a `reqwest::Client` with a 30-second timeout.
///
/// MUST be used by `step9a_populate_adapter_registry` when constructing `SpecDrivenSensorAdapter`
/// instances. Using `reqwest::Client::new()` without a timeout is a P2 finding per
/// CLAUDE.md conventions (TD-S-PLUGIN-PREREQ-B-005).
///
/// Returns `Err(String)` if the client builder fails (should not happen in production;
/// failure mode is malformed TLS configuration).
pub fn build_http_client_with_timeout() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("failed to build reqwest::Client with 30s timeout: {e}"))
}

// ---------------------------------------------------------------------------
// step9a_populate_adapter_registry — boot step 9A
// ---------------------------------------------------------------------------

/// Boot step 9A: iterate `resolved_spec_map` and register one `SpecDrivenSensorAdapter`
/// per `(OrgId, SensorId)` pair.
///
/// Positioned between step 7.5b and step 9 in the ADR-022 §B sequencing table.
/// Emits `event_type = "boot.step9a.adapter_registry_populated"` with `sensor_count`
/// and `org_count` fields per BC-2.16.002 catalog row (SAP-1 obligation).
///
/// # Auth strategy selection
///
/// For each `(org_slug, sensor_id, resolved_spec)` triple:
/// - `CustomViaPlugin`: looks up `plugin_auth_providers.get(&sensor_id)` → `AdapterAuthStrategy::Plugin`.
///   If not found (auth_plugin declared but provider not constructed at step 7.5b), logs E-SPEC-012
///   and skips the sensor.
/// - `BearerStatic`: → `AdapterAuthStrategy::BearerStatic` (no held provider).
/// - `CookieRoundtrip`: constructs `StaticCookieAuthProvider::new(sensor_id)` →
///   `AdapterAuthStrategy::StaticCookie(Arc::new(...))`.
/// - Other `auth_type` values: logs E-SPEC-012 (auth type mismatch for S-DEMO-001 scope) and skips.
///
/// # OrgSlug → OrgId translation
///
/// `resolved_spec_map` is keyed by `(OrgSlug, SensorId)`. `AdapterRegistry` is keyed by
/// `(OrgId, SensorId)`. This function calls `org_registry.id_for_slug(slug)` to translate.
/// If a slug has no matching OrgId (should not happen after step 3 cross-validation), the
/// sensor is skipped with a warning log.
///
/// # AC-006: empty spec_catalog
///
/// If `resolved_spec_map` is empty, the function returns `Ok(0)` — no error, no adapters.
/// Boot continues to step 9 (MCP server start).
///
/// # Errors
///
/// Returns `Err(BootError::InternalError)` if the HTTP client cannot be constructed
/// (reqwest builder failure — should never occur in production).
///
/// BC-2.22.001 postcondition; BC-2.01.013; BC-2.06.014; AC-004; AC-006.
///
/// # BC-5.38.001 Red Gate
///
/// This function is NON-TRIVIAL (nested iteration, OrgSlug→OrgId translation, auth strategy
/// dispatch, registry.register() calls, tracing event emission).
/// Body is `todo!()` — implementer must write the iteration and registration loop.
pub async fn step9a_populate_adapter_registry(
    _resolved_spec_map: &std::collections::HashMap<
        prism_spec_engine::ResolvedSpecKey,
        ResolvedSensorSpec,
    >,
    _org_registry: &prism_core::OrgRegistry,
    _plugin_auth_providers: &std::collections::HashMap<
        String,
        Arc<prism_spec_engine::PluginAuthProvider>,
    >,
    _adapter_registry: &mut prism_sensors::AdapterRegistry,
) -> Result<usize, crate::boot::BootError> {
    todo!(
        "S-DEMO-001: implement step9a_populate_adapter_registry — iterate resolved_spec_map, \
         translate OrgSlug→OrgId via org_registry.id_for_slug(), select auth strategy by \
         spec.auth_type, construct SpecDrivenSensorAdapter per (OrgId, SensorId), call \
         adapter_registry.register(), emit boot.step9a.adapter_registry_populated event \
         with sensor_count + org_count fields per BC-2.16.002 catalog row (SAP-1)"
    )
}

// ---------------------------------------------------------------------------
// Unit tests placeholder
// ---------------------------------------------------------------------------
// Tests are added by the test-writer (next pipeline step). This module
// contains only the stubs. No test code here.
