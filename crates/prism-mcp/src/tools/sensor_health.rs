//! Sensor health tools: check_sensor_health, get_diagnostics (BC-2.13.*).
//!
//! These tools query the connectivity and health status of configured sensors.
//! Results are sensor-originated and therefore classified as `untrusted_external`
//! (BC-2.09.005), wrapped in ResponseEnvelope (BC-2.09.008).
//!
//! Injection defense (BC-2.09.001) applies: `injection_scanner.scan_all()` must
//! run before any QueryEngine or sensor connectivity check.

/// Check the connectivity and authentication status of all configured sensors.
///
/// Entry point for the `check_sensor_health` MCP tool.
pub async fn tool_check_sensor_health() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_check_sensor_health — injection scan → sensor connectivity probe → ResponseEnvelope")
}

/// Retrieve diagnostic information for a specific sensor or all sensors.
///
/// Entry point for the `get_diagnostics` MCP tool.
pub async fn tool_get_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_get_diagnostics — injection scan → sensor diagnostics query → ResponseEnvelope")
}
