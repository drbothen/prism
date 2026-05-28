//! Config management tools: reload_config, add_sensor_spec, list_sensor_specs,
//! validate_config, list_capabilities (BC-2.13.*).
//!
//! These tools interact with `prism-spec-engine::ConfigManager` and
//! `parse_spec_directory`. They are administrative — results are internal
//! metadata (trust_level: "internal") rather than sensor-originated data.
//!
//! Injection defense (BC-2.09.001) still applies: `injection_scanner.scan_all()`
//! before any ConfigManager call (spec names and TOML content are attacker-
//! controlled inputs in the MCP context).

/// Hot-reload the running configuration from disk.
///
/// Entry point for the `reload_config` MCP tool.
pub async fn tool_reload_config() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_reload_config — injection scan → ConfigManager::reload → ResponseEnvelope")
}

/// Add or update a sensor spec from a TOML string.
///
/// Entry point for the `add_sensor_spec` MCP tool.
pub async fn tool_add_sensor_spec() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_add_sensor_spec — injection scan → validate TOML → ConfigManager::upsert_spec → ResponseEnvelope")
}

/// List all currently loaded sensor specs with their metadata.
///
/// Entry point for the `list_sensor_specs` MCP tool.
pub async fn tool_list_sensor_specs() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_list_sensor_specs — injection scan → ConfigManager::list_specs → ResponseEnvelope")
}

/// Validate a sensor spec TOML string without loading it.
///
/// Entry point for the `validate_config` MCP tool.
pub async fn tool_validate_config() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_validate_config — injection scan → parse_spec_directory dry-run → ResponseEnvelope")
}

/// List capabilities available for the calling client's feature flags.
///
/// Entry point for the `list_capabilities` MCP tool.
pub async fn tool_list_capabilities() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_list_capabilities — injection scan → ClientCapabilities lookup → ResponseEnvelope")
}
