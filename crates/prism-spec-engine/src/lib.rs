//! `prism-spec-engine` — Config-driven sensor adapter engine with infusion enrichment.
//!
//! Parses TOML sensor specs, executes multi-step fetch pipelines, maps sensor
//! columns to OCSF fields, and loads infusion enrichment specs for UDF export to prism-query.
//!
//! # Architecture Compliance
//!
//! - MUST NOT depend on DataFusion or Arrow (AD-015). Exports descriptor structs only;
//!   DataFusion registration is prism-query's responsibility (S-3.02).
//! - OCSF field path validation uses an embedded schema — never a runtime fetch.
//! - Infusion credentials MUST NOT appear in log output at any level (INV-INFUSE-005 / AD-017).
//! - `InfusionRegistry` MUST use `arc_swap::ArcSwap` for hot reload — never `RwLock` (AD-007).
//!
//! # Subsystems
//! SS-16 — Spec Engine (Layer 2: Business Logic)
//! SS-19 — Infusion Enrichment Framework (Layer 2: Business Logic)

pub mod column_mapping;
pub mod custom_adapter;
pub mod interpolation;
pub mod pipeline;
pub mod spec_parser;
pub mod validation;
pub mod write_endpoint;

// SS-19 — Infusion Enrichment Framework (S-1.14)
pub mod infusion;

pub(crate) mod proofs;

pub use column_mapping::{ColumnMapping, MappingResult};
pub use custom_adapter::{CustomAdapter, CustomAdapterRegistry};
pub use interpolation::{InterpolationContext, InterpolationError};
pub use pipeline::{FetchContext, PipelineExecutor, PipelineResult};
pub use spec_parser::{
    AuthType, ColumnSpec, FetchStep, PaginationConfig, RateLimitHints, SensorSpec,
    SensorTableDescriptor, SpecLoader, TableSpec,
};
pub use validation::{validate_sensor_spec, ValidationError, ValidationWarning, ValidatorOutput};
<<<<<<< HEAD
pub use write_endpoint::{
    check_reserved_keyword, validate_write_endpoints, BatchMode, WriteEndpointRegistry,
    WriteEndpointSpec, WriteStep, WriteTableDescriptor,
};

// S-1.14 infusion exports
pub use infusion::{
    BuiltInSourceType, CredentialRef, InfusionField, InfusionRegistry, InfusionRegistryInner,
    InfusionSource, InfusionSpec, InfusionSourceConfig, InfusionType, PipeStageConfig, PluginConfig,
};
pub use infusion::cache::QueryScopedInfusionCache;
pub use infusion::enrich_descriptor::EnrichStageDescriptor;
pub use infusion::udf::InfusionUdfDescriptor;
