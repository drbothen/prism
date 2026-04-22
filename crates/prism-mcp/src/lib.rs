//! prism-mcp — MCP transport layer (SS-10).
//!
//! S-1.10 stub: safety envelope middleware and tool registry provenance framing.
//! Effectful shell: wraps prism-security pure scanning in MCP I/O context.

pub mod safety_envelope;
pub mod tool_registry;

pub use safety_envelope::{ResponseEnvelope, SafetyEnvelopeBuilder};
pub use tool_registry::{ToolDescriptionRegistrar, ToolRegistration};
