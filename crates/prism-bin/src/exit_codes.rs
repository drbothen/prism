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
