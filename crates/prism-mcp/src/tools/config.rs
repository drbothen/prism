//! Config management tool handlers — implemented as PrismServer methods (BC-2.13.*).
//!
//! All config management tool handlers live on `PrismServer` in `crate::server` via the
//! `#[tool_router(server_handler)]` macro block:
//!
//! - `PrismServer::reload_config` — hot-reload running configuration from disk
//! - `PrismServer::add_sensor_spec` — add or update sensor spec from TOML string
//! - `PrismServer::list_sensor_specs` — list currently loaded sensor specs
//! - `PrismServer::validate_config` — validate sensor spec TOML without loading
//! - `PrismServer::list_capabilities` — list capabilities for calling client's feature flags
//!
//! These tools interact with the spec engine (`prism-spec-engine::ConfigManager`).
//! They are administrative — results are internal metadata (trust_level: "internal").
//!
//! Injection defense (BC-2.09.001) still applies: spec names and TOML content are
//! attacker-controlled inputs in the MCP context and are scanned before use.
