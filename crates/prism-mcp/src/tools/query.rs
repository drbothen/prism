//! Query tool handlers: query, explain_query, alias CRUD (BC-2.13.*).
//!
//! All handlers follow the injection-first template (BC-2.09.001, ADR-022 §F):
//!   1. `injection_scanner.scan_all(&params.raw_inputs())?`
//!   2. Validate required fields → `-32602` on missing/invalid
//!   3. Execute domain logic via Arc<QueryEngine>
//!   4. Wrap result in `ResponseEnvelope::new(result)` (BC-2.09.008)
//!
//! Tool descriptions include mandatory security sections via `ToolDescriptionRegistrar`
//! (BC-2.09.006): DATA SOURCE, DATA TRUST LEVEL, SECURITY NOTE.

/// Execute a PrismQL query against configured sensor data sources.
///
/// Entry point for the `query` MCP tool (BC-2.13.001 / AC-2 / AC-3 / AC-4).
///
/// # Injection Defense (BC-2.09.001 — NON-NEGOTIABLE)
/// `injection_scanner.scan_all()` MUST be called before `engine.execute()`.
/// Never bypass, even for trusted callers.
///
/// # Response Envelope (BC-2.09.008)
/// All results are wrapped in `ResponseEnvelope` with `_meta.trust_level` and
/// `_meta.safety_flags` populated by `SafetyEnvelopeBuilder`.
pub async fn tool_query() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_query — injection scan → validate params → engine.execute → ResponseEnvelope")
}

/// Explain the execution plan for a PrismQL query without executing it.
///
/// Entry point for the `explain_query` MCP tool.
pub async fn tool_explain_query() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_explain_query — injection scan → validate params → engine.explain")
}

/// Create a named PrismQL alias (stored query shorthand).
///
/// Entry point for the `create_alias` MCP tool.
pub async fn tool_create_alias() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_create_alias — injection scan → validate alias name + body → store")
}

/// List all named PrismQL aliases for the calling client.
///
/// Entry point for the `list_aliases` MCP tool.
pub async fn tool_list_aliases() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_list_aliases — injection scan → list stored aliases")
}

/// Delete a named PrismQL alias.
///
/// Entry point for the `delete_alias` MCP tool.
pub async fn tool_delete_alias() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_delete_alias — injection scan → validate alias name → delete")
}

/// Explain what a named alias expands to, without executing it.
///
/// Entry point for the `explain_alias` MCP tool.
pub async fn tool_explain_alias() -> Result<(), Box<dyn std::error::Error>> {
    todo!("S-5.01-FOLLOWUP-MCP-BOOT: tool_explain_alias — injection scan → validate alias name → expand + describe")
}
