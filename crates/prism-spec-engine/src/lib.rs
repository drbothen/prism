//! `prism-spec-engine` — Config-driven sensor adapter engine with infusion enrichment + WASM plugin runtime.
//!
//! Parses TOML sensor specs, executes multi-step fetch pipelines, maps sensor
//! columns to OCSF fields, and loads infusion enrichment specs for UDF export to prism-query.
//!
//! **S-1.15 adds:** `plugin` module — WASM Component Model plugin runtime per AD-019.
//! Loads `.prx` files using `wasmtime`, enforces sandbox constraints, implements
//! hot reload, and isolates plugin panics from the host process.
//!
//! **S-1.12 adds:** `config_manager`, `hot_reload`, `reload_config`, `add_sensor_spec`,
//! `list_sensor_specs` modules — ArcSwap-based hot reload and runtime management (BC-2.16.005..010).
//!
//! # Architecture Compliance
//!
//! - MUST NOT depend on DataFusion or Arrow (AD-015). Exports descriptor structs only;
//!   DataFusion registration is prism-query's responsibility (S-3.02).
//! - OCSF field path validation uses an embedded schema — never a runtime fetch.
//! - Infusion credentials MUST NOT appear in log output at any level (INV-INFUSE-005 / AD-017).
//! - `InfusionRegistry` MUST use `arc_swap::ArcSwap` for hot reload — never `RwLock` (AD-007).
//! - Config hot reload MUST use `ArcSwap<ConfigSnapshot>` — never `RwLock` (AD-018).
//!
//! # Subsystems
//! SS-16 — Spec Engine (Layer 2: Business Logic)
//! SS-17 — WASM Plugin Runtime (Layer 2: Business Logic)
//! SS-19 — Infusion Enrichment Framework (Layer 2: Business Logic)

pub mod column_mapping;
pub mod custom_adapter;
// SS-19 — Infusion Enrichment Framework (S-1.14)
pub mod infusion;
pub mod interpolation;
pub mod pipeline;
pub mod plugin;
pub mod spec_parser;
pub mod validation;
pub mod write_endpoint;

// S-PLUGIN-PREREQ-B — spec-driven auth interface (BC-2.01.013 / ADR-023 §C2)
pub mod auth_provider;

pub(crate) mod proofs;

// S-1.12 modules — hot reload and runtime management (BC-2.16.005..010 / AD-018)
pub mod add_sensor_spec;
pub mod config_manager;
pub mod error;
pub mod hot_reload;
pub mod list_sensor_specs;
pub mod reload_config;
pub mod types;

// S-3.1.05 — OrgId-keyed spec store (BC-3.1.001 / ADR-006 §4 Step 2)
pub mod org_scoped_store;

// S-1.11 re-exports
pub use column_mapping::{ColumnMapping, MappingResult};
pub use custom_adapter::{CustomAdapter, CustomAdapterRegistry};
// S-1.14 infusion exports
pub use infusion::cache::QueryScopedInfusionCache;
pub use infusion::enrich_descriptor::EnrichStageDescriptor;
pub use infusion::udf::InfusionUdfDescriptor;
pub use infusion::{
    BuiltInSourceType, CredentialRef, InfusionField, InfusionRegistry, InfusionRegistryInner,
    InfusionSource, InfusionSourceConfig, InfusionSpec, InfusionType, PipeStageConfig,
    PluginConfig,
};
pub use interpolation::{InterpolationContext, InterpolationError};
pub use pipeline::{FetchContext, PipelineExecutor, PipelineResult};
pub use plugin::{
    ActionResult, AlertContext, CaseContext, LoadedPlugin, PluginRuntime, PluginType, ReportContext,
};
pub use spec_parser::{
    AuthType, ColumnSpec, FetchStep, PaginationConfig, RateLimitHints, SensorSpec,
    SensorTableDescriptor, SpecLoader, TableSpec,
};
// TableType is now re-exported from prism-core (S-2.08 Defect 2 fix)
pub use prism_core::TableType;
pub use validation::{ValidationError, ValidationWarning, ValidatorOutput, validate_sensor_spec};
pub use write_endpoint::{
    BatchMode, WriteEndpointRegistry, WriteEndpointSpec, WriteStep, WriteTableDescriptor,
    check_reserved_keyword, validate_write_endpoints,
};

// S-1.12 hot-reload re-exports
pub use config_manager::ConfigManager;
pub use error::SpecEngineError;

// S-PLUGIN-PREREQ-B — auth interface re-exports (BC-2.01.013 / ADR-023 §C2)
pub use auth_provider::{AuthProvider, AuthToken};

// NullAuthProvider + MockAuthProvider + FailingAuthProvider: test-only; feature-gated
// to prevent accidental use in production callers that would silently bypass real auth.
// Enable the `test-helpers` Cargo feature in [dev-dependencies] to access these.
#[cfg(any(test, feature = "test-helpers"))]
pub use auth_provider::{
    AuthOutcome, ChainAuthProvider, FailingAuthProvider, MockAuthProvider, NullAuthProvider,
};

// S-3.1.05 re-exports
pub use org_scoped_store::OrgScopedSpecStore;
pub use types::{
    AddSensorSpecArgs, AddSensorSpecResult, ClientStatus, ColumnDef, ColumnType, ConfigSnapshot,
    DtuMode, ListSensorSpecsArgs, ListSensorSpecsResult, ModeChange, ModifiedSpec, PaginationType,
    ReloadConfigArgs, ReloadResult, ReloadStatus, SensorSpecEntry, SpecStatus,
};
