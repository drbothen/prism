//! Write tool handlers: confirm_action (BC-2.10.003, S-3.07).
//!
//! All write operations in Prism flow through `prism query` (SELECT + WRITE pattern).
//! The only direct write tool surface is `confirm_action`, which confirms an
//! irreversible write operation by token after the user has reviewed a WRITE plan.
//!
//! Injection defense (BC-2.09.001) applies here exactly as in query tools —
//! `injection_scanner.scan_all()` before any WriteExecutor call.

/// Confirm an irreversible write operation by confirmation token.
///
/// Entry point for the `confirm_action` MCP tool.
///
/// # Feature Flag (BC-2.10.003)
/// Tool visibility in `tools/list` is stateless — `confirm_action` appears in the
/// tool list if the feature flag is enabled for ANY client. Per-call, the flag
/// is checked via WriteExecutor and returns `McpError::custom(-32002, "Feature flag denied")`
/// if the calling client does not have write capability.
///
/// # Token Expiry (EC-006)
/// If the confirmation token has expired, WriteExecutor returns `TokenExpired`
/// which `map_prism_error` maps to `-32002` with an expiry message.
pub async fn tool_confirm_action() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_confirm_action — injection scan → validate token → write_executor.execute → ResponseEnvelope")
}
