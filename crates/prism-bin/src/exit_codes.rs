//! Exit-code constants for the `prism` binary.
//!
//! These are the canonical exit codes defined in ADR-022 §A.  They are the
//! contract surface between `prism-bin` and any shell wrapper or integration
//! test.  No exit code outside this list may be returned by the binary without
//! a new ADR.
//!
//! The constants are documented in `--help` output via `cli.rs` `long_about`.

/// Clean shutdown / successful operation.
///
/// ADR-022 §A exit-code contract: 0 = success / clean shutdown.
pub const EXIT_SUCCESS: i32 = 0;

/// Unhandled error (generic; includes unexpected panics caught by panic hook).
///
/// ADR-022 §A exit-code contract: 1 = unhandled error.
/// Exit code 1 is emitted ONLY by the panic hook — never deliberately by boot steps.
pub const EXIT_GENERIC_ERROR: i32 = 1;

/// Config-invalid (TOML parse error, schema validation failure, credential ref
/// resolution failure, OrgRegistry construction failure).
///
/// ADR-022 §A exit-code contract: 2 = config-invalid.
/// Anchors: BC-2.06.011, BC-2.21.001, BC-2.03.013 (ref-unresolvable path).
pub const EXIT_CONFIG_INVALID: i32 = 2;

/// Sensor-fail (a required sensor adapter failed to initialize at boot;
/// non-required adapters degrade to a warning).
///
/// ADR-022 §A exit-code contract: 3 = sensor-fail.
pub const EXIT_SENSOR_FAIL: i32 = 3;

/// Internal-error (runtime invariant violation; QueryEngine init failed;
/// RocksDB open failed; AuditEmitter init failed).
///
/// ADR-022 §A exit-code contract: 4 = internal-error.
/// Anchors: BC-2.05.012 (audit subsystem), BC-2.22.001 (steps 7/8 failure).
pub const EXIT_INTERNAL_ERROR: i32 = 4;

/// Permission-denied (credential store access denied at boot).
///
/// ADR-022 §A exit-code contract: 5 = permission-denied.
/// Anchors: BC-2.03.013 (CredentialStore permission-denied path).
pub const EXIT_PERMISSION_DENIED: i32 = 5;

// ---------------------------------------------------------------------------
// Inline unit tests — exit-code constant values
//
// These tests document and enforce the ADR-022 §A canonical table.
// They are NOT Red Gate tests (the constants are already implemented).
// They will fail if the constants are incorrectly changed.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Story: S-WAVE5-PREP-01
    /// BC: BC-2.22.001 exit-code map — constant correctness per ADR-022 §A
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_22_001_exit_code_constants_match_adr_022() {
        assert_eq!(EXIT_SUCCESS, 0, "EXIT_SUCCESS must be 0 (ADR-022 §A)");
        assert_eq!(
            EXIT_GENERIC_ERROR, 1,
            "EXIT_GENERIC_ERROR must be 1 (ADR-022 §A)"
        );
        assert_eq!(
            EXIT_CONFIG_INVALID, 2,
            "EXIT_CONFIG_INVALID must be 2 (ADR-022 §A)"
        );
        assert_eq!(
            EXIT_SENSOR_FAIL, 3,
            "EXIT_SENSOR_FAIL must be 3 (ADR-022 §A)"
        );
        assert_eq!(
            EXIT_INTERNAL_ERROR, 4,
            "EXIT_INTERNAL_ERROR must be 4 (ADR-022 §A)"
        );
        assert_eq!(
            EXIT_PERMISSION_DENIED, 5,
            "EXIT_PERMISSION_DENIED must be 5 (ADR-022 §A)"
        );
    }

    /// Story: S-WAVE5-PREP-01
    /// BC: BC-2.22.001 — exit codes are distinct (no two errors share a code)
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_22_001_exit_code_constants_are_distinct() {
        let all = [
            EXIT_SUCCESS,
            EXIT_GENERIC_ERROR,
            EXIT_CONFIG_INVALID,
            EXIT_SENSOR_FAIL,
            EXIT_INTERNAL_ERROR,
            EXIT_PERMISSION_DENIED,
        ];
        // All codes must be unique.
        for (i, &a) in all.iter().enumerate() {
            for (j, &b) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        a, b,
                        "Exit codes at index {i} and {j} must be distinct; \
                         ADR-022 §A: each failure class has a unique code"
                    );
                }
            }
        }
    }

    /// Story: S-WAVE5-PREP-01
    /// BC: BC-2.06.011 — config-invalid exit code is 2 (not 1, 4, or 5)
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_06_011_config_invalid_exit_code_is_2() {
        assert_eq!(EXIT_CONFIG_INVALID, 2);
        assert_ne!(EXIT_CONFIG_INVALID, EXIT_GENERIC_ERROR);
        assert_ne!(EXIT_CONFIG_INVALID, EXIT_INTERNAL_ERROR);
        assert_ne!(EXIT_CONFIG_INVALID, EXIT_PERMISSION_DENIED);
    }

    /// Story: S-WAVE5-PREP-01  AC-7
    /// BC: BC-2.03.013 — permission-denied exit code is 5 (not 2 or 4)
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_03_013_permission_denied_exit_code_is_5() {
        assert_eq!(EXIT_PERMISSION_DENIED, 5);
        assert_ne!(EXIT_PERMISSION_DENIED, EXIT_CONFIG_INVALID);
        assert_ne!(EXIT_PERMISSION_DENIED, EXIT_INTERNAL_ERROR);
    }

    /// Story: S-WAVE5-PREP-01  AC-8
    /// BC: BC-2.05.012 — internal-error exit code is 4 (not 2 or 5)
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_05_012_internal_error_exit_code_is_4() {
        assert_eq!(EXIT_INTERNAL_ERROR, 4);
        assert_ne!(EXIT_INTERNAL_ERROR, EXIT_CONFIG_INVALID);
        assert_ne!(EXIT_INTERNAL_ERROR, EXIT_PERMISSION_DENIED);
    }
}
