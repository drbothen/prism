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
//!
//! Note: Full tool handler implementations require Arc<QueryEngine> wiring via
//! PrismServer (ADR-022 §F). These standalone functions are stubs that will be
//! superseded by the `#[tool_router]` macro block on PrismServer once rmcp is
//! wired as a workspace dependency (OQ-1 in the story spec).

use std::io::{Error, ErrorKind};

/// Structured "not yet wired" error for tool handlers pending Arc<QueryEngine> wiring.
///
/// These handlers will be superseded by `#[tool_router]` PrismServer methods once
/// rmcp is wired. The error is structured (not a panic) per POL-12 / AC-10.
fn not_yet_wired(tool: &str) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(Error::new(
        ErrorKind::Unsupported,
        format!(
            "Tool '{tool}' requires Arc<QueryEngine> wiring via PrismServer. \
             Wire rmcp workspace dependency (OQ-1) to complete this tool handler."
        ),
    ))
}

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
///
/// # Implementation Status
/// Full implementation is in `PrismServer::tool_query` (method on the
/// `#[tool_router]` impl block) once rmcp is wired as a workspace dep.
pub async fn tool_query() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_wired("query"))
}

/// Explain the execution plan for a PrismQL query without executing it.
///
/// Entry point for the `explain_query` MCP tool.
pub async fn tool_explain_query() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_wired("explain_query"))
}

/// Create a named PrismQL alias (stored query shorthand).
///
/// Entry point for the `create_alias` MCP tool.
pub async fn tool_create_alias() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_wired("create_alias"))
}

/// List all named PrismQL aliases for the calling client.
///
/// Entry point for the `list_aliases` MCP tool.
pub async fn tool_list_aliases() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_wired("list_aliases"))
}

/// Delete a named PrismQL alias.
///
/// Entry point for the `delete_alias` MCP tool.
pub async fn tool_delete_alias() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_wired("delete_alias"))
}

/// Explain what a named alias expands to, without executing it.
///
/// Entry point for the `explain_alias` MCP tool.
pub async fn tool_explain_alias() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err(not_yet_wired("explain_alias"))
}
