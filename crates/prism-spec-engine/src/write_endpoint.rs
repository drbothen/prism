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
// WriteEndpointRegistry — stub (all methods unimplemented!)
// ---------------------------------------------------------------------------

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
    // Implementation detail hidden; use public methods.
    _entries: std::collections::HashMap<(String, String), WriteEndpointSpec>,
}

impl WriteEndpointRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        WriteEndpointRegistry {
            _entries: std::collections::HashMap::new(),
        }
    }

    /// Register write endpoints from a loaded sensor spec.
    ///
    /// Returns `Err` if any verb collides with an already-registered global verb
    /// (global uniqueness enforcement per BC-2.16.009 EC-002).
    ///
    /// # STUB — implement in S-1.13
    pub fn register(
        &mut self,
        sensor_id: &str,
        endpoints: Vec<WriteEndpointSpec>,
    ) -> Result<(), Vec<SpecError>> {
        unimplemented!(
            "WriteEndpointRegistry::register — implement in S-1.13 (BC-2.16.001, BC-2.16.009)"
        )
    }

    /// Look up a write endpoint by (sensor_id, verb).
    ///
    /// Returns `None` if the sensor or verb is not registered.
    ///
    /// # STUB — implement in S-1.13
    pub fn get(&self, sensor: &str, verb: &str) -> Option<&WriteEndpointSpec> {
        unimplemented!(
            "WriteEndpointRegistry::get — implement in S-1.13 (BC-2.16.001)"
        )
    }

    /// Return all registered verbs for a given sensor, in insertion order.
    ///
    /// Used by the PrismQL parser (S-3.06) to build dynamic Chumsky grammar productions.
    ///
    /// # STUB — implement in S-1.13
    pub fn verbs_for_sensor(&self, sensor: &str) -> Vec<&str> {
        unimplemented!(
            "WriteEndpointRegistry::verbs_for_sensor — implement in S-1.13 (BC-2.16.001)"
        )
    }

    /// Export `WriteTableDescriptor` for every registered write endpoint.
    ///
    /// Consumed by prism-query (S-3.07) for DataFusion catalog registration.
    ///
    /// # STUB — implement in S-1.13
    pub fn table_descriptors(&self) -> Vec<WriteTableDescriptor> {
        unimplemented!(
            "WriteEndpointRegistry::table_descriptors — implement in S-1.13 (BC-2.16.001)"
        )
    }

    /// Total number of registered write endpoints across all sensors.
    ///
    /// # STUB — implement in S-1.13
    pub fn len(&self) -> usize {
        unimplemented!("WriteEndpointRegistry::len — implement in S-1.13")
    }

    /// Returns `true` if no write endpoints are registered.
    pub fn is_empty(&self) -> bool {
        self._entries.is_empty()
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
///
/// # STUB — implement in S-1.13
pub fn validate_write_endpoints(
    sensor_id: &str,
    endpoints: &[WriteEndpointSpec],
) -> WriteValidatorOutput {
    unimplemented!(
        "validate_write_endpoints — implement in S-1.13 (BC-2.16.009)"
    )
}

/// Check that a `pipe_verb` does not collide with any reserved PrismQL keyword.
///
/// Returns `Some(SpecError { code: ESpec011, ... })` on collision, `None` if clean.
///
/// # STUB — implement in S-1.13
pub fn check_reserved_keyword(
    verb: &str,
    sensor_id: &str,
    toml_path: Option<&str>,
) -> Option<SpecError> {
    unimplemented!(
        "check_reserved_keyword — implement in S-1.13 (BC-2.16.009, AC-2)"
    )
}

/// Validate `record_id_field` matches `^[a-z0-9_]+$`.
///
/// Returns `Some(SpecError)` if invalid, `None` if valid.
///
/// # STUB — implement in S-1.13
pub fn validate_record_id_field(
    record_id_field: &str,
    sensor_id: &str,
    verb: &str,
    toml_path: Option<&str>,
) -> Option<SpecError> {
    unimplemented!(
        "validate_record_id_field — implement in S-1.13 (BC-2.16.009, EC-005)"
    )
}
