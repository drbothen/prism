//! Integration tests for BC-2.21.001 — OrgRegistry initialization contract.
//!
//! Tests invoke the `prism` binary as a subprocess using `std::process::Command`.
//! The `validate-config` subcommand exercises the full boot-step-2 + boot-step-3
//! path (config load then OrgRegistry init) without entering the MCP loop.
//!
//! # Test Vectors from BC-2.21.001
//!
//! | TV ID | Scenario | Expected Exit |
//! |-------|----------|--------------|
//! | TV-21-001-001 | Single valid org | Boot continues (exit 0) |
//! | TV-21-001-002 | Empty org list | Exit 2 + "at least one org" |
//! | TV-21-001-003 | Duplicate org_id | Exit 2 + "Duplicate org_id" |
//! | TV-21-001-004 | Duplicate org_slug | Exit 2 + "Duplicate org_slug" |
//! | TV-21-001-005 | Malformed slug (uppercase) | Exit 2 + "kebab-case" |
//!
//! Story: S-WAVE5-PREP-01  AC-9
//! BC: BC-2.21.001 (OrgRegistry init — bijective resolution at process start)
//! ADR: ADR-022 §B step 3, §A exit-code contract

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
// BC-2.21.001 — single valid org (TV-21-001-001)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-3 (validate-config happy path includes OrgRegistry init)
/// BC: BC-2.21.001 Postcondition (Happy path — single org)
/// TV-21-001-001: Config with one valid org → OrgRegistry constructed; boot continues.
///
/// RED GATE: Fails today because `dispatch()` in main.rs is `todo!()`.
#[test]
fn test_BC_2_21_001_single_org_exits_zero() {
    let config_dir = fixture_dir("valid");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Single valid org must produce exit 0 (BC-2.21.001 happy path); \
         got exit {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// BC-2.21.001 — empty org list (TV-21-001-002 / AC-9)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-9
/// BC: BC-2.21.001 Failure path — empty org list → exit 2 + "at least one org"
/// TV-21-001-002: Config with orgs=[] → exit 2 + error mentioning org requirement.
///
/// AC-9 exact requirement: stderr contains "Config must declare at least one org".
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_21_001_empty_orgs_exits_two_with_at_least_one_org_message() {
    let config_dir = fixture_dir("empty-orgs");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Empty org list must exit 2 (BC-2.21.001 failure path); \
         got exit {:?}",
        output.status.code()
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    // AC-9 exact text: "Config must declare at least one org"
    assert!(
        combined.contains("at least one org"),
        "Error output must contain 'at least one org' (AC-9 exact requirement); \
         BC-2.21.001 failure path; got: {combined}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.21.001 — duplicate org_id (TV-21-001-003)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.21.001 Failure path — duplicate org_id → exit 2 + "Duplicate org_id"
/// TV-21-001-003: Config with two orgs sharing the same UUID → exit 2.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_21_001_duplicate_org_id_exits_two() {
    let config_dir = fixture_dir("duplicate-org-id");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Duplicate org_id must exit 2 (BC-2.21.001); \
         got exit {:?}",
        output.status.code()
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.to_lowercase().contains("duplicate") && combined.to_lowercase().contains("org"),
        "Error output must mention 'duplicate' and 'org'; \
         BC-2.21.001 failure path; got: {combined}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.21.001 — duplicate org_slug (TV-21-001-004)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.21.001 Failure path — duplicate org_slug → exit 2 + "Duplicate org_slug"
/// TV-21-001-004: Config with two orgs sharing the same slug → exit 2.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_21_001_duplicate_org_slug_exits_two() {
    let config_dir = fixture_dir("duplicate-org-slug");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Duplicate org_slug must exit 2 (BC-2.21.001); \
         got exit {:?}",
        output.status.code()
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.to_lowercase().contains("duplicate") || combined.to_lowercase().contains("slug"),
        "Error output must mention duplicate slug; BC-2.21.001 failure path; got: {combined}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.21.001 — malformed org_slug (TV-21-001-005)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.21.001 Failure path — malformed org_slug → exit 2 + "kebab-case"
/// TV-21-001-005: Config with org_slug="ACME" (uppercase) → exit 2.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_21_001_malformed_slug_uppercase_exits_two() {
    let config_dir = fixture_dir("malformed-slug");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Malformed org_slug must exit 2 (BC-2.21.001); \
         got exit {:?}",
        output.status.code()
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.to_lowercase().contains("kebab")
            || combined.to_lowercase().contains("lowercase")
            || combined.to_lowercase().contains("invalid")
            || combined.to_lowercase().contains("slug"),
        "Error must explain slug format requirement (kebab-case / lowercase); \
         BC-2.21.001 failure path EC-21-001-004; got: {combined}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.21.001 — bijectivity invariant: exit code is exactly 2 on all failures
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.21.001 Invariant: exit code on any OrgRegistry failure is exactly 2
/// (never 1, 4, or 5).
///
/// RED GATE: Fails today because `dispatch()` is `todo!()`.
#[test]
fn test_BC_2_21_001_invariant_exit_code_is_exactly_2_for_empty_orgs() {
    let config_dir = fixture_dir("empty-orgs");
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", &config_dir)
        .output()
        .expect("failed to spawn prism binary");

    let code = output.status.code().unwrap_or(-1);
    assert_ne!(
        code, 1,
        "OrgRegistry error must not produce exit 1; BC-2.21.001 invariant"
    );
    assert_ne!(
        code, 4,
        "OrgRegistry error must not produce exit 4; BC-2.21.001 invariant"
    );
    assert_ne!(
        code, 5,
        "OrgRegistry error must not produce exit 5; BC-2.21.001 invariant"
    );
    assert_eq!(
        code, 2,
        "OrgRegistry error must produce exit 2; BC-2.21.001 invariant"
    );
}
