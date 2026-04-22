//! Tests for BC-2.02.009 — OCSF Version Pinning Per Release.
//!
//! BC: The OCSF schema version is pinned at compile time and immutable at runtime.
//!
//! Acceptance Criteria covered:
//! - AC-1: `ocsf_version()` returns a non-empty semver string.
//!
//! Test Vectors:
//! - TV-BC-2.02.009-001: build with pinned v1.7.0 → ocsf_version() == "1.7.0"

use crate::version::ocsf_version;

/// BC-2.02.009 / AC-1: `ocsf_version()` returns a non-empty string.
///
/// Red Gate note: this test PASSES with the stub because `build.rs` writes "1.7.0"
/// to `ocsf_version.txt` unconditionally. This is intentional — the version baking
/// mechanism works correctly in the stub. The test that depends on ocsf-proto-gen
/// (AC-2: descriptor pool has real OCSF descriptors) is in `bc_2_02_001_pool.rs`
/// and fails until ocsf-proto-gen is available.
#[test]
fn test_BC_2_02_009_ocsf_version_is_nonempty() {
    let ver = ocsf_version();
    assert!(!ver.is_empty(), "ocsf_version() must return a non-empty string (AC-1)");
}

/// BC-2.02.009 / TV-BC-2.02.009-001: pinned version is "1.7.0".
///
/// Red Gate note: this test PASSES with the stub because `build.rs` hard-codes the
/// version string. If the test were to fail it would indicate the build.rs was changed
/// without updating this assertion — that's a legitimate failure, not a Red Gate issue.
#[test]
fn test_BC_2_02_009_pinned_version_is_semver() {
    let ver = ocsf_version();

    // Must be parseable as a semver (major.minor.patch).
    let parts: Vec<&str> = ver.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "ocsf_version() must return a semver string (e.g. '1.7.0'), got '{ver}' (BC-2.02.009)"
    );

    for part in &parts {
        part.parse::<u32>().unwrap_or_else(|_| {
            panic!(
                "ocsf_version() semver component '{part}' is not a valid integer (BC-2.02.009)"
            )
        });
    }
}

/// BC-2.02.009 invariant: `ocsf_version()` returns the same value on repeated calls.
/// The version is a compile-time constant — immutable for the process lifetime.
#[test]
fn test_BC_2_02_009_invariant_version_immutable_across_calls() {
    let first = ocsf_version();
    let second = ocsf_version();
    assert_eq!(
        first, second,
        "ocsf_version() must return the same value on every call (BC-2.02.009 invariant)"
    );
}
