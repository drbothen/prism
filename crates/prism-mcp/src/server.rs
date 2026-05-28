//! PrismServer — MCP ServerHandler implementation (BC-2.10.001).
//!
//! Holds Arc dependencies for QueryEngine, WriteExecutor, audit, security,
//! and injection scanner. Constructed at boot step 9 (ADR-022 §F).
//!
//! Note: rmcp dependency (version 1.4–1.7) is unresolved at stub time per
//! OQ-1 in the story spec. Types are placeholder until the implementer confirms
//! the exact crate name and API surface at TDD time.

use std::sync::Arc;

/// Placeholder for rmcp McpError until rmcp dependency is wired.
///
/// Will be replaced with `rmcp::McpError` once the workspace dep is pinned.
pub type McpError = Box<dyn std::error::Error + Send + Sync>;

/// PrismServer — rmcp ServerHandler implementation (BC-2.10.001).
///
/// All dependencies injected via Arc to satisfy `Send + Sync` requirements
/// for McpServer (ADR-022 §F). Construction enforces that InjectionScanner
/// is always present (BC-2.09.001 structural requirement — no exempt tool paths).
///
/// Tool handlers live in `crate::tools` and are wired via the `#[tool_router]`
/// macro (BC-2.10.002). Each handler must call `injection_scanner.scan_all()`
/// before any domain logic (BC-2.09.001 invariant, NON-NEGOTIABLE per ADR-022 §F).
pub struct PrismServer {
    // Arc dependencies — wired by implementer at TDD time (ADR-022 §F).
    // Placeholder fields keep the struct compilable before real types exist.
    _placeholder: (),
}

impl PrismServer {
    /// Construct a new PrismServer with all dependencies.
    ///
    /// InjectionScanner is constructed internally from prism-security (S-1.10).
    /// Callers provide the remaining Arc dependencies per ADR-022 §F.
    pub fn new() -> Self {
        todo!("S-5.01-FOLLOWUP-MCP-BOOT: PrismServer::new — wire Arc<QueryEngine>, Arc<WriteExecutor>, Arc<AuditEmitter>, Arc<SecurityConfig>, Arc<InjectionScanner>")
    }

    /// Start the MCP server on stdio transport (BC-2.10.006).
    ///
    /// Blocks until stdin closes or SIGTERM/SIGINT received.
    /// On shutdown: flushes audit buffer and exits cleanly (BC-2.10.010).
    pub async fn serve_stdio(self) -> Result<(), McpError> {
        todo!("S-5.01-FOLLOWUP-MCP-BOOT: serve_stdio — wire rmcp::McpServer + stdio_transport")
    }
}

impl Default for PrismServer {
    fn default() -> Self {
        todo!("S-5.01-FOLLOWUP-MCP-BOOT: PrismServer::default — delegates to new()")
    }
}
