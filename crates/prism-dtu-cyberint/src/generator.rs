//! Cyberint fixture generator — all 8 archetypes across 4 API surfaces.
//!
//! Implements `generate(org_id, archetype, opts) -> FixtureSet` using a single
//! deterministic `ChaCha20Rng` stream that advances sequentially through the
//! alert, ASM asset, CVE, and IOC surfaces (EC-003 / BC-3.4.001 invariant 2).
//!
//! Gated behind `#[cfg(feature = "fixture-gen")]` — never compiled into production
//! (AC-007 / D-056).
//!
//! Per-surface baselines at `scale = 1.0` (AC-001):
//! - `HealthyOtEnvironment` : alert=5, asm_asset=10, cve=5, ioc=5
//! - `CompromisedEndpoint`  : alert=20 (≥3 high-severity), asm_asset=10, cve=10, ioc=10
//! - `AuthOutage`           : alert=5, asm_asset=10, cve=5, ioc=5  (same as Healthy)
//! - `LargeScale`           : alert=500, asm_asset=2000, cve=1000, ioc=1000
//! - `PaginationEdgeCases`  : alert surface paginated; others single-page
//! - `SchemaDrift`          : alert[0] intentionally invalid; other surfaces valid
//! - `HighChurn`            : alert=20, asm_asset=30, cve=10, ioc=15 (+ tombstones)
//! - `DormantTenant`        : all surfaces empty (EC-001)
//!
//! Source specs (read-only, test-only validation):
//! - `.references/poller-express/docs/specs/alert_api_specs.json`
//! - `.references/poller-express/docs/specs/asm_assets_api_specs.json`
//! - `.references/poller-express/docs/specs/cve_api_specs.json`
//! - `.references/poller-express/docs/specs/ioc_api_specs.json`

// Stub module — suppress dead_code / unused warnings until the implementer fills in
// the todo!() bodies. These allows are removed when the implementation lands.
#![allow(dead_code, unused_variables, unused_imports)]

use prism_core::types::SensorType;
use prism_dtu_common::{gen_seeded_rng, Archetype, FixtureSet, GenOpts, OrgId, Provenance};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a `FixtureSet` covering all 4 Cyberint API surfaces for the given
/// `org_id` and `archetype`.
///
/// BC-3.4.001: identical inputs produce byte-identical records.
/// BC-3.4.002: each record validates against its surface-specific sub-spec.
/// BC-3.4.004: every record ID carries an org-derived prefix.
///
/// The single RNG stream advances in surface order: alert → asm_asset → cve → ioc.
/// Records from all 4 surfaces are concatenated into `FixtureSet::records` with
/// a `_surface` provenance field to identify origin.
pub fn generate(org_id: &OrgId, archetype: Archetype, opts: &GenOpts) -> FixtureSet {
    todo!("generate: build 4-surface FixtureSet with seeded RNG stream (BC-3.4.001/002/004)")
}

// ---------------------------------------------------------------------------
// Per-surface sub-generators (internal)
// ---------------------------------------------------------------------------

/// Generate alert records for the given archetype baseline and seed state.
///
/// Alert record IDs follow the format `alert-{org_slug}-{seed}-{index}` (AC-004).
/// `SchemaDrift`: record at index 0 is intentionally malformed (AC-003).
fn generate_alerts(
    org_slug: &str,
    seed: u64,
    archetype: Archetype,
    scale: f64,
    rng: &mut rand_chacha::ChaCha20Rng,
) -> Vec<serde_json::Value> {
    todo!("generate_alerts: produce alert records with org-tagged IDs (BC-3.4.004, AC-003)")
}

/// Generate ASM asset records.
///
/// Asset record IDs follow the format `dev-{org_slug}-{seed}-{index}` (AC-004).
fn generate_asm_assets(
    org_slug: &str,
    seed: u64,
    archetype: Archetype,
    scale: f64,
    rng: &mut rand_chacha::ChaCha20Rng,
) -> Vec<serde_json::Value> {
    todo!("generate_asm_assets: produce ASM asset records with org-tagged IDs (BC-3.4.004)")
}

/// Generate CVE records.
///
/// CVE record primary ID follows the format `alert-{org_slug}-{seed}-{index}` (AC-004).
fn generate_cves(
    org_slug: &str,
    seed: u64,
    archetype: Archetype,
    scale: f64,
    rng: &mut rand_chacha::ChaCha20Rng,
) -> Vec<serde_json::Value> {
    todo!("generate_cves: produce CVE records with org-tagged IDs (BC-3.4.004)")
}

/// Generate IOC records.
///
/// IOC record primary ID follows the format `alert-{org_slug}-{seed}-{index}` (AC-004).
fn generate_iocs(
    org_slug: &str,
    seed: u64,
    archetype: Archetype,
    scale: f64,
    rng: &mut rand_chacha::ChaCha20Rng,
) -> Vec<serde_json::Value> {
    todo!("generate_iocs: produce IOC records with org-tagged IDs (BC-3.4.004)")
}

// ---------------------------------------------------------------------------
// Baseline count helpers (internal)
// ---------------------------------------------------------------------------

/// Per-surface baseline record counts for a given archetype at `scale = 1.0`.
///
/// Returns `(alert, asm_asset, cve, ioc)`. The caller applies
/// `floor(baseline × scale)` for non-unit scale values (AC-001).
fn baselines(archetype: Archetype) -> (usize, usize, usize, usize) {
    todo!("baselines: return (alert, asm_asset, cve, ioc) counts per archetype (AC-001)")
}

/// Derive an org slug from an `OrgId` for use in record ID prefixes.
///
/// Returns the first 8 hex characters of the org UUID (EC-005 fallback path).
fn org_slug(org_id: &OrgId) -> String {
    todo!("org_slug: hex-encode first 8 bytes of OrgId for ID prefix (BC-3.4.004)")
}

// ---------------------------------------------------------------------------
// Schema validation (test-only, AC-002 / BC-3.4.002 / AC-007)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod schema_validation {
    use serde_json::Value;

    // Spec paths relative to repo root — loaded at runtime from disk in tests (AC-002).
    // Implementer must resolve these paths via `std::env::var("CARGO_MANIFEST_DIR")`.
    const ALERT_SPEC_PATH: &str = ".references/poller-express/docs/specs/alert_api_specs.json";
    const ASM_ASSETS_SPEC_PATH: &str =
        ".references/poller-express/docs/specs/asm_assets_api_specs.json";
    const CVE_SPEC_PATH: &str = ".references/poller-express/docs/specs/cve_api_specs.json";
    const IOC_SPEC_PATH: &str = ".references/poller-express/docs/specs/ioc_api_specs.json";

    /// Validate an alert record against `alert_api_specs.json`.
    ///
    /// Panics with surface type, spec path, and schema error on violation (AC-002).
    pub(super) fn validate_alert(record: &Value, index: usize) {
        todo!(
            "validate_alert: jsonschema validate record[{}] against alert_api_specs.json (AC-002)",
            index
        )
    }

    /// Validate an ASM asset record against `asm_assets_api_specs.json`.
    pub(super) fn validate_asm_asset(record: &Value, index: usize) {
        todo!(
            "validate_asm_asset: jsonschema validate record[{}] against asm_assets_api_specs.json (AC-002)",
            index
        )
    }

    /// Validate a CVE record against `cve_api_specs.json`.
    pub(super) fn validate_cve(record: &Value, index: usize) {
        todo!(
            "validate_cve: jsonschema validate record[{}] against cve_api_specs.json (AC-002)",
            index
        )
    }

    /// Validate an IOC record against `ioc_api_specs.json`.
    pub(super) fn validate_ioc(record: &Value, index: usize) {
        todo!(
            "validate_ioc: jsonschema validate record[{}] against ioc_api_specs.json (AC-002)",
            index
        )
    }
}

// ---------------------------------------------------------------------------
// Tests (AC-001 … AC-006)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use prism_dtu_common::{all_archetypes, GenOpts};

    /// AC-001: per-surface counts at scale=1.0 for all 8 archetypes.
    #[test]
    fn test_cyberint_all_archetypes_counts() {
        todo!("test_cyberint_all_archetypes_counts: assert per-surface counts for each archetype")
    }

    /// AC-002: each surface validates against its correct sub-spec.
    #[test]
    fn test_cyberint_schema_correct_sub_spec() {
        todo!("test_cyberint_schema_correct_sub_spec: dispatch records to correct spec validator")
    }

    /// AC-003: SchemaDrift — only alert surface record[0] invalid.
    #[test]
    fn test_cyberint_schema_drift_alert_surface() {
        todo!("test_cyberint_schema_drift_alert_surface: assert alert[0] invalid, others valid")
    }

    /// AC-004: all record IDs carry org-slug prefix for correct field per surface.
    #[test]
    fn test_cyberint_org_tagged_ids_per_surface() {
        todo!("test_cyberint_org_tagged_ids_per_surface: assert ID field prefixes per surface")
    }

    /// AC-005: two calls with identical inputs produce byte-identical records.
    #[test]
    fn test_cyberint_determinism() {
        todo!("test_cyberint_determinism: assert two calls → byte-identical records (BC-3.4.001)")
    }

    /// AC-006 / RNG stream: different seed produces different records on all surfaces.
    #[test]
    fn test_cyberint_single_rng_stream() {
        todo!("test_cyberint_single_rng_stream: assert different seed → different alert AND ASM records")
    }
}
