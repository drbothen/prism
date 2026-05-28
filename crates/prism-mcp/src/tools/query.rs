//! Query tool handlers — implemented as PrismServer methods (BC-2.13.001).
//!
//! All query tool handlers live on `PrismServer` in `crate::server` via the
//! `#[tool_router(server_handler)]` macro block:
//!
//! - `PrismServer::query` — execute PrismQL query
//! - `PrismServer::explain_query` — explain query plan without execution
//! - `PrismServer::create_alias` — create named PrismQL alias
//! - `PrismServer::list_aliases` — list named aliases for calling client
//! - `PrismServer::delete_alias` — delete named alias
//! - `PrismServer::explain_alias` — explain alias expansion without execution
//!
//! Each handler calls `scan_inputs(self.injection_scanner, ...)` before any
//! domain logic per BC-2.09.001 (NON-NEGOTIABLE, ADR-022 §F).
