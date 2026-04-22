// S-1.08: list_capabilities Logic — STUB (Red Gate)
//
// All function bodies are `unimplemented!()`.  The implementer must fill them
// in to make the test suite green.
//
// Story:  S-1.08 — prism-security: Feature Flags (P0 Core)
// BC:     BC-2.04.006 (list_capabilities meta-tool)
//
// The `list_capabilities` MCP tool is always registered (not gated by any
// feature flag) and returns a full capability matrix for AI agent introspection.

use std::collections::BTreeMap;

use prism_core::capability::ClientCapabilities;
use prism_core::error::PrismError;

use crate::feature_flag::CompileTimeGate;

// ─────────────────────────────────────────────────────────────
// CapabilityStatus
// ─────────────────────────────────────────────────────────────

/// The full status of a single capability path for a specific client
/// (BC-2.04.006 postconditions).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityStatus {
    /// Combined enablement result (both tiers).
    pub enabled: bool,
    /// Whether the Cargo feature is present in this binary.
    pub compile_time: bool,
    /// Whether the runtime TOML flag permits this capability for the client.
    pub runtime: bool,
    /// Human-readable explanation when disabled (e.g., "Feature not compiled
    /// (crowdstrike-write)" or "Not enabled in client config").
    /// `None` when `enabled == true`.
    pub reason: Option<String>,
}

// ─────────────────────────────────────────────────────────────
// CapabilityMatrixEntry
// ─────────────────────────────────────────────────────────────

/// A single row in the capability matrix returned by `list_capabilities`.
#[derive(Clone, Debug)]
pub struct CapabilityMatrixEntry {
    /// The capability path (e.g., `"sensor.crowdstrike.containment"`).
    pub capability: String,
    /// Status for a specific client, or per-client breakdown if `client_id`
    /// was null.
    pub status: CapabilityStatus,
}

// ─────────────────────────────────────────────────────────────
// ListCapabilitiesQuery
// ─────────────────────────────────────────────────────────────

/// Input to the `list_capabilities` logic.
#[derive(Clone, Debug)]
pub struct ListCapabilitiesQuery {
    /// Optional client ID filter. When `None`, returns a per-client breakdown
    /// for all clients.
    pub client_id: Option<String>,
}

// ─────────────────────────────────────────────────────────────
// ListCapabilitiesEngine
// ─────────────────────────────────────────────────────────────

/// Implements the `list_capabilities` meta-tool logic (BC-2.04.006).
///
/// Always available — not gated by any feature flag.
pub struct ListCapabilitiesEngine {
    /// Per-client capability maps.
    client_capabilities: BTreeMap<String, ClientCapabilities>,
    /// Compile-time gate status for each write code family.
    /// Key: canonical capability prefix (e.g., `"sensor.crowdstrike"`).
    /// Value: whether the corresponding Cargo feature is compiled in.
    compile_gates: BTreeMap<String, CompileTimeGate>,
}

impl ListCapabilitiesEngine {
    /// Construct the engine.
    pub fn new(
        client_capabilities: BTreeMap<String, ClientCapabilities>,
        compile_gates: BTreeMap<String, CompileTimeGate>,
    ) -> Self {
        unimplemented!("S-1.08: ListCapabilitiesEngine::new — implement construction")
    }

    /// Execute the `list_capabilities` query (BC-2.04.006 postconditions).
    ///
    /// - If `query.client_id` is `Some(id)`, returns the capability matrix for
    ///   that specific client.  Returns `PrismError::ConfigValidationFailed`
    ///   ("Client '{id}' not found in configuration") if unknown.
    /// - If `query.client_id` is `None`, returns the global matrix with all
    ///   configured clients.
    pub fn execute(
        &self,
        query: &ListCapabilitiesQuery,
    ) -> Result<Vec<CapabilityMatrixEntry>, PrismError> {
        unimplemented!("S-1.08: ListCapabilitiesEngine::execute — implement list_capabilities logic")
    }

    /// Check whether the `list_capabilities` result is consistent with the
    /// `tools/list` response (invariant from BC-2.04.006).
    ///
    /// Returns `true` if every capability listed as `enabled` maps to a tool
    /// that would appear in the `tools/list` response, and vice versa.
    pub fn is_consistent_with_tools_list(&self, tool_names: &[String]) -> bool {
        unimplemented!(
            "S-1.08: ListCapabilitiesEngine::is_consistent_with_tools_list — implement consistency check"
        )
    }
}
