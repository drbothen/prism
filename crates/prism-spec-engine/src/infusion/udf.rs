//! InfusionUdfDescriptor — exported UDF descriptor for prism-query DataFusion registration.
//!
//! prism-spec-engine produces these descriptors; prism-query (S-3.02) creates the actual
//! `datafusion::logical_expr::ScalarUDF` and registers it in the `SessionContext`.
//! This crate MUST NOT import DataFusion (AD-015).
//!
//! # Stub
//! All methods are `unimplemented!()` — implementation in S-1.14.

use std::sync::Arc;

use super::InfusionSource;

/// Descriptor for a DataFusion scalar UDF backed by an infusion enrichment source.
///
/// One descriptor is produced per `[[infusion.fields]]` entry (INV-INFUSE-001 / BC-2.19.001).
/// Consumed by prism-query (S-3.02) to register `datafusion::logical_expr::ScalarUDF`.
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
}
