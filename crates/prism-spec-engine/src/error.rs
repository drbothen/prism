// S-1.12: Error types for prism-spec-engine.
// E-SPEC-002: filesystem write failure (BC-2.16.008)
// E-RELOAD-001..004: reload error conditions (BC-2.16.005)

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
}
