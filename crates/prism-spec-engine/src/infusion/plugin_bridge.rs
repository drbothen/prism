//! Plugin infusion source — delegates to the WASM runtime (S-1.15).
//!
//! `PluginInfusionSource` implements `InfusionSource` by calling
//! `PluginRuntime::enrich_single` on the named `.prx` plugin.
//!
//! `plugin_id` and `config` are captured at construction time (in
//! `InfusionRegistry::load_spec_with_runtime`) because the `InfusionSource` trait signature
//! only receives `input` and `input_type`.
//!
//! # Architecture Compliance
//! - PROHIBITED in detection rule filters (E-RULE-012 / BC-2.19.003 / INV-INFUSE-003).
//! - Uses the UNTYPED `component::Val` path via `PluginRuntime::enrich_single` — NOT `TypedFunc`.
//! - `post_return` is NOT called — removed in wasmtime >=44 (plugin/mod.rs ~L970).
//! - `PluginError` is mapped to `InfusionError` at this boundary.
//!
//! # Implementation (S-DEMO-ENRICHMENT-PIVOT-001)
//! `enrich_single` delegates to `PluginRuntime::enrich_single` using the UNTYPED
//! `component::Val` path. Maps `PluginError → InfusionError`. Since the `InfusionSource`
//! trait returns `Option<Value>` (not `Result`), failures (including `NotLoaded`) are logged
//! at WARN level and returned as `None` — never panics.

use std::sync::Arc;

use prism_core::InfusionError;

use super::InfusionSource;
use crate::plugin::{PluginConfigMap, PluginRuntime};

/// Plugin-backed infusion source.
///
/// Delegates enrichment calls to the WASM plugin runtime (S-1.15) via
/// `PluginRuntime::enrich_single`. `plugin_id` and `config` must be populated
/// at construction time from the parsed `.infusion.toml` spec.
///
/// This source type is PROHIBITED in detection rule filters (E-RULE-012 / BC-2.19.003).
///
/// `#[non_exhaustive]`: forward-compat per CLAUDE.md §Conventions — new fields may be
/// added (e.g., timeout overrides, retry policy) without a breaking semver change.
/// External callers must use `PluginInfusionSource::new()`.
#[non_exhaustive]
pub struct PluginInfusionSource {
    /// The plugin_id as registered in `PluginRuntime` (e.g., `"threat_intel"`).
    ///
    /// Passed as the first argument to `PluginRuntime::enrich_single(plugin_id, ...)`.
    pub plugin_id: String,

    /// The plugin configuration map for this infusion.
    ///
    /// Constructed as an empty `PluginConfigMap` in `InfusionRegistry::load_spec_with_runtime`;
    /// credential values are resolved at call time from env vars per AD-017, not pre-populated
    /// here. Passed as `config` to `PluginRuntime::enrich_single`.
    pub config: Arc<PluginConfigMap>,

    /// Reference to the shared `PluginRuntime` for WASM dispatch.
    ///
    /// `Arc` so the source can be cloned into `InfusionUdfDescriptor::source` without
    /// copying the runtime engine.
    pub runtime: Arc<PluginRuntime>,
}

impl std::fmt::Debug for PluginInfusionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginInfusionSource")
            .field("plugin_id", &self.plugin_id)
            .field("config", &format!("<{} keys>", self.config.len()))
            .field("runtime", &"<PluginRuntime>")
            .finish()
    }
}

impl PluginInfusionSource {
    /// Construct a `PluginInfusionSource`.
    ///
    /// `#[non_exhaustive]` prevents struct literal construction; use this constructor.
    pub fn new(
        plugin_id: impl Into<String>,
        config: Arc<PluginConfigMap>,
        runtime: Arc<PluginRuntime>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            config,
            runtime,
        }
    }
}

impl InfusionSource for PluginInfusionSource {
    /// Returns `true` — this source IS backed by the WASM plugin runtime.
    ///
    /// Overrides the `InfusionSource::is_plugin_backed()` default (`false`) to distinguish
    /// `PluginInfusionSource` from `NullSource` and local-lookup sources without `Any`
    /// downcasting (Task 13 / F-SV-1 load-bearing assertion path).
    fn is_plugin_backed(&self) -> bool {
        true
    }

    /// Enrich a single input value via the WASM plugin runtime.
    ///
    /// Delegates to `PluginRuntime::enrich_single(plugin_id, input, input_type, config)`
    /// using the UNTYPED `component::Val` path — NOT `TypedFunc`, NOT `post_return`.
    ///
    /// Maps `PluginError → InfusionError` at this boundary. Since `InfusionSource::enrich_single`
    /// returns `Option<Value>` (not `Result`), plugin failures are logged at WARN level
    /// and returned as `None` (no enrichment available for this input).
    ///
    /// # NotLoaded handling
    /// If `PluginRuntime::enrich_single` returns `PluginError::NotLoaded` for the plugin_id,
    /// it means the plugin has not yet been loaded into the runtime (e.g., plugin binary missing at
    /// boot time, S-1.15 boot-wiring not yet reached, or plugin misnamed in the spec). This is a
    /// routine runtime error — the method logs at WARN level and returns `Ok(None)` (no enrichment
    /// available), exactly as all other `PluginError` variants do. A live query MUST NOT panic the
    /// query engine due to a plugin not being loaded at runtime.
    fn enrich_single(&self, input: &str, input_type: &str) -> Option<serde_json::Value> {
        match self
            .runtime
            .enrich_single(&self.plugin_id, input, input_type, &self.config)
        {
            Ok(result) => result,
            Err(prism_core::PluginError::NotLoaded { ref plugin_id }) => {
                // NotLoaded is a routine runtime error (plugin not in the loaded map —
                // misnamed/unbooted/failed-load). Log at WARN and return None so the query
                // engine continues without enrichment for this row.
                let infusion_err = map_plugin_error_to_infusion_error(
                    &self.plugin_id,
                    prism_core::PluginError::NotLoaded {
                        plugin_id: plugin_id.clone(),
                    },
                );
                tracing::warn!(
                    plugin_id = %self.plugin_id,
                    input_type = %input_type,
                    error = %infusion_err,
                    "plugin not loaded in PluginRuntime — returning None for input (boot-wiring required)"
                );
                None
            }
            Err(plugin_err) => {
                let infusion_err = map_plugin_error_to_infusion_error(&self.plugin_id, plugin_err);
                tracing::warn!(
                    plugin_id = %self.plugin_id,
                    input_type = %input_type,
                    error = %infusion_err,
                    "plugin enrichment call failed — returning None for input"
                );
                None
            }
        }
    }

    /// Enrich a batch of input values.
    ///
    /// Delegates to `enrich_single` for each input.
    /// May be overridden for true batching when the plugin ABI supports it.
    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
        inputs
            .iter()
            .map(|input| self.enrich_single(input, input_type))
            .collect()
    }
}

/// Map a `PluginError` to `InfusionError::PluginCallFailed` (E-INFUSE-008).
///
/// Called at the `PluginInfusionSource::enrich_single` boundary so plugin failures
/// propagate through the infusion error surface without leaking WASM internals.
///
/// Uses the first-class `InfusionError::PluginCallFailed { plugin_id, infusion_id, reason }`
/// variant added in S-1.14-REDO (previously used `MissingRequiredField` as a stand-in).
///
/// # Credential / path redaction (E-INFUSE-008 / AD-017 / INV-INFUSE-005)
///
/// Taxonomy E-INFUSE-008 mandates that `reason` is redacted to `"<redacted>"` when the
/// raw error string contains a filesystem path (`/` or `\` path separators) or any
/// credential-like substring (patterns recognised by `contains_credential_like_pattern`).
///
/// Some `PluginError` variants embed `path` fields (e.g., `InvalidInterface`, `CompilationFailed`,
/// `EmptyPluginId`, `MissingAllowedUrls`, `FormatVersionExceeded`) that can expose the plugin
/// binary's filesystem location. These must be redacted before the reason escapes to callers
/// (AD-017: credential/path values must never transit AI context or external logging surfaces).
///
/// Variants that do NOT embed paths or credentials (e.g., `Trapped`, `Timeout`, `MemoryExceeded`,
/// `NotLoaded`, `SandboxViolation`) are passed through verbatim.
pub(crate) fn map_plugin_error_to_infusion_error(
    plugin_id: &str,
    err: prism_core::PluginError,
) -> InfusionError {
    let raw_reason = err.to_string();
    let reason = redact_if_sensitive(&raw_reason);
    InfusionError::PluginCallFailed {
        plugin_id: plugin_id.to_string(),
        // In current wiring, infusion_id == plugin_id (set at PluginInfusionSource construction
        // from spec.infusion_id). Kept as a separate field for future cases where they diverge.
        infusion_id: plugin_id.to_string(),
        reason,
    }
}

/// Return `"<redacted>"` if `s` contains a filesystem path separator or a credential-like
/// pattern; otherwise return `s.to_string()`.
///
/// Filesystem path indicators: `/` (Unix) or `\` (Windows) path separators.
/// Credential-like patterns: `password`, `secret`, `token`, `api_key`, `apikey`, `credential`,
/// `bearer`, `authorization` (case-insensitive). These substrings indicate that the error
/// message may contain a redacted value that was accidentally embedded by a calling layer.
///
/// This is intentionally conservative: any hit → full redaction. The reason string is
/// diagnostic only; loss of diagnostic detail is preferable to path/credential leakage.
fn redact_if_sensitive(s: &str) -> String {
    // Check for filesystem path separators.
    if s.contains('/') || s.contains('\\') {
        return "<redacted>".to_string();
    }
    // Check for credential-like substrings (case-insensitive).
    let lower = s.to_lowercase();
    const CREDENTIAL_PATTERNS: &[&str] = &[
        "password",
        "secret",
        "token",
        "api_key",
        "apikey",
        "credential",
        "bearer",
        "authorization",
    ];
    for pattern in CREDENTIAL_PATTERNS {
        if lower.contains(pattern) {
            return "<redacted>".to_string();
        }
    }
    s.to_string()
}

#[cfg(test)]
mod redaction_tests {
    use super::*;
    use prism_core::PluginError;

    /// E-INFUSE-008: reason containing a filesystem path is redacted to "<redacted>".
    ///
    /// `CompilationFailed` embeds the plugin binary path — a typical case where the reason
    /// string carries a filesystem location. After the HIGH-2 fix, the `reason` field of the
    /// resulting `InfusionError::PluginCallFailed` must be `"<redacted>"`, not the raw path.
    #[test]
    fn test_reason_with_path_is_redacted_to_placeholder() {
        let err = PluginError::CompilationFailed {
            path: "/secret/plugins/threat_intel.prx".to_string(),
            message: "invalid wasm bytes".to_string(),
        };
        let infusion_err = map_plugin_error_to_infusion_error("threat_intel", err);
        match infusion_err {
            prism_core::InfusionError::PluginCallFailed { reason, .. } => {
                assert_eq!(
                    reason, "<redacted>",
                    "E-INFUSE-008 HIGH-2: reason containing a path must be redacted; \
                     raw reason was the PluginError Display which contains the path. Got: {reason}"
                );
            }
            other => panic!("expected InfusionError::PluginCallFailed; got {:?}", other),
        }
    }

    /// E-INFUSE-008: reason with no path or credential-like content passes through verbatim.
    ///
    /// `Trapped` embeds only a plugin_id and a trap message — no path, no credential.
    /// The reason must NOT be redacted; it is safe diagnostic information.
    #[test]
    fn test_reason_without_path_or_credential_passes_through_verbatim() {
        let err = PluginError::Trapped {
            plugin_id: "threat_intel".to_string(),
            message: "unreachable instruction".to_string(),
        };
        let infusion_err = map_plugin_error_to_infusion_error("threat_intel", err);
        match infusion_err {
            prism_core::InfusionError::PluginCallFailed { reason, .. } => {
                assert_ne!(
                    reason, "<redacted>",
                    "E-INFUSE-008 HIGH-2: reason without path or credential must NOT be redacted; \
                     got: {reason}"
                );
                assert!(
                    reason.contains("unreachable"),
                    "E-INFUSE-008 HIGH-2: verbatim reason must contain trap message; got: {reason}"
                );
            }
            other => panic!("expected InfusionError::PluginCallFailed; got {:?}", other),
        }
    }

    /// E-INFUSE-008: reason containing a credential-like substring is redacted.
    ///
    /// Defends against cases where a plugin error message accidentally embeds a credential
    /// keyword (e.g., "failed to load api_key from config"). After the HIGH-2 fix, such
    /// reason strings must be redacted to "<redacted>" per AD-017.
    #[test]
    fn test_reason_with_credential_like_pattern_is_redacted() {
        // NotLoaded with a plugin_id containing "token" would trigger the credential check.
        // Use a Trapped variant where the message contains "api_key".
        let err = PluginError::Trapped {
            plugin_id: "geoip".to_string(),
            message: "failed to read api_key from environment".to_string(),
        };
        let infusion_err = map_plugin_error_to_infusion_error("geoip", err);
        match infusion_err {
            prism_core::InfusionError::PluginCallFailed { reason, .. } => {
                assert_eq!(
                    reason, "<redacted>",
                    "E-INFUSE-008 HIGH-2: reason containing credential-like pattern 'api_key' \
                     must be redacted; got: {reason}"
                );
            }
            other => panic!("expected InfusionError::PluginCallFailed; got {:?}", other),
        }
    }

    /// Unit test for `redact_if_sensitive` directly.
    #[test]
    fn test_redact_if_sensitive_unix_path() {
        assert_eq!(
            redact_if_sensitive("/home/user/plugins/x.prx"),
            "<redacted>",
            "Unix path must be redacted"
        );
    }

    #[test]
    fn test_redact_if_sensitive_windows_path() {
        assert_eq!(
            redact_if_sensitive("C:\\Users\\admin\\plugins\\x.prx"),
            "<redacted>",
            "Windows path must be redacted"
        );
    }

    #[test]
    fn test_redact_if_sensitive_safe_string_passes_through() {
        let safe = "plugin 'threat_intel' timed out after 5000ms";
        assert_eq!(
            redact_if_sensitive(safe),
            safe,
            "Safe string must pass through verbatim"
        );
    }
}
