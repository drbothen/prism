//! Error types for the ocsf-proto-gen crate.

use std::path::PathBuf;

/// Errors that can occur during OCSF proto generation.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to read or parse the OCSF schema JSON.
    #[error("schema error: {0}")]
    Schema(String),

    /// A requested event class was not found in the schema.
    #[error("class '{name}' not found in schema (available: {available})")]
    ClassNotFound { name: String, available: String },

    /// Failed to write generated proto files.
    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to read a file from disk.
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },

    /// JSON parse error with context.
    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// Network error during schema download.
    #[cfg(feature = "download")]
    #[error("download failed: {0}")]
    Download(String),

    /// Proto generation error.
    #[error("codegen error: {0}")]
    Codegen(String),
}

/// Convenience alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
