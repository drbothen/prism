//! Sensor health tools: check_sensor_health, get_diagnostics (BC-2.13.*).
//!
//! These tools query the connectivity and health status of configured sensors.
//! Results are sensor-originated and therefore classified as `untrusted_external`
//! (BC-2.09.005), wrapped in ResponseEnvelope (BC-2.09.008).
//!
//! Injection defense (BC-2.09.001) applies: `injection_scanner.scan_all()` must
//! run before any QueryEngine or sensor connectivity check.
//!
//! Note: Full implementation requires Arc<QueryEngine> wiring via PrismServer
//! (ADR-022 §F). The standalone functions here are stubs that will be superseded
//! by the `#[tool_router]` macro block on PrismServer once rmcp is wired (OQ-1).

use std::io::{Error, ErrorKind};

/// Check the connectivity and authentication status of all configured sensors.
///
/// Entry point for the `check_sensor_health` MCP tool.
///
/// Full implementation is in `PrismServer::tool_check_sensor_health` once
/// rmcp is wired as a workspace dep (OQ-1).
pub async fn tool_check_sensor_health() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(Box::new(Error::new(
        ErrorKind::Unsupported,
        "Tool 'check_sensor_health' requires Arc<QueryEngine> wiring via PrismServer. \
         Wire rmcp workspace dependency (OQ-1) to complete this tool handler.",
    )))
}

/// Retrieve diagnostic information for a specific sensor or all sensors.
///
/// Entry point for the `get_diagnostics` MCP tool.
///
/// Full implementation is in `PrismServer::tool_get_diagnostics` once
/// rmcp is wired as a workspace dep (OQ-1).
pub async fn tool_get_diagnostics() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(Box::new(Error::new(
        ErrorKind::Unsupported,
        "Tool 'get_diagnostics' requires Arc<QueryEngine> wiring via PrismServer. \
         Wire rmcp workspace dependency (OQ-1) to complete this tool handler.",
    )))
}
