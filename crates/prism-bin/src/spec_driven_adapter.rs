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
//! - `BearerStaticAuthProvider` and `BearerStaticCredentialAuthProvider` live here (NOT in
//!   `prism-spec-engine`) because they bridge `SensorAuth` (prism-sensors) ↔ `AuthProvider`
//!   (prism-spec-engine). Only prism-bin imports both crates (ADR-023 §Permitted Patterns).
//! - `StaticCookieAuthProvider` lives in `prism-spec-engine/src/auth_provider.rs`.
//!
//! # Auth strategies (OQ-1 Resolution / ADV-SDEMO002-P01-CRIT-001 fix)
//!
//! `AdapterAuthStrategy` is held at construction time:
//! - `Plugin(Arc<dyn AuthProvider>)` — CrowdStrike: held PluginAuthProvider, ignores SensorAuth arg.
//!   Also used for Armis/Claroty: `BearerStaticCredentialAuthProvider` resolves bearer token
//!   from credential store at acquire_token() time — FAIL-CLOSED (no fabricated token).
//! - `BearerStatic` — DEPRECATED production path. Was: token extracted from SensorAuth arg.
//!   Retained for backward compat only; no longer constructed at step 9A (ADV-SDEMO002-P01-CRIT-001).
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
//! Story: S-DEMO-001 v1.3; ADV-SDEMO002-P01-CRIT-001 fix

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, time::Duration};

use serde_json::Value as JsonValue;

use arrow::{
    array::{
        Array, BooleanArray, Float64Array, Int32Array, Int64Array, StringArray,
        TimestampMicrosecondArray,
    },
    datatypes::{DataType, Field, Schema, TimeUnit},
    record_batch::RecordBatch,
};
use async_trait::async_trait;
use prism_core::{ColumnType, OrgId, SensorId};
use prism_ocsf::{EventClassSelector, OCSF_ENUM_LABEL_FIELDS, OcsfEnumMap};
use prism_sensors::{
    BearerStaticSensorAuth, SensorAdapter,
    adapter::{QueryParams, SensorError, SensorSpec},
    auth::SensorAuth,
};
use prism_spec_engine::{
    AuthProvider, AuthToken, PluginAuthProvider, ResolvedSensorSpec, ResolvedSpecKey,
    error::SpecEngineError,
    extract_at_path, parse_datetime_to_micros,
    pipeline::{FetchContext, PipelineExecutor, PipelineResult},
    spec_parser::{AuthType, ColumnSpec, SensorSpec as SpecEngineSensorSpec, TableSpec},
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
///
/// `#[non_exhaustive]`: new auth strategies (e.g., `ApiKey`, `Oauth2`) may be added in
/// future waves without breaking external match arms. External callers MUST include a
/// wildcard `_ => {}` arm per CLAUDE.md §Conventions non-exhaustive discipline (CR-001).
#[derive(Clone)]
#[non_exhaustive]
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
/// ## Debug impl
///
/// Explicit redacted `Debug` impl per AD-017 (CWE-209): the `token` field is replaced with
/// `"<redacted>"` in debug output so the bearer token never appears in logs or traces.
/// Symmetric with `BearerStaticSensorAuth` in prism-sensors/src/auth/mod.rs (SEC-001).
///
/// ## Non-exhaustive
///
/// `#[non_exhaustive]`: future fields (e.g., optional expiry, client hint) may be added
/// without requiring external callers to update struct literals. External callers MUST use
/// `BearerStaticAuthProvider::new(token)` per CLAUDE.md non-exhaustive discipline (CR-006).
///
/// BC-2.01.013 postcondition 4; ADR-023 §Permitted Patterns; OQ-1; AD-017; SEC-001.
#[non_exhaustive]
pub struct BearerStaticAuthProvider {
    /// Bearer token string.
    ///
    /// AD-017: this field holds the bearer token for the duration of a single fetch() call.
    /// The token is NOT stored at SpecDrivenSensorAdapter construction time.
    token: String,
}

/// AD-017 (CWE-209): redacted Debug impl for `BearerStaticAuthProvider`.
///
/// The bearer token is replaced with `"<redacted>"` in all debug output so it never
/// appears in logs, traces, or error messages. Symmetric with `BearerStaticSensorAuth`
/// in prism-sensors/src/auth/mod.rs (SEC-001 fix, S-DEMO-001 PR review).
impl std::fmt::Debug for BearerStaticAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BearerStaticAuthProvider")
            .field("token", &"<redacted>")
            .finish()
    }
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
// BearerStaticCredentialAuthProvider — production AuthProvider for bearer_static sensors
// ---------------------------------------------------------------------------

/// Production `AuthProvider` for sensors using `auth_type = "bearer_static"` (Armis, Claroty).
///
/// Lives in `prism-bin` (NOT `prism-spec-engine`) because it bridges `prism-credentials`
/// (credential resolution) ↔ `prism-spec-engine` (AuthProvider trait). Only `prism-bin`
/// imports both crates (ADR-023 §Permitted Patterns).
///
/// ## Behaviour (ADV-SDEMO002-P01-CRIT-001 fix)
///
/// Resolves the bearer token from `prism_credentials::resolve_credential` at
/// `acquire_token()` time using the injected `credential_ref_name` (e.g. `"bearer_token"`).
/// Env-var convention: `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` (ADR-032 / BC-2.06.003),
/// e.g. `PRISM_CLIENTS_DEMO_ORG_A_SENSORS_ARMIS_BEARER_TOKEN` for org_slug `demo-org-a`.
///
/// On resolution failure it returns `Err(SpecEngineError::AuthAcquisitionFailed)` —
/// FAIL-CLOSED. It NEVER fabricates or falls back to a placeholder token.
///
/// ## AD-017 Credential Safety
///
/// The resolved bearer token is NEVER stored as a field. It is resolved at
/// `acquire_token()` time and immediately wrapped in `AuthToken(Zeroizing<String>)`.
/// The struct holds ONLY `sensor_id` (credential namespace key) and
/// `credential_ref_name` (name of the credential to resolve) — NOT the token itself.
///
/// ## Object Safety
///
/// `acquire_token` returns `Pin<Box<dyn Future + Send>>` for `dyn AuthProvider`
/// compatibility (BC-2.01.017 / auth_provider.rs object-safety pattern).
///
/// ## Non-exhaustive
///
/// `#[non_exhaustive]`: future fields (e.g., token cache TTL) may be added
/// without breaking external callers. Construct via `BearerStaticCredentialAuthProvider::new`.
///
/// ADV-SDEMO002-P01-CRIT-001; BC-2.06.003; BC-2.01.013; AD-017.
#[non_exhaustive]
pub struct BearerStaticCredentialAuthProvider {
    /// Sensor ID used as the credential namespace key (e.g. `"armis"`, `"claroty"`).
    ///
    /// AD-017: credential value is NEVER stored here — only the namespace key.
    sensor_id: String,

    /// Name of the credential to resolve from the credential store (e.g. `"bearer_token"`).
    ///
    /// Resolved at `acquire_token()` time via `prism_credentials::resolve_credential`.
    credential_ref_name: String,

    /// Injected credential resolver (ADR-022 §C wiring / testability).
    ///
    /// Production code passes `Arc::new(prism_spec_engine::PrismCredentialResolver::new(org_registry, keyring))`.
    /// Tests inject `Arc::new(MockCredentialResolver::new("value"))` or
    /// `Arc::new(NotFoundCredentialResolver)` to drive fail-closed paths without
    /// relying on env vars or the real credential store (SID-1 discipline).
    resolver: std::sync::Arc<dyn prism_spec_engine::CredentialResolver>,
}

impl BearerStaticCredentialAuthProvider {
    /// Construct a `BearerStaticCredentialAuthProvider` for the given sensor and credential ref.
    ///
    /// Uses the production [`PrismCredentialResolver`] with injected `OrgRegistry` and keyring
    /// for Tier-3 resolution (ADR-034 §D1).
    ///
    /// - `sensor_id`: sensor name string from TOML spec (credential namespace key).
    /// - `credential_ref_name`: name of the credential ref declared in the TOML spec (e.g. `"bearer_token"`).
    /// - `org_registry`: for slug → OrgId resolution in `PrismCredentialResolver`.
    /// - `keyring`: OrgId-keyed keyring backend for Tier-3 resolution.
    ///
    /// ADV-SDEMO002-P01-CRIT-001; AD-017; BC-2.06.003; ADR-034 §D1.
    pub fn new(
        sensor_id: impl Into<String>,
        credential_ref_name: impl Into<String>,
        org_registry: std::sync::Arc<prism_core::OrgRegistry>,
        keyring: std::sync::Arc<dyn prism_credentials::CredentialStoreOrgId>,
    ) -> Self {
        Self {
            sensor_id: sensor_id.into(),
            credential_ref_name: credential_ref_name.into(),
            resolver: std::sync::Arc::new(prism_spec_engine::PrismCredentialResolver::new(
                org_registry,
                keyring,
            )),
        }
    }

    /// Construct a `BearerStaticCredentialAuthProvider` with an injectable credential resolver.
    ///
    /// Used by tests to inject a [`MockCredentialResolver`] or [`NotFoundCredentialResolver`]
    /// without relying on env vars or the real credential store (SID-1 discipline).
    ///
    /// ADV-SDEMO002-P01-CRIT-001; ADR-022 §C; SID-1.
    #[cfg(test)]
    pub fn new_with_resolver(
        sensor_id: impl Into<String>,
        credential_ref_name: impl Into<String>,
        resolver: std::sync::Arc<dyn prism_spec_engine::CredentialResolver>,
    ) -> Self {
        Self {
            sensor_id: sensor_id.into(),
            credential_ref_name: credential_ref_name.into(),
            resolver,
        }
    }
}

impl prism_spec_engine::AuthProvider for BearerStaticCredentialAuthProvider {
    /// Acquire the bearer token for the sensor by resolving from the credential store.
    ///
    /// Calls `prism_credentials::resolve_credential(client_id, sensor_id, credential_ref_name,
    /// org_id, keyring)` (5-arg signature per ADR-034 §D1).
    /// Returns `Ok(AuthToken)` wrapping the resolved token on success.
    ///
    /// FAIL-CLOSED: on any resolution failure returns `Err(AuthAcquisitionFailed)` with an
    /// E-AUTH-005 detail — NEVER falls back to a placeholder token.
    ///
    /// ADV-SDEMO002-P01-CRIT-001; BC-2.06.003; AD-017; BC-2.01.013.
    fn acquire_token<'a>(
        &'a self,
        _spec: &'a prism_spec_engine::spec_parser::SensorSpec,
        client_id: &'a prism_core::OrgSlug,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        prism_spec_engine::AuthToken,
                        prism_spec_engine::error::SpecEngineError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        use prism_credentials::CredentialResolutionError;
        use secrecy::ExposeSecret;

        let sensor_id = self.sensor_id.clone();
        let credential_ref_name = self.credential_ref_name.clone();
        let client_id_str = client_id.as_str().to_string();
        let resolver = std::sync::Arc::clone(&self.resolver);

        Box::pin(async move {
            // Resolve the bearer token from the credential store (BC-2.06.003 env-var chain).
            // FAIL-CLOSED: on NotFound or BackendUnavailable, return E-AUTH-005 and abort.
            // NEVER return a placeholder or fallback token (ADV-SDEMO002-P01-CRIT-001).
            let secret = match resolver
                .resolve(&client_id_str, &sensor_id, &credential_ref_name)
                .await
            {
                Ok(s) => s,
                Err(CredentialResolutionError::NotFound { .. }) => {
                    // BC-2.06.003 / ADR-032: per-client env var format.
                    // {ID} = org_slug uppercased with hyphens → underscores.
                    let id_upper =
                        prism_credentials::resolution::slug_to_screaming_snake(&client_id_str);
                    let sensor_upper = sensor_id.to_uppercase().replace('-', "_");
                    let ref_upper = credential_ref_name.to_uppercase().replace('-', "_");
                    let per_client_env =
                        format!("PRISM_CLIENTS_{id_upper}_SENSORS_{sensor_upper}_{ref_upper}");
                    return Err(
                        prism_spec_engine::error::SpecEngineError::AuthAcquisitionFailed {
                            sensor_id: sensor_id.clone(),
                            client_id: client_id_str.clone(),
                            detail: format!(
                                "E-AUTH-005: bearer token not found — no '{credential_ref_name}' \
                                 credential configured for sensor '{sensor_id}', \
                                 client '{client_id_str}'. \
                                 Set env var {per_client_env} (ADR-032 / BC-2.06.003).",
                            ),
                        },
                    );
                }
                Err(CredentialResolutionError::BackendUnavailable { detail, .. }) => {
                    return Err(
                        prism_spec_engine::error::SpecEngineError::AuthAcquisitionFailed {
                            sensor_id: sensor_id.clone(),
                            client_id: client_id_str.clone(),
                            detail: format!(
                                "E-AUTH-007: credential backend unavailable for sensor \
                                 '{sensor_id}' credential '{credential_ref_name}': {detail}. \
                                 Check the configured backend (env file, keyring). \
                                 BC-2.06.003 / ADV-SDEMO002-P01-CRIT-001."
                            ),
                        },
                    );
                }
            };

            // Return the resolved token wrapped in AuthToken(Zeroizing<String>).
            // The Zeroizing wrapper ensures the token bytes are overwritten on drop (AD-017).
            let token = secret.expose_secret().to_string();
            Ok(prism_spec_engine::AuthToken::new(token))
        })
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
/// ## OCSF normalization (BC-2.01.013)
///
/// The `PipelineExecutor` does NOT return Arrow `RecordBatch` — it returns `PipelineResult`
/// (raw JSON records). `SpecDrivenSensorAdapter::fetch()` converts those records to
/// `Vec<RecordBatch>` via `pipeline_result_to_record_batch`, which:
/// 1. Maps spec-declared columns into the Arrow batch via `build_column_array` (typed per
///    the TOML `[[tables.columns]]` spec; absent fields become null).
/// 2. Derives `class_uid` via `EventClassSelector::select_by_class_name(ocsf_class)`;
///    derives `category_uid = class_uid / 1000` (OCSF standard category encoding).
/// 3. Injects `_sensor` as the canonical `SensorId` from the spec — the raw record's
///    `_sensor` field (if any) is never used (untrusted vendor data).
///
/// ## Non-exhaustive
///
/// `#[non_exhaustive]`: future fields (e.g., per-adapter rate-limiter handle, telemetry
/// sink) may be added as the adapter evolves without requiring external callers to update
/// struct literals. External callers MUST use `SpecDrivenSensorAdapter::new(...)` per
/// CLAUDE.md non-exhaustive discipline (CR-006).
///
/// BCs: BC-2.01.013, BC-2.06.014, BC-2.11.005; Story: S-DEMO-001 v1.6.
#[non_exhaustive]
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
    /// - `BearerStatic`: extracts bearer token from `auth: &dyn SensorAuth` via downcast to
    ///   `BearerStaticSensorAuth`, constructs `BearerStaticAuthProvider` per-call (OQ-1 Resolution).
    /// - `StaticCookie`: uses the held `StaticCookieAuthProvider` (NO HTTP calls at
    ///   acquire_token; `build_request` injects `Cookie: access_token={token}` per ADR-031 §D3-b).
    ///
    /// Maps `PipelineResult` (raw JSON) → `Vec<RecordBatch>` via OCSF normalization (BC-2.11.005).
    /// Each table in the sensor spec is executed sequentially; results are concatenated.
    ///
    /// On double-401: propagates `SpecEngineError::AuthRefreshFailed` →
    /// `SensorError::HttpError { status: 401 }` (sensor responded, credentials persistently
    /// invalid; BC-2.08.002 / AC-ERR-001). Prior to DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOW-1 fix,
    /// this incorrectly mapped to `SensorError::Internal`, causing `probe_connectivity` to
    /// classify a reachable sensor as `Down`.
    ///
    /// BC-2.01.013 postcondition 4; OQ-1 Resolution; ADR-028 §D10; ADR-031 §D3-b.
    async fn fetch(
        &self,
        spec: &SensorSpec,
        params: &QueryParams,
        auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        // Select the auth provider based on the held auth_strategy (OQ-1 Resolution).
        // ADR-028 §D10: Plugin and StaticCookie ignore the SensorAuth arg.
        // BearerStatic extracts the token from the SensorAuth arg via downcast.
        let auth_provider: Arc<dyn AuthProvider> = match &self.auth_strategy {
            AdapterAuthStrategy::Plugin(provider) => {
                // CrowdStrike: use the held PluginAuthProvider; ignore SensorAuth arg (ADR-028 §D10).
                Arc::clone(provider)
            }
            AdapterAuthStrategy::BearerStatic => {
                // Armis/Claroty: extract bearer token from SensorAuth arg via downcast (OQ-1).
                // The production path expects &BearerStaticSensorAuth from prism-sensors.
                // If downcast fails (e.g., test-local stub type), log a diagnostic warn
                // (SEC-003: plain warn without event_type= so no BC-2.16.002 catalog row
                // is required — this is an internal diagnostic, not an auditable event;
                // SAP-1 exemption applies to non-event_type= skip-path warns) and return
                // Internal error so the query engine can propagate it cleanly.
                let bearer_auth = match auth.as_any().downcast_ref::<BearerStaticSensorAuth>() {
                    Some(ba) => ba,
                    None => {
                        // No event_type= field: plain diagnostic, SAP-1 exempt.
                        tracing::warn!(
                            sensor_id = %self.sensor_spec.spec.sensor_id,
                            auth_type_name = %auth.auth_type_name(),
                            "bearer static auth downcast failed: expected BearerStaticSensorAuth; \
                             adapter NOT fetching. Ensure caller passes BearerStaticSensorAuth \
                             for bearer_static sensors. S-DEMO-001 OQ-1.",
                        );
                        return Err(SensorError::Internal {
                            detail: format!(
                                "E-SPEC-012: BearerStatic auth strategy requires BearerStaticSensorAuth; \
                                 got auth_type_name='{}'. Ensure the caller passes a BearerStaticSensorAuth \
                                 instance for bearer_static sensors. S-DEMO-001 OQ-1.",
                                auth.auth_type_name()
                            ),
                        });
                    }
                };
                Arc::new(BearerStaticAuthProvider::new(bearer_auth.token.clone()))
            }
            AdapterAuthStrategy::StaticCookie(provider) => {
                // Cyberint: use the held StaticCookieAuthProvider; ignore SensorAuth arg (ADR-028 §D10).
                Arc::clone(provider)
            }
        };

        // Build the FetchContext from the resolved spec's org_slug.
        // Thread params.filters into FetchContext.query_filters (F-003).
        // FilterMap = HashMap<String, serde_json::Value>; FetchContext.query_filters
        // = HashMap<String, String>. Convert by serializing Value → String.
        // The client_id for the pipeline is the org_slug (human-readable org identifier).
        let mut query_filters: std::collections::HashMap<String, String> = params
            .filters
            .iter()
            .map(|(k, v)| {
                let s = match v {
                    JsonValue::String(s) => s.clone(),
                    other => other.to_string(),
                };
                (k.clone(), s)
            })
            .collect();

        // CrowdStrike FQL time-window injection (BC-2.01.013 + ADR-033 T1).
        // Seed `_fql` into query_filters for CrowdStrike so the ${query.filter._fql}
        // slot in the Step 1 path_template always resolves (empty string when no filter).
        // When start_time/end_time are populated (by extract_time_window_from_ast in
        // run_materialization_pipeline), construct the FQL filter string.
        // FQL format: `created_timestamp:>'<ISO8601>'` (start) +
        //             `created_timestamp:<'<ISO8601>'` (end), combined with `+`.
        if self.sensor_spec.spec.sensor_id.as_str() == "crowdstrike" {
            let fql =
                build_crowdstrike_fql(params.start_time.as_deref(), params.end_time.as_deref());
            query_filters.entry("_fql".to_string()).or_insert(fql);
        }

        // Armis AQL time-window augmentation (BC-2.01.013 Mechanism B + ADR-033 T1).
        // F-P1-CRIT-002: wire augment_armis_aql_with_time_window into the real path.
        //
        // When start_time/end_time are populated by extract_time_window_from_ast (ADR-033 T1)
        // in run_materialization_pipeline, augment the base AQL string with canonical
        // Armis time clauses: `after:YYYY-MM-DDTHH:MM:SS` / `before:YYYY-MM-DDTHH:MM:SS`
        // (bare, unquoted, timezone-naive per research-doc §2.2, AC-ARMIS-TW-001).
        //
        // Anti-double-filter guard: if the AQL already contains `after:`, `before:`, or
        // `timeFrame:`, augmentation is skipped (AC-ARMIS-TW-003 / BC-2.01.013 Mechanism B).
        //
        // The augmented AQL overwrites `query_filters["aql"]` and is forwarded via the
        // existing `${query.filter.aql}` path_template interpolation.
        if self.sensor_spec.spec.sensor_id.as_str() == "armis"
            && (params.start_time.is_some() || params.end_time.is_some())
        {
            let base_aql = query_filters.get("aql").cloned().unwrap_or_default();
            let augmented = prism_query::pushdown::augment_armis_aql_with_time_window(
                &base_aql,
                params.start_time.as_deref(),
                params.end_time.as_deref(),
            );
            query_filters.insert("aql".to_string(), augmented);
        }

        // CrowdStrike limit push-down (BC-2.01.013 / F-P1-CRIT-004).
        // Seed `query.limit` into query_filters so the ${query.limit} slot in the
        // CrowdStrike Step 1 path_template resolves to the LIMIT value.
        // When params.limit == 0 (no LIMIT clause), seed an empty string so
        // PipelineExecutor::strip_empty_url_params removes the &limit= param entirely.
        // EC-008: treat limit=0 as "no limit" — empty string causes omission.
        if self.sensor_spec.spec.sensor_id.as_str() == "crowdstrike" {
            // Use entry() to not overwrite a manually-injected query.limit (test path).
            query_filters
                .entry("query.limit".to_string())
                .or_insert_with(|| {
                    if params.limit > 0 {
                        params.limit.to_string()
                    } else {
                        String::new() // empty → stripped by PipelineExecutor::strip_empty_url_params
                    }
                });
        }

        let context = FetchContext::new(self.sensor_spec.org_slug.clone(), query_filters);

        // Resolve which sensor table to execute.
        //
        // `_spec.source_table` is e.g. `"armis_devices"` — the fully-qualified table name
        // used by PrismQL `FROM armis_devices`. The sensor spec's `tables` collection may
        // contain multiple entries (`devices`, `alerts`, etc.). We must only run the table
        // that matches the queried source_table to avoid:
        //   1. Executing unnecessary HTTP requests for non-queried tables.
        //   2. Schema mismatches when DataFusion tries to register multi-schema batches.
        //
        // Extraction: strip the sensor_id prefix and underscore to get the raw table name.
        // E.g., "armis_devices" - "armis_" = "devices". Falls back to running all tables if
        // the prefix is not found (defensive; should not occur in normal operation).
        let sensor_id_str = self.sensor_spec.spec.sensor_id.as_str();
        let queried_table_name: Option<&str> =
            spec.source_table.strip_prefix(&format!("{sensor_id_str}_"));

        // Delegate to PipelineExecutor::execute() for each table in the sensor spec.
        // Collect all RecordBatches by normalizing JSON records → Arrow (BC-2.11.005).
        let mut all_batches: Vec<RecordBatch> = Vec::new();

        for table in &self.sensor_spec.spec.tables {
            // Skip tables that don't match the queried source_table.
            // When `queried_table_name` is Some (normal case), only execute the matching table.
            // When None (strip_prefix failed — defensive fallback), run all tables.
            if queried_table_name.is_some_and(|qtn| table.table_name != qtn) {
                continue;
            }
            let result = PipelineExecutor::execute(
                &self.sensor_spec.spec,
                table,
                &context,
                &self.http_client,
                auth_provider.as_ref(),
            )
            .await
            .map_err(|e| {
                map_spec_engine_error_to_sensor_error(
                    e,
                    &self.sensor_spec.spec.sensor_id,
                    &table.table_name,
                )
            })?;

            // Convert PipelineResult.records (raw JSON) → Arrow RecordBatch
            // with OCSF envelope columns (category_uid, class_uid, _sensor) and
            // spec-defined data columns (BC-2.11.005 / AC-010).
            // BC-2.01.013 OCSF Conformance: pass `table` so that:
            //   - spec-declared columns survive into the Arrow schema (item 1)
            //   - class_uid/category_uid are derived from ocsf_class (item 2)
            //   - _sensor is injected as canonical sensor_id (item 3)
            if !result.records.is_empty() {
                let batch = pipeline_result_to_record_batch(
                    result,
                    table,
                    &self.sensor_spec.spec.sensor_id,
                    &params.filters,
                );
                match batch {
                    Ok(b) => all_batches.push(b),
                    Err(e) => {
                        return Err(SensorError::Internal {
                            detail: format!(
                                "SpecDrivenSensorAdapter: OCSF→Arrow normalization failed for \
                                 sensor='{}' table='{}': {e}",
                                self.sensor_spec.spec.sensor_id, table.table_name
                            ),
                        });
                    }
                }
            }
        }

        Ok(all_batches)
    }
}

// ---------------------------------------------------------------------------
// map_spec_engine_error_to_sensor_error — error taxonomy mapping (BC-2.08.002 / AC-ERR-001..AC-ERR-002)
// ---------------------------------------------------------------------------

/// Map `SpecEngineError` → `SensorError`, classifying HTTP-status-bearing failures (4xx/5xx,
/// and persistent-401 auth failures) into `SensorError::HttpError` and transport/other failures
/// into `SensorError::Internal`, so health probes can distinguish reachable-but-erroring sensors
/// from unreachable ones.
///
/// BC-2.08.002 / AC-ERR-001..AC-ERR-002 HTTP Error Classification (DEFECT-ADAPTER-TLS-XDOME-LIVE-001):
///
/// **Arm 1 — HTTP response received (`status_code > 0`):**
/// `HttpRequestFailed { status_code > 0, .. }` → `SensorError::HttpError { status, body }`.
/// The `status` field carries the numeric HTTP status code. The `body` field carries the
/// RAW sanitized body snippet extracted from `detail` — NOT the full detail string.
///
/// `pipeline.rs` `issue_request_with_retry` formats `detail` as
/// `"HTTP {status_reason}: {body_snippet}"` (e.g. `"HTTP 403 Forbidden: access denied"`).
/// Putting the full detail string into `HttpError.body` caused a doubled prefix in
/// `materialization.rs` (`"{table}: HTTP {status}: HTTP 403 Forbidden: {body}"`), violating
/// BC-2.11.001 EC-11-088/089 (F-P37-HIGH-001). The fix strips the `"HTTP {reason}: "` prefix
/// so consumers (materialization.rs, connectivity.rs) see the raw snippet and build their own
/// presentation from the separate numeric `status` field.
///
/// This allows callers (e.g., `probe_connectivity`) to classify 4xx responses as
/// reachable/auth-invalid rather than erroneously mapping them to `ConnectivityStatus::Down`.
///
/// **Arm 2 — Persistent 401 (auth refresh or cookie auth failed):**
/// `AuthRefreshFailed` and `CookieAuthFailed` both mean the sensor responded with HTTP 401
/// (the sensor IS reachable) but the credentials are persistently invalid. Map to
/// `SensorError::HttpError { status: 401 }` so `probe_connectivity` correctly classifies
/// these as `ConnectivityStatus::Up` and `probe_auth_with_routing` classifies them as
/// `AuthStatus::Invalid` (BC-2.08.002 / DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOW-1 fix).
///
/// **Arm 3 — Transport failure or other error (`status_code == 0` or other variant):**
/// All remaining `SpecEngineError` variants (including `HttpRequestFailed { status_code: 0 }`
/// for connection/send errors) → `SensorError::Internal` with a structured `detail` message
/// containing the sensor ID, table name, and the original error's `Display` representation.
fn map_spec_engine_error_to_sensor_error(
    e: SpecEngineError,
    sensor_id: &str,
    table_name: &str,
) -> SensorError {
    // Arm 1: BC-2.08.002 HTTP Error Classification postcondition (DEFECT-ADAPTER-TLS-XDOME-LIVE-001):
    // HttpRequestFailed { status_code > 0 } = HTTP response received → HttpError.
    // HttpRequestFailed { status_code = 0 } = transport failure (no HTTP response) → skip to Arm 3.
    if let SpecEngineError::HttpRequestFailed {
        status_code,
        ref detail,
        ..
    } = e
        && status_code > 0
    {
        // F-P37-HIGH-001: pipeline.rs formats detail as "HTTP {status_reason}: {body_snippet}"
        // (e.g. "HTTP 403 Forbidden: access denied"). Putting the full detail into HttpError.body
        // caused materialization.rs to double-prefix: "{table}: HTTP {code}: HTTP 403 Forbidden: ...".
        // Fix: strip the "HTTP {reason}: " prefix so HttpError.body = raw sanitized snippet only.
        // Consumers (materialization.rs, connectivity.rs) build their own presentation from the
        // separate numeric status field + raw body snippet.
        // Stripping: skip "HTTP " prefix, then take everything after the first ": " separator
        // (the separator between the reason phrase and the body snippet). If no ": " exists
        // (empty-body case: detail = "HTTP 503 Service Unavailable"), body is empty string.
        let raw_body = detail
            .strip_prefix("HTTP ")
            .and_then(|s| s.find(": ").map(|idx| s[idx + 2..].to_string()))
            .unwrap_or_default();
        return SensorError::HttpError {
            sensor: sensor_id.to_string(),
            status: status_code,
            body: raw_body,
        };
    }
    // Arm 2: Persistent 401 — sensor IS reachable but credentials are persistently invalid.
    // AuthRefreshFailed = double-401 after token refresh (OAuth2/Plugin auth).
    // CookieAuthFailed  = 401 on CookieRoundtrip auth (no refresh possible).
    // Both mean HTTP 401 was received → map to HttpError{status:401} so probe_connectivity
    // correctly classifies as Up + probe_auth_with_routing classifies as AuthStatus::Invalid.
    if matches!(
        e,
        SpecEngineError::AuthRefreshFailed { .. } | SpecEngineError::CookieAuthFailed { .. }
    ) {
        return SensorError::HttpError {
            sensor: sensor_id.to_string(),
            status: 401,
            body: format!("{e}"),
        };
    }
    // Arm 3: transport error and all other SpecEngineError variants → Internal.
    SensorError::Internal {
        detail: format!(
            "SpecDrivenSensorAdapter: PipelineExecutor::execute failed for \
             sensor='{sensor_id}' table='{table_name}': {e}",
        ),
    }
}

// ---------------------------------------------------------------------------
// pipeline_result_to_record_batch — OCSF→Arrow normalization (AC-010 / BC-2.11.005)
// ---------------------------------------------------------------------------

/// Convert `PipelineResult.records` (raw JSON) to an Arrow `RecordBatch`.
///
/// Produces a RecordBatch with (BC-2.01.013 OCSF Conformance Clause):
///
/// **Spec-declared data columns (item 1):**
/// Every column declared in `table.columns` is included in the schema, extracted
/// from the raw record by name, and typed per `ColumnSpec::column_type`.
/// Columns absent from a record become null.
///
/// **Derived OCSF envelope columns (item 2):**
/// - `class_uid` (Int32): derived via `EventClassSelector::select_by_class_name(ocsf_class)`.
///   Falls back to 0 (BASE_EVENT) if no class-name mapping exists (D-925 intentional fallback).
/// - `category_uid` (Int32): `class_uid / 1000` per the OCSF standard category encoding.
///
/// **Canonical _sensor virtual column (item 3):**
/// - `_sensor` (Utf8): always injected as the canonical `sensor_id` from the spec.
///   The raw record's `_sensor` field (if any) is NEVER used — this field can be
///   tampered by the sensor vendor. The spec `sensor_id` is the authoritative value.
///
/// # Column ordering in schema
///
/// Spec-declared data columns appear first (in spec order), followed by the three
/// OCSF envelope columns: `category_uid`, `class_uid`, `_sensor`.
///
/// # Design Rationale (BC-2.11.005)
///
/// `PipelineExecutor::execute()` returns raw JSON; Arrow conversion happens here
/// (not in the query engine's materialization layer). This is the boundary where
/// raw sensor data becomes typed Arrow data with enforced OCSF provenance.
///
/// # Errors
///
/// Returns `arrow::error::ArrowError` if schema/column construction fails.
/// `push_down_filters`: the query push-down filters (from `QueryParams.filters`).
/// Used to populate INDEX-only columns (push-down pseudo-columns) with the actual
/// filter value so DataFusion's WHERE clause evaluates correctly.
///
/// Example: `aql` column with `options = ["INDEX"]` for Armis sensors.
/// `WHERE aql = 'in:devices'` in the SQL is push-downed to the pipeline.
/// Without injection, the `aql` column would be NULL in every row → DataFusion
/// would filter out all rows (NULL != 'in:devices'). With injection, every row
/// has `aql = 'in:devices'` → DataFusion's WHERE clause correctly matches.
fn pipeline_result_to_record_batch(
    result: PipelineResult,
    table: &TableSpec,
    sensor_id: &str,
    push_down_filters: &prism_sensors::types::FilterMap,
) -> Result<RecordBatch, arrow::error::ArrowError> {
    // CR-004: caller guards `if !result.records.is_empty()` before calling here (see fetch()).
    // The n==0 early-return was a dead branch — replaced with a debug_assert to catch
    // misuse in test builds without dead-code overhead in production.
    debug_assert!(
        !result.records.is_empty(),
        "pipeline_result_to_record_batch: caller must not pass empty records \
         (guard `if !result.records.is_empty()` in fetch() ensures this invariant)"
    );

    let n = result.records.len();

    // BC-2.01.013 item 2: derive class_uid from spec ocsf_class via
    // EventClassSelector::select_by_class_name — looks up by OCSF class-name string,
    // not by (sensor_id, record_type) pair. Falls back to 0 (BASE_EVENT) for unmapped
    // tables per D-925 (intentional unwrap_or fallback, not a production error path).
    let derived_class_uid: i32 =
        EventClassSelector::select_by_class_name(&table.ocsf_class).unwrap_or(0) as i32;
    // OCSF standard encoding: category_uid = class_uid / 1000
    // e.g. class_uid=2004 → category_uid=2 (Findings), class_uid=3001 → category_uid=3 (IAM)
    let derived_category_uid: i32 = derived_class_uid / 1000;

    // Build schema: spec-declared data columns first, then OCSF envelope.
    let mut fields: Vec<Field> = table
        .columns
        .iter()
        .map(|col| Field::new(&col.name, column_type_to_arrow(&col.column_type), true))
        .collect();
    fields.push(Field::new("category_uid", DataType::Int32, true));
    fields.push(Field::new("class_uid", DataType::Int32, true));
    fields.push(Field::new("_sensor", DataType::Utf8, true));
    let schema = Arc::new(Schema::new(fields));

    // Build per-column value vectors for spec-declared data columns.
    // Each column is extracted from the raw record by name; absent values → None (null).
    let mut col_arrays: Vec<Arc<dyn Array>> = Vec::with_capacity(table.columns.len() + 3);

    for col_spec in &table.columns {
        // For INDEX-only (push-down pseudo) columns: inject the push-down filter value
        // into every row so DataFusion's WHERE clause evaluates correctly.
        //
        // Example: `aql` column with options = ["INDEX"] on Armis sensors.
        // `WHERE aql = 'in:devices'` is push-downed to the pipeline; without injection,
        // `aql` would be NULL in every row → DataFusion filters out all rows.
        // Injecting the filter value makes the WHERE clause evaluate to TRUE for each row.
        let array = if col_spec.options.contains(&prism_core::ColumnOptions::Index) {
            // Look up the filter value from push_down_filters by column name.
            // If the column name maps to a filter value, inject it as a string constant
            // across all rows. If no filter found, fall through to normal extraction
            // (which will yield NULLs — the row may still be filtered by DataFusion).
            if let Some(filter_val) = push_down_filters.get(&col_spec.name) {
                let s = match filter_val {
                    serde_json::Value::String(sv) => Some(sv.clone()),
                    other => Some(other.to_string()),
                };
                let vals: Vec<Option<String>> = vec![s; n];
                Arc::new(arrow::array::StringArray::from(vals)) as Arc<dyn Array>
            } else {
                build_column_array(&result.records, col_spec, sensor_id)
            }
        } else {
            build_column_array(&result.records, col_spec, sensor_id)
        };
        col_arrays.push(array);
    }

    // BC-2.01.013 item 2: OCSF envelope — class_uid/category_uid derived, not raw-copied.
    // All rows in this batch share the same derived class_uid/category_uid (table-level, not row-level).
    let category_uid_vals: Vec<Option<i32>> = vec![Some(derived_category_uid); n];
    let class_uid_vals: Vec<Option<i32>> = vec![Some(derived_class_uid); n];
    col_arrays.push(Arc::new(Int32Array::from(category_uid_vals)) as Arc<dyn Array>);
    col_arrays.push(Arc::new(Int32Array::from(class_uid_vals)) as Arc<dyn Array>);

    // BC-2.01.013 item 3: _sensor is ALWAYS the canonical sensor_id from the spec.
    // Never reads from raw record — the raw record's _sensor field is untrusted vendor data.
    let sensor_vals: Vec<Option<&str>> = vec![Some(sensor_id); n];
    col_arrays.push(Arc::new(StringArray::from(sensor_vals)) as Arc<dyn Array>);

    RecordBatch::try_new(schema, col_arrays)
}

/// Map a `ColumnType` to the corresponding Arrow `DataType`.
///
/// Used to build the RecordBatch schema from the sensor spec's declared columns.
/// `prism_core::column::ColumnType` canonical variants per ADR-024:
///   String / Integer / Float / Boolean / Datetime / Json
fn column_type_to_arrow(col_type: &ColumnType) -> DataType {
    match col_type {
        ColumnType::String => DataType::Utf8,
        ColumnType::Integer => DataType::Int64,
        ColumnType::Float => DataType::Float64,
        ColumnType::Boolean => DataType::Boolean,
        // Datetime → Timestamp(Microsecond, UTC-tagged) per ADR-052 D1/D2.
        // Sensor data is normalized to RFC-3339 at the adapter boundary;
        // Arrow stores it as i64 microseconds-since-epoch.
        ColumnType::Datetime => DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC"))),
        // Json → Utf8 (serialized JSON string in Arrow column).
        ColumnType::Json => DataType::Utf8,
        // Non-exhaustive guard: future variants default to Utf8.
        _ => DataType::Utf8,
    }
}

// ---------------------------------------------------------------------------
// F-CRIT-002 / BC-2.02.013 — OCSF enum-label normalization in build_column_array
// ---------------------------------------------------------------------------

// OCSF_ENUM_LABEL_FIELDS is imported from prism_ocsf (the single canonical definition).
// Previously duplicated here as a local `const` — removed by F-OBS-3 (LOCAL-pass-11):
// prism_ocsf::OCSF_ENUM_LABEL_FIELDS is now re-exported from prism_ocsf::lib and used
// directly here. Eliminates drift risk from two independent copies (TD-VSDD-060).
//
// The process-wide OcsfEnumMap singleton is now accessed via prism_ocsf::shared_enum_map()
// (F-P16-OBS-001, LOCAL-pass-16). The duplicate OnceLock<OcsfEnumMap> static that
// previously lived here has been removed; both prism-ocsf and prism-bin now share the same
// singleton through the pub re-export. TD-VSDD-060 sibling-site sweep: one call site remains
// below in build_column_array (the single canonical use point in this crate).

/// Build an Arrow array for a single column across all records.
///
/// ENRICH-1: dispatches on `col.source_path`:
/// - `None` (default) → flat `r.get(&col.name)` lookup (pre-ENRICH-1 behavior, backward compat).
/// - `Some(path)` → `extract_at_path(r, path)` via the spec-engine extractor.
///   Wildcard paths (`[*]`) that yield `Value::Array` are serialized to a compact JSON-list
///   string for `String`-typed columns (e.g., `["h1","h2"]`). Non-string types on a wildcard
///   path use first-element extraction with a tracing::warn (unusual; wildcard on non-string).
///   On `Err` from `extract_at_path`, the cell becomes null.
///
/// Records where the field is absent or null produce a null entry in the array.
fn build_column_array(
    records: &[serde_json::Value],
    col: &ColumnSpec,
    sensor_id: &str,
) -> Arc<dyn Array> {
    /// Extract a single raw `serde_json::Value` from a record for this column.
    fn extract_raw(r: &serde_json::Value, col: &ColumnSpec) -> Option<serde_json::Value> {
        if let Some(ref path) = col.source_path {
            match extract_at_path(r, path) {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::warn!(
                        column = %col.name,
                        source_path = %path,
                        error = %e,
                        event_type = "column_source_path_extraction_failed",
                        "ENRICH-1: build_column_array source_path extraction failed; cell is null"
                    );
                    None
                }
            }
        } else {
            r.get(&col.name).cloned()
        }
    }

    match &col.column_type {
        ColumnType::Integer => {
            let vals: Vec<Option<i64>> = records
                .iter()
                .map(|r| {
                    let raw = extract_raw(r, col)?;
                    match raw {
                        serde_json::Value::Array(arr) => {
                            // DD-5 item 3: wildcard path on a numeric/bool column yields an array.
                            // Use first-element extraction with a plain tracing::warn! (no event_type=
                            // field — this is a diagnostic warn, not an auditable structured event;
                            // SAP-1 only catalogs event_type= emissions, so no BC-2.16.002 row needed).
                            tracing::warn!(
                                column = %col.name,
                                source_path = col.source_path.as_deref().unwrap_or("(none)"),
                                array_len = arr.len(),
                                "ENRICH-1 DD-5: wildcard path on Integer column yields array; \
                                 using first element (F-ENRICH-P1-LOW-001)"
                            );
                            arr.into_iter().next().and_then(|v| v.as_i64())
                        }
                        other => other.as_i64(),
                    }
                })
                .collect();
            Arc::new(Int64Array::from(vals))
        }
        ColumnType::Float => {
            let vals: Vec<Option<f64>> = records
                .iter()
                .map(|r| {
                    let raw = extract_raw(r, col)?;
                    match raw {
                        serde_json::Value::Array(arr) => {
                            // DD-5 item 3: first-element with plain warn (no event_type=; SAP-1 exempt).
                            tracing::warn!(
                                column = %col.name,
                                source_path = col.source_path.as_deref().unwrap_or("(none)"),
                                array_len = arr.len(),
                                "ENRICH-1 DD-5: wildcard path on Float column yields array; \
                                 using first element (F-ENRICH-P1-LOW-001)"
                            );
                            arr.into_iter().next().and_then(|v| v.as_f64())
                        }
                        other => other.as_f64(),
                    }
                })
                .collect();
            Arc::new(Float64Array::from(vals))
        }
        ColumnType::Boolean => {
            let vals: Vec<Option<bool>> = records
                .iter()
                .map(|r| {
                    let raw = extract_raw(r, col)?;
                    match raw {
                        serde_json::Value::Array(arr) => {
                            // DD-5 item 3: first-element with plain warn (no event_type=; SAP-1 exempt).
                            tracing::warn!(
                                column = %col.name,
                                source_path = col.source_path.as_deref().unwrap_or("(none)"),
                                array_len = arr.len(),
                                "ENRICH-1 DD-5: wildcard path on Boolean column yields array; \
                                 using first element (F-ENRICH-P1-LOW-001)"
                            );
                            arr.into_iter().next().and_then(|v| v.as_bool())
                        }
                        other => other.as_bool(),
                    }
                })
                .collect();
            Arc::new(BooleanArray::from(vals))
        }
        // Datetime → Timestamp(Microsecond, Some("UTC")) (ADR-052 D5 / AC-013).
        // ISO-8601 datetime strings from sensor APIs are parsed via `parse_datetime_to_micros`
        // → i64 microseconds-since-epoch, matching the Arrow column type registered by
        // `column_type_to_arrow`. Identical chrono strictness to the E-QUERY-041 pre-validator
        // (ADR-052 D4 invariant).
        //
        // Normalizer contract (ADR-052 D5): `prism_spec_engine::pipeline::normalize_timestamp_fields`
        // runs BEFORE this arm and guarantees all non-null datetime values are emitted as
        // RFC-3339 with a Z-suffix via `dt.to_rfc3339_opts(Secs, use_z=true)`. The
        // `Err` branch below (warn + null cell) is defense-in-depth for any sensor
        // adapter that bypasses the normalizer; it is unreachable in the canonical path.
        // Note: the normalizer itself is lenient-IN (tries multiple date formats) but
        // strict-OUT (always emits Z-suffix RFC-3339); this arm is strict-IN via
        // `parse_datetime_to_micros` → `parse_from_rfc3339`, which the normalizer's
        // strict-OUT guarantee satisfies on the canonical path.
        ColumnType::Datetime => {
            let vals: Vec<Option<i64>> = records
                .iter()
                .map(|r| {
                    let raw = extract_raw(r, col)?;
                    let s = match raw {
                        serde_json::Value::String(s) => s,
                        serde_json::Value::Null => return None,
                        other => other.to_string(),
                    };
                    match parse_datetime_to_micros(&s, &col.name, sensor_id) {
                        Ok(micros) => Some(micros),
                        Err(e) => {
                            tracing::warn!(
                                column = %col.name,
                                sensor_id = %sensor_id,
                                // SEC-002 (CWE-532 / AD-017): cap raw sensor value at
                                // 50 codepoints so unbounded strings from untrusted sensor
                                // data cannot flood logs. Consistent with E-QUERY-041/042
                                // value_prefix convention.
                                // CR-004 / SEC-001 (CWE-117): sanitize_for_log strips Unicode Cc
                                // (C0 U+0000–U+001F, DEL U+007F, C1 U+0080–U+009F) + U+2028/U+2029
                                // BEFORE the 50-codepoint cap (BC-2.16.002 catalog row 91 spec
                                // order: sanitize → truncate).
                                value = %prism_core::sanitize_for_log(&s).chars().take(50).collect::<String>(),
                                error = %e,
                                "ADR-052: datetime string not parseable as RFC-3339 UTC; \
                                 cell produced null (sensor data should be RFC-3339)"
                            );
                            None
                        }
                    }
                })
                .collect();
            Arc::new(TimestampMicrosecondArray::from(vals).with_timezone("UTC"))
        }
        // String → Utf8 with OCSF enum-label normalization for the four labeled fields.
        //
        // F-CRIT-002 / BC-2.02.013: columns named in OCSF_ENUM_LABEL_FIELDS have their
        // string values normalized to OCSF canonical Title-case via OcsfEnumMap before Arrow
        // materialization. Unrecognized values pass through as-received with a structured warn.
        // Non-OCSF-labeled String columns pass through unchanged (same as the _ arm below).
        ColumnType::String => {
            let is_ocsf_enum_field = OCSF_ENUM_LABEL_FIELDS.contains(&col.name.as_str());
            let ocsf_map: Option<&OcsfEnumMap> = if is_ocsf_enum_field {
                Some(prism_ocsf::shared_enum_map())
            } else {
                None
            };

            let vals: Vec<Option<String>> = records
                .iter()
                .map(|r| {
                    let raw = extract_raw(r, col)?;
                    match raw {
                        serde_json::Value::Null => None,
                        serde_json::Value::String(s) => {
                            if let Some(map) = ocsf_map {
                                // OBS-3 (S-PRISMQL-CASE-INSENSITIVE-001): empty strings bypass
                                // enum-label normalization.  An empty value indicates an
                                // unset/missing sensor field — there is no corresponding OCSF
                                // canonical label, and emitting ocsf.enum_label_unrecognized would
                                // be misleading noise.  Mirrors the `!s.is_empty()` guard on the
                                // SECONDARY normalizer.rs path (ProtoValue::String branch).
                                if s.is_empty() {
                                    return Some(s);
                                }
                                match map.normalize_enum_label(&col.name, &s) {
                                    Some(canonical) => Some(canonical.to_string()),
                                    None => {
                                        // SEC-002 pattern: cap value at 50 codepoints to bound
                                        // log volume for adversarially long vendor strings.
                                        tracing::warn!(
                                            event_type = "ocsf.enum_label_unrecognized",
                                            field_name = %col.name,
                                            // CR-004 / SEC-001 (CWE-117): sanitize_for_log strips
                                            // Unicode Cc (C0 U+0000–U+001F, DEL U+007F, C1
                                            // U+0080–U+009F) + U+2028/U+2029 BEFORE the
                                            // 50-codepoint cap (BC-2.16.002 catalog row 91 spec
                                            // order: sanitize → truncate) to prevent log injection
                                            // from adversarial sensor enum-label values.
                                            value = %prism_core::sanitize_for_log(&s).chars().take(50).collect::<String>(),
                                            sensor_type = %prism_core::sanitize_for_log(sensor_id).chars().take(50).collect::<String>(),
                                            "build_column_array: OCSF enum-label value not \
                                             recognized; emitting as-received \
                                             (BC-2.02.013 F-CRIT-002)"
                                        );
                                        Some(s)
                                    }
                                }
                            } else {
                                Some(s)
                            }
                        }
                        serde_json::Value::Array(arr) => {
                            // Wildcard result: serialize to compact JSON-list string.
                            // ENRICH-1 Design Decision 2: JSON-list string in string column.
                            let strings: Vec<String> = arr
                                .into_iter()
                                .map(|v| match v {
                                    serde_json::Value::String(s) => s,
                                    other => other.to_string(),
                                })
                                .collect();
                            Some(
                                serde_json::to_string(&strings)
                                    .unwrap_or_else(|_| "[]".to_string()),
                            )
                        }
                        other => Some(other.to_string()),
                    }
                })
                .collect();
            Arc::new(StringArray::from(
                vals.iter().map(|s| s.as_deref()).collect::<Vec<_>>(),
            ))
        }
        // Json / future variants → Utf8 (no OCSF enum-label normalization; Json values
        // are serialized as their compact string representation).
        // Wildcard source_path (`[*]`) arrays are serialized to a compact JSON-list string.
        _ => {
            let vals: Vec<Option<String>> = records
                .iter()
                .map(|r| {
                    let raw = extract_raw(r, col)?;
                    match raw {
                        serde_json::Value::Null => None,
                        serde_json::Value::String(s) => Some(s),
                        serde_json::Value::Array(arr) => {
                            // Wildcard result: serialize to compact JSON-list string.
                            // ENRICH-1 Design Decision 2: JSON-list string in string column.
                            let strings: Vec<String> = arr
                                .into_iter()
                                .map(|v| match v {
                                    serde_json::Value::String(s) => s,
                                    other => other.to_string(),
                                })
                                .collect();
                            Some(
                                serde_json::to_string(&strings)
                                    .unwrap_or_else(|_| "[]".to_string()),
                            )
                        }
                        other => Some(other.to_string()),
                    }
                })
                .collect();
            Arc::new(StringArray::from(
                vals.iter().map(|s| s.as_deref()).collect::<Vec<_>>(),
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// build_http_client_with_timeout — production reqwest::Client factory
// ---------------------------------------------------------------------------

/// Construct a `reqwest::Client` with a caller-supplied timeout `Duration`.
///
/// This is the underlying implementation used by `build_http_client_with_timeout`.
/// Exposed as `pub(crate)` so that in-crate tests can inject a short timeout (e.g. 1 ms)
/// to validate `reqwest::Client::builder()` construction without waiting up to the full
/// 30-second production timeout under load (AC-004/005; S-PERF-GATE-001).
///
/// # Visibility constraint (Architecture Compliance Rule 2 / F-MED-1)
///
/// `pub(crate)` — NOT `pub`. The function must NOT expand the public API surface of
/// `prism-bin`. The construction test `test_BC_2_01_013_build_http_client_with_custom_timeout_accepts_duration`
/// lives in the in-crate `#[cfg(test)] mod tests` block below and accesses this via
/// `pub(crate)`. Cross-crate integration tests use only `build_http_client_with_timeout()`
/// (the `pub` 30-second wrapper).
///
/// # Production use
///
/// Production callers MUST use `build_http_client_with_timeout()` (the 30-second variant).
/// This function is crate-internal; do NOT call it from production paths.
///
/// # Timeout contract
///
/// The constructed client has `.timeout(timeout)` set. A timeout of `Duration::ZERO` is
/// silently accepted by reqwest's builder but will cause every request to fail immediately —
/// use only in construction-only tests that never issue an HTTP request.
///
/// Returns `Err(String)` if the client builder fails (should not happen in practice;
/// failure mode is malformed TLS configuration).
///
/// TD-S-PLUGIN-PREREQ-B-005; AC-004; AC-005; S-PERF-GATE-001.
pub(crate) fn build_http_client_with_custom_timeout(
    timeout: Duration,
) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        // ADR-050 §D6: all sensor/plugin outbound clients MUST set User-Agent.
        // concat! produces a &'static str with zero allocation at runtime.
        .user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))
        .timeout(timeout)
        .build()
        .map_err(|e| {
            format!(
                "failed to build reqwest::Client with timeout {:?}: {e}",
                timeout
            )
        })
}

/// Construct a `reqwest::Client` with a 30-second timeout.
///
/// MUST be used by `step9a_populate_adapter_registry` when constructing `SpecDrivenSensorAdapter`
/// instances. Using `reqwest::Client::new()` without a timeout is a P2 finding per
/// CLAUDE.md conventions (TD-S-PLUGIN-PREREQ-B-005).
///
/// Thin wrapper around `build_http_client_with_custom_timeout(Duration::from_secs(30))`.
/// The 30-second production timeout is unchanged.
///
/// Returns `Err(String)` if the client builder fails (should not happen in production;
/// failure mode is malformed TLS configuration).
pub fn build_http_client_with_timeout() -> Result<reqwest::Client, String> {
    build_http_client_with_custom_timeout(Duration::from_secs(30))
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
/// - `CookieRoundtrip`: constructs `StaticCookieAuthProvider::new(sensor_id, org_registry, keyring)` →
///   `AdapterAuthStrategy::StaticCookie(Arc::new(...))`.
/// - Other `auth_type` values: logs E-SPEC-012 (auth type mismatch for S-DEMO-001 scope) and skips.
///
/// # OrgSlug → OrgId translation
///
/// `resolved_spec_map` is keyed by `(OrgSlug, SensorId)`. `AdapterRegistry` is keyed by
/// `(OrgId, SensorId)`. This function calls `org_registry.resolve(slug)` to translate.
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
pub async fn step9a_populate_adapter_registry(
    resolved_spec_map: &HashMap<ResolvedSpecKey, ResolvedSensorSpec>,
    org_registry: Arc<prism_core::OrgRegistry>,
    plugin_auth_providers: &HashMap<String, Arc<PluginAuthProvider>>,
    adapter_registry: &mut prism_sensors::AdapterRegistry,
    credential_store_org_id: Arc<dyn prism_credentials::CredentialStoreOrgId>,
) -> Result<usize, crate::boot::BootError> {
    // AC-006: empty spec_catalog → 0 registrations, no error.
    if resolved_spec_map.is_empty() {
        tracing::info!(
            target: "boot",
            event_type = "boot.step9a.adapter_registry_populated",
            sensor_count = 0u64,
            org_count = 0u64,
            "boot step 9A: spec catalog is empty — 0 adapters registered",
        );
        return Ok(0);
    }

    // Build HTTP client with 30s timeout (TD-S-PLUGIN-PREREQ-B-005).
    // Shared across all adapters constructed in this boot step.
    let http_client = build_http_client_with_timeout().map_err(|e| {
        crate::boot::BootError::InternalError(format!(
            "boot step 9A: failed to build HTTP client: {e}"
        ))
    })?;

    let mut registered_count: usize = 0;
    // Track unique orgs that had at least one adapter registered (for org_count metric).
    let mut orgs_with_adapters: std::collections::HashSet<OrgId> = std::collections::HashSet::new();

    for ((org_slug, _sensor_id_key), resolved_spec) in resolved_spec_map {
        // OQ-2 Resolution: translate OrgSlug → OrgId via OrgRegistry::resolve().
        // resolve() is the canonical method per story v1.4 (D-922).
        let org_id = match org_registry.resolve(org_slug) {
            Some(id) => id,
            None => {
                // Slug has no matching OrgId — skip with warning (OQ-2 Resolution §skip behavior).
                // No event_type= field: this is an internal diagnostic, not an auditable event.
                // SAP-1: event_type= requires a BC-2.16.002 catalog row.
                tracing::warn!(
                    target: "boot",
                    org_slug = %org_slug.as_str(),
                    sensor_id = %resolved_spec.spec.sensor_id,
                    "boot step 9A: OrgSlug has no matching OrgId in OrgRegistry — \
                     adapter NOT registered; boot continues. Ensure step 3 cross-validation \
                     ran before step 9A. OQ-2 Resolution.",
                );
                continue;
            }
        };

        // Select auth strategy based on sensor spec's auth_type.
        let auth_strategy = match &resolved_spec.spec.auth_type {
            AuthType::CustomViaPlugin => {
                // custom_via_plugin auth_type: look up the PluginAuthProvider constructed at step 7.5b.
                // If not found (auth_plugin declared but provider not constructed), skip with warning (EC-004).
                let sensor_id_str = resolved_spec.spec.sensor_id.as_str();
                match plugin_auth_providers.get(sensor_id_str) {
                    Some(provider) => {
                        AdapterAuthStrategy::Plugin(Arc::clone(provider) as Arc<dyn AuthProvider>)
                    }
                    None => {
                        // No event_type= field: internal diagnostic, not an auditable event.
                        // SAP-1: event_type= requires a BC-2.16.002 catalog row.
                        tracing::warn!(
                            target: "boot",
                            sensor_id = %sensor_id_str,
                            org_slug = %org_slug.as_str(),
                            "boot step 9A: CustomViaPlugin sensor has no PluginAuthProvider \
                             (not constructed at step 7.5b). Adapter NOT registered. \
                             EC-004: step 7.5b failure skips step 9A for this sensor.",
                        );
                        continue;
                    }
                }
            }
            AuthType::Oauth2ClientCredentials => {
                // CrowdStrike: auth_type = "oauth2_client_credentials" + auth_plugin = "crowdstrike-oauth2"
                // (D-747 LOCKED). The plugin implements the OAuth2 client-credentials token fetch.
                //
                // CRITICAL: the global `plugin_auth_providers` map (from step 7.5b) contains a
                // `PluginAuthProvider` constructed with the TYPE spec's `base_url` (e.g.,
                // "https://api.crowdstrike.com"), not the per-org overlay URL (e.g., the DTU URL
                // "http://127.0.0.1:<port>"). At query time, `PluginAuthProvider::acquire_token`
                // posts `token_endpoint = base_url + "/oauth2/token"` — if we used the TYPE spec URL,
                // the WASM plugin would POST to the real CrowdStrike API instead of the DTU.
                //
                // Fix: construct a PER-ORG `PluginAuthProvider` at step 9A using the RESOLVED
                // base_url from `resolved_spec.spec.base_url` (which has the per-org overlay applied).
                // This ensures the OAuth2 token request goes to the correct per-org endpoint
                // (DTU clone in E2E tests; real CrowdStrike API in production).
                //
                // The global `plugin_auth_providers` entry is used to: (a) verify the plugin is
                // registered, and (b) borrow the Arc<PluginRuntime> for the new per-org provider.
                let sensor_id_str = resolved_spec.spec.sensor_id.as_str();
                let global_provider = match plugin_auth_providers.get(sensor_id_str) {
                    Some(p) => p,
                    None => {
                        // auth_plugin was declared but provider not constructed at step 7.5b.
                        tracing::warn!(
                            target: "boot",
                            sensor_id = %sensor_id_str,
                            org_slug = %org_slug.as_str(),
                            "boot step 9A: Oauth2ClientCredentials sensor has no PluginAuthProvider \
                             (not constructed at step 7.5b — check plugin was staged and auth_plugin \
                             is set in the sensor spec). Adapter NOT registered; boot continues.",
                        );
                        continue;
                    }
                };
                // Build the per-org token endpoint from the RESOLVED (overlay) base_url.
                // This is the key correction: the TYPE spec has base_url = "https://api.crowdstrike.com"
                // but the overlay sets base_url = "http://127.0.0.1:<port>" (or the real per-org URL).
                let per_org_token_endpoint =
                    format!("{}/oauth2/token", resolved_spec.spec.base_url);
                // Construct a per-org PluginAuthProvider with the correct token endpoint.
                let per_org_provider = prism_spec_engine::PluginAuthProvider::new(
                    global_provider.runtime_arc(),
                    global_provider.plugin_id().to_string(),
                    sensor_id_str.to_string(),
                    per_org_token_endpoint,
                    Arc::clone(&org_registry),
                    Arc::clone(&credential_store_org_id),
                );
                AdapterAuthStrategy::Plugin(Arc::new(per_org_provider) as Arc<dyn AuthProvider>)
            }
            AuthType::BearerStatic => {
                // Armis/Claroty: resolve bearer token from credential store at acquire_token() time.
                //
                // ADV-SDEMO002-P01-CRIT-001 fix: replaced bare `AdapterAuthStrategy::BearerStatic`
                // (which extracted token from SensorAuth arg — requiring ProductionCredentialResolver
                // to fabricate a "dtu-e2e-bearer-placeholder" token) with
                // `BearerStaticCredentialAuthProvider` which resolves the real token from the
                // credential store (per-client env var `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_BEARER_TOKEN`,
                // e.g. PRISM_CLIENTS_DEMO_ORG_A_SENSORS_ARMIS_BEARER_TOKEN for org_slug demo-org-a).
                //
                // FAIL-CLOSED: if no credential is configured, acquire_token() returns
                // Err(AuthAcquisitionFailed) with E-AUTH-005. No fabricated token ever reaches
                // a real API. BC-2.06.003 resolution chain; AD-017 credential safety.
                //
                // credential_ref_name = "bearer_token" (canonical per architect decision, D-939):
                // env var: PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_BEARER_TOKEN (ADR-032 / BC-2.06.003).
                let provider = BearerStaticCredentialAuthProvider::new(
                    resolved_spec.spec.sensor_id.as_str(),
                    "bearer_token",
                    Arc::clone(&org_registry),
                    Arc::clone(&credential_store_org_id),
                );
                AdapterAuthStrategy::Plugin(std::sync::Arc::new(provider)
                    as std::sync::Arc<dyn prism_spec_engine::AuthProvider>)
            }
            AuthType::CookieRoundtrip => {
                // Cyberint: StaticCookieAuthProvider reads API key from credential store
                // at acquire_token() time with NO HTTP call (ADR-031 §D1-b).
                let provider = prism_spec_engine::StaticCookieAuthProvider::new(
                    resolved_spec.spec.sensor_id.as_str(),
                    Arc::clone(&org_registry),
                    Arc::clone(&credential_store_org_id),
                );
                AdapterAuthStrategy::StaticCookie(Arc::new(provider) as Arc<dyn AuthProvider>)
            }
            other => {
                // EC-007: unsupported auth_type — log E-SPEC-012 and skip.
                // No event_type= field: internal diagnostic, not an auditable event.
                // SAP-1: event_type= requires a BC-2.16.002 catalog row.
                tracing::warn!(
                    target: "boot",
                    sensor_id = %resolved_spec.spec.sensor_id,
                    org_slug = %org_slug.as_str(),
                    auth_type = ?other,
                    "boot step 9A: E-SPEC-012 — unsupported auth_type. \
                     Adapter NOT registered; boot continues. \
                     Supported types: Oauth2ClientCredentials (with auth_plugin), \
                     CustomViaPlugin, BearerStatic, CookieRoundtrip. \
                     EC-007: S-DEMO-001 scope boundary.",
                );
                continue;
            }
        };

        // Construct the SpecDrivenSensorAdapter.
        let adapter = SpecDrivenSensorAdapter::new(
            Arc::new(resolved_spec.clone()),
            auth_strategy,
            http_client.clone(),
        );

        // Register in AdapterRegistry keyed by (OrgId, SensorId) — SensorId from adapter.sensor_type().
        adapter_registry.register(org_id, Arc::new(adapter));
        orgs_with_adapters.insert(org_id);
        registered_count += 1;
    }

    // SAP-1 obligation: emit structured event with sensor_count and org_count fields.
    // BC-2.16.002 catalog row: boot.step9a.adapter_registry_populated.
    let org_count = orgs_with_adapters.len();
    tracing::info!(
        target: "boot",
        event_type = "boot.step9a.adapter_registry_populated",
        sensor_count = registered_count as u64,
        org_count = org_count as u64,
        "boot step 9A complete: adapter registry populated with spec-driven adapters",
    );

    Ok(registered_count)
}

// boot_step9_build_adapter_registry DELETED (F-002-R):
// This duplicate helper was removed per adversary pass-2 finding F-002-R.
// The production wiring is tested by test_BC_2_22_001_production_boot_path_wiring_guard
// which reads boot.rs source directly to verify step9_start_mcp_server calls
// step9a_populate_adapter_registry. That structural guard is stronger than testing
// a parallel helper function that bypasses the real production wiring.
// BC-2.22.001; F-002-R; S-DEMO-001 v1.5.

// ---------------------------------------------------------------------------
// CrowdStrike FQL time-window builder (ADR-033 T1 + BC-2.01.013)
// ---------------------------------------------------------------------------

/// Build a CrowdStrike FQL filter string from optional time bounds.
///
/// Implements BC-2.01.013 Pagination/Push-Down Scope Clause — CrowdStrike row:
/// - `start_time` → `created_timestamp:>'<ISO8601>'`
/// - `end_time`   → `created_timestamp:<'<ISO8601>'`
/// - Both present → combined with `+` (CrowdStrike FQL AND operator)
/// - Neither present → returns empty string (no filter)
///
/// The returned string is seeded into `FetchContext.query_filters["_fql"]` and
/// interpolated into the CrowdStrike Step 1 path_template via `${query.filter._fql}`.
/// An empty return value produces `?filter=` (ignored by the DTU for empty-string filter).
fn build_crowdstrike_fql(start_time: Option<&str>, end_time: Option<&str>) -> String {
    let start_clause = start_time.map(|t| format!("created_timestamp:>'{t}'"));
    let end_clause = end_time.map(|t| format!("created_timestamp:<'{t}'"));
    match (start_clause, end_clause) {
        (Some(start), Some(end)) => format!("{start}+{end}"),
        (Some(start), None) => start,
        (None, Some(end)) => end,
        (None, None) => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Datetime parsing helper — OCSF normalization boundary (ADR-052 D5)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Unit tests — BearerStaticCredentialAuthProvider fail-closed contract
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use prism_core::ColumnType;
    use prism_core::OrgSlug;
    use prism_spec_engine::AuthProvider;
    use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec};

    use super::{
        BearerStaticCredentialAuthProvider, build_http_client_with_custom_timeout,
        column_type_to_arrow, parse_datetime_to_micros,
    };

    /// Build a minimal SensorSpec for bearer_static sensors (Armis fixture).
    fn bearer_static_spec() -> SensorSpec {
        SensorSpec::new(
            "armis",
            "Armis Test Sensor",
            AuthType::BearerStatic,
            "https://mock.invalid",
            vec![TableSpec::new_point_in_time(
                "devices",
                "device",
                vec![ColumnSpec::new(
                    "device_id",
                    ColumnType::String,
                    None,
                    vec![],
                )],
                vec![FetchStep::new(
                    "fetch_devices",
                    "GET",
                    "/api/v1/devices",
                    None,
                    "$.data",
                    None,
                    vec![],
                    None,
                    None,
                )],
            )],
            None,
            "1.0.0",
            vec![],
        )
    }

    /// ADV-SDEMO002-P01-CRIT-001 — fail-closed contract:
    /// `BearerStaticCredentialAuthProvider::acquire_token` with a missing credential
    /// (injected via `NotFoundCredentialResolver`) MUST return `Err(AuthAcquisitionFailed)`
    /// with an E-AUTH-005 detail and MUST NOT return Ok (no fallback token).
    ///
    /// This test is the load-bearing assertion for the fix — if this test passes,
    /// the provider is fail-closed. If it were to return `Ok(AuthToken)`, it would
    /// be fabricating a token (the defect this fix closes).
    ///
    /// SID-1 compliance: uses `NotFoundCredentialResolver` (from `prism_spec_engine`)
    /// injected via `new_with_resolver` — no env var mutation, no real credential store.
    ///
    /// ADV-SDEMO002-P01-CRIT-001; BC-2.06.003; AD-017.
    #[tokio::test]
    async fn test_bearer_static_credential_auth_provider_missing_credential_fails_closed() {
        let provider = BearerStaticCredentialAuthProvider::new_with_resolver(
            "armis",
            "bearer_token",
            Arc::new(prism_spec_engine::auth_provider::NotFoundCredentialResolver),
        );
        let spec = bearer_static_spec();
        let client_id = OrgSlug::new("test-org");

        let result = provider.acquire_token(&spec, &client_id).await;

        assert!(
            result.is_err(),
            "ADV-SDEMO002-P01-CRIT-001: acquire_token MUST return Err when no bearer_token \
             credential is configured — fail-closed, no fabricated token. Got Ok."
        );
        let err_str = result.unwrap_err().to_string();
        assert!(
            err_str.contains("E-AUTH-005"),
            "ADV-SDEMO002-P01-CRIT-001: missing bearer_token MUST produce E-AUTH-005. \
             Got: {err_str}"
        );
    }

    // ---------------------------------------------------------------------------
    // build_column_array source_path unit tests (F-ENRICH-P1-MED-001)
    // ---------------------------------------------------------------------------
    //
    // These tests drive `build_column_array` (a pure fn over &[serde_json::Value] + &ColumnSpec)
    // in-process with no external/DTU dependency. They are load-bearing (TD-VSDD-059) per SID-1:
    // the E2E tests covering the same paths are `#[ignore]`'d due to DTU dependency;
    // these unit tests ensure the behavior is verified without external services.
    //
    // Imports scoped here to avoid polluting the module with test-only pub-use.
    use super::build_column_array;
    use arrow::array::{Array, Int64Array, StringArray as ArrowStringArray};
    use serde_json::json;

    /// Helper: construct a `ColumnSpec` with a `source_path` set.
    ///
    /// `ColumnSpec` is `#[non_exhaustive]` from the defining crate (`prism-spec-engine`).
    /// External crates (including `prism-bin` tests) cannot use struct literal or update
    /// syntax (`..Default::default()`) directly — E0639 applies to both forms.
    /// The correct approach: use `ColumnSpec::new()` (the provided constructor), then
    /// mutate the `source_path` field on the owned value (field mutation is allowed;
    /// only literal/update construction is gated by `#[non_exhaustive]`).
    ///
    /// CLAUDE.md non-exhaustive discipline: external callers MUST use the provided
    /// constructors for forward-compatible construction.
    fn col_with_source_path(
        name: &str,
        col_type: prism_core::ColumnType,
        source_path: &str,
    ) -> ColumnSpec {
        let mut col = ColumnSpec::new(name, col_type, None, vec![]);
        col.source_path = Some(source_path.to_string());
        col
    }

    /// F-ENRICH-P1-MED-001 (load-bearing test, SID-1):
    ///
    /// `build_column_array` with a wildcard `source_path = "$.iocs[*].value"` on a
    /// `ColumnType::String` column over records containing `{"iocs":[{"value":"hash1"},{"value":"hash2"}]}`
    /// MUST produce a non-null cell with the compact JSON-list string `["hash1","hash2"]`.
    ///
    /// This is the core ENRICH-1 behavior that DD-5 item 9 mandated be covered by an in-process test.
    /// The `column_source_path_extraction_failed` tracing emission path (AUDIT-003) is NOT triggered
    /// on success — the emission is covered by the existing `#[ignore]`'d E2E path tests.
    #[test]
    fn test_build_column_array_wildcard_source_path_string_column_non_null() {
        let records = vec![
            json!({"iocs": [{"value": "hash1"}, {"value": "hash2"}]}),
            json!({"iocs": [{"value": "hash3"}]}),
        ];
        let col = col_with_source_path(
            "ioc_values",
            prism_core::ColumnType::String,
            "$.iocs[*].value",
        );

        let array = build_column_array(&records, &col, "test-sensor");
        let string_array = array
            .as_any()
            .downcast_ref::<ArrowStringArray>()
            .expect("expected StringArray");

        // Row 0: ["hash1","hash2"] — non-null, compact JSON-list string.
        assert!(
            !string_array.is_null(0),
            "F-ENRICH-P1-MED-001: wildcard extraction must produce non-null cell (AUDIT-003)"
        );
        assert_eq!(
            string_array.value(0),
            r#"["hash1","hash2"]"#,
            "F-ENRICH-P1-MED-001: wildcard must produce compact JSON-list string"
        );

        // Row 1: ["hash3"] — single-element list.
        assert!(!string_array.is_null(1));
        assert_eq!(string_array.value(1), r#"["hash3"]"#);
    }

    /// F-ENRICH-P1-MED-001 (load-bearing test, null path, SID-1):
    ///
    /// `build_column_array` with a `source_path` pointing to a field ABSENT from the record
    /// MUST produce a null cell. The `column_source_path_extraction_failed` tracing emission
    /// path is exercised by the `extract_at_path` `Err` branch in `extract_raw` — we assert
    /// the null cell outcome here. The tracing emission itself is a side-effect that cannot
    /// be easily asserted in a unit test; it is covered by the existing E2E path tests.
    #[test]
    fn test_build_column_array_wildcard_source_path_missing_field_yields_null() {
        let records = vec![
            json!({"other_field": "value"}), // no "iocs" key
        ];
        let col = col_with_source_path(
            "ioc_values",
            prism_core::ColumnType::String,
            "$.iocs[*].value",
        );

        let array = build_column_array(&records, &col, "test-sensor");
        let string_array = array
            .as_any()
            .downcast_ref::<ArrowStringArray>()
            .expect("expected StringArray");

        // Missing path → extract_at_path returns Err → extract_raw returns None → null cell.
        assert!(
            string_array.is_null(0),
            "F-ENRICH-P1-MED-001: missing source_path field must produce null cell"
        );
    }

    // ---------------------------------------------------------------------------
    // build_column_array first-element-with-warn for numeric wildcard (F-ENRICH-P1-LOW-001)
    // ---------------------------------------------------------------------------

    /// F-ENRICH-P1-LOW-001 (RED GATE — TDD: fails before fix, passes after):
    ///
    /// `build_column_array` with `ColumnType::Integer` column and wildcard `source_path`
    /// (e.g., `"$.arr[*].value"`) over a record yielding an Array from `extract_raw`
    /// MUST return the FIRST element's integer value (42), NOT null.
    ///
    /// Current behavior (before fix): silent-null (the `.and_then(|v| v.as_i64())` call
    /// on `Value::Array` returns None). Expected behavior per DD-5 item 3: first-element
    /// extraction with `tracing::warn!` (no `event_type=` field — plain diagnostic warn).
    ///
    /// This test is RED against the current code. After the fix it must be GREEN.
    #[test]
    fn test_build_column_array_numeric_wildcard_uses_first_element() {
        let records = vec![json!({"arr": [{"value": 42}, {"value": 99}]})];
        let col = col_with_source_path(
            "first_val",
            prism_core::ColumnType::Integer,
            "$.arr[*].value",
        );

        let array = build_column_array(&records, &col, "test-sensor");
        let int_array = array
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("expected Int64Array");

        // DD-5 item 3: first element (42), not null.
        assert!(
            !int_array.is_null(0),
            "F-ENRICH-P1-LOW-001: numeric wildcard must yield first element (42), not null \
             (DD-5 item 3: first-element-with-warn)"
        );
        assert_eq!(
            int_array.value(0),
            42,
            "F-ENRICH-P1-LOW-001: first element must be 42"
        );
    }

    /// `BearerStaticCredentialAuthProvider::acquire_token` MUST return `Ok(AuthToken)`
    /// when the credential is present (injected via `MockCredentialResolver`).
    ///
    /// Verifies the happy path: the resolved token is returned wrapped in `AuthToken`.
    ///
    /// SID-1 compliance: uses `MockCredentialResolver` injected via `new_with_resolver` —
    /// no env var mutation, no real credential store. ADR-022 §C; ADV-SDEMO002-P01-CRIT-001.
    #[tokio::test]
    async fn test_bearer_static_credential_auth_provider_resolves_token_from_credentials() {
        let expected_token = "test-bearer-token-abc123";
        let provider = BearerStaticCredentialAuthProvider::new_with_resolver(
            "armis",
            "bearer_token",
            Arc::new(prism_spec_engine::auth_provider::MockCredentialResolver::new(expected_token)),
        );
        let spec = bearer_static_spec();
        let client_id = OrgSlug::new("test-org");

        let result = provider.acquire_token(&spec, &client_id).await;

        assert!(
            result.is_ok(),
            "acquire_token MUST return Ok(AuthToken) when credential is present. \
             Got: {:?}",
            result
        );
        assert_eq!(
            result.unwrap().as_str(),
            expected_token,
            "AuthToken value must equal the injected resolved credential value"
        );
    }

    /// RG-PERF-001 — `build_http_client_with_custom_timeout` accepts any `Duration`.
    ///
    /// Validates that `reqwest::Client::builder()` construction succeeds regardless of
    /// the timeout value. Uses `Duration::from_millis(1)` so the test completes in
    /// sub-millisecond time — the builder does NOT block for the configured timeout
    /// duration; the timeout only affects subsequent HTTP requests, not construction.
    ///
    /// # Visibility note (F-MED-1 / Architecture Compliance Rule 2)
    ///
    /// `build_http_client_with_custom_timeout` is `pub(crate)` — NOT `pub`. This
    /// in-crate test exercises it via `super::build_http_client_with_custom_timeout`
    /// without requiring any public API surface expansion. Cross-crate callers use
    /// `build_http_client_with_timeout()` (the `pub` 30-second wrapper) exclusively.
    ///
    /// # Error type note
    ///
    /// Returns `Result<reqwest::Client, String>` (NOT `reqwest::Error`). The `String`
    /// error type is the correct signature — `reqwest::Client::builder().build()` returns
    /// `Result<Client, reqwest::Error>`, and `build_http_client_with_custom_timeout`
    /// maps that to `String` via `.map_err(|e| format!(...))`.
    ///
    /// TD-S-PLUGIN-PREREQ-B-005; AC-004; AC-005; S-PERF-GATE-001 F-MED-1.
    #[test]
    fn test_BC_2_01_013_build_http_client_with_custom_timeout_accepts_duration() {
        let result = build_http_client_with_custom_timeout(Duration::from_millis(1));
        assert!(
            result.is_ok(),
            "build_http_client_with_custom_timeout(1ms) must return Ok(Client) — \
             reqwest client construction must succeed regardless of timeout value. \
             Got Err: {:?}",
            result.err()
        );
    }

    // -----------------------------------------------------------------------
    // RG-001 / RG-008 — S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 Red Gate tests
    // -----------------------------------------------------------------------

    /// RG-001: `column_type_to_arrow(ColumnType::Datetime)` must register as
    /// `DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))`.
    ///
    /// # Red Gate pre-implementation failure
    /// Returns `DataType::Utf8` — `assert_eq!` FAILS with:
    ///   left:  `Utf8`
    ///   right: `Timestamp(Microsecond, Some("UTC"))`
    ///
    /// # Why load-bearing
    /// If the Datetime arm remains `DataType::Utf8`, all temporal comparisons are
    /// lexicographic. Every downstream test (RG-009, RG-010) depends on this being
    /// `Timestamp(Microsecond, UTC)`. This is the foundation gate for the migration.
    ///
    /// # Arc form discipline
    /// `Some(Arc::from("UTC"))` produces `Arc<str>` — the correct Arrow API form.
    /// `Some(Arc::new("UTC".into()))` produces `Arc<String>` and is FORBIDDEN
    /// (ADR-052 §D1 canonical form).
    ///
    /// Traces to: BC-2.11.003 §Postconditions; ADR-052 §D1/§D2.
    #[test]
    #[allow(clippy::expect_used)]
    fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_datetime_column_registers_as_timestamp_micros_utc()
    {
        use arrow::datatypes::TimeUnit;

        let result = column_type_to_arrow(&ColumnType::Datetime);
        assert_eq!(
            result,
            arrow::datatypes::DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC"))),
            "RG-001: column_type_to_arrow(ColumnType::Datetime) must return \
             DataType::Timestamp(Microsecond, Some(\"UTC\")) per ADR-052 D1/D2. \
             Currently returns DataType::Utf8 — assertion FAILS until the Datetime arm \
             is updated in column_type_to_arrow (Task 8 of S-PRISMQL-NATIVE-TEMPORAL-TYPING-001)."
        );
    }

    /// RG-008: `parse_datetime_to_micros("2026-07-03T00:00:00Z")` returns the correct
    /// `i64` microseconds-since-epoch value.
    ///
    /// # Red Gate pre-implementation failure
    /// `parse_datetime_to_micros` body is `todo!()` — panics with "not yet implemented"
    /// before the `.expect(...)` assertion is reached.
    ///
    /// # Why load-bearing
    /// Arrow `Timestamp(Microsecond, Some("UTC"))` columns store `i64` micros since epoch.
    /// If sensor datetime strings are stored as-is (Utf8 bytes) into a Timestamp schema
    /// column, Arrow produces null or panics at materialization time — silent data loss.
    ///
    /// # TD-VSDD-091 compliance
    /// The expected value is DERIVED at test time via chrono — NOT a hardcoded magic
    /// constant. If the epoch representation changes (e.g., leap second handling), the
    /// derivation remains correct.
    ///
    /// Traces to: ADR-052 §D5; BC-2.11.003 §Postconditions D4/D5
    /// chrono-strictness invariant.
    #[test]
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_sensor_datetime_string_parsed_to_micros() {
        // Derive the expected microsecond value via chrono at test time (TD-VSDD-091:
        // behavioral anchors, not magic constants).
        let expected_micros = chrono::DateTime::parse_from_rfc3339("2026-07-03T00:00:00Z")
            .expect("known-good RFC-3339 literal must parse cleanly")
            .timestamp_micros();

        // Red Gate: parse_datetime_to_micros is todo!() — panics here before the
        // assertion below is reached. Failure mode: "not yet implemented: ..."
        let micros = parse_datetime_to_micros("2026-07-03T00:00:00Z", "timestamp", "test-sensor")
            .expect("parse_datetime_to_micros must return Ok(i64) for valid RFC-3339 input");

        assert_eq!(
            micros, expected_micros,
            "RG-008: parse_datetime_to_micros(\"2026-07-03T00:00:00Z\") must return \
             Ok({expected_micros}) (microseconds since Unix epoch via chrono derivation). \
             ADR-052 D5 — sensor ISO-8601 strings must be converted to i64 µs-since-epoch."
        );
    }

    /// SEC-002 (CWE-532 / AD-017): `parse_datetime_to_micros` with a >50-codepoint invalid
    /// value must produce a `TimestampParseFailure` whose `value` field is capped at exactly
    /// 50 codepoints.
    ///
    /// Prevents unbounded raw sensor data from flowing into operator logs via the error
    /// Display (`value='{value}'`). Consistent with E-QUERY-041/042 value_prefix convention.
    #[test]
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    fn test_sec_002_parse_datetime_to_micros_caps_value_at_50_codepoints() {
        use prism_spec_engine::error::SpecEngineError;
        // 60-character invalid timestamp string — exceeds the 50-codepoint cap.
        let long_value = "INVALID_TIMESTAMP_STRING_THAT_IS_DEFINITELY_MORE_THAN_FIFTY";
        assert!(
            long_value.chars().count() > 50,
            "test precondition: value must be >50 chars; got {}",
            long_value.chars().count()
        );
        let result = parse_datetime_to_micros(long_value, "created_at", "test-sensor");
        assert!(result.is_err(), "invalid timestamp must produce Err");
        match result.unwrap_err() {
            SpecEngineError::TimestampParseFailure { value, .. } => {
                assert_eq!(
                    value.chars().count(),
                    50,
                    "SEC-002 (AD-017): TimestampParseFailure.value must be capped at \
                     50 codepoints; got {} codepoints in {value:?}",
                    value.chars().count()
                );
            }
            other => panic!("expected TimestampParseFailure, got: {other:?}"),
        }
    }

    // ---------------------------------------------------------------------------
    // BC-2.02.013 / F-CRIT-002: OCSF enum-label normalization in
    // build_column_array (production spec-driven Arrow path)
    // ---------------------------------------------------------------------------
    //
    // F-CRIT-002 insertion point (architect-ratified 2026-07-07):
    //   `build_column_array` must call `OcsfEnumMap::normalize_enum_label(col_name, raw)`
    //   for ColumnType::String columns where col.name ∈ {"severity","status",
    //   "activity_name","disposition"} BEFORE Arrow materialization.
    //   Unrecognized values pass through as-received + emit
    //   tracing::warn!(event_type = "ocsf.enum_label_unrecognized", ...).
    //
    // RED gate (LOCAL-pass-5 F-CRIT-002):
    //   Tests 1-3 FAIL before implementation (raw passthrough — no normalization).
    //   Tests 4-5 PASS before AND after (regression guards for column-selection rule).
    //
    // SID-1 compliance: all in-process, no external/DTU dep, no #[ignore].

    /// BC-2.02.013 / F-CRIT-002 (RED — fails before implementation):
    ///
    /// `build_column_array` for a `ColumnType::String` column named `"severity"`
    /// with raw values `"CRITICAL"`, `"high"`, `"High"` MUST produce a StringArray
    /// with OCSF canonical Title-case values `["Critical", "High", "High"]`.
    ///
    /// FAILS NOW: `build_column_array` does raw passthrough for String columns —
    /// `"CRITICAL"` remains `"CRITICAL"`, not `"Critical"`. The `assert_eq!` for row 0
    /// fires with `left="CRITICAL"`, `right="Critical"`.
    ///
    /// After implementation: `OcsfEnumMap::normalize_enum_label("severity", raw)` is
    /// called for every non-null raw string value before Arrow materialization.
    ///
    /// Traces to: BC-2.02.013 F-CRIT-002; LOCAL-pass-5 adversary finding.
    #[test]
    fn test_BC_2_02_013_build_column_array_normalizes_severity_to_title_case() {
        let records = vec![
            json!({"severity": "CRITICAL"}),
            json!({"severity": "high"}),
            json!({"severity": "High"}),
        ];
        let col = ColumnSpec::new("severity", ColumnType::String, None, vec![]);

        let array = build_column_array(&records, &col, "crowdstrike");
        let string_array = array
            .as_any()
            .downcast_ref::<ArrowStringArray>()
            .expect("expected StringArray for ColumnType::String severity column");

        // BC-2.02.013: "CRITICAL" → "Critical" via OcsfEnumMap (severity_id[5]).
        // FAILS NOW: raw passthrough returns "CRITICAL", not "Critical".
        assert_eq!(
            string_array.value(0),
            "Critical",
            "BC-2.02.013 F-CRIT-002: severity='CRITICAL' must normalize to 'Critical' \
             via OcsfEnumMap::normalize_enum_label; \
             got: {:?}. build_column_array is currently doing raw passthrough.",
            string_array.value(0)
        );
        // BC-2.02.013: "high" → "High" via OcsfEnumMap (severity_id[4]).
        // FAILS NOW: raw passthrough returns "high", not "High".
        assert_eq!(
            string_array.value(1),
            "High",
            "BC-2.02.013 F-CRIT-002: severity='high' must normalize to 'High' \
             via OcsfEnumMap::normalize_enum_label; \
             got: {:?}. build_column_array is currently doing raw passthrough.",
            string_array.value(1)
        );
        // Idempotent: already-canonical "High" stays "High".
        assert_eq!(
            string_array.value(2),
            "High",
            "BC-2.02.013 F-CRIT-002: severity='High' (already OCSF canonical) must \
             remain 'High' after normalization (idempotent); got: {:?}.",
            string_array.value(2)
        );
    }

    /// BC-2.02.013 / F-CRIT-002 (RED — fails before implementation):
    ///
    /// `build_column_array` for `ColumnType::String` columns named `"status"` and
    /// `"disposition"` must normalize raw values to OCSF canonical Title-case:
    ///   - status `"NEW"` → `"New"` (OcsfEnumMap status_id[1001])
    ///   - disposition `"blocked"` → `"Blocked"` (OcsfEnumMap disposition_id[2])
    ///
    /// FAILS NOW: raw passthrough — "NEW" stays "NEW", "blocked" stays "blocked".
    ///
    /// After implementation: `OcsfEnumMap::normalize_enum_label` is called for
    /// every String column whose name is in the in-scope field set.
    ///
    /// Traces to: BC-2.02.013 F-CRIT-002 in-scope field table
    /// (status, disposition guaranteed); LOCAL-pass-5.
    #[test]
    fn test_BC_2_02_013_build_column_array_normalizes_status_and_disposition() {
        // --- status "NEW" → "New" ---
        let status_records = vec![json!({"status": "NEW"})];
        let status_col = ColumnSpec::new("status", ColumnType::String, None, vec![]);

        let status_array = build_column_array(&status_records, &status_col, "crowdstrike");
        let status_str_array = status_array
            .as_any()
            .downcast_ref::<ArrowStringArray>()
            .expect("expected StringArray for status column");

        // FAILS NOW: raw passthrough returns "NEW", not "New".
        assert_eq!(
            status_str_array.value(0),
            "New",
            "BC-2.02.013 F-CRIT-002: status='NEW' must normalize to 'New' via \
             OcsfEnumMap::normalize_enum_label (status_id[1001]=New); \
             got: {:?}. build_column_array is currently doing raw passthrough.",
            status_str_array.value(0)
        );

        // --- disposition "blocked" → "Blocked" ---
        let disp_records = vec![json!({"disposition": "blocked"})];
        let disp_col = ColumnSpec::new("disposition", ColumnType::String, None, vec![]);

        let disp_array = build_column_array(&disp_records, &disp_col, "crowdstrike");
        let disp_str_array = disp_array
            .as_any()
            .downcast_ref::<ArrowStringArray>()
            .expect("expected StringArray for disposition column");

        // FAILS NOW: raw passthrough returns "blocked", not "Blocked".
        assert_eq!(
            disp_str_array.value(0),
            "Blocked",
            "BC-2.02.013 F-CRIT-002: disposition='blocked' must normalize to 'Blocked' via \
             OcsfEnumMap::normalize_enum_label (disposition_id[2]=Blocked); \
             got: {:?}. build_column_array is currently doing raw passthrough.",
            disp_str_array.value(0)
        );
    }

    /// BC-2.02.013 / F-CRIT-002 (RED — fails before implementation):
    ///
    /// `build_column_array` for a `ColumnType::String` column named `"severity"` with
    /// a vendor-specific unrecognized value `"VENDOR_XYZ"` MUST:
    ///   1. Leave the value as-received (`"VENDOR_XYZ"` unchanged, non-fatal).
    ///   2. Emit `tracing::warn!(event_type = "ocsf.enum_label_unrecognized", ...)`.
    ///
    /// FAILS NOW: no normalization attempt is made, so no `ocsf.enum_label_unrecognized`
    /// warn is ever emitted. The value passthrough assertion (1) would pass, but the
    /// warn-capture assertion (2) FAILS because the warn is never fired.
    ///
    /// After implementation: `OcsfEnumMap::normalize_enum_label` returns `None` for
    /// `"VENDOR_XYZ"`, the raw value is kept as-received, and the warn is emitted.
    ///
    /// WarnCapture pattern: matches `test_adapter_normalization.rs` RG-021 (prism-ocsf).
    ///
    /// Traces to: BC-2.02.013 F-CRIT-002 error case;
    /// BC-2.16.002 Canonical Structured Event Catalog (ocsf.enum_label_unrecognized);
    /// LOCAL-pass-5 adversary finding; strengthened for LOCAL-pass-6 F-P6-CRIT-001 +
    /// F-P6-HIGH-003 (catalog row 91 schema completeness).
    #[test]
    fn test_BC_2_02_013_build_column_array_unrecognized_left_as_received_with_warn() {
        use std::sync::Mutex;
        use tracing_subscriber::layer::SubscriberExt;

        // ── Local WarnCapture types — catalog-complete field capture ───────────
        //
        // Captures ALL BC-2.16.002 catalog row 91 fields for ocsf.enum_label_unrecognized:
        //   field_name, value, sensor_type.
        // Also captures the legacy `column` field to diagnose F-P6-CRIT-001 (wrong field name).
        //
        // F-P6-CRIT-001: PRIMARY emit in build_column_array uses `column = %col.name`
        //   instead of the catalog-required `field_name = %col.name`.
        // F-P6-HIGH-003: PRIMARY emit omits `sensor_type` entirely.
        // Assertions (3) and (4) below enforce both catalog row 91 requirements.
        // They FAIL at HEAD 8e4ec972: `field_name` is None (only `column` is set),
        // and `sensor_type` is None (omitted from the warn macro).

        #[derive(Default, Clone, Debug)]
        struct WarnEvent {
            event_type: Option<String>,
            /// BC-2.16.002 catalog row 91 required field (must NOT be `column`).
            field_name: Option<String>,
            /// BC-2.16.002 catalog row 91 required field (cap ≤ 50 codepoints at PRIMARY).
            value: Option<String>,
            /// BC-2.16.002 catalog row 91 required field (absent in current PRIMARY emit).
            sensor_type: Option<String>,
            /// Legacy wrong field name — captured to produce a clear diagnostic on failure.
            column: Option<String>,
        }

        #[derive(Default)]
        struct WarnFieldVisitor {
            event: WarnEvent,
        }

        impl tracing::field::Visit for WarnFieldVisitor {
            fn record_str(&mut self, field: &tracing::field::Field, val: &str) {
                // Handles string literal fields (e.g. `event_type = "ocsf.enum_label_unrecognized"`).
                match field.name() {
                    "event_type" => self.event.event_type = Some(val.to_owned()),
                    "field_name" => self.event.field_name = Some(val.to_owned()),
                    "value" => self.event.value = Some(val.to_owned()),
                    "sensor_type" => self.event.sensor_type = Some(val.to_owned()),
                    "column" => self.event.column = Some(val.to_owned()),
                    _ => {}
                }
            }
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                // Handles `%`-formatted fields (e.g. `column = %col.name`, `value = %s`).
                // tracing routes Display-formatted values through record_debug; the
                // dyn Debug impl for format_args!("{}", x) delegates to Display, so
                // `format!("{value:?}")` gives the Display representation without extra quoting.
                let s = format!("{value:?}");
                match field.name() {
                    "event_type" => {
                        if self.event.event_type.is_none() {
                            self.event.event_type = Some(s);
                        }
                    }
                    "field_name" => {
                        if self.event.field_name.is_none() {
                            self.event.field_name = Some(s);
                        }
                    }
                    "value" => {
                        if self.event.value.is_none() {
                            self.event.value = Some(s);
                        }
                    }
                    "sensor_type" => {
                        if self.event.sensor_type.is_none() {
                            self.event.sensor_type = Some(s);
                        }
                    }
                    "column" => {
                        if self.event.column.is_none() {
                            self.event.column = Some(s);
                        }
                    }
                    _ => {}
                }
            }
        }

        struct WarnCapture {
            events: Arc<Mutex<Vec<WarnEvent>>>,
        }

        impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for WarnCapture {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = WarnFieldVisitor::default();
                    event.record(&mut visitor);
                    if visitor.event.event_type.is_some() {
                        self.events.lock().unwrap().push(visitor.event);
                    }
                }
            }
        }

        // ── Test body ─────────────────────────────────────────────────────────

        let captured: Arc<Mutex<Vec<WarnEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let layer = WarnCapture {
            events: captured.clone(),
        };
        let subscriber = tracing_subscriber::registry().with(layer);

        let records = vec![json!({"severity": "VENDOR_XYZ"})];
        let col = ColumnSpec::new("severity", ColumnType::String, None, vec![]);

        let array = tracing::subscriber::with_default(subscriber, || {
            build_column_array(&records, &col, "crowdstrike")
        });

        let string_array = array
            .as_any()
            .downcast_ref::<ArrowStringArray>()
            .expect("expected StringArray for severity column");

        // (1) Unrecognized value must be left as-received (non-fatal, no panic).
        // PASSES NOW: normalization is wired; VENDOR_XYZ not in map → passthrough.
        assert_eq!(
            string_array.value(0),
            "VENDOR_XYZ",
            "BC-2.02.013 F-CRIT-002 error case: unrecognized severity='VENDOR_XYZ' \
             must be left as-received in the Arrow column (non-fatal); \
             got: {:?}",
            string_array.value(0)
        );

        let warns = captured.lock().unwrap();

        // (2) event_type = "ocsf.enum_label_unrecognized" must be emitted.
        // PASSES NOW: normalization wired → warn fires.
        assert!(
            warns
                .iter()
                .any(|e| e.event_type.as_deref() == Some("ocsf.enum_label_unrecognized")),
            "BC-2.02.013 F-CRIT-002: build_column_array must emit \
             tracing::warn!(event_type = \"ocsf.enum_label_unrecognized\", ...) \
             for unrecognized OCSF enum-label values (BC-2.16.002 catalog row 91); \
             captured events: {:?}",
            *warns
        );

        // Locate the unrecognized event for catalog-schema validation below.
        let evt = warns
            .iter()
            .find(|e| e.event_type.as_deref() == Some("ocsf.enum_label_unrecognized"))
            .expect(
                "ocsf.enum_label_unrecognized event not found \
                 (assertion 2 should have caught this)",
            );

        // (3) F-P6-CRIT-001: warn MUST use `field_name`, NOT `column`.
        // BC-2.16.002 catalog row 91 schema requires the field key `field_name`.
        // FAILS NOW: current PRIMARY emit at line ~1112 uses `column = %col.name`.
        // After fix: change `column = %col.name` → `field_name = %col.name`.
        assert_eq!(
            evt.field_name.as_deref(),
            Some("severity"),
            "F-P6-CRIT-001 (LOCAL pass-6): ocsf.enum_label_unrecognized warn must use \
             field `field_name` (not `column`) per BC-2.16.002 catalog row 91; \
             current PRIMARY emit uses `column = %%col.name`; \
             got field_name={:?}, legacy column={:?}",
            evt.field_name,
            evt.column
        );

        // (4) F-P6-HIGH-003: warn MUST include `sensor_type`.
        // BC-2.16.002 catalog row 91 schema requires `sensor_type`.
        // FAILS NOW: PRIMARY emit omits `sensor_type` entirely.
        // After fix: add `sensor_type = %sensor_id` to the warn macro in build_column_array.
        assert_eq!(
            evt.sensor_type.as_deref(),
            Some("crowdstrike"),
            "F-P6-HIGH-003 (LOCAL pass-6): ocsf.enum_label_unrecognized warn must include \
             `sensor_type` per BC-2.16.002 catalog row 91; \
             sensor_type is absent in current PRIMARY emit; \
             got: {:?}",
            evt.sensor_type
        );

        // (5) value must be the (possibly truncated) raw string.
        // PRIMARY already caps at 50 codepoints (SEC-002 pattern) — regression guard.
        assert_eq!(
            evt.value.as_deref(),
            Some("VENDOR_XYZ"),
            "warn value must carry the raw string (capped at 50 codepoints at PRIMARY); \
             got: {:?}",
            evt.value
        );
    }

    /// BC-2.02.013 / F-CRIT-002 (GREEN before AND after — regression guard):
    ///
    /// `build_column_array` for a `ColumnType::String` column named `"hostname"`
    /// (NOT in the OCSF enum-label field set {"severity","status","activity_name",
    /// "disposition"}) with value `"SERVER-01"` MUST pass the value through unchanged.
    /// No normalization, no warn.
    ///
    /// PASSES NOW and MUST continue to PASS after implementation — this guards the
    /// column-selection rule: only the four designated OCSF enum-label columns are
    /// normalized; all other String columns are untouched.
    ///
    /// Traces to: BC-2.02.013 F-CRIT-002 column-selection invariant.
    #[test]
    fn test_BC_2_02_013_build_column_array_non_enum_string_column_untouched() {
        let records = vec![json!({"hostname": "SERVER-01"})];
        let col = ColumnSpec::new("hostname", ColumnType::String, None, vec![]);

        let array = build_column_array(&records, &col, "crowdstrike");
        let string_array = array
            .as_any()
            .downcast_ref::<ArrowStringArray>()
            .expect("expected StringArray for hostname column");

        // "hostname" is NOT in the enum-label set — raw passthrough required.
        assert_eq!(
            string_array.value(0),
            "SERVER-01",
            "BC-2.02.013 F-CRIT-002 column-selection guard: non-enum-label String \
             column 'hostname' with value 'SERVER-01' must NOT be normalized \
             (column-selection rule: only severity/status/activity_name/disposition); \
             got: {:?}",
            string_array.value(0)
        );
    }

    /// BC-2.02.013 / F-CRIT-002 (GREEN before AND after — regression guard):
    ///
    /// `build_column_array` for a `ColumnType::Integer` column named `"severity_id"`
    /// with integer value `5` MUST produce an Int64Array with value `5` unchanged.
    /// OCSF enum-label normalization is ONLY applied to `ColumnType::String` columns
    /// in the designated field set; `ColumnType::Integer` columns are NEVER normalized.
    ///
    /// PASSES NOW and MUST continue to PASS after implementation — this guards against
    /// normalization accidentally touching non-String columns.
    ///
    /// Traces to: BC-2.02.013 F-CRIT-002 — normalization gated on
    /// `col.column_type == ColumnType::String` AND `col.name` in enum-label set.
    #[test]
    fn test_BC_2_02_013_build_column_array_non_string_column_untouched() {
        let records = vec![json!({"severity_id": 5})];
        let col = ColumnSpec::new("severity_id", ColumnType::Integer, None, vec![]);

        let array = build_column_array(&records, &col, "crowdstrike");
        let int_array = array
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("expected Int64Array for ColumnType::Integer severity_id column");

        // ColumnType::Integer columns are not subject to string normalization.
        assert!(
            !int_array.is_null(0),
            "BC-2.02.013 F-CRIT-002: Integer column severity_id must be non-null"
        );
        assert_eq!(
            int_array.value(0),
            5,
            "BC-2.02.013 F-CRIT-002: Integer column severity_id must remain 5 \
             (OCSF normalization does NOT touch ColumnType::Integer); got: {}",
            int_array.value(0)
        );
    }

    // ---------------------------------------------------------------------------
    // BC-2.02.013 PRIMARY path — GROUP BY de-fragmentation (F-P20-HIGH-001)
    // ---------------------------------------------------------------------------

    /// BC-2.02.013 PRIMARY path — GROUP BY de-fragmentation over mixed-vendor
    /// severity casing (F-P20-HIGH-001 closure):
    ///
    /// `build_column_array` for a `ColumnType::String` column named `"severity"` with
    /// mixed raw-sensor casing (CrowdStrike `'High'` × 3 + Armis `'HIGH'` × 2) MUST
    /// produce a normalized Arrow `StringArray` where ALL 5 values are `'High'`
    /// (OCSF canonical Title-case per `enum_map.rs` severity_id[4]).  A DataFusion
    /// GROUP BY over this normalized column MUST yield exactly ONE bucket
    /// (`'High'`, count = 5) — no fragmentation into separate `'High'` + `'HIGH'`
    /// buckets.
    ///
    /// **Why this test is necessary (F-P20-HIGH-001):** the existing single-value tests
    /// (`test_BC_2_02_013_build_column_array_normalizes_severity_to_title_case`) verify
    /// per-row normalization in isolation.  They do NOT verify that the Arrow
    /// materialization path produces deduplicating GROUP BY output — the combination
    /// "multi-row + DataFusion GROUP BY + assert one bucket" is what AC-019 requires.
    /// Adversarial pass-20 correctly identified this gap.
    ///
    /// **PRIMARY vs SECONDARY path disambiguation:**
    /// - PRIMARY (this test): `build_column_array` in `spec_driven_adapter.rs` —
    ///   the production path through which DataFusion receives and queries sensor data.
    /// - SECONDARY (RG-022 in `test_case_insensitive_operators.rs`): exercises
    ///   `OcsfNormalizer::normalize_with_mappers` + DynamicMessage, which has
    ///   ZERO production callers on the query path per BC-2.02.013 §Postconditions.
    ///
    /// **Expected result:** PASSES at current HEAD because `build_column_array` already
    /// calls `OcsfEnumMap::normalize_enum_label` for OCSF enum-label fields.  This test
    /// is a load-bearing regression guard (TD-VSDD-059) ensuring the behavior cannot
    /// be silently removed.
    ///
    /// Traces to: BC-2.02.013 §Postconditions PRIMARY insertion point; AC-019; EC-02-026;
    /// F-P20-HIGH-001; ADR-047 §Consequences "GROUP BY correct after normalization".
    #[tokio::test]
    #[allow(non_snake_case)]
    async fn test_BC_2_02_013_build_column_array_group_by_severity_cross_sensor_no_fragmentation() {
        use arrow::datatypes::{DataType, Field, Schema};
        use arrow::record_batch::RecordBatch;
        use datafusion::datasource::MemTable;
        use datafusion::execution::context::SessionContext;

        // BC-2.02.013 EC-02-026 test vectors:
        //   CrowdStrike emits Title-case 'High' (already canonical — idempotent path).
        //   Armis emits all-caps 'HIGH' (must normalize to 'High' via OcsfEnumMap).
        // After PRIMARY path normalization both vendor strings must yield 'High'.
        let records = vec![
            json!({"severity": "High"}), // CrowdStrike: canonical Title-case (idempotent)
            json!({"severity": "High"}), // CrowdStrike: canonical Title-case (idempotent)
            json!({"severity": "High"}), // CrowdStrike: canonical Title-case (idempotent)
            json!({"severity": "HIGH"}), // Armis: all-caps → OcsfEnumMap → 'High'
            json!({"severity": "HIGH"}), // Armis: all-caps → OcsfEnumMap → 'High'
        ];
        let col = ColumnSpec::new("severity", ColumnType::String, None, vec![]);

        // PRIMARY path: `build_column_array` is called by `pipeline_result_to_record_batch`
        // in the production spec-driven adapter fetch cycle.  OcsfEnumMap is accessed via
        // `prism_ocsf::shared_enum_map()` — initialized once at process start as a
        // &'static OcsfEnumMap; no I/O, no external dependency, deterministic in tests.
        // OcsfEnumMap::normalize_enum_label("severity", "HIGH") → Some("High").
        // OcsfEnumMap::normalize_enum_label("severity", "High") → Some("High") (idempotent).
        let normalized_array = build_column_array(&records, &col, "cross-sensor");

        // Build Arrow RecordBatch from the normalized column produced by the PRIMARY path.
        let schema = Arc::new(Schema::new(vec![Field::new(
            "severity",
            DataType::Utf8,
            true,
        )]));
        let batch = RecordBatch::try_new(schema.clone(), vec![normalized_array])
            .expect("BC-2.02.013 PRIMARY: RecordBatch must build from normalized severity array");

        // Register as DataFusion MemTable and execute GROUP BY — mirrors what the PrismQL
        // execution engine does after `pipeline_result_to_record_batch` returns.
        let ctx = SessionContext::new();
        let mem_table = MemTable::try_new(schema, vec![vec![batch]])
            .expect("BC-2.02.013 PRIMARY: MemTable must build");
        ctx.register_table("detections", Arc::new(mem_table))
            .expect("BC-2.02.013 PRIMARY: MemTable must register");

        let result = ctx
            .sql("SELECT severity, count(*) AS cnt FROM detections GROUP BY severity")
            .await
            .expect("BC-2.02.013 PRIMARY: GROUP BY query must plan without error")
            .collect()
            .await
            .expect("BC-2.02.013 PRIMARY: GROUP BY query must execute without error");

        let total_buckets: usize = result.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_buckets, 1,
            "BC-2.02.013 PRIMARY path (F-P20-HIGH-001 / AC-019): GROUP BY severity \
             after build_column_array normalization MUST yield exactly 1 bucket \
             (CrowdStrike 'High' × 3 + Armis 'HIGH' × 2 → all canonical 'High'); \
             got {total_buckets} bucket(s). \
             If 2 buckets: build_column_array is not applying OcsfEnumMap normalization \
             to the 'severity' column. \
             PRIMARY path: EC-02-026; ADR-047 §Consequences."
        );

        // Verify the single bucket is OCSF canonical 'High' with count = 5.
        // result[0] is the first (and only) RecordBatch; column 0 = severity, column 1 = cnt.
        let severity_col = result[0]
            .column(0)
            .as_any()
            .downcast_ref::<ArrowStringArray>()
            .expect("BC-2.02.013 PRIMARY: severity GROUP BY column must be StringArray");
        let count_col = result[0]
            .column(1)
            .as_any()
            .downcast_ref::<arrow::array::Int64Array>()
            .expect("BC-2.02.013 PRIMARY: count column must be Int64Array");

        assert_eq!(
            severity_col.value(0),
            "High",
            "BC-2.02.013 PRIMARY path: single GROUP BY bucket must be OCSF canonical \
             'High' (Title-case, enum_map.rs severity_id[4]); got {:?}",
            severity_col.value(0)
        );
        assert_eq!(
            count_col.value(0),
            5,
            "BC-2.02.013 PRIMARY path: all 5 records (3 × CrowdStrike 'High' + \
             2 × Armis 'HIGH') must consolidate into a single 'High' bucket \
             after OcsfEnumMap normalization; count is {}",
            count_col.value(0)
        );
    }

    // ---------------------------------------------------------------------------
    // OBS-3 — empty-string enum-value must NOT emit ocsf.enum_label_unrecognized
    // ---------------------------------------------------------------------------

    /// OBS-3 / BC-2.02.013 (RED — fails before implementation):
    ///
    /// `build_column_array` for a `ColumnType::String` column named `"severity"` with
    /// a record containing `"severity": ""` (empty string) MUST:
    ///   1. Materialize the empty string as-is in the Arrow column (non-null, value `""`).
    ///   2. NOT emit `tracing::warn!(event_type = "ocsf.enum_label_unrecognized", ...)`.
    ///
    /// Empty-string field values are NOT invalid OCSF enum labels — they represent
    /// missing or unset fields coming from vendor APIs that omit the field entirely.
    /// Calling `OcsfEnumMap::normalize_enum_label("severity", "")` returns `None`
    /// (empty string is not a recognized caption), which currently causes a false-positive
    /// `ocsf.enum_label_unrecognized` warn. This mirrors the SECONDARY path's guard in
    /// `prism-ocsf/src/normalizer.rs` (line ~141):
    ///   `ProtoValue::String(s) if !s.is_empty() => s,`
    /// which skips normalization for empty strings without emitting the warn.
    ///
    /// # Red Gate failure (HEAD 18c65590)
    ///
    /// The `ColumnType::String` arm in `build_column_array` calls
    /// `normalize_enum_label(&col.name, &s)` for ALL non-null string values,
    /// including `s = ""`.  The map returns `None` for `""` →
    /// `ocsf.enum_label_unrecognized` warn fires → assertion (2) FAILS.
    ///
    /// # Fix target
    ///
    /// Add `if s.is_empty() { Some(s) }` (or equivalent `!s.is_empty()` guard) before
    /// calling `normalize_enum_label` in the `ColumnType::String` arm of
    /// `build_column_array`, mirroring the SECONDARY path in `normalizer.rs`.
    ///
    /// OBS-3; BC-2.02.013; BC-2.16.002 catalog row 91.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_02_013_build_column_array_empty_string_enum_value_no_warn() {
        use std::sync::Mutex;
        use tracing_subscriber::layer::SubscriberExt;

        // ── Local WarnCapture — captures only event_type-bearing WARN events ──────
        //
        // Minimal variant of the WarnCapture in
        // test_BC_2_02_013_build_column_array_unrecognized_left_as_received_with_warn:
        // we only need to detect whether ocsf.enum_label_unrecognized fired at all;
        // we do not need to inspect catalog-schema fields.

        #[derive(Default, Clone, Debug)]
        struct WarnEvent {
            event_type: Option<String>,
        }

        #[derive(Default)]
        struct WarnFieldVisitor {
            event: WarnEvent,
        }

        impl tracing::field::Visit for WarnFieldVisitor {
            fn record_str(&mut self, field: &tracing::field::Field, val: &str) {
                if field.name() == "event_type" {
                    self.event.event_type = Some(val.to_owned());
                }
            }
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                // tracing routes `%`-formatted Display values through record_debug.
                if field.name() == "event_type" && self.event.event_type.is_none() {
                    self.event.event_type = Some(format!("{value:?}"));
                }
            }
        }

        struct WarnCapture {
            events: Arc<Mutex<Vec<WarnEvent>>>,
        }

        impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for WarnCapture {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = WarnFieldVisitor::default();
                    event.record(&mut visitor);
                    // Only capture events that carry an event_type field (SAP-1 catalog rows).
                    if visitor.event.event_type.is_some() {
                        self.events.lock().unwrap().push(visitor.event);
                    }
                }
            }
        }

        // ── Test body ──────────────────────────────────────────────────────────────

        let captured: Arc<Mutex<Vec<WarnEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let layer = WarnCapture {
            events: captured.clone(),
        };
        let subscriber = tracing_subscriber::registry().with(layer);

        let records = vec![json!({"severity": ""})];
        let col = ColumnSpec::new("severity", ColumnType::String, None, vec![]);

        let array = tracing::subscriber::with_default(subscriber, || {
            build_column_array(&records, &col, "armis")
        });

        let string_array = array
            .as_any()
            .downcast_ref::<ArrowStringArray>()
            .expect("expected StringArray for ColumnType::String severity column");

        // (1) Empty string materializes as-is — non-null empty string in Arrow.
        //
        // The None branch of normalize_enum_label returns Some(s) = Some(""), so
        // the value is already non-null at HEAD.  This assertion is a regression
        // guard: the fix MUST NOT accidentally convert empty strings to null.
        assert!(
            !string_array.is_null(0),
            "OBS-3: empty severity string must materialize as a non-null empty string \
             in the Arrow column (not NULL — it is an unset/missing value, not an absent \
             field); got null"
        );
        assert_eq!(
            string_array.value(0),
            "",
            "OBS-3: empty severity string must round-trip as empty string in Arrow; \
             got: {:?}",
            string_array.value(0)
        );

        let warns = captured.lock().unwrap();

        // (2) No ocsf.enum_label_unrecognized warn must be emitted for empty string.
        //
        // FAILS NOW (HEAD 18c65590): the `ColumnType::String` arm calls
        // `normalize_enum_label("severity", "")` → returns None → warn fires.
        //
        // After fix: add `!s.is_empty()` guard before calling normalize_enum_label,
        // mirroring normalizer.rs SECONDARY path (ProtoValue::String(s) if !s.is_empty()).
        assert!(
            !warns
                .iter()
                .any(|e| e.event_type.as_deref() == Some("ocsf.enum_label_unrecognized")),
            "OBS-3 / BC-2.02.013: build_column_array MUST NOT emit \
             `ocsf.enum_label_unrecognized` for an empty-string severity value. \
             Empty string is a missing/unset field, NOT an invalid OCSF enum label; \
             the warn is a false positive that inflates operator noise. \
             FAILS NOW: normalize_enum_label(\"\") returns None → warn fires. \
             Fix: add `!s.is_empty()` guard before calling normalize_enum_label in \
             the ColumnType::String arm of build_column_array (mirrors normalizer.rs \
             SECONDARY path). \
             Captured event_type-bearing WARN events: {:?}",
            *warns
        );
    }

    // ---------------------------------------------------------------------------
    // CR-004 / SEC-001 — CWE-117 control-char sanitization at PRIMARY emission site
    // (code-review pass-1, fix-burst S-PRISMQL-CASE-INSENSITIVE-001)
    // ---------------------------------------------------------------------------

    /// CR-004 / SEC-001 (CWE-117) — PRIMARY `ocsf.enum_label_unrecognized` warn:
    ///
    /// When `build_column_array` emits `ocsf.enum_label_unrecognized` for an
    /// unrecognized enum-label value that contains a newline control character,
    /// the logged `value` field MUST have the control char stripped before emission.
    ///
    /// RED GATE (current HEAD): FAILS — `.chars().take(50).collect::<String>()`
    /// truncates but does NOT strip control chars; `\n` survives into the log.
    /// GREEN GATE: PASSES after CR-004 applies `prism_core::sanitize_for_log` (which strips
    /// Unicode Cc (C0 U+0000–U+001F, DEL U+007F, C1 U+0080–U+009F) + U+2028/U+2029)
    /// before the 50-codepoint cap.
    ///
    /// Traces to: CR-004/SEC-001 CWE-117; BC-2.16.002 catalog row 91 field-schema
    /// amendment (control-char sanitization note); BC-2.02.013 F-CRIT-002 error case.
    #[test]
    fn test_cr004_build_column_array_enum_label_warn_strips_control_chars() {
        use std::sync::Mutex;
        use tracing_subscriber::layer::SubscriberExt;

        // Minimal capture: only the `value` field from ocsf.enum_label_unrecognized.
        #[derive(Default)]
        struct ValueOnlyVisitor {
            value: Option<String>,
        }
        impl tracing::field::Visit for ValueOnlyVisitor {
            fn record_str(&mut self, field: &tracing::field::Field, val: &str) {
                if field.name() == "value" {
                    self.value = Some(val.to_owned());
                }
            }
            fn record_debug(&mut self, field: &tracing::field::Field, val: &dyn std::fmt::Debug) {
                if field.name() == "value" && self.value.is_none() {
                    self.value = Some(format!("{val:?}"));
                }
            }
        }

        struct WarnValueCapture {
            captured_value: Arc<Mutex<Option<String>>>,
        }
        impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for WarnValueCapture {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = ValueOnlyVisitor::default();
                    event.record(&mut visitor);
                    if let Some(v) = visitor.value {
                        *self.captured_value.lock().unwrap() = Some(v);
                    }
                }
            }
        }

        let captured_value: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let layer = WarnValueCapture {
            captured_value: captured_value.clone(),
        };
        let subscriber = tracing_subscriber::registry().with(layer);

        // "VENDOR\nINJECT" — unrecognized label with embedded newline control char.
        // normalize_enum_label("severity", "VENDOR\nINJECT") returns None → warn fires.
        let records = vec![json!({"severity": "VENDOR\nINJECT"})];
        let col = ColumnSpec::new("severity", ColumnType::String, None, vec![]);

        tracing::subscriber::with_default(subscriber, || {
            let _ = build_column_array(&records, &col, "crowdstrike");
        });

        let val = captured_value.lock().unwrap().clone().expect(
            "CR-004: ocsf.enum_label_unrecognized warn must be emitted for \
                 unrecognized 'VENDOR\\nINJECT'; check that the warn fires",
        );

        assert!(
            !val.contains('\n'),
            "CR-004 / SEC-001 (CWE-117): `value` field in ocsf.enum_label_unrecognized \
             warn must have control chars stripped (sanitize_for_log applied before \
             50-codepoint truncation); found '\\n' in captured value: {:?}",
            val
        );
        assert!(
            !val.contains('\r'),
            "CR-004 / SEC-001 (CWE-117): `value` field must have \\r stripped; \
             got: {:?}",
            val
        );
    }

    // ---------------------------------------------------------------------------
    // LOW-001 (MED-001 order-of-operations) — PRIMARY emission site
    // (ADV-PR-P1 S-PRISMQL-CASE-INSENSITIVE-001)
    // ---------------------------------------------------------------------------

    /// RG-080 / LOW-001 (MED-001 vector) — PRIMARY `ocsf.enum_label_unrecognized` warn:
    /// order-of-operations proof vector for BOTH `value` and `sensor_type` fields.
    ///
    /// The existing `test_cr004_build_column_array_enum_label_warn_strips_control_chars`
    /// uses a short input (`"VENDOR\nINJECT"`, 13 chars) where both orders produce the
    /// same length — it proves the control char is stripped but does NOT prove ORDER.
    ///
    /// This test uses a vector where the ORDER matters, applied to BOTH logged fields:
    ///
    /// **`value` field** (from enum-label input):
    /// - Input: ESC sequence at head (`\u{1b}[31m`, 5 codepoints) + 60 legit 'A' chars
    ///   = 65 codepoints total.
    /// - `normalize_enum_label("severity", input)` returns `None` → warn fires.
    ///
    /// **`sensor_type` field** (from sensor_id parameter):
    /// - Input: ESC sequence at head (`\u{1b}[31m`, 5 codepoints) + 60 legit 'B' chars
    ///   = 65 codepoints total.
    ///
    /// Spec order (sanitize→truncate, BC-2.16.002 catalog row 91) for EACH field:
    ///   `sanitize_for_log` strips ESC → `"[31m"` + 60 chars = 64 codepoints
    ///   `.chars().take(50)` → `"[31m"` + 46 chars = **50 codepoints (length = 50)**
    ///
    /// Wrong order (truncate→sanitize) for EACH field:
    ///   `.chars().take(50)` → ESC + `"[31m"` + 45 chars = 50 codepoints
    ///   `sanitize_for_log` strips ESC → `"[31m"` + 45 chars = **49 codepoints (length = 49)**
    ///
    /// RED GATE (original MED-001): FAILS with wrong-order code — `val.len() == 49`, requires `50`.
    /// GREEN GATE: PASSES after sanitize-first order applied to BOTH `value` and `sensor_type`.
    ///
    /// Traces to: MED-001 (ADV-PR-P1); LOW-001 (S-PRISMQL-CASE-INSENSITIVE-001);
    /// OBS-001 (ADV-PR-P3; sensor_type order-symmetry guard); BC-2.16.002 catalog row 91
    /// spec order; CWE-117 (SEC-001).
    #[test]
    fn test_rg080_low001_build_column_array_enum_label_warn_order_of_operations() {
        use std::sync::Mutex;
        use tracing_subscriber::layer::SubscriberExt;

        #[derive(Default)]
        struct WarnFieldVisitor {
            value: Option<String>,
            sensor_type: Option<String>,
        }
        impl tracing::field::Visit for WarnFieldVisitor {
            fn record_str(&mut self, field: &tracing::field::Field, val: &str) {
                match field.name() {
                    "value" => self.value = Some(val.to_owned()),
                    "sensor_type" => self.sensor_type = Some(val.to_owned()),
                    _ => {}
                }
            }
            fn record_debug(&mut self, field: &tracing::field::Field, val: &dyn std::fmt::Debug) {
                match field.name() {
                    "value" if self.value.is_none() => {
                        self.value = Some(format!("{val:?}"));
                    }
                    "sensor_type" if self.sensor_type.is_none() => {
                        self.sensor_type = Some(format!("{val:?}"));
                    }
                    _ => {}
                }
            }
        }

        struct WarnFieldCapture {
            captured_value: Arc<Mutex<Option<String>>>,
            captured_sensor_type: Arc<Mutex<Option<String>>>,
        }
        impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for WarnFieldCapture {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = WarnFieldVisitor::default();
                    event.record(&mut visitor);
                    if let Some(v) = visitor.value {
                        *self.captured_value.lock().unwrap() = Some(v);
                    }
                    if let Some(st) = visitor.sensor_type {
                        *self.captured_sensor_type.lock().unwrap() = Some(st);
                    }
                }
            }
        }

        let captured_value: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let captured_sensor_type: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let layer = WarnFieldCapture {
            captured_value: captured_value.clone(),
            captured_sensor_type: captured_sensor_type.clone(),
        };
        let subscriber = tracing_subscriber::registry().with(layer);

        // LOW-001 / OBS-001 vectors:
        // value input:       ESC + "[31m" (5 codepoints) + 60 As = 65 codepoints total.
        // sensor_type input: ESC + "[31m" (5 codepoints) + 60 Bs = 65 codepoints total.
        // normalize_enum_label("severity", value_input) returns None → warn fires.
        let input_value = "\u{1b}[31m".to_string() + &"A".repeat(60);
        let input_sensor_id = "\u{1b}[31m".to_string() + &"B".repeat(60);
        let records = vec![json!({"severity": input_value.clone()})];
        let col = ColumnSpec::new("severity", ColumnType::String, None, vec![]);

        tracing::subscriber::with_default(subscriber, || {
            let _ = build_column_array(&records, &col, &input_sensor_id);
        });

        let val = captured_value.lock().unwrap().clone().expect(
            "RG-080: ocsf.enum_label_unrecognized warn must be emitted for \
                 unrecognized severity value '\x1b[31mAAA...'; check the warn path fires",
        );

        // --- `value` field assertions (sanitize→truncate order) ---

        // Primary assertion: spec order (sanitize→truncate) yields length 50.
        // Wrong order (truncate→sanitize) yields length 49.
        assert_eq!(
            val.chars().count(),
            50,
            "RG-080 (MED-001 / LOW-001): `value` field in ocsf.enum_label_unrecognized warn \
             must have 50 codepoints (sanitize_for_log applied BEFORE 50-codepoint \
             truncation, BC-2.16.002 catalog row 91 spec order); \
             got len={} value={:?}",
            val.chars().count(),
            val
        );

        // Secondary assertion: ESC control char must be stripped regardless of order.
        assert!(
            !val.contains('\x1b'),
            "RG-080: `value` field must have ESC (\\x1b) stripped by sanitize_for_log; \
             got: {:?}",
            val
        );

        // Tertiary: exact content check — spec order yields "[31m" + 46 As.
        let expected_val = "[31m".to_string() + &"A".repeat(46);
        assert_eq!(
            val, expected_val,
            "RG-080 (MED-001): spec order (sanitize→truncate) must yield {:?}; \
             wrong order (truncate→sanitize) yields {:?}",
            expected_val, val
        );

        // --- `sensor_type` field assertions (OBS-001: same spec order required) ---

        let st = captured_sensor_type.lock().unwrap().clone().expect(
            "RG-080 (OBS-001): ocsf.enum_label_unrecognized warn must include `sensor_type` \
             field (BC-2.16.002 catalog row 91); field absent from captured warn event",
        );

        // Primary: spec order (sanitize→truncate) yields length 50.
        assert_eq!(
            st.chars().count(),
            50,
            "RG-080 (OBS-001): `sensor_type` field must have 50 codepoints \
             (sanitize_for_log applied BEFORE 50-codepoint truncation, \
             BC-2.16.002 catalog row 91 spec order); \
             got len={} sensor_type={:?}",
            st.chars().count(),
            st
        );

        // Secondary: ESC control char must be stripped.
        assert!(
            !st.contains('\x1b'),
            "RG-080 (OBS-001): `sensor_type` field must have ESC (\\x1b) stripped by \
             sanitize_for_log; got: {:?}",
            st
        );

        // Tertiary: exact content check — spec order yields "[31m" + 46 Bs.
        let expected_st = "[31m".to_string() + &"B".repeat(46);
        assert_eq!(
            st, expected_st,
            "RG-080 (OBS-001): spec order (sanitize→truncate) for sensor_type must yield {:?}; \
             got {:?}",
            expected_st, st
        );
    }

    // ---------------------------------------------------------------------------
    // MEDIUM-6 fix — Claroty array column production-path wire-shape assertions
    //
    // The prism-sensors test `test_claroty_tier2_ip_list_array_column_serializes_to_json_list_string`
    // covers `ColumnMapper::map_record` (a non-production intermediate path).  The tests below
    // cover `build_column_array` — the function that actually materialises TOML columns into Arrow
    // on the query execution path.  These are the load-bearing production-path assertions.
    //
    // Two cases verified:
    //   (a) String-array elements → JSON-list string: ip_list = ["10.0.1.1","10.0.1.2"]
    //   (b) Integer-array elements → JSON-list string: vlan_list = [100, 200] → ["100","200"]
    //       (TOML declares vlan_list column_type = "string"; integer elements stringified via
    //       `other.to_string()` in the Value::Array arm of the String branch.)
    // ---------------------------------------------------------------------------

    /// MEDIUM-6 fix (load-bearing, SID-1):
    ///
    /// `build_column_array` on a `ColumnType::String` column with `source_path = "$.ip_list[*]"`
    /// over a record `{"ip_list": ["10.0.1.1","10.0.1.2"]}` MUST produce the Arrow StringArray
    /// cell `["10.0.1.1","10.0.1.2"]` (compact JSON-list string, no spaces).
    ///
    /// This is the Claroty-specific instance of the ENRICH-1 array-column production path.
    /// The existing prism-sensors test covers `ColumnMapper::map_record` only; this test
    /// covers the `build_column_array` path that the query execution engine actually calls.
    #[test]
    fn test_build_column_array_claroty_ip_list_string_elements_serialize_to_json_list_string() {
        let records = vec![
            json!({"ip_list": ["10.0.1.1", "10.0.1.2"]}),
            json!({"ip_list": ["192.168.1.1"]}),
        ];
        let col = col_with_source_path("ip_list", prism_core::ColumnType::String, "$.ip_list[*]");

        let array = build_column_array(&records, &col, "claroty");
        let string_array = array
            .as_any()
            .downcast_ref::<ArrowStringArray>()
            .expect("ip_list column must produce StringArray (MEDIUM-6)");

        // Row 0: exact compact JSON-list string (no spaces, double-quotes).
        assert!(
            !string_array.is_null(0),
            "MEDIUM-6: ip_list with source_path must produce non-null cell"
        );
        assert_eq!(
            string_array.value(0),
            r#"["10.0.1.1","10.0.1.2"]"#,
            "MEDIUM-6: ip_list=[\"10.0.1.1\",\"10.0.1.2\"] must serialise to \
             compact JSON-list string; got {:?}",
            string_array.value(0)
        );

        // Row 1: single-element list.
        assert!(!string_array.is_null(1));
        assert_eq!(string_array.value(1), r#"["192.168.1.1"]"#);
    }

    /// MEDIUM-6 fix — vlan_list integer-element case (load-bearing, SID-1):
    ///
    /// `build_column_array` on a `ColumnType::String` column with `source_path = "$.vlan_list[*]"`
    /// over a record `{"vlan_list": [100, 200]}` MUST produce the Arrow StringArray
    /// cell `["100","200"]` (integers stringified via `other.to_string()` in the Array arm).
    ///
    /// `claroty.sensor.toml` declares `vlan_list` as `column_type = "string"` with the comment
    /// "integer elements are stringified in JSON-list output (e.g. ["100","200"])".
    /// This test is the load-bearing assertion that the claim holds at the production query path.
    #[test]
    fn test_build_column_array_claroty_vlan_list_integer_elements_stringify_to_json_list_string() {
        let records = vec![
            json!({"vlan_list": [100u32, 200u32]}),
            json!({"vlan_list": [300u32]}),
        ];
        let col = col_with_source_path(
            "vlan_list",
            prism_core::ColumnType::String,
            "$.vlan_list[*]",
        );

        let array = build_column_array(&records, &col, "claroty");
        let string_array = array
            .as_any()
            .downcast_ref::<ArrowStringArray>()
            .expect("vlan_list column must produce StringArray (MEDIUM-6)");

        // Row 0: integer elements must be stringified.
        assert!(
            !string_array.is_null(0),
            "MEDIUM-6: vlan_list with integer elements must produce non-null cell"
        );
        assert_eq!(
            string_array.value(0),
            r#"["100","200"]"#,
            "MEDIUM-6: vlan_list=[100,200] (integers) must serialise to [\"100\",\"200\"]; \
             got {:?}",
            string_array.value(0)
        );

        // Row 1: single integer element.
        assert!(!string_array.is_null(1));
        assert_eq!(string_array.value(1), r#"["300"]"#);
    }

    // ---------------------------------------------------------------------------
    // RG-001: map_spec_engine_error_to_sensor_error maps HttpRequestFailed → HttpError
    // ---------------------------------------------------------------------------

    /// RG-001: `map_spec_engine_error_to_sensor_error` MUST map
    /// `SpecEngineError::HttpRequestFailed { status_code > 0 }` to
    /// `SensorError::HttpError { sensor, status, body }` — NOT `SensorError::Internal`.
    ///
    /// Before fix: ALL `SpecEngineError` variants are mapped to `SensorError::Internal`.
    /// After fix (original): `HttpRequestFailed { status_code > 0 }` maps to `SensorError::HttpError`,
    /// allowing `probe_connectivity` to classify a 4xx response as `ConnectivityStatus::Up`.
    ///
    /// F-P37-HIGH-001 fix (body contract): `HttpError.body` carries the RAW sanitized body
    /// snippet — NOT the full `detail` string. `pipeline.rs` formats `detail` as
    /// `"HTTP {status_reason}: {body_snippet}"` (e.g. `"HTTP 401 Unauthorized"`). The
    /// `map_spec_engine_error_to_sensor_error` Arm 1 strips the `"HTTP {reason}: "` prefix
    /// so body = raw snippet. For an empty-body 401, `detail = "HTTP 401 Unauthorized"` has
    /// no `": "` separator after the reason phrase → body = `""` (empty).
    ///
    /// BC-2.08.002 AC-H1-MAP-001 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-001
    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_map_error_http_401_maps_to_http_error_not_internal() {
        use prism_sensors::adapter::SensorError;
        use prism_spec_engine::error::SpecEngineError;

        // Production-shaped detail: pipeline.rs formats as "HTTP {status_reason}"
        // for empty-body responses. "HTTP 401" is a simplified variant (no reason phrase);
        // production would produce "HTTP 401 Unauthorized". Both yield body = "" after
        // the F-P37-HIGH-001 strip (no ": " separator → empty).
        let result = super::map_spec_engine_error_to_sensor_error(
            SpecEngineError::HttpRequestFailed {
                sensor_id: "claroty".to_string(),
                step_name: "fetch".to_string(),
                status_code: 401,
                detail: "HTTP 401 Unauthorized".to_string(),
            },
            "claroty",
            "alerts",
        );

        assert!(
            matches!(result, SensorError::HttpError { .. }),
            "RG-001: HttpRequestFailed(status_code=401) must map to SensorError::HttpError, \
             got SensorError::Internal instead. \
             Fix: map_spec_engine_error_to_sensor_error must match HttpRequestFailed {{ status_code }} \
             when status_code > 0 and return SensorError::HttpError (BC-2.08.002)"
        );

        // Verify all three fields of HttpError are populated correctly.
        if let SensorError::HttpError {
            sensor,
            status,
            body,
        } = result
        {
            assert_eq!(
                sensor, "claroty",
                "RG-001: HttpError.sensor must be the sensor_id arg passed to map fn"
            );
            assert_eq!(
                status, 401,
                "RG-001: HttpError.status must equal HttpRequestFailed.status_code"
            );
            // F-P37-HIGH-001: HttpError.body is the RAW body snippet (empty for a
            // 401 with no response body). The full detail "HTTP 401 Unauthorized" is
            // NOT the body — it is stripped by Arm 1 to prevent double-prefixing in
            // materialization.rs. detail = "HTTP 401 Unauthorized" has no ": " separator
            // → body = "" (no body content after the status reason phrase).
            assert_eq!(
                body, "",
                "RG-001: HttpError.body must be the raw body snippet (empty for empty-body 401). \
                 F-P37-HIGH-001: Arm 1 strips the 'HTTP {{reason}}: ' prefix from detail so \
                 consumers see only the raw snippet, not the full pre-formatted detail string."
            );
        }
    }

    // ---------------------------------------------------------------------------
    // RG-002: regression guard — status_code=0 must remain SensorError::Internal
    // ---------------------------------------------------------------------------

    /// RG-002: `map_spec_engine_error_to_sensor_error` MUST keep
    /// `SpecEngineError::HttpRequestFailed { status_code: 0 }` as `SensorError::Internal`.
    ///
    /// `status_code: 0` represents a synthetic or connection-error-derived code
    /// (not a real HTTP response). These must NOT be classified as `HttpError`.
    ///
    /// NOTE: This test is GREEN-BY-DESIGN before the fix (all errors currently return Internal).
    /// It becomes a REGRESSION GUARD after the fix: if the implementation maps ALL
    /// `HttpRequestFailed` variants to `HttpError` including `status_code=0`, this test fails.
    ///
    /// BC-2.08.002 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-002
    #[test]
    fn test_map_error_status_0_maps_to_internal() {
        use prism_sensors::adapter::SensorError;
        use prism_spec_engine::error::SpecEngineError;

        let result = super::map_spec_engine_error_to_sensor_error(
            SpecEngineError::HttpRequestFailed {
                sensor_id: "claroty".to_string(),
                step_name: "fetch".to_string(),
                status_code: 0,
                detail: "connection refused".to_string(),
            },
            "claroty",
            "alerts",
        );

        assert!(
            matches!(result, SensorError::Internal { .. }),
            "RG-002 (regression guard): HttpRequestFailed(status_code=0) must remain \
             SensorError::Internal — status_code=0 is not a real HTTP response. \
             Got: {:?}",
            result
        );
    }

    // ---------------------------------------------------------------------------
    // RG-010: AuthRefreshFailed maps to HttpError{status:401}, not Internal
    // ---------------------------------------------------------------------------

    /// RG-010: `map_spec_engine_error_to_sensor_error` MUST map
    /// `SpecEngineError::AuthRefreshFailed` to `SensorError::HttpError { status: 401 }`
    /// (NOT `SensorError::Internal`).
    ///
    /// Rationale: `AuthRefreshFailed` means the sensor responded with HTTP 401 twice
    /// (once before token refresh, once after). The sensor IS reachable — it returned
    /// HTTP. Mapping to `Internal` causes `probe_connectivity` to classify the sensor
    /// as `Down` and `probe_auth_with_routing` to return `AuthStatus::Unknown`, when
    /// the correct classification is `Up` + `AuthStatus::Invalid`
    /// (BC-2.08.002 / DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOW-1 fix).
    ///
    /// BC-2.08.002 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOW-1 (unit guard)
    #[test]
    fn test_map_error_auth_refresh_failed_maps_to_http_error_401() {
        use prism_sensors::adapter::SensorError;
        use prism_spec_engine::error::SpecEngineError;

        let result = super::map_spec_engine_error_to_sensor_error(
            SpecEngineError::AuthRefreshFailed {
                sensor_id: "claroty".to_string(),
                client_id: "client-001".to_string(),
                step_name: "fetch".to_string(),
            },
            "claroty",
            "alerts",
        );

        assert!(
            matches!(result, SensorError::HttpError { status: 401, .. }),
            "RG-010: AuthRefreshFailed must map to SensorError::HttpError {{ status: 401 }}, \
             not Internal. Sensor responded with 401 (reachable) — classify as auth-invalid. \
             Got: {:?}",
            result
        );
    }

    // ---------------------------------------------------------------------------
    // RG-011: CookieAuthFailed maps to HttpError{status:401}, not Internal
    // ---------------------------------------------------------------------------

    /// RG-011: `map_spec_engine_error_to_sensor_error` MUST map
    /// `SpecEngineError::CookieAuthFailed` to `SensorError::HttpError { status: 401 }`
    /// (NOT `SensorError::Internal`).
    ///
    /// Rationale: `CookieAuthFailed` means the sensor returned HTTP 401 on a
    /// `CookieRoundtrip` auth sensor. Same semantics as `AuthRefreshFailed` — sensor
    /// IS reachable, credentials are invalid. Should classify as Up + auth-invalid.
    ///
    /// BC-2.08.002 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOW-1 (unit guard)
    #[test]
    fn test_map_error_cookie_auth_failed_maps_to_http_error_401() {
        use prism_sensors::adapter::SensorError;
        use prism_spec_engine::error::SpecEngineError;

        let result = super::map_spec_engine_error_to_sensor_error(
            SpecEngineError::CookieAuthFailed {
                sensor_id: "claroty".to_string(),
                client_id: "client-001".to_string(),
            },
            "claroty",
            "alerts",
        );

        assert!(
            matches!(result, SensorError::HttpError { status: 401, .. }),
            "RG-011: CookieAuthFailed must map to SensorError::HttpError {{ status: 401 }}, \
             not Internal. Sensor responded with 401 (reachable) — classify as auth-invalid. \
             Got: {:?}",
            result
        );
    }

    // ---------------------------------------------------------------------------
    // RG-014 (map-level): HttpRequestFailed{status_code:503} maps to HttpError{status:503}
    // ---------------------------------------------------------------------------

    /// RG-014 (map-level): `map_spec_engine_error_to_sensor_error` MUST map
    /// `SpecEngineError::HttpRequestFailed { status_code: 503, .. }` to
    /// `SensorError::HttpError { status: 503, .. }` — NOT `SensorError::Internal`.
    ///
    /// This guard exists because this story's `status_code > 0` map guard routes 5xx
    /// responses through `SensorError::HttpError`, which `probe_connectivity` then
    /// classifies as `ConnectivityStatus::Degraded` (status >= 500 branch in connectivity.rs).
    /// Before this story, 5xx flowed to `Internal` → catch-all → `Down`.
    ///
    /// The RG-001/RG-002 tests cover 401 and status_code=0; this test is the 5xx-specific
    /// coverage that was missing (F-P25-OBS-001). The end-to-end path is exercised by
    /// `test_probe_connectivity_503_returns_degraded` (RG-014 end-to-end) in
    /// `tests/defect_adapter_tls_xdome_live_001.rs`.
    ///
    /// F-P37-HIGH-001 fix (body contract): `HttpError.body` carries the RAW sanitized body
    /// snippet — NOT the full `detail` string. `pipeline.rs` formats `detail` as
    /// `"HTTP {status_reason}: {body_snippet}"`. For an empty-body 503, `detail` is
    /// `"HTTP 503 Service Unavailable"` (no `": "` separator). Arm 1 finds no `": "` after
    /// `strip_prefix("HTTP ")` → body = `""` (empty, no response body present).
    ///
    /// BC-2.08.002 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-014
    #[test]
    fn test_map_error_503_maps_to_http_error_503() {
        use prism_sensors::adapter::SensorError;
        use prism_spec_engine::error::SpecEngineError;

        // Production-shaped detail: pipeline.rs formats as "HTTP {status_reason}" for
        // empty-body responses. reqwest StatusCode Display for 503 = "503 Service Unavailable".
        let result = super::map_spec_engine_error_to_sensor_error(
            SpecEngineError::HttpRequestFailed {
                sensor_id: "claroty".to_string(),
                step_name: "fetch_devices".to_string(),
                status_code: 503,
                detail: "HTTP 503 Service Unavailable".to_string(),
            },
            "claroty",
            "devices",
        );

        assert!(
            matches!(result, SensorError::HttpError { .. }),
            "RG-014 (map-level): HttpRequestFailed(status_code=503) must map to \
             SensorError::HttpError, not Internal. \
             This story's `status_code > 0` guard routes 5xx to HttpError, which \
             probe_connectivity classifies as Degraded (not Down). \
             Got: {:?}",
            result
        );

        if let SensorError::HttpError {
            sensor,
            status,
            body,
        } = result
        {
            assert_eq!(
                sensor, "claroty",
                "RG-014 (map-level): HttpError.sensor must equal sensor_id arg"
            );
            assert_eq!(
                status, 503,
                "RG-014 (map-level): HttpError.status must equal 503"
            );
            // F-P37-HIGH-001: HttpError.body is the RAW body snippet (empty for empty-body 503).
            // detail = "HTTP 503 Service Unavailable" has no ": " separator after the reason
            // phrase → strip finds no body content → body = "".
            assert_eq!(
                body, "",
                "RG-014 (map-level): HttpError.body must be the raw body snippet (empty for \
                 empty-body 503). F-P37-HIGH-001: Arm 1 strips 'HTTP {{reason}}: ' prefix; \
                 'HTTP 503 Service Unavailable' has no ': ' separator → body = ''."
            );
        }
    }

    // ---------------------------------------------------------------------------
    // RG-006: build_http_client_with_custom_timeout must set prism/ User-Agent header
    // ---------------------------------------------------------------------------

    /// RG-006: `build_http_client_with_custom_timeout` MUST produce a `reqwest::Client`
    /// that sends a `User-Agent` header beginning with `"prism/"`.
    ///
    /// Before fix: no `.user_agent()` call in the builder → reqwest uses its own default
    /// User-Agent ("reqwest/x.x.x") → assertion FAILS → RED.
    ///
    /// After fix: `.user_agent("prism/{version}")` added → header starts with "prism/" → GREEN.
    ///
    /// BC-2.16.002 (HTTP Client Compliance postconditions) AC-UA-001 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-006
    #[tokio::test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    async fn test_build_http_client_sends_user_agent_header() {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path},
        };

        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/probe"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = build_http_client_with_custom_timeout(Duration::from_secs(5))
            .expect("RG-006: client build must succeed");

        // Fire a request so wiremock records the User-Agent header.
        let _ = client
            .get(format!("{}/probe", mock_server.uri()))
            .send()
            .await;

        let received = mock_server
            .received_requests()
            .await
            .expect("RG-006: wiremock must record received requests");

        assert_eq!(
            received.len(),
            1,
            "RG-006: exactly one request must be recorded by wiremock; got {}",
            received.len()
        );

        let ua = received[0]
            .headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        assert!(
            ua.starts_with("prism/"),
            "RG-006: User-Agent header must start with 'prism/'; \
             got: {:?}. \
             Fix: add .user_agent(\"prism/{{version}}\") call to \
             build_http_client_with_custom_timeout (BC-2.16.002 AC-UA-001). \
             reqwest default UA is 'reqwest/x.x.x', not 'prism/...'.",
            ua
        );
    }

    // ---------------------------------------------------------------------------
    // RG-018: map_spec_engine_error_to_sensor_error Arm 1 — non-empty body strips
    //         "HTTP {reason}: " prefix (F-P38-MED-001 regression guard)
    // ---------------------------------------------------------------------------

    /// RG-018: `map_spec_engine_error_to_sensor_error` Arm 1 MUST strip the
    /// `"HTTP {status_reason}: "` prefix from `detail` before storing into
    /// `SensorError::HttpError.body`.
    ///
    /// # Red Gate status
    ///
    /// This test is **GREEN on current code** (Arm 1 strip already implemented by
    /// F-P37-HIGH-001 fix). It is a LOAD-BEARING REGRESSION GUARD:
    ///   - `RG-001` / `RG-014` only exercise the empty-body path (detail has no `": "`)
    ///   - `RG-016` bypasses Arm 1 by constructing `FanOutError` with a pre-stripped body
    ///   - Neither asserts the Arm-1 strip on a non-empty-body input
    ///
    /// This test closes the F-P38-MED-001 regression gap: the strip logic
    /// (`detail.strip_prefix("HTTP ").and_then(|s| s.find(": ").map(...))`)
    /// is the ONLY place that prevents `materialization.rs` from emitting a
    /// doubled prefix (`"{table}: HTTP 403: HTTP 403 Forbidden: <body>"`).
    ///
    /// # Scenario
    ///
    /// `pipeline.rs` formats `detail` as `"HTTP 403 Forbidden: access denied"` when
    /// it receives a 403 response with body `"access denied"`.
    /// Arm 1 must strip `"HTTP 403 Forbidden: "` → `HttpError.body = "access denied"`.
    ///
    /// # SID-2: composed-output / no-duplicated-HTTP-prefix assertion
    ///
    /// The SID-2 requirement is that at least one test asserts on the FULL composed
    /// output. Here: `HttpError.body` is asserted both equal to the exact raw snippet
    /// AND confirmed to NOT contain the substring `"HTTP"` — catching any regression
    /// where the prefix strip is removed and the raw detail bleeds into `body`.
    ///
    /// BC-2.11.001 §Postconditions | T-QERR-1 raw-body invariant |
    /// F-P38-MED-001 regression closure | SID-2 no-duplicated-HTTP-prefix |
    /// DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-018
    #[test]
    fn test_map_error_http_403_nonempty_body_strips_prefix_to_raw_body() {
        use prism_sensors::adapter::SensorError;
        use prism_spec_engine::error::SpecEngineError;

        // Production-shaped detail: pipeline.rs formats as "HTTP {status_reason}: {body_snippet}"
        // when it receives a non-2xx response with a non-empty body.
        // e.g. 403 Forbidden + body "access denied" → detail = "HTTP 403 Forbidden: access denied"
        let result = super::map_spec_engine_error_to_sensor_error(
            SpecEngineError::HttpRequestFailed {
                sensor_id: "xdome".to_string(),
                step_name: "fetch".to_string(),
                status_code: 403,
                detail: "HTTP 403 Forbidden: access denied".to_string(),
            },
            "xdome",
            "devices",
        );

        // ASSERTION 1: result variant is HttpError (Arm 1 guard fires for status_code=403 > 0).
        assert!(
            matches!(result, SensorError::HttpError { .. }),
            "RG-018: HttpRequestFailed(status_code=403) MUST map to SensorError::HttpError \
             (not SensorError::Internal). Arm 1 guard: `status_code > 0` must match."
        );

        if let SensorError::HttpError { status, body, .. } = result {
            // ASSERTION 2: status field carries the HTTP status code.
            assert_eq!(status, 403, "RG-018: HttpError.status must be 403");

            // ASSERTION 3 (SID-2 exact-match): body is the raw snippet, NOT the full detail.
            // Regression: before F-P37-HIGH-001 source fix, body == "HTTP 403 Forbidden: access denied"
            // (full prefixed detail). After fix, body == "access denied" (raw snippet only).
            assert_eq!(
                body, "access denied",
                "RG-018 FAIL (SID-2 exact-match): HttpError.body MUST be raw snippet \
                 'access denied', NOT the full prefixed detail \
                 'HTTP 403 Forbidden: access denied'. \
                 Arm 1 strip: detail.strip_prefix(\"HTTP \").and_then(s.find(\": \").map(...)). \
                 Got body = '{body}'"
            );

            // ASSERTION 4 (SID-2 no-duplicated-HTTP-prefix): body MUST NOT contain "HTTP".
            // This catches any regression where the prefix strip is removed and the full
            // detail bleeds into body, causing materialization.rs to double-prefix:
            //   "{table}: HTTP 403: HTTP 403 Forbidden: access denied"
            assert!(
                !body.contains("HTTP"),
                "RG-018 FAIL (SID-2 no-duplicated-HTTP-prefix): HttpError.body MUST NOT \
                 contain 'HTTP'. The 'HTTP {{reason}}: ' prefix must be stripped by Arm 1 \
                 so body carries only the raw sanitized snippet. \
                 Without this strip, materialization.rs formats \
                 '{{table}}: HTTP {{code}}: HTTP 403 Forbidden: access denied' (doubled prefix). \
                 body = '{body}'"
            );
        }
    }
}
