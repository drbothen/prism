//! OCSF schema version — compile-time constant.
//!
//! BC-2.02.009: The OCSF schema version is pinned at compile time and immutable at runtime.
//! `ocsf_version()` reads the version string that `build.rs` wrote to `OUT_DIR/ocsf_version.txt`
//! via `include_str!()`. If the file is absent the build fails at compile time — the error
//! is caught at compile time, not runtime (BC-2.02.009 edge case: `ocsf_version()` called
//! when OUT_DIR file is missing → compile error via `include_str!()`).

/// The OCSF schema version pinned at compile time by `build.rs`.
///
/// Format: semver string, e.g. `"1.7.0"`.
///
/// BC-2.02.009 postcondition: this value matches the version passed to `ocsf-proto-gen`
/// in `build.rs` and is exposed in MCP resource metadata as `ocsf_version`.
static OCSF_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/ocsf_version.txt"));

/// Returns the OCSF schema version pinned at compile time.
///
/// # Contract
///
/// - Returns a non-empty semver string. (AC-1, BC-2.02.009)
/// - Value is immutable for the lifetime of the process. (BC-2.02.009 invariant)
/// - Value matches the version passed to `ocsf-proto-gen` in `build.rs`.
pub fn ocsf_version() -> &'static str {
    OCSF_VERSION
}
