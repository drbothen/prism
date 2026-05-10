//! Unit tests for the exit-code constants and BootError::exit_code() mapping.
//!
//! These tests exercise the already-implemented `exit_codes.rs` and
//! `BootError::exit_code()` method. They are NOT Red Gate tests (they pass
//! today), but are included because:
//! 1. They document the ADR-022 §A canonical contract in test form.
//! 2. They will fail if an implementer incorrectly changes the constants.
//! 3. They satisfy the BC-2.22.001 exit-code map traceability requirement.
//!
//! Story: S-WAVE5-PREP-01
//! BC: BC-2.22.001 (exit-code map), BC-2.06.011, BC-2.21.001, BC-2.03.013, BC-2.05.012
//! ADR: ADR-022 §A (canonical exit-code contract)

#![allow(clippy::unwrap_used)]

use prism_bin::{
    BootError, EXIT_CONFIG_INVALID, EXIT_GENERIC_ERROR, EXIT_INTERNAL_ERROR,
    EXIT_PERMISSION_DENIED, EXIT_SENSOR_FAIL, EXIT_SUCCESS,
};

// ---------------------------------------------------------------------------
// Exit-code constant values (ADR-022 §A)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.22.001 exit-code map — canonical constant values per ADR-022 §A
///
/// These constants are the contract surface between prism-bin and any shell
/// wrapper or integration test. Their values MUST match ADR-022 §A exactly.
#[test]
fn test_exit_code_constants_match_adr_022_canonical_table() {
    // ADR-022 §A: 0 = success / clean shutdown
    assert_eq!(EXIT_SUCCESS, 0, "EXIT_SUCCESS must be 0 per ADR-022 §A");
    // ADR-022 §A: 1 = unhandled error (panic)
    assert_eq!(
        EXIT_GENERIC_ERROR, 1,
        "EXIT_GENERIC_ERROR must be 1 per ADR-022 §A"
    );
    // ADR-022 §A: 2 = config-invalid
    assert_eq!(
        EXIT_CONFIG_INVALID, 2,
        "EXIT_CONFIG_INVALID must be 2 per ADR-022 §A"
    );
    // ADR-022 §A: 3 = sensor-fail
    assert_eq!(
        EXIT_SENSOR_FAIL, 3,
        "EXIT_SENSOR_FAIL must be 3 per ADR-022 §A"
    );
    // ADR-022 §A: 4 = internal-error
    assert_eq!(
        EXIT_INTERNAL_ERROR, 4,
        "EXIT_INTERNAL_ERROR must be 4 per ADR-022 §A"
    );
    // ADR-022 §A: 5 = permission-denied
    assert_eq!(
        EXIT_PERMISSION_DENIED, 5,
        "EXIT_PERMISSION_DENIED must be 5 per ADR-022 §A"
    );
}

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.22.001 exit-code map — no exit codes outside 0–5 are defined
///
/// ADR-022 §A: "Exit codes 0–5 are the canonical contract — no other exit
/// codes may be added without ADR."
#[test]
fn test_exit_code_constants_are_within_zero_to_five_range() {
    let all_codes = [
        EXIT_SUCCESS,
        EXIT_GENERIC_ERROR,
        EXIT_CONFIG_INVALID,
        EXIT_SENSOR_FAIL,
        EXIT_INTERNAL_ERROR,
        EXIT_PERMISSION_DENIED,
    ];
    for &code in &all_codes {
        assert!(
            (0..=5).contains(&code),
            "All exit codes must be in range 0–5 per ADR-022 §A; \
             found code {code} outside range"
        );
    }
}

// ---------------------------------------------------------------------------
// BootError::exit_code() — complete mapping
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.06.011 — ConfigInvalid → exit 2
#[test]
fn test_boot_error_config_invalid_maps_to_exit_2() {
    let err = BootError::ConfigInvalid("prism.toml missing spec_dir".to_string());
    assert_eq!(
        err.exit_code(),
        EXIT_CONFIG_INVALID,
        "BootError::ConfigInvalid must map to exit 2 (BC-2.06.011)"
    );
}

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.21.001 — OrgRegistryFailed → exit 2
#[test]
fn test_boot_error_org_registry_failed_maps_to_exit_2() {
    let err = BootError::OrgRegistryFailed("Config must declare at least one org".to_string());
    assert_eq!(
        err.exit_code(),
        EXIT_CONFIG_INVALID,
        "BootError::OrgRegistryFailed must map to exit 2 (BC-2.21.001 + BC-2.22.001)"
    );
}

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.03.013 — CredentialRefInvalid → exit 2 (not exit 5)
#[test]
fn test_boot_error_credential_ref_invalid_maps_to_exit_2() {
    let err = BootError::CredentialRefInvalid("ref crowdstrike.api_key not found".to_string());
    assert_eq!(
        err.exit_code(),
        EXIT_CONFIG_INVALID,
        "BootError::CredentialRefInvalid must map to exit 2 (BC-2.03.013 config-invalid path)"
    );
}

/// Story: S-WAVE5-PREP-01 AC-7
/// BC: BC-2.03.013 — CredentialPermissionDenied → exit 5 (not exit 2 or 4)
#[test]
fn test_boot_error_credential_permission_denied_maps_to_exit_5() {
    let err = BootError::CredentialPermissionDenied("keyring locked".to_string());
    assert_eq!(
        err.exit_code(),
        EXIT_PERMISSION_DENIED,
        "BootError::CredentialPermissionDenied must map to exit 5 (BC-2.03.013 + AC-7)"
    );
}

/// Story: S-WAVE5-PREP-01 AC-8
/// BC: BC-2.05.012 — AuditInitFailed → exit 4 (not exit 2 or 5)
#[test]
fn test_boot_error_audit_init_failed_maps_to_exit_4() {
    let err = BootError::AuditInitFailed("RocksDB audit_buffer CF open failed".to_string());
    assert_eq!(
        err.exit_code(),
        EXIT_INTERNAL_ERROR,
        "BootError::AuditInitFailed must map to exit 4 (BC-2.05.012 + AC-8)"
    );
}

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.22.001 — InternalError → exit 4
#[test]
fn test_boot_error_internal_error_maps_to_exit_4() {
    let err = BootError::InternalError("QueryEngine construction failed".to_string());
    assert_eq!(
        err.exit_code(),
        EXIT_INTERNAL_ERROR,
        "BootError::InternalError must map to exit 4 (BC-2.22.001)"
    );
}

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.22.001 — SensorFail → exit 3
#[test]
fn test_boot_error_sensor_fail_maps_to_exit_3() {
    let err = BootError::SensorFail("required sensor adapter crowdstrike failed".to_string());
    assert_eq!(
        err.exit_code(),
        EXIT_SENSOR_FAIL,
        "BootError::SensorFail must map to exit 3 (ADR-022 §A sensor-fail)"
    );
}

// ---------------------------------------------------------------------------
// BootError display strings — traceability
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.22.001 — BootError Display includes variant name prefix
///
/// Tests that the Display impl includes a recognizable variant prefix, enabling
/// log grepping and audit trail correlation.
#[test]
fn test_boot_error_display_includes_variant_prefix() {
    let cases: &[(BootError, &str)] = &[
        (BootError::ConfigInvalid("x".into()), "config-invalid"),
        (
            BootError::OrgRegistryFailed("x".into()),
            "org-registry-failed",
        ),
        (
            BootError::CredentialRefInvalid("x".into()),
            "credential-ref-invalid",
        ),
        (
            BootError::CredentialPermissionDenied("x".into()),
            "credential-permission-denied",
        ),
        (BootError::AuditInitFailed("x".into()), "audit-init-failed"),
        (BootError::InternalError("x".into()), "internal-error"),
        (BootError::SensorFail("x".into()), "sensor-fail"),
    ];

    for (err, expected_prefix) in cases {
        let msg = err.to_string();
        assert!(
            msg.contains(expected_prefix),
            "BootError::{}  display must contain prefix '{}'; got: {}",
            expected_prefix,
            expected_prefix,
            msg
        );
    }
}
