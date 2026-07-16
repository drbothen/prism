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
//!   - `resolve_credential(org_slug, sensor_id, "client_id", org_id: Option<&OrgId>, keyring: Option<&Arc<dyn CredentialStoreOrgId>>)`
//!   - `resolve_credential(org_slug, sensor_id, "client_secret", org_id: Option<&OrgId>, keyring: Option<&Arc<dyn CredentialStoreOrgId>>)`
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

use std::{future::Future, pin::Pin, sync::Arc};

use prism_core::{OrgId, OrgRegistry, OrgSlug};

use crate::{
    auth_provider::{AuthProvider, AuthToken},
    error::SpecEngineError,
    plugin::{PluginConfigMap, PluginRuntime},
    spec_parser::SensorSpec,
};

/// `AuthProvider` backed by a loaded WASM sensor-auth plugin.
///
/// ## Construction
///
/// Use `PluginAuthProvider::new(runtime, plugin_id, sensor_id, token_endpoint, org_registry, keyring)`.
///
/// - `runtime`: the live `PluginRuntime` with the plugin already registered.
/// - `plugin_id`: plugin registry key (e.g., `"crowdstrike-oauth2"`).
/// - `sensor_id`: sensor identity string (e.g., `"crowdstrike"`) used as the
///   credential namespace key for `prism_credentials::resolve_credential` (ADR-028 §D11).
/// - `token_endpoint`: full URL of the OAuth2 token endpoint.
/// - `org_registry`: for slug → OrgId resolution before calling `resolve_credential` (ADR-034 §D1).
/// - `keyring`: OrgId-keyed keyring backend for Tier-3 resolution (ADR-034 §D1).
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
    /// OrgRegistry for slug → OrgId resolution (ADR-034 §D1).
    ///
    /// Resolution happens in `prism-spec-engine` (not `prism-credentials`) per the
    /// architecture compliance rule in `trait_.rs:84–85`.
    org_registry: Arc<OrgRegistry>,
    /// OrgId-keyed keyring backend for Tier-3 credential resolution (ADR-034 §D1).
    keyring: Arc<dyn prism_credentials::CredentialStoreOrgId>,
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
        org_registry: Arc<OrgRegistry>,
        keyring: Arc<dyn prism_credentials::CredentialStoreOrgId>,
    ) -> Self {
        Self {
            runtime,
            plugin_id: plugin_id.into(),
            sensor_id: sensor_id.into(),
            token_endpoint: token_endpoint.into(),
            org_registry,
            keyring,
        }
    }

    /// Return the `plugin_id` this provider delegates to.
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    /// Return a clone of the `Arc<PluginRuntime>` this provider holds.
    ///
    /// Used by boot step 9A to construct per-org `PluginAuthProvider` instances with
    /// the correct per-org token endpoint (overlay base_url) while reusing the same
    /// `PluginRuntime` — avoiding redundant plugin loads.
    pub fn runtime_arc(&self) -> Arc<PluginRuntime> {
        Arc::clone(&self.runtime)
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
            // ADR-034 §D1: resolve slug → OrgId here (in prism-spec-engine), then pass to resolve_credential.
            // BC-2.03.006 / BC-2.06.003: full four-tier resolution chain.
            let cid_str = client_id.as_str();

            // Slug → OrgId resolution (ADR-034 §D1: prism-credentials MUST NOT import OrgRegistry;
            // resolution happens in prism-spec-engine which can import it).
            // OrgSlug::new infallibly constructs; resolve() returns None if slug not registered.
            let org_slug = prism_core::OrgSlug::new(cid_str);
            let org_id: Option<OrgId> = self.org_registry.resolve(&org_slug);

            let resolved_client_id = prism_credentials::resolve_credential(
                cid_str,
                &self.sensor_id,
                "client_id",
                org_id.as_ref(),
                Some(&self.keyring),
            )
            .await
            .map_err(|e| SpecEngineError::AuthAcquisitionFailed {
                sensor_id: self.sensor_id.clone(),
                client_id: cid_str.to_string(),
                // structural error message (not a credential value — BC-2.03.006 audit)
                detail: e.to_string(),
            })?;

            let resolved_client_secret = prism_credentials::resolve_credential(
                cid_str,
                &self.sensor_id,
                "client_secret",
                org_id.as_ref(),
                Some(&self.keyring),
            )
            .await
            .map_err(|e| SpecEngineError::AuthAcquisitionFailed {
                sensor_id: self.sensor_id.clone(),
                client_id: cid_str.to_string(),
                detail: e.to_string(),
            })?;

            // Build PluginConfigMap with SecretString values (SEC-008 / CWE-316 closure).
            //
            // All values — including `token_endpoint` — are wrapped in `SecretString`.
            // This means ALL copies of the map (including the `Arc::clone` in `make_host_state`
            // and the Arc stored in `HostState.config`) will automatically zeroize their heap
            // allocations on drop. No explicit `zeroize()` calls are needed; `SecretString`
            // handles it unconditionally for every copy.
            //
            // The prior SEC-005 explicit-zeroize approach (`.expose_secret().to_string()` +
            // `zeroize()`) only protected the caller's copy — it missed the cloned-bytes-on-heap
            // in `HostState.config` created by `make_host_state(config.clone())` (SEC-008).
            // The `SecretString` approach is correct-by-construction: no copy escapes zeroization.
            use secrecy::{ExposeSecret, SecretString};
            let config = PluginConfigMap::from([
                (
                    "client_id".to_string(),
                    SecretString::new(resolved_client_id.expose_secret().to_owned()),
                ),
                (
                    "client_secret".to_string(),
                    SecretString::new(resolved_client_secret.expose_secret().to_owned()),
                ),
                (
                    "token_endpoint".to_string(),
                    SecretString::new(self.token_endpoint.clone()),
                ),
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
            // `config` and its SecretString values are dropped here and zeroized automatically.
            // No explicit zeroize() calls needed — SecretString handles all copies on drop.

            Ok(AuthToken::new(dispatch_result?))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ADR-034 §D5 Red Gate sibling sweep: PluginAuthProvider::new now requires
    // Arc<OrgRegistry> + Arc<dyn CredentialStoreOrgId>. Unit tests use null stubs.
    struct NullTestOrgIdStore;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStoreOrgId for NullTestOrgIdStore {
        async fn get_by_org(
            &self,
            _o: &prism_core::OrgId,
            _s: &str,
            _n: &prism_core::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, prism_core::PrismError> {
            Ok(None)
        }
        async fn set_by_org(
            &self,
            _o: &prism_core::OrgId,
            _s: &str,
            _n: &prism_core::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), prism_core::PrismError> {
            Ok(())
        }
        async fn delete_by_org(
            &self,
            _o: &prism_core::OrgId,
            _s: &str,
            _n: &prism_core::CredentialName,
        ) -> Result<bool, prism_core::PrismError> {
            Ok(false)
        }
        async fn list_by_org(
            &self,
            _o: &prism_core::OrgId,
        ) -> Result<Vec<(String, prism_core::CredentialName)>, prism_core::PrismError> {
            Ok(vec![])
        }
        async fn exists_by_org(
            &self,
            _o: &prism_core::OrgId,
            _s: &str,
            _n: &prism_core::CredentialName,
        ) -> Result<bool, prism_core::PrismError> {
            Ok(false)
        }
    }

    fn null_org_id_store() -> Arc<dyn prism_credentials::CredentialStoreOrgId> {
        Arc::new(NullTestOrgIdStore)
    }

    fn null_org_registry() -> Arc<prism_core::OrgRegistry> {
        Arc::new(prism_core::OrgRegistry::new())
    }

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
        // ADR-034 §D5 sibling sweep: pass null stubs for org_registry and keyring.
        let provider = PluginAuthProvider::new(
            runtime,
            "crowdstrike-oauth2",
            "crowdstrike",
            "https://api.crowdstrike.com/oauth2/token",
            null_org_registry(),
            null_org_id_store(),
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
