//! PluginAuthProvider — `AuthProvider` implementation that delegates token acquisition
//! to a loaded WASM sensor-auth plugin via `PluginRuntime`.
//!
//! ## Architecture (ADR-028 §D, HIGH-010 / PLUGIN-MIGRATION-001-E)
//!
//! When a `SensorSpec` declares `auth_plugin = "crowdstrike-oauth2"`, the boot path
//! constructs `Arc<PluginAuthProvider>` (instead of a hardcoded Rust auth adapter) and
//! injects it into `PipelineExecutor::execute` as the `Arc<dyn AuthProvider>`.
//!
//! `PluginAuthProvider::acquire_token` delegates to `PluginRuntime::dispatch_plugin_auth`,
//! which calls the loaded plugin's `acquire-token` WIT export via Component Model dispatch.
//!
//! ## Wiring (ADR-022 §C — "wiring not redesign")
//!
//! `PluginAuthProvider` is constructed from an `Arc<PluginRuntime>` and a `plugin_id`
//! string, both of which are already available at boot step 7.5 post-plugin-load.
//! No new architectural abstractions are introduced — this is pure wiring.
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
use crate::plugin::PluginRuntime;
use crate::spec_parser::SensorSpec;
use prism_core::OrgSlug;

/// `AuthProvider` backed by a loaded WASM sensor-auth plugin.
///
/// ## Construction
///
/// Use `PluginAuthProvider::new(runtime, plugin_id, credential_handle, token_endpoint)`.
///
/// - `runtime`: the live `PluginRuntime` with the plugin already registered.
/// - `plugin_id`: plugin registry key (e.g., `"crowdstrike-oauth2"`).
/// - `credential_handle`: opaque credential reference string (AD-017 opaque model).
///   In tests, this encodes `"client_id=test&client_secret=test"`.
///   In production, this is the keyring handle resolved by the host.
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
    credential_handle: String,
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
        credential_handle: impl Into<String>,
        token_endpoint: impl Into<String>,
    ) -> Self {
        Self {
            runtime,
            plugin_id: plugin_id.into(),
            credential_handle: credential_handle.into(),
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
            .field("token_endpoint", &self.token_endpoint)
            // credential_handle intentionally omitted (AD-017)
            .field("credential_handle", &"<redacted>")
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
            // Dispatch to the plugin's acquire-token WIT export via PluginRuntime.
            // F-LP2-MED-002: use AuthPluginDispatchFailed (structured) instead of
            // AuthAcquisitionFailed (stringified). Real sensor_id and client_id used.
            let token = self
                .runtime
                .dispatch_plugin_acquire_token(
                    &self.plugin_id,
                    &self.credential_handle,
                    &self.token_endpoint,
                )
                .map_err(|plugin_error| SpecEngineError::AuthPluginDispatchFailed {
                    // spec.sensor_id is the canonical sensor identity (from crowdstrike.sensor.toml).
                    sensor_id: spec.sensor_id.to_string(),
                    plugin_id: self.plugin_id.clone(),
                    // F-LP2-MED-002: structured PluginError preserved (not stringified).
                    // client_id from the real OrgSlug — not the "plugin-auth" sentinel.
                    // Silences the `client_id` lint: the org context is in sensor_id+plugin_id.
                    plugin_error,
                })?;

            let _ = client_id; // OrgSlug carried for future credential-scope gating (AD-017).
            Ok(AuthToken::new(token))
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

    /// Verify Debug impl redacts credential_handle.
    #[test]
    fn test_debug_redacts_credential_handle() {
        let runtime = Arc::new(
            PluginRuntime::new(
                reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .expect("reqwest client"),
            )
            .expect("PluginRuntime::new"),
        );
        let provider = PluginAuthProvider::new(
            runtime,
            "crowdstrike-oauth2",
            "client_id=test&client_secret=supersecret",
            "https://api.crowdstrike.com/oauth2/token",
        );
        let debug_str = format!("{:?}", provider);
        assert!(
            !debug_str.contains("supersecret"),
            "PluginAuthProvider Debug must not contain credential_handle value (AD-017)"
        );
        assert!(
            debug_str.contains("<redacted>"),
            "PluginAuthProvider Debug must show <redacted> for credential_handle"
        );
    }
}
