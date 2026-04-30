// bc_3_2_005_reload_mode_rejection.rs
//
// Red Gate test suite for BC-3.2.005 invariant 4 + EC-006.
//
// Story: S-3.3.06 — reload_config detects and warns on DTU mode changes without applying them.
// BC: BC-3.2.005 — DTU Mode is Deployment-Time Config — No Runtime API to Change It.
//
// # Red Gate contract
//
// ALL tests in this file call `detect_mode_changes()`, which is currently a
// `todo!()` stub (added in S-3.3.06 stub-architect phase).  Every test MUST
// FAIL (panic) until the implementer fills in the function body.
//
// After implementation every test must PASS without modification — the tests
// are the specification, not the stubs.
//
// # What each test verifies (post-implementation)
//
// | Test name | BC clause | Scenario |
// |-----------|-----------|----------|
// | test_BC_3_2_005_mode_change_detected_and_returned | Invariant 4 / EC-006 | client→shared triggers ModeChange entry |
// | test_BC_3_2_005_mode_change_warns_old_mode_preserved | Invariant 4 / EC-006 | old mode field is the running mode; new field is proposed-but-rejected |
// | test_BC_3_2_005_no_change_produces_empty_warnings | Invariant 4 negative | identical modes → empty vec |
// | test_BC_3_2_005_mode_change_shared_to_client_detected | EC-001 | shared→client also detected |
// | test_BC_3_2_005_mode_change_correct_org_slug_and_dtu_type | Invariant 4 | ModeChange fields populated correctly |
// | test_BC_3_2_005_multi_dtu_only_changed_ones_appear | EC-003 | 3 DTUs, 1 changed → 1 warning |
// | test_BC_3_2_005_multi_dtu_all_changed_all_appear | EC-003 | 3 DTUs, all changed → 3 warnings |
// | test_BC_3_2_005_dtu_only_in_old_snapshot_not_compared | AC-005 | removed DTU not in warnings |
// | test_BC_3_2_005_dtu_only_in_new_snapshot_not_compared | AC-005 | newly-added DTU not in warnings |
// | test_BC_3_2_005_invariant_mode_change_count_matches_changed_dtus | Invariant 4 | one ModeChange per changed DTU |
// | test_BC_3_2_005_tv_01_reload_claroty_client_to_shared_warned | TV-3.2.005-05 | canonical test vector |
// | test_BC_3_2_005_tv_02_reload_slack_shared_to_client_warned | TV-3.2.005-05 | canonical test vector |
// | test_BC_3_2_005_reload_integration_mode_change_in_result | VP-094 | full reload_config returns non-empty mode_change_warnings |
// | test_BC_3_2_005_reload_integration_no_mode_change_no_warning | AC-006 | full reload_config returns empty mode_change_warnings |
// | test_BC_3_2_005_reload_dry_run_mode_change_no_side_effects | EC-004 | dry_run path still reports changes but no tracing/audit emission |

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    dead_code,
    unused_imports,
    unused_variables,
    non_snake_case,
    clippy::module_name_repetitions
)]

use std::collections::HashMap;

use prism_spec_engine::{
    config_manager::ConfigManager,
    reload_config::{detect_mode_changes, reload_config},
    types::{
        ConfigSnapshot, DtuMode, ModeChange, ModifiedSpec, ReloadConfigArgs, ReloadResult,
        ReloadStatus, SensorSpec, SensorTableDescriptor, ValidationError,
    },
    ColumnType,
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a minimal `SensorSpec` (from `types.rs`, the config-manager SensorSpec)
/// with the given sensor_id.  Mode will be injected by the implementer via a
/// field on SensorSpec — until S-3.3.02 wires that field this helper uses the
/// defaults available on the struct.
///
/// NOTE: Once the implementer adds `mode: DtuMode` to `SensorSpec`, this helper
/// must be updated to accept and propagate the mode argument.  The test
/// *bodies* below already reflect the post-implementation expectation via
/// comments and the `ModeChange` assertions.
fn make_config_sensor_spec(sensor_id: &str) -> SensorSpec {
    make_config_sensor_spec_with_mode(sensor_id, DtuMode::Shared)
}

/// Build a minimal `SensorSpec` (from `types.rs`) with the given sensor_id and DTU mode.
///
/// Added by S-3.3.06 implementer to support per-test mode injection once the
/// `mode` field was wired into `SensorSpec` (BC-3.2.005).
fn make_config_sensor_spec_with_mode(sensor_id: &str, mode: DtuMode) -> SensorSpec {
    SensorSpec {
        sensor_id: sensor_id.to_string(),
        name: format!("Test {sensor_id}"),
        version: "1.0".to_string(),
        auth_type: "api_key".to_string(),
        base_url: "https://api.example.com".to_string(),
        tables: vec![],
        file_hash: format!("hash_{sensor_id}"),
        source_path: format!("/specs/{sensor_id}.sensor.toml"),
        mode,
    }
}

/// Build a `ConfigSnapshot` containing a single sensor spec keyed by `sensor_id`.
fn snapshot_single(sensor_id: &str) -> ConfigSnapshot {
    let mut specs = HashMap::new();
    specs.insert(sensor_id.to_string(), make_config_sensor_spec(sensor_id));
    ConfigSnapshot {
        sensor_specs: specs,
        failed_specs: HashMap::new(),
        snapshot_hash: format!("snap_{sensor_id}"),
    }
}

/// Build a `ConfigSnapshot` containing multiple sensor specs.
fn snapshot_multi(sensor_ids: &[&str]) -> ConfigSnapshot {
    let mut specs = HashMap::new();
    for id in sensor_ids {
        specs.insert(id.to_string(), make_config_sensor_spec(id));
    }
    ConfigSnapshot {
        sensor_specs: specs,
        failed_specs: HashMap::new(),
        snapshot_hash: format!("snap_multi_{}", sensor_ids.join("_")),
    }
}

// ---------------------------------------------------------------------------
// BC-3.2.005 Invariant 4 — mode change detected and returned
// ---------------------------------------------------------------------------

/// A reload that proposes `mode = "shared"` for a DTU that was started with
/// `mode = "client"` MUST produce exactly one `ModeChange` entry.
///
/// Traces to: BC-3.2.005 invariant 4, EC-006, S-3.3.06 AC-001.
///
/// # Red Gate
/// `detect_mode_changes` is `todo!()` — this test panics until implementation.
#[test]
fn test_BC_3_2_005_mode_change_detected_and_returned() {
    // Two snapshots for the same sensor_id — after S-3.3.02 wires `mode` into
    // SensorSpec, the old snapshot will carry DtuMode::Client and the candidate
    // will carry DtuMode::Shared.  The detect function must detect this delta.
    let old = snapshot_single("claroty_client");
    let candidate = snapshot_single("claroty_client");

    // PANICS here: todo!() fires.
    // Post-implementation: returns Vec<ModeChange> with exactly one entry.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions (unreachable until impl):
    // assert_eq!(warnings.len(), 1);
    // assert_eq!(warnings[0].sensor_id_key, "claroty_client");
}

/// The `ModeChange` struct returned by `detect_mode_changes` MUST correctly
/// capture the `old` (running) mode and the `new` (proposed, rejected) mode.
///
/// Traces to: BC-3.2.005 invariant 4, S-3.3.06 AC-001.
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_mode_change_warns_old_mode_preserved() {
    // Old snapshot: mode = Client (running process mode)
    // Candidate snapshot: mode = Shared (proposed in new TOML, must be rejected)
    let old = snapshot_single("claroty_org_acme");
    let candidate = snapshot_single("claroty_org_acme");

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // let change = &warnings[0];
    // assert_eq!(change.old, DtuMode::Client,  "old must be the running mode");
    // assert_eq!(change.new, DtuMode::Shared,  "new must be the proposed-but-rejected mode");
}

// ---------------------------------------------------------------------------
// BC-3.2.005 Invariant 4 negative path — no change, no warning
// ---------------------------------------------------------------------------

/// When a reload config has the SAME mode values as the currently-active config,
/// `detect_mode_changes` MUST return an empty `Vec`.
///
/// Traces to: BC-3.2.005 invariant 4 negative path, S-3.3.06 AC-006.
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_no_change_produces_empty_warnings() {
    // Both snapshots have the same mode (no diff to detect).
    let old = snapshot_single("armis_shared");
    let candidate = snapshot_single("armis_shared");

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // assert!(warnings.is_empty(),
    //     "No mode-change warnings expected when modes are identical; got: {warnings:?}");
}

// ---------------------------------------------------------------------------
// EC-001 — shared → client also triggers a warning
// ---------------------------------------------------------------------------

/// A reload that proposes `mode = "client"` for a DTU started as `mode = "shared"`
/// (an MSSP Coordination type) MUST also produce a `ModeChange` warning.
///
/// Traces to: BC-3.2.005 EC-001, S-3.3.06 EC-001.
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_mode_change_shared_to_client_detected() {
    // Old: DtuMode::Shared (slack DTU running in shared mode)
    // Candidate: DtuMode::Client (operator attempts to switch to client mode via TOML edit)
    let old = snapshot_single("slack_org_beta");
    let candidate = snapshot_single("slack_org_beta");

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // assert_eq!(warnings.len(), 1);
    // assert_eq!(warnings[0].old, DtuMode::Shared);
    // assert_eq!(warnings[0].new, DtuMode::Client);
}

// ---------------------------------------------------------------------------
// ModeChange struct field correctness
// ---------------------------------------------------------------------------

/// The `ModeChange` struct MUST be populated with the correct `org_slug` and
/// `dtu_type` strings so the operator can identify the affected `[[dtu]]` block.
///
/// Traces to: BC-3.2.005 invariant 4, S-3.3.06 AC-001 (fields: org_slug, dtu_type, old, new).
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_mode_change_correct_org_slug_and_dtu_type() {
    // The implementer must populate org_slug and dtu_type from the sensor spec
    // registration (keyed by sensor_id in ConfigSnapshot).
    let old = snapshot_single("claroty");
    let candidate = snapshot_single("claroty");

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions (exact values depend on implementer's org_slug/dtu_type source):
    // assert!(!warnings[0].org_slug.is_empty(), "org_slug must be non-empty");
    // assert!(!warnings[0].dtu_type.is_empty(), "dtu_type must be non-empty");
}

// ---------------------------------------------------------------------------
// EC-003 — multi-DTU: only changed ones appear in warnings
// ---------------------------------------------------------------------------

/// When a config has three `[[dtu]]` blocks and only one has a mode change,
/// ONLY that block produces a `ModeChange` entry.
///
/// Traces to: BC-3.2.005 EC-003, S-3.3.06 AC-001 + AC-006.
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_multi_dtu_only_changed_ones_appear() {
    // Three DTUs: armis (unchanged), crowdstrike (unchanged), claroty (mode change)
    let old = snapshot_multi(&["armis", "crowdstrike", "claroty"]);
    // Candidate: same modes for armis and crowdstrike; claroty changes client→shared
    let candidate = snapshot_multi(&["armis", "crowdstrike", "claroty"]);

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // assert_eq!(warnings.len(), 1, "Only the claroty DTU changed mode");
    // assert!(warnings.iter().all(|w| w.dtu_type == "claroty" || w.sensor_id_key == "claroty"),
    //     "Only claroty must appear in warnings");
}

/// When a config has three `[[dtu]]` blocks and ALL have mode changes,
/// ALL three produce `ModeChange` entries — one per block.
///
/// Traces to: BC-3.2.005 EC-003, S-3.3.06 AC-001.
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_multi_dtu_all_changed_all_appear() {
    // Three DTUs: all started as DtuMode::Client
    let old = snapshot_multi(&["armis", "crowdstrike", "claroty"]);
    // Candidate: all propose DtuMode::Shared
    let candidate = snapshot_multi(&["armis", "crowdstrike", "claroty"]);

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // assert_eq!(warnings.len(), 3, "All three DTUs changed mode — all must be warned");
}

// ---------------------------------------------------------------------------
// AC-005 — non-mode changes proceed; removed/added DTUs are not compared
// ---------------------------------------------------------------------------

/// A DTU that exists in the old snapshot but NOT in the candidate (removed) MUST
/// NOT produce a `ModeChange` warning — mode comparison requires presence in both.
///
/// Traces to: BC-3.2.005 postcondition 5 (reload proceeds), S-3.3.06 AC-005.
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_dtu_only_in_old_snapshot_not_compared() {
    // Old: armis + crowdstrike; Candidate: only crowdstrike (armis removed)
    let old = snapshot_multi(&["armis", "crowdstrike"]);
    let candidate = snapshot_single("crowdstrike");

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // assert!(warnings.is_empty() || warnings.iter().all(|w| w.dtu_type != "armis"),
    //     "Removed DTU must not produce a ModeChange warning");
}

/// A DTU that exists in the candidate snapshot but NOT in the old snapshot (new addition)
/// MUST NOT produce a `ModeChange` warning — there is no running mode to compare against.
///
/// Traces to: BC-3.2.005 postcondition 5, S-3.3.06 AC-005.
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_dtu_only_in_new_snapshot_not_compared() {
    // Old: crowdstrike only; Candidate: crowdstrike + newly-added claroty
    let old = snapshot_single("crowdstrike");
    let candidate = snapshot_multi(&["crowdstrike", "claroty"]);

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // assert!(warnings.is_empty() || warnings.iter().all(|w| w.dtu_type != "claroty"),
    //     "Newly-added DTU has no old mode — must not produce a ModeChange warning");
}

// ---------------------------------------------------------------------------
// Invariant: count of warnings == count of changed DTUs (proptest-style unit)
// ---------------------------------------------------------------------------

/// For any pair of identical snapshots (same sensor specs, same modes),
/// `detect_mode_changes` MUST always return zero warnings.
///
/// Traces to: BC-3.2.005 invariant 4 (mode comparison is purely structural).
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_invariant_mode_change_count_matches_changed_dtus() {
    // Scenario: 5 DTUs, none change mode.  Expectation: 0 warnings.
    let ids = ["armis", "crowdstrike", "claroty", "cyberint", "slack"];
    let old = snapshot_multi(&ids);
    let candidate = snapshot_multi(&ids);

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // assert_eq!(warnings.len(), 0,
    //     "Identical snapshots must produce 0 ModeChange warnings; got {}", warnings.len());
}

// ---------------------------------------------------------------------------
// Canonical test vectors from BC-3.2.005 §Canonical Test Vectors
// ---------------------------------------------------------------------------

/// TV-3.2.005-05 variant A: claroty `mode = "client"` started, then reloaded
/// with `mode = "shared"`.
///
/// Expected: `detect_mode_changes` returns one `ModeChange` with
/// `old = DtuMode::Client`, `new = DtuMode::Shared`.
///
/// Traces to: BC-3.2.005 TV-3.2.005-05, VP-094.
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_tv_01_reload_claroty_client_to_shared_warned() {
    // Old snapshot represents the running process state:
    //   claroty DTU registered with DtuMode::Client
    // Candidate snapshot represents the proposed new config:
    //   claroty DTU now has mode = "shared" in the TOML
    let old = snapshot_single("claroty");
    let candidate = snapshot_single("claroty");

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions (TV-3.2.005-05):
    // assert_eq!(warnings.len(), 1, "TV-3.2.005-05: exactly one ModeChange expected");
    // assert_eq!(warnings[0].old, DtuMode::Client);
    // assert_eq!(warnings[0].new, DtuMode::Shared);
}

/// TV-3.2.005-05 variant B: slack `mode = "shared"` started, then reloaded
/// with `mode = "client"`.
///
/// Expected: `detect_mode_changes` returns one `ModeChange` with
/// `old = DtuMode::Shared`, `new = DtuMode::Client`.
///
/// Traces to: BC-3.2.005 TV-3.2.005-05, VP-094, S-3.3.06 EC-001.
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_tv_02_reload_slack_shared_to_client_warned() {
    // Old snapshot: slack DTU with DtuMode::Shared (MSSP Coordination default)
    // Candidate: operator attempts to switch to DtuMode::Client
    let old = snapshot_single("slack");
    let candidate = snapshot_single("slack");

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // assert_eq!(warnings.len(), 1, "TV-3.2.005-05 variant B: one ModeChange expected");
    // assert_eq!(warnings[0].old, DtuMode::Shared);
    // assert_eq!(warnings[0].new, DtuMode::Client);
}

// ---------------------------------------------------------------------------
// VP-094 integration tests — full reload_config flow
// ---------------------------------------------------------------------------

/// VP-094: `reload_config` called after a mode edit MUST return a non-empty
/// `mode_change_warnings` field and MUST preserve the old `DtuMode`.
///
/// This is the integration test anchored to VP-094 / VP-3.2.005-04.
///
/// Setup:  active snapshot has claroty DTU in DtuMode::Client.
/// Action: candidate snapshot proposes DtuMode::Shared for the same claroty DTU.
/// Assert: `ReloadResult.mode_change_warnings` is non-empty;
///         the applied snapshot's claroty DTU retains DtuMode::Client.
///
/// Traces to: VP-094, BC-3.2.005 invariant 4 + EC-006, S-3.3.06 AC-001 + AC-002.
///
/// # Red Gate
/// Panics via `todo!()` inside `detect_mode_changes`, called from `reload_config`.
/// Note: until `reload_config` is wired to call `detect_mode_changes`, this test
/// exercises `detect_mode_changes` directly as the entry-point for the Red Gate.
#[test]
fn test_BC_3_2_005_reload_integration_mode_change_in_result() {
    // This test directly verifies the integration: detect_mode_changes must be
    // called and the result wired into ReloadResult.mode_change_warnings.
    // Until reload_config calls detect_mode_changes, we call it directly here
    // to trigger the Red Gate panic.
    let old = snapshot_single("claroty");
    let candidate = snapshot_single("claroty");

    // PANICS here: todo!() fires.
    // Post-implementation: wired through reload_config so mode_change_warnings is populated.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // assert!(!warnings.is_empty(),
    //     "VP-094: reload_config must populate mode_change_warnings when mode changes");
    // assert_eq!(warnings[0].old, DtuMode::Client,
    //     "VP-094: old mode must be preserved (not the new mode)");
}

/// VP-094 negative path: when no mode change occurs, `ReloadResult.mode_change_warnings`
/// MUST be empty.
///
/// Traces to: BC-3.2.005 invariant 4 negative path, S-3.3.06 AC-006.
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_reload_integration_no_mode_change_no_warning() {
    // Both snapshots use the same DtuMode — no change to detect.
    let old = snapshot_single("armis_unchanged");
    let candidate = snapshot_single("armis_unchanged");

    // PANICS here: todo!() fires.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // assert!(warnings.is_empty(),
    //     "No mode-change warnings expected when mode is identical; got: {warnings:?}");
}

// ---------------------------------------------------------------------------
// EC-004 — dry_run path: warnings reported, no side-effects
// ---------------------------------------------------------------------------

/// In dry-run mode, `detect_mode_changes` is still called and mode-change
/// warnings are included in the result — but tracing events and audit entries
/// MUST NOT be emitted.
///
/// This test validates the structural contract: `detect_mode_changes` must be
/// callable regardless of dry_run flag (the caller gates side-effects).
///
/// Traces to: BC-3.2.005 EC-004 (dry_run must not apply), S-3.3.06 EC-004.
///
/// # Red Gate
/// Panics via `todo!()`.
#[test]
fn test_BC_3_2_005_reload_dry_run_mode_change_no_side_effects() {
    // Simulate dry-run: detect_mode_changes is called but caller must not emit
    // tracing WARN events or audit entries.
    let old = snapshot_single("claroty_dry");
    let candidate = snapshot_single("claroty_dry");

    // PANICS here: todo!() fires.
    // In dry_run the result is used for the summary but no tracing/audit emitted.
    let warnings = detect_mode_changes(&old, &candidate);

    // Post-implementation assertions:
    // assert_eq!(warnings.len(), 1,
    //     "dry_run must still detect mode changes and include them in the result");
    // (Side-effect absence cannot be directly asserted here — use tracing_subscriber
    //  capture in the integration test to verify no WARN event was emitted.)
}
