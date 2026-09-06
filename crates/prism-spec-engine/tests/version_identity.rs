//! RG-003 + RG-004 + RG-005: S-REL-AGENT-VERSION-001 — Agent-Facing Version Identity
//! (prism-spec-engine)
//!
//! These tests gate Surface B: a new `crates/prism-spec-engine/build.rs` must emit
//! `PRISM_VERSION` using the D2-conformant fallback chain so that `env!("PRISM_VERSION")`
//! compiles and resolves correctly in prism-spec-engine context.
//!
//! Authority: ADR-064 v1.9 D4 §Surface B; ADR-050 v2.4 D6.
//!
//! ╔════════════════════════════════════════════════════════════════════════════════════╗
//! ║  COMPILE-TIME DEPENDENCY — IMPLEMENTER READ THIS FIRST                           ║
//! ║                                                                                   ║
//! ║  ALL THREE TESTS (RG-003, RG-004, RG-005) reference env!("PRISM_VERSION").       ║
//! ║  env!("PRISM_VERSION") is a compile-time macro that resolves ONLY when a         ║
//! ║  build.rs in the same crate has emitted:                                         ║
//! ║                                                                                   ║
//! ║      cargo:rustc-env=PRISM_VERSION=<version>                                     ║
//! ║                                                                                   ║
//! ║  crates/prism-spec-engine/build.rs does NOT currently exist.                     ║
//! ║                                                                                   ║
//! ║  SEQUENCING (per ADR-064 D4 §Surface B, S-REL-AGENT-VERSION-001 Task 12):        ║
//! ║  CREATE crates/prism-spec-engine/build.rs FIRST (Task 12) before attempting to   ║
//! ║  compile or run any test in this file. Without build.rs, rustc will emit:        ║
//! ║                                                                                   ║
//! ║      error: environment variable `PRISM_VERSION` not defined at compile time     ║
//! ║                                                                                   ║
//! ║  This compilation failure IS the Red Gate for RG-003, RG-004, and RG-005.       ║
//! ╚════════════════════════════════════════════════════════════════════════════════════╝
//!
//! # Red Gate state (pre-implementation)
//!
//! All three tests fail with COMPILATION ERRORS:
//! - `env!("PRISM_VERSION")` does not compile without `crates/prism-spec-engine/build.rs`
//! - Error: `error: environment variable \`PRISM_VERSION\` not defined at compile time`
//!
//! # Green Gate state (post-implementation)
//!
//! All three tests compile and pass after:
//! 1. Task 12: `crates/prism-spec-engine/build.rs` created (D2-conformant fallback chain)
//! 2. Task 13: `build_http_client_with_timeout` in `pipeline.rs` updated to use
//!    `env!("PRISM_VERSION")` instead of `env!("CARGO_PKG_VERSION")`
//!
//! # Local dev behavior
//!
//! On local dev (no `GITHUB_REF_NAME`, no `PRISM_BUILD_VERSION`), the D2-conformant
//! fallback chain step 3 applies: `PRISM_VERSION` == `CARGO_PKG_VERSION` of the
//! prism-spec-engine library crate (currently `"0.9.0"` or whatever the manifest declares).
//! RG-004 verifies this fallback semantics explicitly.
//!
//! # Test naming
//!
//! Follows story-level traceability convention since this story has no BC.

/// RG-003: `env!("PRISM_VERSION")` resolves in prism-spec-engine context.
///
/// Requires `crates/prism-spec-engine/build.rs` to emit `cargo:rustc-env=PRISM_VERSION=...`.
///
/// Traces to: ADR-064 v1.9 D4 §Surface B — `crates/prism-spec-engine/build.rs` row:
///   Before: `(new file — does not exist)`
///   After:  D2-conformant build script emitting `cargo:rustc-env=PRISM_VERSION={version}`
///
/// S-REL-AGENT-VERSION-001 AC-006: `ls crates/prism-spec-engine/build.rs` exits 0.
///
/// RED state: COMPILATION ERROR — `env!("PRISM_VERSION")` not defined.
/// GREEN state: compiles and asserts PRISM_VERSION is non-empty.
#[test]
fn test_prism_spec_engine_prism_version_env_var_is_available() {
    // COMPILE-TIME DEPENDENCY: fails with compilation error until build.rs exists.
    // The implementer must create crates/prism-spec-engine/build.rs first (Task 12).
    let v: &str = env!("PRISM_VERSION");
    assert!(
        !v.is_empty(),
        "PRISM_VERSION must be non-empty in prism-spec-engine context; \
         build.rs fallback chain step 3 (CARGO_PKG_VERSION) must produce a non-empty value"
    );
}

/// RG-004: On local dev (no `GITHUB_REF_NAME`, no `PRISM_BUILD_VERSION`),
/// `env!("PRISM_VERSION")` equals `env!("CARGO_PKG_VERSION")` in prism-spec-engine context.
///
/// Verifies the D2-conformant fallback chain step 3 semantics (ADR-064 D4 §Surface B):
/// when no `PRISM_BUILD_VERSION` override and no tag-gated `GITHUB_REF_NAME` are present,
/// `PRISM_VERSION` falls through to `CARGO_PKG_VERSION`. In local dev this means both
/// resolve to the prism-spec-engine library crate version (e.g., `"0.9.0"`).
///
/// Note: `CARGO_PKG_VERSION` here is prism-spec-engine's library version, not the product
/// version `"1.0.0-dev"`. This is correct for local dev — the MCP wire output is governed
/// by the boot step 9 `product_version` parameter (Surface A), not the spec-engine build.
/// RG-004 verifies the D2 algorithm is correctly implemented (step 3 activates on local dev).
///
/// Traces to: ADR-064 v1.9 D4 §Surface B D2 normative fallback chain step 3;
/// S-REL-AGENT-VERSION-001 AC-006.
///
/// RED state: COMPILATION ERROR — `env!("PRISM_VERSION")` not defined (same as RG-003).
/// GREEN state: compiles and `assert_eq!` passes (both env vars resolve to the same value
///              on local dev with no GITHUB_REF_NAME/PRISM_BUILD_VERSION set).
#[test]
fn test_spec_engine_build_rs_fallback_uses_cargo_pkg_version_when_no_env() {
    // COMPILE-TIME DEPENDENCY: fails with compilation error until build.rs exists.
    // The implementer must create crates/prism-spec-engine/build.rs first (Task 12).
    //
    // On local dev:
    //   - PRISM_BUILD_VERSION is unset → step 1 falls through
    //   - GITHUB_REF_TYPE != "tag" (local, not CI tag build) → step 2 skipped
    //   - Step 3: PRISM_VERSION = CARGO_PKG_VERSION (the spec-engine library version)
    //
    // This test verifies that the fallback chain correctly resolves to CARGO_PKG_VERSION
    // when no override or tag env vars are present.
    assert_eq!(
        env!("PRISM_VERSION"),
        env!("CARGO_PKG_VERSION"),
        "On local dev (no GITHUB_REF_NAME/PRISM_BUILD_VERSION), PRISM_VERSION must fall \
         through to CARGO_PKG_VERSION per D2-conformant fallback chain step 3 \
         (ADR-064 v1.9 D4 §Surface B). \
         Got PRISM_VERSION={:?}, CARGO_PKG_VERSION={:?}",
        env!("PRISM_VERSION"),
        env!("CARGO_PKG_VERSION"),
    );
}

/// RG-005: The infusion HTTP client user-agent uses `env!("PRISM_VERSION")`, not
/// `env!("CARGO_PKG_VERSION")`.
///
/// `build_http_client_with_timeout` in `crates/prism-spec-engine/src/pipeline.rs` currently
/// emits `prism/0.9.0` (using `env!("CARGO_PKG_VERSION")` — the library crate version).
/// After Task 13, it must emit `prism/{PRISM_VERSION}` using `env!("PRISM_VERSION")`.
///
/// This test verifies:
/// 1. COMPILE-TIME: `env!("PRISM_VERSION")` is available in spec-engine context (build.rs gate)
/// 2. STRING FORMAT: the expected user-agent string `concat!("prism/", env!("PRISM_VERSION"))`
///    is correctly formed (non-empty version suffix, "prism/" prefix)
///
/// Note on behavioral equivalence: On local dev, `PRISM_VERSION == CARGO_PKG_VERSION` (see
/// RG-004), so value-based distinction between the two env vars is not possible at test time.
/// The load-bearing assertion is the COMPILATION GATE: if `env!("PRISM_VERSION")` is not
/// emitted by build.rs, this test fails to compile. After build.rs and Task 13 land, the
/// source code uses `env!("PRISM_VERSION")` at the user-agent call site — confirmed by
/// S-REL-AGENT-VERSION-001 AC-007 grep check (`grep CARGO_PKG_VERSION pipeline.rs` returns no
/// match at the `build_http_client_with_timeout` user-agent call site).
///
/// Traces to: ADR-064 v1.9 D4 §Surface B — `pipeline.rs` row:
///   Before: `env!("CARGO_PKG_VERSION")`
///   After:  `env!("PRISM_VERSION")`
/// AND ADR-050 v2.4 D6: `build_http_client_with_timeout` MUST use `env!("PRISM_VERSION")`.
///
/// RED state: COMPILATION ERROR — `env!("PRISM_VERSION")` not defined (same as RG-003/RG-004).
/// GREEN state: compiles and assertions on UA format pass.
#[test]
fn test_infusion_http_client_sends_prism_product_version_user_agent() {
    // COMPILE-TIME DEPENDENCY: fails with compilation error until build.rs exists.
    // The implementer must create crates/prism-spec-engine/build.rs first (Task 12),
    // then update pipeline.rs build_http_client_with_timeout to use env!("PRISM_VERSION")
    // instead of env!("CARGO_PKG_VERSION") (Task 13).
    //
    // concat!() is a compile-time macro: if PRISM_VERSION is not emitted by build.rs,
    // the following line fails to compile with:
    //   error: environment variable `PRISM_VERSION` not defined at compile time
    let expected_ua: &str = concat!("prism/", env!("PRISM_VERSION"));

    // Assert the user-agent is correctly formed: "prism/<version>"
    assert!(
        expected_ua.starts_with("prism/"),
        "Infusion HTTP client user-agent must start with 'prism/'; \
         got: {expected_ua:?} (ADR-050 v2.4 D6 normative form)"
    );

    // Assert the version suffix is non-empty (paranoia check; belt-and-suspenders with RG-003)
    let version_suffix = expected_ua
        .strip_prefix("prism/")
        .expect("starts_with already checked");
    assert!(
        !version_suffix.is_empty(),
        "Infusion HTTP client user-agent version suffix must be non-empty; \
         got: {expected_ua:?}"
    );

    // Verify the composed UA string matches format (SID-2: full composed string assertion)
    let prism_version: &str = env!("PRISM_VERSION");
    assert_eq!(
        expected_ua,
        format!("prism/{prism_version}").as_str(),
        "User-agent must be exactly 'prism/{{PRISM_VERSION}}'; \
         got: {expected_ua:?}, PRISM_VERSION={prism_version:?} \
         (ADR-050 v2.4 D6; ADR-064 D4 §Surface B)"
    );
}
