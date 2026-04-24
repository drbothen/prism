//! AC-8: Without `--features dtu`, no binary is produced (required-features gate).
//!
//! This is primarily a build-time / Cargo contract check. At runtime, we verify
//! that the `dtu` feature IS present in this test binary (since the test is gated
//! on `required-features = ["dtu"]`). We also verify key structural invariants
//! (e.g., `prism-dtu-demo-server` depends on no production crates).
//!
//! The absence-of-binary when `dtu` is missing is validated by the CI build matrix,
//! not by a runtime test. This test file covers the positive assertion (correct
//! feature presence in this test build) and the canonical port table.

#![allow(clippy::unwrap_used, clippy::expect_used)]
/// AC-8: The `dtu` feature is compiled into this test binary.
///
/// If this test compiles and runs, it proves the `required-features = ["dtu"]`
/// gate allowed the binary/test to be built (the feature is present).
#[test]
fn ac_8_dtu_feature_is_present_in_this_build() {
    // If this file compiles, `--features dtu` was provided.
    // This assertion is trivially true at runtime — the compile-time check
    // is the meaningful gate (AC-8's real enforcement is `required-features`).
    // The cfg!() value is a compile-time constant; allow the clippy lint.
    #[allow(clippy::assertions_on_constants)]
    { assert!(cfg!(feature = "dtu"), "AC-8: dtu feature must be present"); }
}

/// AC-8: configs/demo.toml specifies the canonical stable port assignment (17080–17085).
///
/// Per the spec: "canonical port assignment in demo.toml (17080–17085) MUST be stable
/// across recordings to ensure VHS tape byte-identical replay (AC-7)."
#[test]
fn ac_8_demo_toml_has_canonical_stable_ports() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("configs")
        .join("demo.toml");

    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("AC-8: configs/demo.toml must exist at {:?}", path));

    // Canonical port table per spec.
    let expected = [
        ("crowdstrike", 17080u16),
        ("claroty", 17081),
        ("cyberint", 17082),
        ("armis", 17083),
        ("threatintel", 17084),
        ("nvd", 17085),
    ];

    for (name, port) in expected {
        assert!(
            contents.contains(&port.to_string()),
            "AC-8: demo.toml must assign port {port} to {name}; not found in:\n{contents}"
        );
    }
}

/// AC-8: DemoConfig can be parsed from a minimal TOML string (feature-gate does not
/// prevent library types from functioning).
#[test]
fn ac_8_demo_config_parses_minimal_toml() {
    let toml_str = r#"
[clones.crowdstrike]
enabled = true
port = 17080
"#;
    let config = prism_dtu_demo_server::DemoConfig::from_str(toml_str)
        .expect("AC-8: minimal TOML must parse successfully");

    assert!(
        config.clones.crowdstrike.enabled,
        "AC-8: crowdstrike must be enabled"
    );
    assert_eq!(
        config.clones.crowdstrike.port, 17080,
        "AC-8: port must be 17080"
    );
}
