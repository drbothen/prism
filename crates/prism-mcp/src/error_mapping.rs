//! PrismError → MCP error code mapping (ADR-022 §F).
//!
//! Every `PrismError` variant maps to a structured MCP error code per the
//! error-taxonomy.md table. This module is pure (BC-2.10.007 purity classification):
//! deterministic variant → code mapping, no I/O.
//!
//! MCP error codes used:
//! - `-32602` Invalid params  — parse errors, missing required fields, validation failures
//! - `-32003` NotImplemented  — write not supported for sensor, prism-operations not merged
//! - `-32002` Forbidden       — feature flag denied, permission denied, injection detected
//! - `-32001` Timeout         — query execution timeout
//! - `-32000` Internal error  — all other PrismError variants (audit log has detail)

use prism_core::error::PrismError;

/// Map a `PrismError` to an MCP-compatible error representation.
///
/// Returns `(code, message)` where `code` is the JSON-RPC error code and
/// `message` is the human-readable description suitable for MCP client display.
///
/// The caller wraps this in the rmcp `McpError::custom(code, message)` call.
/// The signature uses `(i32, String)` so the stub compiles without the rmcp dep.
/// Implementer replaces return type with `McpError` once rmcp is wired.
///
/// Per ADR-022 §F error mapping table.
pub fn map_prism_error(_err: PrismError) -> (i32, String) {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: map_prism_error — implement full PrismError variant → (code, message) match per ADR-022 §F + error-taxonomy.md")
}

/// MCP error code constants per ADR-022 §F.
pub mod codes {
    /// Invalid parameters — parse errors, missing required fields.
    pub const INVALID_PARAMS: i32 = -32602;
    /// Feature not implemented — write not supported, prism-operations not merged.
    pub const NOT_IMPLEMENTED: i32 = -32003;
    /// Forbidden — feature flag denied, permission denied, injection detected.
    pub const FORBIDDEN: i32 = -32002;
    /// Timeout — query execution exceeded configured limit.
    pub const TIMEOUT: i32 = -32001;
    /// Internal error — all other variants; audit log has detail.
    pub const INTERNAL_ERROR: i32 = -32000;
}
