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

use prism_core::{OrgRegistry, OrgSlug, PrismError, SensorId, SpecError, SpecErrorCode};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

use crate::env_resolver::resolve_env_tokens_in_string_field;
use crate::error::SpecEngineError;
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
/// directly; use the TOML deserialization path via `OverlayLoader::load_overlays`.
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
/// Produced at boot by `OverlayLoader::load_overlays` for each
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
/// via `Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>` with no mutex on
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
/// `SensorId` is from `prism-core` (already a direct dependency of this crate);
/// using the newtype avoids the raw-String footgun and aligns with the ADR-024
/// canonical sensor ID type (ADV-010 fix).
pub type ResolvedSpecKey = (OrgSlug, SensorId);

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

/// Maximum permitted overlay file size in bytes (SEC-REDUX-005, CWE-400).
///
/// Overlay files are scalar-only tunables (BC-2.06.013 INV-SCALAR-001).
/// A typical overlay is ~100 bytes; 64 KiB is a generous upper bound that
/// prevents boot-time DoS from maliciously large files while accommodating
/// any realistic overlay content.
const MAX_OVERLAY_FILE_BYTES: u64 = 64 * 1024; // 64 KiB

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
        // PRR-009 fix: compute OrgSlug once per entry and store it alongside
        // the registration flag — eliminates the duplicate OrgSlug::new() in the
        // second pass (the regex runs only once per org directory).
        //
        // `org_entries` stores (slug_str, path, org_slug, is_registered) tuples.
        // Collect E-SPEC-022 errors for ALL unregistered slugs before continuing.
        struct OrgDirEntry {
            slug_str: String,
            path: std::path::PathBuf,
            /// The parsed OrgSlug — always present; carry validity state in the OrgSlug newtype.
            org_slug: OrgSlug,
            is_registered: bool,
        }

        let mut org_entries: Vec<OrgDirEntry> = Vec::with_capacity(org_dirs.len());
        for (slug_str, path) in org_dirs {
            // PRR-009: parse OrgSlug once; store in OrgDirEntry for reuse in second pass.
            let org_slug = OrgSlug::new(slug_str.as_str());

            // PRR-012 fix: use OrgRegistry::slug_exists (spec AC-004 method name alignment).
            let is_registered = if org_slug.is_ok() {
                org_registry.slug_exists(&org_slug)
            } else {
                false
            };

            if !is_registered {
                // BC-2.06.015 failure path: unknown org slug directory.
                // PRR-006 fix: use free fn make_e_spec_022_unknown_org_slug (consistent
                // with sibling make_e_spec_019..021..023 naming pattern).
                // SEC-PASS6-001: sanitize slug_str at derivation — readdir-sourced and may contain
                // control characters.  `customers_dir_name` is embedded verbatim in the E-SPEC-022
                // error message body (make_e_spec_022_unknown_org_slug line ~821) and file_path
                // field; sanitize_for_display here mirrors the SEC-PASS5-002 pattern at line 405-406
                // (overlay_file_path) to complete the TD-VSDD-060 sibling-sweep (CWE-117).
                let dir_display = format!("customers/{}/", sanitize_for_display(&slug_str));
                errors.push(make_e_spec_022_unknown_org_slug(&dir_display, &slug_str));
            }

            org_entries.push(OrgDirEntry {
                slug_str,
                path,
                org_slug,
                is_registered,
            });
        }

        // EC-016-002: continue scanning ALL directories (registered AND unregistered)
        // to collect any E-SPEC-021/023 errors within them.  BC-2.06.016 requires
        // BOTH directory-level errors (E-SPEC-022) AND file-level errors (E-SPEC-021/023)
        // to be collected before returning — removing the early-return guard here.
        //
        // Walk each org directory and load overlay files.
        for OrgDirEntry {
            slug_str,
            path: org_dir_path,
            org_slug,
            is_registered,
        } in &org_entries
        {
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

                // SEC-REDUX-002: reject symlinks at file level (CWE-59).
                // DirEntry::file_type() uses lstat() on POSIX — is_file() returns false
                // for symlinks, blocking path traversal and file-disclosure vectors.
                let file_ft = match file_entry.file_type() {
                    Ok(ft) => ft,
                    Err(io_err) => {
                        errors.push(PrismError::Io(io_err.to_string()));
                        continue;
                    }
                };
                if !file_ft.is_file() {
                    // Skip symlinks, directories, and special files within org dirs.
                    continue;
                }

                let file_name = file_entry.file_name().to_string_lossy().to_string();

                // Only process *.sensor.toml files.
                if !file_name.ends_with(".sensor.toml") {
                    continue;
                }

                // Derive sensor_id from filename stem: strip ".sensor.toml" suffix.
                let sensor_id = &file_name[..file_name.len() - ".sensor.toml".len()];

                // Build a human-readable file path for error messages.
                // SEC-PASS5-002: sanitize at derivation point — `file_name` comes from readdir
                // and can contain control characters on Linux/macOS (CWE-117, SEC-REDUX-004).
                // Sanitizing here covers all 9 downstream error constructors that embed
                // `overlay_file_path`: E-SPEC-001 size-check, TOML parse, table-type, deser,
                // SSRF rejection, and E-SPEC-019/020/021/023 validation errors.
                let overlay_file_path =
                    sanitize_for_display(&format!("customers/{slug_str}/{file_name}"));

                // SEC-REDUX-005: enforce overlay file size limit (CWE-400).
                // Pre-check size via metadata() before reading to prevent boot-time DoS.
                // Overlay files are scalar-only tunables (BC-2.06.013); 64 KiB is generous.
                match file_entry.metadata() {
                    Ok(meta) if meta.len() > MAX_OVERLAY_FILE_BYTES => {
                        errors.push(PrismError::Spec(SpecError {
                            code: SpecErrorCode::ESpec001,
                            message: format!(
                                "Per-org overlay '{overlay_file_path}' exceeds maximum allowed \
                                 size ({} bytes > {MAX_OVERLAY_FILE_BYTES} bytes limit). \
                                 Overlay files must be scalar-only tunables.",
                                meta.len()
                            ),
                            toml_path: None,
                            file_path: Some(overlay_file_path.clone()),
                            line_number: None,
                        }));
                        continue;
                    }
                    Err(io_err) => {
                        errors.push(PrismError::Io(io_err.to_string()));
                        continue;
                    }
                    Ok(_) => {} // size OK, proceed
                }

                // Read TOML content.
                let toml_content = match std::fs::read_to_string(file_entry.path()) {
                    Ok(c) => c,
                    Err(io_err) => {
                        errors.push(PrismError::Io(format!(
                            "Failed to read overlay file '{}': {}",
                            overlay_file_path, io_err
                        )));
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
                        // Only insert into resolved map when the slug is registered
                        // (EC-016-002: unregistered directories are scanned for file-level
                        // errors but their overlays are NOT merged into resolved).
                        if !is_registered {
                            // E-SPEC-022 was already emitted for this slug above.
                            // Do not insert into resolved; continue to collect file-level errors.
                            continue;
                        }

                        // validate_overlay_toml (check 4 — E-SPEC-019) guarantees that
                        // overlay.extends names a loaded TYPE spec when Ok(...) is returned.
                        // The defensive arm below is unreachable in correct flow but guards
                        // against future refactors that relax the validate_overlay_toml
                        // invariant (OBS-001 / CLAUDE.md §Forbidden patterns: no .expect()).
                        let type_spec = match type_specs.get(sensor_id) {
                            Some(ts) => ts,
                            None => {
                                // This arm is logically unreachable: validate_overlay_toml
                                // returns Err(E-SPEC-019) when extends is unresolvable.
                                // Emit an error and skip this overlay to avoid a silent gap.
                                errors.push(PrismError::Internal {
                                    detail: format!(
                                        "internal: overlay '{overlay_file_path}' passed validation \
                                         but TYPE spec '{sensor_id}' not found in type_specs map; \
                                         this is a bug — E-SPEC-019 check should have caught this"
                                    ),
                                });
                                continue;
                            }
                        };

                        // PRR-009: reuse the OrgSlug parsed in the first pass (no second regex run).
                        // is_registered=true guarantees org_slug.is_ok() by construction.
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

                        let key = (org_slug.clone(), SensorId::from(sensor_id));
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
            // SEC-PASS4-002 / TD-VSDD-060: sanitize expected_sensor_id before embedding in error
            // message — it originates from the filesystem path (attacker-controlled) and must be
            // sanitized for log injection prevention (CWE-117, SEC-REDUX-004).
            // expected_org_slug is also filesystem-sourced; sanitized via make_e_spec_022 elsewhere
            // but sanitized here too for defense-in-depth at the concatenation site.
            let instance_id_for_msg = format!(
                "{}@{}",
                sanitize_for_display(expected_sensor_id),
                sanitize_for_display(expected_org_slug)
            );
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
                validation_errors.push(make_e_spec_023_unrecognized_field(
                    overlay_file_path,
                    field_name,
                ));
            }
        }

        // If structural errors found, return early — don't proceed to semantic checks.
        if !validation_errors.is_empty() {
            return Err(validation_errors);
        }

        // Deserialize into SensorInstanceOverlay from the already-parsed raw TomlValue.
        // ADV-011: avoid double-parse by reusing `raw` instead of calling toml::from_str again.
        // `mut` is required for the env resolver pass (EC-009-007) which mutates overlay.base_url.
        let mut overlay: SensorInstanceOverlay = match raw.clone().try_into() {
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

        // BC-2.16.009 §VR6 EC-009-007: resolve ${env.VAR_NAME} tokens in overlay base_url
        // BEFORE the SSRF scheme check (SEC-REDUX-006). Without this pass, a raw token
        // like "${env.ARMIS_URL}" would either (a) fail starts_with("https://") → wrong
        // E-SPEC-001 error, or (b) for partial tokens that start with "https://", survive
        // into the merged spec's base_url and be routed as a live HTTP URL with the
        // unresolved token embedded (CWE-918 / garbage-URL vector).
        //
        // The resolver runs in-place on overlay.base_url; SpecEngineError::EnvVarNotSet
        // errors are converted to PrismError::Spec(ESpec024) for consistency with the
        // overlay error type.
        //
        // AD-017: the conversion copies only var_name + toml_path — no resolved VALUE.
        //
        // `base_url_env_resolved` tracks whether the env resolution pass ran without errors.
        // When false (env errors collected), the SSRF scheme check is SKIPPED for base_url:
        // (a) the resolver has already reported E-SPEC-024; an additional E-SPEC-001 on the
        //     unresolved raw token would be a misleading second error on the same field.
        // (b) the raw token is guaranteed to be rejected anyway (spec fails on E-SPEC-024).
        let mut base_url_env_resolved = true;
        if let Some(ref mut base_url_field) = overlay.base_url {
            let env_errors =
                resolve_env_tokens_in_string_field(base_url_field, "base_url", overlay_file_path);
            if !env_errors.is_empty() {
                base_url_env_resolved = false;
                for env_err in env_errors {
                    // Convert SpecEngineError::EnvVarNotSet → PrismError::Spec(ESpec024).
                    // Only EnvVarNotSet is emitted by the resolver (per env_resolver.rs contract).
                    // Use a match to ensure we don't silently swallow future resolver error variants.
                    match &env_err {
                        SpecEngineError::EnvVarNotSet {
                            toml_path,
                            file_path: err_file_path,
                            ..
                        } => {
                            validation_errors.push(PrismError::Spec(SpecError {
                                code: SpecErrorCode::ESpec024,
                                // Route through the pinned Display on SpecEngineError::EnvVarNotSet
                                // (error.rs #[error(...)]) rather than a duplicate format!() literal.
                                // This makes error.rs the single source of truth for the E-SPEC-024
                                // message — test_E_SPEC_024_display_matches_error_taxonomy_template_
                                // byte_for_byte in error.rs pins that Display byte-for-byte, so any
                                // taxonomy change is caught by that single test and propagates here
                                // automatically (F-P2-MED-001 / POL-24 / POL-25).
                                //
                                // AD-017: the Display emits var NAME and field path only — never value.
                                message: env_err.to_string(),
                                toml_path: Some(toml_path.clone()),
                                file_path: Some(err_file_path.clone()),
                                line_number: None,
                            }));
                        }
                        // This arm is unreachable: resolve_env_tokens_in_string_field only emits
                        // EnvVarNotSet. If a future refactor adds a new variant, this arm will
                        // catch it and surface a structured error rather than silently swallowing.
                        other => {
                            validation_errors.push(PrismError::Internal {
                                detail: format!(
                                    "unexpected error from env resolver on overlay base_url: {other}"
                                ),
                            });
                        }
                    }
                }
            }
        }

        // SEC-REDUX-006: validate overlay base_url scheme (CWE-918 SSRF prevention).
        // The TYPE spec base_url is validated by validation.rs; the overlay must match.
        // Allows http:// and https:// only — rejects file://, ftp://, and other schemes.
        // At this point, overlay.base_url has been through the env resolver above (EC-009-007),
        // so the scheme check sees the RESOLVED URL, not any raw ${env.VAR} token.
        //
        // Guard: skip this check when env resolution for base_url produced errors — the
        // resolver already reported E-SPEC-024 and the spec will be rejected; adding a
        // second E-SPEC-001 on the raw unresolved token would be misleading noise.
        if base_url_env_resolved
            && let Some(ref overlay_base_url) = overlay.base_url
            && !overlay_base_url.starts_with("https://")
            && !overlay_base_url.starts_with("http://")
        {
            validation_errors.push(PrismError::Spec(SpecError {
                code: SpecErrorCode::ESpec001,
                message: format!(
                    "Per-org overlay '{}' base_url '{}' is not a valid URL \
                     (must start with http:// or https://). Non-HTTP schemes are \
                     rejected to prevent SSRF attacks (CWE-918).",
                    overlay_file_path,
                    sanitize_for_display(overlay_base_url)
                ),
                toml_path: Some("base_url".to_string()),
                file_path: Some(overlay_file_path.to_string()),
                line_number: None,
            }));
        }

        // BC-2.06.013 Check 3: instance_id convention mismatch → E-SPEC-020.
        // SEC-PASS5-001: sanitize both components before concatenating — expected_sensor_id is a
        // raw filesystem stem (no regex validation) and expected_org_slug is unsafe in the
        // EC-016-002 path (CWE-117, SEC-REDUX-004). Mirrors the E-SPEC-021 sanitization at the
        // [[tables]] check above (SEC-PASS4-002 / TD-VSDD-060).
        let expected_instance_id = format!(
            "{}@{}",
            sanitize_for_display(expected_sensor_id),
            sanitize_for_display(expected_org_slug)
        );
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
        if let Some(timeout_secs) = overlay.timeout_secs {
            provenance.timeout_secs_from_overlay = true;
            tracing::warn!(
                event_type = "overlay.timeout_secs_ignored",
                sensor_id = %type_spec.sensor_id,
                timeout_secs = timeout_secs,
                "timeout_secs overlay field accepted but not yet wired to HTTP client; \
                 deferred to S-CONFIG-MULTI-TENANT-OVERRIDE-002"
            );
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
}

// ---------------------------------------------------------------------------
// Display-injection sanitizer (SEC-REDUX-004, CWE-117)
// ---------------------------------------------------------------------------

/// Sanitize a user-controlled value before embedding it in a display-facing error message.
///
/// **Contract B — display sanitizer:** replaces control characters (including `\n`, `\r`,
/// `\t`, null bytes, and all Unicode control points) with U+FFFD (replacement character) and
/// caps the output at 256 Unicode scalar values. This prevents log injection when error messages
/// are forwarded to SIEM/log aggregators (CWE-117).
///
/// Distinct from `prism_core::error::sanitize_for_log` (Contract A) which strips control chars
/// with no replacement and no length cap — used for structured log field sanitization.
///
/// Called on all TOML-sourced values that land in error message bodies:
/// - `actual_instance_id` in `make_e_spec_020_instance_id_mismatch`
/// - `field_name` in `make_e_spec_023_unrecognized_field`
/// - `slug` in `make_e_spec_022_unknown_org_slug`
/// - `extends_value` in `make_e_spec_019_unknown_extends`
/// - `overlay_base_url` in the SEC-REDUX-006 SSRF rejection branch of `validate_overlay_toml`
/// - `expected_sensor_id` / `expected_org_slug` in the E-SPEC-021 `[[tables]]` error path
///   of `validate_overlay_toml` (SEC-PASS4-002 — final TD-VSDD-060 sibling-sweep site)
/// - `overlay_file_path` derived from `readdir` `file_name` — sanitized at derivation point
///   to cover all 9 error constructors that embed the path (SEC-PASS5-002, CWE-117)
/// - `expected_instance_id` components in the E-SPEC-020 `instance_id` mismatch check
///   (SEC-PASS5-001 — `expected_sensor_id` is an unvalidated filesystem stem;
///   `expected_org_slug` is unsafe in the EC-016-002 registration path)
/// - `dir_display` in the E-SPEC-022 unknown-org-slug error path — `slug_str` is readdir-sourced
///   and sanitized at derivation before constructing the `customers/{slug}/` display string
///   (SEC-PASS6-001 — TD-VSDD-060 sibling-sweep completion, CWE-117)
fn sanitize_for_display(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_control() { '\u{FFFD}' } else { c })
        .take(256)
        .collect()
}

// ---------------------------------------------------------------------------
// Public error constructor helpers — canonical template builders
// ---------------------------------------------------------------------------

/// Build the E-SPEC-022 error for an unregistered org slug directory.
///
/// Canonical message template per `.factory/specs/prd-supplements/error-taxonomy.md`
/// row E-SPEC-022. The `format!` body below produces the exact emission text.
///
/// `slug` is sanitized via `sanitize_for_display` before embedding in the message
/// to prevent log injection (PRR-010 / SEC-REDUX-004, CWE-117): the raw filesystem
/// directory name may contain attacker-controlled data.
pub fn make_e_spec_022_unknown_org_slug(customers_dir_name: &str, slug: &str) -> PrismError {
    let safe_slug = sanitize_for_display(slug);
    PrismError::Spec(SpecError {
        code: SpecErrorCode::ESpec022,
        message: format!(
            "Per-org overlay directory '{customers_dir_name}' references org slug '{safe_slug}' \
             which is not registered in OrgRegistry. Check for typos or register the org in \
             prism.toml [[orgs]]."
        ),
        toml_path: None,
        file_path: Some(customers_dir_name.to_string()),
        line_number: None,
    })
}

/// Build the E-SPEC-021 error for a `[[tables]]` block in an overlay file.
///
/// Canonical message template per `.factory/specs/prd-supplements/error-taxonomy.md`
/// row E-SPEC-021. The `format!` body below produces the exact emission text.
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
/// Canonical message template per `.factory/specs/prd-supplements/error-taxonomy.md`
/// row E-SPEC-023. The `format!` body below produces the exact emission text.
///
/// `field_name` is sanitized via `sanitize_for_display` before embedding in the message
/// to prevent log injection (SEC-REDUX-004, CWE-117).
pub fn make_e_spec_023_unrecognized_field(file_path: &str, field_name: &str) -> PrismError {
    let safe_field = sanitize_for_display(field_name);
    PrismError::Spec(SpecError {
        code: SpecErrorCode::ESpec023,
        message: format!(
            "Per-org overlay '{file_path}' contains unrecognized field '{safe_field}'. \
             Allowed overlay fields are: extends, instance_id, base_url, timeout_secs, \
             rate_limit_hints (with sub-fields: requests_per_second, burst_size)."
        ),
        toml_path: Some(safe_field.to_string()),
        file_path: Some(file_path.to_string()),
        line_number: None,
    })
}

/// Build the E-SPEC-020 error for an `instance_id` mismatch in an overlay file.
///
/// Canonical message template per `.factory/specs/prd-supplements/error-taxonomy.md`
/// row E-SPEC-020. The `format!` body below produces the exact emission text.
///
/// `actual_instance_id` is sanitized via `sanitize_for_display` before embedding in the
/// message to prevent log injection (SEC-REDUX-004, CWE-117).
pub fn make_e_spec_020_instance_id_mismatch(
    file_path: &str,
    actual_instance_id: &str,
    expected_instance_id: &str,
) -> PrismError {
    let safe_actual = sanitize_for_display(actual_instance_id);
    PrismError::Spec(SpecError {
        code: SpecErrorCode::ESpec020,
        message: format!(
            "Per-org overlay '{file_path}' declares instance_id='{safe_actual}' but \
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
/// Canonical message template per `.factory/specs/prd-supplements/error-taxonomy.md`
/// row E-SPEC-019. The `format!` body below produces the exact emission text.
///
/// `extends_value` is sanitized via `sanitize_for_display` before embedding in the message
/// to prevent log injection (SEC-PASS2-002 / SEC-REDUX-004, CWE-117): the TOML-sourced
/// `extends` field may contain attacker-controlled data.  All three occurrences use the
/// sanitized value (TD-VSDD-060 sibling-sweep compliance).
pub fn make_e_spec_019_unknown_extends(file_path: &str, extends_value: &str) -> PrismError {
    let safe_extends = sanitize_for_display(extends_value);
    PrismError::Spec(SpecError {
        code: SpecErrorCode::ESpec019,
        message: format!(
            "Per-org overlay '{file_path}' declares extends='{safe_extends}' but no sensor \
             TYPE named '{safe_extends}' is loaded. Check spelling or add a TYPE spec file \
             named '{safe_extends}.sensor.toml'."
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
    use super::sanitize_for_display;

    /// Placeholder — real Red Gate tests for overlay loading are in
    /// `prism-spec-engine/tests/overlay_loading_tests.rs` (S-CONFIG-MULTI-TENANT-OVERRIDE-001).
    /// This placeholder ensures the module is reachable from the test binary.
    #[test]
    fn placeholder_overlay_module_compiles() {
        // This test intentionally does NOT call any stubbed functions —
        // those are in overlay_loading_tests.rs (the real Red Gate tests).
        // This placeholder passes by design (compilation check only).
    }

    // ---------------------------------------------------------------------------
    // sanitize_for_display unit tests (F-PR155-P2-003)
    // SEC-REDUX-004 / CWE-117: log injection sanitizer correctness
    // ---------------------------------------------------------------------------

    #[test]
    fn sanitize_for_display_replaces_newline_with_replacement_char() {
        let input = "value\nwith\nnewlines";
        let output = sanitize_for_display(input);
        assert!(
            !output.contains('\n'),
            "newlines must be replaced; got: {output:?}"
        );
        assert!(
            output.contains('\u{FFFD}'),
            "replacement char U+FFFD expected; got: {output:?}"
        );
    }

    #[test]
    fn sanitize_for_display_replaces_carriage_return() {
        let input = "value\r\ninjected";
        let output = sanitize_for_display(input);
        assert!(
            !output.contains('\r'),
            "CR must be replaced; got: {output:?}"
        );
        assert!(
            !output.contains('\n'),
            "LF must be replaced; got: {output:?}"
        );
    }

    #[test]
    fn sanitize_for_display_replaces_null_byte() {
        let input = "value\x00null";
        let output = sanitize_for_display(input);
        assert!(
            !output.contains('\x00'),
            "null byte must be replaced; got: {output:?}"
        );
        assert!(
            output.contains('\u{FFFD}'),
            "replacement char U+FFFD expected; got: {output:?}"
        );
    }

    #[test]
    fn sanitize_for_display_truncates_at_256_chars() {
        let input = "a".repeat(300);
        let output = sanitize_for_display(&input);
        let char_count = output.chars().count();
        assert_eq!(
            char_count, 256,
            "output must be capped at 256 chars; got {char_count}"
        );
    }

    #[test]
    fn sanitize_for_display_passes_clean_ascii_unchanged() {
        let input = "clean-ascii-value_123";
        let output = sanitize_for_display(input);
        assert_eq!(
            output, input,
            "clean ASCII must pass through unchanged; got: {output:?}"
        );
    }

    #[test]
    fn sanitize_for_display_preserves_unicode_non_control() {
        // Emoji and CJK are NOT control characters — must not be replaced.
        let input = "hello \u{1F600} \u{4E2D}\u{6587}";
        let output = sanitize_for_display(input);
        assert!(
            output.contains('\u{1F600}'),
            "emoji must be preserved; got: {output:?}"
        );
        assert!(
            output.contains('\u{4E2D}'),
            "CJK char must be preserved; got: {output:?}"
        );
        assert!(
            !output.contains('\u{FFFD}'),
            "replacement char must NOT appear for non-control unicode; got: {output:?}"
        );
    }
}
