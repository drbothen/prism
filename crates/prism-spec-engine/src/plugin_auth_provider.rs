//! PluginAuthProvider — `AuthProvider` implementation that delegates token acquisition
//! to a loaded WASM sensor-auth plugin via `PluginRuntime`.
//!
//! ## Architecture (ADR-028 §D11 Option C / PLUGIN-MIGRATION-001-E)
//!
//! When a `SensorSpec` declares `auth_plugin = "crowdstrike-oauth2"`, the boot path
//! constructs `Arc<PluginAuthProvider>` (instead of a hardcoded Rust auth adapter) and
//! injects it into `PipelineExecutor::execute` as the `Arc<dyn AuthProvider>`.
//!
//! `PluginAuthProvider::acquire_token` resolves credentials from `prism_credentials`
//! before delegating to `PluginRuntime::dispatch_plugin_acquire_token` with an explicit
//! `PluginConfigMap` containing `client_id`, `client_secret`, and `token_endpoint`.
//!
//! ## Credential Substitution (ADR-028 §D11 Option C)
//!
//! The `credential_handle` opaque string is replaced by credential resolution at
//! dispatch time. `PluginAuthProvider` stores `sensor_id` and resolves:
//!   - `resolve_credential(org_slug, sensor_id, "client_id")`
//!   - `resolve_credential(org_slug, sensor_id, "client_secret")`
//!
//! The resolved `SecretString` values are materialized once via `expose_secret()` to
//! populate `PluginConfigMap` for the duration of the single dispatch call.
//!
//! ## Wiring (ADR-022 §C — "wiring not redesign")
//!
//! `PluginAuthProvider` is constructed from `Arc<PluginRuntime>`, `plugin_id`,
//! `sensor_id`, and `token_endpoint`. The `sensor_id` is used as the credential
//! namespace key in the prism-credentials resolution chain (BC-2.03.006).
//!
//! ## Object Safety
//!
//! `PluginAuthProvider` implements `AuthProvider` which requires object safety. The
//! `acquire_token` method returns `Pin<Box<dyn Future<...>>>` — the canonical Rust pattern
//! for object-safe async traits (established in S-PLUGIN-PREREQ-B).

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::auth_provider::{AuthProvider, AuthToken};
use crate::error::SpecEngineError;
use crate::plugin::{PluginConfigMap, PluginRuntime};
use crate::spec_parser::SensorSpec;
use prism_core::OrgSlug;

/// `AuthProvider` backed by a loaded WASM sensor-auth plugin.
///
/// ## Construction
///
/// Use `PluginAuthProvider::new(runtime, plugin_id, sensor_id, token_endpoint)`.
///
/// - `runtime`: the live `PluginRuntime` with the plugin already registered.
/// - `plugin_id`: plugin registry key (e.g., `"crowdstrike-oauth2"`).
/// - `sensor_id`: sensor identity string (e.g., `"crowdstrike"`) used as the
///   credential namespace key for `prism_credentials::resolve_credential` (ADR-028 §D11).
/// - `token_endpoint`: full URL of the OAuth2 token endpoint (e.g.,
///   `"https://api.crowdstrike.com/oauth2/token"` or DTU clone URL in tests).
///
/// ## `#[non_exhaustive]`
///
/// Public struct; marked `#[non_exhaustive]` per CLAUDE.md conventions — new fields
/// may be added in future stories without breaking external callers.
#[non_exhaustive]
pub struct PluginAuthProvider {
    runtime: Arc<PluginRuntime>,
    plugin_id: String,
    /// Sensor identity used as credential namespace key (BC-2.03.006 / ADR-028 §D11).
    ///
    /// Replaces `credential_handle` (which was an opaque `"sensor:{sensor_id}"` string).
    /// The `sensor_id` is the canonical identifier from the TOML sensor spec.
    sensor_id: String,
    token_endpoint: String,
}

impl PluginAuthProvider {
    /// Construct a `PluginAuthProvider`.
    ///
    /// # Panics
    ///
    /// Does NOT panic on construction — panics only occur at dispatch time if the
    /// plugin is not registered in `runtime.registry`.
    pub fn new(
        runtime: Arc<PluginRuntime>,
        plugin_id: impl Into<String>,
        sensor_id: impl Into<String>,
        token_endpoint: impl Into<String>,
    ) -> Self {
        Self {
            runtime,
            plugin_id: plugin_id.into(),
            sensor_id: sensor_id.into(),
            token_endpoint: token_endpoint.into(),
        }
    }

    /// Return the `plugin_id` this provider delegates to.
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }
}

impl std::fmt::Debug for PluginAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginAuthProvider")
            .field("plugin_id", &self.plugin_id)
            .field("sensor_id", &self.sensor_id)
            .field("token_endpoint", &self.token_endpoint)
            // credential values never in Debug output (AD-017)
            .finish()
    }
}

impl AuthProvider for PluginAuthProvider {
    fn acquire_token<'a>(
        &'a self,
        spec: &'a SensorSpec,
        client_id: &'a OrgSlug,
    ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
        Box::pin(async move {
            // ADR-028 §D11 Option C: resolve credentials from prism_credentials before dispatch.
            // BC-2.03.006 resolution chain: env var → crud store → NotFound.
            let cid_str = client_id.as_str();

            let resolved_client_id =
                prism_credentials::resolve_credential(cid_str, &self.sensor_id, "client_id")
                    .await
                    .map_err(|e| SpecEngineError::AuthAcquisitionFailed {
                        sensor_id: self.sensor_id.clone(),
                        client_id: cid_str.to_string(),
                        // structural error message (not a credential value — BC-2.03.006 audit)
                        detail: e.to_string(),
                    })?;

            let resolved_client_secret =
                prism_credentials::resolve_credential(cid_str, &self.sensor_id, "client_secret")
                    .await
                    .map_err(|e| SpecEngineError::AuthAcquisitionFailed {
                        sensor_id: self.sensor_id.clone(),
                        client_id: cid_str.to_string(),
                        detail: e.to_string(),
                    })?;

            // Materialize credential values at the PluginConfigMap boundary.
            // This is the SOLE location where SecretString values are exposed.
            // The PluginConfigMap lifetime is bounded to the dispatch_plugin_acquire_token
            // call frame (dropped on function return per ADR-028 §D11 AD-017 analysis).
            //
            // SEC-005 (CWE-316): credential String allocations are explicitly zeroized before
            // dropping via `zeroize::Zeroize`. The upstream `SecretString` values zeroize on
            // drop; `.expose_secret().to_string()` creates a new heap allocation that does NOT
            // inherit that guarantee. We hold the credential entries as mutable and zeroize them
            // explicitly after dispatch returns, before `config` is dropped.
            use secrecy::ExposeSecret;
            use zeroize::Zeroize;
            let mut config = PluginConfigMap::from([
                (
                    "client_id".to_string(),
                    resolved_client_id.expose_secret().to_string(),
                ),
                (
                    "client_secret".to_string(),
                    resolved_client_secret.expose_secret().to_string(),
                ),
                ("token_endpoint".to_string(), self.token_endpoint.clone()),
            ]);

            // Dispatch to the plugin's acquire-token WIT export via PluginRuntime.
            // F-LP2-MED-002: use AuthPluginDispatchFailed (structured) instead of
            // AuthAcquisitionFailed (stringified). Real sensor_id and client_id used.
            let dispatch_result = self
                .runtime
                .dispatch_plugin_acquire_token(&self.plugin_id, &config)
                .map_err(|plugin_error| SpecEngineError::AuthPluginDispatchFailed {
                    // spec.sensor_id is the canonical sensor identity (from crowdstrike.sensor.toml).
                    sensor_id: spec.sensor_id.to_string(),
                    plugin_id: self.plugin_id.clone(),
                    // F-LP2-MED-002: structured PluginError preserved (not stringified).
                    plugin_error,
                });

            // SEC-005: zeroize credential heap allocations before config drops.
            // `token_endpoint` is not a credential; only client_id and client_secret require zeroing.
            if let Some(v) = config.get_mut("client_id") {
                v.zeroize();
            }
            if let Some(v) = config.get_mut("client_secret") {
                v.zeroize();
            }

            Ok(AuthToken::new(dispatch_result?))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify PluginAuthProvider is object-safe (can be coerced to &dyn AuthProvider).
    #[test]
    fn test_plugin_auth_provider_is_object_safe() {
        // If this compiles, AuthProvider is object-safe for PluginAuthProvider.
        // The actual runtime test (with a loaded plugin) is in crowdstrike_oauth2_plugin_tests.rs.
        fn _assert_dyn_compatible(_: &dyn AuthProvider) {}
        // We cannot easily construct PluginAuthProvider without a PluginRuntime in a unit test,
        // but the trait object coercion is verified by the type system here.
        // The compile-time check is the test: if AuthProvider is not object-safe for
        // PluginAuthProvider, this file fails to compile.
    }

    /// Verify Debug impl shows sensor_id and plugin_id (structural identifiers),
    /// and does NOT store or expose credential values.
    ///
    /// ADR-028 §D11 Option C: credentials are NEVER stored in PluginAuthProvider.
    /// They are resolved from prism_credentials at dispatch time and materialized only
    /// at the PluginConfigMap boundary. This is the structural AD-017 guarantee:
    /// no credential value can appear in Debug output because the struct never holds one.
    #[test]
    fn test_debug_shows_structural_ids_not_credentials() {
        let runtime = Arc::new(
            PluginRuntime::new(
                reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .expect("reqwest client"),
            )
            .expect("PluginRuntime::new"),
        );
        // sensor_id = "crowdstrike" (structural identifier, NOT a credential value).
        // token_endpoint is a URL (not a credential).
        let provider = PluginAuthProvider::new(
            runtime,
            "crowdstrike-oauth2",
            "crowdstrike",
            "https://api.crowdstrike.com/oauth2/token",
        );
        let debug_str = format!("{:?}", provider);

        // Structural identifiers MUST appear in Debug output for operator diagnostics.
        assert!(
            debug_str.contains("crowdstrike-oauth2"),
            "PluginAuthProvider Debug must contain plugin_id; got: {debug_str}"
        );
        assert!(
            debug_str.contains("crowdstrike"),
            "PluginAuthProvider Debug must contain sensor_id; got: {debug_str}"
        );

        // AD-017: credential values NEVER stored in struct fields → cannot appear in Debug.
        // The struct stores only: runtime (Arc<PluginRuntime>), plugin_id, sensor_id, token_endpoint.
        // No client_id value, no client_secret value, no credential bytes anywhere.
        // (The old credential_handle "client_id=...&client_secret=..." field is GONE.)
        assert!(
            !debug_str.contains("client_secret"),
            "PluginAuthProvider Debug must never contain 'client_secret' (AD-017); got: {debug_str}"
        );
    }
}
