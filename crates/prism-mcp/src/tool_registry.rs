//! ToolDescriptionRegistrar — provenance framing and security warning enforcement
//! for tool descriptions (BC-2.09.002, BC-2.09.006).
//!
//! Stub: `unimplemented!()` bodies. Red Gate — tests must fail.

/// A registered MCP tool definition with description and optional outputSchema.
pub struct ToolRegistration {
    pub name: String,
    pub description: String,
    pub is_sensor_tool: bool,
    pub output_schema: Option<serde_json::Value>,
}

/// Registrar that enforces provenance framing and security warnings on all
/// sensor query tool descriptions.
pub struct ToolDescriptionRegistrar;

impl ToolDescriptionRegistrar {
    /// Register a tool, appending required security sections to sensor tools.
    ///
    /// BC-2.09.006: security warning sections are appended by the framework,
    /// not embedded by hand — they must survive description updates.
    pub fn register(&self, tool: ToolRegistration) -> ToolRegistration {
        unimplemented!("ToolDescriptionRegistrar::register — stub (Red Gate)")
    }

    /// Returns `true` if every registered sensor tool description contains the
    /// required 9-section template (BC-2.09.006 postcondition 1).
    pub fn all_sensor_tools_have_required_sections(&self) -> bool {
        unimplemented!("ToolDescriptionRegistrar::all_sensor_tools_have_required_sections — stub (Red Gate)")
    }

    /// Returns `true` if every registered tool has an `outputSchema` that includes
    /// `_meta.safety_flags` as a typed array (BC-2.09.007 postcondition 4).
    pub fn all_tools_have_valid_output_schema(&self) -> bool {
        unimplemented!("ToolDescriptionRegistrar::all_tools_have_valid_output_schema — stub (Red Gate)")
    }
}
