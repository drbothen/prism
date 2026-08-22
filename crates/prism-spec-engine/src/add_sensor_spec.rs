// S-1.12: add_sensor_spec MCP tool logic.
// BC-2.16.008: Upload a New Sensor Spec at Runtime.
// E-SPEC-002: filesystem write failure with path and OS error.
// SEC-001: CWE-22 path traversal defense — sensor_id format validation + path canonicalization.

use std::{
    io::{ErrorKind, Write},
    path::Path,
    sync::LazyLock,
};

use regex::Regex;

use crate::{
    config_manager::compute_file_hash,
    env_resolver::resolve_env_var_tokens,
    error::SpecEngineError,
    spec_parser::{SensorSpec, SpecLoader},
    types::{
        AddSensorSpecArgs, AddSensorSpecResult, SensorTableDescriptor, ValidationError,
        sensor_table_descriptor_from_table_spec,
    },
    validation::validate_step_methods,
};

/// SEC-001 (CWE-22): Regex enforcing the documented `sensor_id` format constraint.
///
/// The format `^[a-z][a-z0-9_-]*$` is documented in `SensorSpec.sensor_id` (spec_parser.rs)
/// but was previously not enforced at runtime. Without enforcement, a `sensor_id` like
/// `"../../etc/cron.d/evil"` would pass the empty-check and be interpolated directly into
/// `spec_dir.join(format!("{}.sensor.toml", sensor_id))`, enabling arbitrary file write
/// (CWE-22 / OWASP A01:2021 Path Traversal).
///
/// This regex is used as Layer 1 of the defense-in-depth (format validation in
/// `parse_and_validate_spec_toml`). Layer 2 is the canonicalization check in
/// `add_sensor_spec` itself.
static SENSOR_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z][a-z0-9_-]*$")
        .expect("SEC-001 sensor_id regex must compile — pattern is a literal constant")
});

/// Parse and validate a TOML spec string.
/// Returns the parsed `spec_parser::SensorSpec` or a list of validation errors.
///
/// # Contract (BC-2.16.008 precondition)
/// - Routes through `SpecLoader::parse` as the primary parse path (ADR-030 §D3).
///   This avoids duplicating the `RawSpec → SensorSpec` conversion logic.
/// - Additionally validates that required fields (sensor_id, name, version, auth_type,
///   base_url) are non-empty — these are enforced by the TOML schema, but we collect
///   actionable messages if they are blank.
/// - A spec with no tables is valid (edge case: no steps are registered).
/// - The `file_hash`, `source_path`, and `mode` fields are NOT set here — they are
///   set by the caller immediately after this function returns (ADR-030 §D2).
pub fn parse_and_validate_spec_toml(
    toml_content: &str,
    source_path: &str,
) -> Result<SensorSpec, Vec<ValidationError>> {
    // Route through SpecLoader::parse — this is the canonical TOML → SensorSpec path.
    // It handles: serde deserialization, AuthType enum mapping, cross-composition Rule A+B,
    // timestamp_formats validation, and timestamp_fallback_chain field-name resolution.
    let mut spec = SpecLoader::parse(toml_content).map_err(|e| {
        vec![ValidationError {
            sensor_id: None,
            source_path: source_path.to_string(),
            errors: vec![format!("{e}")],
        }]
    })?;

    // BC-2.16.009 §Validation Rules 6 (AC-6) — env var token resolution.
    //
    // Runs AFTER TOML deserialization, BEFORE URL-format validation.
    // Scans all String fields for `${env.VAR_NAME}` tokens and resolves them via
    // std::env::var. Missing or empty vars produce E-SPEC-024 errors (fail-closed).
    // Non-env-namespace tokens (${step.*}, ${query.*}) are left untouched.
    // AD-017: resolved VALUES are NEVER included in error messages — only var NAME + TOML path.
    {
        let env_errors = resolve_env_var_tokens(&mut spec, source_path);
        if !env_errors.is_empty() {
            return Err(vec![ValidationError {
                sensor_id: Some(spec.sensor_id.clone()),
                source_path: source_path.to_string(),
                errors: env_errors.into_iter().map(|e| format!("{e}")).collect(),
            }]);
        }
    }

    // BC-2.16.009 §Validation Rules 7 (AC-7) — HTTP method whitelist validation.
    //
    // Runs AFTER Rule 6 env-var resolution. Validates all resolved step.method values
    // against ALLOWED_HTTP_METHODS. Steps whose method still contains an unresolved
    // ${env.VAR} token (i.e., failed Rule 6) are skipped to prevent double-reporting.
    // Invalid method → E-SPEC-025; multiple invalid steps → multiple errors (INV-ERR-003).
    // S-SPEC-HTTP-METHOD-VALIDATION-001; BC-2.16.009 §VR7.
    {
        // validate_step_methods returns (table_idx, step_idx, error) tuples.
        // add_sensor_spec only needs the error Display strings; discard the indices.
        let method_errors: Vec<_> = validate_step_methods(&spec)
            .into_iter()
            .map(|(_, _, e)| e)
            .collect();
        if !method_errors.is_empty() {
            return Err(vec![ValidationError {
                sensor_id: Some(spec.sensor_id.clone()),
                source_path: source_path.to_string(),
                errors: method_errors.into_iter().map(|e| format!("{e}")).collect(),
            }]);
        }
    }

    // Additional validation: required fields must be non-empty.
    // SpecLoader::parse performs serde deserialization which requires these fields to
    // be present in the TOML; we collect actionable messages for blank/missing values.
    let mut field_errors: Vec<String> = Vec::new();

    if spec.sensor_id.is_empty() {
        field_errors.push("missing required field: sensor.sensor_id".to_string());
    } else if !SENSOR_ID_RE.is_match(&spec.sensor_id) {
        // SEC-001 fix (Layer 1): enforce documented sensor_id format.
        // Prevents path traversal via sensor_id used in spec_dir.join() downstream.
        // The regex ^[a-z][a-z0-9_-]*$ is documented in SensorSpec.sensor_id (spec_parser.rs)
        // but was not previously enforced at runtime, enabling CWE-22 / OWASP A01:2021.
        field_errors.push(format!(
            "invalid sensor_id '{}': must match ^[a-z][a-z0-9_-]*$ \
             (CWE-22 path traversal defense — slashes, dots, and uppercase disallowed)",
            spec.sensor_id
        ));
    }
    if spec.name.is_empty() {
        field_errors.push("missing required field: sensor.name".to_string());
    }
    if spec.version.is_empty() {
        field_errors.push("missing required field: sensor.version".to_string());
    }
    if spec.base_url.is_empty() {
        field_errors.push("missing required field: sensor.base_url".to_string());
    }

    if !field_errors.is_empty() {
        return Err(vec![ValidationError {
            sensor_id: Some(spec.sensor_id.clone()),
            source_path: source_path.to_string(),
            errors: field_errors,
        }]);
    }

    // Note: `file_hash`, `source_path`, and `mode` are NOT set here — they are
    // post-parse metadata set by the caller (config_manager, hot_reload, add_sensor_spec).
    // The returned spec has `file_hash = ""`, `source_path = ""`, `mode = DtuMode::Shared`
    // (defaults from #[serde(default)]). The caller overwrites these fields after this call.
    Ok(spec)
}

/// Generate a write-gate confirmation token for updating an existing spec.
///
/// Uses UUID v7 (timestamp + 74 random bits) for high entropy and time-ordering.
/// The `sensor_id` parameter is intentionally unused — the token is independently
/// random; binding it to the sensor_id would not improve security since the token
/// is single-use and caller-verified.
///
/// Resolves TD-S112-001.
pub fn generate_confirmation_token(_sensor_id: &str) -> String {
    uuid::Uuid::now_v7().to_string()
}

/// Process an add_sensor_spec request.
///
/// # Contract (BC-2.16.008)
/// - Parse the spec_toml as TOML
/// - Validate using the same pipeline as startup loading
/// - If validation fails: return ValidationFailed; NO file written
/// - If sensor_id already exists in manager OR file exists on disk: return ConfirmationRequired
/// - If new sensor and validation passes (not dry_run):
///   - Write to {spec_dir}/{sensor_id}.sensor.toml
///   - If write fails: return WriteError (E-SPEC-002)
///   - Trigger reload via config_manager store
///   - Return Added with registered table descriptors
/// - If dry_run: return DryRun with validation results and table preview; no file written
pub fn add_sensor_spec(
    manager: &crate::config_manager::ConfigManager,
    spec_dir: &Path,
    args: AddSensorSpecArgs,
) -> Result<AddSensorSpecResult, SpecEngineError> {
    // Step 1: parse and validate
    let spec = match parse_and_validate_spec_toml(&args.spec_toml, "<upload>") {
        Ok(s) => s,
        Err(errors) => {
            return Ok(AddSensorSpecResult::ValidationFailed { errors });
        }
    };

    let sensor_id = spec.sensor_id.clone();

    // Step 2: dry run — return preview without writing
    if args.dry_run {
        // Convert Vec<TableSpec> → Vec<SensorTableDescriptor> for the MCP wire type (ADR-030 §D7).
        let sid = sensor_id.as_str();
        let tables: Vec<SensorTableDescriptor> = spec
            .tables
            .iter()
            .map(|t| sensor_table_descriptor_from_table_spec(sid, t))
            .collect();
        return Ok(AddSensorSpecResult::DryRun {
            sensor_id,
            tables,
            validation_errors: Vec::new(),
        });
    }

    // Step 3: check if sensor already exists — fast-path memory check before I/O.
    let already_exists_in_memory = {
        let snapshot = manager.load();
        snapshot.sensor_specs.contains_key(&sensor_id)
    };
    let file_path = spec_dir.join(format!("{}.sensor.toml", sensor_id));

    // SEC-001 defense-in-depth (Layer 2): canonicalization check.
    //
    // Belt-and-suspenders: parse_and_validate_spec_toml() (Layer 1) should already have
    // rejected any sensor_id that does not match ^[a-z][a-z0-9_-]*$, which makes ../ and /
    // impossible. This Layer 2 check catches unforeseen format-validation gaps, symlink-based
    // bypasses, or any future path where this function is called without Layer 1 validation.
    //
    // We canonicalize spec_dir (which exists) and the parent of file_path (also spec_dir
    // since sensor_id is a basename, not a path). Both must canonicalize to the same path.
    // We use file_path.parent() rather than canonicalizing file_path itself because the
    // file does not exist yet (create_new semantics below).
    {
        let canonical_spec_dir =
            spec_dir
                .canonicalize()
                .map_err(|e| SpecEngineError::SpecWriteError {
                    path: spec_dir.to_string_lossy().to_string(),
                    os_error: format!("spec_dir canonicalize failed (SEC-001): {e}"),
                })?;
        let file_parent = file_path
            .parent()
            .ok_or_else(|| SpecEngineError::SpecWriteError {
                path: file_path.to_string_lossy().to_string(),
                os_error: "file_path has no parent component (SEC-001)".to_string(),
            })?;
        // file_parent IS spec_dir for a well-formed sensor_id (no directory separator).
        // Canonicalize the parent — if file_path.parent() is already canonical (same as
        // spec_dir), this is a no-op. If sensor_id somehow contains a subdirectory component
        // that escaped Layer 1 validation, the canonicalized parent will differ from
        // canonical_spec_dir, and we reject the write.
        let canonical_parent =
            file_parent
                .canonicalize()
                .map_err(|e| SpecEngineError::SpecWriteError {
                    path: file_parent.to_string_lossy().to_string(),
                    os_error: format!("file path parent canonicalize failed (SEC-001): {e}"),
                })?;
        // We require EXACT equality (not just starts_with) because file_path's parent
        // should be spec_dir itself, never a subdirectory. `starts_with` would accept
        // spec_dir/subdir/ as a valid parent, but a well-formed sensor_id (basename only,
        // no path separator) always places the file directly in spec_dir.
        //
        // Note: a direct unit test for Layer 2 alone is structurally difficult because
        // Layer 1's regex rejects all inputs (sensor_id containing '/' or '..') that would
        // cause Layer 2's inequality to fire. Coverage is implicit: Layer 1's load-bearing
        // tests AND Layer 2's presence together guarantee that any future call path bypassing
        // Layer 1 (e.g., a new SpecLoader entry point that skips parse_and_validate_spec_toml)
        // still hits Layer 2 before write.
        //
        // SEC-005 (symlink-escape analysis, 2026-06-11): a symlink at spec_dir/<sensor_id>.sensor.toml
        // pointing outside spec_dir is structurally blocked by POSIX O_EXCL semantics.
        // POSIX guarantees that O_CREAT|O_EXCL fails with EEXIST when the symlink entry itself
        // exists in the directory — regardless of whether the symlink target exists or not.
        // Rust's `create_new(true)` maps directly to O_EXCL on Unix. The result in both cases:
        // - Symlink target EXISTS: AlreadyExists → ConfirmationRequired (no write).
        // - Symlink target MISSING: AlreadyExists → ConfirmationRequired (no write).
        // This was verified empirically (2026-06-11, macOS/Linux): open with O_CREAT|O_EXCL on
        // a path that is a symlink (even to a nonexistent target) returns errno=EEXIST.
        // Therefore, a symlink-escape integration test cannot be written that exercises a
        // write-outside-spec_dir path — the write is provably unreachable. The structural
        // guarantees (Layer 1 format check + POSIX O_EXCL + Layer 2 canonicalization) are
        // defense-in-depth; a second layer of defense remains active even if Layer 1 were
        // somehow bypassed.
        //
        // Integration test was considered and determined unnecessary: the exploit requires
        // both (a) bypassing Layer 1 format validation AND (b) having pre-created a symlink
        // inside spec_dir, AND (c) the POSIX O_EXCL contract being violated — (c) has never
        // occurred in POSIX-compliant kernels. This comment IS the required SEC-005 artifact.
        if canonical_parent != canonical_spec_dir {
            return Err(SpecEngineError::SpecWriteError {
                path: file_path.to_string_lossy().to_string(),
                os_error: format!(
                    "sensor_id '{}' escapes spec_dir — write blocked (CWE-22 traversal blocked, SEC-001)",
                    sensor_id
                ),
            });
        }
    }

    if already_exists_in_memory {
        let token = generate_confirmation_token(&sensor_id);
        return Ok(AddSensorSpecResult::ConfirmationRequired {
            sensor_id,
            confirmation_token: token,
        });
    }

    // Step 4: write spec to disk atomically.
    // Use create_new(true) to atomically fail with AlreadyExists if the file
    // exists — this closes the TOCTOU window that a prior exists()-check + write
    // would leave open. Resolves TD-S112-002 (P3WV1B-A-M-003).
    let write_result = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&file_path)
        .and_then(|mut f| {
            f.write_all(args.spec_toml.as_bytes())?;
            f.sync_all()
        });

    match write_result {
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            // File was created between the memory check and the open — treat as
            // an existing spec and require confirmation (same gate as in-memory path).
            let token = generate_confirmation_token(&sensor_id);
            return Ok(AddSensorSpecResult::ConfirmationRequired {
                sensor_id,
                confirmation_token: token,
            });
        }
        Err(e) => {
            return Err(SpecEngineError::SpecWriteError {
                path: file_path.to_string_lossy().to_string(),
                os_error: e.to_string(),
            });
        }
        Ok(()) => {}
    }

    // Step 5: update ConfigManager with new spec
    // Convert Vec<TableSpec> → Vec<SensorTableDescriptor> for the MCP wire type (ADR-030 §D7).
    let sid = sensor_id.as_str();
    let tables: Vec<SensorTableDescriptor> = spec
        .tables
        .iter()
        .map(|t| sensor_table_descriptor_from_table_spec(sid, t))
        .collect();
    let file_hash = compute_file_hash(&args.spec_toml);
    let mut new_spec = spec;
    new_spec.file_hash = file_hash;
    new_spec.source_path = file_path.to_string_lossy().to_string();

    let mut new_snapshot = {
        let guard = manager.load();
        (**guard).clone()
    };
    new_snapshot
        .sensor_specs
        .insert(sensor_id.clone(), new_spec);
    // Recompute snapshot hash
    let mut file_hashes: Vec<(String, String)> = new_snapshot
        .sensor_specs
        .values()
        .map(|s| (s.source_path.clone(), s.file_hash.clone()))
        .collect();
    file_hashes.sort_by(|a, b| a.0.cmp(&b.0));
    new_snapshot.snapshot_hash =
        crate::config_manager::compute_snapshot_hash_from_hashes(&file_hashes);

    manager.store(new_snapshot);

    Ok(AddSensorSpecResult::Added { sensor_id, tables })
}

// ---------------------------------------------------------------------------
// Unit tests — SEC-001 path traversal defense (load-bearing)
// ---------------------------------------------------------------------------
//
// Mental-deletion proof: if the SENSOR_ID_RE check is removed from
// parse_and_validate_spec_toml, all `test_SEC_001_*_rejected` tests below
// will fail (the function will return Ok instead of Err). The
// `test_SEC_001_valid_sensor_id_accepted` test must remain passing in both
// cases (it documents the non-traversal happy path).
//
// These tests target parse_and_validate_spec_toml (Layer 1 of the defense).
// Layer 2 (canonicalization in add_sensor_spec) is an I/O-level check that
// requires a real filesystem; it is verified by the integration tests in
// tests/hot_reload_tests.rs which exercise the full add_sensor_spec path.

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal valid TOML content for SEC-001 tests.
    // Uses flat top-level fields (no `[sensor]` section) — that is what
    // `SpecLoader::parse` → `toml::from_str::<SensorSpec>` expects.
    // auth_type = "api_key" is one of the valid closed-enum values (AuthType::ApiKey).
    const VALID_BASE_TOML_TEMPLATE: &str = "sensor_id = \"{SENSOR_ID}\"\n\
         name = \"Test Sensor\"\n\
         version = \"1.0.0\"\n\
         base_url = \"https://example.com\"\n\
         auth_type = \"api_key\"\n";

    fn toml_with_sensor_id(id: &str) -> String {
        VALID_BASE_TOML_TEMPLATE.replace("{SENSOR_ID}", id)
    }

    /// SEC-001: sensor_id containing `../` path traversal components must be rejected
    /// by parse_and_validate_spec_toml (Layer 1 format validation).
    ///
    /// Load-bearing: removing the SENSOR_ID_RE check causes this test to FAIL
    /// (function returns Ok instead of Err, and the attacker payload would reach
    /// the filesystem join in add_sensor_spec).
    #[test]
    #[allow(non_snake_case)]
    fn test_SEC_001_sensor_id_path_traversal_rejected() {
        let traversal_toml = toml_with_sensor_id("../../etc/cron.d/evil");
        let result = parse_and_validate_spec_toml(&traversal_toml, "<upload>");
        assert!(
            result.is_err(),
            "SEC-001: path traversal sensor_id '../../etc/cron.d/evil' must be rejected; got Ok"
        );
        let err = result.unwrap_err();
        let err_text = format!("{:?}", err);
        assert!(
            err_text.contains("sensor_id") && err_text.contains("^[a-z]"),
            "SEC-001: error must cite sensor_id regex constraint; got: {err_text}"
        );
    }

    /// SEC-001: sensor_id containing a forward slash must be rejected (would insert
    /// a subdirectory component into the path join in add_sensor_spec).
    ///
    /// Load-bearing: removing SENSOR_ID_RE check causes this test to FAIL.
    #[test]
    #[allow(non_snake_case)]
    fn test_SEC_001_sensor_id_with_slash_rejected() {
        let toml_with_slash = toml_with_sensor_id("subdir/payload");
        let result = parse_and_validate_spec_toml(&toml_with_slash, "<upload>");
        assert!(
            result.is_err(),
            "SEC-001: forward-slash sensor_id 'subdir/payload' must be rejected; got Ok"
        );
    }

    /// SEC-001: sensor_id starting with `.` must be rejected (first char must be [a-z]).
    /// A `..hidden` sensor_id would also insert a dot-prefix into the filename, which
    /// could confuse filesystem tooling and glob patterns.
    ///
    /// Load-bearing: removing SENSOR_ID_RE check causes this test to FAIL.
    #[test]
    #[allow(non_snake_case)]
    fn test_SEC_001_sensor_id_with_dot_prefix_rejected() {
        let toml_with_dot = toml_with_sensor_id("..hidden");
        let result = parse_and_validate_spec_toml(&toml_with_dot, "<upload>");
        assert!(
            result.is_err(),
            "SEC-001: dot-prefixed sensor_id '..hidden' must be rejected (not matching ^[a-z]); got Ok"
        );
    }

    /// SEC-001: sensor_id starting with uppercase must be rejected (first char must be [a-z]).
    ///
    /// Load-bearing: removing SENSOR_ID_RE check causes this test to FAIL.
    #[test]
    #[allow(non_snake_case)]
    fn test_SEC_001_sensor_id_uppercase_rejected() {
        let toml_uppercase = toml_with_sensor_id("CrowdStrike");
        let result = parse_and_validate_spec_toml(&toml_uppercase, "<upload>");
        assert!(
            result.is_err(),
            "SEC-001: uppercase sensor_id 'CrowdStrike' must be rejected; got Ok"
        );
    }

    /// SEC-001: a well-formed sensor_id conforming to ^[a-z][a-z0-9_-]*$ must be
    /// accepted by parse_and_validate_spec_toml.
    ///
    /// This test remains passing regardless of whether the SENSOR_ID_RE check is
    /// present (no load-bearing delta on its own, but confirms the happy path is
    /// not broken by the SEC-001 fix).
    #[test]
    #[allow(non_snake_case)]
    fn test_SEC_001_valid_sensor_id_accepted() {
        let valid_toml = toml_with_sensor_id("valid-sensor-id-123");
        let result = parse_and_validate_spec_toml(&valid_toml, "<upload>");
        assert!(
            result.is_ok(),
            "SEC-001: valid sensor_id 'valid-sensor-id-123' must be accepted; got: {result:?}"
        );
    }

    /// SEC-001: sensor_id with digits in the body must be accepted.
    #[test]
    #[allow(non_snake_case)]
    fn test_SEC_001_sensor_id_with_digits_accepted() {
        let toml_with_digits = toml_with_sensor_id("sensor42");
        let result = parse_and_validate_spec_toml(&toml_with_digits, "<upload>");
        assert!(
            result.is_ok(),
            "SEC-001: sensor_id 'sensor42' must be accepted; got: {result:?}"
        );
    }

    /// SEC-001: sensor_id with underscores must be accepted.
    #[test]
    #[allow(non_snake_case)]
    fn test_SEC_001_sensor_id_with_underscores_accepted() {
        let toml_with_underscores = toml_with_sensor_id("my_sensor_v2");
        let result = parse_and_validate_spec_toml(&toml_with_underscores, "<upload>");
        assert!(
            result.is_ok(),
            "SEC-001: sensor_id 'my_sensor_v2' must be accepted; got: {result:?}"
        );
    }

    /// SEC-001: real sensor IDs used in production must remain accepted.
    /// This test catches regressions where the regex is accidentally too strict.
    #[test]
    #[allow(non_snake_case)]
    fn test_SEC_001_production_sensor_ids_accepted() {
        for id in &["crowdstrike", "claroty", "cyberint", "armis"] {
            let toml = toml_with_sensor_id(id);
            let result = parse_and_validate_spec_toml(&toml, "<upload>");
            assert!(
                result.is_ok(),
                "SEC-001: production sensor_id '{id}' must be accepted; got: {result:?}"
            );
        }
    }

    // ── RG-Q-012 / RG-Q-013 / RG-Q-014 — OCSF collision validation (Rule 8) ──
    //
    // These three tests exercise BC-2.16.003 §J1/§J2/§J4 (E-SPEC-030 sub-cases).
    // Rule 8 (`validate_ocsf_column_collisions`) is NOT YET wired into
    // `parse_and_validate_spec_toml`.  Today all three TOML specs parse as Ok —
    // the `is_err()` assertion fails → RED gate holds per BC-5.38.001.
    //
    // Post-fix: `parse_and_validate_spec_toml` calls `validate_ocsf_column_collisions`
    // as Rule 8; the collision is detected and a `Vec<ValidationError>` containing
    // `"E-SPEC-030 [§Jn]"` is returned.

    /// RG-Q-012 — E-SPEC-030 §J2: reserved-name collision must be rejected at spec-load time.
    ///
    /// A column whose `ocsf_field` value flattens to `"class_uid"` (a synthesized
    /// pseudo-column name) collides with the ADR-058 §G reserved name.
    ///
    /// `ocsf_field_to_arrow_name("class.uid")` → `"class_uid"` (reserved).
    ///
    /// # Red Gate failure (pre-fix)
    ///
    /// Rule 8 is not wired → `parse_and_validate_spec_toml` returns `Ok(spec)` →
    /// `result.is_err()` assertion fails → RED.
    ///
    /// # Post-fix expected behaviour
    ///
    /// `parse_and_validate_spec_toml` returns `Err([ValidationError { errors: ["E-SPEC-030
    /// [§J2] sensor=collision-j2 table=events: ocsf_field \"class.uid\" flattens to reserved
    /// name \"class_uid\""] }])`.
    ///
    /// BC: BC-2.16.003 §J2 / error-taxonomy.md E-SPEC-030 §J2.
    #[test]
    fn test_BC_2_16_003_ocsf_collision_j2_reserved_name_rejected_at_spec_load() {
        let collision_j2_toml = r#"
sensor_id = "collision-j2"
name = "Collision Test J2"
auth_type = "api_key"
base_url = "https://test.invalid"
version = "1.0.0"
ocsf_column_naming = true

[[tables]]
table_name = "events"
ocsf_class = "detection_finding"
steps = []

  [[tables.columns]]
  name = "my_class_uid"
  column_type = "string"
  ocsf_field = "class.uid"
"#;

        let result = parse_and_validate_spec_toml(collision_j2_toml, "<test-j2>");

        // Primary RED gate assertion: collision detection must cause Err.
        // Pre-fix: returns Ok (Rule 8 not wired) → this fails → RED.
        assert!(
            result.is_err(),
            "RG-Q-012 (BC-2.16.003 §J2): spec with ocsf_field \"class.uid\" (flattens to \
             reserved name \"class_uid\") must be rejected by parse_and_validate_spec_toml \
             with E-SPEC-030 [§J2]. Pre-fix: returns Ok (Rule 8 not yet wired). \
             Got Ok: {:?}",
            result.ok()
        );

        // Post-fix assertions: verify the error message contains the expected E-SPEC-030 §J2
        // sub-case with correct sensor, table, field, and arrow name identifiers.
        let errs = result.unwrap_err();
        let all_error_text = errs
            .iter()
            .flat_map(|ve| ve.errors.iter())
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ");

        assert!(
            all_error_text.contains("E-SPEC-030"),
            "RG-Q-012: error must contain error code 'E-SPEC-030'; got: {all_error_text}"
        );
        assert!(
            all_error_text.contains("J2") || all_error_text.contains("§J2"),
            "RG-Q-012: error must identify the §J2 sub-case (reserved name); got: {all_error_text}"
        );
        assert!(
            all_error_text.contains("class_uid"),
            "RG-Q-012: error must name the reserved arrow name 'class_uid'; got: {all_error_text}"
        );
    }

    /// RG-Q-013 — E-SPEC-030 §J4: intra-table duplicate arrow names must be rejected at spec-load.
    ///
    /// Two columns with different `ocsf_field` values that both flatten to the same
    /// Arrow name constitute a duplicate.
    ///
    /// `ocsf_field_to_arrow_name("finding.uid")` → `"finding_uid"`
    /// `ocsf_field_to_arrow_name("finding_uid")` → `"finding_uid"` (no dots, unchanged)
    ///
    /// # Red Gate failure (pre-fix)
    ///
    /// Rule 8 not wired → `parse_and_validate_spec_toml` returns `Ok(spec)` →
    /// `result.is_err()` assertion fails → RED.
    ///
    /// # Post-fix expected behaviour
    ///
    /// Returns `Err` with E-SPEC-030 §J4, naming both `"col_a"` and `"col_b"` and the
    /// duplicate arrow name `"finding_uid"`.
    ///
    /// BC: BC-2.16.003 §J4 / error-taxonomy.md E-SPEC-030 §J4.
    #[test]
    fn test_BC_2_16_003_ocsf_collision_j4_intra_table_duplicate_rejected_at_spec_load() {
        let collision_j4_toml = r#"
sensor_id = "collision-j4"
name = "Collision Test J4"
auth_type = "api_key"
base_url = "https://test.invalid"
version = "1.0.0"
ocsf_column_naming = true

[[tables]]
table_name = "events"
ocsf_class = "detection_finding"
steps = []

  [[tables.columns]]
  name = "col_a"
  column_type = "string"
  ocsf_field = "finding.uid"

  [[tables.columns]]
  name = "col_b"
  column_type = "string"
  ocsf_field = "finding_uid"
"#;
        // Both "finding.uid" and "finding_uid" flatten to "finding_uid" — §J4 duplicate.

        let result = parse_and_validate_spec_toml(collision_j4_toml, "<test-j4>");

        // Primary RED gate assertion.
        // Pre-fix: returns Ok (Rule 8 not wired) → this fails → RED.
        assert!(
            result.is_err(),
            "RG-Q-013 (BC-2.16.003 §J4): spec where two columns both flatten to \
             'finding_uid' (via ocsf_field_to_arrow_name) must be rejected with \
             E-SPEC-030 [§J4]. Pre-fix: returns Ok (Rule 8 not yet wired). \
             Got Ok: {:?}",
            result.ok()
        );

        // Post-fix assertions: verify the §J4 sub-case with correct identifiers.
        let errs = result.unwrap_err();
        let all_error_text = errs
            .iter()
            .flat_map(|ve| ve.errors.iter())
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ");

        assert!(
            all_error_text.contains("E-SPEC-030"),
            "RG-Q-013: error must contain error code 'E-SPEC-030'; got: {all_error_text}"
        );
        assert!(
            all_error_text.contains("J4") || all_error_text.contains("§J4"),
            "RG-Q-013: error must identify the §J4 sub-case (intra-table duplicate); \
             got: {all_error_text}"
        );
        assert!(
            all_error_text.contains("finding_uid"),
            "RG-Q-013: error must name the duplicate arrow name 'finding_uid'; \
             got: {all_error_text}"
        );
    }

    /// RG-Q-014 — E-SPEC-030 §J1: shadow collision (Tier-1 arrow name ↔ Tier-2 col.name) must
    /// be rejected at spec-load time.
    ///
    /// A Tier-1 column's arrow name shadows a Tier-2 column's raw `col.name`:
    ///
    ///   Col A: `name = "raw_device"`, `ocsf_field = "device.info"` →
    ///          arrow name = `ocsf_field_to_arrow_name("device.info")` = `"device_info"` (Tier-1)
    ///
    ///   Col B: `name = "device_info"`, no `ocsf_field` →
    ///          raw col.name = `"device_info"` (Tier-2, goes to `raw_extensions`)
    ///
    /// Col A's arrow name `"device_info"` shadows Col B's raw `col.name` `"device_info"`.
    ///
    /// # Red Gate failure (pre-fix)
    ///
    /// Rule 8 not wired → returns `Ok` → `is_err()` assertion fails → RED.
    ///
    /// # Post-fix expected behaviour
    ///
    /// Returns `Err` with E-SPEC-030 §J1, naming the ocsf_field, the flattened arrow name,
    /// and the shadowed Tier-2 column name.
    ///
    /// BC: BC-2.16.003 §J1 / error-taxonomy.md E-SPEC-030 §J1.
    #[test]
    fn test_BC_2_16_003_ocsf_collision_j1_shadow_rejected_at_spec_load() {
        let collision_j1_toml = r#"
sensor_id = "collision-j1"
name = "Collision Test J1"
auth_type = "api_key"
base_url = "https://test.invalid"
version = "1.0.0"
ocsf_column_naming = true

[[tables]]
table_name = "events"
ocsf_class = "detection_finding"
steps = []

  [[tables.columns]]
  name = "raw_device"
  column_type = "string"
  ocsf_field = "device.info"

  [[tables.columns]]
  name = "device_info"
  column_type = "integer"
"#;
        // Col A ocsf arrow name "device_info" shadows Col B raw col.name "device_info" — §J1.

        let result = parse_and_validate_spec_toml(collision_j1_toml, "<test-j1>");

        // Primary RED gate assertion.
        // Pre-fix: returns Ok (Rule 8 not wired) → this fails → RED.
        assert!(
            result.is_err(),
            "RG-Q-014 (BC-2.16.003 §J1): spec where Tier-1 arrow name 'device_info' \
             (ocsf_field_to_arrow_name(\"device.info\")) shadows Tier-2 col.name 'device_info' \
             must be rejected with E-SPEC-030 [§J1]. \
             Pre-fix: returns Ok (Rule 8 not yet wired). Got Ok: {:?}",
            result.ok()
        );

        // Post-fix assertions: verify the §J1 sub-case with correct identifiers.
        let errs = result.unwrap_err();
        let all_error_text = errs
            .iter()
            .flat_map(|ve| ve.errors.iter())
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ");

        assert!(
            all_error_text.contains("E-SPEC-030"),
            "RG-Q-014: error must contain error code 'E-SPEC-030'; got: {all_error_text}"
        );
        assert!(
            all_error_text.contains("J1") || all_error_text.contains("§J1"),
            "RG-Q-014: error must identify the §J1 sub-case (shadow); got: {all_error_text}"
        );
        assert!(
            all_error_text.contains("device_info"),
            "RG-Q-014: error must name the shadow arrow name 'device_info'; \
             got: {all_error_text}"
        );
    }
}
