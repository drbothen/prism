//! AC-6: configs/prism-demo.toml exists, uses bare-name credential_ref, and sensor
//! URLs point to the demo server ports (17080–17085).
//!
//! This test is a structural/file-content test that does NOT spin up the harness.
//! It validates the committed config artifact per AC-6 requirements.
//!
//! Was Red Gate at implementation start; config file now exists.
//! directory was created; file content is part of Phase 2 implementation).

/// AC-6: configs/prism-demo.toml must exist under the crate directory.
#[test]
fn ac_6_prism_demo_toml_exists() {
    // The spec requires: `configs/prism-demo.toml` committed under
    // `crates/prism-dtu-demo-server/configs/prism-demo.toml`.
    // This test fails until the file is created by the implementer.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("configs")
        .join("prism-demo.toml");

    assert!(
        path.exists(),
        "AC-6: configs/prism-demo.toml must exist at {:?}",
        path
    );
}

/// AC-6: prism-demo.toml must use bare-name credential_ref (no `env:` prefix).
#[test]
fn ac_6_prism_demo_toml_uses_bare_name_credential_ref() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("configs")
        .join("prism-demo.toml");

    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("AC-6: could not read {:?}", path));

    // Must contain DEMO_FAKE_* credential refs.
    assert!(
        contents.contains("DEMO_FAKE_"),
        "AC-6: prism-demo.toml must reference DEMO_FAKE_* credential tokens; file was:\n{contents}"
    );

    // Must NOT use `env:` scheme prefix (bare-name convention per S-5.05 Task 3).
    assert!(
        !contents.contains("env:DEMO_FAKE_"),
        "AC-6: credential_ref must use bare-name (no `env:` prefix); found `env:` scheme in:\n{contents}"
    );
}

/// AC-6: prism-demo.toml must reference all 6 clone sensor URLs (ports 17080–17085).
#[test]
fn ac_6_prism_demo_toml_contains_all_six_clone_ports() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("configs")
        .join("prism-demo.toml");

    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("AC-6: could not read {:?}", path));

    let expected_ports = [17080u16, 17081, 17082, 17083, 17084, 17085];
    for port in expected_ports {
        assert!(
            contents.contains(&port.to_string()),
            "AC-6: prism-demo.toml must reference port {port} for one of the 6 clones"
        );
    }
}
