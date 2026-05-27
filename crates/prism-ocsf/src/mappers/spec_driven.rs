//! Spec-driven OCSF field mapper — reads `ocsf_field` column annotations from the
//! spec-catalog and dispatches to WASM plugins for complex transforms.
//!
//! # Architecture (PLUGIN-MIGRATION-001-C)
//!
//! `SpecDrivenMapper` replaces the four hardcoded sensor mapper modules
//! (crowdstrike, cyberint, claroty, armis) with a single config-driven implementation
//! that reads `ocsf_field` column annotations from `TableSpec` / `ColumnSpec` at
//! runtime and dispatches complex transforms to WASM plugins via `PluginRuntime`.
//!
//! # Behavioral Contracts
//!
//! - VP-PLUGIN-006: spec-driven field mapping correctness

use std::sync::Arc;

use prism_core::PrismError;
use prism_spec_engine::{ConfigManager, PluginRuntime};
use prost_reflect::DynamicMessage;

/// Spec-driven OCSF field mapper.
///
/// Holds an `Arc<ConfigManager>` for live sensor-spec lookup (supports hot reload)
/// and an `Arc<PluginRuntime>` for WASM plugin dispatch on complex transforms.
///
/// The mapper reads `ocsf_field` column annotations from the spec-catalog
/// `TableSpec` / `ColumnSpec` at `map()` call time, so spec reloads are
/// reflected on the next call without restart.
// Fields are wired in `new()` and read in `map()`. Allow dead_code for the stub phase —
// the implementer will read these fields; this suppression is removed at implementation time.
#[allow(dead_code)]
pub struct SpecDrivenMapper {
    /// Live spec catalog — supports ArcSwap hot reload per ADR-007 / AD-018.
    config_manager: Arc<ConfigManager>,
    /// WASM plugin runtime for complex field transforms.
    plugin_runtime: Arc<PluginRuntime>,
}

impl SpecDrivenMapper {
    /// Construct a `SpecDrivenMapper` with the given config manager and plugin runtime.
    ///
    /// # Parameters
    ///
    /// - `config_manager`: live spec catalog (Arc<ConfigManager> per ADR-022 Arc-DI wiring).
    /// - `plugin_runtime`: WASM plugin runtime for dispatching complex transforms.
    pub fn new(_config_manager: Arc<ConfigManager>, _plugin_runtime: Arc<PluginRuntime>) -> Self {
        todo!()
    }
}

impl super::SensorMapper for SpecDrivenMapper {
    /// Returns the sensor identifier for this mapper.
    ///
    /// For the spec-driven mapper this is looked up from the spec-catalog at dispatch time.
    fn sensor_id(&self) -> &'static str {
        todo!()
    }

    /// Returns the record types this mapper handles.
    ///
    /// For the spec-driven mapper these are derived from the spec-catalog `TableSpec` entries.
    fn record_types(&self) -> &'static [&'static str] {
        todo!()
    }

    /// Maps vendor-specific fields from `raw` into `msg` (known OCSF paths) and
    /// `extensions` (unknown fields) using `ocsf_field` annotations from the spec-catalog.
    ///
    /// Complex transforms are dispatched to WASM plugins via `PluginRuntime`.
    ///
    /// # Errors
    ///
    /// - `PrismError::OcsfNormalizationFailed` — required field missing or type mismatch.
    /// - `PrismError::OcsfTimestampParseError` — timestamp could not be parsed.
    /// - `PrismError::OcsfUnknownRecordType` — this mapper does not handle `record_type`.
    fn map(
        &self,
        _record_type: &str,
        _raw: &serde_json::Value,
        _msg: &mut DynamicMessage,
        _extensions: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<String, PrismError> {
        todo!()
    }
}
