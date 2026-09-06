// Version identity Red Gate tests
// S-REL-DEV-RESET-001: RG-001, RG-002  (ADR-064 D1 — develop carries 1.0.0-dev)
// S-REL-BVERSION-INJECT-001: RG-001..RG-004  (ADR-064 D2 — build.rs PRISM_VERSION injection)
// S-REL-VERSION-IDENTITY pass-2 HIGH-1: shared resolver exercises + env-var assertions
//
// HIGH-1 fix: `resolve_prism_version` now lives in `src/version_resolver.rs` (single
// source of truth).  It is `include!`'d by `build.rs` AND exposed via `pub mod
// version_resolver` in `lib.rs`.  The tests below import it directly from the lib
// target — deleting the `is_tag_build` gate in `version_resolver.rs` causes
// `test_shared_resolver_non_tag_ref_must_not_leak_branch_name` to fail.
//
// All tests are written to FAIL before implementation.
// S-REL-BVERSION-INJECT-001 RG-001..RG-004 use the source-level / pure-helper approach
// (per orchestrator guidance: avoid compile-fail tests that block the whole crate from building
// and therefore also block verification of RG-001/RG-002 above).  The documented red-failure
// mode for each test is stated in its doc-comment.

// Import the shared resolvers from the prism_bin lib target.
// These are the SAME functions include!'d by build.rs — any mutation to
// version_resolver.rs affects both the emitted PRISM_VERSION AND these tests.
use prism_bin::version_resolver::resolve_is_tag_build;
use prism_bin::version_resolver::resolve_prism_version;

// ============================================================================
// S-REL-DEV-RESET-001
// ============================================================================

/// RG-001 (S-REL-DEV-RESET-001)
///
/// Assert that `prism-bin/Cargo.toml` `version` satisfies the ADR-064 D1 invariant:
/// `CARGO_PKG_VERSION` must be `X.Y.Z` (stable, e.g. after release-prep.yml bumps before
/// a stable tag) or `X.Y.Z-dev` (develop channel).  It must NEVER be `X.Y.Z-rc.N`,
/// `X.Y.Z-beta.N`, `X.Y.Z-alpha.N`, etc. — those stale channel suffixes are exactly
/// what S-REL-DEV-RESET-001 was created to fix.
///
/// **RED before implementation:** `CARGO_PKG_VERSION` == `"1.0.0-rc.1"` — pre-release
/// segment is `rc.1`, assertion fails.
/// **GREEN after implementation:** Cargo.toml reset to `1.0.0-dev` (pre == "dev") or to
/// `1.0.0` (pre empty, stable release build).
///
/// B-3 fix (S-REL-VERSION-IDENTITY review-cycle-2): replaced literal `"1.0.0-dev"` equality
/// with the ADR-064 D1 invariant so that release-prep.yml stable-release PRs (which bump
/// Cargo.toml to `X.Y.Z`) still pass the CI suite on the `on: push:` trigger.
///
/// Authority: ADR-064 D1, S-REL-DEV-RESET-001 AC-001, RELEASE-CHANNELS.md §3.
#[test]
fn test_prism_bin_cargo_toml_version_is_dev() {
    // The ADR-064 D1 invariant: CARGO_PKG_VERSION on develop is either X.Y.Z (stable,
    // e.g. when release-prep.yml bumps before a stable tag) or X.Y.Z-dev (develop channel).
    // It must never be X.Y.Z-rc.N, X.Y.Z-beta.N, etc. — those are the stale suffixes
    // that S-REL-DEV-RESET-001 was created to fix.
    let cargo_version = env!("CARGO_PKG_VERSION");
    let parsed = semver::Version::parse(cargo_version).unwrap_or_else(|e| {
        panic!(
            "CARGO_PKG_VERSION '{}' is not valid semver: {}",
            cargo_version, e
        )
    });
    let pre = &parsed.pre;
    assert!(
        pre.is_empty() || pre.as_str() == "dev",
        "CARGO_PKG_VERSION '{}' has pre-release segment '{}' — must be empty (stable) or 'dev' (develop channel). \
         Channel suffixes like rc.N, beta.N, alpha.N must only appear in PRISM_VERSION via build.rs tag injection.",
        cargo_version,
        pre
    );
}

/// RG-002 (S-REL-DEV-RESET-001)
///
/// Assert that `tests/external/non-exhaustive-violation/Cargo.lock` records
/// `prism-bin` at the same version as `CARGO_PKG_VERSION`.
///
/// **RED before implementation:** the nested workspace Cargo.lock still pins `"1.0.0-rc.1"`.
/// **GREEN after implementation:** `cargo update -p prism-bin` regenerates both lockfiles
/// per S-REL-DEV-RESET-001 AC-003, and the lockfile matches CARGO_PKG_VERSION.
///
/// B-3 fix (S-REL-VERSION-IDENTITY review-cycle-2): replaced literal `"1.0.0-dev"` in the
/// lockfile check with a dynamic `env!("CARGO_PKG_VERSION")` format so that the assertion
/// holds after release-prep.yml bumps Cargo.toml (and regenerates lockfiles) to `X.Y.Z`.
///
/// Authority: ADR-064 D1 ownership map, S-REL-DEV-RESET-001 AC-003.
#[test]
fn test_non_exhaustive_lockfile_version_matches() {
    let lock = include_str!("../../../tests/external/non-exhaustive-violation/Cargo.lock");
    let idx = lock
        .find(r#"name = "prism-bin""#)
        .expect("prism-bin entry not found in non-exhaustive-violation/Cargo.lock");
    let tail = &lock[idx..];
    // B-3 fix: build the expected version line dynamically from CARGO_PKG_VERSION so that
    // this test stays green after release-prep.yml bumps the workspace to a stable version.
    // The lockfile must always match Cargo.toml — using the literal "1.0.0-dev" would fail
    // when release-prep.yml cuts the stable release and both files are updated to X.Y.Z.
    let expected_version_line = format!(r#"version = "{}""#, env!("CARGO_PKG_VERSION"));
    assert!(
        tail.contains(&expected_version_line),
        "tests/external/non-exhaustive-violation/Cargo.lock still pins prism-bin to a \
         different version than CARGO_PKG_VERSION (expected '{}'). \
         Run: cd tests/external/non-exhaustive-violation && cargo update -p prism-bin \
         (S-REL-DEV-RESET-001 AC-003)",
        expected_version_line
    );
}

// ============================================================================
// S-REL-BVERSION-INJECT-001
//
// Source-level / pure-helper approach: tests read the prism-bin source tree at
// runtime via CARGO_MANIFEST_DIR and std::fs::read_to_string.  This keeps the
// test file compilable (no use of env!("PRISM_VERSION") which would require
// build.rs to already exist), allowing the DEV-RESET tests above to be verified
// independently.
// ============================================================================

/// RG-001 (S-REL-BVERSION-INJECT-001)
///
/// Assert that `crates/prism-bin/build.rs` exists and emits
/// `cargo:rustc-env=PRISM_VERSION=` per ADR-064 D2 normative contract.
///
/// **RED before implementation:** `build.rs` does not exist — `read_to_string` panics.
/// **GREEN after implementation:** `build.rs` created with the normative fallback chain.
///
/// Authority: ADR-064 D2, S-REL-BVERSION-INJECT-001 AC-001.
#[test]
fn test_prism_version_env_var_is_available() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let build_rs_path = format!("{manifest_dir}/build.rs");
    let content = std::fs::read_to_string(&build_rs_path).unwrap_or_else(|_| {
        panic!(
            "crates/prism-bin/build.rs must exist per ADR-064 D2 \
             (S-REL-BVERSION-INJECT-001 AC-001). \
             Checked path: {build_rs_path}"
        )
    });
    assert!(
        content.contains("cargo:rustc-env=PRISM_VERSION="),
        "build.rs must emit PRISM_VERSION via 'cargo:rustc-env=PRISM_VERSION=' \
         per ADR-064 D2 normative contract (S-REL-BVERSION-INJECT-001 AC-001). \
         Found build.rs but directive is absent."
    );
}

/// RG-002 (S-REL-BVERSION-INJECT-001)
///
/// Pure-helper test of the ADR-064 D2 normative fallback chain:
///   `PRISM_BUILD_VERSION` → `GITHUB_REF_NAME` (stripped of leading `v`) → `CARGO_PKG_VERSION`
///
/// Also asserts `build.rs` contains all three chain references, and that
/// `CARGO_PKG_VERSION == "1.0.0-dev"` (gating S-REL-DEV-RESET-001 completion).
///
/// **RED before S-REL-DEV-RESET-001:** `CARGO_PKG_VERSION == "1.0.0-rc.1"` — assertion fails.
/// **RED after DEV-RESET but before BVERSION:** `build.rs` not found — first assertion panics.
/// **GREEN after both:** build.rs exists with correct chain and CARGO_PKG_VERSION == "1.0.0-dev".
///
/// Authority: ADR-064 D2 fallback chain, S-REL-BVERSION-INJECT-001 AC-005.
#[test]
fn test_build_rs_fallback_uses_cargo_pkg_version_when_no_env() {
    // ---- (1) Structural check: build.rs contains all chain references ----
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let build_rs_path = format!("{manifest_dir}/build.rs");
    let content = std::fs::read_to_string(&build_rs_path).unwrap_or_else(|_| {
        panic!(
            "crates/prism-bin/build.rs must exist per ADR-064 D2 \
             (S-REL-BVERSION-INJECT-001 AC-001). Checked: {build_rs_path}"
        )
    });
    assert!(
        content.contains("PRISM_BUILD_VERSION"),
        "build.rs must check PRISM_BUILD_VERSION (chain step 1 per ADR-064 D2 v1.5)"
    );
    assert!(
        content.contains("GITHUB_REF_TYPE"),
        "build.rs must gate GITHUB_REF_NAME on GITHUB_REF_TYPE == 'tag' \
         (F-VID-P1-CRIT-001, ADR-064 D2 v1.5 — prevents baking branch names into binary)"
    );
    assert!(
        content.contains("GITHUB_REF_NAME"),
        "build.rs must check GITHUB_REF_NAME (chain step 2 per ADR-064 D2 v1.5)"
    );
    assert!(
        content.contains("GITHUB_REF"),
        "build.rs must include GITHUB_REF as fallback for GITHUB_REF_TYPE \
         (ADR-064 D2 v1.5 rerun-if-env-changed and is_tag_build fallback)"
    );
    assert!(
        content.contains("CARGO_PKG_VERSION"),
        "build.rs must fall back to CARGO_PKG_VERSION (chain step 3 per ADR-064 D2 v1.5)"
    );
    assert!(
        content.contains("strip_prefix"),
        "build.rs must use strip_prefix (not trim_start_matches) for single-v semantics \
         (F-VID-P1-LOW-001, ADR-064 D2 v1.5)"
    );

    // ---- (2) Shared resolver (prism_bin::version_resolver::resolve_prism_version) ----
    // This is the EXACT function include!'d by build.rs.  No local copy — single
    // source of truth (HIGH-1, S-REL-VERSION-IDENTITY pass-2).

    // Local-dev path: no env vars → CARGO_PKG_VERSION.
    assert_eq!(
        resolve_prism_version(None, false, None, "1.0.0-dev"),
        "1.0.0-dev",
        "fallback chain must return CARGO_PKG_VERSION when no overrides set (ADR-064 D2 step 3)"
    );

    // GITHUB_REF_NAME path on tag build: must strip leading 'v' (ADR-064 D2 step 2).
    assert_eq!(
        resolve_prism_version(None, true, Some("v1.0.0-beta.1"), "1.0.0-dev"),
        "1.0.0-beta.1",
        "GITHUB_REF_NAME must have leading 'v' stripped on tag build (ADR-064 D2 step 2)"
    );
    assert_eq!(
        resolve_prism_version(None, true, Some("1.0.0-beta.1"), "1.0.0-dev"),
        "1.0.0-beta.1",
        "GITHUB_REF_NAME without leading 'v' must pass through unchanged"
    );

    // PRISM_BUILD_VERSION wins over GITHUB_REF_NAME (ADR-064 D2 step 1).
    assert_eq!(
        resolve_prism_version(
            Some("custom-override"),
            true,
            Some("v1.0.0-beta.1"),
            "1.0.0-dev"
        ),
        "custom-override",
        "PRISM_BUILD_VERSION must take priority over GITHUB_REF_NAME (ADR-064 D2 step 1)"
    );

    // ---- (3) CI-leak prevention cases (F-VID-P1-CRIT-001) ----
    // (a) Non-tag ref (GITHUB_REF_TYPE="branch") must resolve to CARGO_PKG_VERSION.
    assert_eq!(
        resolve_prism_version(None, false, Some("develop"), "1.0.0-dev"),
        "1.0.0-dev",
        "non-tag ref 'develop' must NOT bake branch name into binary (F-VID-P1-CRIT-001)"
    );
    assert_eq!(
        resolve_prism_version(None, false, Some("feature/S-3.01"), "1.0.0-dev"),
        "1.0.0-dev",
        "non-tag ref 'feature/S-3.01' must NOT bake branch name into binary (F-VID-P1-CRIT-001)"
    );

    // (b) Tag ref (GITHUB_REF_TYPE="tag") resolves to stripped version.
    assert_eq!(
        resolve_prism_version(None, true, Some("v1.0.0-beta.1"), "1.0.0-dev"),
        "1.0.0-beta.1",
        "tag ref 'v1.0.0-beta.1' must resolve to '1.0.0-beta.1' (ADR-064 D2 step 2)"
    );

    // (c) PRISM_BUILD_VERSION wins even on non-tag build.
    assert_eq!(
        resolve_prism_version(Some("1.0.0-custom"), false, Some("develop"), "1.0.0-dev"),
        "1.0.0-custom",
        "PRISM_BUILD_VERSION must win over everything (chain step 1)"
    );

    // (d) Set-but-empty PRISM_BUILD_VERSION falls through (F-VID-P1-MED-001).
    assert_eq!(
        resolve_prism_version(Some(""), true, Some("v1.0.0-beta.1"), "1.0.0-dev"),
        "1.0.0-beta.1",
        "empty PRISM_BUILD_VERSION must be treated as absent (F-VID-P1-MED-001)"
    );
    assert_eq!(
        resolve_prism_version(Some("   "), false, None, "1.0.0-dev"),
        "1.0.0-dev",
        "whitespace-only PRISM_BUILD_VERSION must be treated as absent (F-VID-P1-MED-001)"
    );

    // (e) Single-v strip: "vv1.0.0" → "v1.0.0" (strip_prefix not trim_start_matches).
    assert_eq!(
        resolve_prism_version(None, true, Some("vv1.0.0"), "1.0.0-dev"),
        "v1.0.0",
        "strip_prefix removes exactly one v; 'vv1.0.0' → 'v1.0.0' not '1.0.0' (F-VID-P1-LOW-001)"
    );

    // ---- (3) CARGO_PKG_VERSION ADR-064 D1 invariant ----
    // B-3 fix (S-REL-VERSION-IDENTITY review-cycle-2): replaced literal "1.0.0-dev" equality
    // with the ADR-064 D1 invariant — CARGO_PKG_VERSION must be X.Y.Z (stable) or X.Y.Z-dev.
    // Literal equality would fail on the release-prep.yml stable-release PR where Cargo.toml
    // is bumped to X.Y.Z and CI runs the full suite on the resulting `on: push:` trigger.
    {
        let cargo_version = env!("CARGO_PKG_VERSION");
        let parsed = semver::Version::parse(cargo_version).unwrap_or_else(|e| {
            panic!(
                "CARGO_PKG_VERSION '{}' is not valid semver: {}",
                cargo_version, e
            )
        });
        let pre = &parsed.pre;
        assert!(
            pre.is_empty() || pre.as_str() == "dev",
            "CARGO_PKG_VERSION '{}' has pre-release segment '{}' — must be empty (stable) or 'dev' (develop channel). \
             Channel suffixes like rc.N, beta.N, alpha.N must only appear in PRISM_VERSION via build.rs tag injection.",
            cargo_version,
            pre
        );
    }
}

/// RG-003 (S-REL-BVERSION-INJECT-001)
///
/// Source-level assertion: no `CARGO_PKG_VERSION` reference remains in any
/// `crates/prism-bin/src/` file after the six-site migration.
///
/// **RED before implementation:** `main.rs`, `cli.rs`, `boot.rs`,
/// `spec_driven_adapter.rs` all reference `CARGO_PKG_VERSION` — assertion fails
/// on the first file containing it.
/// **GREEN after implementation:** all six sites migrated to `PRISM_VERSION`
/// per ADR-064 D2 (S-REL-BVERSION-INJECT-001 AC-003).
///
/// Authority: ADR-064 D2 six-site table, S-REL-BVERSION-INJECT-001 AC-003.
#[test]
fn test_prism_version_site_version_subcommand() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // All four source files that carry CARGO_PKG_VERSION sites per ADR-064 D2.
    for rel_path in &[
        "src/main.rs",
        "src/cli.rs",
        "src/boot.rs",
        "src/spec_driven_adapter.rs",
    ] {
        let path = format!("{manifest_dir}/{rel_path}");
        let content =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("could not read {path}: {e}"));
        // B-6 (S-REL-VERSION-IDENTITY review-cycle-2): content grep; a doc-comment mentioning
        // CARGO_PKG_VERSION (e.g. "// Previously: CARGO_PKG_VERSION, now using PRISM_VERSION")
        // would cause a false-fail here.  If this starts failing unexpectedly, check for new
        // doc-comments referencing the old env var before assuming the migration regressed.
        assert!(
            !content.contains("CARGO_PKG_VERSION"),
            "prism-bin/{rel_path} must not reference CARGO_PKG_VERSION after \
             S-REL-BVERSION-INJECT-001 six-site migration (ADR-064 D2). \
             Migrate this site to env!(\"PRISM_VERSION\") per ADR-064 D2 site table."
        );
    }
}

/// RG-004 (S-REL-BVERSION-INJECT-001)
///
/// Source-level assertion: the `build_http_client_with_timeout` user-agent in
/// `boot.rs` (2 call sites) and the spec-driven adapter user-agent in
/// `spec_driven_adapter.rs` (1 call site) no longer use `CARGO_PKG_VERSION`.
///
/// **RED before implementation:** both files contain
/// `concat!("prism/", env!("CARGO_PKG_VERSION"))` — assertion fails.
/// **GREEN after implementation:** sites migrated to
/// `concat!("prism/", env!("PRISM_VERSION"))` per ADR-064 D2.
///
/// Authority: ADR-064 D2 user-agent sites, S-REL-BVERSION-INJECT-001 AC-003.
#[test]
fn test_prism_version_site_user_agent() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let boot_rs = std::fs::read_to_string(format!("{manifest_dir}/src/boot.rs"))
        .expect("could not read crates/prism-bin/src/boot.rs");
    let spec_driven = std::fs::read_to_string(format!("{manifest_dir}/src/spec_driven_adapter.rs"))
        .expect("could not read crates/prism-bin/src/spec_driven_adapter.rs");

    // boot.rs: 2 user-agent call sites in build_http_client_with_timeout.
    // B-6 (S-REL-VERSION-IDENTITY review-cycle-2): content grep; a doc-comment mentioning
    // CARGO_PKG_VERSION would cause a false-fail.  If this starts failing unexpectedly,
    // check for new doc-comments referencing the old env var before assuming regression.
    assert!(
        !boot_rs.contains(r#"concat!("prism/", env!("CARGO_PKG_VERSION"))"#),
        "boot.rs user-agent still uses CARGO_PKG_VERSION; migrate to PRISM_VERSION per \
         ADR-064 D2 (S-REL-BVERSION-INJECT-001 AC-003). Both call sites in \
         build_http_client_with_timeout must be updated."
    );

    // spec_driven_adapter.rs: 1 user-agent call site in spec-driven fetch builder.
    // B-6 (S-REL-VERSION-IDENTITY review-cycle-2): same content-grep caveat as boot.rs above.
    assert!(
        !spec_driven.contains(r#"concat!("prism/", env!("CARGO_PKG_VERSION"))"#),
        "spec_driven_adapter.rs user-agent still uses CARGO_PKG_VERSION; migrate to \
         PRISM_VERSION per ADR-064 D2 (S-REL-BVERSION-INJECT-001 AC-003)."
    );
}

// ============================================================================
// S-REL-VERSION-IDENTITY pass-2 HIGH-1 — shared resolver exercised tests
// These tests use the IMPORTED `resolve_prism_version` from
// `prism_bin::version_resolver` — the exact same source that `build.rs` uses
// via `include!("src/version_resolver.rs")`.  Mutating the `is_tag_build` gate
// in `version_resolver.rs` causes `test_shared_resolver_non_tag_ref_must_not_leak_branch_name`
// to fail.
// ============================================================================

/// AC-005 (S-REL-BVERSION-INJECT-001): on a non-tag build, `PRISM_VERSION` equals
/// `CARGO_PKG_VERSION`.  On a tag build, `PRISM_VERSION` is the stripped tag — valid
/// semver but not equal to `CARGO_PKG_VERSION` ("1.0.0-dev").
///
/// `PRISM_VERSION_IS_TAG_BUILD` is emitted by `build.rs` alongside `PRISM_VERSION`,
/// allowing this assertion to be conditional on the actual build context.
/// `ci.yml` has an unfiltered `on: push:` that fires on tag refs too, so a literal
/// `"1.0.0-dev"` equality assertion would fail on the `v1.0.0-beta.1` tag push
/// across all 6 CI legs (B-1, S-REL-VERSION-IDENTITY review-cycle-1).
///
/// Authority: ADR-064 D2 v1.5, S-REL-BVERSION-INJECT-001 AC-005.
#[test]
fn test_prism_version_equals_cargo_pkg_version_on_non_tag_build() {
    // On local dev and non-tag CI builds, PRISM_VERSION must equal CARGO_PKG_VERSION.
    // PRISM_VERSION_IS_TAG_BUILD is emitted by build.rs alongside PRISM_VERSION,
    // allowing this assertion to be conditional on the actual build context.
    // ci.yml has an unfiltered `on: push:` that fires on tag refs, so a literal
    // "1.0.0-dev" assertion would fail on the beta.1 tag push (6 CI legs).
    if env!("PRISM_VERSION_IS_TAG_BUILD") == "false" {
        assert_eq!(
            env!("PRISM_VERSION"),
            env!("CARGO_PKG_VERSION"),
            "On non-tag builds, PRISM_VERSION must equal CARGO_PKG_VERSION \
             (fallback chain step 3, AC-005, ADR-064 D2)"
        );
    } else {
        // Tag build: PRISM_VERSION is the stripped tag, not CARGO_PKG_VERSION.
        // Assert it is valid semver and not a leaked branch ref.
        let v = env!("PRISM_VERSION");
        assert!(
            semver::Version::parse(v).is_ok(),
            "On tag builds, PRISM_VERSION must be valid semver; got: {v}"
        );
        assert!(!v.contains('/'), "leaked branch ref in PRISM_VERSION: {v}");
    }
}

/// Load-bearing gate (HIGH-1): non-tag refs must NOT leak branch name into the binary.
///
/// This test is the KEY load-bearing test.  If the `is_tag_build &&` guard is removed
/// from `version_resolver.rs`, `resolve_prism_version(None, false, Some("develop"), …)`
/// returns `"develop"` instead of `"1.0.0-dev"` and this test FAILS.
///
/// Exercises the SHARED resolver directly, so the failure propagates to build.rs
/// (which includes the same source via `include!`).
///
/// Authority: F-VID-P1-CRIT-001, ADR-064 D2 v1.5, HIGH-1.
#[test]
fn test_shared_resolver_non_tag_ref_must_not_leak_branch_name() {
    // (a) "develop" branch on non-tag build → CARGO_PKG_VERSION, not branch name.
    assert_eq!(
        resolve_prism_version(None, false, Some("develop"), "1.0.0-dev"),
        "1.0.0-dev",
        "SHARED RESOLVER: non-tag ref 'develop' must NOT bake branch name into binary \
         (F-VID-P1-CRIT-001, HIGH-1). Without is_tag_build guard, returns 'develop'."
    );
    // (b) Feature branch with slash.
    assert_eq!(
        resolve_prism_version(None, false, Some("feature/S-3.01"), "1.0.0-dev"),
        "1.0.0-dev",
        "SHARED RESOLVER: feature branch 'feature/S-3.01' must NOT bake into binary \
         (F-VID-P1-CRIT-001)"
    );
    // (c) PR merge ref with slash.
    assert_eq!(
        resolve_prism_version(None, false, Some("260/merge"), "1.0.0-dev"),
        "1.0.0-dev",
        "SHARED RESOLVER: PR merge ref '260/merge' must NOT bake into binary \
         (F-VID-P1-CRIT-001)"
    );
}

/// Tag ref must resolve to stripped version via shared resolver.
///
/// Authority: ADR-064 D2 step-2, S-REL-BVERSION-INJECT-001.
#[test]
fn test_shared_resolver_tag_ref_strips_single_v() {
    // (a) "v1.0.0-beta.1" on tag build → "1.0.0-beta.1".
    assert_eq!(
        resolve_prism_version(None, true, Some("v1.0.0-beta.1"), "1.0.0-dev"),
        "1.0.0-beta.1",
        "SHARED RESOLVER: tag ref 'v1.0.0-beta.1' must resolve to '1.0.0-beta.1' \
         (ADR-064 D2 step-2, single-v strip)"
    );
    // (b) Without leading v — passes through unchanged.
    assert_eq!(
        resolve_prism_version(None, true, Some("1.0.0-beta.1"), "1.0.0-dev"),
        "1.0.0-beta.1",
        "SHARED RESOLVER: tag ref without leading 'v' passes through unchanged"
    );
    // (c) Double-v: "vv1.0.0" → "v1.0.0" (strip_prefix, NOT trim_start_matches).
    assert_eq!(
        resolve_prism_version(None, true, Some("vv1.0.0"), "1.0.0-dev"),
        "v1.0.0",
        "SHARED RESOLVER: 'vv1.0.0' must strip exactly one v → 'v1.0.0' \
         (F-VID-P1-LOW-001: strip_prefix not trim_start_matches)"
    );
}

/// PRISM_BUILD_VERSION precedence and empty/whitespace fall-through via shared resolver.
///
/// Authority: ADR-064 D2 step-1, F-VID-P1-MED-001, S-REL-BVERSION-INJECT-001.
// NOTE: `resolve_is_tag_build` tests appear below this section (F-VID-MED-001, pass-3).
#[test]
fn test_shared_resolver_build_version_precedence_and_empty_fallthrough() {
    // (a) PRISM_BUILD_VERSION wins over GITHUB_REF_NAME on tag build.
    assert_eq!(
        resolve_prism_version(
            Some("custom-override"),
            true,
            Some("v1.0.0-beta.1"),
            "1.0.0-dev"
        ),
        "custom-override",
        "SHARED RESOLVER: PRISM_BUILD_VERSION must win over everything (chain step 1)"
    );
    // (b) PRISM_BUILD_VERSION wins even on non-tag build.
    assert_eq!(
        resolve_prism_version(Some("1.0.0-custom"), false, Some("develop"), "1.0.0-dev"),
        "1.0.0-custom",
        "SHARED RESOLVER: PRISM_BUILD_VERSION wins over branch name on non-tag build"
    );
    // (c) Set-but-empty PRISM_BUILD_VERSION falls through (F-VID-P1-MED-001).
    assert_eq!(
        resolve_prism_version(Some(""), true, Some("v1.0.0-beta.1"), "1.0.0-dev"),
        "1.0.0-beta.1",
        "SHARED RESOLVER: empty PRISM_BUILD_VERSION treated as absent (F-VID-P1-MED-001)"
    );
    // (d) Whitespace-only PRISM_BUILD_VERSION falls through.
    assert_eq!(
        resolve_prism_version(Some("   "), false, None, "1.0.0-dev"),
        "1.0.0-dev",
        "SHARED RESOLVER: whitespace-only PRISM_BUILD_VERSION treated as absent (F-VID-P1-MED-001)"
    );
    // (e) Set-but-empty GITHUB_REF_NAME on tag build falls through to CARGO_PKG_VERSION.
    assert_eq!(
        resolve_prism_version(None, true, Some(""), "1.0.0-dev"),
        "1.0.0-dev",
        "SHARED RESOLVER: empty GITHUB_REF_NAME must fall through to CARGO_PKG_VERSION"
    );
}

/// OBS-1 (S-REL-VERSION-IDENTITY pass-5): degenerate tag `ref_name == "v"` must fall
/// through to CARGO_PKG_VERSION, NOT return an empty PRISM_VERSION.
///
/// Without the post-strip empty guard introduced in pass-5, `strip_prefix('v')` on `"v"`
/// yields `Some("")` → `unwrap_or(name)` returns `""` → PRISM_VERSION="" (violates ADR-064
/// D2 never-empty promise). The guard checks `version.trim().is_empty()` after strip and
/// falls through to `cargo_version` when empty.
///
/// Authority: ADR-064 D2 v1.6 never-empty invariant, S-REL-VERSION-IDENTITY pass-5 OBS-1.
#[test]
fn test_shared_resolver_degenerate_v_only_tag_falls_through() {
    // The primary case from OBS-1: ref_name="v" post-strip yields "" → CARGO_PKG_VERSION.
    assert_eq!(
        resolve_prism_version(None, true, Some("v"), "1.0.0-dev"),
        "1.0.0-dev",
        "SHARED RESOLVER: degenerate tag 'v' must fall through to CARGO_PKG_VERSION, \
         not emit empty PRISM_VERSION (OBS-1, ADR-064 D2 v1.6)"
    );
    // Confirm normal tags still work: "v1.0.0-beta.1" → "1.0.0-beta.1" (regression guard).
    assert_eq!(
        resolve_prism_version(None, true, Some("v1.0.0-beta.1"), "1.0.0-dev"),
        "1.0.0-beta.1",
        "SHARED RESOLVER: normal tag 'v1.0.0-beta.1' must still resolve correctly after OBS-1 fix"
    );
    // Confirm "1.0.0" (no leading v) still works (regression guard).
    assert_eq!(
        resolve_prism_version(None, true, Some("1.0.0"), "1.0.0-dev"),
        "1.0.0",
        "SHARED RESOLVER: tag without leading 'v' must still pass through unchanged after OBS-1 fix"
    );
}

// ============================================================================
// S-REL-VERSION-IDENTITY pass-3 F-VID-MED-001 — resolve_is_tag_build tests
//
// These tests exercise `resolve_is_tag_build` from `prism_bin::version_resolver`
// — the SAME function that `build.rs` calls after the MED-001 extraction.
// Mutating the "tag" comparison (e.g. to "branch") causes
// `test_is_tag_build_ref_type_tag` to fail; removing the refs/tags/ fallback
// causes `test_is_tag_build_ref_val_fallback` to fail.
// ============================================================================

/// F-VID-MED-001 (pass-3): GITHUB_REF_TYPE="tag" → true.
/// Mutating `== "tag"` to `== "branch"` causes this test to fail.
///
/// Authority: ADR-064 D2 v1.5, S-REL-VERSION-IDENTITY pass-3 F-VID-MED-001.
#[test]
fn test_is_tag_build_ref_type_tag() {
    assert!(
        resolve_is_tag_build(Some("tag"), None),
        "GITHUB_REF_TYPE='tag' must yield is_tag_build=true (F-VID-MED-001)"
    );
}

/// F-VID-MED-001 (pass-3): GITHUB_REF_TYPE="branch" → false (short-circuit, no fallback).
/// Load-bearing: without short-circuit, passing ref_val="refs/tags/v1.0.0" here would
/// yield true instead of the correct false.
///
/// Authority: ADR-064 D2 v1.5, S-REL-VERSION-IDENTITY pass-3 F-VID-MED-001.
#[test]
fn test_is_tag_build_ref_type_branch_is_false() {
    // ref_type present and non-empty → short-circuit; ref_val not consulted.
    assert!(
        !resolve_is_tag_build(Some("branch"), None),
        "GITHUB_REF_TYPE='branch' must yield is_tag_build=false (F-VID-MED-001)"
    );
    // Even with a tag-looking ref_val, ref_type="branch" still returns false.
    assert!(
        !resolve_is_tag_build(Some("branch"), Some("refs/tags/v1.0.0")),
        "GITHUB_REF_TYPE='branch' must short-circuit to false even with tag-looking GITHUB_REF"
    );
}

/// F-VID-MED-001 (pass-3): empty and whitespace GITHUB_REF_TYPE treated as absent,
/// falling through to the GITHUB_REF arm.
///
/// Authority: ADR-064 D2 v1.5, S-REL-VERSION-IDENTITY pass-3 F-VID-MED-001.
#[test]
fn test_is_tag_build_empty_ref_type_falls_through() {
    // Empty string → treated as absent → check ref_val (None here) → false.
    assert!(
        !resolve_is_tag_build(Some(""), None),
        "empty GITHUB_REF_TYPE must be treated as absent (F-VID-MED-001)"
    );
    // Whitespace-only → treated as absent → check ref_val (None here) → false.
    assert!(
        !resolve_is_tag_build(Some("  "), None),
        "whitespace-only GITHUB_REF_TYPE must be treated as absent (F-VID-MED-001)"
    );
}

/// F-VID-MED-001 (pass-3): GITHUB_REF fallback when GITHUB_REF_TYPE is absent.
/// Mutating `starts_with("refs/tags/")` causes `_true` case to fail.
///
/// Authority: ADR-064 D2 v1.5, S-REL-VERSION-IDENTITY pass-3 F-VID-MED-001.
#[test]
fn test_is_tag_build_ref_val_fallback() {
    // ref_type absent, ref_val is a tag ref → true.
    assert!(
        resolve_is_tag_build(None, Some("refs/tags/v1.0.0-beta.1")),
        "GITHUB_REF='refs/tags/v1.0.0-beta.1' with absent ref_type must yield true (F-VID-MED-001)"
    );
    // ref_type absent, ref_val is a branch ref → false.
    assert!(
        !resolve_is_tag_build(None, Some("refs/heads/develop")),
        "GITHUB_REF='refs/heads/develop' with absent ref_type must yield false (F-VID-MED-001)"
    );
    // ref_type absent, ref_val also absent → false.
    assert!(
        !resolve_is_tag_build(None, None),
        "both absent must yield false (F-VID-MED-001)"
    );
    // ref_type absent, ref_val is empty → treated as absent → false.
    assert!(
        !resolve_is_tag_build(None, Some("")),
        "empty GITHUB_REF with absent ref_type must yield false (F-VID-MED-001)"
    );
}
