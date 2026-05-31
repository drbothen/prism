// S-1.12: list_sensor_specs MCP tool logic.
// BC-2.16.010: List Loaded Sensor Specs with Table Schemas and Status.

use crate::{
    config_manager::ConfigManager,
    error::SpecEngineError,
    types::{
        ClientStatus, ListSensorSpecsArgs, ListSensorSpecsResult, SensorSpecEntry, SpecStatus,
        sensor_table_descriptor_from_table_spec,
    },
};

/// Return all loaded sensor specs from the current ConfigSnapshot.
///
/// # Contract (BC-2.16.010)
/// - Reads the current ConfigSnapshot lock-free via ConfigManager::load()
/// - Returns all loaded specs with tables, source count, and status
/// - Includes failed_specs with status = FailedValidation
/// - If sensor_id filter provided: returns only that spec (empty list if not found)
/// - If client_id provided: includes per-spec client_status
/// - Read-only: does not modify any state
pub fn list_sensor_specs(
    manager: &ConfigManager,
    args: ListSensorSpecsArgs,
) -> Result<ListSensorSpecsResult, SpecEngineError> {
    let snapshot = manager.load();

    let mut specs: Vec<SensorSpecEntry> = Vec::new();

    // Add loaded specs
    for (sensor_id, spec) in &snapshot.sensor_specs {
        // Apply sensor_id filter if provided
        if args.sensor_id.as_deref().is_some_and(|f| sensor_id != f) {
            continue;
        }

        let client_status = args.client_id.as_ref().map(|_| ClientStatus::Configured);

        // Convert spec_parser::SensorSpec fields to SensorSpecEntry (MCP wire type).
        // ADR-030 §D7: auth_type is AuthType enum → use .as_str() for wire String.
        // tables is Vec<TableSpec> → convert to Vec<SensorTableDescriptor> on-demand.
        // Pass sensor_id so the converter can produce fully-qualified table names
        // ("sensor_id.table_name") — TableSpec stores unqualified names from TOML.
        let wire_tables = spec
            .tables
            .iter()
            .map(|t| sensor_table_descriptor_from_table_spec(sensor_id.as_str(), t))
            .collect();
        specs.push(SensorSpecEntry {
            sensor_id: sensor_id.clone(),
            name: spec.name.clone(),
            version: spec.version.clone(),
            auth_type: spec.auth_type.as_str().to_string(),
            base_url: spec.base_url.clone(),
            tables: wire_tables,
            status: SpecStatus::Loaded,
            client_status,
        });
    }

    // Add failed specs
    for sensor_id in snapshot.failed_specs.keys() {
        // Apply sensor_id filter if provided
        if args.sensor_id.as_deref().is_some_and(|f| sensor_id != f) {
            continue;
        }

        let client_status = args.client_id.as_ref().map(|_| ClientStatus::NotConfigured);

        specs.push(SensorSpecEntry {
            sensor_id: sensor_id.clone(),
            name: sensor_id.clone(), // name may not be known for failed specs
            version: String::new(),
            auth_type: String::new(),
            base_url: String::new(),
            tables: Vec::new(),
            status: SpecStatus::FailedValidation,
            client_status,
        });
    }

    let total_specs = specs.len();
    let total_tables: usize = specs.iter().map(|s| s.tables.len()).sum();

    Ok(ListSensorSpecsResult {
        specs,
        total_specs,
        total_tables,
    })
}
