//! Infusion Enrichment Framework — SS-19.
//!
//! Defines `InfusionSpec`, `InfusionRegistry`, and `InfusionSource` trait for
//! loading `.infusion.toml` specs and exporting `InfusionUdfDescriptor` values
//! for downstream DataFusion UDF registration by prism-query (S-3.02).
//!
//! # Architecture Compliance (AD-020, AD-007, AD-017)
//! - `InfusionRegistry` uses `arc_swap::ArcSwap` for hot reload — never `RwLock`.
//! - Credential values from `[infusion.credentials]` MUST NOT appear in logs or errors.
//! - This crate MUST NOT import DataFusion or Arrow.
//!
//! # Plugin-type specs (BC-2.19.001)
//! Use `load_spec_with_runtime` to populate plugin-type specs with a real
//! `Arc<PluginInfusionSource>`. Bare `load_spec` wires real file-backed sources for
//! `LocalLookup` specs; plugin-type specs receive `NullSource` and should use
//! `load_spec_with_runtime` for live enrichment.

pub mod cache;
pub mod enrich_descriptor;
pub mod loader;
pub mod plugin_bridge;
pub mod sources;
pub mod udf;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use arc_swap::ArcSwap;
use prism_core::InfusionError;
use serde::{Deserialize, Serialize};

use crate::plugin::{PluginConfigMap, PluginRuntime};

// ---------------------------------------------------------------------------
// Infusion type
// ---------------------------------------------------------------------------

/// The backing source type for an infusion spec.
///
/// Determines how the `InfusionSource` is constructed and whether API-backed
/// calls are permitted in detection rule filters (BC-2.19.003 / INV-INFUSE-003).
///
/// `#[non_exhaustive]`: forward-compat for infusion schema evolution — new source types
/// (e.g., remote API lookup, streaming source) may be added without a breaking semver change.
/// External match arms must include a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfusionType {
    /// Local lookup from a file-backed source (MMDB, CSV, JSON).
    /// Permitted in detection rule filters.
    LocalLookup,
    /// WASM plugin delegation (may make external HTTP calls).
    /// PROHIBITED in detection rule filters (E-RULE-012).
    Plugin,
    /// HTTP lookup (single GET → JSONPath extraction). PROHIBITED in detection rule filters
    /// (E-RULE-012) — API-backed.
    HttpLookup,
}

// ---------------------------------------------------------------------------
// Source configuration
// ---------------------------------------------------------------------------

/// Source type for a local lookup infusion.
///
/// `#[non_exhaustive]`: forward-compat for infusion schema evolution — new built-in
/// source types (e.g., SQLite lookup, parquet reference) may be added without a breaking
/// semver change. External match arms must include a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuiltInSourceType {
    /// MaxMind MMDB GeoIP database.
    MaxmindMmdb,
    /// CSV file with designated key column.
    Csv,
    /// JSON static reference data (key → object).
    JsonLookup,
}

/// Source configuration block from the infusion TOML spec.
///
/// `#[non_exhaustive]`: forward-compat for infusion schema evolution — fields may expand
/// (e.g., auth config, cache policy, compression settings) without a breaking semver change.
/// Use `..Default::default()` for forward-compatible external construction.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfusionSourceConfig {
    /// The source type discriminant.
    pub source_type: BuiltInSourceType,
    /// Path to the source data file (MMDB, CSV, or JSON).
    pub file_path: String,
    /// For CSV: the column to use as lookup key.
    pub key_column: Option<String>,
    /// Reserved: interval-driven source refresh — currently INERT (not consumed by any runtime code).
    ///
    /// This field is parsed from TOML and stored for forward-compatibility, but no background refresh
    /// task reads it. Hot-reload of source data is handled by the file-watcher path deferred to
    /// S-1.12-FOLLOWUP (BC-2.22.001 §step10-deferred-contract). Until that story ships, writing
    /// `refresh_interval_secs = N` in a spec has no effect at runtime.
    pub refresh_interval_secs: Option<u64>,
}

impl Default for InfusionSourceConfig {
    fn default() -> Self {
        Self {
            source_type: BuiltInSourceType::JsonLookup,
            file_path: String::new(),
            key_column: None,
            refresh_interval_secs: None,
        }
    }
}

impl InfusionSourceConfig {
    /// Construct an `InfusionSourceConfig`.
    ///
    /// `#[non_exhaustive]` prevents struct literal construction from external crates;
    /// use this constructor for forward-compatible external construction.
    pub fn new(
        source_type: BuiltInSourceType,
        file_path: impl Into<String>,
        key_column: Option<String>,
        refresh_interval_secs: Option<u64>,
    ) -> Self {
        Self {
            source_type,
            file_path: file_path.into(),
            key_column,
            refresh_interval_secs,
        }
    }
}

// ---------------------------------------------------------------------------
// Credential reference (AI-opaque, AD-017)
// ---------------------------------------------------------------------------

/// A credential reference — stores the reference path only, never the value.
///
/// Values are resolved at runtime from env vars or keyring; they MUST NOT
/// be stored in this struct or included in any log output (INV-INFUSE-005).
///
/// `#[non_exhaustive]`: forward-compat for infusion schema evolution — fields may
/// expand as new credential resolution mechanisms are added (e.g., vault paths,
/// rotation policies). Use `CredentialRef::new()` for forward-compatible construction.
#[non_exhaustive]
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CredentialRef {
    /// Credential field name (for diagnostics only — safe to log).
    pub field_name: String,
    /// Environment variable name to resolve the value from.
    pub env_var: String,
}

impl std::fmt::Debug for CredentialRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Credential VALUES must never appear in Debug output (INV-INFUSE-005).
        f.debug_struct("CredentialRef")
            .field("field_name", &self.field_name)
            .field("env_var", &self.env_var)
            .field("value", &"<redacted>")
            .finish()
    }
}

impl CredentialRef {
    /// Construct a `CredentialRef` with the given field name and env var.
    ///
    /// Internal construction shortcut. External callers should use `CredentialRef::new()`
    /// for forward-compatible construction when new fields are added.
    pub fn new(field_name: impl Into<String>, env_var: impl Into<String>) -> Self {
        Self {
            field_name: field_name.into(),
            env_var: env_var.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// InfusionField
// ---------------------------------------------------------------------------

/// A single enrichment field declared in `[[infusion.fields]]`.
///
/// Each field produces exactly one `InfusionUdfDescriptor` (INV-INFUSE-001 / BC-2.19.001).
///
/// `#[non_exhaustive]`: forward-compat for infusion schema evolution — fields may expand
/// (e.g., transformation functions, output format hints) without a breaking semver change.
/// External callers must use `InfusionField::new()` or `InfusionField::with_all()`.
#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InfusionField {
    /// UDF name (global within DataFusion SessionContext — must be unique across all specs).
    pub name: String,
    /// The input column fed to the enrichment lookup.
    pub input_field: String,
    /// The input data type (e.g., `"ip"`, `"string"`).
    pub input_type: String,
    /// The output data type (e.g., `"string"`, `"boolean"`).
    pub output_type: String,
    /// Human-readable description.
    pub description: Option<String>,
    /// For CSV/JSON: the column name in the source data to extract.
    pub source_column: Option<String>,
}

impl InfusionField {
    /// Construct an `InfusionField` with the common fields.
    ///
    /// Use `..Default::default()` is not available for `#[non_exhaustive]` structs
    /// from external crates; use this constructor instead for forward-compatible
    /// external construction.
    pub fn new(
        name: impl Into<String>,
        input_field: impl Into<String>,
        input_type: impl Into<String>,
        output_type: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            input_field: input_field.into(),
            input_type: input_type.into(),
            output_type: output_type.into(),
            description: None,
            source_column: None,
        }
    }

    /// Construct an `InfusionField` with all fields (for test fixtures).
    pub fn with_all(
        name: impl Into<String>,
        input_field: impl Into<String>,
        input_type: impl Into<String>,
        output_type: impl Into<String>,
        description: Option<String>,
        source_column: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            input_field: input_field.into(),
            input_type: input_type.into(),
            output_type: output_type.into(),
            description,
            source_column,
        }
    }
}

// ---------------------------------------------------------------------------
// Pipe stage config
// ---------------------------------------------------------------------------

/// Configuration for the `| enrich` PrismQL pipe stage (BC-2.19.001 / AC-3).
///
/// `#[non_exhaustive]`: forward-compat for infusion schema evolution — pipe stage config
/// may expand (e.g., filter conditions, pass-through columns) without a breaking semver
/// change. External callers must use `PipeStageConfig::new()`.
#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipeStageConfig {
    /// Column names added to the upstream result schema by this enrich stage.
    pub adds_columns: Vec<String>,
}

impl PipeStageConfig {
    /// Construct a `PipeStageConfig` with the given column list.
    ///
    /// `#[non_exhaustive]` prevents struct literal construction from external crates;
    /// use this constructor for forward-compatible external construction.
    pub fn new(adds_columns: Vec<String>) -> Self {
        Self { adds_columns }
    }
}

// ---------------------------------------------------------------------------
// Plugin configuration
// ---------------------------------------------------------------------------

/// Configuration for a `type = "plugin"` infusion.
///
/// `#[non_exhaustive]`: forward-compat for infusion schema evolution — plugin config
/// may expand (e.g., plugin version pinning, sandbox config) without a breaking semver
/// change. External callers must use `PluginConfig::new()`.
#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Path to the `.prx` WASM plugin file.
    pub plugin_path: String,
}

impl PluginConfig {
    /// Construct a `PluginConfig` with the given plugin path.
    ///
    /// `#[non_exhaustive]` prevents struct literal construction from external crates;
    /// use this constructor for forward-compatible external construction.
    pub fn new(plugin_path: impl Into<String>) -> Self {
        Self {
            plugin_path: plugin_path.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// HttpLookup configuration (ADR-040 v2.0 D8.2)
// ---------------------------------------------------------------------------

/// Authentication type for HTTP lookup credentials.
///
/// `#[non_exhaustive]`: forward-compat for new auth mechanisms.
/// External match arms must include a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HttpLookupAuthType {
    /// Append `?{param_name}={credential_value}` to the URL.
    QueryParam { param_name: String },
    /// Add `Authorization: Bearer {credential_value}` header.
    BearerHeader,
    /// Add `{header_name}: {credential_value}` header.
    ApiKeyHeader { header_name: String },
}

/// Credential configuration for an http_lookup-type infusion (AD-017).
///
/// The credential value is resolved at call time from `env_var`; it is NEVER stored
/// in this struct. Only `ref_name` (logical name, safe to log) and `env_var` (name only,
/// not value) are stored.
///
/// `#[non_exhaustive]`: forward-compat for new credential resolution mechanisms.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpLookupCredentialConfig {
    /// Logical credential reference name (for diagnostics — safe to log).
    pub ref_name: String,
    /// Environment variable name from which the credential value is resolved at call time.
    pub env_var: String,
    /// How the resolved credential value is applied to the HTTP request.
    pub auth: HttpLookupAuthType,
}

impl HttpLookupCredentialConfig {
    /// Construct an `HttpLookupCredentialConfig`.
    pub fn new(
        ref_name: impl Into<String>,
        env_var: impl Into<String>,
        auth: HttpLookupAuthType,
    ) -> Self {
        Self {
            ref_name: ref_name.into(),
            env_var: env_var.into(),
            auth,
        }
    }
}

/// Configuration for an `InfusionType::HttpLookup` infusion source.
///
/// Contains the HTTP endpoint specification and optional credential reference.
/// Credential VALUES are never stored here — only references (AD-017).
///
/// `#[non_exhaustive]`: forward-compat for new HTTP configuration options.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpLookupConfig {
    /// Base URL of the HTTP service (e.g., `"https://services.nvd.nist.gov"`).
    pub base_url: String,
    /// URL path template with `${input}` placeholder (e.g., `"/rest/json/cves/2.0?cveId=${input}"`).
    pub url_template: String,
    /// HTTP method: `"GET"` or `"POST"`.
    pub method: String,
    /// JSONPath to the subtree containing enrichment fields (e.g., `"$.vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData"`).
    pub response_path: String,
    /// Optional credential reference. `None` = unauthenticated.
    pub credential: Option<HttpLookupCredentialConfig>,
}

impl HttpLookupConfig {
    /// Construct an `HttpLookupConfig`.
    pub fn new(
        base_url: impl Into<String>,
        url_template: impl Into<String>,
        method: impl Into<String>,
        response_path: impl Into<String>,
        credential: Option<HttpLookupCredentialConfig>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            url_template: url_template.into(),
            method: method.into(),
            response_path: response_path.into(),
            credential,
        }
    }
}

// ---------------------------------------------------------------------------
// InfusionSpec
// ---------------------------------------------------------------------------

/// Top-level infusion enrichment spec parsed from an `.infusion.toml` file.
///
/// Loaded by `InfusionRegistry::load_spec` (BC-2.19.001).
/// Credentials use the reference-based model (AD-017) — values are never stored here.
///
/// `#[non_exhaustive]`: forward-compat for infusion schema evolution — root spec type;
/// fields may expand (e.g., schema version, loaded_at timestamp, reload policy) without
/// a breaking semver change. Use `..Default::default()` for forward-compatible external
/// construction.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfusionSpec {
    /// Unique infusion identifier (e.g., `"geoip"`).
    pub infusion_id: String,
    /// Human-readable name.
    pub name: String,
    /// The backing source type.
    pub infusion_type: InfusionType,
    /// Source configuration (for LocalLookup infusions).
    pub source: Option<InfusionSourceConfig>,
    /// Enrichment fields — each produces one UDF descriptor (INV-INFUSE-001).
    pub fields: Vec<InfusionField>,
    /// Pipe stage configuration for `| enrich` queries.
    pub pipe_stage: Option<PipeStageConfig>,
    /// Plugin configuration (for Plugin infusions).
    pub plugin_config: Option<PluginConfig>,
    /// HTTP lookup configuration (for HttpLookup infusions).
    pub http_lookup_config: Option<HttpLookupConfig>,
    /// Credential references (AI-opaque — values resolved at runtime).
    pub credentials: Vec<CredentialRef>,
    /// Path of the source file this spec was loaded from.
    pub source_path: String,
    /// Per-infusion TTL for cache entries (seconds). Default 3600.
    pub cache_ttl_secs: Option<u64>,
}

impl Default for InfusionSpec {
    fn default() -> Self {
        Self {
            infusion_id: String::new(),
            name: String::new(),
            infusion_type: InfusionType::LocalLookup,
            source: None,
            fields: vec![],
            pipe_stage: None,
            plugin_config: None,
            http_lookup_config: None,
            credentials: vec![],
            source_path: String::new(),
            cache_ttl_secs: None,
        }
    }
}

impl InfusionSpec {
    /// Construct an `InfusionSpec` with the essential fields.
    ///
    /// `#[non_exhaustive]` prevents struct literal construction from external crates;
    /// use this constructor for forward-compatible external construction.
    pub fn new(
        infusion_id: impl Into<String>,
        name: impl Into<String>,
        infusion_type: InfusionType,
        fields: Vec<InfusionField>,
        source_path: impl Into<String>,
    ) -> Self {
        Self {
            infusion_id: infusion_id.into(),
            name: name.into(),
            infusion_type,
            source: None,
            fields,
            pipe_stage: None,
            plugin_config: None,
            http_lookup_config: None,
            credentials: vec![],
            source_path: source_path.into(),
            cache_ttl_secs: None,
        }
    }
}

// ---------------------------------------------------------------------------
// InfusionSource trait
// ---------------------------------------------------------------------------

/// Trait implemented by all built-in and plugin infusion source backends.
///
/// Implemented by MmdbSource, CsvSource, JsonLookupSource, and PluginInfusionSource.
/// The per-query dedup cache wraps this trait to ensure unique calls only (BC-2.19.002).
pub trait InfusionSource: Send + Sync + std::fmt::Debug {
    /// Enrich a single input value. Returns `None` if no enrichment is available.
    fn enrich_single(&self, input: &str, input_type: &str) -> Option<serde_json::Value>;

    /// Enrich a batch of input values. Returns parallel `Option<Value>` results.
    fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>>;

    /// Returns `true` if this source is backed by the WASM plugin runtime.
    ///
    /// Default implementation returns `false` (for `NullSource`, `MmdbSource`, `CsvSource`,
    /// `JsonLookupSource`). `PluginInfusionSource` overrides to return `true`.
    ///
    /// Used by tests (and diagnostics) to assert that `infusion_load_step` wired a
    /// `PluginInfusionSource` — not a `NullSource` — for plugin-type infusion specs
    /// (Task 13 / F-SV-1 load-bearing assertion).
    fn is_plugin_backed(&self) -> bool {
        false
    }

    /// Returns `true` if this source is a built-in `HttpLookupSource`.
    ///
    /// Default implementation returns `false` (for `NullSource`, `MmdbSource`, `CsvSource`,
    /// `JsonLookupSource`, `PluginInfusionSource`). `HttpLookupSource` overrides to return `true`.
    ///
    /// Used by tests (AC-002 load-bearing assertion) to verify that `load_spec` wired a real
    /// `HttpLookupSource` — not a `NullSource` — for `InfusionType::HttpLookup` specs.
    /// Without this assertion, the hollow-feature defect (FIX-1) would not be caught
    /// by tests: a NullSource passes descriptor-count checks but silently returns None
    /// for every enrichment call (TD-VSDD-059 paper-fix guard).
    fn is_http_lookup_backed(&self) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// InfusionRegistryInner — the swappable payload
// ---------------------------------------------------------------------------

/// The registry data swapped atomically during hot reload (CI-002 / AD-007).
#[derive(Debug)]
pub struct InfusionRegistryInner {
    /// Map of infusion_id → (spec, source).
    pub entries: HashMap<String, (InfusionSpec, Arc<dyn InfusionSource>)>,
    /// Global UDF name → infusion_id reverse index (for duplicate detection and is_api_backed).
    pub udf_to_infusion: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// InfusionRegistry
// ---------------------------------------------------------------------------

/// Registry of loaded infusion specs and their source backends.
///
/// Uses `arc_swap::ArcSwap` for lock-free hot reload (AD-007 / CI-002).
///
/// Public API:
/// - [`InfusionRegistry::new`] — create an empty registry.
/// - [`InfusionRegistry::load_spec`] — validate and register a spec using `NullSource`
///   (suitable for local-lookup specs or tests that do not need live enrichment).
/// - [`InfusionRegistry::load_spec_with_runtime`] — validate and register a plugin-type
///   spec wired to a real `PluginInfusionSource` backed by `Arc<PluginRuntime>`.
/// - [`InfusionRegistry::udf_descriptors`] — return all registered `InfusionUdfDescriptor`
///   values for DataFusion UDF registration (consumed by prism-query, S-3.02).
/// - [`InfusionRegistry::is_api_backed`] — check whether a UDF name maps to an API-backed
///   plugin infusion.
/// - [`InfusionRegistry::enrich_descriptor`] — return the `EnrichStageDescriptor` for a
///   named infusion (used by the pipe-stage enrichment planner).
pub struct InfusionRegistry {
    inner: ArcSwap<InfusionRegistryInner>,
}

impl std::fmt::Debug for InfusionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfusionRegistry").finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// NullSource — placeholder source for specs without a file-backed source
// ---------------------------------------------------------------------------

/// A no-op source used when no source config is provided (e.g., during unit tests
/// that exercise descriptor export without real data files).
#[derive(Debug)]
struct NullSource;

impl InfusionSource for NullSource {
    fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
        None
    }

    fn enrich_batch(&self, inputs: &[String], _input_type: &str) -> Vec<Option<serde_json::Value>> {
        inputs.iter().map(|_| None).collect()
    }
}

// ---------------------------------------------------------------------------
// InfusionRegistry implementation
// ---------------------------------------------------------------------------

impl InfusionRegistry {
    /// Create an empty `InfusionRegistry`.
    pub fn new() -> Self {
        InfusionRegistry {
            inner: ArcSwap::new(Arc::new(InfusionRegistryInner {
                entries: HashMap::new(),
                udf_to_infusion: HashMap::new(),
            })),
        }
    }

    /// Validate a spec and produce descriptors without touching the shared registry.
    ///
    /// Returns `Err` if validation fails so callers can abort before touching shared state.
    fn validate_spec_against(
        &self,
        spec: &InfusionSpec,
        existing_inner: &InfusionRegistryInner,
    ) -> Result<Vec<udf::InfusionUdfDescriptor>, InfusionError> {
        // BC-2.19.001: at least one field required.
        if spec.fields.is_empty() {
            return Err(InfusionError::MissingRequiredField {
                field: "fields".to_string(),
                spec_path: spec.source_path.clone(),
            });
        }

        // VP-048: check for within-spec duplicate field names.
        let mut seen_within_spec: HashSet<&str> = HashSet::new();
        for field in &spec.fields {
            if !seen_within_spec.insert(field.name.as_str()) {
                return Err(InfusionError::DuplicateUdfName {
                    udf_name: field.name.clone(),
                    path1: spec.source_path.clone(),
                    path2: spec.source_path.clone(),
                });
            }
        }

        // BC-2.19.001: check for cross-spec duplicate UDF names.
        for field in &spec.fields {
            if let Some(existing_infusion_id) = existing_inner.udf_to_infusion.get(&field.name) {
                // Find the source path of the existing registration.
                let existing_path = existing_inner
                    .entries
                    .get(existing_infusion_id)
                    .map(|(s, _)| s.source_path.as_str())
                    .unwrap_or("<unknown>");
                return Err(InfusionError::DuplicateUdfName {
                    udf_name: field.name.clone(),
                    path1: existing_path.to_string(),
                    path2: spec.source_path.clone(),
                });
            }
        }

        // Story Task 1 / BC-2.19.001: pipe_stage.adds_columns must be non-empty (if pipe_stage present)
        // and must reference only declared field names. Both constraints are enforced here so
        // load_spec (in-memory path) and load_spec_with_runtime share the same gate as parse
        // (TOML path). validate_pipe_stage_columns is the single implementation of both checks.
        loader::InfusionLoader::validate_pipe_stage_columns(spec)?;

        // Build descriptors — one per field (INV-INFUSE-001 / VP-048).
        let source: Arc<dyn InfusionSource> = Arc::new(NullSource);
        let cache_ttl_secs = spec.cache_ttl_secs.unwrap_or(3600);
        let descriptors: Vec<udf::InfusionUdfDescriptor> = spec
            .fields
            .iter()
            .map(|field| udf::InfusionUdfDescriptor {
                name: field.name.clone(),
                input_type: field.input_type.clone(),
                output_type: field.output_type.clone(),
                infusion_id: spec.infusion_id.clone(),
                source: source.clone(),
                source_column: field.source_column.clone(),
                cache_ttl_secs,
                input_field: field.input_field.clone(),
            })
            .collect();

        Ok(descriptors)
    }

    /// Load and validate a single `InfusionSpec` into the registry.
    ///
    /// Produces exactly N `InfusionUdfDescriptor` values for a spec with N fields.
    /// Returns `Err(InfusionError::DuplicateUdfName)` if any field name conflicts with
    /// an already-registered UDF (BC-2.19.001 / INV-INFUSE-001 / VP-048).
    ///
    /// For `LocalLookup` specs, the real file-backed `InfusionSource` is constructed via
    /// `sources::load_source` and stored in the registry. Plugin-type specs use
    /// `NullSource` here — use `load_spec_with_runtime` to wire a real `PluginInfusionSource`.
    ///
    /// On validation error: returns `Err` — does NOT partially register.
    /// On success: the registry `ArcSwap` is updated atomically.
    pub fn load_spec(
        &self,
        spec: InfusionSpec,
    ) -> Result<Vec<udf::InfusionUdfDescriptor>, InfusionError> {
        let current = self.inner.load();

        // BC-2.19.001 v2.0 A3 — last-writer-wins overwrite: if this infusion_id is already
        // registered, build a temporary view of the registry WITHOUT the old spec before
        // running duplicate-name validation. This mirrors hot_reload's pattern and ensures:
        // 1. validate_spec_against doesn't false-positive on the old spec's field names
        //    (a reload may legitimately reuse the same UDF names under the same infusion_id).
        // 2. stale udf_to_infusion entries for the OLD spec's fields are removed before the
        //    new spec's entries are inserted (TV-19-001-overwrite-purge).
        let infusion_id = spec.infusion_id.clone();
        let (validation_inner, mut new_entries, mut new_udf_to_infusion) =
            if current.entries.contains_key(&infusion_id) {
                // Remove the old spec's entries from a working copy for validation.
                let mut temp_entries = current.entries.clone();
                let mut temp_udf_map = current.udf_to_infusion.clone();
                if let Some((old_spec, _)) = temp_entries.remove(&infusion_id) {
                    for old_field in &old_spec.fields {
                        temp_udf_map.remove(&old_field.name);
                    }
                }
                let validation_inner = InfusionRegistryInner {
                    entries: temp_entries.clone(),
                    udf_to_infusion: temp_udf_map.clone(),
                };
                (validation_inner, temp_entries, temp_udf_map)
            } else {
                // First registration for this infusion_id — validate against full current state.
                let new_entries = current.entries.clone();
                let new_udf_to_infusion = current.udf_to_infusion.clone();
                let validation_inner = InfusionRegistryInner {
                    entries: new_entries.clone(),
                    udf_to_infusion: new_udf_to_infusion.clone(),
                };
                (validation_inner, new_entries, new_udf_to_infusion)
            };

        // Validate against the (purged) state — pure, does not mutate shared state.
        let descriptors = self.validate_spec_against(&spec, &validation_inner)?;

        // Wire the real source based on infusion type:
        // - LocalLookup with source config: real file-backed source via load_source.
        // - HttpLookup: real HttpLookupSource (AC-002 / FIX-1 hollow-feature fix). MUST NOT
        //   fall through to NullSource — a NullSource silently returns None for every
        //   enrichment call, making the entire NVD enrichment path dead at runtime.
        //   Propagate construction errors as Err (same pattern as SourceFileTooLarge).
        // - Plugin: NullSource here; callers must use load_spec_with_runtime for plugin.
        // - Other / no source config: NullSource.
        let source: Arc<dyn InfusionSource> = if spec.infusion_type == InfusionType::HttpLookup {
            // AC-002 / ADR-040 v2.0 D8: wire HttpLookupSource for HttpLookup specs.
            // Construction validates SSRF rules (CWE-918) and sets the 30s HTTP timeout
            // (CLAUDE.md §Conventions). SsrfRejected and other construction errors propagate
            // as Err — do NOT degrade to NullSource (hollow-feature prohibition, FIX-1).
            let http_config = spec.http_lookup_config.clone().ok_or_else(|| {
                InfusionError::MissingRequiredField {
                    field: "http_lookup_config".to_string(),
                    spec_path: spec.infusion_id.clone(),
                }
            })?;
            let client = crate::pipeline::build_http_client_with_timeout();
            Arc::new(sources::http_lookup::HttpLookupSource::new(
                client,
                http_config,
                spec.infusion_id.clone(),
            )?)
        } else if spec.infusion_type == InfusionType::LocalLookup {
            if let Some(ref source_config) = spec.source {
                match sources::load_source(source_config) {
                    Ok(s) => s,
                    // E-INFUSE-012: source file exceeds the 100 MiB OOM guard — MUST NOT degrade
                    // to NullSource. Propagate as Err so load_spec returns Err and registers NO
                    // entry (AC-11 / EC-19-007). The atomic swap is NOT performed.
                    // SEC-001 (CWE-400); BC-2.19.001 §Error Conditions E-INFUSE-012.
                    Err(err @ InfusionError::SourceFileTooLarge { .. }) => {
                        return Err(err);
                    }
                    // Non-oversize failure (file not found or corrupt): fall through to NullSource
                    // so load_all continues loading other specs rather than aborting. Emit WARN
                    // (EC-19-004 / LOW-A: failed source must be distinguishable from no-match).
                    // NO event_type field to avoid BC-2.16.002 catalog requirement (SAP-1).
                    // NO file_path or credential values in the log (AD-017).
                    Err(ref err) => {
                        tracing::warn!(
                            infusion_id = %spec.infusion_id,
                            source_type = ?source_config.source_type,
                            error_kind = %err,
                            "infusion: source load failed for LocalLookup spec — falling back to NullSource; \
                             enrichment calls will return None until the source file is corrected and the \
                             spec is hot-reloaded (LOW-A, S-1.14-REDO)"
                        );
                        Arc::new(NullSource)
                    }
                }
            } else {
                Arc::new(NullSource)
            }
        } else {
            Arc::new(NullSource)
        };

        // Build updated inner: insert the new spec into the purged working copy.
        for field in &spec.fields {
            new_udf_to_infusion.insert(field.name.clone(), spec.infusion_id.clone());
        }
        new_entries.insert(spec.infusion_id.clone(), (spec, source.clone()));

        // Atomic swap (AD-007 / CI-002).
        self.inner.store(Arc::new(InfusionRegistryInner {
            entries: new_entries,
            udf_to_infusion: new_udf_to_infusion,
        }));

        // OBS-1 fix: rebuild returned descriptors with the REAL source so the caller always
        // receives a descriptor whose `source` matches the stored entry. `validate_spec_against`
        // builds descriptors with `NullSource` for duplicate-detection purposes only; the real
        // source is constructed above. Returning the NullSource-backed descriptors would be a
        // latent footgun: a future caller using the return value to register UDFs would silently
        // get NullSource (all enrichment → None) even though the registry holds a real source.
        // F-TTL-1: `cache_ttl_secs` is preserved from the spec (already set on `d` by
        // `validate_spec_against`), so the real descriptor carries the correct per-spec TTL.
        let real_descriptors: Vec<udf::InfusionUdfDescriptor> = descriptors
            .iter()
            .map(|d| udf::InfusionUdfDescriptor {
                name: d.name.clone(),
                input_type: d.input_type.clone(),
                output_type: d.output_type.clone(),
                infusion_id: d.infusion_id.clone(),
                source: source.clone(),
                source_column: d.source_column.clone(),
                cache_ttl_secs: d.cache_ttl_secs,
                input_field: d.input_field.clone(),
            })
            .collect();

        Ok(real_descriptors)
    }

    /// Load and validate a single `InfusionSpec` into the registry, wiring a real
    /// `Arc<PluginInfusionSource>` for plugin-type specs.
    ///
    /// For `InfusionType::Plugin` specs the `PluginInfusionSource` is constructed with:
    /// - `plugin_id` = `spec.infusion_id` (the plugin ID under which the `.prx` is loaded
    ///   into the `PluginRuntime`)
    /// - `config` = empty `PluginConfigMap` (credential values are resolved at call time
    ///   from env vars per AD-017; the config map is not pre-populated here)
    /// - `runtime` = the supplied `Arc<PluginRuntime>`
    ///
    /// For non-plugin specs with a `source` config, wires the real file-backed source via
    /// `sources::load_source` (MMDB/CSV/JSON-lookup). Without a source config: `NullSource`.
    ///
    /// Returns `Err(InfusionError::DuplicateUdfName)` if any field name conflicts with an
    /// already-registered UDF (BC-2.19.001 / INV-INFUSE-001 / VP-048).
    pub fn load_spec_with_runtime(
        &self,
        spec: InfusionSpec,
        runtime: Arc<PluginRuntime>,
    ) -> Result<Vec<udf::InfusionUdfDescriptor>, InfusionError> {
        let current = self.inner.load();

        // BC-2.19.001 v2.0 A3 — last-writer-wins overwrite: same purge-before-validate
        // pattern as load_spec and hot_reload (see load_spec for detailed rationale).
        let infusion_id = spec.infusion_id.clone();
        let (validation_inner, mut new_entries, mut new_udf_to_infusion) =
            if current.entries.contains_key(&infusion_id) {
                let mut temp_entries = current.entries.clone();
                let mut temp_udf_map = current.udf_to_infusion.clone();
                if let Some((old_spec, _)) = temp_entries.remove(&infusion_id) {
                    for old_field in &old_spec.fields {
                        temp_udf_map.remove(&old_field.name);
                    }
                }
                let validation_inner = InfusionRegistryInner {
                    entries: temp_entries.clone(),
                    udf_to_infusion: temp_udf_map.clone(),
                };
                (validation_inner, temp_entries, temp_udf_map)
            } else {
                let new_entries = current.entries.clone();
                let new_udf_to_infusion = current.udf_to_infusion.clone();
                let validation_inner = InfusionRegistryInner {
                    entries: new_entries.clone(),
                    udf_to_infusion: new_udf_to_infusion.clone(),
                };
                (validation_inner, new_entries, new_udf_to_infusion)
            };

        // Validate against the (purged) state — pure, does not mutate shared state.
        let descriptors = self.validate_spec_against(&spec, &validation_inner)?;

        // Build the real source:
        // - Plugin specs: PluginInfusionSource wired to the runtime.
        // - HttpLookup specs: HttpLookupSource (AC-002 / FIX-1 hollow-feature fix).
        //   MUST NOT fall through to NullSource — same rationale as in load_spec.
        // - LocalLookup specs with a source_config: real file-backed source via load_source.
        // - LocalLookup specs without source_config (test stubs): NullSource.
        let source: Arc<dyn InfusionSource> = if spec.infusion_type == InfusionType::Plugin {
            // Resolve credentials at spec load time per AD-017 pattern.
            // Each CredentialRef carries the env_var name; the value is read from the environment
            // and inserted into the PluginConfigMap keyed by field_name.
            // PluginConfigMap values are SecretString — credential bytes never stored as plain String.
            // Credential values MUST NOT be logged or stored beyond the PluginConfigMap (INV-INFUSE-005).
            let mut config_map = PluginConfigMap::new();
            for cred in &spec.credentials {
                let value = std::env::var(&cred.env_var).unwrap_or_default();
                config_map.insert(cred.field_name.clone(), secrecy::SecretString::new(value));
            }
            Arc::new(plugin_bridge::PluginInfusionSource::new(
                spec.infusion_id.clone(),
                Arc::new(config_map),
                runtime,
            ))
        } else if spec.infusion_type == InfusionType::HttpLookup {
            // AC-002 / ADR-040 v2.0 D8: wire HttpLookupSource for HttpLookup specs.
            // Propagate construction errors as Err — do NOT degrade to NullSource.
            let http_config = spec.http_lookup_config.clone().ok_or_else(|| {
                InfusionError::MissingRequiredField {
                    field: "http_lookup_config".to_string(),
                    spec_path: spec.infusion_id.clone(),
                }
            })?;
            let client = crate::pipeline::build_http_client_with_timeout();
            Arc::new(sources::http_lookup::HttpLookupSource::new(
                client,
                http_config,
                spec.infusion_id.clone(),
            )?)
        } else if let Some(ref source_config) = spec.source {
            match sources::load_source(source_config) {
                Ok(s) => s,
                // E-INFUSE-012: source file exceeds the 100 MiB OOM guard — MUST NOT degrade
                // to NullSource. Propagate as Err so load_spec_with_runtime returns Err and
                // registers NO entry (AC-11 / EC-19-007). The atomic swap is NOT performed.
                // SEC-001 (CWE-400); BC-2.19.001 §Error Conditions E-INFUSE-012.
                Err(err @ InfusionError::SourceFileTooLarge { .. }) => {
                    return Err(err);
                }
                // Non-oversize failure (file not found or corrupt): fall back to NullSource.
                // NO event_type field to avoid BC-2.16.002 catalog requirement (SAP-1).
                // NO file_path or credential values in the log (AD-017).
                Err(ref err) => {
                    tracing::warn!(
                        infusion_id = %spec.infusion_id,
                        source_type = ?source_config.source_type,
                        error_kind = %err,
                        "infusion: source load failed in load_spec_with_runtime — falling back to \
                         NullSource; enrichment calls will return None until source is corrected \
                         and the spec is hot-reloaded (LOW-A, S-1.14-REDO)"
                    );
                    Arc::new(NullSource)
                }
            }
        } else {
            Arc::new(NullSource)
        };

        // Build updated inner: insert the new spec into the purged working copy.
        for field in &spec.fields {
            new_udf_to_infusion.insert(field.name.clone(), spec.infusion_id.clone());
        }
        new_entries.insert(spec.infusion_id.clone(), (spec, source.clone()));

        // Atomic swap (AD-007 / CI-002).
        self.inner.store(Arc::new(InfusionRegistryInner {
            entries: new_entries,
            udf_to_infusion: new_udf_to_infusion,
        }));

        // OBS-1 fix: rebuild returned descriptors with the REAL source (same pattern as
        // load_spec — see that method for the detailed rationale).
        // F-TTL-1: `cache_ttl_secs` preserved from descriptor (set by `validate_spec_against`).
        let real_descriptors: Vec<udf::InfusionUdfDescriptor> = descriptors
            .iter()
            .map(|d| udf::InfusionUdfDescriptor {
                name: d.name.clone(),
                input_type: d.input_type.clone(),
                output_type: d.output_type.clone(),
                infusion_id: d.infusion_id.clone(),
                source: source.clone(),
                source_column: d.source_column.clone(),
                cache_ttl_secs: d.cache_ttl_secs,
                input_field: d.input_field.clone(),
            })
            .collect();

        Ok(real_descriptors)
    }

    /// Return all currently registered UDF descriptors.
    ///
    /// Consumed by prism-query (S-3.02) to register DataFusion ScalarUDFs.
    ///
    /// Uses the stored `InfusionSource` for each entry (BC-2.19.001: plugin-type
    /// descriptors carry a real `PluginInfusionSource` when the registry was populated via
    /// `load_spec_with_runtime`; entries loaded via bare `load_spec` carry the real
    /// constructed source — matching the stored registry state).
    pub fn udf_descriptors(&self) -> Vec<udf::InfusionUdfDescriptor> {
        let current = self.inner.load();
        current
            .entries
            .values()
            .flat_map(|(spec, stored_source)| {
                let source = stored_source.clone();
                // F-TTL-1: propagate per-spec TTL into every descriptor so prism-query
                // uses the correct TTL when writing to Tier 2 / Tier 3 cache.
                let cache_ttl_secs = spec.cache_ttl_secs.unwrap_or(3600);
                spec.fields
                    .iter()
                    .map(move |field| udf::InfusionUdfDescriptor {
                        name: field.name.clone(),
                        input_type: field.input_type.clone(),
                        output_type: field.output_type.clone(),
                        infusion_id: spec.infusion_id.clone(),
                        source: source.clone(),
                        source_column: field.source_column.clone(),
                        cache_ttl_secs,
                        input_field: field.input_field.clone(),
                    })
            })
            .collect()
    }

    /// Return the `EnrichStageDescriptor` for a named infusion.
    ///
    /// Missing name → `Err(InfusionError::UnknownInfusion)` (E-INFUSE-001).
    pub fn enrich_descriptor(
        &self,
        name: &str,
    ) -> Result<enrich_descriptor::EnrichStageDescriptor, InfusionError> {
        let current = self.inner.load();
        let (spec, _) =
            current
                .entries
                .get(name)
                .ok_or_else(|| InfusionError::UnknownInfusion {
                    name: name.to_string(),
                })?;

        // Build output columns from the pipe_stage config if available,
        // falling back to the field names (BC-2.19.001 / AC-3).
        let output_columns: Vec<String> = spec
            .pipe_stage
            .as_ref()
            .map(|ps| ps.adds_columns.clone())
            .unwrap_or_else(|| spec.fields.iter().map(|f| f.name.clone()).collect());

        // The input_field is the first field's input_field (all fields share the same input).
        let input_field = spec
            .fields
            .first()
            .map(|f| f.input_field.clone())
            .unwrap_or_default();

        Ok(enrich_descriptor::EnrichStageDescriptor {
            infusion_name: name.to_string(),
            input_field,
            output_columns,
            infusion_id: spec.infusion_id.clone(),
        })
    }

    /// Returns `true` if the named UDF comes from a `type = "plugin"` infusion.
    ///
    /// Consumed by S-4.03 (detection rule loader) to enforce E-RULE-012.
    /// Returns `false` for unknown UDF names (unknown is not API-backed).
    /// (BC-2.19.003 / INV-INFUSE-003 / AC-4)
    pub fn is_api_backed(&self, udf_name: &str) -> bool {
        let current = self.inner.load();
        if let Some(infusion_id) = current.udf_to_infusion.get(udf_name)
            && let Some((spec, _)) = current.entries.get(infusion_id)
        {
            return matches!(
                spec.infusion_type,
                InfusionType::Plugin | InfusionType::HttpLookup
            );
        }
        false
    }

    /// Hot reload: atomically swap the registry after successful spec re-validation.
    ///
    /// If validation fails, the previous registry is retained unchanged (CI-002 / BC-2.19.004).
    /// Returns the new set of UDF descriptors on success, or an error retaining the previous state.
    pub fn hot_reload(
        &self,
        updated_spec: InfusionSpec,
    ) -> Result<Vec<udf::InfusionUdfDescriptor>, InfusionError> {
        let current = self.inner.load();

        // Build a temporary view of the registry without the infusion being reloaded
        // (so we don't get false duplicate errors for the same infusion_id).
        let infusion_id = updated_spec.infusion_id.clone();
        let mut temp_entries = current.entries.clone();
        let mut temp_udf_map = current.udf_to_infusion.clone();

        // Remove existing entries for this infusion_id so the duplicate check only
        // catches conflicts with OTHER infusions.
        if let Some((old_spec, _)) = temp_entries.remove(&infusion_id) {
            for field in &old_spec.fields {
                temp_udf_map.remove(&field.name);
            }
        }

        let temp_inner = InfusionRegistryInner {
            entries: temp_entries,
            udf_to_infusion: temp_udf_map,
        };

        // Validate against the temporary view (without holding a lock — pure check).
        let descriptors = self.validate_spec_against(&updated_spec, &temp_inner)?;

        // Validation passed — build new inner and swap atomically.
        // Wire the real source based on infusion type (same policy as load_spec + FIX-1).
        let source: Arc<dyn InfusionSource> =
            if updated_spec.infusion_type == InfusionType::HttpLookup {
                // AC-002 / FIX-1: wire HttpLookupSource for HttpLookup specs at hot-reload time.
                // Propagate construction errors as Err — preserve PRIOR registration unchanged.
                let http_config = updated_spec.http_lookup_config.clone().ok_or_else(|| {
                    InfusionError::MissingRequiredField {
                        field: "http_lookup_config".to_string(),
                        spec_path: updated_spec.infusion_id.clone(),
                    }
                })?;
                let client = crate::pipeline::build_http_client_with_timeout();
                Arc::new(sources::http_lookup::HttpLookupSource::new(
                    client,
                    http_config,
                    updated_spec.infusion_id.clone(),
                )?)
            } else if updated_spec.infusion_type == InfusionType::LocalLookup {
                if let Some(ref source_config) = updated_spec.source {
                    match sources::load_source(source_config) {
                        Ok(s) => s,
                        // E-INFUSE-012: source file exceeds the 100 MiB OOM guard — MUST NOT
                        // degrade to NullSource and MUST NOT perform the atomic swap. Return Err
                        // so hot_reload returns Err and the PRIOR registration is preserved
                        // unchanged (EC-19-007 / AC-11). SEC-001 (CWE-400).
                        Err(err @ InfusionError::SourceFileTooLarge { .. }) => {
                            return Err(err);
                        }
                        // Non-oversize failure (file not found or corrupt) at hot-reload time:
                        // fall back to NullSource with WARN (EC-19-004 preserved).
                        // NO event_type field to avoid BC-2.16.002 catalog requirement (SAP-1).
                        // NO file_path or credential values in the log (AD-017).
                        Err(ref err) => {
                            tracing::warn!(
                                infusion_id = %updated_spec.infusion_id,
                                source_type = ?source_config.source_type,
                                error_kind = %err,
                                "infusion: source load failed during hot_reload — falling back to \
                                 NullSource; enrichment calls will return None until the source \
                                 file is corrected and hot-reload retried (LOW-A, S-1.14-REDO)"
                            );
                            Arc::new(NullSource)
                        }
                    }
                } else {
                    Arc::new(NullSource)
                }
            } else {
                Arc::new(NullSource)
            };
        let mut new_entries = temp_inner.entries;
        let mut new_udf_to_infusion = temp_inner.udf_to_infusion;

        for field in &updated_spec.fields {
            new_udf_to_infusion.insert(field.name.clone(), updated_spec.infusion_id.clone());
        }
        new_entries.insert(
            updated_spec.infusion_id.clone(),
            (updated_spec, source.clone()),
        );

        self.inner.store(Arc::new(InfusionRegistryInner {
            entries: new_entries,
            udf_to_infusion: new_udf_to_infusion,
        }));

        // OBS-1 fix: rebuild returned descriptors with the REAL source (same pattern as
        // load_spec — see that method for the detailed rationale).
        // F-TTL-1: `cache_ttl_secs` preserved from descriptor (set by `validate_spec_against`).
        let real_descriptors: Vec<udf::InfusionUdfDescriptor> = descriptors
            .iter()
            .map(|d| udf::InfusionUdfDescriptor {
                name: d.name.clone(),
                input_type: d.input_type.clone(),
                output_type: d.output_type.clone(),
                infusion_id: d.infusion_id.clone(),
                source: source.clone(),
                source_column: d.source_column.clone(),
                cache_ttl_secs: d.cache_ttl_secs,
                input_field: d.input_field.clone(),
            })
            .collect();

        Ok(real_descriptors)
    }
}

impl Default for InfusionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
