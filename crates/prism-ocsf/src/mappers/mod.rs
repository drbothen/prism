//! Spec-driven OCSF field mapper for sensor normalization.
//!
//! This module defines the `SensorMapper` trait and re-exports `SpecDrivenMapper`,
//! the single config-driven implementation that replaced the four hardcoded
//! per-sensor modules (CrowdStrike, Cyberint, Claroty, Armis) as part of
//! PLUGIN-MIGRATION-001-C.
//!
//! # Architecture (PLUGIN-MIGRATION-001-C)
//!
//! `SpecDrivenMapper` reads `ocsf_field` column annotations from the spec-catalog
//! (`TableSpec` / `ColumnSpec`) at runtime and dispatches complex transforms to
//! WASM plugins via `PluginRuntime`. New sensors are added by writing a TOML spec —
//! no Rust code changes required.
//!
//! The normalizer dispatches via `SensorMapper::sensor_id()` — never via
//! `match sensor {}`. (S-1.05 Architecture Compliance Rules)

use prism_core::PrismError;
use prost_reflect::DynamicMessage;

pub mod spec_driven;

pub use spec_driven::SpecDrivenMapper;

/// Trait implemented by each sensor-specific field mapper.
///
/// The `OcsfNormalizer` holds a `Vec<Box<dyn SensorMapper>>` and selects the
/// appropriate mapper by matching `sensor_id()` against the incoming record's sensor
/// label, and `record_types()` against the incoming `record_type`.
///
/// # Contract (S-1.05 Task 1, BC-2.02.003–006)
///
/// - `map()` MUST populate `msg` with every vendor field that has a defined OCSF path.
/// - `map()` MUST populate `extensions` with every vendor field that has NO OCSF path.
/// - After `map()` returns, the union of fields in `msg` + keys in `extensions` MUST
///   equal the set of all input field keys. (BC-2.02.007, VP-017)
/// - `map()` MUST NOT call `unwrap()` or `expect()` — all errors returned via `Result`.
/// - `map()` returns the source record ID as `Ok(String)` on success.
pub trait SensorMapper: Send + Sync {
    /// Identifies this mapper's sensor (e.g., `"crowdstrike"`, `"cyberint"`).
    fn sensor_id(&self) -> &'static str;

    /// The record types this mapper handles (e.g., `&["detection", "incident"]`).
    fn record_types(&self) -> &'static [&'static str];

    /// Maps vendor-specific fields from `raw` into `msg` (known OCSF paths) and
    /// `extensions` (unknown fields). Returns the source record ID on success.
    ///
    /// # Parameters
    ///
    /// - `record_type`: the vendor record type (e.g., `"detection"`)
    /// - `raw`: the raw vendor JSON object
    /// - `msg`: the empty `DynamicMessage` to populate
    /// - `extensions`: the accumulator for unmapped fields
    ///
    /// # Returns
    ///
    /// `Ok(source_record_id)` — the vendor-specific record identifier string extracted
    /// from `raw` (e.g., CrowdStrike `detection_id`, Cyberint `ref_id`).
    ///
    /// # Errors
    ///
    /// - `PrismError::OcsfNormalizationFailed` — required field missing or type mismatch.
    /// - `PrismError::OcsfTimestampParseError` — timestamp could not be parsed.
    /// - `PrismError::OcsfUnknownRecordType` — this mapper does not handle `record_type`.
    fn map(
        &self,
        record_type: &str,
        raw: &serde_json::Value,
        msg: &mut DynamicMessage,
        extensions: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<String, PrismError>;
}
