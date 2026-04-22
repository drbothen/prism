// S-1.12: list_sensor_specs MCP tool logic.
// BC-2.16.010: List Loaded Sensor Specs with Table Schemas and Status.
//
// STUB — implementation not yet written. Tests in hot_reload_tests.rs will fail
// until implementation exists (Red Gate).

use crate::config_manager::ConfigManager;
use crate::error::SpecEngineError;
use crate::types::{ListSensorSpecsArgs, ListSensorSpecsResult};

/// STUB: Return all loaded sensor specs from the current ConfigSnapshot.
///
/// # Contract (BC-2.16.010)
/// - Reads the current ConfigSnapshot lock-free via ConfigManager::load()
/// - For each loaded SensorSpec: returns sensor_id, name, version, auth_type,
///   base_url, tables (with column schemas and OCSF mappings), and status
/// - status is one of:
///   - "loaded" (available): spec loaded, credentials configured for at least one client
///   - "no_credentials": spec loaded but no client has credentials (DEC-036)
///   - "failed_validation": spec failed validation (from failed_specs in snapshot)
///   - "validation_warnings": spec loaded with warnings
/// - If client_id provided: includes per-spec client_status (configured | not_configured)
/// - If sensor_id filter provided: returns only that spec (empty list if not found — not error)
/// - Empty directory: empty list (no error)
/// - Read-only: does not modify any state
/// - Always visible: no capability gating
/// - Response uses structuredContent for machine-parseable output
///
/// # Invariants
/// - Returns empty list (not error) for unknown sensor_id or empty directory
/// - Does not modify any state (pure read from ConfigSnapshot)
pub fn list_sensor_specs(
    _manager: &ConfigManager,
    _args: ListSensorSpecsArgs,
) -> Result<ListSensorSpecsResult, SpecEngineError> {
    unimplemented!("S-1.12: list_sensor_specs not yet implemented — Red Gate stub")
}
