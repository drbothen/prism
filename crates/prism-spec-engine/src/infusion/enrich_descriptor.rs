//! EnrichStageDescriptor — exported descriptor for the `| enrich` PrismQL pipe stage.
//!
//! Actual RecordBatch manipulation (extracting values, calling InfusionSource::enrich_batch,
//! joining columns) lives in prism-query (S-3.02). This crate MUST NOT import Arrow (AD-015).

/// Descriptor for the `ENRICH <infusion_name> ON <field_ref>` PrismQL pipe stage.
///
/// Produced by `InfusionRegistry::enrich_descriptor(name)` (BC-2.19.001 / AC-3).
/// Consumed by prism-query (S-3.02) to execute the enrich transformation.
///
/// # `#[non_exhaustive]` note
/// Marked `#[non_exhaustive]` so that future fields (e.g., per-stage timeout hints,
/// a `description` string, or batch-size metadata) can be added without a
/// semver-breaking change. External callers MUST use `EnrichStageDescriptor::new(...)`
/// rather than struct-literal construction (E0639 will fire otherwise).
#[non_exhaustive]
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

impl EnrichStageDescriptor {
    /// Construct an `EnrichStageDescriptor`.
    ///
    /// Required because `#[non_exhaustive]` prevents struct-literal construction from
    /// outside `prism-spec-engine`. (CLAUDE.md `#[non_exhaustive]` discipline)
    pub fn new(
        infusion_name: impl Into<String>,
        input_field: impl Into<String>,
        output_columns: Vec<String>,
        infusion_id: impl Into<String>,
    ) -> Self {
        Self {
            infusion_name: infusion_name.into(),
            input_field: input_field.into(),
            output_columns,
            infusion_id: infusion_id.into(),
        }
    }
}
