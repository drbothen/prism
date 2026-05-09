//! Integration tests for BC-2.06.011 — ConfigManager initialization contract.
//!
//! Tests invoke the `prism` binary as a subprocess using `std::process::Command`.
//! All tests exercise the `validate-config` subcommand which runs boot steps 1–6
//! without entering the MCP serving loop (EC-06-011-006).
//!
//! # Test Vectors from BC-2.06.011
//!
//! | TV ID | Scenario | Expected Exit |
//! |-------|----------|--------------|
//! | TV-06-011-001 | Valid config | Boot continues (exit 0 for validate-config) |
//! | TV-06-011-003 | Config dir missing | Exit 2 + "not found" |
//! | TV-06-011-004 | TOML syntax error | Exit 2 + line number context |
//! | TV-06-011-005 | Missing required field | Exit 2 + field name |
//! | TV-06-011-006 | Empty file | Exit 2 |
//!
//! Story: S-WAVE5-PREP-01
//! BC: BC-2.06.011 (ConfigManager init — prism.toml schema validation at process start)
//! ADR: ADR-022 §A (exit-code contract), §B step 2

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// Return the path to the compiled `prism` binary.
///
/// `cargo nextest` sets CARGO_BIN_EXE_prism for us. Fall back to a best-effort
/// path for `cargo test` invocations.
fn prism_bin() -> PathBuf {
    // nextest / cargo-test set this env var for [[bin]] targets.
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_prism") {
        return PathBuf::from(path);
    }
    // Fallback: walk up from the manifest dir to target/debug/prism.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("../../target/debug/prism")
}

/// Path to a named fixture config directory.
fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/config")
        .join(name)
}

// ---------------------------------------------------------------------------
// BC-2.06.011 — valid config (happy path)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-3
/// BC: BC-2.06.011 Postcondition (Happy path)
/// TV-06-011-001: Valid prism.toml → validate-config exits 0.
///
/// Tests that `prism validate-config` exits 0 when prism.toml is well-formed
/// and all required fields are present.
///
/// RED GATE: Fails today because `dispatch()` in main.rs is `todo!()`.
#[test]
fn test_BC_2_06_011_valid_config_exits_zero() {
    let config_dir = fixture_dir("valid");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(0),
        "validate-config with valid config must exit 0 (BC-2.06.011 happy path); \
         got exit {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.011 — config directory missing (EC-06-011-001 / TV-06-011-003)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-4 (partial)
/// BC: BC-2.06.011 Failure path — $PRISM_CONFIG_DIR set but directory does not exist
/// TV-06-011-003: Missing config dir → exit 2 + "not found" in error output.
///
/// The invariant: PRISM_CONFIG_DIR pointing to a non-existent dir MUST exit 2;
/// the binary MUST NOT fall back to ~/.prism/.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_06_011_missing_config_dir_exits_two() {
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", "/nonexistent-prism-config-dir-abc123")
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Missing PRISM_CONFIG_DIR must exit 2 (BC-2.06.011 invariant); \
         got exit {:?}",
        output.status.code()
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.to_lowercase().contains("not found")
            || combined.to_lowercase().contains("no such file")
            || combined.contains("/nonexistent-prism-config-dir-abc123"),
        "Error output must mention the missing path or 'not found'; \
         BC-2.06.011 EC-06-011-001: 'Config directory not found: {{path}}'; \
         got: {combined}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.011 — TOML syntax error (TV-06-011-004)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-4
/// BC: BC-2.06.011 Failure path — TOML syntax error → exit 2
/// TV-06-011-004: prism.toml with syntax error → exit 2 + parse context.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_06_011_toml_syntax_error_exits_two() {
    let config_dir = fixture_dir("invalid-toml");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "TOML syntax error must exit 2 (BC-2.06.011); \
         got exit {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.011 — missing required field (TV-06-011-005)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 (maps to BC-2.06.011 schema validation failure)
/// BC: BC-2.06.011 Failure path — schema validation failure → exit 2
/// TV-06-011-005: prism.toml missing 'spec_dir' field → exit 2.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_06_011_missing_required_field_exits_two() {
    let config_dir = fixture_dir("missing-fields");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Missing required field in prism.toml must exit 2 (BC-2.06.011); \
         got exit {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.011 — empty prism.toml (EC-06-011-003 / TV-06-011-006)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.06.011 EC-06-011-003: empty prism.toml → exit 2
/// TV-06-011-006: 0-byte prism.toml → exit 2.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_06_011_empty_config_file_exits_two() {
    let config_dir = fixture_dir("empty-toml");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Empty prism.toml must exit 2 (BC-2.06.011 EC-06-011-003); \
         got exit {:?}",
        output.status.code()
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.011 — PRISM_CONFIG_DIR must not fall back to ~/.prism (invariant)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.06.011 Invariant: PRISM_CONFIG_DIR set to non-existent path → exit 2,
/// MUST NOT fall back to default ~/.prism/.
///
/// This test verifies the "no fallback" invariant from BC-2.06.011 §Invariants.
/// We set PRISM_CONFIG_DIR to a unique non-existent path and assert exit is 2,
/// even if ~/.prism/ happens to exist on the test machine.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_06_011_invariant_no_fallback_when_config_dir_env_set() {
    // Use a path that definitely cannot exist.
    let no_such_path = "/tmp/prism-no-such-config-dir-xyzzy-42";
    let _ = std::fs::remove_dir_all(no_such_path); // ensure it doesn't exist

    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", no_such_path)
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "When PRISM_CONFIG_DIR is set to non-existent path, \
         binary MUST exit 2 and MUST NOT fall back to ~/.prism/; \
         BC-2.06.011 invariant 3; got exit {:?}",
        output.status.code()
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.011 — exit code is exactly 2, never 1, 4, or 5 (invariant)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.06.011 Invariant: exit code on config failure is exactly 2.
///
/// Verifies that even a config directory that exists but has an invalid file
/// maps to exit code 2 (not 1 = panic, 4 = internal, 5 = permission).
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_06_011_invariant_exit_code_is_exactly_2_not_other() {
    let config_dir = fixture_dir("invalid-toml");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    let code = output.status.code().unwrap_or(-1);
    assert_ne!(
        code, 1,
        "Config error must not produce exit 1 (panic code); BC-2.06.011 invariant"
    );
    assert_ne!(
        code, 4,
        "Config error must not produce exit 4 (internal-error); BC-2.06.011 invariant"
    );
    assert_ne!(
        code, 5,
        "Config error must not produce exit 5 (permission-denied); BC-2.06.011 invariant"
    );
    assert_eq!(
        code, 2,
        "Config error must produce exit 2 exactly; BC-2.06.011 invariant"
    );
}
