//! TOML infusion spec parser and validator.
//!
//! Loads `*.infusion.toml` files from `{config_dir}/infusions/`, validates each spec,
//! and returns `InfusionSpec` values for registration into `InfusionRegistry`.
//!
//! # Validation rules (BC-2.19.001)
//! - `infusion_id` must be present and non-empty.
//! - At least one `[[infusion.fields]]` entry required.
//! - `source.type` must be one of: `maxmind_mmdb`, `csv`, `json_lookup`, `plugin`.
//! - Credential references must use reference-based model (no inline values, AD-017).
//! - `pipe_stage.adds_columns` must match the `[[infusion.fields]]` names.
//! - On validation error: return `Err` — do NOT partially register.
//!
//! # Credential redaction (INV-INFUSE-005 / AD-017)
//! Credential values MUST NOT appear in any error message or log output.
//!
//! # S-DEMO-ENRICHMENT-PIVOT-001
//! Implements the `source.type = "plugin"` path only.
//! `source.type = "maxmind_mmdb"`, `"csv"`, `"json_lookup"` return
//! `InfusionError::UnknownSourceType` (deferred to S-1.14-REDO).

use std::io::Read;
use std::path::Path;

use prism_core::InfusionError;
use serde::Deserialize;

use super::{
    CredentialRef, InfusionField, InfusionSpec, InfusionType, PipeStageConfig, PluginConfig,
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
/// Fields `file_path`, `key_column`, `refresh_interval_secs` are parsed but unused
/// in the plugin-type path (S-DEMO-ENRICHMENT-PIVOT-001); they become active when
/// S-1.14-REDO implements the MMDB/CSV/JSON-lookup paths.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RawTopLevelSource {
    #[serde(rename = "type")]
    source_type: String,
    /// For plugin-type: reference to the `.prx` plugin file.
    plugin_ref: Option<String>,
    /// For file-backed types: path to the data file (S-1.14-REDO).
    file_path: Option<String>,
    /// For CSV: the key column name (S-1.14-REDO).
    key_column: Option<String>,
    /// Refresh interval (S-1.14-REDO).
    refresh_interval_secs: Option<u64>,
}

/// Nested `[infusion.source]` block (used in the fixture TOML schema).
///
/// Fields `file_path`, `key_column`, `refresh_interval_secs` are parsed but unused
/// in the plugin-type path (S-DEMO-ENRICHMENT-PIVOT-001); they become active in S-1.14-REDO.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RawNestedSource {
    #[serde(rename = "type")]
    source_type: String,
    /// For plugin-type: reference to the `.prx` plugin file.
    plugin_ref: Option<String>,
    /// For file-backed types (S-1.14-REDO).
    file_path: Option<String>,
    /// For CSV (S-1.14-REDO).
    key_column: Option<String>,
    /// Refresh interval (S-1.14-REDO).
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
    /// Implements the `source.type = "plugin"` path (S-DEMO-ENRICHMENT-PIVOT-001).
    /// Returns `InfusionError::UnknownSourceType` for `maxmind_mmdb`, `csv`, `json_lookup`
    /// (deferred to S-1.14-REDO).
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
            "local_lookup" => {
                // local_lookup is a grouping type — the actual source subtype
                // is determined by [infusion.source].type or [source].type.
                // For local_lookup group, reject as unsupported (deferred to S-1.14-REDO).
                return Err(InfusionError::UnknownSourceType {
                    type_name: source_type_str,
                });
            }
            "maxmind_mmdb" | "csv" | "json_lookup" => {
                // Deferred to S-1.14-REDO.
                return Err(InfusionError::UnknownSourceType {
                    type_name: source_type_str,
                });
            }
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

        Ok(InfusionSpec {
            infusion_id: raw_infusion.infusion_id,
            name: raw_infusion.name,
            infusion_type,
            source: None, // LocalLookup source config — not used for plugin type
            fields,
            pipe_stage,
            plugin_config,
            credentials,
            source_path: source_path.to_string(),
            cache_ttl_secs: raw_infusion.cache_ttl_secs,
        })
    }

    /// Load all `*.infusion.toml` files from `{config_dir}/infusions/`.
    ///
    /// Returns (specs, errors): valid specs continue loading even if others fail.
    /// Invalid specs produce `InfusionError` values but do not block valid specs.
    ///
    /// Implements only the `source.type = "plugin"` path (S-DEMO-ENRICHMENT-PIVOT-001).
    /// Other source types return `InfusionError::UnknownSourceType` in the errors vec.
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
    /// Returns `Ok(())` or a list of mismatched names.
    pub fn validate_pipe_stage_columns(spec: &InfusionSpec) -> Result<(), InfusionError> {
        if let Some(ref pipe_stage) = spec.pipe_stage {
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

    /// Validate that all credential entries use the reference-based model (no inline values).
    ///
    /// Returns `Ok(())` or `Err` — credential values MUST NOT appear in the error (INV-INFUSE-005).
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
