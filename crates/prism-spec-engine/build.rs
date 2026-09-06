//! Build script for `prism-spec-engine`.
//!
//! Resolves `PRISM_VERSION` via the ADR-064 §D2 normative fallback chain
//! (per ADR-064 D4 §Surface B: "The build.rs algorithm is identical to D2.
//! The D2 normative contract in this ADR is the single authoritative spec."):
//!
//!   1. `PRISM_BUILD_VERSION` env var — used only when set, the first line is
//!      non-empty after trim, AND the first line is semver-shaped (SEC-001/CWE-93
//!      takes the first line only; SEC-002/CWE-20 gate rejects branch names and
//!      other non-version strings).
//!   2. `GITHUB_REF_NAME` env var — used only when (a) non-empty AND (b) `is_tag_build`
//!      (`GITHUB_REF_TYPE == "tag"`, or fallback: `GITHUB_REF` starts with
//!      `"refs/tags/"`). A single leading `v` is stripped via `strip_prefix`.
//!      Post-strip empty guard: degenerate tag `"v"` → stripped `""` → falls through
//!      to CARGO_PKG_VERSION (OBS-1, ADR-064 D2 pass-5).
//!   3. `CARGO_PKG_VERSION` — final fallback; resolves to the prism-spec-engine
//!      library crate version on local dev (no env vars set).
//!
//! **Critical:** `GITHUB_REF_NAME` is set on ALL GitHub Actions runs — branch name
//! on push/pull_request, tag name only on tag-push. Without the `GITHUB_REF_TYPE`
//! gate, non-release CI builds bake `PRISM_VERSION="develop"` into compiled code,
//! causing incorrect user-agent strings. (F-VID-P1-CRIT-001, ADR-064 §D2)
//!
//! The resolver logic lives in `src/version_resolver.rs` (shared with the
//! `prism_spec_engine` lib target via `pub mod version_resolver` so that
//! unit tests in `version_resolver.rs` exercise the EXACT same function bodies
//! called here via `include!`).
//!
//! MED-1 (S-REL-AGENT-VERSION-001 LOCAL pass-1): the previous arrangement had all
//! resolver logic inlined in this file (private functions, structurally untestable).
//! None of the security-critical branches (SEC-001 CWE-93, SEC-002 CWE-20, the
//! is_tag_build gate, prism_build_version_rejected) were exercised by any test.
//! This `include!` arrangement mirrors the ratified prism-bin HIGH-1/MED-001 fix.
//!
//! Authority: ADR-064 D4 §Surface B, S-REL-AGENT-VERSION-001 AC-006.

fn main() {
    // Re-run if the build script itself or the included resolver source changes.
    // Without these, Cargo's implicit dep-info tracking may not detect changes to
    // the include!'d file, causing stale PRISM_VERSION values in incremental builds.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/version_resolver.rs");
    // Re-run if any relevant env var changes (ADR-064 D4 §Surface B — four directives).
    println!("cargo:rerun-if-env-changed=PRISM_BUILD_VERSION");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_TYPE");
    println!("cargo:rerun-if-env-changed=GITHUB_REF");

    let cargo_version = env!("CARGO_PKG_VERSION");

    // GITHUB_REF_NAME is set on ALL GitHub Actions runs — branch name on push/pull_request
    // events, tag name only on tag-push events. Gate on GITHUB_REF_TYPE == "tag" (or GITHUB_REF
    // starts with "refs/tags/") to avoid baking "develop" / "feature/..." into non-release
    // binaries. F-VID-P1-CRIT-001.
    //
    // MED-1 (S-REL-AGENT-VERSION-001): derivation extracted to resolve_is_tag_build() in
    // version_resolver.rs so that the #[cfg(test)] block there exercises the EXACT same logic.
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

    // OBS-1 (ADR-064 D2): emit a build-time diagnostic when PRISM_BUILD_VERSION is
    // non-empty but rejected by the semver-shape gate, so the operator knows their
    // explicit override was ignored and the fallback chain engaged. Does NOT change
    // resolution behavior — silent-rejection-then-fallback is the deliberate SEC-002
    // design (CWE-20); this only adds observability for the manual-override path.
    if let Some(rejected) = prism_build_version_rejected(build_version.as_deref()) {
        println!(
            "cargo:warning=PRISM_BUILD_VERSION override ignored: '{rejected}' is not \
             semver-shaped (X.Y.Z[-pre][+build]). Falling back to GITHUB_REF_NAME / CARGO_PKG_VERSION."
        );
    }

    println!("cargo:rustc-env=PRISM_VERSION={version}");
}

// Shared pure resolvers — include! brings the same source into the build-script
// context that `pub mod version_resolver` brings into the lib target.
// Unit tests in `src/version_resolver.rs` import and exercise resolve_prism_version,
// resolve_is_tag_build, prism_build_version_rejected, and is_semver_shaped against
// the EXACT same function bodies called here. (MED-1 S-REL-AGENT-VERSION-001.)
include!("src/version_resolver.rs");
