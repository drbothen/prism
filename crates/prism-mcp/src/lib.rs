//! prism-mcp — MCP transport layer (SS-10).
//!
//! Safety envelope middleware, tool registry provenance framing (S-1.10),
//! and PrismServer MCP handler with full tool router (S-5.01-FOLLOWUP-MCP-BOOT).
//!
//! Effectful shell: wraps prism-security pure scanning in MCP I/O context.

pub mod context;
pub mod error_mapping;
pub mod health;
pub mod prompts;
pub mod proofs;
pub mod resources;
pub mod safety_envelope;
pub mod server;
pub mod tool_registry;
pub mod tools;

pub use context::PrismContext;
pub use prompts::{
    build_prompt_router, render_client_overview, render_cross_client_status,
    render_investigate_host, render_query_tutorial, render_triage_alerts, PROMPT_CLIENT_OVERVIEW,
    PROMPT_CROSS_CLIENT_STATUS, PROMPT_INVESTIGATE_HOST, PROMPT_QUERY_TUTORIAL,
    PROMPT_TRIAGE_ALERTS, SECURITY_REMINDER,
};
pub use resources::schema::{SchemaSubscriberRegistry, URI_PQL_REFERENCE, URI_TEMPLATE_PQL_SCHEMA};
pub use resources::{
    dispatch_hot_reload_notifications, render_client_list_resource, render_client_sensors_resource,
    render_sensor_inventory_resource, render_sensors_health_resource, ClientInventoryEntry,
    HealthSummary, RateLimitInfo, ResourcePressure, SensorConfigEntry, SensorHealthResult,
    SensorHealthStructuredContent, URI_CONFIG_CLIENTS, URI_SENSORS_HEALTH,
};
pub use safety_envelope::{ResponseEnvelope, ResponseEnvelopeSchema, SafetyEnvelopeBuilder};
pub use server::{
    CapabilityEntry, CapabilityStatus, CheckSensorHealthParams, ListCapabilitiesParams,
    PrismServer, ResolutionStep,
};
pub use tool_registry::{ToolDescriptionRegistrar, ToolRegistration};
pub use tools::prism_describe::{ColumnDescriptor, PrismDescribeResponse, TableDescriptor};
