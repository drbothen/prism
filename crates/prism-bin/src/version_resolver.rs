// ADR-064 D2 v1.5 normative fallback chain — shared pure resolver.
//
// This file is the SINGLE source of truth for `resolve_prism_version`.
// It is `include!`'d by `build.rs` (build-script context) AND loaded as
// `pub mod version_resolver` by `lib.rs` (library context).  Changing the
// logic here changes BOTH contexts atomically, making integration tests
// in `tests/version_identity.rs` load-bearing against the exact function
// that `build.rs` uses to emit `PRISM_VERSION`.
//
// HIGH-1 (S-REL-VERSION-IDENTITY pass-2): the previous arrangement had a
// dead `#[cfg(test)]` block in `build.rs` (build scripts have no test harness)
// and a divergent local copy of the resolver inside `version_identity.rs`.
// Deleting either copy left zero executed tests guarding the `is_tag_build`
// gate.  This shared file closes that gap.
//
// Note: uses `//` (not `//!`) throughout so the file is valid in BOTH the
// `include!` context (mid-file in build.rs) and the `mod` context (lib.rs).
//
// Authority: ADR-064 D2 v1.5, S-REL-BVERSION-INJECT-001 HIGH-1.

/// ADR-064 D2 v1.5 normative fallback chain (pure function — shared source).
///
/// Fallback chain:
/// 1. `build_version` wins if present and non-empty/non-whitespace (F-VID-P1-MED-001).
/// 2. On `is_tag_build=true`, `ref_name` with a SINGLE leading `v` stripped wins if
///    present and non-empty. `strip_prefix('v')` removes exactly one `v`; the rejected
///    `trim_start_matches` would strip ALL leading v's — e.g. `vv1.0.0` would become
///    `1.0.0` instead of the correct `v1.0.0` (F-VID-P1-LOW-001).
/// 3. `cargo_version` is the final fallback (resolves to `1.0.0-dev` on develop).
///
/// All empty-string / whitespace-only values are treated as absent (F-VID-P1-MED-001).
///
/// Authority: ADR-064 D2 v1.5, S-REL-BVERSION-INJECT-001 AC-001.
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
    if is_tag_build && let Some(r) = ref_name.filter(|s| !s.trim().is_empty()) {
        let name = r.trim();
        return name.strip_prefix('v').unwrap_or(name).to_string();
    }
    // Step 3: CARGO_PKG_VERSION fallback.
    cargo_version.to_string()
}
