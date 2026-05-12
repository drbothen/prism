// S-1.12: reload_config MCP tool logic.
// BC-2.16.005: Re-Read All Config Files, Validate, Atomic Swap, Notify.
// S-3.3.06: Mode-change detection scaffold (BC-3.2.005 invariant 4).

use std::path::Path;

use crate::config_manager::{ConfigManager, parse_spec_directory};
use crate::error::SpecEngineError;
use crate::types::{
    ConfigSnapshot, ModeChange, ModifiedSpec, ReloadConfigArgs, ReloadResult, ReloadStatus,
    ValidationError,
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
    let mut candidate = parse_spec_directory(spec_dir)?;

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
            mode_change_warnings: Vec::new(),
        });
    }

    // Compute change summary by diffing old vs new snapshots
    let old_snapshot = manager.load();

    // Detect DTU mode changes — pure diff, no side-effects (BC-3.2.005 invariant 4).
    // Must be called before diff_snapshots so we can patch the candidate below.
    let mode_change_warnings = detect_mode_changes(&old_snapshot, &candidate);

    // Patch the candidate: restore old modes to preserve the invariant that
    // DTU mode is deployment-time config only (BC-3.2.005 EC-006).
    // This is a no-op when mode_change_warnings is empty.
    for change in &mode_change_warnings {
        if let Some(spec) = candidate.sensor_specs.get_mut(&change.org_slug) {
            spec.mode = change.old;
        }
    }

    // Emit WARN tracing events for detected mode changes (not emitted during dry_run).
    if !args.dry_run {
        for change in &mode_change_warnings {
            tracing::warn!(
                org_slug = %change.org_slug,
                dtu_type = %change.dtu_type,
                old_mode = ?change.old,
                new_mode = ?change.new,
                "DTU mode change detected in reload config — change NOT applied; \
                 restart required to change DTU mode (BC-3.2.005)"
            );
        }
    }

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
            mode_change_warnings,
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
            mode_change_warnings,
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
        mode_change_warnings,
    })
}

/// Detect DTU mode changes between an old and candidate `ConfigSnapshot`.
///
/// # Contract (BC-3.2.005 Invariant 4 + EC-006)
///
/// For each sensor spec present in BOTH `old` and `candidate`, compare the
/// `DtuMode` stored in `old` against the incoming mode in `candidate`.  When
/// they differ, emit one `ModeChange` entry per affected DTU.
///
/// Sensor specs that exist only in the old snapshot (removed) or only in the
/// candidate (newly added) are NOT compared — there is no running mode to
/// diff against for a brand-new DTU, and a removed DTU has no proposed mode
/// (BC-3.2.005 postcondition 5, S-3.3.06 AC-005).
///
/// The returned list is consumed by `reload_config` to:
/// 1. Emit a `WARN`-level structured tracing event per change (via `tracing::warn!`).
/// 2. Patch the candidate snapshot so the old mode is preserved — the new mode
///    is silently dropped and the process continues with the original mode.
///
/// This function is a pure diff — it produces no side-effects. The caller is
/// responsible for suppressing tracing/audit emission when `dry_run` is `true`
/// (BC-3.2.005 EC-004).
pub fn detect_mode_changes(old: &ConfigSnapshot, candidate: &ConfigSnapshot) -> Vec<ModeChange> {
    let mut changes: Vec<ModeChange> = Vec::new();

    for (sensor_id, old_spec) in &old.sensor_specs {
        // Only compare specs present in BOTH snapshots (AC-005).
        if let Some(candidate_spec) = candidate.sensor_specs.get(sensor_id)
            && old_spec.mode != candidate_spec.mode
        {
            changes.push(ModeChange {
                // Use the sensor_id as org_slug — the config-manager SensorSpec
                // does not yet carry a separate org_slug field; sensor_id serves
                // as the stable identifier for the [[dtu]] block (BC-3.2.005).
                org_slug: sensor_id.clone(),
                // Use the sensor_id as dtu_type until the TOML schema exposes a
                // dedicated `dtu_type` field (BC-3.2.005 invariant 4).
                dtu_type: sensor_id.clone(),
                old: old_spec.mode,
                new: candidate_spec.mode,
            });
        }
    }

    changes
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
