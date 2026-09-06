// ADR-064 §D2 normative fallback chain — shared pure resolver.
//
// This file is the SINGLE source of truth for `resolve_prism_version` AND
// `resolve_is_tag_build`.  It is `include!`'d by `build.rs` (build-script
// context) AND loaded as `pub mod version_resolver` by `lib.rs` (library
// context).  Changing the logic here changes BOTH contexts atomically, making
// integration tests in `tests/version_identity.rs` load-bearing against the
// exact functions that `build.rs` calls to emit `PRISM_VERSION`.
//
// HIGH-1 (S-REL-VERSION-IDENTITY pass-2): the previous arrangement had a
// dead `#[cfg(test)]` block in `build.rs` (build scripts have no test harness)
// and a divergent local copy of the resolver inside `version_identity.rs`.
// Deleting either copy left zero executed tests guarding the `is_tag_build`
// gate.  This shared file closes that gap.
//
// MED-001 (S-REL-VERSION-IDENTITY pass-3): the `is_tag_build` derivation
// (GITHUB_REF_TYPE=="tag" + GITHUB_REF "refs/tags/" fallback + empty/whitespace
// filtering) was inline in `build.rs::main` with ZERO behavioral test coverage.
// `resolve_is_tag_build` extracts it into this shared file so that
// `tests/version_identity.rs` exercises the EXACT same logic build.rs calls.
//
// Note: uses `//` (not `//!`) throughout so the file is valid in BOTH the
// `include!` context (mid-file in build.rs) and the `mod` context (lib.rs).
//
// Authority: ADR-064 §D2, S-REL-BVERSION-INJECT-001 HIGH-1,
//            S-REL-VERSION-IDENTITY pass-3 F-VID-MED-001,
//            S-REL-VERSION-IDENTITY pass-5 OBS-1.

/// Lightweight semver-shape validator for `PRISM_BUILD_VERSION` inputs.
///
/// Accepts `X.Y.Z`, `X.Y.Z-<prerelease>`, `X.Y.Z+<build>`, or
/// `X.Y.Z-<prerelease>+<build>` where:
/// - X, Y, Z are non-empty ASCII digit sequences.
/// - `<prerelease>` and `<build>` are one-or-more dot-separated identifiers,
///   each NON-EMPTY, consisting of chars `[0-9A-Za-z-]`.
///
/// Parse order: strip build metadata on the FIRST `+`, then strip prerelease
/// on the FIRST `-`, then validate the `X.Y.Z` core.  This is intentionally
/// NOT a full semver parser — the goal is to reject obviously-wrong values
/// (e.g. branch names, CRLF-injected payloads) without adding the `semver`
/// crate to `[build-dependencies]`.  Full semver validation happens at the
/// test layer via `semver::Version::parse`.
///
/// Returns `true` for values shaped like `1.0.0`, `1.0.0-beta.1`,
/// `1.0.0+build.5`, or `1.0.0-beta.1+exp.sha.5114f85`.
/// Returns `false` for empty prerelease (`1.0.0-`), empty build (`1.0.0+`),
/// empty dot-chain identifier (`1.0.0-a..b`), four-component core (`1.0.0.0`),
/// non-numeric core (`a.b.c`), or two-component core (`1.0`).
///
/// SEC-001/SEC-002 (S-REL-VERSION-IDENTITY review-cycle-5): called by
/// `resolve_prism_version` to prevent CWE-93 header injection (CRLF) and
/// CWE-20 unvalidated input from reaching `cargo:rustc-env=`.
fn is_semver_shaped(s: &str) -> bool {
    // Step 1: split off optional build metadata at the FIRST '+'.
    let (core_and_pre, build) = match s.split_once('+') {
        Some((c, b)) => (c, Some(b)),
        None => (s, None),
    };
    // Validate build identifiers: one-or-more dot-separated,
    // each NON-EMPTY, chars [0-9A-Za-z-].
    if let Some(b) = build {
        for id in b.split('.') {
            if id.is_empty()
                || !id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            {
                return false;
            }
        }
    }

    // Step 2: split off optional prerelease at the FIRST '-'.
    let (base, pre) = match core_and_pre.split_once('-') {
        Some((b, p)) => (b, Some(p)),
        None => (core_and_pre, None),
    };
    // Validate prerelease identifiers: one-or-more dot-separated,
    // each NON-EMPTY, chars [0-9A-Za-z-].
    if let Some(p) = pre {
        for id in p.split('.') {
            if id.is_empty()
                || !id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            {
                return false;
            }
        }
    }

    // Step 3: the base must be exactly three dot-separated non-empty digit sequences.
    let mut parts = base.split('.');
    let major = parts.next().unwrap_or("");
    let minor = parts.next().unwrap_or("");
    let patch = parts.next().unwrap_or("");
    !major.is_empty()
        && !minor.is_empty()
        && !patch.is_empty()
        && parts.next().is_none() // no fourth component
        && major.bytes().all(|b| b.is_ascii_digit())
        && minor.bytes().all(|b| b.is_ascii_digit())
        && patch.bytes().all(|b| b.is_ascii_digit())
}

/// ADR-064 §D2 normative fallback chain (pure function — shared source).
///
/// Fallback chain:
/// 1. `build_version` wins if present, its first line is non-empty after trim, AND the
///    trimmed first line is semver-shaped (X.Y.Z, X.Y.Z-<pre>, X.Y.Z+<build>, or
///    X.Y.Z-<pre>+<build>).  If the value is
///    multi-line (CWE-93 injection attempt) only the first line is considered.  If the
///    first line is empty or not semver-shaped, fall through to step 2
///    (SEC-001/SEC-002, S-REL-VERSION-IDENTITY review-cycle-5).
/// 2. On `is_tag_build=true`, `ref_name` with a SINGLE leading `v` stripped wins if
///    present and the post-strip result is non-empty/non-whitespace.
///    `strip_prefix('v')` removes exactly one `v`; the rejected `trim_start_matches`
///    would strip ALL leading v's — e.g. `vv1.0.0` would become `1.0.0` instead of
///    the correct `v1.0.0` (F-VID-P1-LOW-001).
///    Post-strip empty guard: a degenerate tag `ref_name == "v"` yields `strip_prefix`
///    result `""`, which must fall through to CARGO_PKG_VERSION (not emit an empty
///    PRISM_VERSION). (OBS-1, S-REL-VERSION-IDENTITY pass-5.)
/// 3. `cargo_version` is the final fallback (resolves to `1.0.0-dev` on develop).
///
/// All empty-string / whitespace-only values are treated as absent (F-VID-P1-MED-001).
///
/// Authority: ADR-064 §D2, S-REL-BVERSION-INJECT-001 AC-001.
pub fn resolve_prism_version(
    build_version: Option<&str>,
    is_tag_build: bool,
    ref_name: Option<&str>,
    cargo_version: &str,
) -> String {
    // Step 1: PRISM_BUILD_VERSION explicit override.
    // SEC-001 (CWE-93): take only the first line to prevent CRLF injection into
    // cargo:rustc-env directives emitted by build.rs.
    // SEC-002 (CWE-20): validate semver shape before baking the value in; a branch
    // name or other non-version string falls through to CARGO_PKG_VERSION.
    if let Some(v) = build_version {
        let first_line = v.lines().next().unwrap_or(v).trim();
        if !first_line.is_empty() && is_semver_shaped(first_line) {
            return first_line.to_string();
        }
        // Empty first line, multi-line-only, or non-semver-shaped: fall through.
    }
    // Step 2: GITHUB_REF_NAME — only on tag builds.
    // strip_prefix removes exactly ONE leading 'v'; trim_start_matches('v') would strip all
    // leading v's (e.g. "vv1.0.0" → "1.0.0" instead of "v1.0.0"). F-VID-P1-LOW-001.
    // Post-strip empty guard: ref_name="v" → strip_prefix → "" → must fall through,
    // not return empty string (OBS-1, S-REL-VERSION-IDENTITY pass-5).
    if is_tag_build && let Some(r) = ref_name.filter(|s| !s.trim().is_empty()) {
        let name = r.trim();
        let version = name.strip_prefix('v').unwrap_or(name);
        if !version.trim().is_empty() {
            return version.to_string();
        }
        // Post-strip result is empty (degenerate tag "v") — fall through to CARGO_PKG_VERSION.
    }
    // Step 3: CARGO_PKG_VERSION fallback.
    cargo_version.to_string()
}

/// Derive `is_tag_build` from CI environment variable inputs (pure function).
///
/// Derivation logic (EXACTLY mirrors the inline derivation previously in `build.rs::main`,
/// extracted for testability per F-VID-MED-001):
/// 1. If `ref_type` is present and non-empty/non-whitespace (trimmed) → return `trimmed == "tag"`.
///    A present ref_type short-circuits: `Some("branch")` returns `false` immediately without
///    consulting `ref_val`.
/// 2. Else if `ref_val` is present and non-empty/non-whitespace → return `starts_with("refs/tags/")`.
///    (Fallback for environments that provide GITHUB_REF but not GITHUB_REF_TYPE.)
/// 3. Else → `false`.
///
/// Parameters match the env var sources in `build.rs::main`:
/// - `ref_type`: `std::env::var("GITHUB_REF_TYPE").ok().as_deref()`
/// - `ref_val`:  `std::env::var("GITHUB_REF").ok().as_deref()`
///
/// Authority: ADR-064 §D2, S-REL-VERSION-IDENTITY pass-3 F-VID-MED-001.
pub fn resolve_is_tag_build(ref_type: Option<&str>, ref_val: Option<&str>) -> bool {
    // Step 1: GITHUB_REF_TYPE primary discriminator.
    // Empty/whitespace values are filtered out (treated as absent).
    // When present and non-empty, the trimmed comparison result is returned immediately —
    // ref_val (GITHUB_REF) is NOT consulted even when ref_type is "branch" or any non-tag value.
    if let Some(t) = ref_type.filter(|s| !s.trim().is_empty()) {
        return t.trim() == "tag";
    }
    // Step 2: GITHUB_REF fallback (only reached when GITHUB_REF_TYPE is absent/empty/whitespace).
    // Empty/whitespace values are filtered out (treated as absent).
    if let Some(r) = ref_val.filter(|s| !s.trim().is_empty()) {
        return r.starts_with("refs/tags/");
    }
    // Step 3: no tag signals present.
    false
}

// SEC-001/SEC-002 unit tests (S-REL-VERSION-IDENTITY review-cycle-5).
//
// These tests are in this file so that when `build.rs` does `include!("src/version_resolver.rs")`
// the same source is exercised by both the build-script path AND the lib-target test harness.
// `#[cfg(test)]` is inactive in build-script context (build scripts have no test harness),
// so the module is only compiled when running `cargo test` / `cargo nextest` against prism-bin.
#[cfg(test)]
mod tests {
    use super::{is_semver_shaped, resolve_prism_version};

    // -------------------------------------------------------------------------
    // SEC-001 (CWE-93): embedded newline must be stripped — only first line used
    // RED before implementation: current code returns the full raw value including
    // the injected second Cargo directive.
    // -------------------------------------------------------------------------

    /// Embedded `\n` in PRISM_BUILD_VERSION must not propagate; only the first line
    /// is returned.  This prevents CWE-93 CRLF injection of additional
    /// `cargo:rustc-env=` directives via a compromised CI env var.
    ///
    /// Authority: SEC-001, S-REL-VERSION-IDENTITY review-cycle-5.
    #[test]
    fn test_sec001_lf_newline_returns_first_line_only() {
        // First line is semver-shaped and must be returned; second line must be dropped.
        assert_eq!(
            resolve_prism_version(
                Some("1.0.0-beta.1\ncargo:rustc-env=EVIL=inject"),
                false,
                None,
                "1.0.0-dev",
            ),
            "1.0.0-beta.1",
            "SEC-001: \\n-embedded PRISM_BUILD_VERSION must return first line only"
        );
    }

    /// Embedded `\r\n` (Windows line ending) must be handled identically.
    ///
    /// Authority: SEC-001, S-REL-VERSION-IDENTITY review-cycle-5.
    #[test]
    fn test_sec001_crlf_newline_returns_first_line_only() {
        assert_eq!(
            resolve_prism_version(
                Some("2.0.0\r\ncargo:rustc-env=EVIL=inject"),
                false,
                None,
                "1.0.0-dev",
            ),
            "2.0.0",
            "SEC-001: \\r\\n-embedded PRISM_BUILD_VERSION must return first line only"
        );
    }

    // -------------------------------------------------------------------------
    // SEC-002 (CWE-20): non-semver PRISM_BUILD_VERSION must fall through
    // RED before implementation: current code returns garbage non-version strings
    // directly into PRISM_VERSION.
    // -------------------------------------------------------------------------

    /// Non-semver PRISM_BUILD_VERSION (no dot-separated numeric base) must fall
    /// through to CARGO_PKG_VERSION, not bake an arbitrary string into the binary.
    ///
    /// Authority: SEC-002, S-REL-VERSION-IDENTITY review-cycle-5.
    #[test]
    fn test_sec002_non_semver_build_version_falls_through_to_cargo() {
        assert_eq!(
            resolve_prism_version(Some("not-a-version"), false, None, "1.0.0-dev"),
            "1.0.0-dev",
            "SEC-002: non-semver PRISM_BUILD_VERSION must fall through to CARGO_PKG_VERSION"
        );
    }

    /// Non-semver PRISM_BUILD_VERSION falls through to GITHUB_REF_NAME on tag builds.
    ///
    /// Authority: SEC-002, S-REL-VERSION-IDENTITY review-cycle-5.
    #[test]
    fn test_sec002_non_semver_build_version_falls_through_to_ref_name_on_tag_build() {
        assert_eq!(
            resolve_prism_version(
                Some("not-a-version"),
                true,
                Some("v1.0.0-beta.1"),
                "1.0.0-dev",
            ),
            "1.0.0-beta.1",
            "SEC-002: non-semver PRISM_BUILD_VERSION falls through to GITHUB_REF_NAME on tag build"
        );
    }

    // -------------------------------------------------------------------------
    // Regression guard: valid semver PRISM_BUILD_VERSION must still pass through
    // (GREEN before and after implementation — this guards against over-restriction)
    // -------------------------------------------------------------------------

    /// Valid semver-shaped PRISM_BUILD_VERSION must still override GITHUB_REF_NAME.
    ///
    /// Authority: ADR-064 D2 step-1, S-REL-VERSION-IDENTITY review-cycle-5.
    #[test]
    fn test_sec001_sec002_valid_semver_build_version_passthrough() {
        // Stable version override.
        assert_eq!(
            resolve_prism_version(Some("1.0.0-beta.1"), false, None, "1.0.0-dev"),
            "1.0.0-beta.1",
            "valid semver PRISM_BUILD_VERSION must still override (regression guard)"
        );
        // Wins over GITHUB_REF_NAME on tag build.
        assert_eq!(
            resolve_prism_version(Some("2.0.0"), true, Some("v1.0.0"), "1.0.0-dev"),
            "2.0.0",
            "valid semver PRISM_BUILD_VERSION must still win over GITHUB_REF_NAME"
        );
        // Whitespace around valid value must be trimmed (F-VID-P1-MED-001 compat).
        assert_eq!(
            resolve_prism_version(Some("  1.2.3  "), false, None, "1.0.0-dev"),
            "1.2.3",
            "leading/trailing whitespace must be trimmed from valid semver PRISM_BUILD_VERSION"
        );
    }

    // -------------------------------------------------------------------------
    // is_semver_shaped helper tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_semver_shaped_accepts_valid_forms() {
        assert!(is_semver_shaped("1.0.0"));
        assert!(is_semver_shaped("0.0.0"));
        assert!(is_semver_shaped("10.20.30"));
        assert!(is_semver_shaped("1.0.0-dev"));
        assert!(is_semver_shaped("1.0.0-beta.1"));
        assert!(is_semver_shaped("1.0.0-rc.2"));
        assert!(is_semver_shaped("1.0.0-alpha.0"));
        // Build metadata must be accepted (OBS finding — previous impl rejected these).
        assert!(is_semver_shaped("1.0.0+build.5"));
        assert!(is_semver_shaped("1.0.0-beta.1+exp.sha.5114f85"));
    }

    #[test]
    fn test_is_semver_shaped_rejects_invalid_forms() {
        assert!(!is_semver_shaped("not-a-version"));
        assert!(!is_semver_shaped("custom-override"));
        assert!(!is_semver_shaped("develop"));
        assert!(!is_semver_shaped("feature/S-3.01"));
        assert!(!is_semver_shaped(""));
        assert!(!is_semver_shaped("1.0"));
        assert!(!is_semver_shaped("1.0.0.0")); // four components
        assert!(!is_semver_shaped("a.b.c"));
        assert!(!is_semver_shaped("1.x.0"));
        // Malformed prerelease/build identifiers must be rejected.
        assert!(!is_semver_shaped("1.0.0-")); // empty prerelease identifier
        assert!(!is_semver_shaped("1.0.0+")); // empty build identifier
        assert!(!is_semver_shaped("1.0.0-a..b")); // empty identifier in prerelease dot-chain
    }
}
