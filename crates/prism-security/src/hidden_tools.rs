// S-1.08: Hidden Tools Registry — STUB (Red Gate)
//
// All function bodies are `unimplemented!()`.  The implementer must fill them
// in to make the test suite green.
//
// Story:  S-1.08 — prism-security: Feature Flags (P0 Core)
// BC:     BC-2.04.005 (Hidden Tools Pattern — Stateless Tool List)
//
// Architecture compliance rules:
//   - Hidden tools are NOT compiled out — they exist in the binary but are
//     excluded from the tools/list response at runtime (BC-2.04.005).
//   - The tools/list response is stateless: same regardless of prior calls.
//   - Write tools absent for ALL clients are completely absent from tools/list.
//   - Write tools enabled for at least one client appear in tools/list.
//   - Per-client denial is enforced at invocation time via E-FLAG-001.

use std::collections::BTreeMap;

use prism_core::capability::{CapabilityPath, ClientCapabilities};

// ─────────────────────────────────────────────────────────────
// ToolKind
// ─────────────────────────────────────────────────────────────

/// Classifies a registered MCP tool as read-only or write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolKind {
    /// Read-only tools always appear in `tools/list`.
    Read,
    /// Write tools are conditionally visible based on runtime capability config.
    Write {
        /// The capability path that gates this write tool
        /// (e.g., `"sensor.crowdstrike.containment"`).
        required_capability: String,
    },
}

// ─────────────────────────────────────────────────────────────
// RegisteredTool
// ─────────────────────────────────────────────────────────────

/// An MCP tool entry in the hidden tools registry.
#[derive(Clone, Debug)]
pub struct RegisteredTool {
    /// Stable MCP tool name (e.g., `"crowdstrike_contain_host"`).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Whether this is a read or write tool.
    pub kind: ToolKind,
}

// ─────────────────────────────────────────────────────────────
// HiddenToolsRegistry
// ─────────────────────────────────────────────────────────────

/// Stateless registry of all MCP tools.
///
/// Produces a `tools/list` response by filtering out write tools that are
/// disabled for ALL configured clients (BC-2.04.005).
///
/// Tools are never compiled out — they are excluded from the response at
/// runtime when no client has the required capability enabled. This is
/// intentional: compile-time removal is done via Cargo features; runtime
/// hiding is a UX layer.
pub struct HiddenToolsRegistry {
    /// All registered tools, keyed by name for fast lookup.
    tools: BTreeMap<String, RegisteredTool>,
}

impl HiddenToolsRegistry {
    /// Construct a registry from a list of tool registrations.
    pub fn new(tools: Vec<RegisteredTool>) -> Self {
        unimplemented!("S-1.08: HiddenToolsRegistry::new — implement registry construction")
    }

    /// Return the `tools/list` response filtered for the given per-client
    /// capability maps.
    ///
    /// Rules (BC-2.04.005 postconditions):
    /// - Read tools: always included.
    /// - Write tools: included if enabled for at least one client in `client_capabilities`.
    ///   Disabled write tools (disabled for ALL clients) are completely absent.
    ///
    /// The response is stateless — same `client_capabilities` always produces
    /// the same list regardless of call order.
    pub fn tools_list(
        &self,
        client_capabilities: &BTreeMap<String, ClientCapabilities>,
    ) -> Vec<&RegisteredTool> {
        unimplemented!("S-1.08: HiddenToolsRegistry::tools_list — implement filtered tool list")
    }

    /// Return a tool by name, regardless of visibility (for invocation routing).
    pub fn get_tool(&self, name: &str) -> Option<&RegisteredTool> {
        unimplemented!("S-1.08: HiddenToolsRegistry::get_tool — implement tool lookup")
    }
}
