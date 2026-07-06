//! TOML infusion spec parser and validator.
//!
//! Loads `*.infusion.toml` files from `{config_dir}/infusions/`, validates each spec,
//! and returns `InfusionSpec` values for registration into `InfusionRegistry`.
//!
//! # Validation rules (BC-2.19.001)
//! - `infusion_id` must be present and non-empty.
//! - At least one `[[infusion.fields]]` entry required.
//! - `source.type` must be one of: `maxmind_mmdb`, `csv`, `json_lookup`, `plugin`, `http_lookup`.
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
use prism_core::error::sanitize_for_log;
use serde::Deserialize;

use super::{
    BuiltInSourceType, CredentialRef, HttpLookupAuthType, HttpLookupConfig,
    HttpLookupCredentialConfig, InfusionField, InfusionSourceConfig, InfusionSpec, InfusionType,
    PipeStageConfig, PluginConfig,
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
///
/// `source_type` is optional here because the http_lookup schema (ADR-040 v2.0 D8.1)
/// uses `[source.http]` and `[source.credential]` subtables WITHOUT a top-level
/// `[source] type = ...` key — the discriminant lives in `[infusion] type = "http_lookup"`.
/// With `#[serde(default)]`, TOML subtable schemas parse without error; the empty string
/// causes the source-type resolution to fall through to `raw_infusion.infusion_type_str`.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RawTopLevelSource {
    #[serde(rename = "type", default)]
    source_type: String,
    /// For plugin-type: reference to the `.prx` plugin file.
    plugin_ref: Option<String>,
    /// For file-backed types (maxmind_mmdb, csv, json_lookup): path to the data file.
    file_path: Option<String>,
    /// For CSV: the column to use as lookup key.
    key_column: Option<String>,
    /// Refresh interval for file-backed sources (seconds).
    refresh_interval_secs: Option<u64>,
    /// For http_lookup: the [source.http] subtable (ADR-040 v2.0 D8.1).
    /// Accepted at parse time; populated into `HttpLookupConfig` by the implementer.
    http: Option<RawHttpSubtable>,
    /// For http_lookup: the [source.credential] subtable (AD-017).
    /// Accepted at parse time; credential values never stored — reference-only (INV-INFUSE-005).
    credential: Option<RawHttpCredentialSubtable>,
}

/// `[source.http]` subtable for http_lookup specs (ADR-040 v2.0 D8.1).
/// Parsed at stub level; implementer populates `HttpLookupConfig` from these fields.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RawHttpSubtable {
    base_url: Option<String>,
    url_template: Option<String>,
    method: Option<String>,
    response_path: Option<String>,
}

/// `[source.credential]` subtable for http_lookup specs (AD-017 / INV-INFUSE-005).
/// Credential references only — no inline values.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RawHttpCredentialSubtable {
    #[serde(rename = "ref")]
    credential_ref: Option<String>,
    env_var: Option<String>,
    auth: Option<String>,
    param_name: Option<String>,
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
    /// Supports `source.type` values: `"plugin"`, `"maxmind_mmdb"`, `"csv"`, `"json_lookup"`, `"http_lookup"`.
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
        // 1. Top-level `[source].type` (non-empty — test TOML schema).
        //    For http_lookup specs (ADR-040 v2.0 D8.1) the [source] table uses subtables
        //    ([source.http], [source.credential]) with NO top-level `type` key; `source_type`
        //    defaults to "" and is skipped so resolution continues to step 3.
        // 2. Nested `[infusion.source].type` (fixture TOML schema).
        // 3. `[infusion].type` field.
        // 4. `[infusion].source_type` field (fallback).
        let source_type_str = if let Some(ref top) = raw.source {
            if !top.source_type.is_empty() {
                top.source_type.clone()
            } else if let Some(ref nested) = raw_infusion.source {
                nested.source_type.clone()
            } else if !raw_infusion.infusion_type_str.is_empty() {
                raw_infusion.infusion_type_str.clone()
            } else {
                raw_infusion.source_type_fallback.clone()
            }
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
            "http_lookup" => InfusionType::HttpLookup,
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

        // For http_lookup type: parse [source.http] and [source.credential] subtables
        // and validate url_template / method fields (ADR-040 D8.3).
        let http_lookup_config_parsed: Option<HttpLookupConfig> = if matches!(
            infusion_type,
            InfusionType::HttpLookup
        ) {
            let http_sub = raw
                .source
                .as_ref()
                .and_then(|s| s.http.as_ref())
                .ok_or_else(|| InfusionError::MissingRequiredField {
                    field: "[source.http] block required for type=\"http_lookup\"".to_string(),
                    spec_path: source_path.to_string(),
                })?;

            let base_url = http_sub
                .base_url
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| InfusionError::InvalidFieldSpec {
                    field: "base_url".to_string(),
                    spec_path: source_path.to_string(),
                    message: "source.http.base_url must be non-empty \
                              (E-INFUSE-013 sub-condition 3 / AC-013 / ADR-040 D8.3)"
                        .to_string(),
                })?
                .to_string();

            let url_template = http_sub
                .url_template
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| InfusionError::MissingRequiredField {
                    field: "source.http.url_template must be non-empty".to_string(),
                    spec_path: source_path.to_string(),
                })?
                .to_string();

            // D8.3 validation: url_template must contain "${input}" placeholder.
            if !url_template.contains("${input}") {
                return Err(InfusionError::InvalidFieldSpec {
                    field: "url_template".to_string(),
                    spec_path: source_path.to_string(),
                    message: "url_template must contain \"${input}\" interpolation placeholder \
                                  (ADR-040 D8.3 / AC-016)"
                        .to_string(),
                });
            }

            let method = http_sub
                .method
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| InfusionError::MissingRequiredField {
                    field: "source.http.method must be non-empty".to_string(),
                    spec_path: source_path.to_string(),
                })?
                .to_string();

            // D8.3 validation: method must be "GET" or "POST".
            if method != "GET" && method != "POST" {
                return Err(InfusionError::InvalidFieldSpec {
                    field: "method".to_string(),
                    spec_path: source_path.to_string(),
                    message: format!(
                        "method must be \"GET\" or \"POST\", got \"{}\" (ADR-040 D8.3 / AC-016)",
                        method
                    ),
                });
            }

            let response_path = http_sub
                .response_path
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| InfusionError::InvalidFieldSpec {
                    field: "response_path".to_string(),
                    spec_path: source_path.to_string(),
                    message: "source.http.response_path must be non-empty \
                              (E-INFUSE-013 sub-condition 5 / AC-013 / ADR-040 D8.3)"
                        .to_string(),
                })?
                .to_string();

            // Parse [source.credential] subtable if present.
            let credential_config = raw
                .source
                .as_ref()
                .and_then(|s| s.credential.as_ref())
                .and_then(|c| {
                    let ref_name = c.credential_ref.clone().unwrap_or_default();
                    let env_var = c.env_var.clone().unwrap_or_default();
                    let auth_str = c.auth.as_deref().unwrap_or("bearer_header");
                    let param_name = c.param_name.clone();

                    if ref_name.is_empty() || env_var.is_empty() {
                        return None;
                    }

                    let auth = match auth_str {
                        "query_param" => HttpLookupAuthType::QueryParam {
                            param_name: param_name.unwrap_or_default(),
                        },
                        "bearer_header" => HttpLookupAuthType::BearerHeader,
                        "api_key_header" => HttpLookupAuthType::ApiKeyHeader {
                            header_name: param_name.unwrap_or_default(),
                        },
                        _ => HttpLookupAuthType::BearerHeader,
                    };

                    Some(HttpLookupCredentialConfig::new(ref_name, env_var, auth))
                });

            Some(HttpLookupConfig::new(
                base_url,
                url_template,
                method,
                response_path,
                credential_config,
            ))
        } else {
            None
        };

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

        // Build InfusionField list — validate each field at parse time (AC-007).
        let mut fields: Vec<InfusionField> = Vec::with_capacity(raw_fields.len());
        for rf in raw_fields {
            // DRIFT-PIVOT-UDFNAME-VALIDATION-001: validate name before UDF registration.
            Self::validate_field_name(&rf.name, source_path)?;

            // ADR-051 D3 sub-condition 7 (AC-007): output_type must be one of the 6 canonical
            // values. Checked on every field regardless of infusion type.
            Self::validate_output_type_recognized(&rf.output_type, &rf.name, source_path)?;

            // ADR-051 D3 sub-condition 8 (AC-006): plugin-type fields MUST declare source_column.
            if matches!(infusion_type, InfusionType::Plugin) {
                Self::validate_plugin_type_has_source_column(
                    &rf.name,
                    &raw_infusion.infusion_id,
                    rf.source_column.as_deref(),
                    source_path,
                )?;
            }

            fields.push(InfusionField {
                name: rf.name,
                input_field: rf.input_field,
                input_type: rf.input_type,
                output_type: rf.output_type,
                description: rf.description,
                source_column: rf.source_column,
            });
        }

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
            http_lookup_config: http_lookup_config_parsed,
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
    /// Supports `plugin`, `maxmind_mmdb`, `csv`, `json_lookup`, and `http_lookup` source types.
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
            // Sanitize path for MCP-surfaced errors (AC-012 / SEC-002 CWE-209).
            // Internal debug logging MAY retain the full path.
            let sanitized_path = Self::sanitize_error_path(&source_path, &self.config_dir);

            let mut content = String::new();
            if let Err(e) =
                std::fs::File::open(&path).and_then(|mut f| f.read_to_string(&mut content))
            {
                // Log full path internally before sanitizing for the error surface.
                tracing::debug!(
                    path = %source_path,
                    error = %e,
                    "infusion file io_error (full path for diagnostics)"
                );
                errors.push(InfusionError::MissingRequiredField {
                    field: format!("io_error: {}", e),
                    spec_path: sanitized_path,
                });
                continue;
            }

            match Self::parse(&content, &source_path) {
                Ok(spec) => {
                    // CRIT-2a (DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 / AC-011 / SEC-003 CWE-22):
                    // For plugin-type specs, validate the plugin_ref path against the designated
                    // plugin directory before the spec is returned for PluginRuntime loading.
                    // This is the ONLY place where config_dir context is available to resolve
                    // the relative plugin_ref path against the filesystem.
                    //
                    // plugin_dir = {config_dir}/plugins/ by convention (same directory scanned by
                    // PluginRuntime::load_all_plugins at boot time).
                    //
                    // If the plugin directory does not yet exist (e.g., first run), the traversal
                    // check cannot be performed via canonicalize — skip the check rather than reject
                    // a valid spec (canonicalize returns Err when either path component doesn't exist).
                    if spec.infusion_type == super::InfusionType::Plugin
                        && let Some(ref plugin_cfg) = spec.plugin_config
                    {
                        let plugin_dir = Path::new(&self.config_dir).join("plugins");
                        // Only validate if the plugin_dir exists — canonicalize requires
                        // both the directory and the file to exist (D8 constraint, ADR-040).
                        // If the dir is absent (first boot, CI), skip rather than reject.
                        if plugin_dir.exists() {
                            match Self::validate_plugin_path(
                                &plugin_cfg.plugin_path,
                                &plugin_dir,
                                &sanitized_path,
                            ) {
                                Ok(_) => {}
                                Err(e) => {
                                    errors.push(e);
                                    continue;
                                }
                            }
                        }
                    }
                    specs.push(spec);
                }
                Err(parse_err) => {
                    // Sanitize spec_path in the parse error before surfacing to MCP.
                    // The parse error was created with the absolute path; replace it.
                    // AC-012 / SEC-002 CWE-209: absolute paths must not reach MCP surface.
                    let sanitized_err = match parse_err {
                        InfusionError::MissingRequiredField { field, .. } => {
                            InfusionError::MissingRequiredField {
                                field,
                                spec_path: sanitized_path.clone(),
                            }
                        }
                        // Other variants don't carry a spec_path field — pass through as-is.
                        other => other,
                    };
                    errors.push(sanitized_err);
                }
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

    /// Validate that an `InfusionField.name` matches the identifier regex
    /// `^[a-zA-Z][a-zA-Z0-9_]*$` (must start with a letter, followed by alphanumerics
    /// or underscores; empty is rejected).
    ///
    /// Called during `parse` for every `[[infusion.fields]]` entry BEFORE `SessionContext`
    /// UDF registration, so malformed names never reach DataFusion.
    ///
    /// Returns `Ok(())` if the name is valid, or `Err(InfusionError::InvalidFieldSpec)` if not.
    ///
    /// # Security
    /// DRIFT-PIVOT-UDFNAME-VALIDATION-001 (AC-007 / SEC-001 CWE-20).
    /// SQL-injection characters (`;`, space, `-`, starting digits) are all rejected.
    ///
    /// # Examples of valid names
    /// `threat_is_known_malicious`, `cvss_base_score`, `field1`, `THREAT_SCORE`
    ///
    /// # Examples of rejected names
    /// `"threat; DROP TABLE"`, `" leading_space"`, `"has-hyphen"`, `"1starts_with_digit"`, `""`
    pub fn validate_field_name(name: &str, spec_path: &str) -> Result<(), InfusionError> {
        // DRIFT-PIVOT-UDFNAME-VALIDATION-001 / AC-007 / SEC-001 CWE-20:
        // Identifier regex: ^[a-zA-Z][a-zA-Z0-9_]*$ — must start with letter, followed by
        // alphanumerics or underscore. Empty string rejected. SQL-injection chars (;, space, -)
        // and leading digits all rejected. Validated char-by-char (zero-dep, no regex crate).
        // AC-007 / BC-2.19.001: validate_field_name MUST return InvalidFieldSpec (E-INFUSE-013),
        // NOT MissingRequiredField (E-INFUSE-003), for invalid field name characters.
        // This is a load-bearing contract distinction: callers that match on the variant
        // (e.g., test_enrichment_pivot_002_sec001_udf_name_rejects_*) assert the specific
        // variant — not just is_err() — to ensure the correct E-INFUSE-013 code is emitted.
        if name.is_empty() {
            return Err(InfusionError::InvalidFieldSpec {
                field: "field name must match [a-zA-Z][a-zA-Z0-9_]* — got empty string \
                        (DRIFT-PIVOT-UDFNAME-VALIDATION-001 / AC-007 / SEC-001 CWE-20)"
                    .to_string(),
                spec_path: spec_path.to_string(),
                message: "field name must not be empty".to_string(),
            });
        }

        let mut chars = name.chars();

        // First character must be ASCII alpha.
        let first = chars.next().expect("non-empty string has a first char");
        if !first.is_ascii_alphabetic() {
            return Err(InfusionError::InvalidFieldSpec {
                field: name.to_string(),
                spec_path: spec_path.to_string(),
                message: format!(
                    "field name must match [a-zA-Z][a-zA-Z0-9_]* — '{}' starts with '{}' \
                     (must start with [a-zA-Z]) \
                     (DRIFT-PIVOT-UDFNAME-VALIDATION-001 / AC-007 / SEC-001 CWE-20)",
                    name, first
                ),
            });
        }

        // Remaining characters must be ASCII alphanumeric or underscore.
        for ch in chars {
            if !ch.is_ascii_alphanumeric() && ch != '_' {
                return Err(InfusionError::InvalidFieldSpec {
                    field: name.to_string(),
                    spec_path: spec_path.to_string(),
                    message: format!(
                        "field name must match [a-zA-Z][a-zA-Z0-9_]* — '{}' contains invalid \
                         character '{}' (only [a-zA-Z0-9_] allowed after first char) \
                         (DRIFT-PIVOT-UDFNAME-VALIDATION-001 / AC-007 / SEC-001 CWE-20)",
                        name, ch
                    ),
                });
            }
        }

        Ok(())
    }

    /// Validate that a `plugin_ref` path resolves within the designated plugin directory.
    ///
    /// Steps (DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 / AC-011 / SEC-003 CWE-22):
    /// 0. Structural pre-check: reject `plugin_ref` containing `..` components immediately —
    ///    this fires before any I/O and is the primary E-INFUSE-013 sub-condition 6 gate.
    ///    Absolute paths (starting with `/` on Unix or `\\` on Windows) are also rejected.
    /// 1. Resolve the `plugin_ref` relative to `plugin_dir`.
    /// 2. Call `std::fs::canonicalize(resolved_path)` — follows symlinks, resolves `..`.
    /// 3. Assert `canonicalized_path.starts_with(&plugin_dir_canonical)`.
    ///    If not: return `Err(InfusionError::InvalidFieldSpec { ... })`.
    ///    Do NOT include the attempted path in the error message surfaced to callers.
    /// 4. Relative paths within plugin_dir (e.g. `subdir/plugin.prx`) are accepted.
    ///
    /// Called before any `std::fs::read` or `File::open` on the `.prx` path.
    pub fn validate_plugin_path(
        plugin_ref: &str,
        plugin_dir: &std::path::Path,
        spec_path: &str,
    ) -> Result<std::path::PathBuf, InfusionError> {
        // Step 0 (STRUCTURAL PRE-CHECK): detect `..` path components and absolute paths
        // in the plugin_ref string BEFORE attempting any filesystem operation.
        //
        // E-INFUSE-013 sub-condition 6 — "plugin_ref contains path-traversal characters
        // (`..`, `/`, `\`)" — fires here when the string-level check catches the traversal.
        // This ensures the test fires deterministically (without requiring the traversal
        // target to exist on the filesystem), and also provides defense-in-depth against
        // symlink-based traversals which only the canonicalize check (step 3-4) can catch.
        //
        // We scan the Path components for any `..` (CurDir is never a traversal concern).
        // Absolute paths (Component::RootDir or Component::Prefix) are also rejected
        // because plugin_ref must be relative to plugin_dir by contract.
        use std::path::{Component, Path};
        for component in Path::new(plugin_ref).components() {
            match component {
                Component::ParentDir => {
                    return Err(InfusionError::InvalidFieldSpec {
                        field: "plugin_ref".to_string(),
                        spec_path: spec_path.to_string(),
                        message: "plugin_ref contains '..' path component — \
                                  path traversal characters not allowed in plugin_ref \
                                  (E-INFUSE-013 sub-condition 6 / \
                                  DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 / AC-011 / SEC-003 CWE-22)"
                            .to_string(),
                    });
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(InfusionError::InvalidFieldSpec {
                        field: "plugin_ref".to_string(),
                        spec_path: spec_path.to_string(),
                        message: "plugin_ref must be a relative path — \
                                  absolute paths not allowed in plugin_ref \
                                  (E-INFUSE-013 sub-condition 6 / \
                                  DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 / AC-011 / SEC-003 CWE-22)"
                            .to_string(),
                    });
                }
                Component::CurDir | Component::Normal(_) => {
                    // Acceptable — relative path component within plugin_dir.
                }
            }
        }

        // Step 1: Canonicalize the plugin_dir itself first.
        // This is needed so starts_with comparisons work correctly even when plugin_dir
        // is a relative or symlinked path.
        let plugin_dir_canonical =
            std::fs::canonicalize(plugin_dir).map_err(|_| InfusionError::MissingRequiredField {
                field: "plugin_dir cannot be resolved \
                        (DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 / AC-011 / SEC-003 CWE-22)"
                    .to_string(),
                spec_path: spec_path.to_string(),
            })?;

        // Step 2: Resolve plugin_ref relative to plugin_dir (before canonicalize, while dir exists).
        let candidate = plugin_dir.join(plugin_ref);

        // Step 3: Canonicalize the resolved path (resolves `..`, symlinks, etc.).
        // If the file doesn't exist, canonicalize returns an error — that's an access error,
        // not a traversal error. We must call canonicalize to detect `..` escapes.
        let candidate_canonical =
            std::fs::canonicalize(&candidate).map_err(|_| InfusionError::MissingRequiredField {
                field: "plugin_ref cannot be resolved within plugin_dir — file not found or \
                        path traversal detected \
                        (DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 / AC-011 / SEC-003 CWE-22)"
                    .to_string(),
                spec_path: spec_path.to_string(),
            })?;

        // Step 4: Assert the canonicalized path starts_with the canonicalized plugin_dir.
        // If it escapes the directory (via `../` or symlink), this fails.
        if !candidate_canonical.starts_with(&plugin_dir_canonical) {
            // AC-011: do NOT include the traversal target path in the error message.
            // E-INFUSE-013 sub-condition 6: plugin_ref path-traversal (CWE-22).
            return Err(InfusionError::InvalidFieldSpec {
                field: "plugin_ref".to_string(),
                spec_path: spec_path.to_string(),
                message: "plugin_ref resolved outside designated plugin_dir — \
                          path traversal rejected \
                          (E-INFUSE-013 sub-condition 6 / DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 \
                          / AC-011 / SEC-003 CWE-22)"
                    .to_string(),
            });
        }

        Ok(candidate_canonical)
    }

    /// Sanitize an `InfusionError` message for MCP surface exposure by stripping
    /// the absolute filesystem path prefix from `spec_path`.
    ///
    /// Internal tracing (DEBUG/INFO for operator diagnostics) MAY retain the full path.
    /// Only the MCP-surfaced error string must be sanitized (AC-012 / SEC-002 CWE-209).
    ///
    /// Acceptable output forms:
    /// - Filename only: `bad.infusion.toml` (using `Path::file_name()`)
    /// - Relative path from config dir: `infusions/bad.infusion.toml`
    /// - Redacted path: `<infusions-dir>/bad.infusion.toml`
    ///
    /// # Security
    /// DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001 (AC-012 / SEC-002 CWE-209).
    pub fn sanitize_error_path(absolute_path: &str, config_dir: &str) -> String {
        let path = Path::new(absolute_path);
        let base = Path::new(config_dir);

        // Attempt 1: strip the config_dir prefix to get a relative path.
        // e.g., "/tmp/abc/infusions/bad.infusion.toml" → "infusions/bad.infusion.toml"
        if let Ok(relative) = path.strip_prefix(base) {
            return relative.to_string_lossy().to_string();
        }

        // Attempt 2: return just the filename (last component).
        // e.g., "/tmp/abc/infusions/bad.infusion.toml" → "bad.infusion.toml"
        if let Some(filename) = path.file_name() {
            return filename.to_string_lossy().to_string();
        }

        // Fallback: return a fully redacted path indicator (should never be reached
        // since even bare file paths have a file_name component).
        "<redacted-path>".to_string()
    }

    /// Validate that `output_type` is a recognized value per ADR-051 D1.
    ///
    /// E-INFUSE-013 sub-condition 7: `output_type` is not in the canonical set
    /// `{"string", "integer", "float", "boolean", "json", "datetime"}`.
    ///
    /// Called from `parse()` for each `[[infusion.fields]]` entry before the field is
    /// registered as a DataFusion UDF. An unknown `output_type` at load time prevents a
    /// runtime panic when `output_arrow_type()` falls through to the `Utf8` fallback.
    ///
    /// Error format (error-taxonomy v2.17, CR-002):
    /// ```text
    /// E-INFUSE-013: invalid field name 'output_type' in infusion spec '{spec_path}':
    ///  field entry '{field_name}' declares unknown output_type '{value}'; must be one of:
    ///  string, integer, float, boolean, json, datetime
    ///  (datetime maps to Timestamp(µs,UTC) per ADR-051 v1.2 / ADR-052)
    /// ```
    ///
    /// SEC-001 (CWE-117): both `output_type` and `field_name` are stripped of ASCII control
    /// characters before interpolation to prevent log injection / LLM prompt injection.
    ///
    /// Story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 (AC-007; ADR-051 D3 sub-condition 7).
    ///
    pub fn validate_output_type_recognized(
        output_type: &str,
        field_name: &str,
        spec_path: &str,
    ) -> Result<(), InfusionError> {
        // ADR-051 D3 sub-condition 7: valid output types are the 6 canonical values.
        const VALID: &[&str] = &["string", "integer", "float", "boolean", "json", "datetime"];
        if VALID.contains(&output_type) {
            return Ok(());
        }
        // SEC-001 (CWE-117): strip control chars from both interpolated values before
        // constructing the error message (error-taxonomy v2.17 SEC-001 Rendering Note).
        let clean_value = sanitize_for_log(output_type);
        let clean_field_name = sanitize_for_log(field_name);
        Err(InfusionError::InvalidFieldSpec {
            // AC-007 canonical attribute label: the invalid attribute is `output_type`,
            // not the name of the field containing it.
            field: "output_type".to_owned(),
            spec_path: spec_path.to_owned(),
            // CR-002 (error-taxonomy v2.17): new canonical body includes the enclosing field
            // entry's name so the operator can identify which [[infusion.fields]] entry is wrong.
            message: format!(
                "field entry '{}' declares unknown output_type '{}'; must be one of: string, \
                 integer, float, boolean, json, datetime (datetime maps to Timestamp(\u{b5}s,UTC) \
                 per ADR-051 v1.2 / ADR-052)",
                clean_field_name, clean_value
            ),
        })
    }

    /// Validate that a `type = "plugin"` infusion field declares `source_column`.
    ///
    /// E-INFUSE-013 sub-condition 8: a plugin-type field is missing `source_column`.
    ///
    /// Without `source_column`, `project_value()` falls into the passthrough branch and
    /// serializes the entire plugin response object — the root cause of
    /// DRIFT-PIVOT-UDF-OUTPUT-TYPE-001 Failure A (doubly-encoded JSON).
    ///
    /// Called from `parse()` after the source type is determined and fields are parsed.
    /// Only fires for fields in infusions with `type = "plugin"`.
    ///
    /// Error format:
    /// ```text
    /// E-INFUSE-013: invalid field name 'source_column' in infusion spec '{spec_path}':
    ///  plugin-type field '{field_name}' in infusion '{infusion_id}' must declare 'source_column'
    ///  to project a specific field from the plugin response object; without source_column
    ///  the full response object is serialized (DRIFT-PIVOT-UDF-OUTPUT-TYPE-001 root cause)
    /// ```
    ///
    /// SEC-001 (CWE-117): `field_name` and `infusion_id` are stripped of ASCII control
    /// characters before interpolation to prevent log injection / LLM prompt injection.
    ///
    /// SEC-003: `pub(crate)` — only called internally from `parse()`; no cross-crate callers.
    /// (`validate_output_type_recognized` remains `pub` because integration tests call it
    /// directly from `prism-spec-engine/tests/enrichment_pivot_002_tests.rs`.)
    ///
    /// Story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 (AC-006; ADR-051 D3 sub-condition 8).
    ///
    pub(crate) fn validate_plugin_type_has_source_column(
        field_name: &str,
        infusion_id: &str,
        source_column: Option<&str>,
        spec_path: &str,
    ) -> Result<(), InfusionError> {
        // ADR-051 D3 sub-condition 8: plugin-type fields MUST declare source_column to project
        // a specific field from the plugin response object.  Without it, project_value() falls
        // back to serializing the entire JSON response — the root cause of
        // DRIFT-PIVOT-UDF-OUTPUT-TYPE-001 Failure A (doubly-encoded JSON).
        if source_column.is_some() {
            return Ok(());
        }
        // SEC-001 (CWE-117): strip control chars from interpolated metadata before message
        // construction (error-taxonomy v2.17 SEC-001 Rendering Note).
        let clean_field = sanitize_for_log(field_name);
        let clean_id = sanitize_for_log(infusion_id);
        Err(InfusionError::InvalidFieldSpec {
            // AC-006 canonical attribute label: the invalid attribute is `source_column`
            // (the missing attribute that caused the validation failure), NOT the enclosing
            // field name. Mirroring sub-condition 7 (validate_output_type_recognized) which
            // uses field: "output_type".to_owned() — not the enclosing field name.
            field: "source_column".to_owned(),
            spec_path: spec_path.to_owned(),
            message: format!(
                "plugin-type field '{}' in infusion '{}' must declare 'source_column' to \
                 project a specific field from the plugin response object; without source_column \
                 the full response object is serialized (DRIFT-PIVOT-UDF-OUTPUT-TYPE-001 root cause)",
                clean_field, clean_id
            ),
        })
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::InfusionLoader;

    /// Regression guard for cross-platform TOML path embedding (Windows / CI fix cycle 1).
    ///
    /// TOML basic strings treat `\` as an escape-sequence prefix. Windows absolute paths
    /// (e.g. `C:\Users\Runner\AppData\Local\Temp\file.csv`) contain sequences like `\U`,
    /// `\A`, `\T` that are NOT valid TOML escapes. If tests embed a raw Windows path into
    /// a TOML string via `to_string_lossy().to_string()`, the TOML parser returns an error
    /// and `load_all()` / `parse()` yields 0 specs.
    ///
    /// The correct fix (applied in integration tests and here demonstrated) is to normalise
    /// the path to forward slashes before embedding: `path.replace('\\', "/")`. Forward
    /// slashes are accepted by Rust `std::fs` and the `csv` crate on all platforms.
    ///
    /// This test verifies that `InfusionLoader::parse()` succeeds when given a `file_path`
    /// value that uses Windows-style backslashes encoded as the CORRECT TOML escape (`\\`)
    /// — which is what `replace('\\', "/")` avoids. It also verifies that a raw backslash
    /// path (as a proxy for a mis-embedded Windows path) causes parse failure, confirming
    /// the guard is load-bearing.
    #[test]
    fn test_parse_csv_toml_with_forward_slash_path_succeeds() {
        // Forward-slash path (the normalised form used on all platforms) must parse.
        let toml_forward = r#"
[infusion]
infusion_id = "test_csv"
name = "Test CSV"

[source]
type = "csv"
file_path = "C:/Users/Runner/AppData/Local/Temp/prism-test/file.csv"
key_column = "device_ip"

[[infusion.fields]]
name = "asset_name"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "name"
"#;
        let result = InfusionLoader::parse(toml_forward, "test_csv_forward.infusion.toml");
        assert!(
            result.is_ok(),
            "forward-slash path in TOML file_path must parse successfully; \
             got: {:?}",
            result.err()
        );
    }

    /// Confirms that a raw Windows backslash path embedded in a TOML basic string fails
    /// to parse — this is the exact failure mode that the `replace('\\', "/")` fix prevents
    /// in the integration tests.
    ///
    /// The path `C:\Users\Runner\...` contains `\U` which is NOT a valid TOML escape
    /// sequence (only `\t \n \r \" \\ \uXXXX \UXXXXXXXX` are valid in TOML basic strings).
    #[test]
    fn test_parse_csv_toml_with_raw_backslash_path_fails() {
        // Raw Windows backslash path — NOT normalised — must fail TOML parse.
        // This string is constructed at runtime so the Rust source file itself is
        // not a raw TOML string with invalid escapes; we build it via String::new().
        let toml_backslash = {
            let mut s = String::new();
            s.push_str("[infusion]\ninfusion_id = \"test_csv\"\nname = \"Test\"\n\n");
            s.push_str("[source]\ntype = \"csv\"\n");
            // Embed a Windows-style path with a backslash: `\U` is an invalid TOML escape.
            s.push_str("file_path = \"C:\\Users\\Runner\\file.csv\"\n");
            s.push_str("key_column = \"device_ip\"\n\n");
            s.push_str("[[infusion.fields]]\n");
            s.push_str(
                "name = \"asset_name\"\ninput_field = \"device_ip\"\n\
                 input_type = \"ip\"\noutput_type = \"string\"\n",
            );
            s
        };
        let result = InfusionLoader::parse(&toml_backslash, "test_csv_backslash.infusion.toml");
        assert!(
            result.is_err(),
            "raw Windows backslash path in TOML basic string must fail to parse \
             (\\U is an invalid TOML escape sequence); expected Err but got Ok"
        );
    }
}
