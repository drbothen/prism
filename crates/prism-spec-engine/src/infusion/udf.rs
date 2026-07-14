//! InfusionUdfDescriptor — exported UDF descriptor for prism-query DataFusion registration.
//!
//! prism-spec-engine produces these descriptors; prism-query (S-DEMO-ENRICHMENT-PIVOT-001)
//! creates the actual `datafusion::logical_expr::AsyncScalarUDF` and registers it in the
//! `SessionContext` via `register_infusion_udfs` (BC-2.19.001).
//!
//! This crate MUST NOT import DataFusion (AD-015). The descriptor carries the source
//! (`Arc<dyn InfusionSource>`) so prism-query can invoke enrichment without knowing the
//! concrete source type.
//!
//! # Implementation status (S-DEMO-ENRICHMENT-PIVOT-001 — fully implemented)
//! `InfusionUdfDescriptor` is actively consumed by prism-query's `InfusionAsyncUdf` wrapper.
//! Plugin-type descriptors are built by `InfusionRegistry::load_spec_with_runtime` with a
//! real `Arc<PluginInfusionSource>`. `load_spec` returns descriptors carrying the REAL
//! constructed source (same as the stored registry state — OBS-1 fix, S-1.14-REDO).

use std::sync::Arc;

use super::InfusionSource;

/// Descriptor for a DataFusion scalar UDF backed by an infusion enrichment source.
///
/// One descriptor is produced per `[[infusion.fields]]` entry (INV-INFUSE-001 / BC-2.19.001).
/// Consumed by prism-query (S-3.02) to register `datafusion::logical_expr::ScalarUDF`.
///
/// # `#[non_exhaustive]` note
/// Marked `#[non_exhaustive]` so that future fields (e.g., per-UDF rate-limit hints,
/// a `description` string, or additional caching metadata) can be added without a
/// semver-breaking change. External callers MUST use `InfusionUdfDescriptor::new(...)`
/// rather than struct-literal construction (E0639 will fire otherwise).
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct InfusionUdfDescriptor {
    /// UDF name (global within a DataFusion SessionContext).
    /// Example: `"geoip_country"`, `"asset_owner"`.
    pub name: String,
    /// The input type string (e.g., `"ip"`, `"string"`).
    pub input_type: String,
    /// The output type string (e.g., `"string"`, `"boolean"`).
    pub output_type: String,
    /// The infusion_id this UDF belongs to.
    pub infusion_id: String,
    /// Reference to the source backend for lookup.
    pub source: Arc<dyn InfusionSource>,
    /// The source column to extract from the enrichment result.
    pub source_column: Option<String>,
    /// Per-infusion cache TTL (seconds). Sourced from `InfusionSpec::cache_ttl_secs`; default 3600.
    ///
    /// Used by `prism-query::InfusionAsyncUdf` when writing entries to Tier 2 (LRU) and
    /// Tier 3 (RocksDB) after a live source call (BC-2.19.002 / Story Task 6 + Task 8).
    pub cache_ttl_secs: u64,
    /// The input column fed to this UDF (from `InfusionField::input_field`).
    ///
    /// Used by `prism_describe` to build Category-2 enrichment-discovery hints
    /// (BC-2.10.012 §pql_hints, AC-CAT2). Example: `"device_cves_first"` for a
    /// NVD CVE lookup keyed on the device CVE field.
    pub input_field: String,
}

impl InfusionUdfDescriptor {
    /// Construct an `InfusionUdfDescriptor`.
    ///
    /// Required because `#[non_exhaustive]` prevents struct-literal construction from
    /// outside `prism-spec-engine`. (CLAUDE.md `#[non_exhaustive]` discipline)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        input_type: impl Into<String>,
        output_type: impl Into<String>,
        infusion_id: impl Into<String>,
        source: Arc<dyn InfusionSource>,
        source_column: Option<String>,
        cache_ttl_secs: u64,
        input_field: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            input_type: input_type.into(),
            output_type: output_type.into(),
            infusion_id: infusion_id.into(),
            source,
            source_column,
            cache_ttl_secs,
            input_field: input_field.into(),
        }
    }
}
