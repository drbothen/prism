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
//!
//! Note: Full implementation requires Arc<ConfigManager> wiring via PrismServer
//! (ADR-022 §F). The standalone functions here are stubs that will be superseded
//! by the `#[tool_router]` macro block on PrismServer once rmcp is wired (OQ-1).

use std::io::{Error, ErrorKind};

/// Structured "not yet wired" error for config tool handlers pending Arc<ConfigManager> wiring.
fn not_yet_wired(tool: &str) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(Error::new(
        ErrorKind::Unsupported,
        format!(
            "Tool '{tool}' requires Arc<ConfigManager> wiring via PrismServer. \
             Wire rmcp workspace dependency (OQ-1) to complete this tool handler."
        ),
    ))
}

/// Hot-reload the running configuration from disk.
///
/// Entry point for the `reload_config` MCP tool.
///
/// Full implementation is in `PrismServer::tool_reload_config` once
/// rmcp is wired as a workspace dep (OQ-1).
pub async fn tool_reload_config() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_wired("reload_config"))
}

/// Add or update a sensor spec from a TOML string.
///
/// Entry point for the `add_sensor_spec` MCP tool.
///
/// Full implementation is in `PrismServer::tool_add_sensor_spec` once
/// rmcp is wired as a workspace dep (OQ-1).
pub async fn tool_add_sensor_spec() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_wired("add_sensor_spec"))
}

/// List all currently loaded sensor specs with their metadata.
///
/// Entry point for the `list_sensor_specs` MCP tool.
///
/// Full implementation is in `PrismServer::tool_list_sensor_specs` once
/// rmcp is wired as a workspace dep (OQ-1).
pub async fn tool_list_sensor_specs() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_wired("list_sensor_specs"))
}

/// Validate a sensor spec TOML string without loading it.
///
/// Entry point for the `validate_config` MCP tool.
///
/// Full implementation is in `PrismServer::tool_validate_config` once
/// rmcp is wired as a workspace dep (OQ-1).
pub async fn tool_validate_config() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_wired("validate_config"))
}

/// List capabilities available for the calling client's feature flags.
///
/// Entry point for the `list_capabilities` MCP tool.
///
/// Full implementation is in `PrismServer::tool_list_capabilities` once
/// rmcp is wired as a workspace dep (OQ-1).
pub async fn tool_list_capabilities() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_wired("list_capabilities"))
}
