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
//! # Stubs
//! All method bodies are `unimplemented!()`. Implementation lives in S-1.14.

pub mod cache;
pub mod enrich_descriptor;
pub mod loader;
pub mod plugin_bridge;
pub mod sources;
pub mod udf;

use std::collections::HashMap;
use std::sync::Arc;

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};

use prism_core::InfusionError;

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
    /// Refresh interval in seconds (0 = no refresh).
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
/// All public methods are stubs (`unimplemented!()`) — implementation in S-1.14.
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
        let mut seen_within_spec: HashMap<&str, ()> = HashMap::new();
        for field in &spec.fields {
            if seen_within_spec.insert(field.name.as_str(), ()).is_some() {
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

        // Build descriptors — one per field (INV-INFUSE-001 / VP-048).
        let source: Arc<dyn InfusionSource> = Arc::new(NullSource);
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
    /// On validation error: returns `Err` — does NOT partially register.
    /// On success: the registry `ArcSwap` is updated atomically.
    pub fn load_spec(
        &self,
        spec: InfusionSpec,
    ) -> Result<Vec<udf::InfusionUdfDescriptor>, InfusionError> {
        let current = self.inner.load();

        // Validate against current state (pure — does not mutate).
        let descriptors = self.validate_spec_against(&spec, &current)?;

        // Build updated inner: clone existing state and add the new spec.
        let source: Arc<dyn InfusionSource> = Arc::new(NullSource);
        let mut new_entries = current.entries.clone();
        let mut new_udf_to_infusion = current.udf_to_infusion.clone();

        for field in &spec.fields {
            new_udf_to_infusion.insert(field.name.clone(), spec.infusion_id.clone());
        }
        new_entries.insert(spec.infusion_id.clone(), (spec, source));

        // Atomic swap (AD-007 / CI-002).
        self.inner.store(Arc::new(InfusionRegistryInner {
            entries: new_entries,
            udf_to_infusion: new_udf_to_infusion,
        }));

        Ok(descriptors)
    }

    /// Return all currently registered UDF descriptors.
    ///
    /// Consumed by prism-query (S-3.02) to register DataFusion ScalarUDFs.
    pub fn udf_descriptors(&self) -> Vec<udf::InfusionUdfDescriptor> {
        let current = self.inner.load();
        let source: Arc<dyn InfusionSource> = Arc::new(NullSource);
        current
            .entries
            .values()
            .flat_map(|(spec, _)| {
                spec.fields.iter().map(|field| udf::InfusionUdfDescriptor {
                    name: field.name.clone(),
                    input_type: field.input_type.clone(),
                    output_type: field.output_type.clone(),
                    infusion_id: spec.infusion_id.clone(),
                    source: source.clone(),
                    source_column: field.source_column.clone(),
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
            return spec.infusion_type == InfusionType::Plugin;
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
        let source: Arc<dyn InfusionSource> = Arc::new(NullSource);
        let mut new_entries = temp_inner.entries;
        let mut new_udf_to_infusion = temp_inner.udf_to_infusion;

        for field in &updated_spec.fields {
            new_udf_to_infusion.insert(field.name.clone(), updated_spec.infusion_id.clone());
        }
        new_entries.insert(updated_spec.infusion_id.clone(), (updated_spec, source));

        self.inner.store(Arc::new(InfusionRegistryInner {
            entries: new_entries,
            udf_to_infusion: new_udf_to_infusion,
        }));

        Ok(descriptors)
    }
}

impl Default for InfusionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
