// S-1.12: add_sensor_spec MCP tool logic.
// BC-2.16.008: Upload a New Sensor Spec at Runtime.
// E-SPEC-002: filesystem write failure with path and OS error.

use std::io::{ErrorKind, Write};
use std::path::Path;

use crate::config_manager::compute_file_hash;
use crate::error::SpecEngineError;
use crate::types::{
    AddSensorSpecArgs, AddSensorSpecResult, ColumnDef, ColumnType, PaginationType, SensorSpec,
    SensorTableDescriptor, ValidationError,
};

// ──────────────────────────────────────────────────────────────────────────────
// TOML shape mirrors the minimal_valid_sensor_toml helper in the test suite.
// ──────────────────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct RawSpec {
    sensor: RawSensorSection,
    #[serde(default)]
    tables: Vec<RawTable>,
}

#[derive(serde::Deserialize)]
struct RawSensorSection {
    sensor_id: Option<String>,
    name: Option<String>,
    version: Option<String>,
    auth_type: Option<String>,
    base_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct RawTable {
    table_name: String,
    #[serde(default)]
    columns: Vec<RawColumn>,
    #[serde(default)]
    steps: Vec<RawStep>,
}

#[derive(serde::Deserialize)]
struct RawColumn {
    name: String,
    #[serde(rename = "type")]
    column_type: String,
    #[serde(default = "default_true")]
    nullable: bool,
}

fn default_true() -> bool {
    true
}

#[derive(serde::Deserialize, Default)]
struct RawStep {
    #[allow(dead_code)]
    url: Option<String>,
    #[allow(dead_code)]
    method: Option<String>,
    #[serde(default)]
    pagination: Option<String>,
}

/// Parse and validate a TOML spec string.
/// Returns the parsed SensorSpec or a list of validation errors.
///
/// # Contract (BC-2.16.008 precondition)
/// - Rejects invalid TOML syntax before any I/O
/// - Validates required fields: sensor_id, name, version, auth_type, base_url
/// - A spec with no tables is valid (edge case: no steps are registered)
pub fn parse_and_validate_spec_toml(
    toml_content: &str,
    source_path: &str,
) -> Result<SensorSpec, Vec<ValidationError>> {
    // Step 1: parse TOML syntax
    let raw: RawSpec = toml::from_str(toml_content).map_err(|e| {
        vec![ValidationError {
            sensor_id: None,
            source_path: source_path.to_string(),
            errors: vec![format!("TOML parse error: {}", e)],
        }]
    })?;

    // Step 2: validate required sensor section fields
    let mut field_errors: Vec<String> = Vec::new();

    if raw.sensor.sensor_id.as_deref().unwrap_or("").is_empty() {
        field_errors.push("missing required field: sensor.sensor_id".to_string());
    }
    if raw.sensor.name.as_deref().unwrap_or("").is_empty() {
        field_errors.push("missing required field: sensor.name".to_string());
    }
    if raw.sensor.version.as_deref().unwrap_or("").is_empty() {
        field_errors.push("missing required field: sensor.version".to_string());
    }
    if raw.sensor.auth_type.as_deref().unwrap_or("").is_empty() {
        field_errors.push("missing required field: sensor.auth_type".to_string());
    }
    if raw.sensor.base_url.as_deref().unwrap_or("").is_empty() {
        field_errors.push("missing required field: sensor.base_url".to_string());
    }

    if !field_errors.is_empty() {
        return Err(vec![ValidationError {
            sensor_id: raw.sensor.sensor_id.clone(),
            source_path: source_path.to_string(),
            errors: field_errors,
        }]);
    }

    // Safety: all fields validated above — None/empty checked and returned early
    let sensor_id = raw.sensor.sensor_id.expect("sensor_id validated above");
    let name = raw.sensor.name.expect("name validated above");
    let version = raw.sensor.version.expect("version validated above");
    let auth_type = raw.sensor.auth_type.expect("auth_type validated above");
    let base_url = raw.sensor.base_url.expect("base_url validated above");

    // Step 3: convert tables
    let mut tables = Vec::new();
    for raw_table in &raw.tables {
        let columns: Vec<ColumnDef> = raw_table
            .columns
            .iter()
            .map(|c| ColumnDef {
                name: c.name.clone(),
                column_type: parse_column_type(&c.column_type),
                ocsf_field: None,
                nullable: c.nullable,
            })
            .collect();

        let pagination_type = raw_table
            .steps
            .first()
            .and_then(|s| s.pagination.as_deref())
            .map(parse_pagination_type)
            .unwrap_or(PaginationType::None);

        // Fully-qualified table name: "{sensor_id}.{table_name}"
        let table_name = format!("{}.{}", sensor_id, raw_table.table_name);

        tables.push(SensorTableDescriptor {
            table_name,
            columns,
            steps_count: raw_table.steps.len(),
            pagination_type,
        });
    }

    Ok(SensorSpec {
        sensor_id,
        name,
        version,
        auth_type,
        base_url,
        tables,
        file_hash: String::new(), // filled by caller
        source_path: source_path.to_string(),
    })
}

fn parse_column_type(s: &str) -> ColumnType {
    match s.to_lowercase().as_str() {
        "string" | "text" | "varchar" => ColumnType::String,
        "int64" | "int" | "integer" | "bigint" => ColumnType::Int64,
        "float64" | "float" | "double" | "real" => ColumnType::Float64,
        "boolean" | "bool" => ColumnType::Boolean,
        "timestamp" | "datetime" => ColumnType::Timestamp,
        "json" | "object" => ColumnType::Json,
        _ => ColumnType::String, // default to string for unknown types
    }
}

fn parse_pagination_type(s: &str) -> PaginationType {
    match s.to_lowercase().as_str() {
        "cursor" => PaginationType::Cursor,
        "offset" => PaginationType::Offset,
        _ => PaginationType::None,
    }
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
        return Ok(AddSensorSpecResult::DryRun {
            sensor_id,
            tables: spec.tables.clone(),
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
    let tables = spec.tables.clone();
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
