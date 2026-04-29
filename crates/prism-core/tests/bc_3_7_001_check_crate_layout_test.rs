//! Conformance tests for BC-3.7.001 — Workspace src/ Convention Lint Enforcement
//!
//! Story: S-3.5.01
//! Verification Properties: VP-134, VP-135, VP-136
//!
//! These tests exercise `scripts/check-crate-layout.sh` via std::process::Command.
//! The script is a bash script and requires a Unix-compatible shell.
//! All tests in this file are skipped on non-Unix platforms (Windows).
//!
//! Red Gate discipline: ALL tests that invoke the script MUST FAIL until the
//! implementer replaces the stub body with the real validation logic.
//! The stub unconditionally exits 1, so:
//!   - VP-134 (all 22 crates pass) FAILS — stub returns exit 1 even for conformant workspace
//!   - VP-136 (script is read-only) FAILS — exits 1
//!   - VP-135 (synthetic bad crate fails) may appear to pass on exit-code alone, but the
//!     output format assertion will fail (stub emits wrong message).
//!
//! Canonical test vectors from BC-3.7.001:
//!   TV-1: conformant crate passes (exit 0)
//!   TV-2: lib.rs at root fails (exit non-zero + violation line)
//!   TV-3: tests/fixtures/ triggers violation (exit non-zero + violation message)
//!   TV-4: prism-ocsf no tests/ passes (exit 0)
//!   TV-5: build.rs at root is permitted (exit 0)

// check-crate-layout.sh is a bash script — not runnable on Windows without WSL.
// Skip the entire file on non-Unix targets to keep Windows CI green.
#![cfg(unix)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the workspace root — two levels above CARGO_MANIFEST_DIR
/// (CARGO_MANIFEST_DIR = crates/prism-core; root = ../../)
fn workspace_root() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo"))
        .parent()
        .expect("crates/prism-core parent is crates/")
        .parent()
        .expect("crates/ parent is workspace root")
        .to_path_buf()
}

/// Full path to the script under test.
fn script_path() -> PathBuf {
    workspace_root()
        .join("scripts")
        .join("check-crate-layout.sh")
}

/// Run the script against the real workspace root (no arguments).
fn run_script_workspace() -> std::process::Output {
    Command::new("bash")
        .arg(script_path())
        .current_dir(workspace_root())
        .output()
        .expect("failed to spawn check-crate-layout.sh")
}

/// Run the script with a synthetic workspace root override.
/// The script is called with WORKSPACE_ROOT env var pointing at the fixture
/// dir so it scans a synthetic tree instead of the real workspace.
fn run_script_on_dir(dir: &std::path::Path) -> std::process::Output {
    Command::new("bash")
        .arg(script_path())
        .env("WORKSPACE_ROOT", dir)
        .current_dir(dir)
        .output()
        .expect("failed to spawn check-crate-layout.sh against fixture dir")
}

/// Create a minimal conformant fake crate at `parent/name/`.
/// Conformant = Cargo.toml + src/lib.rs, no loose .rs at root, no tests/fixtures/.
fn make_conformant_crate(parent: &std::path::Path, name: &str) {
    let crate_dir = parent.join("crates").join(name);
    fs::create_dir_all(crate_dir.join("src")).expect("create src/");
    fs::write(
        crate_dir.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
    )
    .expect("write Cargo.toml");
    fs::write(crate_dir.join("src").join("lib.rs"), "// conformant\n").expect("write src/lib.rs");
}

// ---------------------------------------------------------------------------
// VP-134: check-crate-layout.sh exits 0 for all 22 existing workspace crates
// after prism-spec-engine fixture migration.
//
// MUST FAIL at Red Gate: stub exits 1 unconditionally.
// ---------------------------------------------------------------------------

/// test_BC_3_7_001_vp134_all_existing_crates_pass
///
/// VP-134: running the script against the real 22-crate workspace exits 0
/// with no violation lines.  Traces to BC-3.7.001 postcondition 1 and AC-001.
///
/// MUST FAIL at Red Gate — the stub exits 1.
#[test]
fn test_bc_3_7_001_vp134_all_existing_crates_pass() {
    let output = run_script_workspace();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "VP-134 FAIL: check-crate-layout.sh must exit 0 for the existing 22-crate workspace.\n\
         Exit code: {}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code().unwrap_or(-1),
    );

    // No per-crate violation lines should appear (format: "crates/<name>: <rule>")
    let violations: Vec<&str> = stdout
        .lines()
        .filter(|l| l.starts_with("crates/"))
        .collect();

    assert!(
        violations.is_empty(),
        "VP-134 FAIL: script produced violation lines for conformant workspace:\n{}",
        violations.join("\n")
    );
}

/// test_BC_3_7_001_vp134_workspace_crate_count
///
/// VP-134 supplement: the script must check at least 22 crates (current workspace
/// size).  A summary line like "Checked N crates" is optional but accepted.
/// The primary assertion is the exit-code test above; this test ensures the
/// script doesn't silently pass by checking zero crates.
///
/// MUST FAIL at Red Gate.
#[test]
fn test_bc_3_7_001_vp134_workspace_crate_count() {
    let workspace = workspace_root();
    let crates_dir = workspace.join("crates");

    // Count crate directories in the workspace (each has a Cargo.toml).
    let crate_count = fs::read_dir(&crates_dir)
        .expect("read crates/")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                && e.path().join("Cargo.toml").exists()
        })
        .count();

    assert!(
        crate_count >= 22,
        "Workspace should contain at least 22 crates; found {crate_count}. \
         Test precondition violated — was a crate removed?"
    );

    // The script must succeed (exit 0) on a workspace with this many crates.
    // This re-asserts VP-134 with explicit crate count context.
    let output = run_script_workspace();
    assert!(
        output.status.success(),
        "VP-134 FAIL: script must exit 0 for a {crate_count}-crate workspace. \
         Exit code: {}",
        output.status.code().unwrap_or(-1),
    );
}

// ---------------------------------------------------------------------------
// VP-135: check-crate-layout.sh exits non-zero for a synthetic non-conformant crate.
// Test Vectors TV-2, TV-3, TV-7.
//
// These tests set up a temp workspace with a bad crate and run the script.
// MUST FAIL at Red Gate: the stub exits 1 but the expected output format
// ("crates/<name>: <rule description>") is not emitted — the stub outputs a
// different error message, so the output-format assertions fail.
// ---------------------------------------------------------------------------

/// test_BC_3_7_001_vp135_tv2_lib_rs_at_root_fails
///
/// TV-2: a crate with lib.rs at the crate root (no src/) triggers a violation.
/// BC-3.7.001 postcondition 2 + AC-002 + EC-001.
///
/// The violation output MUST contain the crate path and rule description
/// (postcondition 3 / AC-003).
///
/// MUST FAIL at Red Gate — stub does not produce the required output format.
#[test]
fn test_bc_3_7_001_vp135_tv2_lib_rs_at_root_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let crates_dir = tmp.path().join("crates");
    fs::create_dir_all(&crates_dir).expect("create crates/");

    // Bad crate: Cargo.toml + lib.rs at root, no src/
    let bad = crates_dir.join("test-bad-no-src");
    fs::create_dir_all(&bad).expect("create bad crate dir");
    fs::write(
        bad.join("Cargo.toml"),
        "[package]\nname = \"test-bad-no-src\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(bad.join("lib.rs"), "// bad: loose lib.rs at root\n").expect("write lib.rs");

    let output = run_script_on_dir(tmp.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let all_output = format!("{stdout}{stderr}");

    // Exit code must be non-zero
    assert!(
        !output.status.success(),
        "VP-135 / TV-2 FAIL: script must exit non-zero when a crate has lib.rs at root \
         (no src/).\nstdout: {stdout}\nstderr: {stderr}"
    );

    // Violation line must name the crate path (postcondition 3 / AC-003)
    assert!(
        all_output.contains("test-bad-no-src"),
        "VP-135 / TV-2 FAIL: violation output must identify crate 'test-bad-no-src'.\n\
         Got: {all_output}"
    );

    // Violation line must describe the rule: no src/lib.rs or src/main.rs
    let rule_mentioned = all_output.contains("src/lib.rs")
        || all_output.contains("src/main.rs")
        || all_output.contains("no src/");
    assert!(
        rule_mentioned,
        "VP-135 / TV-2 FAIL: violation output must describe the missing src/lib.rs rule.\n\
         Got: {all_output}"
    );
}

/// test_BC_3_7_001_vp135_tv3_tests_fixtures_triggers_violation
///
/// TV-3: a crate with src/lib.rs but tests/fixtures/ triggers a violation.
/// BC-3.7.001 postcondition 2 + AC-002 + EC-002.
///
/// MUST FAIL at Red Gate — stub does not produce the required output format.
#[test]
fn test_bc_3_7_001_vp135_tv3_tests_fixtures_triggers_violation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let crates_dir = tmp.path().join("crates");

    let bad = crates_dir.join("test-bad-tests-fixtures");
    fs::create_dir_all(bad.join("src")).expect("create src/");
    fs::create_dir_all(bad.join("tests").join("fixtures")).expect("create tests/fixtures/");
    fs::write(
        bad.join("Cargo.toml"),
        "[package]\nname = \"test-bad-tests-fixtures\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(bad.join("src").join("lib.rs"), "// ok src\n").expect("write src/lib.rs");
    fs::write(bad.join("tests").join("fixtures").join("data.json"), "{}")
        .expect("write fixture file");

    let output = run_script_on_dir(tmp.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let all_output = format!("{stdout}{stderr}");

    assert!(
        !output.status.success(),
        "VP-135 / TV-3 FAIL: script must exit non-zero when a crate has tests/fixtures/.\n\
         stdout: {stdout}\nstderr: {stderr}"
    );

    assert!(
        all_output.contains("test-bad-tests-fixtures"),
        "VP-135 / TV-3 FAIL: violation must name crate 'test-bad-tests-fixtures'.\n\
         Got: {all_output}"
    );

    // Rule: "fixtures should be in fixtures/, not tests/fixtures/"
    let rule_mentioned = all_output.contains("tests/fixtures") || all_output.contains("fixtures/");
    assert!(
        rule_mentioned,
        "VP-135 / TV-3 FAIL: violation must mention the fixtures-placement rule.\n\
         Got: {all_output}"
    );
}

/// test_BC_3_7_001_vp135_tv7_loose_rs_at_root_fails
///
/// EC-007: a .rs file at crate root with a name other than build.rs triggers a violation.
/// BC-3.7.001 edge case EC-007.
///
/// MUST FAIL at Red Gate.
#[test]
fn test_bc_3_7_001_vp135_tv7_loose_rs_at_root_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let crates_dir = tmp.path().join("crates");

    let bad = crates_dir.join("test-bad-loose-helpers");
    fs::create_dir_all(bad.join("src")).expect("create src/");
    fs::write(
        bad.join("Cargo.toml"),
        "[package]\nname = \"test-bad-loose-helpers\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(bad.join("src").join("lib.rs"), "// ok\n").expect("write src/lib.rs");
    // Loose helpers.rs at crate root — EC-007 violation
    fs::write(bad.join("helpers.rs"), "// bad: loose .rs\n").expect("write helpers.rs");

    let output = run_script_on_dir(tmp.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let all_output = format!("{stdout}{stderr}");

    assert!(
        !output.status.success(),
        "VP-135 / EC-007 FAIL: script must exit non-zero for loose helpers.rs at crate root.\n\
         stdout: {stdout}\nstderr: {stderr}"
    );

    assert!(
        all_output.contains("helpers.rs") || all_output.contains("test-bad-loose-helpers"),
        "VP-135 / EC-007 FAIL: violation must mention the loose file or crate name.\n\
         Got: {all_output}"
    );
}

// ---------------------------------------------------------------------------
// TV-4 / EC-003: prism-ocsf without tests/ is conformant (exit 0)
// MUST FAIL at Red Gate — stub exits 1.
// ---------------------------------------------------------------------------

/// test_BC_3_7_001_tv4_prism_ocsf_no_tests_passes
///
/// TV-4 / EC-003: a crate with Cargo.toml + src/lib.rs but no tests/ directory
/// must NOT produce a violation.  This validates the prism-ocsf exception:
/// tests/ is optional per ADR-012 §2.1 Rule 1.
///
/// MUST FAIL at Red Gate.
#[test]
fn test_bc_3_7_001_tv4_prism_ocsf_no_tests_passes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let crates_dir = tmp.path().join("crates");

    // Mimic prism-ocsf: Cargo.toml + src/lib.rs, no tests/
    let crate_dir = crates_dir.join("mock-prism-ocsf");
    fs::create_dir_all(crate_dir.join("src")).expect("create src/");
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"mock-prism-ocsf\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(crate_dir.join("src").join("lib.rs"), "// no tests/ dir\n")
        .expect("write src/lib.rs");
    // Intentionally NO tests/ directory

    let output = run_script_on_dir(tmp.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "TV-4 / EC-003 FAIL: crate without tests/ must NOT be flagged as a violation.\n\
         Exit code: {}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code().unwrap_or(-1),
    );

    let violations: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("mock-prism-ocsf"))
        .collect();

    assert!(
        violations.is_empty(),
        "TV-4 / EC-003 FAIL: unexpected violation for 'mock-prism-ocsf' (no tests/ is OK):\n{}",
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------------
// TV-5 / EC-004: build.rs at crate root is permitted (exit 0)
// MUST FAIL at Red Gate — stub exits 1.
// ---------------------------------------------------------------------------

/// test_BC_3_7_001_tv5_build_rs_at_root_permitted
///
/// TV-5 / EC-004: a crate with src/lib.rs AND build.rs at root must NOT
/// trigger a "loose .rs at crate root" violation.  build.rs is the
/// Cargo-mandated build script location (ADR-012 §7 OQ-1).
///
/// MUST FAIL at Red Gate.
#[test]
fn test_bc_3_7_001_tv5_build_rs_at_root_permitted() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let crates_dir = tmp.path().join("crates");

    let crate_dir = crates_dir.join("mock-with-build-rs");
    fs::create_dir_all(crate_dir.join("src")).expect("create src/");
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"mock-with-build-rs\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(crate_dir.join("src").join("lib.rs"), "// ok\n").expect("write src/lib.rs");
    fs::write(crate_dir.join("build.rs"), "fn main() {}\n").expect("write build.rs");

    let output = run_script_on_dir(tmp.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "TV-5 / EC-004 FAIL: build.rs at crate root must NOT be flagged as a violation.\n\
         Exit code: {}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code().unwrap_or(-1),
    );
}

// ---------------------------------------------------------------------------
// VP-136: check-crate-layout.sh is read-only — no files created, modified, or deleted.
//
// MUST FAIL at Red Gate — stub exits 1 (the exit-code assertion fails).
// ---------------------------------------------------------------------------

/// test_BC_3_7_001_vp136_script_is_readonly
///
/// VP-136: running the script must not create, modify, or delete any file in the
/// workspace.  This test compares git status before and after a script run against
/// the real workspace.  BC-3.7.001 invariant 2 + AC-008.
///
/// MUST FAIL at Red Gate — stub exits 1.
#[test]
fn test_bc_3_7_001_vp136_script_is_readonly() {
    let workspace = workspace_root();

    // Capture git status (porcelain) before running the script.
    let before = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&workspace)
        .output()
        .expect("git status before");
    let before_str = String::from_utf8_lossy(&before.stdout).to_string();

    // Run the script (ignore exit code — we only care about filesystem effect).
    let _ = run_script_workspace();

    // Capture git status after.
    let after = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&workspace)
        .output()
        .expect("git status after");
    let after_str = String::from_utf8_lossy(&after.stdout).to_string();

    assert_eq!(
        before_str, after_str,
        "VP-136 FAIL: check-crate-layout.sh modified workspace state.\n\
         Before git status:\n{before_str}\n\
         After git status:\n{after_str}"
    );

    // Also assert the script exits 0 (required by VP-136 / AC-001):
    // read-only + exit-0 are both invariants of a correct implementation.
    let output = run_script_workspace();
    assert!(
        output.status.success(),
        "VP-136 FAIL: script must exit 0 for a conformant workspace (precondition for \
         read-only property to be meaningful).\n\
         Exit code: {}",
        output.status.code().unwrap_or(-1),
    );
}

// ---------------------------------------------------------------------------
// AC-003: violation output format — "crates/<name>: <rule description>"
// MUST FAIL at Red Gate — stub output does not match required format.
// ---------------------------------------------------------------------------

/// test_BC_3_7_001_ac003_violation_output_format
///
/// BC-3.7.001 postcondition 3 / AC-003: the violation line format must be
/// `"crates/<name>: <rule description>"` — both the crate path and violated
/// rule must be present.
///
/// MUST FAIL at Red Gate — stub prints wrong message format.
#[test]
fn test_bc_3_7_001_ac003_violation_output_format() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let crates_dir = tmp.path().join("crates");

    // Create a crate that violates Rule 1 (no src/lib.rs).
    let bad = crates_dir.join("format-check-bad-crate");
    fs::create_dir_all(&bad).expect("create crate dir");
    fs::write(
        bad.join("Cargo.toml"),
        "[package]\nname = \"format-check-bad-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(bad.join("lib.rs"), "// bad\n").expect("write lib.rs");

    let output = run_script_on_dir(tmp.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let all_output = format!("{stdout}{stderr}");

    // Script must exit non-zero
    assert!(
        !output.status.success(),
        "AC-003 FAIL: script must exit non-zero for non-conformant crate.\n\
         stdout: {stdout}\nstderr: {stderr}"
    );

    // The violation line must follow the pattern "crates/<name>: <rule>"
    let has_format_line = all_output
        .lines()
        .any(|l| l.starts_with("crates/") && l.contains(": "));

    assert!(
        has_format_line,
        "AC-003 FAIL: no violation line matching 'crates/<name>: <rule description>' found.\n\
         Output:\n{all_output}"
    );
}

// ---------------------------------------------------------------------------
// TV-1: conformant synthetic crate passes — complementary to VP-134
// Uses a known-good synthetic fixture to isolate the pass behavior.
// MUST FAIL at Red Gate — stub exits 1.
// ---------------------------------------------------------------------------

/// test_BC_3_7_001_tv1_conformant_synthetic_crate_passes
///
/// TV-1: a fully conformant crate (Cargo.toml + src/lib.rs + tests/ + fixtures/)
/// must pass with exit 0 and no violation lines.
/// Traces to BC-3.7.001 postcondition 1 + AC-001.
///
/// MUST FAIL at Red Gate.
#[test]
fn test_bc_3_7_001_tv1_conformant_synthetic_crate_passes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let crates_dir = tmp.path().join("crates");

    let good = crates_dir.join("test-good-crate");
    fs::create_dir_all(good.join("src")).expect("create src/");
    fs::create_dir_all(good.join("tests")).expect("create tests/");
    fs::create_dir_all(good.join("fixtures")).expect("create fixtures/");
    fs::write(
        good.join("Cargo.toml"),
        "[package]\nname = \"test-good-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(good.join("src").join("lib.rs"), "// conformant\n").expect("write src/lib.rs");

    let output = run_script_on_dir(tmp.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "TV-1 FAIL: fully conformant crate must pass (exit 0).\n\
         Exit code: {}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code().unwrap_or(-1),
    );

    let violations: Vec<&str> = stdout
        .lines()
        .filter(|l| l.starts_with("crates/"))
        .collect();

    assert!(
        violations.is_empty(),
        "TV-1 FAIL: unexpected violation lines for conformant crate:\n{}",
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------------
// prism-spec-engine fixture migration check (BC-3.7.001 postcondition 6 / AC-006)
//
// This test checks the CURRENT state of the workspace — before the implementer
// runs the migration.  It asserts that `tests/fixtures/` does NOT exist and
// `fixtures/` DOES exist.
//
// MUST FAIL at Red Gate — prism-spec-engine currently has tests/fixtures/.
// ---------------------------------------------------------------------------

/// test_BC_3_7_001_ac006_prism_spec_engine_fixture_migration
///
/// AC-006 / BC-3.7.001 postcondition 6: after migration, prism-spec-engine must
/// have `fixtures/` at the crate root and NOT have `tests/fixtures/`.
///
/// MUST FAIL at Red Gate — tests/fixtures/ currently exists in the workspace.
#[test]
fn test_bc_3_7_001_ac006_prism_spec_engine_fixture_migration() {
    let workspace = workspace_root();
    let spec_engine = workspace.join("crates").join("prism-spec-engine");

    let old_path = spec_engine.join("tests").join("fixtures");
    let new_path = spec_engine.join("fixtures");

    assert!(
        !old_path.exists(),
        "AC-006 FAIL: crates/prism-spec-engine/tests/fixtures/ must NOT exist after migration. \
         Found: {old_path:?}\n\
         The implementer must run: mv crates/prism-spec-engine/tests/fixtures/ \
         crates/prism-spec-engine/fixtures/"
    );

    assert!(
        new_path.exists(),
        "AC-006 FAIL: crates/prism-spec-engine/fixtures/ must exist after migration. \
         Not found: {new_path:?}"
    );
}

// ---------------------------------------------------------------------------
// Invariant 2 (BC-3.7.001): script enforces rules identically for new crates.
// Test that a brand-new crate introduced AFTER this story is also checked.
// MUST FAIL at Red Gate.
// ---------------------------------------------------------------------------

/// test_BC_3_7_001_invariant2_new_crate_checked
///
/// BC-3.7.001 invariant 1: canonical layout rules are enforced identically for
/// every workspace crate, including newly created ones.
///
/// Approach: place a non-conformant "new" crate alongside one conformant crate
/// in a synthetic workspace; assert the script flags only the bad one.
///
/// MUST FAIL at Red Gate.
#[test]
fn test_bc_3_7_001_invariant2_new_crate_checked() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let crates_dir = tmp.path().join("crates");

    // Good crate: fully conformant
    make_conformant_crate(tmp.path(), "existing-good-crate");

    // Bad new crate: lib.rs at root, no src/
    let bad = crates_dir.join("new-bad-crate");
    fs::create_dir_all(&bad).expect("create bad crate dir");
    fs::write(
        bad.join("Cargo.toml"),
        "[package]\nname = \"new-bad-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(bad.join("lib.rs"), "// bad\n").expect("write lib.rs");

    let output = run_script_on_dir(tmp.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let all_output = format!("{stdout}{stderr}");

    // Script must exit non-zero (the bad crate is caught)
    assert!(
        !output.status.success(),
        "Invariant-1 FAIL: new non-conformant crate must be flagged.\n\
         stdout: {stdout}\nstderr: {stderr}"
    );

    // Only the bad crate must be named in output — not the good one
    let bad_mentioned = all_output.contains("new-bad-crate");
    assert!(
        bad_mentioned,
        "Invariant-1 FAIL: 'new-bad-crate' must appear in violation output.\n\
         Got: {all_output}"
    );

    let good_mentioned = stdout.contains("existing-good-crate");
    assert!(
        !good_mentioned,
        "Invariant-1 FAIL: 'existing-good-crate' must NOT appear as a violation.\n\
         Got stdout: {stdout}"
    );
}
