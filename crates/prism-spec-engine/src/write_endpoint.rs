//! Write endpoint spec types and registry (S-1.13, BC-2.16.001, BC-2.16.009).
//!
//! Extends the sensor TOML spec format with `[write_endpoints.{verb}]` sections.
//! Parses, validates, and exports write endpoint descriptors via `WriteEndpointRegistry`
//! for downstream DataFusion registration by prism-query (S-3.07).
//!
//! # Architecture Compliance
//! - MUST NOT depend on DataFusion or Arrow (AD-015).
//! - Exports plain Rust structs only; DataFusion `TableProvider` registration
//!   (`WriteCapableTableProvider`) is prism-query's responsibility (S-3.07).
//! - `WriteEndpointRegistry` is the single source of truth for registered write verbs.
//! - Write verb uniqueness is enforced GLOBALLY (no two sensors can register the same verb).
//!
//! # Subsystem
//! SS-16 — Spec Engine

use prism_core::{RiskTier, SpecError, SpecErrorCode};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Reserved keywords that write pipe_verb must not collide with (BC-2.16.009)
// ---------------------------------------------------------------------------

/// Reserved PrismQL keywords that `pipe_verb` must not shadow.
///
/// Returns E-SPEC-011 if a write endpoint declares a colliding verb.
pub const RESERVED_KEYWORDS: &[&str] = &["where", "sort", "limit", "join", "enrich", "head"];

// ---------------------------------------------------------------------------
// Write endpoint data model
// ---------------------------------------------------------------------------

/// Execution mode for batched write operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BatchMode {
    /// Execute write steps one record at a time, sequentially.
    Serial,
    /// Execute write steps for all records concurrently.
    Parallel,
}

/// Risk tier for write endpoints — mirrors `prism_core::RiskTier` for TOML deserialization.
///
/// Deserialized from the spec's `risk_tier = "reversible"` or `risk_tier = "irreversible"`.
/// See `prism_core::RiskTier` for the canonical type.
pub type RiskTierSpec = RiskTier;

/// A single HTTP step within a write endpoint pipeline.
///
/// Corresponds to `[[write_endpoints.{verb}.steps]]` in the sensor TOML.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteStep {
    /// HTTP method ("POST", "PUT", "PATCH", "DELETE").
    pub method: String,
    /// URL path template; supports `${record_ids}` and `${params.KEY}` interpolation.
    pub url: String,
    /// Optional JSON body template; supports `${record_ids}` and `${params.KEY}`.
    pub body_template: Option<String>,
    /// Optional JSONPath expression to extract a field from the response.
    pub response_path: Option<String>,
}

/// Specification for a single write endpoint on a sensor.
///
/// Corresponds to `[write_endpoints.{verb}]` in the sensor TOML.
///
/// # Validation (BC-2.16.009)
/// - `pipe_verb` must not collide with RESERVED_KEYWORDS → E-SPEC-011
/// - `pipe_verb` must be unique across all sensors
/// - `risk_tier` must be "reversible" or "irreversible"
/// - `batch_limit = 0` + `risk_tier = Irreversible`: structured warning (not error)
/// - `steps` must be non-empty
/// - `record_id_field` must match `[a-z0-9_]+`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteEndpointSpec {
    /// Verb name used in PrismQL pipe stages (e.g., `contain`, `acknowledge`).
    pub pipe_verb: String,
    /// DataFusion table name for write operations (write-only, not queryable as read).
    pub sql_table: String,
    /// Risk classification (AD-022).
    pub risk_tier: RiskTierSpec,
    /// Dot-path capability identifier (e.g., `"crowdstrike.hosts.write"`).
    pub capability_path: String,
    /// Maximum number of records per batch. 0 = unlimited (with warning if irreversible).
    pub batch_limit: u32,
    /// Whether steps execute serially or in parallel for a batch.
    pub batch_mode: BatchMode,
    /// Column name used as the record identifier in `${record_ids}` expansion.
    pub record_id_field: String,
    /// HTTP steps to execute for this write operation (must be non-empty).
    pub steps: Vec<WriteStep>,
}

/// Descriptor exported from a loaded write endpoint for downstream consumption.
///
/// prism-query (S-3.07) uses these to register DataFusion write-capable TableProviders.
/// prism-spec-engine MUST NOT import DataFusion — it exports descriptors only (AD-015).
#[derive(Debug, Clone, PartialEq)]
pub struct WriteTableDescriptor {
    /// DataFusion table name for this write endpoint.
    pub sql_table: String,
    /// Always `true` for write endpoint tables — not queryable as a read source.
    pub write_only: bool,
    /// The sensor_id that owns this write endpoint.
    pub sensor: String,
    /// The pipe_verb for this write endpoint (e.g., `"contain"`).
    pub verb: String,
    /// Risk tier for confirmation gating.
    pub risk_tier: RiskTierSpec,
}

// ---------------------------------------------------------------------------
// WriteEndpointRegistry — full implementation (BC-2.16.001)
// ---------------------------------------------------------------------------

/// Internal entry storing spec + sensor metadata, preserving insertion order.
struct RegistryEntry {
    sensor: String,
    verb: String,
    spec: WriteEndpointSpec,
}

/// Registry of all write endpoints loaded from sensor specs (BC-2.16.001).
///
/// Maps `(sensor_id, verb) -> WriteEndpointSpec`.
/// Enforces global verb uniqueness across all sensors.
/// Consumed by prism-query (S-3.07) for DataFusion write table registration.
///
/// # Invariants
/// - A verb is registered at most once globally (no two sensors own the same verb).
/// - `verbs_for_sensor` returns verbs in insertion order.
pub struct WriteEndpointRegistry {
    /// Ordered list of entries, preserving insertion order for `verbs_for_sensor`.
    entries: Vec<RegistryEntry>,
    /// Global verb → sensor_id map for uniqueness enforcement.
    global_verbs: std::collections::HashMap<String, String>,
}

impl WriteEndpointRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        WriteEndpointRegistry {
            entries: Vec::new(),
            global_verbs: std::collections::HashMap::new(),
        }
    }

    /// Register write endpoints from a loaded sensor spec.
    ///
    /// Returns `Err` if any verb collides with an already-registered global verb
    /// (global uniqueness enforcement per BC-2.16.009 EC-002).
    pub fn register(
        &mut self,
        sensor_id: &str,
        endpoints: Vec<WriteEndpointSpec>,
    ) -> Result<(), Vec<SpecError>> {
        let mut errors: Vec<SpecError> = Vec::new();

        // Pre-check: verify no global uniqueness collision before inserting any entries.
        for endpoint in &endpoints {
            if let Some(existing_sensor) = self.global_verbs.get(&endpoint.pipe_verb) {
                errors.push(SpecError {
                    code: SpecErrorCode::ESpec009,
                    message: format!(
                        "pipe_verb '{}' for sensor '{}' already registered by sensor '{}' — \
                         write verb must be globally unique (BC-2.16.009 EC-002)",
                        endpoint.pipe_verb, sensor_id, existing_sensor
                    ),
                    toml_path: Some(format!("write_endpoints.{}.pipe_verb", endpoint.pipe_verb)),
                    file_path: None,
                    line_number: None,
                });
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Insert all endpoints (no conflicts found).
        for endpoint in endpoints {
            self.global_verbs
                .insert(endpoint.pipe_verb.clone(), sensor_id.to_string());
            self.entries.push(RegistryEntry {
                sensor: sensor_id.to_string(),
                verb: endpoint.pipe_verb.clone(),
                spec: endpoint,
            });
        }

        Ok(())
    }

    /// Look up a write endpoint by (sensor_id, verb).
    ///
    /// Returns `None` if the sensor or verb is not registered.
    pub fn get(&self, sensor: &str, verb: &str) -> Option<&WriteEndpointSpec> {
        self.entries
            .iter()
            .find(|e| e.sensor == sensor && e.verb == verb)
            .map(|e| &e.spec)
    }

    /// Return all registered verbs for a given sensor, in insertion order.
    ///
    /// Used by the PrismQL parser (S-3.06) to build dynamic Chumsky grammar productions.
    pub fn verbs_for_sensor(&self, sensor: &str) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|e| e.sensor == sensor)
            .map(|e| e.verb.as_str())
            .collect()
    }

    /// Export `WriteTableDescriptor` for every registered write endpoint.
    ///
    /// Consumed by prism-query (S-3.07) for DataFusion catalog registration.
    pub fn table_descriptors(&self) -> Vec<WriteTableDescriptor> {
        self.entries
            .iter()
            .map(|e| WriteTableDescriptor {
                sql_table: e.spec.sql_table.clone(),
                write_only: true,
                sensor: e.sensor.clone(),
                verb: e.verb.clone(),
                risk_tier: e.spec.risk_tier.clone(),
            })
            .collect()
    }

    /// Total number of registered write endpoints across all sensors.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if no write endpoints are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for WriteEndpointRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Write endpoint validation (BC-2.16.009)
// ---------------------------------------------------------------------------

/// A non-fatal warning emitted during write endpoint validation.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteValidationWarning {
    /// Human-readable warning message.
    pub message: String,
    /// TOML path identifying the problematic field.
    pub toml_path: Option<String>,
}

/// Result of write endpoint validation.
///
/// - `Ok(warnings)` — all endpoints valid; warnings may be present
/// - `Err(errors)` — one or more endpoints invalid; all errors collected (VP-059 invariant)
pub type WriteValidatorOutput = Result<Vec<WriteValidationWarning>, Vec<SpecError>>;

/// Validate a slice of write endpoint specs for a single sensor (BC-2.16.009).
///
/// Validation rules applied:
/// 1. `pipe_verb` must not match any entry in RESERVED_KEYWORDS → E-SPEC-011
/// 2. `pipe_verb` must be unique within this sensor (cross-sensor checked at register time)
/// 3. `risk_tier` deserialization enforced by `RiskTierSpec` enum (parse error on invalid value)
/// 4. `batch_limit = 0` + `risk_tier = Irreversible` → structured warning (not error)
/// 5. `steps` must be non-empty → E-SPEC-001
/// 6. `record_id_field` must match `^[a-z0-9_]+$` → E-SPEC-001
///
/// All-errors-collected, no fail-fast (VP-059 invariant).
pub fn validate_write_endpoints(
    sensor_id: &str,
    endpoints: &[WriteEndpointSpec],
) -> WriteValidatorOutput {
    let mut errors: Vec<SpecError> = Vec::new();
    let mut warnings: Vec<WriteValidationWarning> = Vec::new();

    // Track within-sensor verb uniqueness.
    let mut seen_verbs: std::collections::HashSet<&str> = std::collections::HashSet::new();

    for endpoint in endpoints {
        let verb = endpoint.pipe_verb.as_str();
        let toml_prefix = format!("write_endpoints.{}", verb);

        // Rule 1: pipe_verb must not collide with reserved keywords → E-SPEC-011
        if let Some(err) =
            check_reserved_keyword(verb, sensor_id, Some(&format!("{toml_prefix}.pipe_verb")))
        {
            errors.push(err);
        }

        // Rule 2: within-sensor verb uniqueness
        if !seen_verbs.insert(verb) {
            errors.push(SpecError {
                code: SpecErrorCode::ESpec004,
                message: format!(
                    "duplicate pipe_verb '{}' within sensor '{}' — verbs must be unique per sensor",
                    verb, sensor_id
                ),
                toml_path: Some(format!("{toml_prefix}.pipe_verb")),
                file_path: None,
                line_number: None,
            });
        }

        // Rule 4: batch_limit=0 + risk_tier=Irreversible → warning (not error)
        if endpoint.batch_limit == 0 && endpoint.risk_tier == RiskTier::Irreversible {
            warnings.push(WriteValidationWarning {
                message: format!(
                    "sensor '{}' write endpoint '{}': batch_limit=0 (unlimited) combined with \
                     risk_tier=irreversible is dangerous — no upper bound on records modified per \
                     operation; consider setting an explicit batch_limit",
                    sensor_id, verb
                ),
                toml_path: Some(format!("{toml_prefix}.batch_limit")),
            });
        }

        // Rule 5: steps must be non-empty
        if endpoint.steps.is_empty() {
            errors.push(SpecError {
                code: SpecErrorCode::ESpec001,
                message: format!(
                    "sensor '{}' write endpoint '{}': steps array must not be empty — \
                     at least one HTTP step is required (BC-2.16.009 EC-004)",
                    sensor_id, verb
                ),
                toml_path: Some(format!("{toml_prefix}.steps")),
                file_path: None,
                line_number: None,
            });
        }

        // Rule 6: record_id_field must match ^[a-z0-9_]+$
        if let Some(err) = validate_record_id_field(
            &endpoint.record_id_field,
            sensor_id,
            verb,
            Some(&format!("{toml_prefix}.record_id_field")),
        ) {
            errors.push(err);
        }
    }

    if errors.is_empty() {
        Ok(warnings)
    } else {
        Err(errors)
    }
}

/// Check that a `pipe_verb` does not collide with any reserved PrismQL keyword.
///
/// Returns `Some(SpecError { code: ESpec011, ... })` on collision, `None` if clean.
pub fn check_reserved_keyword(
    verb: &str,
    sensor_id: &str,
    toml_path: Option<&str>,
) -> Option<SpecError> {
    if RESERVED_KEYWORDS.contains(&verb) {
        Some(SpecError {
            code: SpecErrorCode::ESpec011,
            message: format!(
                "sensor '{}': pipe_verb '{}' collides with reserved PrismQL keyword — \
                 choose a different verb (reserved: {:?}) (BC-2.16.009 E-SPEC-011)",
                sensor_id, verb, RESERVED_KEYWORDS
            ),
            toml_path: toml_path.map(|p| p.to_string()),
            file_path: None,
            line_number: None,
        })
    } else {
        None
    }
}

/// Validate `record_id_field` matches `^[a-z0-9_]+$`.
///
/// Returns `Some(SpecError)` if invalid, `None` if valid.
pub fn validate_record_id_field(
    record_id_field: &str,
    sensor_id: &str,
    verb: &str,
    toml_path: Option<&str>,
) -> Option<SpecError> {
    // Must be non-empty and match [a-z0-9_]+
    let valid = !record_id_field.is_empty()
        && record_id_field
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

    if !valid {
        Some(SpecError {
            code: SpecErrorCode::ESpec001,
            message: format!(
                "sensor '{}' write endpoint '{}': record_id_field '{}' is invalid — \
                 must match [a-z0-9_]+ (BC-2.16.009 EC-005)",
                sensor_id, verb, record_id_field
            ),
            toml_path: toml_path.map(|p| p.to_string()),
            file_path: None,
            line_number: None,
        })
    } else {
        None
    }
}
