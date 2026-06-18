//! TOML infusion spec parser and validator.
//!
//! Loads `*.infusion.toml` files from `{config_dir}/infusions/`, validates each spec,
//! and returns `InfusionSpec` values for registration into `InfusionRegistry`.
//!
//! # Validation rules (BC-2.19.001)
//! - `infusion_id` must be present and non-empty.
//! - At least one `[[infusion.fields]]` entry required.
//! - `source.type` must be one of: `maxmind_mmdb`, `csv`, `json_lookup`, `plugin`.
//!   Unknown types return `InfusionError::UnknownSourceType`.
//! - Credential references must use reference-based model (no inline values, AD-017).
//! - `pipe_stage.adds_columns` must match the `[[infusion.fields]]` names.
//! - On validation error: return `Err` — do NOT partially register.
//!
//! # Credential redaction (INV-INFUSE-005 / AD-017)
//! Credential values MUST NOT appear in any error message or log output.

use std::io::Read;
use std::path::Path;

use prism_core::InfusionError;
use serde::Deserialize;

use super::{
    BuiltInSourceType, CredentialRef, InfusionField, InfusionSourceConfig, InfusionSpec,
    InfusionType, PipeStageConfig, PluginConfig,
};

// ---------------------------------------------------------------------------
// Raw TOML deserialization structs (internal — not public API)
// ---------------------------------------------------------------------------

/// Top-level deserialization envelope from a `.infusion.toml` file.
///
/// The TOML schema uses a nested `[infusion]` table as the root content holder.
/// A top-level `[source]` table carries source-type discriminant for "plugin".
#[derive(Debug, Deserialize)]
struct RawInfusionToml {
    infusion: RawInfusion,
    /// Top-level `[source]` block — present for plugin-type specs.
    source: Option<RawTopLevelSource>,
}

/// The `[infusion]` table content.
#[derive(Debug, Deserialize)]
struct RawInfusion {
    infusion_id: String,
    name: String,
    /// Legacy: `source_type` key in the `[infusion]` block (e.g. "plugin", "local_lookup").
    /// Not the canonical source discriminant — `[source].type` is authoritative.
    #[serde(rename = "type", default)]
    infusion_type_str: String,
    fields: Option<Vec<RawField>>,
    pipe_stage: Option<RawPipeStage>,
    plugin_config: Option<RawPluginConfig>,
    credentials: Option<Vec<RawCredential>>,
    source: Option<RawNestedSource>,
    /// Fallback in the `[infusion]` block: `source_type = "plugin"`.
    #[serde(rename = "source_type", default)]
    source_type_fallback: String,
    cache_ttl_secs: Option<u64>,
}

/// Top-level `[source]` block (alternative schema form used by tests).
///
/// Fields `file_path`, `key_column`, `refresh_interval_secs` are active for MMDB/CSV/JSON-lookup
/// paths; `plugin_ref` is used for the plugin-type path.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RawTopLevelSource {
    #[serde(rename = "type")]
    source_type: String,
    /// For plugin-type: reference to the `.prx` plugin file.
    plugin_ref: Option<String>,
    /// For file-backed types (maxmind_mmdb, csv, json_lookup): path to the data file.
    file_path: Option<String>,
    /// For CSV: the column to use as lookup key.
    key_column: Option<String>,
    /// Refresh interval for file-backed sources (seconds).
    refresh_interval_secs: Option<u64>,
}

/// Nested `[infusion.source]` block (used in the fixture TOML schema).
///
/// Fields `file_path`, `key_column`, `refresh_interval_secs` are active for MMDB/CSV/JSON-lookup
/// paths; `plugin_ref` is used for the plugin-type path.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RawNestedSource {
    #[serde(rename = "type")]
    source_type: String,
    /// For plugin-type: reference to the `.prx` plugin file.
    plugin_ref: Option<String>,
    /// For file-backed types (maxmind_mmdb, csv, json_lookup): path to the data file.
    file_path: Option<String>,
    /// For CSV: the column to use as lookup key.
    key_column: Option<String>,
    /// Refresh interval for file-backed sources (seconds).
    refresh_interval_secs: Option<u64>,
}

/// A single `[[infusion.fields]]` entry.
#[derive(Debug, Deserialize)]
struct RawField {
    name: String,
    input_field: String,
    input_type: String,
    output_type: String,
    description: Option<String>,
    source_column: Option<String>,
}

/// `[infusion.pipe_stage]` block.
#[derive(Debug, Deserialize)]
struct RawPipeStage {
    adds_columns: Vec<String>,
}

/// `[infusion.plugin_config]` block (fixture schema form).
#[derive(Debug, Deserialize)]
struct RawPluginConfig {
    plugin_path: String,
}

/// `[[infusion.credentials]]` entry.
#[derive(Debug, Deserialize)]
struct RawCredential {
    field_name: String,
    env_var: String,
}

// ---------------------------------------------------------------------------
// InfusionLoader
// ---------------------------------------------------------------------------

/// Loads and validates infusion specs from a directory.
pub struct InfusionLoader {
    config_dir: String,
}

impl InfusionLoader {
    /// Create a new `InfusionLoader` for the given config directory.
    pub fn new(config_dir: impl Into<String>) -> Self {
        InfusionLoader {
            config_dir: config_dir.into(),
        }
    }

    /// Parse a single TOML string into an `InfusionSpec`.
    ///
    /// Supports `source.type` values: `"plugin"`, `"maxmind_mmdb"`, `"csv"`, `"json_lookup"`.
    /// Unknown source types return `InfusionError::UnknownSourceType`.
    ///
    /// Returns `Ok(InfusionSpec)` or `Err(InfusionError)` — never panics.
    /// Validation failures return descriptive errors without credential values.
    pub fn parse(toml_input: &str, source_path: &str) -> Result<InfusionSpec, InfusionError> {
        // Parse the raw TOML envelope.
        let raw: RawInfusionToml =
            toml::from_str(toml_input).map_err(|e| InfusionError::MissingRequiredField {
                field: format!("toml_parse_error: {}", e),
                spec_path: source_path.to_string(),
            })?;

        let raw_infusion = raw.infusion;

        // Determine source type — authoritative resolution order:
        // 1. Top-level `[source].type` (test TOML schema).
        // 2. Nested `[infusion.source].type` (fixture TOML schema).
        // 3. `[infusion].type` field.
        // 4. `[infusion].source_type` field (fallback).
        let source_type_str = if let Some(ref top) = raw.source {
            top.source_type.clone()
        } else if let Some(ref nested) = raw_infusion.source {
            nested.source_type.clone()
        } else if !raw_infusion.infusion_type_str.is_empty() {
            raw_infusion.infusion_type_str.clone()
        } else {
            raw_infusion.source_type_fallback.clone()
        };

        // Resolve infusion type variant.
        let infusion_type = match source_type_str.as_str() {
            "plugin" => InfusionType::Plugin,
            "local_lookup" => InfusionType::LocalLookup,
            "maxmind_mmdb" | "csv" | "json_lookup" => InfusionType::LocalLookup,
            "" => {
                return Err(InfusionError::MissingRequiredField {
                    field: "source.type".to_string(),
                    spec_path: source_path.to_string(),
                });
            }
            other => {
                return Err(InfusionError::UnknownSourceType {
                    type_name: other.to_string(),
                });
            }
        };

        // Validate infusion_id is non-empty.
        if raw_infusion.infusion_id.is_empty() {
            return Err(InfusionError::MissingRequiredField {
                field: "infusion_id".to_string(),
                spec_path: source_path.to_string(),
            });
        }

        // Validate infusion_id does NOT contain ':' (the cache-key delimiter).
        //
        // All three infusion cache tiers compose keys as `format!("{}:{}", infusion_id, input_value)`.
        // A colon in infusion_id makes the composite key non-injective: id="a:b" enriching "c"
        // produces key "a:b:c", which is indistinguishable from id="a" enriching "b:c".
        // This is the only delimiter used across all three tiers (TD-VSDD-060 grep confirmed:
        // cache.rs lines 50, 58, 140, 161 all use `format!("{}:{}", infusion_id, input_value)`).
        // Tier 3 hashes the composed key via SHA-256 so the raw collision still applies.
        // Guard-at-parse prevents any infusion_id containing ':' from ever reaching the cache.
        if raw_infusion.infusion_id.contains(':') {
            return Err(InfusionError::MissingRequiredField {
                field: "infusion_id must not contain ':' (cache-key delimiter — \
                        prevents cross-infusion cache key collision)"
                    .to_string(),
                spec_path: source_path.to_string(),
            });
        }

        // Validate at least one field.
        let raw_fields = raw_infusion.fields.unwrap_or_default();
        if raw_fields.is_empty() {
            return Err(InfusionError::MissingRequiredField {
                field: "infusion.fields".to_string(),
                spec_path: source_path.to_string(),
            });
        }

        // For plugin type: validate plugin_ref is present.
        if matches!(infusion_type, InfusionType::Plugin) {
            let has_plugin_ref = raw
                .source
                .as_ref()
                .and_then(|s| s.plugin_ref.as_ref())
                .is_some()
                || raw_infusion
                    .source
                    .as_ref()
                    .and_then(|s| s.plugin_ref.as_ref())
                    .is_some()
                || raw_infusion.plugin_config.as_ref().is_some();

            if !has_plugin_ref {
                return Err(InfusionError::MissingRequiredField {
                    field: "plugin_ref (E-INFUSE-003: required for source.type = \"plugin\")"
                        .to_string(),
                    spec_path: source_path.to_string(),
                });
            }
        }

        // Build InfusionField list.
        let fields: Vec<InfusionField> = raw_fields
            .into_iter()
            .map(|rf| InfusionField {
                name: rf.name,
                input_field: rf.input_field,
                input_type: rf.input_type,
                output_type: rf.output_type,
                description: rf.description,
                source_column: rf.source_column,
            })
            .collect();

        // Build plugin_config from either [source].plugin_ref or [infusion.plugin_config].
        let plugin_config = if let Some(ref top_source) = raw.source {
            top_source.plugin_ref.as_ref().map(|pr| PluginConfig {
                plugin_path: pr.clone(),
            })
        } else if let Some(ref nested_source) = raw_infusion.source {
            nested_source.plugin_ref.as_ref().map(|pr| PluginConfig {
                plugin_path: pr.clone(),
            })
        } else {
            raw_infusion.plugin_config.map(|pc| PluginConfig {
                plugin_path: pc.plugin_path,
            })
        };

        // Build pipe_stage.
        let pipe_stage = raw_infusion
            .pipe_stage
            .map(|ps| PipeStageConfig::new(ps.adds_columns));

        // Build credentials.
        let credentials: Vec<CredentialRef> = raw_infusion
            .credentials
            .unwrap_or_default()
            .into_iter()
            .map(|rc| CredentialRef {
                field_name: rc.field_name,
                env_var: rc.env_var,
            })
            .collect();

        // Build source config for LocalLookup types from [source] or [infusion.source] block.
        let source_config: Option<InfusionSourceConfig> =
            if matches!(infusion_type, InfusionType::LocalLookup) {
                // Extract file_path and key_column from whichever source block is present.
                let (raw_type, file_path, key_column, refresh_interval_secs) =
                    if let Some(ref top_source) = raw.source {
                        (
                            top_source.source_type.as_str(),
                            top_source.file_path.clone(),
                            top_source.key_column.clone(),
                            top_source.refresh_interval_secs,
                        )
                    } else if let Some(ref nested_source) = raw_infusion.source {
                        (
                            nested_source.source_type.as_str(),
                            nested_source.file_path.clone(),
                            nested_source.key_column.clone(),
                            nested_source.refresh_interval_secs,
                        )
                    } else {
                        (source_type_str.as_str(), None, None, None)
                    };

                // Resolve the actual source type.
                // Unknown types error — no silent default to JsonLookup (MED-3 / BC-2.19.001).
                let built_in_type = match raw_type {
                    "maxmind_mmdb" => BuiltInSourceType::MaxmindMmdb,
                    "csv" => BuiltInSourceType::Csv,
                    "json_lookup" => BuiltInSourceType::JsonLookup,
                    other => {
                        return Err(InfusionError::UnknownSourceType {
                            type_name: other.to_string(),
                        });
                    }
                };

                Some(InfusionSourceConfig::new(
                    built_in_type,
                    file_path.unwrap_or_default(),
                    key_column,
                    refresh_interval_secs,
                ))
            } else {
                None
            };

        let spec = InfusionSpec {
            infusion_id: raw_infusion.infusion_id,
            name: raw_infusion.name,
            infusion_type,
            source: source_config,
            fields,
            pipe_stage,
            plugin_config,
            credentials,
            source_path: source_path.to_string(),
            cache_ttl_secs: raw_infusion.cache_ttl_secs,
        };

        // Validate credentials use reference-based model (no empty env_var, INV-INFUSE-005 / AD-017).
        // Fully determinable at spec-load time — checks TOML struct fields only.
        Self::validate_credentials(&spec)?;

        // Validate pipe_stage.adds_columns references only declared field names.
        // Fully determinable at spec-load time — all data is in the InfusionSpec struct.
        Self::validate_pipe_stage_columns(&spec)?;

        Ok(spec)
    }

    /// Load all `*.infusion.toml` files from `{config_dir}/infusions/`.
    ///
    /// Returns (specs, errors): valid specs continue loading even if others fail.
    /// Invalid specs produce `InfusionError` values but do not block valid specs.
    ///
    /// Supports `plugin`, `maxmind_mmdb`, `csv`, and `json_lookup` source types.
    /// Unknown source types produce `InfusionError::UnknownSourceType` in the errors vec.
    pub fn load_all(&self) -> (Vec<InfusionSpec>, Vec<InfusionError>) {
        let infusions_dir = Path::new(&self.config_dir).join("infusions");

        let entries = match std::fs::read_dir(&infusions_dir) {
            Ok(e) => e,
            Err(e) => {
                // If the directory doesn't exist, return empty with no errors.
                // load_all is allowed to produce 0 specs when the dir is missing.
                tracing::debug!(
                    dir = %infusions_dir.display(),
                    error = %e,
                    "infusions directory not found or unreadable — returning empty"
                );
                return (vec![], vec![]);
            }
        };

        let mut specs: Vec<InfusionSpec> = vec![];
        let mut errors: Vec<InfusionError> = vec![];

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml")
                || !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .ends_with(".infusion.toml")
            {
                continue;
            }

            let source_path = path.to_string_lossy().to_string();
            let mut content = String::new();
            if let Err(e) =
                std::fs::File::open(&path).and_then(|mut f| f.read_to_string(&mut content))
            {
                errors.push(InfusionError::MissingRequiredField {
                    field: format!("io_error: {}", e),
                    spec_path: source_path,
                });
                continue;
            }

            match Self::parse(&content, &source_path) {
                Ok(spec) => specs.push(spec),
                Err(e) => errors.push(e),
            }
        }

        (specs, errors)
    }

    /// Validate that `pipe_stage.adds_columns` matches the `[[infusion.fields]]` names.
    ///
    /// Enforces two constraints (BC-2.19.001 / Story Task 1):
    /// 1. Every name in `adds_columns` must be a declared `[[infusion.fields]]` name (subset rule).
    /// 2. `adds_columns` must be non-empty — a `pipe_stage` present with an empty column list is
    ///    rejected; callers must either omit `pipe_stage` entirely or list at least one column.
    ///
    /// Returns `Ok(())` on success; `Err(InfusionError::MissingRequiredField)` otherwise.
    pub fn validate_pipe_stage_columns(spec: &InfusionSpec) -> Result<(), InfusionError> {
        if let Some(ref pipe_stage) = spec.pipe_stage {
            // Story Task 1: non-empty constraint — pipe_stage present with 0 adds_columns is invalid.
            if pipe_stage.adds_columns.is_empty() {
                return Err(InfusionError::MissingRequiredField {
                    field: "pipe_stage.adds_columns must not be empty — \
                            omit pipe_stage entirely or list at least one column (E-INFUSE-003)"
                        .to_string(),
                    spec_path: spec.source_path.clone(),
                });
            }

            let field_names: std::collections::HashSet<&str> =
                spec.fields.iter().map(|f| f.name.as_str()).collect();
            for col in &pipe_stage.adds_columns {
                if !field_names.contains(col.as_str()) {
                    return Err(InfusionError::MissingRequiredField {
                        field: format!(
                            "pipe_stage.adds_columns references unknown field '{}'",
                            col
                        ),
                        spec_path: spec.source_path.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Validate that all credential entries use the reference-based model (structural check only).
    ///
    /// Checks that every `CredentialRef.env_var` field is non-empty — i.e., the TOML spec
    /// provides a named environment-variable reference for each credential. This is a
    /// **structural** check performed at spec load time.
    ///
    /// It does NOT resolve the environment variable or verify the credential value exists at
    /// this point. Per AD-017, credentials are resolved at call time (never at load time);
    /// the actual env-var lookup happens in the source backend when `enrich_single` is called.
    ///
    /// Returns `Ok(())` if all credential refs are structurally valid, or
    /// `Err(InfusionError::CredentialUnresolved)` for the first empty `env_var` found.
    /// Credential VALUES MUST NOT appear in any returned error (INV-INFUSE-005 / AD-017).
    pub fn validate_credentials(spec: &InfusionSpec) -> Result<(), InfusionError> {
        for cred in &spec.credentials {
            if cred.env_var.is_empty() {
                return Err(InfusionError::CredentialUnresolved {
                    field_name: cred.field_name.clone(),
                    infusion_id: spec.infusion_id.clone(),
                    env_var_name: "<empty>".to_string(),
                });
            }
        }
        Ok(())
    }
}
