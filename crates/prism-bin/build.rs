//! Build script for `prism-bin`.
//!
//! Resolves `PRISM_VERSION` via the ADR-064 D2 normative fallback chain:
//!   1. `PRISM_BUILD_VERSION` env var (explicit override, CI artefact signing)
//!   2. `GITHUB_REF_NAME` env var with a single leading `v` stripped
//!      (GitHub Actions tag/branch ref, e.g. `v1.0.0-beta.1` → `1.0.0-beta.1`)
//!   3. `CARGO_PKG_VERSION` (local dev default; always `1.0.0-dev` on `develop`)
//!
//! The resolved value is emitted as `PRISM_VERSION` and available at compile
//! time via `env!("PRISM_VERSION")` in all `prism-bin` source files.
//!
//! Authority: ADR-064 D2, S-REL-BVERSION-INJECT-001 AC-001.

fn main() {
    // Re-run if either override env var changes.
    println!("cargo:rerun-if-env-changed=PRISM_BUILD_VERSION");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");

    let build_version = std::env::var("PRISM_BUILD_VERSION").ok();
    let ref_name = std::env::var("GITHUB_REF_NAME").ok();
    let version = resolve_prism_version(
        build_version.as_deref(),
        ref_name.as_deref(),
        env!("CARGO_PKG_VERSION"),
    );

    println!("cargo:rustc-env=PRISM_VERSION={version}");
}

/// ADR-064 D2 normative fallback chain (pure function — testable).
///
/// 1. `build_version` wins if present.
/// 2. `ref_name` with a single leading `v` stripped wins next.
/// 3. `cargo_version` is the final fallback.
fn resolve_prism_version<'a>(
    build_version: Option<&'a str>,
    ref_name: Option<&'a str>,
    cargo_version: &'a str,
) -> &'a str {
    if let Some(v) = build_version {
        return v;
    }
    if let Some(r) = ref_name {
        return r.trim_start_matches('v');
    }
    cargo_version
}

#[cfg(test)]
mod tests {
    use super::resolve_prism_version;

    #[test]
    fn fallback_to_cargo_pkg_version() {
        assert_eq!(resolve_prism_version(None, None, "1.0.0-dev"), "1.0.0-dev");
    }

    #[test]
    fn github_ref_name_strips_leading_v() {
        assert_eq!(
            resolve_prism_version(None, Some("v1.0.0-beta.1"), "1.0.0-dev"),
            "1.0.0-beta.1"
        );
    }

    #[test]
    fn github_ref_name_without_v_passes_through() {
        assert_eq!(
            resolve_prism_version(None, Some("1.0.0-beta.1"), "1.0.0-dev"),
            "1.0.0-beta.1"
        );
    }

    #[test]
    fn prism_build_version_wins_over_ref_name() {
        assert_eq!(
            resolve_prism_version(Some("custom-override"), Some("v1.0.0-beta.1"), "1.0.0-dev"),
            "custom-override"
        );
    }
}
