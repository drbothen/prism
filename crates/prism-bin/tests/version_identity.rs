// Version identity Red Gate tests
// S-REL-DEV-RESET-001: RG-001, RG-002  (ADR-064 D1 — develop carries 1.0.0-dev)
// S-REL-BVERSION-INJECT-001: RG-001..RG-004  (ADR-064 D2 — build.rs PRISM_VERSION injection)
//
// All tests are written to FAIL before implementation.
// S-REL-BVERSION-INJECT-001 RG-001..RG-004 use the source-level / pure-helper approach
// (per orchestrator guidance: avoid compile-fail tests that block the whole crate from building
// and therefore also block verification of RG-001/RG-002 above).  The documented red-failure
// mode for each test is stated in its doc-comment.

// ============================================================================
// S-REL-DEV-RESET-001
// ============================================================================

/// RG-001 (S-REL-DEV-RESET-001)
///
/// Assert that `prism-bin/Cargo.toml` `version` is `"1.0.0-dev"`.
///
/// **RED before implementation:** `CARGO_PKG_VERSION` == `"1.0.0-rc.1"` — assertion fails.
/// **GREEN after implementation:** Cargo.toml reset to `1.0.0-dev` per ADR-064 D1.
///
/// Authority: ADR-064 D1, S-REL-DEV-RESET-001 AC-001, RELEASE-CHANNELS.md §3.
#[test]
fn test_prism_bin_cargo_toml_version_is_dev() {
    assert_eq!(
        env!("CARGO_PKG_VERSION"),
        "1.0.0-dev",
        "prism-bin Cargo.toml must be reset to 1.0.0-dev per ADR-064 D1 \
         (S-REL-DEV-RESET-001 AC-001). Current value indicates the Cargo.toml \
         edit has not yet been applied."
    );
}

/// RG-002 (S-REL-DEV-RESET-001)
///
/// Assert that `tests/external/non-exhaustive-violation/Cargo.lock` records
/// `prism-bin` at version `"1.0.0-dev"`.
///
/// **RED before implementation:** the nested workspace Cargo.lock still pins `"1.0.0-rc.1"`.
/// **GREEN after implementation:** `cargo update -p prism-bin` regenerates both lockfiles
/// per S-REL-DEV-RESET-001 AC-003.
///
/// Authority: ADR-064 D1 ownership map, S-REL-DEV-RESET-001 AC-003.
#[test]
fn test_non_exhaustive_lockfile_version_matches() {
    let lock = include_str!("../../../tests/external/non-exhaustive-violation/Cargo.lock");
    let idx = lock
        .find(r#"name = "prism-bin""#)
        .expect("prism-bin entry not found in non-exhaustive-violation/Cargo.lock");
    let tail = &lock[idx..];
    assert!(
        tail.contains(r#"version = "1.0.0-dev""#),
        "tests/external/non-exhaustive-violation/Cargo.lock still pins prism-bin to an \
         old version. Run: cd tests/external/non-exhaustive-violation && cargo update -p prism-bin \
         (S-REL-DEV-RESET-001 AC-003)"
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
        "build.rs must check PRISM_BUILD_VERSION (chain step 1 per ADR-064 D2)"
    );
    assert!(
        content.contains("GITHUB_REF_NAME"),
        "build.rs must check GITHUB_REF_NAME (chain step 2 per ADR-064 D2)"
    );
    assert!(
        content.contains("CARGO_PKG_VERSION"),
        "build.rs must fall back to CARGO_PKG_VERSION (chain step 3 per ADR-064 D2)"
    );

    // ---- (2) Pure helper mirrors ADR-064 D2 normative fallback logic ----
    /// Mirror of the ADR-064 D2 normative fallback chain.
    /// Used to verify chain semantics independently of build infrastructure.
    fn resolve_prism_version<'a>(
        build_version: Option<&'a str>,
        ref_name: Option<&'a str>,
        cargo_version: &'a str,
    ) -> &'a str {
        if let Some(v) = build_version {
            v
        } else if let Some(r) = ref_name {
            r.trim_start_matches('v')
        } else {
            cargo_version
        }
    }

    // Local-dev path: no env vars → CARGO_PKG_VERSION.
    assert_eq!(
        resolve_prism_version(None, None, "1.0.0-dev"),
        "1.0.0-dev",
        "fallback chain must return CARGO_PKG_VERSION when no overrides set (ADR-064 D2 step 3)"
    );

    // GITHUB_REF_NAME path: must strip leading 'v' (ADR-064 D2 step 2).
    assert_eq!(
        resolve_prism_version(None, Some("v1.0.0-beta.1"), "1.0.0-dev"),
        "1.0.0-beta.1",
        "GITHUB_REF_NAME must have leading 'v' stripped (ADR-064 D2 step 2)"
    );
    assert_eq!(
        resolve_prism_version(None, Some("1.0.0-beta.1"), "1.0.0-dev"),
        "1.0.0-beta.1",
        "GITHUB_REF_NAME without leading 'v' must pass through unchanged"
    );

    // PRISM_BUILD_VERSION wins over GITHUB_REF_NAME (ADR-064 D2 step 1).
    assert_eq!(
        resolve_prism_version(Some("custom-override"), Some("v1.0.0-beta.1"), "1.0.0-dev"),
        "custom-override",
        "PRISM_BUILD_VERSION must take priority over GITHUB_REF_NAME (ADR-064 D2 step 1)"
    );

    // ---- (3) CARGO_PKG_VERSION gate (S-REL-DEV-RESET-001 pre-condition) ----
    assert_eq!(
        env!("CARGO_PKG_VERSION"),
        "1.0.0-dev",
        "CARGO_PKG_VERSION must be '1.0.0-dev' after S-REL-DEV-RESET-001 \
         (ADR-064 D1). Local dev fallback path requires this value. \
         Current value shows S-REL-DEV-RESET-001 has not been applied."
    );
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
    assert!(
        !boot_rs.contains(r#"concat!("prism/", env!("CARGO_PKG_VERSION"))"#),
        "boot.rs user-agent still uses CARGO_PKG_VERSION; migrate to PRISM_VERSION per \
         ADR-064 D2 (S-REL-BVERSION-INJECT-001 AC-003). Both call sites in \
         build_http_client_with_timeout must be updated."
    );

    // spec_driven_adapter.rs: 1 user-agent call site in spec-driven fetch builder.
    assert!(
        !spec_driven.contains(r#"concat!("prism/", env!("CARGO_PKG_VERSION"))"#),
        "spec_driven_adapter.rs user-agent still uses CARGO_PKG_VERSION; migrate to \
         PRISM_VERSION per ADR-064 D2 (S-REL-BVERSION-INJECT-001 AC-003)."
    );
}
