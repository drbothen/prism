//! Per-sensor page-size constants (BC-3.4.003, D-055).
//!
//! Values sourced from S-3.7.00 DERIVATION.md for Armis and CrowdStrike;
//! from poller-bear and poller-express SDK constants for Claroty and Cyberint.

use prism_core::SensorId;

/// Returns the default page size for the given sensor (BC-3.4.003 / D-055).
///
/// Used by `PaginationEdgeCases` archetype to compute `page_size × 3` baseline
/// record count. Values sourced from S-3.7.00 DERIVATION.md (Armis, CrowdStrike)
/// and poller-bear / poller-express SDK constants (Claroty, Cyberint).
pub fn default_page_size(sensor: SensorId) -> usize {
    match sensor.as_ref() {
        // poller-bear SDK default: 100 (per .references/poller-bear/docs/specs.json)
        "claroty" => 100,
        // Armis AQL default: 100 (per S-3.7.00 DERIVATION.md)
        "armis" => 100,
        // CrowdStrike FQL default: 100 (per S-3.7.00 DERIVATION.md)
        "crowdstrike" => 100,
        // poller-express SDK default: 100 (per .references/poller-express/docs/specs/)
        "cyberint" => 100,
        // Unknown sensors: default page size
        _ => 100,
    }
}
