//! `prism-spec-engine` — Config-driven sensor adapter engine.
//!
//! Parses TOML sensor specs, executes multi-step fetch pipelines, and maps sensor
//! columns to OCSF fields. New sensors can be onboarded via a TOML file without
//! writing Rust; a `CustomAdapter` escape hatch exists for sensors requiring exotic
//! auth flows, binary protocols, or complex response transformations.
//!
//! # Architecture Compliance
//!
//! - MUST NOT depend on DataFusion or Arrow (AD-015). Exports `SensorTableDescriptor`
//!   structs only; DataFusion table registration is prism-query's responsibility (S-3.02).
//! - OCSF field path validation uses an embedded schema — never a runtime fetch.
//!
//! # Subsystem
//! SS-16 — Spec Engine (Layer 2: Business Logic)

pub mod column_mapping;
pub mod custom_adapter;
pub mod interpolation;
pub mod pipeline;
pub mod spec_parser;
pub mod validation;

#[cfg(test)]
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
