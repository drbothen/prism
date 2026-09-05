// ADR-064 D2 v1.5 normative fallback chain — shared pure resolver.
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
// Authority: ADR-064 D2 v1.6, S-REL-BVERSION-INJECT-001 HIGH-1,
//            S-REL-VERSION-IDENTITY pass-3 F-VID-MED-001,
//            S-REL-VERSION-IDENTITY pass-5 OBS-1.

/// ADR-064 D2 v1.6 normative fallback chain (pure function — shared source).
///
/// Fallback chain:
/// 1. `build_version` wins if present and non-empty/non-whitespace (F-VID-P1-MED-001).
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
/// Authority: ADR-064 D2 v1.6, S-REL-BVERSION-INJECT-001 AC-001.
pub fn resolve_prism_version(
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
/// Authority: ADR-064 D2 v1.5, S-REL-VERSION-IDENTITY pass-3 F-VID-MED-001.
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
