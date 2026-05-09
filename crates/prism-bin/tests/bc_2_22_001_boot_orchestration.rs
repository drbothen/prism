//! Integration tests for BC-2.22.001 — Boot orchestration contract.
//!
//! Tests cover:
//! 1. Step sequencing: failure at step N blocks steps N+1..9
//! 2. Exit-code map: each failure class maps to the canonical exit code
//! 3. Pre-traffic gate: MCP server does NOT start before step 8 completes
//!
//! # Sequencing Invariant (BC-2.22.001)
//!
//! ```text
//! Step 2: BC-2.06.011 (ConfigManager init)
//!            ↓ blocks
//! Step 3: BC-2.21.001 (OrgRegistry init)
//!            ↓ blocks
//! Step 5: BC-2.03.013 (CredentialStore init)
//!            ↓ blocks
//! Step 6: BC-2.05.012 (AuditEmitter init)
//!            ↓ blocks
//! Steps 7–8: (storage + QueryEngine) → blocked by todo!() stubs
//!            ↓ blocks
//! Step 9: MCP server stdio bind — TRAFFIC GATE OPEN
//! ```
//!
//! # Test Vectors from BC-2.22.001
//!
//! | TV ID | Injected Failure | Expected Exit |
//! |-------|-----------------|--------------|
//! | TV-22-001-002 | Config missing | 2 |
//! | TV-22-001-003 | Duplicate slug | 2 |
//! | TV-22-001-004 | Cred permission denied | 5 |
//! | TV-22-001-005 | Cred ref unresolvable | 2 |
//! | TV-22-001-006 | Audit CF open fails | 4 |
//! | TV-22-001-008 | Traffic gate: no MCP before step 8 | No MCP output |
//!
//! Story: S-WAVE5-PREP-01  AC-5, AC-7, AC-8, AC-9, AC-10
//! BC: BC-2.22.001 (Boot orchestration — sequencing, exit-code map, traffic gate)
//! ADR: ADR-022 §A, §B

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::Command;

fn prism_bin() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_prism") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/prism")
}

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/config")
        .join(name)
}

// ---------------------------------------------------------------------------
// BC-2.22.001 — exit-code map: config failure → exit 2
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.22.001 Exit-code map — ConfigManager fails → exit 2
/// TV-22-001-002: Config missing → exit 2; steps 3–9 never begin.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_22_001_exit_code_config_error() {
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", "/nonexistent-config-dir-xyzzy")
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Config missing must map to exit 2 (BC-2.22.001 exit-code table); \
         got exit {:?}",
        output.status.code()
    );
}

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.22.001 Exit-code map — OrgRegistry fails → exit 2
/// TV-22-001-003: Duplicate org_slug → exit 2; steps 4–9 never begin.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_22_001_exit_code_org_registry_error() {
    let config_dir = fixture_dir("duplicate-org-slug");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "OrgRegistry failure (duplicate slug) must map to exit 2 \
         (BC-2.22.001 exit-code table); got exit {:?}",
        output.status.code()
    );
}

/// Story: S-WAVE5-PREP-01 AC-7
/// BC: BC-2.22.001 Exit-code map — CredentialStore permission-denied → exit 5
/// TV-22-001-004: Keyring locked → exit 5; steps 6–9 never begin.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_22_001_exit_code_cred_permission_denied() {
    let config_dir = fixture_dir("valid");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .env("PRISM_TEST_INJECT_FAIL_STEP", "5_permission")
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(5),
        "Credential permission-denied must map to exit 5 \
         (BC-2.22.001 exit-code table + AC-7); got exit {:?}",
        output.status.code()
    );
}

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.22.001 Exit-code map — CredentialStore config-invalid ref → exit 2
/// TV-22-001-005: Missing credential ref → exit 2.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_22_001_exit_code_cred_invalid_ref() {
    let config_dir = fixture_dir("valid");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .env("PRISM_TEST_INJECT_FAIL_STEP", "5_missing_ref")
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Credential config-invalid ref must map to exit 2 \
         (BC-2.22.001 exit-code table); got exit {:?}",
        output.status.code()
    );
}

/// Story: S-WAVE5-PREP-01 AC-8
/// BC: BC-2.22.001 Exit-code map — AuditEmitter fails → exit 4
/// TV-22-001-006: Audit CF open fails → exit 4; steps 7–9 never begin.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_22_001_exit_code_audit_failure() {
    let config_dir = fixture_dir("valid");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .env("PRISM_TEST_INJECT_FAIL_STEP", "6_audit_failure")
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(4),
        "AuditEmitter failure must map to exit 4 \
         (BC-2.22.001 exit-code table + AC-8); got exit {:?}",
        output.status.code()
    );
}

// ---------------------------------------------------------------------------
// BC-2.22.001 — step 2 failure blocks step 3 from beginning
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.22.001 §Sequencing Invariant — step 2 failure blocks steps 3–9
/// EC-22-001-001: Config fails → no OrgRegistry log line in output.
///
/// Tests that when config load fails, the OrgRegistry log line
/// ("OrgRegistry initialized") never appears in output.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_22_001_step2_failure_blocks_step3() {
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", "/nonexistent-config-dir-xyzzy")
        .output()
        .expect("failed to spawn prism binary");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // If step 2 fails, step 3 must never have been attempted.
    // We check that the OrgRegistry success log does NOT appear.
    assert!(
        !combined.contains("OrgRegistry initialized"),
        "If config load fails, OrgRegistry must never be initialized; \
         BC-2.22.001 sequencing invariant (step 2 failure blocks step 3); \
         got output: {combined}"
    );

    assert_eq!(
        output.status.code(),
        Some(2),
        "Config failure must exit 2 (not a later-step code)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.22.001 — pre-traffic gate: no MCP output before step 8 (AC-10)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-10
/// BC: BC-2.22.001 §Pre-Traffic Gate Invariant
/// TV-22-001-008: Steps 7-8 are todo!() stubs — the binary must NOT emit
/// MCP JSON-RPC on stdout before step 8 completes.
///
/// Since steps 7 and 8 are currently todo!() stubs, running `prism start`
/// must panic (exit non-zero) before reaching step 9 (MCP stdio bind).
/// No MCP protocol bytes (JSON-RPC lines) should appear on stdout.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()` — but the
/// assertion we want verified is: if the binary somehow doesn't panic at
/// steps 7/8, it still must not emit MCP traffic before those complete.
#[test]
fn test_BC_2_22_001_no_mcp_output_before_step8_completes() {
    // Run prism start with a valid config. It will panic at step 7 (todo!()).
    // Assert:
    // 1. The process exits non-zero (steps 7-8 are stubs)
    // 2. Stdout does NOT begin with MCP JSON-RPC handshake bytes
    let config_dir = fixture_dir("valid");

    // Use a timeout via std::process: give the process 5 seconds max.
    // Steps 7/8 are todo!() so it will exit quickly.
    let output = Command::new(prism_bin())
        .args(["start"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        // Ensure steps 1-6 complete (valid config, no injected failures).
        // Steps 7-8 will panic on todo!().
        .output()
        .expect("failed to spawn prism binary");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // The MCP JSON-RPC initialized notification that would appear if the
    // server accidentally started: '{"jsonrpc":"2.0","method":"notifications/initialized"}'.
    // BC-2.22.001 §Pre-Traffic Gate: this must NOT appear before step 8 completes.
    assert!(
        !stdout.contains("\"jsonrpc\""),
        "MCP JSON-RPC bytes must not appear on stdout before step 8 completes; \
         BC-2.22.001 pre-traffic gate invariant (AC-10); \
         stdout: {stdout}"
    );

    // The process must NOT exit 0 (steps 7-8 are stubs that todo!()-panic).
    assert_ne!(
        output.status.code(),
        Some(0),
        "prism start must not exit 0 while steps 7-11 are todo!() stubs; \
         BC-2.22.001 (steps 7-8 stubs must remain unfilled)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.22.001 — canonical exit-code table: BootError variants
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.22.001 Exit-code map — unit test of the BootError::exit_code() method.
///
/// Verifies the complete canonical table from ADR-022 §A via the BootError type.
/// This test exercises already-implemented code (exit_codes.rs + boot.rs BootError).
#[test]
fn test_BC_2_22_001_exit_code_map_all_variants() {
    use prism_bin::BootError;

    // Config-invalid class → exit 2
    assert_eq!(
        BootError::ConfigInvalid("test".into()).exit_code(),
        2,
        "ConfigInvalid must map to exit 2"
    );
    assert_eq!(
        BootError::OrgRegistryFailed("test".into()).exit_code(),
        2,
        "OrgRegistryFailed must map to exit 2"
    );
    assert_eq!(
        BootError::CredentialRefInvalid("test".into()).exit_code(),
        2,
        "CredentialRefInvalid must map to exit 2"
    );

    // Permission-denied → exit 5
    assert_eq!(
        BootError::CredentialPermissionDenied("test".into()).exit_code(),
        5,
        "CredentialPermissionDenied must map to exit 5"
    );

    // Internal-error → exit 4
    assert_eq!(
        BootError::AuditInitFailed("test".into()).exit_code(),
        4,
        "AuditInitFailed must map to exit 4"
    );
    assert_eq!(
        BootError::InternalError("test".into()).exit_code(),
        4,
        "InternalError must map to exit 4"
    );

    // Sensor-fail → exit 3
    assert_eq!(
        BootError::SensorFail("test".into()).exit_code(),
        3,
        "SensorFail must map to exit 3"
    );
}
