//! EnrichStageDescriptor — exported descriptor for the `| enrich` PrismQL pipe stage.
//!
//! Actual RecordBatch manipulation (extracting values, calling InfusionSource::enrich_batch,
//! joining columns) lives in prism-query (S-3.02). This crate MUST NOT import Arrow (AD-015).
//!
//! # Stub
//! All methods are `unimplemented!()` — implementation in S-1.14.

/// Descriptor for the `ENRICH <infusion_name> ON <field_ref>` PrismQL pipe stage.
///
/// Produced by `InfusionRegistry::enrich_descriptor(name)` (BC-2.19.001 / AC-3).
/// Consumed by prism-query (S-3.02) to execute the enrich transformation.
#[derive(Debug, Clone)]
pub struct EnrichStageDescriptor {
    /// The infusion name (e.g., `"geoip"`).
    pub infusion_name: String,
    /// The input column from the upstream result to pass to the infusion source.
    pub input_field: String,
    /// The column names added to the upstream result schema (from `pipe_stage.adds_columns`).
    /// Must match the `[[infusion.fields]]` names declared in the spec.
    pub output_columns: Vec<String>,
    /// The infusion_id of the backing infusion spec.
    pub infusion_id: String,
}
