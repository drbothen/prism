//! `prism-customer-config` — TOML schema, parser, and startup validator for
//! Prism customer configuration files.
//!
//! # Public API
//!
//! ```text
//! load_and_validate(dir: &Path) -> Result<Vec<CustomerConfig>, Vec<ConfigError>>
//! ```
//!
//! Returns `Ok(configs)` when all files in `dir` pass validation, or
//! `Err(errors)` containing ALL errors from ALL files (multi-error, not fail-fast).
//!
//! # Crate Layout
//!
//! | Module | Classification | Purpose |
//! |--------|---------------|---------|
//! | `schema` | pure-core | Serde structs for TOML schema |
//! | `error` | pure-core | `ConfigError` enum + Display |
//! | `credential_check` | pure-core | Recursive credential heuristic scanner |
//! | `validator` | effectful-shell | Multi-error validation pass (disk I/O) |
//!
//! Architecture: SS-06 (Client Configuration), ADR-010.

use std::path::Path;

pub mod credential_check;
pub mod error;
pub mod schema;
pub mod validator;

pub use error::ConfigError;
pub use schema::{CustomerConfig, DtuBlock, DtuData, SharedInfra};

/// Load and validate all customer config files from the given directory.
///
/// Scans `dir` for `*.toml` files, parses each one, and runs the full
/// multi-error validation pass (schema version, credential heuristics,
/// structural rules, cross-file duplicate checks).
///
/// Returns:
/// - `Ok(configs)` if all files pass (empty vec if no `.toml` files found)
/// - `Err(errors)` if any validation error is found (contains ALL errors)
///
/// Traces to: BC-3.3.004, BC-3.3.003, BC-3.3.002, BC-3.3.001
pub fn load_and_validate(_dir: &Path) -> Result<Vec<CustomerConfig>, Vec<ConfigError>> {
    todo!("load_and_validate — implemented in Red Gate phase")
}
