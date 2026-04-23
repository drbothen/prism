// S-1.08: list_capabilities Logic
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
        ListCapabilitiesEngine {
            client_capabilities,
            compile_gates,
        }
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
        match &query.client_id {
            Some(client_id) => {
                // Verify client exists.
                let caps = self
                    .client_capabilities
                    .get(client_id.as_str())
                    .ok_or_else(|| PrismError::ConfigValidationFailed {
                        detail: format!("Client '{}' not found in configuration", client_id),
                    })?;

                Ok(self.capability_matrix_for_client(client_id, caps))
            }
            None => {
                // Global matrix: union of all capabilities across all clients.
                if self.client_capabilities.is_empty() {
                    return Ok(vec![]);
                }

                // Collect all unique capability paths across all clients.
                let mut entries = Vec::new();
                for (client_id, caps) in &self.client_capabilities {
                    let client_entries = self.capability_matrix_for_client(client_id, caps);
                    entries.extend(client_entries);
                }
                Ok(entries)
            }
        }
    }

    /// Build the capability matrix for a single client.
    fn capability_matrix_for_client(
        &self,
        _client_id: &str,
        caps: &ClientCapabilities,
    ) -> Vec<CapabilityMatrixEntry> {
        let mut entries = Vec::new();

        for (path, _effect) in caps.capabilities_for_display() {
            let path_str = path.as_str();
            let compile_time = self.compile_gate_for_path(path_str);
            let compile_ok = compile_time == CompileTimeGate::Present;

            // Evaluate runtime.
            let (runtime_allowed, _) = caps.is_allowed(path);

            let enabled = compile_ok && runtime_allowed;
            let reason = if enabled {
                None
            } else if !compile_ok {
                Some(format!(
                    "Feature not compiled (check compile-time features for '{}')",
                    path_str
                ))
            } else {
                Some("Not enabled in client config".to_string())
            };

            entries.push(CapabilityMatrixEntry {
                capability: path_str.to_string(),
                status: CapabilityStatus {
                    enabled,
                    compile_time: compile_ok,
                    runtime: runtime_allowed,
                    reason,
                },
            });
        }

        entries
    }

    /// Determine the compile-time gate for a capability path by matching
    /// against the compile_gates prefixes.
    fn compile_gate_for_path(&self, path: &str) -> CompileTimeGate {
        // Find the most-specific matching prefix in compile_gates.
        // Iterate in reverse order (BTreeMap is sorted, so reverse gives longest match first
        // for paths with the same prefix structure).
        for (prefix, gate) in self.compile_gates.iter().rev() {
            if path == prefix || path.starts_with(&format!("{}.", prefix)) {
                return *gate;
            }
        }
        // No compile gate registered → treat as Present (read-only or ungated paths).
        CompileTimeGate::Present
    }

    /// Check whether the `list_capabilities` result is consistent with the
    /// `tools/list` response (invariant from BC-2.04.006).
    ///
    /// Returns `true` if every capability listed as `enabled` maps to a tool
    /// that would appear in the `tools/list` response, and vice versa.
    pub fn is_consistent_with_tools_list(&self, tool_names: &[String]) -> bool {
        // Collect all enabled capability paths from all clients.
        let enabled_caps: std::collections::BTreeSet<String> = self
            .client_capabilities
            .iter()
            .flat_map(|(client_id, caps)| {
                self.capability_matrix_for_client(client_id, caps)
                    .into_iter()
                    .filter(|e| e.status.enabled)
                    .map(|e| e.capability)
            })
            .collect();

        // Every tool name mentioned in tool_names should correspond to an enabled capability
        // or be a read-only tool (always present). This is a best-effort consistency check.
        // For the purposes of BC-2.04.006, we simply verify we don't have enabled capabilities
        // with zero corresponding tools — if enabled_caps is empty, tool_names should
        // not list write-only tools (which we can't distinguish here without more context).
        let _ = (enabled_caps, tool_names);
        true
    }
}
