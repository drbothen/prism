//! Sensor health tool handlers — implemented as PrismServer methods (BC-2.13.*).
//!
//! All sensor health tool handlers live on `PrismServer` in `crate::server` via the
//! `#[tool_router(server_handler)]` macro block:
//!
//! - `PrismServer::check_sensor_health` — check connectivity + auth status of sensors
//! - `PrismServer::get_diagnostics` — retrieve diagnostic information for sensors
//!
//! Results are sensor-originated and therefore classified as `untrusted_external`
//! (BC-2.09.005), wrapped in ResponseEnvelope (BC-2.09.008).
//!
//! Each handler calls `scan_inputs(self.injection_scanner, ...)` before any
//! domain logic per BC-2.09.001 (NON-NEGOTIABLE, ADR-022 §F).
