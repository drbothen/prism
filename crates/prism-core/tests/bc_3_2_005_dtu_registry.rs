//! Acceptance tests for S-3.0.02 — DTU_DEFAULT_MODE registry.
//!
//! Traces to BC-3.2.005 (DTU mode is deployment-time config; no runtime API).
//! Verification properties covered: VP-091, VP-092, VP-093, VP-094.
//!
//! Red Gate discipline: AC-4 through AC-8 MUST fail until the implementer
//! populates DTU_DEFAULT_MODE with the 10 canonical entries.

use prism_core::{DtuMode, DtuRegistryEntry, DTU_DEFAULT_MODE};
use std::process::Command;

// ---------------------------------------------------------------------------
// AC-1 / BC-3.2.005 precondition 3
// DtuMode serde round-trip: "shared" -> Shared, "client" -> Client.
// These pass at Red Gate because the enum already exists.
// ---------------------------------------------------------------------------

/// AC-1: "shared" deserializes to DtuMode::Shared.
#[test]
fn test_bc_3_2_005_ac1_serde_shared() {
    let mode: DtuMode = serde_json::from_str("\"shared\"").expect("deserialize 'shared'");
    assert_eq!(mode, DtuMode::Shared);
}

/// AC-1: "client" deserializes to DtuMode::Client.
#[test]
fn test_bc_3_2_005_ac1_serde_client() {
    let mode: DtuMode = serde_json::from_str("\"client\"").expect("deserialize 'client'");
    assert_eq!(mode, DtuMode::Client);
}

/// AC-1: Unknown mode string is rejected by serde (VP-092).
/// VP-092: startup rejects unknown mode values.
#[test]
fn test_bc_3_2_005_ac1_vp092_serde_rejects_unknown_mode() {
    let result: Result<DtuMode, _> = serde_json::from_str("\"Hybrid\"");
    assert!(
        result.is_err(),
        "DtuMode serde must reject any value other than 'shared'/'client'"
    );
}

/// AC-1: Mixed-case variants are rejected (serde rename_all = "lowercase").
#[test]
fn test_bc_3_2_005_ac1_serde_rejects_titlecase_shared() {
    let result: Result<DtuMode, _> = serde_json::from_str("\"Shared\"");
    assert!(result.is_err(), "DtuMode must reject 'Shared' (title case)");
}

#[test]
fn test_bc_3_2_005_ac1_serde_rejects_titlecase_client() {
    let result: Result<DtuMode, _> = serde_json::from_str("\"Client\"");
    assert!(result.is_err(), "DtuMode must reject 'Client' (title case)");
}

// ---------------------------------------------------------------------------
// AC-2 / BC-3.2.005 invariant 1
// DtuMode is a Copy value type — derives Debug, Clone, Copy, PartialEq, Eq.
// Passes at Red Gate.
// ---------------------------------------------------------------------------

/// AC-2 / VP-091: DtuMode is Copy — value semantics, no interior mutability.
#[test]
fn test_bc_3_2_005_ac2_vp091_dtu_mode_is_copy() {
    let a = DtuMode::Shared;
    let b = a; // copy, not move
    assert_eq!(a, b);

    let c = DtuMode::Client;
    let d = c;
    assert_eq!(c, d);
}

/// AC-2: DtuMode supports equality comparison.
#[test]
fn test_bc_3_2_005_ac2_dtu_mode_equality() {
    assert_eq!(DtuMode::Shared, DtuMode::Shared);
    assert_eq!(DtuMode::Client, DtuMode::Client);
    assert_ne!(DtuMode::Shared, DtuMode::Client);
}

/// AC-2: DtuMode implements Clone.
#[test]
fn test_bc_3_2_005_ac2_dtu_mode_clone() {
    let original = DtuMode::Client;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

/// AC-2: DtuMode implements Debug (format must not panic).
#[test]
fn test_bc_3_2_005_ac2_dtu_mode_debug() {
    let shared_str = format!("{:?}", DtuMode::Shared);
    let client_str = format!("{:?}", DtuMode::Client);
    assert!(!shared_str.is_empty());
    assert!(!client_str.is_empty());
}

// ---------------------------------------------------------------------------
// AC-3 / BC-3.2.005 precondition 3
// DtuRegistryEntry has fields type_name, default_mode, test_only.
// Construction test passes at Red Gate.
// ---------------------------------------------------------------------------

/// AC-3: DtuRegistryEntry can be constructed with all required fields.
#[test]
fn test_bc_3_2_005_ac3_registry_entry_fields() {
    let entry = DtuRegistryEntry {
        type_name: "test-type",
        default_mode: DtuMode::Shared,
        test_only: false,
    };
    assert_eq!(entry.type_name, "test-type");
    assert_eq!(entry.default_mode, DtuMode::Shared);
    assert!(!entry.test_only);
}

// ---------------------------------------------------------------------------
// AC-4 / BC-3.2.005 postcondition 1 — FAILS at Red Gate (registry is empty)
// ---------------------------------------------------------------------------

/// AC-4: DTU_DEFAULT_MODE contains exactly 10 entries (9 production + demo-server).
/// MUST FAIL at Red Gate: registry is currently empty (&[]).
#[test]
fn test_bc_3_2_005_ac4_registry_len_is_10() {
    assert_eq!(
        DTU_DEFAULT_MODE.len(),
        10,
        "DTU_DEFAULT_MODE must contain exactly 10 entries; got {}",
        DTU_DEFAULT_MODE.len()
    );
}

// ---------------------------------------------------------------------------
// AC-5 / BC-3.2.005 postcondition 1 — FAILS at Red Gate
// MSSP Coordination entries: slack, pagerduty, jira, nvd, threatintel
// ---------------------------------------------------------------------------

const MSSP_COORDINATION_TYPES: &[&str] = &["slack", "pagerduty", "jira", "nvd", "threatintel"];

/// AC-5: All 5 MSSP Coordination entries exist with default_mode == Shared and test_only == false.
/// MUST FAIL at Red Gate.
#[test]
fn test_bc_3_2_005_ac5_mssp_coordination_entries_are_shared() {
    for type_name in MSSP_COORDINATION_TYPES {
        let entry = DTU_DEFAULT_MODE
            .iter()
            .find(|e| e.type_name == *type_name)
            .unwrap_or_else(|| panic!("AC-5: missing MSSP Coordination entry '{type_name}'"));

        assert_eq!(
            entry.default_mode,
            DtuMode::Shared,
            "AC-5: '{type_name}' must have default_mode == Shared"
        );
        assert!(
            !entry.test_only,
            "AC-5: '{type_name}' must have test_only == false"
        );
    }
}

/// AC-5: Exactly 5 MSSP Coordination entries are present (no extras, no missing).
#[test]
fn test_bc_3_2_005_ac5_mssp_coordination_count_is_5() {
    let shared_production: Vec<&str> = DTU_DEFAULT_MODE
        .iter()
        .filter(|e| e.default_mode == DtuMode::Shared && !e.test_only)
        .map(|e| e.type_name)
        .collect();

    assert_eq!(
        shared_production.len(),
        5,
        "AC-5: expected exactly 5 production Shared entries, got {}: {:?}",
        shared_production.len(),
        shared_production
    );
}

// ---------------------------------------------------------------------------
// AC-6 / BC-3.2.005 postcondition 2 — FAILS at Red Gate
// Security Telemetry entries: claroty, armis, crowdstrike, cyberint
// ---------------------------------------------------------------------------

const SECURITY_TELEMETRY_TYPES: &[&str] = &["claroty", "armis", "crowdstrike", "cyberint"];

/// AC-6: All 4 Security Telemetry entries exist with default_mode == Client and test_only == false.
/// MUST FAIL at Red Gate.
#[test]
fn test_bc_3_2_005_ac6_security_telemetry_entries_are_client() {
    for type_name in SECURITY_TELEMETRY_TYPES {
        let entry = DTU_DEFAULT_MODE
            .iter()
            .find(|e| e.type_name == *type_name)
            .unwrap_or_else(|| panic!("AC-6: missing Security Telemetry entry '{type_name}'"));

        assert_eq!(
            entry.default_mode,
            DtuMode::Client,
            "AC-6 / VP-093: '{type_name}' (Security Telemetry) must have default_mode == Client"
        );
        assert!(
            !entry.test_only,
            "AC-6: '{type_name}' must have test_only == false"
        );
    }
}

/// AC-6: Exactly 4 production Client entries are present.
#[test]
fn test_bc_3_2_005_ac6_security_telemetry_count_is_4() {
    let client_production: Vec<&str> = DTU_DEFAULT_MODE
        .iter()
        .filter(|e| e.default_mode == DtuMode::Client && !e.test_only)
        .map(|e| e.type_name)
        .collect();

    assert_eq!(
        client_production.len(),
        4,
        "AC-6: expected exactly 4 production Client entries, got {}: {:?}",
        client_production.len(),
        client_production
    );
}

// ---------------------------------------------------------------------------
// AC-7 / BC-3.2.005 invariant 1 + D-051 — FAILS at Red Gate
// demo-server: Client mode, test_only == true
// ---------------------------------------------------------------------------

/// AC-7: demo-server entry exists with default_mode == Client and test_only == true (D-051).
/// MUST FAIL at Red Gate.
#[test]
fn test_bc_3_2_005_ac7_demo_server_is_client_test_only() {
    let entry = DTU_DEFAULT_MODE
        .iter()
        .find(|e| e.type_name == "demo-server")
        .expect("AC-7: 'demo-server' entry must exist in DTU_DEFAULT_MODE");

    assert_eq!(
        entry.default_mode,
        DtuMode::Client,
        "AC-7: 'demo-server' must have default_mode == Client"
    );
    assert!(
        entry.test_only,
        "AC-7 / D-051: 'demo-server' must have test_only == true"
    );
}

// ---------------------------------------------------------------------------
// AC-8 / BC-3.2.005 postcondition 4 + ADR-007 §2.3 — architectural boundary
// No prism-dtu-* crate may export DTU_DEFAULT_MODE or dtu_default_mode().
// Uses grep over workspace source to enforce the boundary.
// ---------------------------------------------------------------------------

/// AC-8 / VP-091: No prism-dtu-* crate defines DTU_DEFAULT_MODE or dtu_default_mode.
/// Classification must live exclusively in prism-core (ADR-007 §2.3).
#[test]
fn test_bc_3_2_005_ac8_vp091_dtu_default_mode_not_in_dtu_crates() {
    // Determine workspace root relative to this manifest's location.
    // CARGO_MANIFEST_DIR is set by cargo during test execution.
    let workspace_root = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo"),
    )
    .parent()
    .expect("prism-core is inside workspace root")
    .parent()
    .expect("crates/ parent is workspace root")
    .to_path_buf();

    let dtu_crates_dir = workspace_root.join("crates");

    // grep -rIn for forbidden symbols in prism-dtu-* source trees only.
    let output = Command::new("grep")
        .args([
            "-rIn",
            "--include=*.rs",
            r"DTU_DEFAULT_MODE\|dtu_default_mode",
        ])
        .arg(&dtu_crates_dir)
        .output()
        .expect("failed to run grep for AC-8 boundary check");

    // Filter matches to only those inside prism-dtu-* directories.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let violations: Vec<&str> = stdout
        .lines()
        .filter(|line| {
            // Accept only hits inside prism-dtu-* crates; exclude prism-core itself.
            let lower = line.to_ascii_lowercase();
            lower.contains("/prism-dtu-") && !lower.contains("/prism-core/")
        })
        .collect();

    assert!(
        violations.is_empty(),
        "AC-8: DTU_DEFAULT_MODE or dtu_default_mode() found in prism-dtu-* crate(s).\n\
         Classification must live exclusively in prism-core (ADR-007 §2.3).\n\
         Violations:\n{}",
        violations.join("\n")
    );
}
