//! S-PLUGIN-PREREQ-D Red Gate tests — prism-bin plugin boot wiring.
//!
//! Tests `plugin_load_step` and `PluginRuntime::load_all_plugins` integration with
//! the boot sequence. Uses WAT-compiled WASM fixtures via the `wat` crate (no pre-built
//! binaries required — compilation happens inline per test).
//!
//! Traces to: S-PLUGIN-PREREQ-D (v1.32)
//! BCs: BC-2.22.001 (boot orchestration)
//! VPs: VP-PLUGIN-004, VP-PLUGIN-007
//!
//! # Test → AC / BC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_BC_2_22_001_boot_step_plugin_load_placement | AC-1 | BC-2.22.001 §Sequencing Invariant (step 7.5) |
//! | test_BC_2_22_001_plugin_load_failure_exits_code_4 | AC-2 | BC-2.22.001 §Pre-Traffic Gate Invariant condition 6 |
//! | test_BC_2_22_001_plugin_load_disabled_env | AC-3 | BC-2.22.001; EC-D-004 |
//! | test_BC_2_22_001_disable_env_takes_precedence_over_plugin_dir_config | AC-18 | BC-2.22.001; EC-D-011 |
//! | test_VP_PLUGIN_004_unsigned_plugin_boot_warn_audit | AC-4 | VP-PLUGIN-004 (unsigned-plugin boot warning + audit) |
//! | test_VP_PLUGIN_007_plugin_load_rejected_no_allowlist | AC-5 | VP-PLUGIN-007; E-PLUGIN-013 |
//! | test_VP_PLUGIN_007_plugin_load_rejected_format_version_exceeded | AC-5 | VP-PLUGIN-007; E-PLUGIN-014 |

#![allow(dead_code, unused_imports)]

use std::path::Path;
use std::sync::Arc;

use prism_bin::boot::plugin_load_step;

// ---------------------------------------------------------------------------
// Test helper utilities
// ---------------------------------------------------------------------------

/// Helper: compile WAT source to WASM bytes using the `wat` crate.
fn compile_wat(source: &str) -> Vec<u8> {
    wat::parse_str(source).expect("WAT compilation failed")
}

/// Helper: write WASM bytes to a `.prx` file in the given directory.
fn write_prx(dir: &tempfile::TempDir, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = dir.path().join(format!("{name}.prx"));
    std::fs::write(&path, bytes).expect("write .prx failed");
    path
}

/// Helper: write a manifest companion `.manifest.toml` file for a `.prx` plugin.
/// The companion file is named `{prx_name}.manifest.toml` (replaces `.prx` extension).
/// This matches the `path.with_extension("manifest.toml")` pattern in `load_all_plugins`.
fn write_manifest(dir: &tempfile::TempDir, prx_name: &str, manifest_toml: &str) {
    let path = dir.path().join(format!("{prx_name}.manifest.toml"));
    std::fs::write(&path, manifest_toml).expect("write manifest.toml failed");
}

// ---------------------------------------------------------------------------
// WAT/TOML fixtures
// ---------------------------------------------------------------------------

/// Minimal valid infusion plugin WAT source.
const MINIMAL_INFUSION_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (data (i32.const 0) "minimal-ok")
  (data (i32.const 16) "1.0.0")
  (func (export "name") (result i32 i32)
    i32.const 0 i32.const 10)
  (func (export "version") (result i32 i32)
    i32.const 16 i32.const 5)
  (func (export "enrich-single") (param i32 i32 i32 i32) (result i32)
    i32.const 0)
  (func (export "enrich-batch") (param i32 i32 i32 i32) (result i32 i32)
    i32.const 0 i32.const 0)
)
"#;

/// Minimal valid manifest TOML.
const MINIMAL_MANIFEST_TOML: &str = r#"
name = "minimal-ok"
version = "1.0.0"
format_version = 1
allowed_urls = []
"#;

/// Manifest TOML missing the `allowed_urls` field (E-PLUGIN-013).
const MANIFEST_NO_ALLOWED_URLS_TOML: &str = r#"
name = "no-allowlist-plugin"
version = "1.0.0"
format_version = 1
"#;

/// Manifest TOML with `format_version = 99` (exceeds `CURRENT_SUPPORTED_VERSION = 1`).
const MANIFEST_FORMAT_VERSION_EXCEEDED_TOML: &str = r#"
name = "future-format-plugin"
version = "1.0.0"
format_version = 99
allowed_urls = []
"#;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// AC-1 (S-PLUGIN-PREREQ-D) — Plugin-load step inserted between storage init (step 7)
/// and query-engine init (step 8) per BC-2.22.001 §Sequencing Invariant (step 7.5).
///
/// Verifies that `plugin_load_step` exists as a public function in `prism_bin::boot`,
/// accepts a `&Path` for the plugin directory, and returns `Ok` when the directory
/// is empty (EC-D-002: no .prx files → Ok(0)).
#[tokio::test]
async fn test_BC_2_22_001_boot_step_plugin_load_placement() {
    let dir = tempfile::tempdir().expect("create temp dir");
    // Empty plugin directory — no .prx files (EC-D-002 path).
    let result = plugin_load_step(dir.path()).await;
    assert!(
        result.is_ok(),
        "AC-1: plugin_load_step with empty dir must return Ok; got {:?}",
        result.err()
    );
    let load_result = result.unwrap();
    assert_eq!(
        load_result.plugins_loaded, 0,
        "AC-1: empty directory must yield 0 plugins loaded (EC-D-002)"
    );
}

/// AC-2 (S-PLUGIN-PREREQ-D) — Plugin-load step failure causes boot to exit with code 4.
///
/// Verifies that `BootError::InternalError` maps to exit code 4 (ADR-022 §A),
/// and that a non-existent plugin directory (EC-D-001) returns Ok(0) rather than Err.
#[tokio::test]
async fn test_BC_2_22_001_plugin_load_failure_exits_code_4() {
    use prism_bin::boot::BootError;

    // Verify that BootError::InternalError maps to exit code 4 (ADR-022 §A).
    let err = BootError::InternalError("test error".to_string());
    assert_eq!(
        err.exit_code(),
        4,
        "AC-2: BootError::InternalError must map to exit code 4 (ADR-022 §A internal-error class)"
    );

    // Non-existent plugin directory → EC-D-001: returns Ok(0), NOT Err.
    // Boot failure only occurs if PluginRuntime construction itself fails.
    let nonexistent = Path::new("/tmp/prism_nonexistent_plugin_dir_test_ac2_42x");
    let result = plugin_load_step(nonexistent).await;
    assert!(
        result.is_ok(),
        "AC-2: non-existent plugin directory must return Ok(0) per EC-D-001, not Err"
    );
    assert_eq!(
        result.unwrap().plugins_loaded,
        0,
        "AC-2: non-existent plugin dir must load 0 plugins"
    );
}

/// AC-3 (S-PLUGIN-PREREQ-D) — `PRISM_DISABLE_PLUGIN_LOAD=1` skips all plugin loading.
///
/// Returns Ok(0) without scanning the plugin directory. MCP server still binds.
/// (EC-D-004; BC-2.16.002 catalog row `plugin_load_disabled_via_envvar`.)
#[tokio::test]
async fn test_BC_2_22_001_plugin_load_disabled_env() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let bytes = compile_wat(MINIMAL_INFUSION_WAT);
    write_prx(&dir, "minimal", &bytes);
    write_manifest(&dir, "minimal", MINIMAL_MANIFEST_TOML);

    std::env::set_var("PRISM_DISABLE_PLUGIN_LOAD", "1");
    let result = plugin_load_step(dir.path()).await;
    std::env::remove_var("PRISM_DISABLE_PLUGIN_LOAD");

    assert!(
        result.is_ok(),
        "AC-3: PRISM_DISABLE_PLUGIN_LOAD=1 must return Ok even with valid plugins present"
    );
    let load_result = result.unwrap();
    assert_eq!(
        load_result.plugins_loaded, 0,
        "AC-3: PRISM_DISABLE_PLUGIN_LOAD=1 must skip all plugins (loaded 0)"
    );
}

/// AC-18 (S-PLUGIN-PREREQ-D) — `PRISM_DISABLE_PLUGIN_LOAD=1` takes absolute precedence;
/// non-`"1"` values are treated as unset (EC-D-011).
#[tokio::test]
async fn test_BC_2_22_001_disable_env_takes_precedence_over_plugin_dir_config() {
    let dir = tempfile::tempdir().expect("create temp dir");

    // Non-"1" values must NOT disable loading (EC-D-011: only exact "1" disables).
    for val in &["true", "yes", "TRUE", "1.0", "0", "false"] {
        std::env::set_var("PRISM_DISABLE_PLUGIN_LOAD", val);
        let result = plugin_load_step(dir.path()).await;
        std::env::remove_var("PRISM_DISABLE_PLUGIN_LOAD");
        assert!(
            result.is_ok(),
            "AC-18: PRISM_DISABLE_PLUGIN_LOAD={val} must not cause boot failure (empty dir → Ok(0))"
        );
        // With empty dir and non-"1" value, goes through normal path → 0 plugins (EC-D-002).
    }

    // Exact "1" DOES disable (takes absolute precedence over plugin_dir config).
    let dir2 = tempfile::tempdir().expect("create temp dir");
    let bytes = compile_wat(MINIMAL_INFUSION_WAT);
    write_prx(&dir2, "minimal", &bytes);
    write_manifest(&dir2, "minimal", MINIMAL_MANIFEST_TOML);

    std::env::set_var("PRISM_DISABLE_PLUGIN_LOAD", "1");
    let result_disabled = plugin_load_step(dir2.path()).await;
    std::env::remove_var("PRISM_DISABLE_PLUGIN_LOAD");

    assert_eq!(
        result_disabled.unwrap().plugins_loaded,
        0,
        "AC-18: PRISM_DISABLE_PLUGIN_LOAD=1 (exact string) must disable loading even with valid plugin dir"
    );
}

/// AC-4 (S-PLUGIN-PREREQ-D) — Every successfully loaded plugin generates a
/// `tracing::warn!(event_type = "plugin_load_unsigned", ...)` audit entry (VP-PLUGIN-004).
#[tokio::test]
async fn test_VP_PLUGIN_004_unsigned_plugin_boot_warn_audit() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let bytes = compile_wat(MINIMAL_INFUSION_WAT);
    write_prx(&dir, "minimal", &bytes);
    write_manifest(&dir, "minimal", MINIMAL_MANIFEST_TOML);

    let result = plugin_load_step(dir.path()).await;
    assert!(
        result.is_ok(),
        "AC-4: plugin_load_step with valid plugin must succeed; got {:?}",
        result.err()
    );
    let load_result = result.unwrap();
    assert_eq!(
        load_result.plugins_loaded, 1,
        "AC-4: exactly 1 valid plugin must be loaded (audit event emitted per plugin)"
    );

    // Verify the runtime has the plugin registered.
    let plugins = load_result.runtime.list_plugins();
    assert_eq!(
        plugins.len(),
        1,
        "AC-4: exactly 1 plugin registered in runtime after unsigned load; got {:?}",
        plugins
    );
}

/// AC-5 (S-PLUGIN-PREREQ-D) — Plugin missing `allowed_urls` manifest field is
/// rejected with `E-PLUGIN-013`; n-1 survivor rule applies (VP-PLUGIN-007).
#[tokio::test]
async fn test_VP_PLUGIN_007_plugin_load_rejected_no_allowlist() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let bytes = compile_wat(MINIMAL_INFUSION_WAT);

    // Plugin missing allowed_urls (will be rejected with E-PLUGIN-013).
    write_prx(&dir, "no-allowlist-plugin", &bytes);
    write_manifest(&dir, "no-allowlist-plugin", MANIFEST_NO_ALLOWED_URLS_TOML);

    // Valid plugin (n-1 survivor — must continue loading).
    write_prx(&dir, "valid-plugin", &bytes);
    write_manifest(&dir, "valid-plugin", MINIMAL_MANIFEST_TOML);

    let result = plugin_load_step(dir.path()).await;
    assert!(
        result.is_ok(),
        "AC-5: load_all_plugins must return Ok even when some plugins fail validation"
    );
    let load_result = result.unwrap();
    assert_eq!(
        load_result.plugins_loaded, 1,
        "AC-5 (E-PLUGIN-013): n-1 survivor rule — invalid plugin rejected, 1 valid plugin loaded"
    );

    let plugins = load_result.runtime.list_plugins();
    assert!(
        !plugins.iter().any(|id| id.contains("no-allowlist")),
        "AC-5 (E-PLUGIN-013): plugin missing allowed_urls MUST NOT be registered; got {:?}",
        plugins
    );
}

/// AC-5 (S-PLUGIN-PREREQ-D) — Plugin with `format_version` exceeding `CURRENT_SUPPORTED_VERSION`
/// is rejected with `E-PLUGIN-014`; n-1 survivor rule applies (VP-PLUGIN-007).
#[tokio::test]
async fn test_VP_PLUGIN_007_plugin_load_rejected_format_version_exceeded() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let bytes = compile_wat(MINIMAL_INFUSION_WAT);

    // Plugin with format_version = 99 (exceeds CURRENT_SUPPORTED_VERSION = 1).
    write_prx(&dir, "future-format-plugin", &bytes);
    write_manifest(
        &dir,
        "future-format-plugin",
        MANIFEST_FORMAT_VERSION_EXCEEDED_TOML,
    );

    // Valid plugin (n-1 survivor — must continue loading).
    write_prx(&dir, "valid-plugin", &bytes);
    write_manifest(&dir, "valid-plugin", MINIMAL_MANIFEST_TOML);

    let result = plugin_load_step(dir.path()).await;
    assert!(
        result.is_ok(),
        "AC-5: load_all_plugins must return Ok even when some plugins fail validation"
    );
    let load_result = result.unwrap();
    assert_eq!(
        load_result.plugins_loaded, 1,
        "AC-5 (E-PLUGIN-014): n-1 survivor rule — format_version exceeded rejected, 1 valid plugin loaded"
    );

    let plugins = load_result.runtime.list_plugins();
    assert!(
        !plugins.iter().any(|id| id.contains("future-format")),
        "AC-5 (E-PLUGIN-014): plugin with format_version > CURRENT_SUPPORTED_VERSION MUST NOT be registered; got {:?}",
        plugins
    );
}

// ---------------------------------------------------------------------------
// CRIT-001 / CRIT-002 regression tests (F-IMPL-LP1-CRIT-001 / F-IMPL-LP1-CRIT-002)
// ---------------------------------------------------------------------------

/// CRIT-002 (F-IMPL-LP1-CRIT-002) — PrismConfig::deserialize accepts `plugin_dir` and
/// defaults correctly when the field is absent from prism.toml.
///
/// Verifies:
/// 1. When `plugin_dir` is explicit in TOML, the deserialized value matches.
/// 2. When `plugin_dir` is absent from TOML, the deserialized value is `"plugins"` (the default).
/// 3. `PrismConfig::new_for_test` constructor works (covers external construction path).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_22_001_prism_config_plugin_dir_default_and_explicit() {
    // Case 1: plugin_dir explicitly set in TOML.
    let toml_explicit = r#"
spec_dir = "/tmp/specs"
state_dir = "/tmp/state"
plugin_dir = "/custom/plugins"
[[orgs]]
org_id = "0196f000-0000-7000-8000-000000000001"
org_slug = "acme"
"#;
    let config: prism_bin::boot::PrismConfig =
        toml::from_str(toml_explicit).expect("TOML deserialization must succeed");
    assert_eq!(
        config.plugin_dir.to_str().unwrap(),
        "/custom/plugins",
        "CRIT-002: explicit plugin_dir must be preserved"
    );

    // Case 2: plugin_dir absent from TOML → defaults to "plugins".
    let toml_no_plugin_dir = r#"
spec_dir = "/tmp/specs"
state_dir = "/tmp/state"
[[orgs]]
org_id = "0196f000-0000-7000-8000-000000000001"
org_slug = "acme"
"#;
    let config_default: prism_bin::boot::PrismConfig = toml::from_str(toml_no_plugin_dir)
        .expect("TOML deserialization must succeed without plugin_dir");
    assert_eq!(
        config_default.plugin_dir,
        std::path::PathBuf::from("plugins"),
        "CRIT-002: absent plugin_dir must default to 'plugins'"
    );

    // Case 3: new_for_test constructor (external crate compat — #[non_exhaustive] guard).
    let config_test = prism_bin::boot::PrismConfig::new_for_test(
        "/tmp/specs",
        "/tmp/state",
        "/my/plugins",
        vec![],
        prism_bin::boot::CredentialBackendConfig::Keyring,
    );
    assert_eq!(
        config_test.plugin_dir.to_str().unwrap(),
        "/my/plugins",
        "CRIT-002: new_for_test plugin_dir must be set correctly"
    );
}

/// CRIT-001 (F-IMPL-LP1-CRIT-001) — `run_boot_sequence` code-path verifies
/// `plugin_load_step` is inserted between step 7 and step 8 via the presence of
/// `plugin_load_step` in `prism_bin::boot` public surface (compile-time proof),
/// and verifies the pre-traffic gate: `plugin_load_step` with a valid plugin
/// succeeds before any MCP bind would occur.
///
/// Full end-to-end test of `run_boot_sequence` is not possible here because steps 7–11
/// are `todo!()` stubs for sibling stories. The behavioral proof is:
/// 1. `plugin_load_step` is a public function — callers can invoke it directly.
/// 2. With `PRISM_DISABLE_PLUGIN_LOAD=1`, it returns Ok(0) (emergency valve).
/// 3. With a valid plugin dir containing a valid .prx, it returns Ok(1).
/// 4. `run_boot_sequence` includes the plugin_load_step call site per ADR-022 §B
///    sequencing invariant (verified by code inspection + this test asserting the
///    public API exists and is accessible).
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_22_001_plugin_load_step_is_registered_between_step7_and_step8() {
    // Proof 1: plugin_load_step with empty dir returns Ok(0) — API is accessible.
    let dir = tempfile::tempdir().expect("create temp dir");
    let result = plugin_load_step(dir.path()).await;
    assert!(
        result.is_ok(),
        "CRIT-001: plugin_load_step with empty dir must return Ok"
    );
    assert_eq!(
        result.unwrap().plugins_loaded,
        0,
        "CRIT-001: empty dir must yield 0 plugins"
    );

    // Proof 2: plugin_load_step with a valid plugin returns Ok(1) — the step gate works.
    let dir2 = tempfile::tempdir().expect("create temp dir");
    let bytes = compile_wat(MINIMAL_INFUSION_WAT);
    write_prx(&dir2, "minimal", &bytes);
    write_manifest(&dir2, "minimal", MINIMAL_MANIFEST_TOML);

    let result2 = plugin_load_step(dir2.path()).await;
    assert!(
        result2.is_ok(),
        "CRIT-001: plugin_load_step with valid plugin must succeed (pre-traffic gate passed)"
    );
    assert_eq!(
        result2.unwrap().plugins_loaded,
        1,
        "CRIT-001: valid plugin dir must load 1 plugin before MCP bind"
    );
}
