//! Integration tests for CLI subcommand parsing and dispatch.
//!
//! Tests verify:
//! - `prism version` exits 0 and prints version string (AC-2)
//! - `prism --help` exits 0 and lists all 4 subcommands (AC-1)
//! - `prism validate-config` accepts --config-dir flag and PRISM_CONFIG_DIR env var
//! - `prism query <query>` subcommand exists and dispatches
//! - Unknown subcommand exits 2 (clap default; EC-007)
//!
//! Story: S-WAVE5-PREP-01  AC-1, AC-2
//! BC: BC-2.10.001 (binary entry point contract), BC-2.10.006 (stdio transport scaffold)
//! ADR: ADR-022 §A (CLI surface, exit-code contract)

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::Command;

fn prism_bin() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_prism") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/prism")
}

// ---------------------------------------------------------------------------
// AC-2: prism version exits 0 + prints version string
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-2
/// BC: BC-2.10.006 postcondition — stdio transport scaffold present; binary invocable
/// ADR-022 §A: `version` subcommand → exit 0 + version string
///
/// Tests that `prism version` exits 0 and prints a line containing the semver
/// from Cargo.toml (e.g., "prism 0.1.0" or similar).
///
/// RED GATE: Fails today because `dispatch()` in main.rs is `todo!()`.
#[test]
fn test_cli_version_subcommand_exits_zero() {
    let output = Command::new(prism_bin())
        .args(["version"])
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(0),
        "prism version must exit 0 (AC-2; ADR-022 §A version subcommand); \
         got exit {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Story: S-WAVE5-PREP-01 AC-2
/// BC: BC-2.10.006 postcondition
/// ADR-022 §A: version subcommand must print semver.
///
/// AC-2 exact requirement: `prism X.Y.Z` (semantic version from Cargo.toml).
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_cli_version_output_contains_semver() {
    let expected_version = env!("CARGO_PKG_VERSION");
    let output = Command::new(prism_bin())
        .args(["version"])
        .output()
        .expect("failed to spawn prism binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(expected_version),
        "prism version output must contain semver '{}' (AC-2); \
         got stdout: {}",
        expected_version,
        stdout
    );
}

// ---------------------------------------------------------------------------
// AC-1: prism --help exits 0 + lists all 4 subcommands
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-1
/// BC: BC-2.10.001 — binary entry point; --help lists all subcommands
/// ADR-022 §A: 4 minimum viable subcommands: start, query, validate-config, version
///
/// clap --help returns exit 0; we verify all 4 subcommand names appear.
///
/// Note: clap handles --help natively without touching `dispatch()`, so this
/// test may pass before implementation. Kept because it validates the CLI surface
/// contract (AC-1 is explicitly about --help listing subcommands + exit codes).
#[test]
fn test_cli_help_exits_zero_and_lists_subcommands() {
    let output = Command::new(prism_bin())
        .args(["--help"])
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(0),
        "--help must exit 0 (AC-1; clap standard behavior)"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    // All 4 minimum viable subcommands must appear in --help output (AC-1).
    for subcommand in &["start", "query", "validate-config", "version"] {
        assert!(
            combined.contains(subcommand),
            "--help output must list subcommand '{}' (AC-1; ADR-022 §A minimum viable set); \
             help output: {}",
            subcommand,
            combined
        );
    }
}

/// Story: S-WAVE5-PREP-01 AC-1
/// BC: BC-2.10.001 — help output must document exit codes
/// ADR-022 §A: exit codes 0–5 must be documented in --help output.
///
/// Note: clap handles --help natively IF install_panic_hook() is implemented
/// and does not panic. Today (Red Gate), install_panic_hook() is todo!() and
/// panics before clap parses the args, so --help fails.
///
/// RED GATE: Fails today because install_panic_hook() is todo!().
#[test]
fn test_cli_help_documents_exit_codes() {
    let output = Command::new(prism_bin())
        .args(["--help"])
        .output()
        .expect("failed to spawn prism binary");

    // First assert that --help actually exits 0 (requires panic hook to not panic).
    assert_eq!(
        output.status.code(),
        Some(0),
        "--help must exit 0 (requires install_panic_hook() to be implemented); \
         got exit {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    // After the panic hook is implemented, --help must document the exit code table.
    // ADR-022 §A: "exit codes are documented in the binary's --help output".
    // We check for the "permission-denied" or "config-invalid" strings which come
    // from the long_about exit code documentation.
    assert!(
        combined.contains("config-invalid")
            || combined.contains("permission-denied")
            || combined.contains("internal-error"),
        "--help output must document exit code names (ADR-022 §A; AC-1); \
         got combined help: {combined}"
    );
}

// ---------------------------------------------------------------------------
// CLI: prism start subcommand (dispatches boot sequence)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.10.001 — start subcommand dispatches boot sequence
///
/// `prism start` with valid config will panic at step 7 (todo!() stub).
/// This test verifies that the subcommand reaches dispatch() and invokes
/// the boot sequence (not just returning 0 without doing anything).
///
/// RED GATE: Fails today because `dispatch()` is `todo!()` — it panics
/// at the dispatch level before even reaching step 1.
#[test]
fn test_cli_start_subcommand_reaches_boot_sequence() {
    let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/config/valid");

    let output = Command::new(prism_bin())
        .args(["start"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    // The process must exit non-zero because steps 7-8 are todo!() stubs.
    // If it exits 0, the boot sequence was short-circuited (test design error).
    assert_ne!(
        output.status.code(),
        Some(0),
        "prism start must not exit 0 while boot steps 7-11 are todo!() stubs; \
         the boot sequence must be invoked and fail at step 7"
    );

    // The process must exit with a process code (not signal-killed, i.e., code is Some).
    // todo!() panics produce exit code 101 on Rust by default without a panic hook;
    // with the panic hook installed, they produce exit 1 (AC-12).
    assert!(
        output.status.code().is_some(),
        "prism start must exit with a code (not signal-killed); \
         panic hook must call process::exit(1)"
    );
}

// ---------------------------------------------------------------------------
// CLI: prism query subcommand exists
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.10.001 — query subcommand exists
/// ADR-022 §A: `query <query-string>` subcommand is part of minimum viable set.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_cli_query_subcommand_exists_and_dispatches() {
    let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/config/valid");

    let output = Command::new(prism_bin())
        .args(["query", "SELECT * FROM crowdstrike.detections"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    // query exits 4 (internal-error) because QueryEngine::execute is todo!().
    // It must NOT exit with clap's 2 (unknown subcommand) — the subcommand exists.
    let code = output.status.code().unwrap_or(-1);
    assert_ne!(
        code, 2,
        "prism query must not exit 2 as an 'unknown subcommand'; \
         'query' is part of the minimum viable subcommand set per ADR-022 §A"
    );
}

// ---------------------------------------------------------------------------
// CLI: unknown subcommand → clap exits 2 (EC-007)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.10.001 edge case EC-007 — unknown subcommand → exit 2 (clap default)
///
/// Note: clap handles this natively; dispatch() is not reached.
/// This test validates the EC-007 behavior from the story's edge case table.
#[test]
fn test_cli_unknown_subcommand_exits_two() {
    let output = Command::new(prism_bin())
        .args(["totally-unknown-subcommand-xyz"])
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Unknown subcommand must exit 2 (EC-007; clap default behavior for unknown subcommand)"
    );
}
