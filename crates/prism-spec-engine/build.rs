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
//! Note: this build.rs is self-contained — it does NOT use `include!` from
//! `prism-bin/src/version_resolver.rs`. Build scripts run in the crate's own context;
//! cross-crate `include!` would reference a path that is not available during
//! `prism-spec-engine`'s build. The algorithm is small and stable (~40 lines) and is
//! covered independently by unit tests added to `prism-spec-engine/tests/` (RG-003..005).
//!
//! Authority: ADR-064 D4 §Surface B, S-REL-AGENT-VERSION-001 AC-006.

fn main() {
    // Re-run if the build script itself changes.
    println!("cargo:rerun-if-changed=build.rs");
    // Re-run if any relevant env var changes (ADR-064 D4 §Surface B — four directives).
    println!("cargo:rerun-if-env-changed=PRISM_BUILD_VERSION");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_TYPE");
    println!("cargo:rerun-if-env-changed=GITHUB_REF");

    let cargo_version = env!("CARGO_PKG_VERSION");

    // GITHUB_REF_NAME is set on ALL GitHub Actions runs — branch name on push/pull_request
    // events, tag name only on tag-push events. Gate on GITHUB_REF_TYPE == "tag" (or
    // GITHUB_REF starts with "refs/tags/") to avoid baking "develop" / "feature/..."
    // into non-release binaries. F-VID-P1-CRIT-001.
    let is_tag_build = std::env::var("GITHUB_REF_TYPE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|t| t.trim() == "tag")
        .unwrap_or_else(|| {
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

// ─── Inline resolver (D2-conformant, dependency-free) ────────────────────────
//
// These functions are inlined (not include!'d from prism-bin) because build.rs
// runs in the prism-spec-engine build context and cannot reference prism-bin source.
// The algorithm is identical to prism-bin/src/version_resolver.rs; any change to
// the D2 normative contract MUST be applied to BOTH files (ADR-064 D4 §Surface B:
// "parallel implementations are acceptable when the spec is authoritative").
//
// Unit tests in crates/prism-spec-engine/tests/version_identity.rs provide
// independent coverage of the compiled env var (RG-003/RG-004/RG-005), confirming
// that this inline copy functions correctly as a parallel implementation.

/// Lightweight semver-shape validator for `PRISM_BUILD_VERSION` inputs.
///
/// Accepts `X.Y.Z`, `X.Y.Z-<prerelease>`, `X.Y.Z+<build>`, or
/// `X.Y.Z-<prerelease>+<build>` where X, Y, Z are non-empty ASCII digit sequences
/// and prerelease/build identifiers are non-empty dot-separated `[0-9A-Za-z-]` strings.
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
    // Step 3: base must be exactly three dot-separated non-empty digit sequences.
    let mut parts = base.split('.');
    let major = parts.next().unwrap_or("");
    let minor = parts.next().unwrap_or("");
    let patch = parts.next().unwrap_or("");
    !major.is_empty()
        && !minor.is_empty()
        && !patch.is_empty()
        && parts.next().is_none()
        && major.bytes().all(|b| b.is_ascii_digit())
        && minor.bytes().all(|b| b.is_ascii_digit())
        && patch.bytes().all(|b| b.is_ascii_digit())
}

/// ADR-064 §D2 normative fallback chain (pure function, inline copy for build.rs context).
///
/// Fallback chain:
/// 1. `build_version` — wins if present, first line non-empty after trim, AND semver-shaped.
///    SEC-001 (CWE-93): first line only. SEC-002 (CWE-20): shape-gated.
/// 2. `ref_name` with single leading `v` stripped — only on `is_tag_build=true`,
///    only when post-strip result is non-empty/non-whitespace (OBS-1 post-strip guard).
/// 3. `cargo_version` — final fallback (library crate version on local dev).
fn resolve_prism_version(
    build_version: Option<&str>,
    is_tag_build: bool,
    ref_name: Option<&str>,
    cargo_version: &str,
) -> String {
    // Step 1: PRISM_BUILD_VERSION explicit override.
    if let Some(v) = build_version {
        let first_line = v.lines().next().unwrap_or(v).trim();
        if !first_line.is_empty() && is_semver_shaped(first_line) {
            return first_line.to_string();
        }
    }
    // Step 2: GITHUB_REF_NAME — only on tag builds.
    if is_tag_build && let Some(r) = ref_name.filter(|s| !s.trim().is_empty()) {
        let name = r.trim();
        let version = name.strip_prefix('v').unwrap_or(name);
        if !version.trim().is_empty() {
            return version.to_string();
        }
    }
    // Step 3: CARGO_PKG_VERSION fallback.
    cargo_version.to_string()
}

/// Returns the offending first-line value when a non-empty PRISM_BUILD_VERSION fails
/// the semver-shape gate, or None if absent/accepted.
/// Used to emit `cargo:warning=` diagnostics (OBS-1, ADR-064 D2).
fn prism_build_version_rejected(raw: Option<&str>) -> Option<String> {
    let v = raw?;
    let first_line = v.lines().next().unwrap_or(v).trim();
    if first_line.is_empty() {
        return None;
    }
    if is_semver_shaped(first_line) {
        return None;
    }
    Some(first_line.to_string())
}
