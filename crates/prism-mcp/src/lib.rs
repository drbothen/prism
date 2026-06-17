//! prism-mcp — MCP transport layer (SS-10).
//!
//! Safety envelope middleware, tool registry provenance framing (S-1.10),
//! and PrismServer MCP handler with full tool router (S-5.01-FOLLOWUP-MCP-BOOT).
//!
//! Effectful shell: wraps prism-security pure scanning in MCP I/O context.

pub mod error_mapping;
pub mod safety_envelope;
pub mod server;
pub mod tool_registry;
pub mod tools;

pub use safety_envelope::{ResponseEnvelope, ResponseEnvelopeSchema, SafetyEnvelopeBuilder};
pub use server::{
    CapabilityEntry, CapabilityStatus, ListCapabilitiesParams, PrismServer, ResolutionStep,
};
pub use tool_registry::{ToolDescriptionRegistrar, ToolRegistration};
