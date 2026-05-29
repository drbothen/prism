//! ToolDescriptionRegistrar — provenance framing and security warning enforcement
//! for tool descriptions (BC-2.09.002, BC-2.09.006).
//!
//! Security sections are appended by the registrar (framework layer), not embedded
//! by hand in each tool's description string. This ensures sections survive updates.

use prism_security::provenance::{SecurityWarning, ToolDescriptionTemplate};

/// A registered MCP tool definition with description and optional outputSchema.
#[non_exhaustive]
pub struct ToolRegistration {
    pub name: String,
    pub description: String,
    pub is_sensor_tool: bool,
    pub output_schema: Option<serde_json::Value>,
}

impl ToolRegistration {
    /// Construct a new ToolRegistration.
    ///
    /// This constructor allows external crates to build ToolRegistrations without
    /// breaking the `#[non_exhaustive]` struct literal restriction. New fields added
    /// to ToolRegistration in future versions can be initialized here with defaults.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        is_sensor_tool: bool,
        output_schema: Option<serde_json::Value>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            is_sensor_tool,
            output_schema,
        }
    }
}

/// Registrar that enforces provenance framing and security warnings on all
/// sensor query tool descriptions.
pub struct ToolDescriptionRegistrar;

impl ToolDescriptionRegistrar {
    /// Register a tool, appending required security sections to sensor tools.
    ///
    /// BC-2.09.006: security warning sections are appended by the framework,
    /// not embedded by hand — they must survive description updates.
    ///
    /// Idempotent: if the description already contains the required sections,
    /// they are not duplicated.
    pub fn register(&self, tool: ToolRegistration) -> ToolRegistration {
        if !tool.is_sensor_tool {
            return tool;
        }

        let mut description = tool.description;

        // Append missing sections idempotently
        let missing = ToolDescriptionTemplate::missing_sections(&description);
        if missing.is_empty() {
            return ToolRegistration {
                name: tool.name,
                description,
                is_sensor_tool: tool.is_sensor_tool,
                output_schema: tool.output_schema,
            };
        }

        // Append all missing sections in canonical order
        for section in ToolDescriptionTemplate::SENSOR_TOOL_REQUIRED_SECTIONS {
            if !description.contains(section) {
                let section_content = Self::default_section_content(section);
                description.push('\n');
                description.push_str(&section_content);
            }
        }

        ToolRegistration {
            name: tool.name,
            description,
            is_sensor_tool: tool.is_sensor_tool,
            output_schema: tool.output_schema,
        }
    }

    /// Returns the default content for a missing section.
    fn default_section_content(section: &str) -> String {
        match section {
            "DATA SOURCE:" => "DATA SOURCE: External sensor API".to_owned(),
            "DATA TRUST LEVEL:" => format!(
                "DATA TRUST LEVEL: {}",
                ToolDescriptionTemplate::DATA_TRUST_LEVEL_TEXT
            ),
            "WHEN TO USE:" => "WHEN TO USE: when querying sensor data".to_owned(),
            "WHEN NOT TO USE:" => "WHEN NOT TO USE: for non-sensor operations".to_owned(),
            "PARAMETERS:" => "PARAMETERS: See tool schema for parameter definitions.".to_owned(),
            "PAGINATION:" => "PAGINATION: cursor-based via next_cursor field".to_owned(),
            "RESPONSE:" => "RESPONSE: results array with sensor record fields".to_owned(),
            "ERRORS:" => "ERRORS: sensor_unavailable, rate_limited, auth_error".to_owned(),
            "SECURITY NOTE:" => SecurityWarning::security_note_content().to_owned(),
            _ => format!("{section} (see tool documentation)"),
        }
    }
}
