// S-1.12: add_sensor_spec MCP tool logic.
// BC-2.16.008: Upload a New Sensor Spec at Runtime.
// E-SPEC-002: filesystem write failure with path and OS error.
//
// STUB — implementation not yet written. Tests in hot_reload_tests.rs will fail
// until implementation exists (Red Gate).

use std::path::Path;

use crate::config_manager::ConfigManager;
use crate::error::SpecEngineError;
use crate::types::{AddSensorSpecArgs, AddSensorSpecResult};

/// STUB: Process an add_sensor_spec request.
///
/// # Contract (BC-2.16.008)
/// - Parse the spec_toml as TOML
/// - Validate using the same pipeline as startup loading (BC-2.16.009)
/// - If validation fails: return ValidationFailed with all errors; NO file written
/// - If sensor_id already exists: return ConfirmationRequired with a confirmation token
/// - If new sensor and validation passes (not dry_run):
///   - Write atomically: temp file + fsync + rename to {spec_dir}/{sensor_id}.sensor.toml
///   - If write fails: return WriteError with E-SPEC-002 (path + OS error)
///   - Trigger internal reload_config to pick up the new spec
///   - Return Added with registered table descriptors
/// - If dry_run: return DryRun with validation results and table preview; no file written
/// - Every invocation is audit-logged (DI-004)
///
/// # Invariants
/// - Spec is validated BEFORE any file write (invalid spec never reaches disk)
/// - File write is atomic (temp file + fsync + rename) — no partial spec files
/// - sensor_id extracted from parsed spec [sensor] section (no separate parameter)
pub fn add_sensor_spec(
    _manager: &ConfigManager,
    _spec_dir: &Path,
    _args: AddSensorSpecArgs,
) -> Result<AddSensorSpecResult, SpecEngineError> {
    unimplemented!("S-1.12: add_sensor_spec not yet implemented — Red Gate stub")
}

/// STUB: Parse and validate a TOML spec string.
/// Returns the parsed SensorSpec or a list of validation errors.
pub fn parse_and_validate_spec_toml(
    _toml_content: &str,
    _source_path: &str,
) -> Result<crate::types::SensorSpec, Vec<crate::types::ValidationError>> {
    unimplemented!("S-1.12: parse_and_validate_spec_toml not yet implemented — Red Gate stub")
}

/// STUB: Generate a write-gate confirmation token for updating an existing spec.
/// Token is a random string that the caller must echo back to confirm the update.
pub fn generate_confirmation_token(_sensor_id: &str) -> String {
    unimplemented!("S-1.12: generate_confirmation_token not yet implemented — Red Gate stub")
}
