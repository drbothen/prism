// S-1.12: add_sensor_spec MCP tool logic.
// BC-2.16.008: Upload a New Sensor Spec at Runtime.
// E-SPEC-002: filesystem write failure with path and OS error.

use std::io::{ErrorKind, Write};
use std::path::Path;

use crate::config_manager::compute_file_hash;
use crate::error::SpecEngineError;
use crate::spec_parser::{SensorSpec, SpecLoader};
use crate::types::{
    AddSensorSpecArgs, AddSensorSpecResult, SensorTableDescriptor, ValidationError,
    sensor_table_descriptor_from_table_spec,
};

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
    let spec = SpecLoader::parse(toml_content).map_err(|e| {
        vec![ValidationError {
            sensor_id: None,
            source_path: source_path.to_string(),
            errors: vec![format!("{e}")],
        }]
    })?;

    // Additional validation: required fields must be non-empty.
    // SpecLoader::parse performs serde deserialization which requires these fields to
    // be present in the TOML; we collect actionable messages for blank/missing values.
    let mut field_errors: Vec<String> = Vec::new();

    if spec.sensor_id.is_empty() {
        field_errors.push("missing required field: sensor.sensor_id".to_string());
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
