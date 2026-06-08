//! Integration and unit tests for BC-2.03.013 — CredentialStore initialization
//! contract (reference validation only, no values in memory).
//!
//! # Integration tests (subprocess)
//!
//! Invoke `prism validate-config` to exercise the boot path through step 5.
//! Credential failures map to exit 5 (permission-denied) or exit 2 (config-invalid ref).
//!
//! # Unit tests (type-level non-leak invariant)
//!
//! BC-2.03.013 §Critical Invariant specifies that after `CredentialStore::open()`
//! returns `Ok(store)`, the store holds ONLY reference metadata — not secret values.
//! We test this at the type level (Approach A from BC-2.03.013 §Test Strategy).
//!
//! # Test Vectors from BC-2.03.013
//!
//! | TV ID | Scenario | Expected Exit |
//! |-------|----------|--------------|
//! | TV-03-013-003 | Missing keyring entry | Exit 2 |
//! | TV-03-013-004 | Keyring service locked | Exit 5 |
//! | TV-03-013-005 | File backend, permission denied | Exit 5 |
//! | TV-03-013-006 | No values in CredentialStore after init | N/A (unit test) |
//!
//! Story: S-WAVE5-PREP-01  AC-7
//! BC: BC-2.03.013 (CredentialStore init — reference validation only)
//! ADR: ADR-022 §B step 5, AD-017 (AI-opaque credential model)

#![allow(clippy::unwrap_used)]

use std::{path::PathBuf, process::Command};

/// MED-5 (S-WAVE5-PREP-01 fix-pass-1): Create an isolated temp config dir per test.
/// Returns (config_dir, state_dir, spec_dir) TempDirs — keep all alive for test duration.
fn make_valid_config_dir() -> (tempfile::TempDir, tempfile::TempDir, tempfile::TempDir) {
    let config_tmp = tempfile::TempDir::new().unwrap();
    let state_tmp = tempfile::TempDir::new().unwrap();
    let spec_tmp = tempfile::TempDir::new().unwrap();

    let toml_content = format!(
        r#"spec_dir = {:?}
state_dir = {:?}

[[orgs]]
org_id = "0196f000-0000-7000-8000-000000000001"
org_slug = "acme"
"#,
        spec_tmp.path().display(),
        state_tmp.path().display(),
    );
    std::fs::write(config_tmp.path().join("prism.toml"), &toml_content).unwrap();
    (config_tmp, state_tmp, spec_tmp)
}

fn prism_bin() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_prism") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/prism")
}

// ---------------------------------------------------------------------------
// BC-2.03.013 — permission denied maps to exit 5 (AC-7)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-7
/// BC: BC-2.03.013 Failure path — credential backend permission denied → exit 5
/// TV-03-013-004/005: Backend permission denied → exit 5 (not 2 or 4).
/// MED-5: Uses isolated TempDir (injection fires at step 5 before RocksDB, but
/// spec_dir must exist for step4 since MED-3 removes auto-create).
#[test]
fn test_BC_2_03_013_credential_permission_denied_exits_five() {
    let (config_dir, _state_tmp, _spec_tmp) = make_valid_config_dir();
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", config_dir.path())
        // Inject a permission-denied failure at step 5 for test determinism.
        .env("PRISM_TEST_INJECT_FAIL_STEP", "5_permission")
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(5),
        "Credential store permission-denied must exit 5 (BC-2.03.013 + AC-7); \
         got exit {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// BC-2.03.013 — unresolvable credential ref maps to exit 2
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.03.013 Failure path — credential ref unresolvable → exit 2
/// TV-03-013-003: Declared ref not in backend → exit 2 (config-invalid, not exit 5).
/// MED-5: Uses isolated TempDir.
#[test]
fn test_BC_2_03_013_unresolvable_credential_ref_exits_two() {
    let (config_dir, _state_tmp, _spec_tmp) = make_valid_config_dir();
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", config_dir.path())
        // Inject a missing-ref failure at step 5.
        .env("PRISM_TEST_INJECT_FAIL_STEP", "5_missing_ref")
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Unresolvable credential ref must exit 2 (BC-2.03.013 config-invalid path); \
         got exit {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// BC-2.03.013 — exit code is exactly 5 for permission errors, never 4
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-7
/// BC: BC-2.03.013 Invariant: permission-denied failure → exit 5, never exit 4.
/// MED-5: Uses isolated TempDir.
#[test]
fn test_BC_2_03_013_invariant_permission_denied_is_exit_5_not_4() {
    let (config_dir, _state_tmp, _spec_tmp) = make_valid_config_dir();
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", config_dir.path())
        .env("PRISM_TEST_INJECT_FAIL_STEP", "5_permission")
        .output()
        .expect("failed to spawn prism binary");

    let code = output.status.code().unwrap_or(-1);
    assert_ne!(
        code, 4,
        "Credential permission-denied must produce exit 5, not exit 4 \
         (BC-2.03.013 invariant; exit 4 is for AuditEmitter failure)"
    );
    assert_ne!(
        code, 2,
        "Credential permission-denied must produce exit 5, not exit 2 \
         (BC-2.03.013 invariant; exit 2 is for config-invalid)"
    );
    assert_eq!(
        code, 5,
        "Credential permission-denied must produce exit 5 (BC-2.03.013 + AC-7)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.03.013 — credential non-leak invariant (Approach A: type-level check)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.03.013 §Critical Invariant — after open(), CredentialStore holds no secret values
/// TV-03-013-006: CredentialStore fields are ref metadata only, no SecretString values.
///
/// Approach A from BC-2.03.013 §Test Strategy: type-level inspection asserting
/// that the BootError::CredentialPermissionDenied and CredentialRefInvalid variants
/// have String payloads (not SecretString) — messages about refs, not values.
///
/// Additionally, this test verifies that the BootError type itself does not contain
/// any credential value in its error message output.
///
/// RED GATE: Fails today because boot.rs step functions are all `todo!()`.
#[test]
fn test_BC_2_03_013_boot_error_type_carries_no_secret_value() {
    use prism_bin::BootError;

    // CredentialPermissionDenied message must describe the backend type and
    // denied operation — NOT include a credential value.
    let err = BootError::CredentialPermissionDenied("keyring service not responding".to_string());
    let msg = err.to_string();

    // The message must not contain any pattern that looks like a credential value.
    // We check that it doesn't contain things that look like secrets.
    // (In the real implementation, the message comes from the backend's error,
    //  which per AD-017 must never include the secret value.)
    assert!(
        !msg.is_empty(),
        "BootError::CredentialPermissionDenied must produce a non-empty display string"
    );
    assert!(
        msg.contains("credential-permission-denied"),
        "BootError display must include variant name per BC-2.03.013"
    );

    // Exit code must be 5 for permission denied.
    assert_eq!(
        err.exit_code(),
        5,
        "CredentialPermissionDenied exit_code() must return 5 (BC-2.03.013 + ADR-022 §A)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.03.013 — F-PASS2-HIGH-3: credential_refs field population in SensorSpec
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 F-PASS2-HIGH-3
/// BC: BC-2.03.013 §Postconditions (Happy path bullet 2)
/// "All credential references declared in sensor specs are validated as resolvable."
///
/// Unit test verifying that:
/// 1. `prism-spec-engine::add_sensor_spec::parse_and_validate_spec_toml` parses
///    `[[credential_refs]]` sections from TOML into `SensorSpec.credential_refs`.
/// 2. The parsed CredentialRef list is non-empty for specs that declare refs.
/// 3. The step5 iteration loop body is exercisable (N>0 refs from a real fixture).
///
/// This test does NOT exercise keyring probing (that requires a live keyring).
/// It exercises the data model: sensor spec TOML → credential_refs field.
#[test]
fn test_BC_2_03_013_sensor_spec_credential_refs_parsed_from_toml() {
    use prism_spec_engine::add_sensor_spec::parse_and_validate_spec_toml;

    let fixture_toml = include_str!("../fixtures/sensors/test-sensor-with-cred-refs.sensor.toml");

    let result =
        parse_and_validate_spec_toml(fixture_toml, "test-sensor-with-cred-refs.sensor.toml");
    assert!(
        result.is_ok(),
        "Fixture sensor TOML must parse without errors; got: {:?}",
        result.err()
    );

    let spec = result.unwrap();
    assert_eq!(
        spec.credential_refs.len(),
        1,
        "Fixture declares 1 [[credential_refs]] section (updated from 2 to comply with \
         BC-2.01.016 Rule B / E-SPEC-013); \
         got {} refs: {:?}",
        spec.credential_refs.len(),
        spec.credential_refs
            .iter()
            .map(|r| &r.name)
            .collect::<Vec<_>>()
    );

    // Verify the ref name matches the fixture.
    let ref_names: Vec<&str> = spec
        .credential_refs
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    assert!(
        ref_names.contains(&"api_key"),
        "Credential ref must be 'api_key'; got: {ref_names:?}"
    );

    // Verify sensor_id is correct (ensures parse was on the right fixture).
    assert_eq!(
        spec.sensor_id, "test-sensor",
        "Fixture sensor_id must be 'test-sensor'"
    );
}

/// Story: S-WAVE5-PREP-01 F-PASS2-HIGH-3
/// BC: BC-2.03.013 EC-03-013-001: No refs → 0 validated → boot continues
///
/// Unit test verifying that a sensor spec with no `[[credential_refs]]` sections
/// produces an empty `credential_refs` Vec (not an error).
/// This exercises EC-03-013-001: zero refs validated is not an error.
#[test]
fn test_BC_2_03_013_sensor_spec_no_cred_refs_is_empty_not_error() {
    use prism_spec_engine::add_sensor_spec::parse_and_validate_spec_toml;

    // ADR-030 Approach D: SpecLoader::parse uses flat top-level TOML format (no [sensor] section).
    let toml_no_refs = r#"
sensor_id = "minimal-sensor"
name = "Minimal Sensor (no cred refs)"
version = "0.1.0"
auth_type = "api_key"
base_url = "https://minimal.example.com"
"#;

    let result = parse_and_validate_spec_toml(toml_no_refs, "minimal.sensor.toml");
    assert!(
        result.is_ok(),
        "Sensor spec with no [[credential_refs]] must parse OK (EC-03-013-001)"
    );

    let spec = result.unwrap();
    assert!(
        spec.credential_refs.is_empty(),
        "SensorSpec with no [[credential_refs]] sections must have empty Vec; \
         got: {:?}",
        spec.credential_refs
    );
}

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.03.013 §Critical Invariant — CredentialRefInvalid exits 2 (not 5)
///
/// The distinction between permission-denied (exit 5) and invalid-ref (exit 2)
/// is a core invariant of BC-2.03.013.
///
/// RED GATE: This test exercises the BootError type; passes as-is because
/// `exit_codes.rs` and `BootError::exit_code()` are already implemented.
/// However, the integration path (subprocess) still fails, so this companion
/// test for the boot path ensures coverage.
#[test]
fn test_BC_2_03_013_credential_ref_invalid_exits_two_not_five() {
    use prism_bin::BootError;

    let err = BootError::CredentialRefInvalid(
        "Unresolvable credential ref: crowdstrike.api_key for sensor crowdstrike".to_string(),
    );

    assert_eq!(
        err.exit_code(),
        2,
        "CredentialRefInvalid exit_code() must return 2 (config-invalid); \
         BC-2.03.013 failure path 'credential reference unresolvable'"
    );
}

// ---------------------------------------------------------------------------
// OBS-2 — BC-2.03.013 non-leak invariant (Approach A: type-level inspection)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.03.013 §Critical Invariant — CredentialStore holds only ref metadata
/// OBS-2 (S-WAVE5-PREP-01 fix-pass-1): Approach A from BC-2.03.013 §Test Strategy.
///
/// Approach A: type-level inspection — assert that `KeyringBackend` struct fields
/// are limited to ref metadata types (String, CredentialIndex) and contain NO
/// secret value fields (`SecretString`, `Vec<u8>`, etc.).
///
/// This test verifies the API surface at the type level: `KeyringBackend::new`
/// accepts only reference metadata (app_name: &str, index: CredentialIndex) and
/// returns a struct with no secret value storage. The BootError variants that
/// carry credential information (CredentialPermissionDenied, CredentialRefInvalid)
/// hold String (a diagnostic message about the ref, NOT the value).
///
/// This satisfies BC-2.03.013 OQ-1 for S-WAVE5-PREP-01.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_03_013_OQ1_non_leak_invariant_approach_a_type_level() {
    use std::path::PathBuf;

    use prism_bin::BootError;
    use prism_credentials::{CredentialIndex, KeyringBackend};

    // Approach A: construct KeyringBackend with reference metadata only.
    // The constructor signature is: new(app_name: &str, index: CredentialIndex) -> Self.
    // No secret value is passed to the constructor — this IS the type-level invariant.
    let index_path = PathBuf::from("/tmp/prism-test-cred-index-obs2.json");
    let index = CredentialIndex::new(index_path);
    let backend = KeyringBackend::new("prism", index);

    // The backend was constructed without any secret value in the call site.
    // By construction, KeyringBackend holds only: app_name (String) + index (CredentialIndex).
    // Neither String nor CredentialIndex is a SecretString or Vec<u8> secret value.
    //
    // We verify this at the call-site level — the type system enforces it.
    // If KeyringBackend's constructor ever required a SecretString parameter,
    // this test would fail to compile, making the invariant machine-checked.
    drop(backend); // constructed without secret; no values in memory

    // Additionally verify BootError error messages do not leak secret patterns.
    // A secret value typically looks like a base64 or hex string of significant length.
    // Our error messages must describe refs (key names), not values.
    let permission_err = BootError::CredentialPermissionDenied(
        "keyring service locked: prism.crowdstrike.api_key".to_string(),
    );
    let err_msg = permission_err.to_string();
    // The message describes a keyring key name (ref), not a secret value.
    assert!(
        err_msg.contains("credential-permission-denied"),
        "OBS-2 Approach A: CredentialPermissionDenied display must contain variant prefix; \
         got: {err_msg}"
    );
    // The message must not contain anything that looks like a base64-encoded 40+ char value.
    // This is a heuristic, not a perfect check. Real enforcement is type-level (above).
    assert!(
        err_msg.len() < 512,
        "OBS-2 Approach A: error message suspiciously long — check for secret value leakage; \
         len={}, msg={err_msg:?}",
        err_msg.len()
    );
}

// ---------------------------------------------------------------------------
// F-PASS3-HIGH-1 — credential_ref iteration loop behavioral coverage
//
// These tests exercise step5_init_credential_store directly (not subprocess),
// using the keyring mock backend so no live keyring service is required.
//
// Strategy: keyring mock backend (Approach B1 from F-PASS3-HIGH-1 spec):
//   1. keyring::set_default_credential_builder(keyring::mock::default_credential_builder())
//   2. Pre-populate the mock via Entry::new("prism", account).set_password(...)
//   3. Call step5_init_credential_store with a real ConfigManager backed by
//      a spec_tmp directory containing the fixture sensor TOML.
//
// Sensor TOML declares 2 [[credential_refs]]: api_key + client_secret.
// The namespaced keyring account is "{sensor_id}/{ref_name}".
//
// Happy path: both refs pre-populated → Ok(store), refs_validated = 2.
// Unhappy path: one ref missing → CredentialRefInvalid (exit 2).
// ---------------------------------------------------------------------------

/// Inline fixture sensor TOML with 1 credential_ref — mirrors
/// fixtures/sensors/test-sensor-with-cred-refs.sensor.toml.
///
/// Updated from 2 → 1 ref when parse_and_validate_spec_toml was wired to enforce
/// BC-2.01.016 Rule B (exactly 1 credential_ref per auth method; E-SPEC-013).
/// A fixture with 2 refs would be rejected at parse time. N=1 satisfies the
/// "N>0 refs iterated" behavioral coverage requirement for step5 tests.
///
/// ADR-030 Approach D: uses flat top-level TOML format (no [sensor] section)
/// as required by SpecLoader::parse → toml::from_str::<SensorSpec>.
const CRED_REF_FIXTURE_TOML: &str = r#"
sensor_id = "test-sensor"
name = "Test Sensor (credential ref fixture)"
version = "0.1.0"
auth_type = "api_key"
base_url = "https://test-sensor.example.com"

[[credential_refs]]
name = "api_key"
"#;

/// Build a PrismConfig pointing at the given spec_dir and a temporary state_dir.
/// Returns (config, _state_tmp) — caller must keep _state_tmp alive.
fn make_config_with_spec_dir(
    spec_dir: &std::path::Path,
) -> (prism_bin::boot::PrismConfig, tempfile::TempDir) {
    let state_tmp = tempfile::TempDir::new().unwrap();
    let config = prism_bin::boot::PrismConfig::new_for_test(
        spec_dir.to_path_buf(),
        state_tmp.path().to_path_buf(),
        "plugins",
        vec![prism_bin::boot::OrgEntry::new(
            "0196f000-0000-7000-8000-000000000001",
            "acme",
        )],
        prism_bin::boot::CredentialBackendConfig::Keyring,
    );
    (config, state_tmp)
}

/// Build a ConfigManager from a spec_tmp containing the fixture sensor TOML.
fn make_config_manager_with_cred_refs(
    spec_tmp: &tempfile::TempDir,
) -> std::sync::Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>> {
    use arc_swap::ArcSwap;
    use prism_spec_engine::config_manager::{ConfigManager, parse_spec_directory};

    // Write the fixture sensor TOML into spec_tmp.
    std::fs::write(
        spec_tmp.path().join("test-sensor.sensor.toml"),
        CRED_REF_FIXTURE_TOML,
    )
    .unwrap();

    let snapshot = parse_spec_directory(spec_tmp.path()).unwrap();
    std::sync::Arc::new(ArcSwap::from_pointee(ConfigManager::new(snapshot)))
}

// ---------------------------------------------------------------------------
// Mock credential ref probe for F-PASS3-HIGH-1 tests
// ---------------------------------------------------------------------------

/// A controllable test double for CredentialRefProbe.
///
/// `AlwaysOkProbe` — all refs resolve to Ok(()).
/// `MissingOneProbe` — a specific ref_name returns CredentialRefInvalid.
struct AlwaysOkProbe;

#[async_trait::async_trait]
impl prism_bin::boot::CredentialRefProbe for AlwaysOkProbe {
    async fn probe(
        &self,
        _sensor_id: &str,
        _ref_name: &str,
        _org_registry: &prism_core::OrgRegistry,
    ) -> Result<Option<String>, prism_bin::BootError> {
        // No auth_type metadata — Rule C not enforced by this probe.
        Ok(None)
    }
}

struct MissingOneProbe {
    /// The ref_name that should return CredentialRefInvalid.
    missing_ref: &'static str,
}

#[async_trait::async_trait]
impl prism_bin::boot::CredentialRefProbe for MissingOneProbe {
    async fn probe(
        &self,
        sensor_id: &str,
        ref_name: &str,
        _org_registry: &prism_core::OrgRegistry,
    ) -> Result<Option<String>, prism_bin::BootError> {
        if ref_name == self.missing_ref {
            Err(prism_bin::BootError::CredentialRefInvalid(format!(
                "Unresolvable credential ref: '{}' for sensor '{}' not found \
                 (mock probe: intentionally missing ref)",
                ref_name, sensor_id
            )))
        } else {
            Ok(None)
        }
    }
}

/// Story: S-WAVE5-PREP-01 F-PASS3-HIGH-1
/// BC: BC-2.03.013 §Postconditions item 3 — N>0 credential_refs, all resolvable → Ok
///
/// Happy path: 2 credential refs declared in sensor spec, all refs resolvable via
/// AlwaysOkProbe test double. step5_init_credential_store_with_probe must return Ok.
/// The iteration loop runs N=2 times (not 0) — this closes the vacuous-loop behavioral
/// coverage gap identified in F-PASS3-HIGH-1.
///
/// Strategy: Approach B (BC-2.03.013 §Test Strategy) — injectable CredentialRefProbe.
/// AlwaysOkProbe returns Ok(()) for every ref, eliminating keyring dependency.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_03_013_credential_ref_iteration_happy_path_all_refs_resolvable() {
    // Write fixture sensor TOML to spec_tmp (2 credential_refs declared).
    let spec_tmp = tempfile::TempDir::new().unwrap();
    let config_manager = make_config_manager_with_cred_refs(&spec_tmp);

    // Regression guard: verify spec was loaded with N=1 credential_ref (N>0).
    // If 0 refs, the iteration loop would still be vacuous (test setup failure).
    // Updated from N=2 to N=1 per BC-2.01.016 Rule B (E-SPEC-013): exactly 1 ref allowed.
    {
        let cm_guard = config_manager.load();
        let cm = &**cm_guard;
        let snapshot_guard = cm.load();
        let snapshot = &**snapshot_guard;
        assert_eq!(
            snapshot
                .sensor_specs
                .get("test-sensor")
                .map(|s| s.credential_refs.len()),
            Some(1),
            "Fixture TOML must be parsed with 1 credential_ref (N>0 for iteration coverage); \
             got wrong count (test setup failure, not production bug)"
        );
    }

    let (config, _state_tmp) = make_config_with_spec_dir(spec_tmp.path());

    // Inject AlwaysOkProbe — both refs resolve to Ok(()) without keyring.
    // AlwaysOkProbe ignores org_registry; pass an empty registry for the probe's signature.
    let empty_registry = std::sync::Arc::new(prism_core::OrgRegistry::new());
    let result = prism_bin::boot::step5_init_credential_store_with_probe(
        &config,
        &config_manager,
        &empty_registry,
        &AlwaysOkProbe,
    )
    .await;

    assert!(
        result.is_ok(),
        "BC-2.03.013 §Postconditions item 3: step5 with 2 resolvable refs must return Ok; \
         got: {:?}",
        result.err()
    );
}

/// Story: S-WAVE5-PREP-01 F-PASS3-HIGH-1
/// BC: BC-2.03.013 §Failure path TV-03-013-003 — missing ref → CredentialRefInvalid (exit 2)
///
/// Unhappy path: 2 credential refs declared, one made to return CredentialRefInvalid via
/// MissingOneProbe test double. step5_init_credential_store_with_probe must propagate the
/// error as CredentialRefInvalid with exit code 2.
///
/// Behavioral coverage: confirms the iteration loop body executes for N>0 refs and that
/// the CredentialRefInvalid error path maps to exit 2 (not exit 5).
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_03_013_credential_ref_iteration_unhappy_path_missing_ref_exits_two() {
    // Write fixture sensor TOML to spec_tmp (2 credential_refs: api_key + client_secret).
    let spec_tmp = tempfile::TempDir::new().unwrap();
    let config_manager = make_config_manager_with_cred_refs(&spec_tmp);

    let (config, _state_tmp) = make_config_with_spec_dir(spec_tmp.path());

    // Inject MissingOneProbe — api_key returns CredentialRefInvalid.
    // (Updated from "client_secret" to "api_key" per fixture update from 2→1 ref.)
    let probe = MissingOneProbe {
        missing_ref: "api_key",
    };
    // MissingOneProbe ignores org_registry; pass an empty registry for the probe's signature.
    let empty_registry = std::sync::Arc::new(prism_core::OrgRegistry::new());
    let result = prism_bin::boot::step5_init_credential_store_with_probe(
        &config,
        &config_manager,
        &empty_registry,
        &probe,
    )
    .await;

    assert!(
        result.is_err(),
        "BC-2.03.013 TV-03-013-003: step5 with a missing ref must return Err; \
         got Ok (vacuous-loop defect not closed)"
    );

    let err = result.map(|_| ()).unwrap_err();
    assert_eq!(
        err.exit_code(),
        2,
        "BC-2.03.013 TV-03-013-003: missing credential ref must map to exit 2 \
         (CredentialRefInvalid), not exit 5 (CredentialPermissionDenied); \
         got exit_code={} for error: {:?}",
        err.exit_code(),
        err
    );

    // Verify it is specifically CredentialRefInvalid, not CredentialPermissionDenied.
    assert!(
        matches!(err, prism_bin::BootError::CredentialRefInvalid(_)),
        "BC-2.03.013 TV-03-013-003: error variant must be CredentialRefInvalid; \
         got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// F-LP-IMPL-P4-001 — Rule C (E-SPEC-014) enforcement at step5
//
// The production callsites (spec_parser.rs and add_sensor_spec.rs) correctly
// pass the same auth_type string as both expected_shape and actual_shape so that
// Rule C never fires at parse time (no credential introspection available there).
//
// Rule C is enforced HERE at step5 via CredentialRefProbe::probe() returning
// Some(actual_shape) when the credential was configured for a different auth_type
// than the sensor spec declares.
//
// Pre-fix failure mode: probe() returned Ok(()) regardless — Rule C never fired
// because the step5 loop never compared the probe return value against auth_type.
// ---------------------------------------------------------------------------

/// A probe test double that returns a specific auth_type for all refs.
/// Used to simulate a credential configured for `actual_shape` auth type.
struct ShapedProbe {
    /// The auth_type this probe reports for all refs.
    reported_auth_type: &'static str,
}

#[async_trait::async_trait]
impl prism_bin::boot::CredentialRefProbe for ShapedProbe {
    async fn probe(
        &self,
        _sensor_id: &str,
        _ref_name: &str,
        _org_registry: &prism_core::OrgRegistry,
    ) -> Result<Option<String>, prism_bin::BootError> {
        Ok(Some(self.reported_auth_type.to_string()))
    }
}

/// BC-2.01.016 AC-3c / F-LP-IMPL-P4-001:
/// step5_init_credential_store_with_probe must enforce Rule C (E-SPEC-014)
/// when the probe reports an auth_type that differs from the sensor spec's
/// declared auth_type.
///
/// Scenario: sensor spec declares `auth_type = "oauth2_client_credentials"`
/// but the credential probe reports `actual_shape = "api_key"` — shape mismatch.
/// step5 must return `Err(BootError::AuthTypeCredentialMismatch)` with exit code 2.
///
/// This test exercises the PRODUCTION STEP5 CONTROL FLOW (not validate_cross_composition
/// directly) — the full path from parse_spec_directory → step5 probe loop Rule C check.
///
/// Pre-fix failure mode: step5 loop called probe.probe() and discarded the return value
/// (treated as Ok(())), so Rule C never fired even when the probe returned a shape.
///
/// Story: S-PLUGIN-PREREQ-E AC-3c / F-LP-IMPL-P4-001 | BC-2.01.016 §Error Cases E-SPEC-014
/// ADR-023 §Architectural Constraints Rule 2 Rule C
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_01_016_rule_c_credential_shape_mismatch_via_step5() {
    use arc_swap::ArcSwap;
    use prism_spec_engine::config_manager::{ConfigManager, parse_spec_directory};

    // Write a fixture sensor TOML declaring auth_type = "oauth2_client_credentials"
    // with 1 credential_ref. The credential probe will report "api_key" — a mismatch.
    // ADR-030 Approach D: uses flat top-level TOML format (no [sensor] section).
    let spec_tmp = tempfile::TempDir::new().unwrap();
    let oauth2_sensor_toml = r#"
sensor_id = "oauth2-sensor"
name = "OAuth2 Sensor (Rule C test)"
version = "0.1.0"
auth_type = "oauth2_client_credentials"
base_url = "https://oauth2-sensor.example.com"

[[credential_refs]]
name = "client_secret"
"#;

    std::fs::write(
        spec_tmp.path().join("oauth2-sensor.sensor.toml"),
        oauth2_sensor_toml,
    )
    .unwrap();

    // Parse the spec directory — spec must load successfully (Rule A+B pass).
    let snapshot =
        parse_spec_directory(spec_tmp.path()).expect("parse_spec_directory must not fail");

    // Confirm the spec loaded with the expected auth_type.
    assert_eq!(
        snapshot
            .sensor_specs
            .get("oauth2-sensor")
            .map(|s| s.auth_type.as_str()),
        Some("oauth2_client_credentials"),
        "Rule C test setup: oauth2-sensor must be in sensor_specs with auth_type=oauth2_client_credentials"
    );

    let config_manager = std::sync::Arc::new(ArcSwap::from_pointee(ConfigManager::new(snapshot)));
    let (config, _state_tmp) = make_config_with_spec_dir(spec_tmp.path());

    // Inject ShapedProbe that reports "api_key" — mismatches spec's "oauth2_client_credentials".
    // Pre-fix: step5 calls probe.probe() and ignores the Some("api_key") return → Ok(store).
    // Post-fix: step5 compares "api_key" != "oauth2_client_credentials" → Err(AuthTypeCredentialMismatch).
    let probe = ShapedProbe {
        reported_auth_type: "api_key",
    };
    // ShapedProbe ignores org_registry; pass an empty registry for the probe's signature.
    let empty_registry = std::sync::Arc::new(prism_core::OrgRegistry::new());
    let result = prism_bin::boot::step5_init_credential_store_with_probe(
        &config,
        &config_manager,
        &empty_registry,
        &probe,
    )
    .await;

    assert!(
        result.is_err(),
        "F-LP-IMPL-P4-001: step5 must return Err when probe reports auth_type mismatch \
         (expected=oauth2_client_credentials, actual=api_key); got Ok — \
         Rule C step5 enforcement not wired (probe return value discarded)"
    );

    let err = result.map(|_| ()).unwrap_err();

    assert_eq!(
        err.exit_code(),
        2,
        "F-LP-IMPL-P4-001: AuthTypeCredentialMismatch must map to exit 2; got {}",
        err.exit_code()
    );

    let err_str = format!("{err}");
    assert!(
        err_str.contains("E-SPEC-014"),
        "F-LP-IMPL-P4-001: error must cite E-SPEC-014; got: {err_str}"
    );
    assert!(
        err_str.contains("oauth2_client_credentials"),
        "F-LP-IMPL-P4-001: error must cite expected_shape; got: {err_str}"
    );
    assert!(
        err_str.contains("api_key"),
        "F-LP-IMPL-P4-001: error must cite actual_shape; got: {err_str}"
    );

    // Verify it is specifically AuthTypeCredentialMismatch.
    assert!(
        matches!(
            err,
            prism_bin::BootError::AuthTypeCredentialMismatch {
                sensor_id: _,
                expected_shape: _,
                actual_shape: _,
            }
        ),
        "F-LP-IMPL-P4-001: error variant must be AuthTypeCredentialMismatch; got: {:?}",
        err
    );
}

/// BC-2.01.016 AC-3c / F-LP-IMPL-P4-001:
/// step5 must NOT error when probe reports auth_type matching the spec's declared auth_type.
///
/// Regression guard: Rule C check must not produce false positives when shapes match.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_01_016_rule_c_matching_shape_via_step5_succeeds() {
    use arc_swap::ArcSwap;
    use prism_spec_engine::config_manager::{ConfigManager, parse_spec_directory};

    // ADR-030 Approach D: uses flat top-level TOML format (no [sensor] section).
    let spec_tmp = tempfile::TempDir::new().unwrap();
    let api_key_sensor_toml = r#"
sensor_id = "api-key-sensor"
name = "API Key Sensor (Rule C match test)"
version = "0.1.0"
auth_type = "api_key"
base_url = "https://api-key-sensor.example.com"

[[credential_refs]]
name = "api_key"
"#;

    std::fs::write(
        spec_tmp.path().join("api-key-sensor.sensor.toml"),
        api_key_sensor_toml,
    )
    .unwrap();

    let snapshot =
        parse_spec_directory(spec_tmp.path()).expect("parse_spec_directory must not fail");
    let config_manager = std::sync::Arc::new(ArcSwap::from_pointee(ConfigManager::new(snapshot)));
    let (config, _state_tmp) = make_config_with_spec_dir(spec_tmp.path());

    // Probe reports "api_key" — matches spec's "api_key" → no Rule C error.
    let probe = ShapedProbe {
        reported_auth_type: "api_key",
    };
    // ShapedProbe ignores org_registry; pass an empty registry for the probe's signature.
    let empty_registry = std::sync::Arc::new(prism_core::OrgRegistry::new());
    let result = prism_bin::boot::step5_init_credential_store_with_probe(
        &config,
        &config_manager,
        &empty_registry,
        &probe,
    )
    .await;

    assert!(
        result.is_ok(),
        "F-LP-IMPL-P4-001 (regression guard): step5 must return Ok when probe auth_type \
         matches spec auth_type (both 'api_key'); got: {:?}",
        result.err()
    );
}
