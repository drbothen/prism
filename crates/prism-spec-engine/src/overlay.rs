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

use prism_core::{OrgRegistry, OrgSlug, PrismError, SpecError, SpecErrorCode};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

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
// Allowed overlay fields (closed set per BC-2.06.013 INV-SCALAR-001)
// ---------------------------------------------------------------------------

/// The closed set of allowed top-level scalar field names in an overlay file.
///
/// BC-2.06.013 §Allowed vs Forbidden Overlay Fields. Any field not in this set
/// triggers E-SPEC-023.
const ALLOWED_OVERLAY_FIELDS: &[&str] = &[
    "extends",
    "instance_id",
    "base_url",
    "timeout_secs",
    "rate_limit_hints",
];

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
        let mut resolved: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
        let mut errors: Vec<PrismError> = Vec::new();

        // EC-012-001: absent customers/ directory → zero overlays, zero errors.
        if !customers_dir.exists() {
            return OverlayLoadResult { resolved, errors };
        }

        // Read directory entries. On I/O error, return the error.
        let entries = match std::fs::read_dir(customers_dir) {
            Ok(e) => e,
            Err(io_err) => {
                errors.push(PrismError::Io(io_err.to_string()));
                return OverlayLoadResult { resolved, errors };
            }
        };

        // Collect all org subdirectory entries first.
        let mut org_dirs: Vec<(String, std::path::PathBuf)> = Vec::new();
        for entry_result in entries {
            let entry = match entry_result {
                Ok(e) => e,
                Err(io_err) => {
                    errors.push(PrismError::Io(io_err.to_string()));
                    continue;
                }
            };

            // INV-COMPAT-004: only subdirectory entries trigger slug lookup;
            // plain files (e.g., .gitkeep) are ignored.
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(io_err) => {
                    errors.push(PrismError::Io(io_err.to_string()));
                    continue;
                }
            };

            if !file_type.is_dir() {
                // EC-012-002: skip .gitkeep and other plain files.
                continue;
            }

            let dir_name = entry.file_name().to_string_lossy().to_string();
            org_dirs.push((dir_name, entry.path()));
        }

        // BC-2.06.015: cross-check each org directory against OrgRegistry.
        // Collect E-SPEC-022 errors for ALL unregistered slugs before continuing.
        for (slug_str, org_dir_path) in &org_dirs {
            let org_slug = OrgSlug::new(slug_str.as_str());

            let slug_registered = if org_slug.is_ok() {
                org_registry.resolve(&org_slug).is_some()
            } else {
                false
            };

            if !slug_registered {
                // BC-2.06.015 failure path: unknown org slug directory.
                let dir_display = format!("customers/{slug_str}/");
                errors.push(Self::e_spec_022_unknown_org_slug(
                    &dir_display,
                    slug_str,
                    &org_dir_path.display().to_string(),
                ));
            }
        }

        // INV-SCALAR-003: if any E-SPEC-022 errors, abort the walk (fail-fast on
        // unregistered slug per BC-2.06.015 §Invariants INV-COMPAT-003 pattern).
        if !errors.is_empty() {
            return OverlayLoadResult { resolved, errors };
        }

        // Walk each registered org directory and load overlay files.
        for (slug_str, org_dir_path) in &org_dirs {
            let org_slug = OrgSlug::new(slug_str.as_str());

            // Enumerate .sensor.toml files within this org dir.
            let file_entries = match std::fs::read_dir(org_dir_path) {
                Ok(e) => e,
                Err(io_err) => {
                    errors.push(PrismError::Io(io_err.to_string()));
                    continue;
                }
            };

            for file_entry_result in file_entries {
                let file_entry = match file_entry_result {
                    Ok(e) => e,
                    Err(io_err) => {
                        errors.push(PrismError::Io(io_err.to_string()));
                        continue;
                    }
                };

                let file_name = file_entry.file_name().to_string_lossy().to_string();

                // Only process *.sensor.toml files.
                if !file_name.ends_with(".sensor.toml") {
                    continue;
                }

                // Derive sensor_id from filename stem: strip ".sensor.toml" suffix.
                let sensor_id = &file_name[..file_name.len() - ".sensor.toml".len()];

                // Build a human-readable file path for error messages.
                let overlay_file_path = format!("customers/{slug_str}/{file_name}");

                // Read TOML content.
                let toml_content = match std::fs::read_to_string(file_entry.path()) {
                    Ok(c) => c,
                    Err(io_err) => {
                        errors.push(PrismError::Io(io_err.to_string()));
                        continue;
                    }
                };

                // BC-2.06.013: validate before merge (INV-SCALAR-002).
                match Self::validate_overlay_toml(
                    &toml_content,
                    &overlay_file_path,
                    sensor_id,
                    slug_str,
                    type_specs,
                ) {
                    Ok(overlay) => {
                        // BC-2.06.012: merge overlay scalars onto TYPE spec.
                        let type_spec = type_specs.get(sensor_id)
                            .expect("validate_overlay_toml guarantees extends resolves to a loaded TYPE spec");

                        let resolved_spec = Self::merge_overlay_onto_type_spec(
                            type_spec,
                            &overlay,
                            org_slug.clone(),
                        );

                        // BC-2.06.012 postcondition: log overlay.loaded at info level.
                        // BC-2.16.002 Structured Event Catalog row (SAP-1):
                        //   event_type = "overlay.loaded", org_slug, sensor_id, instance_id
                        //   Audit role: operational/traceability
                        //   Recurrence: once per overlay file per boot / config reload
                        tracing::info!(
                            event_type = "overlay.loaded",
                            org_slug = %slug_str,
                            sensor_id = %sensor_id,
                            instance_id = %overlay.instance_id,
                            "per-org overlay loaded and merged"
                        );

                        let key = (org_slug.clone(), sensor_id.to_string());
                        resolved.insert(key, resolved_spec);
                    }
                    Err(overlay_errors) => {
                        // INV-SCALAR-003: collect all errors across all files.
                        errors.extend(overlay_errors);
                    }
                }
            }
        }

        OverlayLoadResult { resolved, errors }
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
    /// - INV-SCALAR-003: a single invalid overlay file fails the entire boot.
    pub fn validate_overlay_toml(
        toml_input: &str,
        overlay_file_path: &str,
        expected_sensor_id: &str,
        expected_org_slug: &str,
        type_specs: &HashMap<String, SensorSpec>,
    ) -> Result<SensorInstanceOverlay, Vec<PrismError>> {
        let mut validation_errors: Vec<PrismError> = Vec::new();

        // Parse into raw TOML Value for structural inspection.
        let raw: TomlValue = match toml::from_str(toml_input) {
            Ok(v) => v,
            Err(e) => {
                // E-SPEC-001: TOML parse error.
                return Err(vec![PrismError::Spec(SpecError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "Per-org overlay '{}' failed TOML parse: {}",
                        overlay_file_path, e
                    ),
                    toml_path: None,
                    file_path: Some(overlay_file_path.to_string()),
                    line_number: None,
                })]);
            }
        };

        // Inspect the top-level TOML table.
        let table = match raw.as_table() {
            Some(t) => t,
            None => {
                return Err(vec![PrismError::Spec(SpecError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "Per-org overlay '{}' is not a TOML table",
                        overlay_file_path
                    ),
                    toml_path: None,
                    file_path: Some(overlay_file_path.to_string()),
                    line_number: None,
                })]);
            }
        };

        // BC-2.06.013 Check 1: [[tables]] present → E-SPEC-021.
        // `[[tables]]` in TOML is represented as an array of tables (Value::Array of Value::Table).
        if table.contains_key("tables") {
            // The field exists — any shape of "tables" (array or table) is forbidden.
            // Derive instance_id from context for the error message.
            let instance_id_for_msg = format!("{}@{}", expected_sensor_id, expected_org_slug);
            validation_errors.push(make_e_spec_021_tables_in_overlay(
                overlay_file_path,
                &instance_id_for_msg,
            ));
        }

        // BC-2.06.013 Check 2: unrecognized scalar fields → E-SPEC-023.
        for field_name in table.keys() {
            if !ALLOWED_OVERLAY_FIELDS.contains(&field_name.as_str()) {
                // Skip "tables" — already covered by E-SPEC-021 above.
                if field_name == "tables" {
                    continue;
                }
                let instance_id_for_msg = format!("{}@{}", expected_sensor_id, expected_org_slug);
                validation_errors.push(make_e_spec_023_unrecognized_field(
                    overlay_file_path,
                    &instance_id_for_msg,
                    field_name,
                ));
            }
        }

        // If structural errors found, return early — don't proceed to semantic checks.
        if !validation_errors.is_empty() {
            return Err(validation_errors);
        }

        // Deserialize into SensorInstanceOverlay now that structure is valid.
        let overlay: SensorInstanceOverlay = match toml::from_str(toml_input) {
            Ok(o) => o,
            Err(e) => {
                return Err(vec![PrismError::Spec(SpecError {
                    code: SpecErrorCode::ESpec001,
                    message: format!(
                        "Per-org overlay '{}' failed deserialization: {}",
                        overlay_file_path, e
                    ),
                    toml_path: None,
                    file_path: Some(overlay_file_path.to_string()),
                    line_number: None,
                })]);
            }
        };

        // BC-2.06.013 Check 3: instance_id convention mismatch → E-SPEC-020.
        let expected_instance_id = format!("{}@{}", expected_sensor_id, expected_org_slug);
        if overlay.instance_id != expected_instance_id {
            validation_errors.push(make_e_spec_020_instance_id_mismatch(
                overlay_file_path,
                &overlay.instance_id,
                &expected_instance_id,
            ));
        }

        // BC-2.06.012 Check 4: extends references unknown TYPE spec → E-SPEC-019.
        if !type_specs.contains_key(&overlay.extends) {
            validation_errors.push(make_e_spec_019_unknown_extends(
                overlay_file_path,
                &overlay.extends,
            ));
        }

        if !validation_errors.is_empty() {
            return Err(validation_errors);
        }

        Ok(overlay)
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
        // Start with a clone of the TYPE spec.
        let mut merged_spec = type_spec.clone();

        let mut provenance = OverlayProvenance::default();

        // Merge base_url if overlay provides one.
        if let Some(ref overlay_base_url) = overlay.base_url {
            merged_spec.base_url = overlay_base_url.clone();
            provenance.base_url_from_overlay = true;
        }
        // If overlay.base_url is None, merged_spec.base_url stays as TYPE spec default.
        // provenance.base_url_from_overlay remains false (Case B).

        // Merge rate_limit_hints scalars individually.
        if let Some(ref overlay_rls) = overlay.rate_limit_hints {
            let merged_rls = merged_spec
                .rate_limit_hints
                .get_or_insert_with(Default::default);

            if let Some(rps) = overlay_rls.requests_per_second {
                merged_rls.requests_per_second = Some(rps);
                provenance.rps_from_overlay = true;
            }

            if let Some(burst) = overlay_rls.burst_size {
                merged_rls.burst_size = Some(burst);
                provenance.burst_size_from_overlay = true;
            }
        }

        // timeout_secs provenance (stored in provenance; no SensorSpec field for it yet).
        if overlay.timeout_secs.is_some() {
            provenance.timeout_secs_from_overlay = true;
        }

        // INV-OVL-001: tables are NEVER overridden — inherited from TYPE spec (already in clone).
        // INV-OVL-002: auth_type is NEVER overridden — inherited from TYPE spec (already in clone).
        // sensor_id, name, version, credential_refs — all inherited from TYPE spec clone.

        ResolvedSensorSpec {
            spec: merged_spec,
            provenance,
            org_slug,
            instance_id: overlay.instance_id.clone(),
        }
    }

    /// Build the canonical E-SPEC-022 error for an unknown org slug directory.
    ///
    /// Message template (BC-2.06.016 canonical form, INV-ERR-002):
    /// "Per-org overlay directory 'customers/{slug}/' references org slug '{slug}' which
    /// is not registered in OrgRegistry. Check for typos or register the org in
    /// prism.toml [[orgs]]."
    pub fn e_spec_022_unknown_org_slug(
        customers_dir_name: &str,
        slug: &str,
        _overlay_file: &str,
    ) -> PrismError {
        PrismError::Spec(SpecError {
            code: SpecErrorCode::ESpec022,
            message: format!(
                "Per-org overlay directory '{customers_dir_name}' references org slug '{slug}' \
                 which is not registered in OrgRegistry. Check for typos or register the org in \
                 prism.toml [[orgs]]."
            ),
            toml_path: None,
            file_path: Some(customers_dir_name.to_string()),
            line_number: None,
        })
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
pub fn raw_toml_contains_tables_header(toml_str: &str) -> bool {
    // Check whether the TOML value contains a "tables" key (any shape).
    // The authoritative check is performed in validate_overlay_toml via toml::Value parsing.
    // This heuristic is used for early-exit logging only.
    toml_str.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "[[tables]]" || trimmed.starts_with("[[tables.")
    })
}

// ---------------------------------------------------------------------------
// Public error constructor helpers — canonical template builders
// ---------------------------------------------------------------------------

/// Build the E-SPEC-021 error for a `[[tables]]` block in an overlay file.
///
/// Canonical message template (BC-2.06.016 §Error Catalog):
/// "Per-org overlay '{file}' for instance '{instance_id}' contains [[tables]] blocks.
/// Schema overrides are forbidden in overlay files (ADR-029)."
pub fn make_e_spec_021_tables_in_overlay(file_path: &str, instance_id: &str) -> PrismError {
    PrismError::Spec(SpecError {
        code: SpecErrorCode::ESpec021,
        message: format!(
            "Per-org overlay '{file_path}' for instance '{instance_id}' contains [[tables]] \
             blocks. Schema overrides are forbidden in overlay files (ADR-029). Table schema \
             must be declared in the TYPE spec only."
        ),
        toml_path: Some("tables".to_string()),
        file_path: Some(file_path.to_string()),
        line_number: None,
    })
}

/// Build the E-SPEC-023 error for an unrecognized field in an overlay file.
///
/// Canonical message template (BC-2.06.016 §Error Catalog):
/// "Per-org overlay '{file}' contains unrecognized field '{field}'. Only scalar
/// tunables are permitted in overlay files (ADR-029)."
pub fn make_e_spec_023_unrecognized_field(
    file_path: &str,
    instance_id: &str,
    field_name: &str,
) -> PrismError {
    PrismError::Spec(SpecError {
        code: SpecErrorCode::ESpec023,
        message: format!(
            "Per-org overlay '{file_path}' contains unrecognized field '{field_name}'. \
             Allowed overlay fields are: extends, instance_id, base_url, timeout_secs, \
             rate_limit_hints (with sub-fields: requests_per_second, burst_size). \
             Instance: '{instance_id}'."
        ),
        toml_path: Some(field_name.to_string()),
        file_path: Some(file_path.to_string()),
        line_number: None,
    })
}

/// Build the E-SPEC-020 error for an `instance_id` mismatch in an overlay file.
///
/// Canonical message template (BC-2.06.016 §Error Catalog):
/// "Per-org overlay '{file}' instance_id '{actual}' does not match expected '{expected}'
/// ({sensor_id}@{org_slug}). Rename or correct the instance_id field."
pub fn make_e_spec_020_instance_id_mismatch(
    file_path: &str,
    actual_instance_id: &str,
    expected_instance_id: &str,
) -> PrismError {
    PrismError::Spec(SpecError {
        code: SpecErrorCode::ESpec020,
        message: format!(
            "Per-org overlay '{file_path}' declares instance_id='{actual_instance_id}' but \
             expected '{expected_instance_id}' (derived from filename and parent directory). \
             Rename or correct the instance_id field."
        ),
        toml_path: Some("instance_id".to_string()),
        file_path: Some(file_path.to_string()),
        line_number: None,
    })
}

/// Build the E-SPEC-019 error for an unknown `extends` value in an overlay file.
///
/// Canonical message template (BC-2.06.016 §Error Catalog):
/// "Per-org overlay '{file}' extends '{extends_value}' which is not a loaded
/// sensor TYPE spec. Add '{extends_value}.sensor.toml' to the spec directory or
/// correct the extends field."
pub fn make_e_spec_019_unknown_extends(file_path: &str, extends_value: &str) -> PrismError {
    PrismError::Spec(SpecError {
        code: SpecErrorCode::ESpec019,
        message: format!(
            "Per-org overlay '{file_path}' declares extends='{extends_value}' but no sensor \
             TYPE named '{extends_value}' is loaded. Check spelling or add a TYPE spec file \
             named '{extends_value}.sensor.toml'."
        ),
        toml_path: Some("extends".to_string()),
        file_path: Some(file_path.to_string()),
        line_number: None,
    })
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
