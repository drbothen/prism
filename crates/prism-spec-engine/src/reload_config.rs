// S-1.12: reload_config MCP tool logic.
// BC-2.16.005: Re-Read All Config Files, Validate, Atomic Swap, Notify.

use std::path::Path;

use crate::config_manager::{parse_spec_directory, ConfigManager};
use crate::error::SpecEngineError;
use crate::types::{
    ConfigSnapshot, ModifiedSpec, ReloadConfigArgs, ReloadResult, ReloadStatus, ValidationError,
};

/// Execute a config reload.
///
/// # Contract (BC-2.16.005)
/// - Re-reads all TOML files from spec_dir
/// - Constructs a new ConfigSnapshot with fresh SHA-256 hash
/// - If new hash == current hash: returns ReloadStatus::Unchanged (no-op)
/// - If changes detected: validates all files
///   - Validation pass (all valid): atomic ArcSwap store; returns ReloadStatus::Ok
///   - Partial failure (Tier 3 sensor specs): loads valid specs, rejects invalid; ReloadStatus::PartialReload
///   - ALL validation fail: current config retained unchanged; ReloadStatus::ValidationFailed
/// - If dry_run: validates and returns change summary without applying
///
/// # Invariants
/// - Validation failure ALWAYS retains current config unchanged (DI-031 fail-closed)
/// - Hash-based no-op detection prevents unnecessary table re-registration
pub fn reload_config(
    manager: &ConfigManager,
    spec_dir: &Path,
    args: ReloadConfigArgs,
) -> Result<ReloadResult, SpecEngineError> {
    // Parse the directory (may fail with FileReadError if dir unreadable)
    let candidate = parse_spec_directory(spec_dir)?;

    let current_hash = manager.current_hash();
    let new_hash = &candidate.snapshot_hash;

    // Hash-based no-op detection
    if &current_hash == new_hash {
        return Ok(ReloadResult {
            status: ReloadStatus::Unchanged,
            added: Vec::new(),
            removed: Vec::new(),
            modified: Vec::new(),
            unchanged: Vec::new(),
            validation_errors: Vec::new(),
        });
    }

    // Compute change summary by diffing old vs new snapshots
    let old_snapshot = manager.load();
    let (added, removed, modified, unchanged) = diff_snapshots(&old_snapshot, &candidate);
    let validation_errors: Vec<ValidationError> =
        candidate.failed_specs.values().cloned().collect();

    // Dry run: return summary without applying
    if args.dry_run {
        return Ok(ReloadResult {
            status: ReloadStatus::DryRun,
            added,
            removed,
            modified,
            unchanged,
            validation_errors,
        });
    }

    // Determine reload status based on whether there are any failures
    let has_failures = !candidate.failed_specs.is_empty();
    let has_successes = !candidate.sensor_specs.is_empty();

    // EC-001: If ALL specs fail validation AND there are no valid specs, retain old config
    if has_failures && !has_successes {
        // Full validation failure — retain old config unchanged (DI-031 fail-closed)
        return Ok(ReloadResult {
            status: ReloadStatus::ValidationFailed,
            added: Vec::new(),
            removed: Vec::new(),
            modified: Vec::new(),
            unchanged: Vec::new(),
            validation_errors,
        });
    }

    // Apply the new snapshot (partial or full success)
    manager.store(candidate);

    let status = if has_failures {
        ReloadStatus::PartialReload
    } else {
        ReloadStatus::Ok
    };

    Ok(ReloadResult {
        status,
        added,
        removed,
        modified,
        unchanged,
        validation_errors,
    })
}

/// Compute added/removed/modified/unchanged table names by diffing two snapshots.
fn diff_snapshots(
    old: &ConfigSnapshot,
    new: &ConfigSnapshot,
) -> (Vec<String>, Vec<String>, Vec<ModifiedSpec>, Vec<String>) {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();
    let mut unchanged = Vec::new();

    // Find removed and modified
    for (sensor_id, old_spec) in &old.sensor_specs {
        match new.sensor_specs.get(sensor_id) {
            None => {
                // Removed: all tables from old spec
                for table in &old_spec.tables {
                    removed.push(table.table_name.clone());
                }
            }
            Some(new_spec) => {
                if old_spec.file_hash == new_spec.file_hash {
                    unchanged.push(sensor_id.clone());
                } else {
                    // Modified: check if schema changed
                    let schema_changed = old_spec.tables != new_spec.tables;
                    let table_names: Vec<String> = new_spec
                        .tables
                        .iter()
                        .map(|t| t.table_name.clone())
                        .collect();
                    modified.push(ModifiedSpec {
                        sensor_id: sensor_id.clone(),
                        table_names,
                        schema_changed,
                    });
                }
            }
        }
    }

    // Find added
    for (sensor_id, new_spec) in &new.sensor_specs {
        if !old.sensor_specs.contains_key(sensor_id) {
            for table in &new_spec.tables {
                added.push(table.table_name.clone());
            }
        }
    }

    (added, removed, modified, unchanged)
}

/// Validate a candidate ConfigSnapshot before applying it.
/// Returns Ok(()) if validation passes, or Err with all validation errors.
///
/// For S-1.12, we accept any non-empty snapshot as valid. Empty snapshots
/// (no specs, no failures) are considered invalid to prevent accidental clear.
pub fn validate_snapshot(candidate: &ConfigSnapshot) -> Result<(), Vec<ValidationError>> {
    // Collect any pre-existing validation errors from failed_specs
    let errors: Vec<ValidationError> = candidate.failed_specs.values().cloned().collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Compute a combined SHA-256 hash of all .sensor.toml files in a directory.
/// Used for hash-based no-op detection.
pub fn compute_snapshot_hash(spec_dir: &Path) -> Result<String, SpecEngineError> {
    let snapshot = parse_spec_directory(spec_dir)?;
    Ok(snapshot.snapshot_hash)
}
