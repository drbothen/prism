// S-1.12: Error types for prism-spec-engine.
// E-SPEC-002: filesystem write failure (BC-2.16.008)
// E-RELOAD-001..004: reload error conditions (BC-2.16.005)
// S-3.1.05: OrgId-scoped store errors (BC-3.1.001 AC-1, AC-3)

use prism_core::OrgSlug;
use thiserror::Error;

/// Top-level spec engine error.
#[derive(Debug, Error)]
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
}
