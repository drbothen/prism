// ADR-064 §D2 normative fallback chain — shared pure resolver (prism-spec-engine).
//
// This file is the SINGLE source of truth for `resolve_prism_version`,
// `resolve_is_tag_build`, `is_semver_shaped`, and `prism_build_version_rejected`
// within the prism-spec-engine crate.  It is `include!`'d by `build.rs`
// (build-script context) AND loaded as `pub mod version_resolver` by `lib.rs`
// (library context).  Changing the logic here changes BOTH contexts atomically,
// making tests in the `#[cfg(test)] mod tests` block load-bearing against the
// exact functions that `build.rs` calls to emit `PRISM_VERSION`.
//
// MED-1 (S-REL-AGENT-VERSION-001 pass-1): the previous arrangement had all
// resolver logic inlined inside build.rs (private functions, not testable).
// The doc-comment falsely claimed that `tests/version_identity.rs` (RG-003..005)
// covered the security-critical branches; those tests only observe the compiled
// `env!("PRISM_VERSION")`, which on local dev is exclusively the CARGO_PKG_VERSION
// step-3 fallback.  None of the CWE-20/CWE-93 gates, the is_tag_build derivation,
// or prism_build_version_rejected were exercised by any test.
// This shared file closes that gap — the same pattern as prism-bin HIGH-1/MED-001.
//
// Note: uses `//` (not `//!`) throughout so the file is valid in BOTH the
// `include!` context (mid-file in build.rs) and the `mod` context (lib.rs).
//
// Authority: ADR-064 §D2, ADR-064 D4 §Surface B,
//            S-REL-AGENT-VERSION-001 MED-1 (LOCAL pass-1 fix-burst),
//            S-REL-VERSION-IDENTITY HIGH-1 / MED-001 (prism-bin precedent).

/// Lightweight semver-shape validator for `PRISM_BUILD_VERSION` inputs.
///
/// Accepts `X.Y.Z`, `X.Y.Z-<prerelease>`, `X.Y.Z+<build>`, or
/// `X.Y.Z-<prerelease>+<build>` where:
/// - X, Y, Z are non-empty ASCII digit sequences.
/// - `<prerelease>` and `<build>` are one-or-more dot-separated identifiers,
///   each NON-EMPTY, consisting of chars `[0-9A-Za-z-]`.
///
/// Parse order: strip build metadata on the FIRST `+`, then strip prerelease
/// on the FIRST `-`, then validate the `X.Y.Z` core.  Intentionally NOT a
/// full semver parser — goal is to reject obviously-wrong values (branch names,
/// CRLF-injected payloads) without adding the `semver` crate to
/// `[build-dependencies]`.
///
/// Returns `false` for: empty prerelease (`1.0.0-`), empty build (`1.0.0+`),
/// empty dot-chain identifier (`1.0.0-a..b`), four-component core (`1.0.0.0`),
/// non-numeric core (`a.b.c`), or two-component core (`1.0`).
///
/// Authority: SEC-001 (CWE-93) + SEC-002 (CWE-20), ADR-064 D2.
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
///    trimmed first line is semver-shaped. If the value is multi-line (CWE-93 injection
///    attempt) only the first line is considered. If the first line is empty or not
///    semver-shaped, fall through to step 2 (SEC-001/SEC-002).
/// 2. On `is_tag_build=true`, `ref_name` with a SINGLE leading `v` stripped wins if
///    present and the post-strip result is non-empty/non-whitespace.
///    `strip_prefix('v')` removes exactly one `v`; `trim_start_matches('v')` would
///    strip all leading v's — e.g. `vv1.0.0` would become `1.0.0` instead of the
///    correct `v1.0.0` (F-VID-P1-LOW-001).
///    Post-strip empty guard: degenerate tag `ref_name == "v"` yields `strip_prefix`
///    result `""`, which must fall through to CARGO_PKG_VERSION, not emit empty string
///    (OBS-1, ADR-064 D2 pass-5).
/// 3. `cargo_version` — final fallback (library crate version on local dev).
///
/// All empty-string / whitespace-only values are treated as absent (F-VID-P1-MED-001).
///
/// Authority: ADR-064 §D2, ADR-064 D4 §Surface B.
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
    // not return empty string (OBS-1, ADR-064 D2 pass-5).
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
/// Derivation logic (exactly mirrors the inline derivation previously in `build.rs::main`,
/// extracted for testability per MED-1):
/// 1. If `ref_type` is present and non-empty/non-whitespace (trimmed) → return
///    `trimmed == "tag"`. A present ref_type short-circuits: `Some("branch")` returns
///    `false` immediately without consulting `ref_val`.
/// 2. Else if `ref_val` is present and non-empty/non-whitespace → return
///    `starts_with("refs/tags/")`. (Fallback for environments providing GITHUB_REF
///    but not GITHUB_REF_TYPE.)
/// 3. Else → `false`.
///
/// Parameters match the env var sources in `build.rs::main`:
/// - `ref_type`: `std::env::var("GITHUB_REF_TYPE").ok().as_deref()`
/// - `ref_val`:  `std::env::var("GITHUB_REF").ok().as_deref()`
///
/// Authority: ADR-064 §D2, S-REL-AGENT-VERSION-001 MED-1.
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

/// Returns the offending trimmed first-line value when a non-empty `PRISM_BUILD_VERSION`
/// override fails the semver-shape gate, or `None` if the value is absent,
/// empty/whitespace, or accepted by `is_semver_shaped`.
///
/// Used by `build.rs` to emit a `cargo:warning=` diagnostic when an operator's explicit
/// `PRISM_BUILD_VERSION` is silently ignored and the fallback chain engages — turning a
/// silent fallback into an observable one without changing resolution behavior (the
/// silent-rejection-then-fallback is the deliberate SEC-002 design).
///
/// The returned value, when `Some`, is the trimmed first line only (newline-safe
/// per SEC-001 first-line discipline), so it can be embedded directly in a
/// `cargo:warning=` message without re-introducing the CWE-93 injection risk.
///
/// Authority: OBS-1 (ADR-064 D2), SEC-001/SEC-002.
pub fn prism_build_version_rejected(raw: Option<&str>) -> Option<String> {
    let v = raw?;
    let first_line = v.lines().next().unwrap_or(v).trim();
    if first_line.is_empty() {
        return None; // empty / whitespace-only → treated as absent (F-VID-P1-MED-001)
    }
    if is_semver_shaped(first_line) {
        return None; // accepted by resolve_prism_version chain step 1
    }
    Some(first_line.to_string()) // non-empty, non-semver-shaped → rejected
}

// Unit tests for the resolver functions.
//
// These tests are in this file so that when `build.rs` does `include!("src/version_resolver.rs")`
// the same source is exercised by both the build-script path AND the lib-target test harness.
// `#[cfg(test)]` is inactive in build-script context (build scripts have no test harness),
// so the module is only compiled when running `cargo test` / `cargo nextest` against prism-spec-engine.
//
// The tests are load-bearing: a mutation to the tag-gate, CWE-93 first-line extraction,
// or semver-shape check will fail at least one test here (TD-VSDD-059 paper-fix guard).
//
// Authority: S-REL-AGENT-VERSION-001 MED-1, mirroring prism-bin HIGH-1/MED-001 test suite.
#[cfg(test)]
mod tests {
    use super::{
        is_semver_shaped, prism_build_version_rejected, resolve_is_tag_build, resolve_prism_version,
    };

    // ─── is_semver_shaped accept set ──────────────────────────────────────────

    #[test]
    fn test_is_semver_shaped_accepts_valid_forms() {
        // Core X.Y.Z forms.
        assert!(is_semver_shaped("1.0.0"));
        assert!(is_semver_shaped("0.0.0"));
        assert!(is_semver_shaped("10.20.30"));
        // Prerelease suffixes.
        assert!(is_semver_shaped("1.0.0-dev"));
        assert!(is_semver_shaped("1.0.0-beta.1"));
        assert!(is_semver_shaped("1.0.0-rc.2"));
        assert!(is_semver_shaped("1.0.0-alpha.0"));
        // Build metadata (must be accepted per OBS-1 finding that previous impl rejected these).
        assert!(is_semver_shaped("1.0.0+build.5"));
        assert!(is_semver_shaped("1.0.0-beta.1+exp.sha.5114f85"));
    }

    // ─── is_semver_shaped reject set ──────────────────────────────────────────

    #[test]
    fn test_is_semver_shaped_rejects_invalid_forms() {
        // Non-numeric / non-version strings (SEC-002 CWE-20 targets).
        assert!(!is_semver_shaped("not-a-version"));
        assert!(!is_semver_shaped("custom-override"));
        assert!(!is_semver_shaped("develop"));
        assert!(!is_semver_shaped("feature/S-3.01"));
        assert!(!is_semver_shaped(""));
        // Structural rejects.
        assert!(!is_semver_shaped("1.0")); // two-component core
        assert!(!is_semver_shaped("1.0.0.0")); // four-component core
        assert!(!is_semver_shaped("a.b.c")); // non-numeric core
        assert!(!is_semver_shaped("1.x.0")); // non-numeric minor
        // Malformed prerelease/build identifiers.
        assert!(!is_semver_shaped("1.0.0-")); // empty prerelease identifier
        assert!(!is_semver_shaped("1.0.0+")); // empty build identifier
        assert!(!is_semver_shaped("1.0.0-a..b")); // empty identifier in prerelease dot-chain
    }

    // ─── SEC-001 (CWE-93): first-line extraction ──────────────────────────────

    /// Embedded `\n` in PRISM_BUILD_VERSION must not propagate; only the first line
    /// is used.  Prevents CWE-93 CRLF injection of additional `cargo:rustc-env=`
    /// directives via a compromised CI env var.
    ///
    /// Authority: SEC-001, ADR-064 D2.
    #[test]
    fn test_sec001_lf_newline_returns_first_line_only() {
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
    /// Authority: SEC-001, ADR-064 D2.
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

    // ─── SEC-002 (CWE-20): non-semver PRISM_BUILD_VERSION falls through ────────

    /// Non-semver PRISM_BUILD_VERSION must fall through to CARGO_PKG_VERSION, not bake
    /// an arbitrary string into the compiled binary.
    ///
    /// Authority: SEC-002, ADR-064 D2.
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
    /// Authority: SEC-002, ADR-064 D2.
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

    // ─── Regression guard: valid semver must still pass through ───────────────

    /// Valid semver-shaped PRISM_BUILD_VERSION must still override GITHUB_REF_NAME
    /// (guard against over-restriction).
    ///
    /// Authority: ADR-064 D2 step-1.
    #[test]
    fn test_valid_semver_build_version_passthrough() {
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

    // ─── 3-step fallback order ─────────────────────────────────────────────────

    /// Step 1 wins over steps 2 and 3 when PRISM_BUILD_VERSION is valid semver.
    #[test]
    fn test_fallback_step1_wins_over_step2_and_step3() {
        assert_eq!(
            resolve_prism_version(Some("1.5.0"), true, Some("v2.0.0"), "3.0.0-dev"),
            "1.5.0",
            "step-1 (PRISM_BUILD_VERSION) must win over step-2 (GITHUB_REF_NAME)"
        );
    }

    /// Step 2 wins over step 3 on a tag build with no PRISM_BUILD_VERSION.
    #[test]
    fn test_fallback_step2_wins_over_step3_on_tag_build() {
        assert_eq!(
            resolve_prism_version(None, true, Some("v1.9.0"), "3.0.0-dev"),
            "1.9.0",
            "step-2 (GITHUB_REF_NAME) must win over step-3 (CARGO_PKG_VERSION) on tag build"
        );
    }

    /// Step 2 is skipped entirely on non-tag builds.
    #[test]
    fn test_fallback_step2_skipped_on_non_tag_build() {
        assert_eq!(
            resolve_prism_version(None, false, Some("develop"), "3.0.0-dev"),
            "3.0.0-dev",
            "step-2 must be skipped when is_tag_build=false (branch-name leak guard)"
        );
    }

    /// Degenerate tag "v" produces empty strip_prefix result and must fall through to step 3,
    /// not return an empty PRISM_VERSION (OBS-1, ADR-064 D2 pass-5).
    #[test]
    fn test_fallback_degenerate_v_tag_falls_through_to_cargo() {
        assert_eq!(
            resolve_prism_version(None, true, Some("v"), "3.0.0-dev"),
            "3.0.0-dev",
            "degenerate tag 'v' strips to empty string and must fall through to CARGO_PKG_VERSION"
        );
    }

    // ─── resolve_is_tag_build ─────────────────────────────────────────────────

    /// GITHUB_REF_TYPE="tag" → is_tag_build=true.
    #[test]
    fn test_resolve_is_tag_build_ref_type_tag_returns_true() {
        assert!(
            resolve_is_tag_build(Some("tag"), None),
            "GITHUB_REF_TYPE='tag' must return true"
        );
    }

    /// GITHUB_REF_TYPE="branch" → is_tag_build=false (short-circuits without consulting GITHUB_REF).
    #[test]
    fn test_resolve_is_tag_build_ref_type_branch_returns_false() {
        // Even when GITHUB_REF looks like a tag, GITHUB_REF_TYPE="branch" must win.
        assert!(
            !resolve_is_tag_build(Some("branch"), Some("refs/tags/v1.0.0")),
            "GITHUB_REF_TYPE='branch' must return false (branch-leak guard)"
        );
    }

    /// GITHUB_REF_TYPE absent, GITHUB_REF starts with "refs/tags/" → is_tag_build=true.
    #[test]
    fn test_resolve_is_tag_build_fallback_refs_tags_prefix_returns_true() {
        assert!(
            resolve_is_tag_build(None, Some("refs/tags/v1.0.0")),
            "GITHUB_REF 'refs/tags/v1.0.0' must return true when GITHUB_REF_TYPE absent"
        );
    }

    /// GITHUB_REF_TYPE absent, GITHUB_REF starts with "refs/heads/" → is_tag_build=false.
    #[test]
    fn test_resolve_is_tag_build_fallback_refs_heads_prefix_returns_false() {
        assert!(
            !resolve_is_tag_build(None, Some("refs/heads/develop")),
            "GITHUB_REF 'refs/heads/develop' must return false"
        );
    }

    /// Both absent → is_tag_build=false (local dev default).
    #[test]
    fn test_resolve_is_tag_build_both_absent_returns_false() {
        assert!(
            !resolve_is_tag_build(None, None),
            "no tag signals present must return false (local dev default)"
        );
    }

    /// Empty/whitespace GITHUB_REF_TYPE treated as absent; GITHUB_REF fallback applies.
    #[test]
    fn test_resolve_is_tag_build_empty_ref_type_falls_through_to_ref_val() {
        assert!(
            resolve_is_tag_build(Some(""), Some("refs/tags/v2.0.0")),
            "empty GITHUB_REF_TYPE treated as absent; GITHUB_REF tag fallback must apply"
        );
        assert!(
            resolve_is_tag_build(Some("   "), Some("refs/tags/v2.0.0")),
            "whitespace-only GITHUB_REF_TYPE treated as absent; GITHUB_REF tag fallback must apply"
        );
    }

    // ─── prism_build_version_rejected ─────────────────────────────────────────

    /// Non-empty, non-semver PRISM_BUILD_VERSION → Some(offending first-line value).
    ///
    /// Authority: OBS-1, ADR-064 D2.
    #[test]
    fn test_rejected_non_empty_non_semver_returns_some() {
        assert_eq!(
            prism_build_version_rejected(Some("not-a-version")),
            Some("not-a-version".to_string()),
            "non-semver non-empty value must return Some(value)"
        );
        assert_eq!(
            prism_build_version_rejected(Some("my-branch")),
            Some("my-branch".to_string()),
            "branch name must return Some(branch)"
        );
        assert_eq!(
            prism_build_version_rejected(Some("feature/S-3.01")),
            Some("feature/S-3.01".to_string()),
            "feature branch name must return Some(value)"
        );
    }

    /// Valid semver-shaped value → None (accepted, no warning needed).
    ///
    /// Authority: OBS-1, ADR-064 D2.
    #[test]
    fn test_rejected_valid_semver_returns_none() {
        assert_eq!(
            prism_build_version_rejected(Some("1.0.0")),
            None,
            "stable semver must return None (accepted)"
        );
        assert_eq!(
            prism_build_version_rejected(Some("1.0.0-beta.1")),
            None,
            "pre-release semver must return None (accepted)"
        );
        assert_eq!(
            prism_build_version_rejected(Some("2.0.0+build.5")),
            None,
            "build-metadata semver must return None (accepted)"
        );
        assert_eq!(
            prism_build_version_rejected(Some("1.0.0-beta.1+exp.sha.5114f85")),
            None,
            "full semver with pre+build must return None (accepted)"
        );
    }

    /// Empty and whitespace-only → None (treated as absent; F-VID-P1-MED-001).
    ///
    /// Authority: OBS-1, F-VID-P1-MED-001.
    #[test]
    fn test_rejected_empty_and_whitespace_return_none() {
        assert_eq!(
            prism_build_version_rejected(Some("")),
            None,
            "empty string must return None (treated as absent)"
        );
        assert_eq!(
            prism_build_version_rejected(Some("   ")),
            None,
            "whitespace-only must return None (treated as absent)"
        );
        assert_eq!(
            prism_build_version_rejected(Some("\t")),
            None,
            "tab-only must return None (treated as absent)"
        );
    }

    /// None (env var not set) → None (nothing to warn about).
    ///
    /// Authority: OBS-1, ADR-064 D2.
    #[test]
    fn test_rejected_none_input_returns_none() {
        assert_eq!(
            prism_build_version_rejected(None),
            None,
            "None (absent env var) must return None"
        );
    }

    /// Embedded newline, non-semver first line → Some(trimmed first line only).
    /// The returned value is newline-safe so it can be embedded in a `cargo:warning=`
    /// message without re-introducing the CWE-93 injection risk.
    ///
    /// Authority: OBS-1, SEC-001, ADR-064 D2.
    #[test]
    fn test_rejected_embedded_newline_non_semver_returns_first_line() {
        // Multi-line: non-semver first line → Some(first line, not the full value).
        assert_eq!(
            prism_build_version_rejected(Some("my-branch\ncargo:rustc-env=EVIL=inject")),
            Some("my-branch".to_string()),
            "embedded \\n: non-semver first line must return Some(first line only)"
        );
        // Multi-line: valid semver first line → None (accepted, no warning).
        assert_eq!(
            prism_build_version_rejected(Some("1.0.0-beta.1\ncargo:rustc-env=EVIL=inject")),
            None,
            "embedded \\n: valid semver first line must return None (accepted)"
        );
        // CRLF: non-semver first line.
        assert_eq!(
            prism_build_version_rejected(Some("my-branch\r\ncargo:rustc-env=EVIL=inject")),
            Some("my-branch".to_string()),
            "embedded \\r\\n: non-semver first line must return Some(first line only)"
        );
    }
}
