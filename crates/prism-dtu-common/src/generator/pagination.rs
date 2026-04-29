//! Per-sensor page-size constants (BC-3.4.003, D-055).
//!
//! Values sourced from S-3.7.00 DERIVATION.md for Armis and CrowdStrike;
//! from poller-bear and poller-express SDK constants for Claroty and Cyberint.

use prism_core::types::SensorType;

/// Returns the default page size for the given sensor (BC-3.4.003 / D-055).
///
/// Used by `PaginationEdgeCases` archetype to compute `page_size × 3` baseline
/// record count.
pub fn default_page_size(_sensor: SensorType) -> usize {
    todo!()
}
