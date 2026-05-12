// S-1.12: Error types for prism-spec-engine.
// E-SPEC-002: filesystem write failure (BC-2.16.008)
// E-RELOAD-001..004: reload error conditions (BC-2.16.005)
// S-3.1.05: OrgId-scoped store errors (BC-3.1.001 AC-1, AC-3)

use prism_core::OrgSlug;
use thiserror::Error;

/// Top-level spec engine error.
///
/// Marked `#[non_exhaustive]` to allow adding variants in future stories
/// without a major version bump (workspace convention, see `prism-core::PrismError`).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SpecEngineError {
    /// E-RELOAD-001: Config file read error (file not found, permission denied)
    #[error("E-RELOAD-001: Failed to read config file '{path}': {os_error}")]
    FileReadError { path: String, os_error: String },

    /// E-RELOAD-002: Validation failed for prism.toml or aliases.toml (Tier 1/2)
    #[error("E-RELOAD-002: Config validation failed: {errors:?}")]
    ValidationFailed { errors: Vec<String> },

    /// E-RELOAD-003: Partial reload — some sensor spec files failed validation
    #[error("E-RELOAD-003: Partial reload: {failed_count} spec(s) failed validation")]
    PartialReloadFailure { failed_count: usize },

    /// E-RELOAD-004: No changes detected (all files match previous hash)
    #[error("E-RELOAD-004: No changes detected; reload is a no-op")]
    NoChangesDetected,

    /// E-SPEC-002: Filesystem write failure with path and OS error (BC-2.16.008)
    #[error("E-SPEC-002: Failed to write spec file '{path}': {os_error}")]
    SpecWriteError { path: String, os_error: String },

    /// TOML parse error
    #[error("TOML parse error in '{path}': {detail}")]
    TomlParseError { path: String, detail: String },

    /// Watcher setup error
    #[error("Filesystem watcher setup failed: {detail}")]
    WatcherError { detail: String },

    // -------------------------------------------------------------------------
    // S-3.1.05 — OrgId-scoped spec store errors (BC-3.1.001)
    // -------------------------------------------------------------------------
    /// E-SPEC-ORG-001: Slug not registered in OrgRegistry (BC-3.1.001 AC-1).
    ///
    /// Returned by `OrgScopedSpecStore::get_spec` when
    /// `OrgRegistry::resolve(slug)` returns `None`.
    #[error("E-SPEC-ORG-001: unknown org slug '{slug}' — not registered in OrgRegistry")]
    UnknownOrg { slug: OrgSlug },

    /// E-SPEC-ORG-002: Org exists but has no spec for the requested sensor (BC-3.1.001 AC-1, EC-002).
    ///
    /// Returned by `OrgScopedSpecStore::get_spec` when the org is known but
    /// `(OrgId, sensor_name)` is absent from the internal store.
    #[error("E-SPEC-ORG-002: org '{slug}' has no spec for sensor '{sensor}'")]
    SensorNotFound { slug: OrgSlug, sensor: String },

    /// E-SPEC-ORG-003: OrgRegistry not yet initialized (BC-3.1.001 AC-3).
    ///
    /// In practice unreachable post-startup (BC-3.1.001 invariant 3), but the
    /// API MUST NOT panic — it returns this error instead.
    #[error("E-SPEC-ORG-003: OrgRegistry not initialized; call is too early in startup sequence")]
    RegistryNotInitialized,

    // -------------------------------------------------------------------------
    // S-PLUGIN-PREREQ-B — PipelineExecutor auth errors (BC-2.16.002, BC-2.01.013)
    // -------------------------------------------------------------------------
    /// E-AUTH-001: Auth token acquisition failed for the given sensor / client.
    ///
    /// Returned by `AuthProvider::acquire_token` implementations when credentials
    /// cannot be resolved (bad config, network failure, invalid auth_type).
    #[error(
        "E-AUTH-001: auth token acquisition failed for sensor '{sensor_id}', client '{client_id}': {detail}"
    )]
    AuthAcquisitionFailed {
        sensor_id: String,
        client_id: String,
        detail: String,
    },

    /// E-AUTH-002: Auth refresh failed — double-401 after token re-acquisition.
    ///
    /// Returned by `PipelineExecutor::execute` when a step returns HTTP 401,
    /// `acquire_token` is called to get a fresh token, and the retry ALSO
    /// returns HTTP 401. Pipeline aborts; no further retries (AC-5 abort condition).
    #[error(
        "E-AUTH-002: auth refresh failed for sensor '{sensor_id}', client '{client_id}': \
         HTTP 401 persisted after token re-acquisition on step '{step_name}'"
    )]
    AuthRefreshFailed {
        sensor_id: String,
        client_id: String,
        step_name: String,
    },

    /// E-HTTP-001: HTTP request failed (non-401 error, e.g., 500, network error).
    ///
    /// Returned by `PipelineExecutor` when a step receives a non-retryable HTTP
    /// error. Pipeline aborts (EC-007).
    #[error(
        "E-HTTP-001: HTTP {status_code} from sensor '{sensor_id}' step '{step_name}': {detail}"
    )]
    HttpRequestFailed {
        sensor_id: String,
        step_name: String,
        status_code: u16,
        detail: String,
    },

    /// E-JSONPATH-001: JSONPath extraction failed — `response_path` did not match
    /// the response structure (E-SPEC-010).
    ///
    /// Step name and path are included for diagnostics (BC-2.16.002 error table).
    #[error(
        "E-JSONPATH-001 (E-SPEC-010): response_path '{path}' did not match response \
         from sensor '{sensor_id}' step '{step_name}'"
    )]
    JsonPathExtractionFailed {
        sensor_id: String,
        step_name: String,
        path: String,
    },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// F-LP2-LOW-001: `AuthAcquisitionFailed` variant is constructible and its
    /// Display output includes sensor_id and client_id for operator diagnostics.
    #[test]
    fn test_auth_acquisition_failed_error_constructs() {
        let err = SpecEngineError::AuthAcquisitionFailed {
            sensor_id: "crowdstrike".to_string(),
            client_id: "test-org".to_string(),
            detail: "credentials store unavailable".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("crowdstrike"),
            "error message must contain sensor_id; got: {msg}"
        );
        assert!(
            msg.contains("test-org"),
            "error message must contain client_id; got: {msg}"
        );
        assert!(
            msg.contains("credentials store unavailable"),
            "error message must contain detail; got: {msg}"
        );
    }
}
