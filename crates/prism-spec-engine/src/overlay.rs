//! Per-org sensor endpoint overlay types and loading logic (ADR-029).
//!
//! Implements the hybrid Sensor Instance with Per-Org Composition Directory
//! approach described in ADR-029.  The global `<sensor>.sensor.toml` (TYPE spec)
//! defines the sensor schema, `auth_type`, and a default `base_url`.  Per-org
//! `customers/<org_slug>/<sensor_id>.sensor.toml` INSTANCE overlay files declare
//! scalar-only tunables (primarily `base_url`) for sensors that vary per MSSP
//! client (e.g., Armis Centrix, Claroty on-prem).
//!
//! At boot step 4, overlays are discovered, validated, and merged onto TYPE specs
//! to produce a `ResolvedSensorSpec` per `(org_slug, sensor_id)` pair.  The
//! fanout engine resolves `(org_id, sensor_id)` → `ResolvedSensorSpec` in O(1)
//! at dispatch time (INV-FANOUT-002).
//!
//! # Architecture Compliance
//! - `prism-spec-engine` MUST NOT gain a dependency on `prism-sensors`
//!   (Forbidden Dependencies rule in story §Architecture Compliance Rules).
//! - `OrgRegistry` is passed in from the caller (dependency injection).
//! - `ResolvedSensorSpec` map is read-only after boot (INV-OVL-006).
//! - All new public types carry `#[non_exhaustive]` per CLAUDE.md conventions.
//!
//! # BCs implemented
//! - BC-2.06.012 (Per-Tenant Overlay Loading and Merge Semantics)
//! - BC-2.06.013 (Scalar-Only Overlay Enforcement)
//! - BC-2.06.014 (Instance Identity Resolution at Fanout)
//! - BC-2.06.015 (OrgRegistry Cross-Validation at Boot)
//! - BC-2.06.016 (Error Taxonomy for Override Violations)
//!
//! Story: S-CONFIG-MULTI-TENANT-OVERRIDE-001

use std::collections::HashMap;

// SpecError and SpecErrorCode are not referenced in stub function bodies (todo!())
// but are needed by the implementer to construct E-SPEC-019..E-SPEC-023 errors.
// Allow unused-imports at stub stage; the implementer will use them in real bodies.
#[allow(unused_imports)]
use prism_core::{OrgRegistry, OrgSlug, PrismError, SpecError, SpecErrorCode};
use serde::{Deserialize, Serialize};

use crate::spec_parser::{RateLimitHints, SensorSpec};

// ---------------------------------------------------------------------------
// SensorInstanceOverlay — per-org overlay parsed from customers/<slug>/<sensor>.sensor.toml
// ---------------------------------------------------------------------------

/// A per-org overlay file that tunes scalar fields of a TYPE sensor spec.
///
/// Parsed from `customers/<org_slug>/<sensor_id>.sensor.toml`.  Only scalar
/// fields are permitted (INV-SCALAR-001):
/// - `extends`            — the TYPE spec sensor_id this overlay is based on
/// - `instance_id`        — must equal `{sensor_id}@{org_slug}` (INV-SCALAR-003)
/// - `base_url`           — override the TYPE spec default endpoint (optional)
/// - `timeout_secs`       — HTTP timeout override for this org's instance (optional)
/// - `rate_limit_hints`   — rate limiter tunables for this org's instance (optional)
///
/// `[[tables]]`, `auth_type`, `version`, and `sensor_id` are NEVER permitted in
/// overlay files (INV-OVL-001, INV-OVL-002) — they are always inherited from the
/// TYPE spec (BC-2.06.012 postcondition 1; BC-2.06.013).
///
/// `#[non_exhaustive]`: forward-compat — new tunable scalar fields may be added
/// without a semver bump.  External callers MUST NOT construct this struct
/// directly; use the TOML deserialization path via `SpecLoader::load_all_with_overlays`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorInstanceOverlay {
    /// The TYPE spec sensor_id this overlay extends.
    ///
    /// E.g., `extends = "armis"` links this overlay to `armis.sensor.toml`.
    /// References a non-existent TYPE spec → E-SPEC-019 at boot.
    pub extends: String,

    /// Canonical instance identity in the form `{sensor_id}@{org_slug}`.
    ///
    /// Must exactly equal the value derived from the file path:
    /// `<stem>@<parent_dir_name>`.  Mismatch → E-SPEC-020 at boot.
    pub instance_id: String,

    /// Override the TYPE spec `base_url` for this org's sensor instance.
    ///
    /// E.g., `base_url = "https://armis.acme-corp.io"` routes this org's
    /// Armis queries to their dedicated on-prem instance.  When `None`, the
    /// TYPE spec default `base_url` is used (Case B — backwards-compatible).
    pub base_url: Option<String>,

    /// Optional HTTP timeout override for this org's instance (seconds).
    ///
    /// When `None`, the TYPE spec or global default timeout is used.
    #[serde(default)]
    pub timeout_secs: Option<u64>,

    /// Optional rate limit hints for this org's instance.
    ///
    /// When `None`, the TYPE spec rate limit hints (or no hints) apply.
    #[serde(default)]
    pub rate_limit_hints: Option<RateLimitHints>,
}

// ---------------------------------------------------------------------------
// OverlayProvenance — tracks which fields came from overlay vs TYPE spec
// ---------------------------------------------------------------------------

/// Records which scalar fields in a `ResolvedSensorSpec` came from an overlay
/// vs the TYPE spec.
///
/// Used for `prism config show --sensor <instance_id>` provenance display
/// (AC-001 postcondition; follow-up story S-CONFIG-MULTI-TENANT-OVERRIDE-002).
///
/// `#[non_exhaustive]`: new provenance fields will be added as tunable scalars
/// are extended in later stories.  External callers MUST use `..Default::default()`
/// for construction.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct OverlayProvenance {
    /// Whether `base_url` came from the overlay (true) or TYPE spec (false).
    pub base_url_from_overlay: bool,
    /// Whether `rate_limit_hints.requests_per_second` came from the overlay.
    pub rps_from_overlay: bool,
    /// Whether `rate_limit_hints.burst_size` came from the overlay.
    pub burst_size_from_overlay: bool,
    /// Whether `timeout_secs` came from the overlay.
    pub timeout_secs_from_overlay: bool,
}

// ---------------------------------------------------------------------------
// ResolvedSensorSpec — TYPE spec merged with per-org overlay scalars
// ---------------------------------------------------------------------------

/// A `SensorSpec` with per-org scalar overrides merged in.
///
/// Produced at boot by `SpecLoader::load_all_with_overlays` for each
/// `(org_slug, sensor_id)` pair that has a `customers/<org_slug>/<sensor_id>.sensor.toml`
/// overlay file.
///
/// The `spec` field holds the merged sensor spec:
/// - `base_url` comes from the overlay if `base_url` is `Some(...)` in the overlay;
///   otherwise inherited from TYPE spec.
/// - `rate_limit_hints.requests_per_second` and `burst_size` are individually merged.
/// - `tables`, `auth_type`, `version`, `sensor_id`, `name`, `credential_refs` are
///   ALWAYS from the TYPE spec (INV-OVL-001, INV-OVL-002).
///
/// The `provenance` field records which scalar fields were overridden.
///
/// After boot, this map is read-only (INV-OVL-006); the fanout engine accesses it
/// via `Arc<HashMap<(OrgSlug, SensorId), ResolvedSensorSpec>>` with no mutex on
/// the hot path (INV-FANOUT-002).
///
/// `#[non_exhaustive]`: forward-compat for future provenance fields.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSensorSpec {
    /// The sensor spec with overlay scalars merged in.
    pub spec: SensorSpec,
    /// Tracks which scalar fields came from the overlay vs the TYPE spec.
    pub provenance: OverlayProvenance,
    /// The org slug that owns this resolved instance.
    pub org_slug: OrgSlug,
    /// The instance identity string: `{sensor_id}@{org_slug}`.
    pub instance_id: String,
}

// ---------------------------------------------------------------------------
// Overlay key type
// ---------------------------------------------------------------------------

/// Lookup key for the `ResolvedSensorSpec` map: `(org_slug, sensor_id)`.
///
/// Used by the fanout engine for O(1) dispatch (INV-FANOUT-002).
/// `sensor_id` is a `String` here because `SensorId` from `prism-core` is not
/// used directly in `prism-spec-engine` (no prism-sensors dependency; see
/// Forbidden Dependencies rule in §Architecture Compliance Rules).
pub type ResolvedSpecKey = (OrgSlug, String);

// ---------------------------------------------------------------------------
// Overlay load result
// ---------------------------------------------------------------------------

/// Result of `load_overlays_from_customers_dir`.
///
/// Contains both the successfully resolved specs and any validation errors
/// collected during the walk.  Boot step 4 treats a non-empty `errors` vec
/// as a fatal config error (INV-ERR-003 — collect ALL errors before aborting).
#[derive(Debug)]
pub struct OverlayLoadResult {
    /// All successfully resolved `(org_slug, sensor_id)` → `ResolvedSensorSpec` pairs.
    pub resolved: HashMap<ResolvedSpecKey, ResolvedSensorSpec>,
    /// All validation errors found during the overlay walk (multi-error collection).
    pub errors: Vec<PrismError>,
}

// ---------------------------------------------------------------------------
// OverlayLoader — core overlay discovery, validation, and merge logic
// ---------------------------------------------------------------------------

/// Loads and validates per-org overlay files from a `customers/` directory.
///
/// This is the implementation heart of BC-2.06.012 through BC-2.06.015.  The
/// loader is separate from `SpecLoader` to keep the overlay concern isolated and
/// to allow testing without constructing a full `SpecLoader` instance.
///
/// `OverlayLoader` is intentionally stateless — all inputs are passed as
/// arguments to `load_overlays`.  The `OrgRegistry` reference is dependency-
/// injected from boot step 3 (BC-2.06.015 INV-COMPAT-002).
pub struct OverlayLoader;

// All methods on OverlayLoader have todo!() bodies — parameters are named for the implementer
// but unused at stub stage.  Suppress unused_variable warnings for the impl block.
#[allow(unused_variables)]
impl OverlayLoader {
    /// Walk `customers_dir`, discover overlay files, validate, and merge.
    ///
    /// For each `<org_slug>/<sensor_id>.sensor.toml` found:
    /// 1. Cross-check `<org_slug>` against `OrgRegistry` (E-SPEC-022).
    /// 2. Run structural validator (E-SPEC-019, E-SPEC-020, E-SPEC-021, E-SPEC-023).
    /// 3. If valid, merge overlay scalars onto the TYPE spec to produce `ResolvedSensorSpec`.
    /// 4. Index by `(OrgSlug::from(org_slug), sensor_id)`.
    ///
    /// Multi-error aggregation: ALL errors across all overlay files are collected
    /// before returning.  The caller (boot step 4) must treat a non-empty `errors`
    /// vec as a fatal config error (INV-ERR-003, BC-2.06.016).
    ///
    /// # Arguments
    /// - `customers_dir` — path to the `customers/` directory; if absent, returns empty result.
    /// - `type_specs` — the set of loaded TYPE specs, keyed by `sensor_id`.
    /// - `org_registry` — the `OrgRegistry` built in boot step 3.
    ///
    /// # BC-2.06.012 edge cases
    /// - Absent `customers/` directory → zero overlays, zero errors (EC-012-001).
    /// - Only `customers/.gitkeep` (no subdirectories) → zero overlays, zero errors (EC-012-002).
    /// - Only plain files directly under `customers/` are not treated as slugs (INV-COMPAT-004).
    pub fn load_overlays(
        customers_dir: &std::path::Path,
        type_specs: &HashMap<String, SensorSpec>,
        org_registry: &OrgRegistry,
    ) -> OverlayLoadResult {
        todo!()
    }

    /// Validate that an overlay TOML string is structurally permitted.
    ///
    /// Checks (in order, collecting all errors per INV-ERR-003):
    /// 1. `[[tables]]` blocks → E-SPEC-021.
    /// 2. Unrecognized scalar fields → E-SPEC-023.
    /// 3. `instance_id` format mismatch (`{sensor_id}@{org_slug}`) → E-SPEC-020.
    /// 4. `extends` references unknown TYPE spec → E-SPEC-019.
    ///
    /// Returns `Ok(SensorInstanceOverlay)` when all checks pass, or
    /// `Err(Vec<PrismError>)` with ALL collected errors.
    ///
    /// # Invariants
    /// - INV-SCALAR-002: validation BEFORE merge — this function is always called
    ///   before `merge_overlay_onto_type_spec`.
    /// - INV-SCALAR-003: a single invalid overlay fails the entire boot.
    pub fn validate_overlay_toml(
        toml_input: &str,
        overlay_file_path: &str,
        expected_sensor_id: &str,
        expected_org_slug: &str,
        type_specs: &HashMap<String, SensorSpec>,
    ) -> Result<SensorInstanceOverlay, Vec<PrismError>> {
        todo!()
    }

    /// Merge overlay scalar fields onto a TYPE spec to produce a `ResolvedSensorSpec`.
    ///
    /// Merges only the permitted tunable scalars:
    /// - `base_url` — if `overlay.base_url` is `Some`, replaces TYPE spec value.
    /// - `rate_limit_hints.requests_per_second` — if `Some`, replaces TYPE spec value.
    /// - `rate_limit_hints.burst_size` — if `Some`, replaces TYPE spec value.
    /// - `timeout_secs` — if `Some`, stored in provenance metadata.
    ///
    /// `tables`, `auth_type`, `version`, `sensor_id`, `name`, `credential_refs`
    /// are NEVER overridden (INV-OVL-001, INV-OVL-002).
    ///
    /// Provenance is tracked per field (BC-2.06.012 postcondition — merged spec
    /// carries provenance metadata).
    ///
    /// # BC-2.06.012 invariants
    /// - INV-OVL-001: `tables` schema is immutable per overlay.
    /// - INV-OVL-002: `auth_type` is immutable per overlay.
    /// - INV-OVL-006: returned `ResolvedSensorSpec` is not further mutated.
    pub fn merge_overlay_onto_type_spec(
        type_spec: &SensorSpec,
        overlay: &SensorInstanceOverlay,
        org_slug: OrgSlug,
    ) -> ResolvedSensorSpec {
        todo!()
    }

    /// Build the canonical E-SPEC-022 error for an unknown org slug directory.
    ///
    /// Message template (BC-2.06.016 canonical form, INV-ERR-002):
    /// "Per-org overlay directory 'customers/{slug}/' references org slug '{slug}' which
    /// is not registered in OrgRegistry. Check for typos or register the org in
    /// prism.toml [[orgs]]."
    ///
    /// # GREEN-BY-DESIGN self-check (BC-5.38.005 invariant 1)
    /// "If I include this real implementation, will the test for this function pass
    /// trivially without any implementer work?" — YES for the string template; however
    /// the message template is load-bearing for INV-ERR-002 canonical match. We keep
    /// this as todo!() to preserve the Red Gate.
    pub fn e_spec_022_unknown_org_slug(
        customers_dir_name: &str,
        slug: &str,
        overlay_file: &str,
    ) -> PrismError {
        todo!()
    }
}

// ---------------------------------------------------------------------------
// Helper: detect [[tables]] in raw TOML string
// ---------------------------------------------------------------------------

/// Returns `true` if the TOML string contains any `[[tables]]` array-of-table
/// declarations (a proxy for BC-2.06.013 INV-SCALAR-004 detection).
///
/// Used by `validate_overlay_toml` before full deserialization so the rejection
/// error (E-SPEC-021) can be emitted before serde parses the value.
///
/// This is a heuristic line-scan — it matches `[[tables]]` as a standalone
/// TOML header line.  A deserialization-level check is the authoritative gate;
/// this function provides an early-exit path for the common case.
///
/// # BC-5.38.005 invariant 1 — Self-Check
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — Yes; this is a string scan helper.
/// Kept as `todo!()` per BC-5.38.001 because the implementer must decide whether
/// to use a heuristic line-scan or a post-deserialization structural check.
#[allow(unused_variables)]
pub fn raw_toml_contains_tables_header(toml_str: &str) -> bool {
    todo!()
}

// ---------------------------------------------------------------------------
// Public error constructor helpers — canonical template builders
// ---------------------------------------------------------------------------

/// Build the E-SPEC-021 error for a `[[tables]]` block in an overlay file.
///
/// Canonical message template (BC-2.06.016 §Error Catalog):
/// "Per-org overlay '{file}' for instance '{instance_id}' contains [[tables]] blocks.
/// Schema overrides are forbidden in overlay files (ADR-029)."
#[allow(unused_variables)]
pub fn make_e_spec_021_tables_in_overlay(file_path: &str, instance_id: &str) -> PrismError {
    todo!()
}

/// Build the E-SPEC-023 error for an unrecognized field in an overlay file.
///
/// Canonical message template (BC-2.06.016 §Error Catalog):
/// "Per-org overlay '{file}' contains unrecognized field '{field}'. Only scalar
/// tunables are permitted in overlay files (ADR-029)."
#[allow(unused_variables)]
pub fn make_e_spec_023_unrecognized_field(
    file_path: &str,
    instance_id: &str,
    field_name: &str,
) -> PrismError {
    todo!()
}

/// Build the E-SPEC-020 error for an `instance_id` mismatch in an overlay file.
///
/// Canonical message template (BC-2.06.016 §Error Catalog):
/// "Per-org overlay '{file}' instance_id '{actual}' does not match expected '{expected}'
/// ({sensor_id}@{org_slug}). Rename or correct the instance_id field."
#[allow(unused_variables)]
pub fn make_e_spec_020_instance_id_mismatch(
    file_path: &str,
    actual_instance_id: &str,
    expected_instance_id: &str,
) -> PrismError {
    todo!()
}

/// Build the E-SPEC-019 error for an unknown `extends` value in an overlay file.
///
/// Canonical message template (BC-2.06.016 §Error Catalog):
/// "Per-org overlay '{file}' extends '{extends_value}' which is not a loaded
/// sensor TYPE spec. Add '{extends_value}.sensor.toml' to the spec directory or
/// correct the extends field."
#[allow(unused_variables)]
pub fn make_e_spec_019_unknown_extends(file_path: &str, extends_value: &str) -> PrismError {
    todo!()
}

// ---------------------------------------------------------------------------
// Tests — placeholder only; real Red Gate tests authored by test-writer
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    /// Placeholder — real Red Gate tests for overlay loading are in
    /// `prism-spec-engine/tests/overlay_loading_tests.rs` (S-CONFIG-MULTI-TENANT-OVERRIDE-001).
    /// This placeholder ensures the module is reachable from the test binary.
    #[test]
    fn placeholder_overlay_module_compiles() {
        // This test intentionally does NOT call any stubbed functions —
        // those are in overlay_loading_tests.rs (the real Red Gate tests).
        // This placeholder passes by design (compilation check only).
    }
}
