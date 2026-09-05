//! Build script for `prism-bin`.
//!
//! Resolves `PRISM_VERSION` via the ADR-064 D2 v1.5 normative fallback chain:
//!   1. `PRISM_BUILD_VERSION` env var — used only when set AND non-empty (explicit
//!      override for tooling/testing)
//!   2. `GITHUB_REF_NAME` env var — used only when (a) non-empty AND (b) `is_tag_build`
//!      (`GITHUB_REF_TYPE == "tag"`, or fallback: `GITHUB_REF` starts with
//!      `"refs/tags/"`). A single leading `v` is stripped via `strip_prefix`.
//!      On a non-tag run (branch/PR), this arm is skipped entirely.
//!   3. `CARGO_PKG_VERSION` — final fallback; resolves to `1.0.0-dev` on `develop`.
//!
//! **Critical:** `GITHUB_REF_NAME` is set on ALL GitHub Actions runs — branch name
//! on push/pull_request, tag name only on tag-push. Without the `GITHUB_REF_TYPE`
//! gate, non-release CI builds bake `PRISM_VERSION="develop"` into the binary,
//! failing `test_cli_version_output_contains_semver` on all 5 ci.yml legs.
//! (F-VID-P1-CRIT-001, ADR-064 D2 v1.5)
//!
//! The resolved value is emitted as `PRISM_VERSION` and available at compile
//! time via `env!("PRISM_VERSION")` in all `prism-bin` source files.
//!
//! The resolver logic lives in `src/version_resolver.rs` (shared with the
//! `prism_bin` lib target so that `tests/version_identity.rs` exercises the
//! EXACT same function this script calls — HIGH-1, S-REL-VERSION-IDENTITY pass-2).
//!
//! Authority: ADR-064 D2 v1.5, S-REL-BVERSION-INJECT-001 AC-001.

fn main() {
    // Re-run if any relevant env var changes.
    println!("cargo:rerun-if-env-changed=PRISM_BUILD_VERSION");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_TYPE");
    println!("cargo:rerun-if-env-changed=GITHUB_REF");

    let cargo_version = env!("CARGO_PKG_VERSION");

    // GITHUB_REF_NAME is set on ALL GitHub Actions runs — branch name on push/pull_request
    // events, tag name only on tag-push events. Gate on GITHUB_REF_TYPE == "tag" (or GITHUB_REF
    // starts with "refs/tags/") to avoid baking "develop" / "feature/..." into non-release
    // binaries. F-VID-P1-CRIT-001: unconditional use baked PRISM_VERSION="develop" into ci.yml
    // builds, failing test_cli_version_output_contains_semver on all 5 legs.
    //
    // MED-001 (pass-3): derivation extracted to resolve_is_tag_build() in version_resolver.rs
    // so that tests/version_identity.rs exercises the EXACT same logic.
    let ref_type = std::env::var("GITHUB_REF_TYPE").ok();
    let ref_val = std::env::var("GITHUB_REF").ok();
    let is_tag_build = resolve_is_tag_build(ref_type.as_deref(), ref_val.as_deref());

    let build_version = std::env::var("PRISM_BUILD_VERSION").ok();
    let ref_name = std::env::var("GITHUB_REF_NAME").ok();

    let version = resolve_prism_version(
        build_version.as_deref(),
        is_tag_build,
        ref_name.as_deref(),
        cargo_version,
    );

    println!("cargo:rustc-env=PRISM_VERSION={version}");
}

// Shared pure resolvers — include! brings the same source into the build-script
// context that `pub mod version_resolver` brings into the lib target.
// Tests in `tests/version_identity.rs` import resolve_prism_version and
// resolve_is_tag_build from `prism_bin::version_resolver` and exercise the EXACT
// same function bodies called here. (HIGH-1 pass-2, MED-001 pass-3.)
include!("src/version_resolver.rs");
