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
// S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 — shared RFC-3339→µs helper (ADR-052 D2/D5)
pub mod datetime;
// S-SPEC-ENV-VAR-001 — Post-TOML-parse env var token resolver (BC-2.16.009 §Validation Rules 6)
pub mod env_resolver;
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

// S-CONFIG-MULTI-TENANT-OVERRIDE-001 — Per-org overlay loading (ADR-029)
pub mod overlay;

// S-PLUGIN-PREREQ-D — Plugin load audit sink (BC-2.05.012 / HIGH-002)
pub mod plugin_audit_sink;

// PLUGIN-MIGRATION-001-E — PluginAuthProvider adapter (HIGH-010)
// AuthProvider implementation that delegates acquire_token to a loaded WASM sensor-auth plugin.
// Wired in boot path when SensorSpec.auth_plugin = Some(plugin_id) (ADR-028 §D).
pub mod plugin_auth_provider;

// S-1.11 re-exports
// S-DTU-CYBERINT-AUTH-FIDELITY-001 — production auth provider for cookie_roundtrip sensors
// (BC-2.01.017; ADR-031 §D3-b; NOT feature-gated — this is a production type)
// NullAuthProvider + MockAuthProvider + FailingAuthProvider + MockCredentialResolver: test-only;
// feature-gated to prevent accidental use in production callers that would silently bypass real
// auth. Enable the `test-helpers` Cargo feature in [dev-dependencies] to access these.
#[cfg(any(test, feature = "test-helpers"))]
pub use auth_provider::{
    AuthOutcome, BackendUnavailableCredentialResolver, ChainAuthProvider, FailingAuthProvider,
    MockAuthProvider, MockCredentialResolver, NotFoundCredentialResolver, NullAuthProvider,
};
// S-PLUGIN-PREREQ-B — auth interface re-exports (BC-2.01.013 / ADR-023 §C2)
pub use auth_provider::{AuthProvider, AuthToken};
pub use auth_provider::{CredentialResolver, PrismCredentialResolver, StaticCookieAuthProvider};
pub use column_mapping::{ColumnMapping, MappingResult};
// S-1.12 hot-reload re-exports
pub use config_manager::ConfigManager;
pub use datetime::parse_datetime_to_micros;
pub use error::SpecEngineError;
// S-1.14 infusion exports
pub use infusion::cache::{InfusionLruCache, InfusionTier3Cache, QueryScopedInfusionCache};
pub use infusion::sources::MAX_SOURCE_FILE_BYTES;
pub use infusion::{
    BuiltInSourceType, CredentialRef, HttpLookupAuthType, HttpLookupConfig,
    HttpLookupCredentialConfig, InfusionField, InfusionRegistry, InfusionRegistryInner,
    InfusionSource, InfusionSourceConfig, InfusionSpec, InfusionType, PipeStageConfig,
    PluginConfig, enrich_descriptor::EnrichStageDescriptor, udf::InfusionUdfDescriptor,
};
// S-DEMO-ENRICHMENT-PIVOT-001: infusion plugin-bridge + loader (BC-2.19.001)
pub use infusion::loader::InfusionLoader;
pub use infusion::plugin_bridge::PluginInfusionSource;
// S-DEMO-ENRICHMENT-PIVOT-002 v1.3: HttpLookupSource (ADR-040 v2.0 D8)
pub use infusion::sources::HttpLookupSource;
// PluginConfigMap re-export for infusion plugin-bridge tests
pub use interpolation::{InterpolationContext, InterpolationError};
pub use plugin::PluginConfigMap;
// S-3.1.05 re-exports
pub use org_scoped_store::OrgScopedSpecStore;
// S-CONFIG-MULTI-TENANT-OVERRIDE-001 re-exports (ADR-029)
pub use overlay::{
    OverlayLoadResult, OverlayLoader, OverlayProvenance, ResolvedSensorSpec, ResolvedSpecKey,
    SensorInstanceOverlay,
};
pub use pipeline::{FetchContext, PipelineExecutor, PipelineResult, extract_at_path};
pub use plugin::{
    ActionResult, AlertContext, CaseContext, LoadedPlugin, ManifestWriteTool, PluginRuntime,
    PluginType, ReportContext,
};
// PLUGIN-MIGRATION-001-E — PluginAuthProvider re-export (HIGH-010)
pub use plugin_auth_provider::PluginAuthProvider;
// TableType is now re-exported from prism-core (S-2.08 Defect 2 fix)
pub use env_resolver::resolve_env_var_tokens;
pub use prism_core::TableType;
pub use spec_parser::{
    AuthType, ColumnSpec, FetchStep, PaginationConfig, RateLimitHints, SensorSpec,
    SensorTableDescriptor, SpecLoader, TableSpec,
};
pub use types::{
    AddSensorSpecArgs, AddSensorSpecResult, ClientStatus, ColumnDef, ColumnType, ConfigSnapshot,
    DtuMode, ListSensorSpecsArgs, ListSensorSpecsResult, ModeChange, ModifiedSpec, PaginationType,
    ReloadConfigArgs, ReloadResult, ReloadStatus, SensorSpecEntry, SpecStatus,
};
pub use validation::{
    ValidationError, ValidationWarning, ValidatorOutput, validate_auth_plugin_fields,
    validate_auth_plugin_registered, validate_sensor_spec, validate_step_methods,
};
pub use write_endpoint::{
    BatchMode, WriteEndpointRegistry, WriteEndpointSpec, WriteStep, WriteTableDescriptor,
    check_reserved_keyword, validate_write_endpoints,
};
