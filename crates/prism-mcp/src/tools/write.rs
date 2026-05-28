//! Write tool handlers — implemented as PrismServer methods (BC-2.10.003).
//!
//! All write tool handlers live on `PrismServer` in `crate::server` via the
//! `#[tool_router(server_handler)]` macro block:
//!
//! - `PrismServer::confirm_action` — confirm irreversible write by confirmation token
//!
//! The handler calls `scan_inputs(self.injection_scanner, ...)` before any
//! domain logic per BC-2.09.001 (NON-NEGOTIABLE, ADR-022 §F).
//!
//! # Feature Flag (BC-2.10.003)
//!
//! Tool visibility in `tools/list` is stateless — `confirm_action` appears in the
//! tool list if the feature flag is enabled for ANY client. Per-call, the flag
//! is checked via WriteExecutor and returns a forbidden response if the calling
//! client does not have write capability.
