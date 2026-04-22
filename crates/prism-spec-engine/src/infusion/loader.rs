//! TOML infusion spec parser and validator.
//!
//! Loads `*.infusion.toml` files from `{config_dir}/infusions/`, validates each spec,
//! and returns `InfusionSpec` values for registration into `InfusionRegistry`.
//!
//! # Validation rules (BC-2.19.001)
//! - `infusion_id` must be present and non-empty.
//! - At least one `[[infusion.fields]]` entry required.
//! - `source.type` must be one of: `maxmind_mmdb`, `csv`, `json_lookup`, `plugin`.
//! - Credential references must use reference-based model (no inline values, AD-017).
//! - `pipe_stage.adds_columns` must match the `[[infusion.fields]]` names.
//! - On validation error: return `Err` — do NOT partially register.
//!
//! # Credential redaction (INV-INFUSE-005 / AD-017)
//! Credential values MUST NOT appear in any error message or log output.
//!
//! # Stub
//! All methods are `unimplemented!()` — implementation in S-1.14.

use prism_core::InfusionError;

use super::InfusionSpec;

/// Loads and validates infusion specs from a directory.
pub struct InfusionLoader {
    _config_dir: String,
}

impl InfusionLoader {
    /// Create a new `InfusionLoader` for the given config directory.
    pub fn new(config_dir: impl Into<String>) -> Self {
        InfusionLoader {
            _config_dir: config_dir.into(),
        }
    }

    /// Parse a single TOML string into an `InfusionSpec`.
    ///
    /// Returns `Ok(InfusionSpec)` or `Err(InfusionError)` — never panics.
    /// Validation failures return descriptive errors without credential values.
    pub fn parse(toml_input: &str, source_path: &str) -> Result<InfusionSpec, InfusionError> {
        unimplemented!(
            "InfusionLoader::parse — implement in S-1.14 (BC-2.19.001)"
        )
    }

    /// Load all `*.infusion.toml` files from `{config_dir}/infusions/`.
    ///
    /// Returns (specs, errors): valid specs continue loading even if others fail.
    /// Invalid specs produce `InfusionError` values but do not block valid specs.
    pub fn load_all(&self) -> (Vec<InfusionSpec>, Vec<InfusionError>) {
        unimplemented!(
            "InfusionLoader::load_all — implement in S-1.14 (BC-2.19.001)"
        )
    }

    /// Validate that `pipe_stage.adds_columns` matches the `[[infusion.fields]]` names.
    ///
    /// Returns `Ok(())` or a list of mismatched names.
    pub fn validate_pipe_stage_columns(spec: &InfusionSpec) -> Result<(), InfusionError> {
        unimplemented!(
            "InfusionLoader::validate_pipe_stage_columns — implement in S-1.14 (BC-2.19.001)"
        )
    }

    /// Validate that all credential entries use the reference-based model (no inline values).
    ///
    /// Returns `Ok(())` or `Err` — credential values MUST NOT appear in the error (INV-INFUSE-005).
    pub fn validate_credentials(spec: &InfusionSpec) -> Result<(), InfusionError> {
        unimplemented!(
            "InfusionLoader::validate_credentials — implement in S-1.14 (BC-2.19.005)"
        )
    }
}
