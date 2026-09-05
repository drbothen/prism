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
    let is_tag_build = std::env::var("GITHUB_REF_TYPE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|t| t.trim() == "tag")
        .unwrap_or_else(|| {
            // Fallback for environments that provide GITHUB_REF but not GITHUB_REF_TYPE.
            std::env::var("GITHUB_REF")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .map(|r| r.starts_with("refs/tags/"))
                .unwrap_or(false)
        });

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

/// ADR-064 D2 v1.5 normative fallback chain (pure function — testable).
///
/// 1. `build_version` wins if present and non-empty/non-whitespace (F-VID-P1-MED-001).
/// 2. On `is_tag_build=true`, `ref_name` with a SINGLE leading `v` stripped wins if
///    present and non-empty. `strip_prefix('v')` removes exactly one `v`; the rejected
///    `trim_start_matches` would strip ALL leading v's — e.g. `vv1.0.0` would become
///    `1.0.0` instead of the correct `v1.0.0` (F-VID-P1-LOW-001).
/// 3. `cargo_version` is the final fallback (resolves to `1.0.0-dev` on develop).
///
/// All empty-string / whitespace-only values are treated as absent (F-VID-P1-MED-001).
///
/// This function is invoked by `main()` with real environment variable values (POL-42).
fn resolve_prism_version(
    build_version: Option<&str>,
    is_tag_build: bool,
    ref_name: Option<&str>,
    cargo_version: &str,
) -> String {
    // Step 1: PRISM_BUILD_VERSION explicit override.
    if let Some(v) = build_version.filter(|s| !s.trim().is_empty()) {
        return v.to_string();
    }
    // Step 2: GITHUB_REF_NAME — only on tag builds.
    // strip_prefix removes exactly ONE leading 'v'; trim_start_matches('v') would strip all
    // leading v's (e.g. "vv1.0.0" → "1.0.0" instead of "v1.0.0"). F-VID-P1-LOW-001.
    if is_tag_build && let Some(r) = ref_name.filter(|s| !s.trim().is_empty()) {
        let name = r.trim();
        return name.strip_prefix('v').unwrap_or(name).to_string();
    }
    // Step 3: CARGO_PKG_VERSION fallback.
    cargo_version.to_string()
}

#[cfg(test)]
mod tests {
    use super::resolve_prism_version;

    // --- Local-dev path ---

    #[test]
    fn fallback_to_cargo_pkg_version_when_no_env() {
        assert_eq!(
            resolve_prism_version(None, false, None, "1.0.0-dev"),
            "1.0.0-dev"
        );
    }

    // --- Tag-build path ---

    #[test]
    fn github_ref_name_strips_leading_v_on_tag_build() {
        assert_eq!(
            resolve_prism_version(None, true, Some("v1.0.0-beta.1"), "1.0.0-dev"),
            "1.0.0-beta.1"
        );
    }

    #[test]
    fn github_ref_name_without_v_passes_through_on_tag_build() {
        assert_eq!(
            resolve_prism_version(None, true, Some("1.0.0-beta.1"), "1.0.0-dev"),
            "1.0.0-beta.1"
        );
    }

    #[test]
    fn prism_build_version_wins_over_ref_name_on_tag_build() {
        assert_eq!(
            resolve_prism_version(
                Some("custom-override"),
                true,
                Some("v1.0.0-beta.1"),
                "1.0.0-dev"
            ),
            "custom-override"
        );
    }

    // --- CI-leak prevention (F-VID-P1-CRIT-001) ---

    /// (a) Non-tag ref: GITHUB_REF_TYPE="branch", GITHUB_REF_NAME="develop"
    /// must resolve to CARGO_PKG_VERSION, NOT "develop".
    #[test]
    fn non_tag_ref_resolves_to_cargo_pkg_version_not_branch_name() {
        assert_eq!(
            resolve_prism_version(None, false, Some("develop"), "1.0.0-dev"),
            "1.0.0-dev",
            "non-tag ref must not bake branch name into binary (F-VID-P1-CRIT-001)"
        );
    }

    /// Feature branch names with slashes must also fall through.
    #[test]
    fn non_tag_ref_feature_branch_resolves_to_cargo_pkg_version() {
        assert_eq!(
            resolve_prism_version(None, false, Some("feature/S-3.01"), "1.0.0-dev"),
            "1.0.0-dev",
            "feature branch must not bake into binary (F-VID-P1-CRIT-001)"
        );
    }

    /// PR merge-ref pattern (e.g. "260/merge") must also fall through.
    #[test]
    fn non_tag_ref_pr_merge_ref_resolves_to_cargo_pkg_version() {
        assert_eq!(
            resolve_prism_version(None, false, Some("260/merge"), "1.0.0-dev"),
            "1.0.0-dev",
            "PR merge ref must not bake into binary (F-VID-P1-CRIT-001)"
        );
    }

    /// (b) Tag ref: GITHUB_REF_TYPE="tag", GITHUB_REF_NAME="v1.0.0-beta.1"
    /// must resolve to "1.0.0-beta.1".
    #[test]
    fn tag_ref_v_prefix_resolves_to_stripped_version() {
        assert_eq!(
            resolve_prism_version(None, true, Some("v1.0.0-beta.1"), "1.0.0-dev"),
            "1.0.0-beta.1",
            "tag ref must resolve to version with single leading v stripped"
        );
    }

    /// (c) PRISM_BUILD_VERSION takes precedence even on a non-tag build.
    #[test]
    fn prism_build_version_wins_even_on_non_tag_build() {
        assert_eq!(
            resolve_prism_version(Some("1.0.0-custom"), false, Some("develop"), "1.0.0-dev"),
            "1.0.0-custom",
            "PRISM_BUILD_VERSION must win over everything (chain step 1, ADR-064 D2)"
        );
    }

    /// (d) Set-but-empty PRISM_BUILD_VERSION falls through to next chain step.
    #[test]
    fn set_but_empty_build_version_falls_through() {
        assert_eq!(
            resolve_prism_version(Some(""), true, Some("v1.0.0-beta.1"), "1.0.0-dev"),
            "1.0.0-beta.1",
            "empty PRISM_BUILD_VERSION must be treated as absent (F-VID-P1-MED-001)"
        );
    }

    /// Whitespace-only PRISM_BUILD_VERSION also falls through.
    #[test]
    fn whitespace_only_build_version_falls_through() {
        assert_eq!(
            resolve_prism_version(Some("   "), true, Some("v1.0.0-beta.1"), "1.0.0-dev"),
            "1.0.0-beta.1",
            "whitespace-only PRISM_BUILD_VERSION treated as absent (F-VID-P1-MED-001)"
        );
    }

    /// Set-but-empty GITHUB_REF_NAME on a tag build falls through to CARGO_PKG_VERSION.
    #[test]
    fn set_but_empty_ref_name_falls_through_on_tag_build() {
        assert_eq!(
            resolve_prism_version(None, true, Some(""), "1.0.0-dev"),
            "1.0.0-dev",
            "empty GITHUB_REF_NAME must fall through to CARGO_PKG_VERSION"
        );
    }

    /// (e) Single-v strip: "vv1.0.0" → "v1.0.0" (NOT "1.0.0").
    /// strip_prefix removes exactly one leading v; trim_start_matches would strip all.
    #[test]
    fn single_v_strip_not_all_v_strip() {
        assert_eq!(
            resolve_prism_version(None, true, Some("vv1.0.0"), "1.0.0-dev"),
            "v1.0.0",
            "strip_prefix removes exactly one leading v; vv1.0.0 → v1.0.0 not 1.0.0 (F-VID-P1-LOW-001)"
        );
    }
}
