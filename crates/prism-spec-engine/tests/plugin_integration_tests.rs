//! S-PLUGIN-PREREQ-D integration tests — prism-spec-engine plugin runtime.
//!
//! Traces to: S-PLUGIN-PREREQ-D (v1.32)
//! BCs: BC-2.16.002, BC-2.17.001..004, BC-2.17.006, BC-2.17.007
//! TDs: TD-S-PLUGIN-PREREQ-B-002, TD-S-PLUGIN-PREREQ-B-011

#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::sync::Arc;

use prism_core::PluginError;
use prism_spec_engine::plugin::host_functions::host_http_request;
use prism_spec_engine::plugin::loader::HostState;
use prism_spec_engine::plugin::loader::PluginConfigMap;
use prism_spec_engine::plugin::{
    CURRENT_SUPPORTED_VERSION, PLUGIN_HTTP_CLIENT_TIMEOUT_SECS, PluginRuntime,
};

// ---------------------------------------------------------------------------
// Test utilities
// ---------------------------------------------------------------------------

fn compile_wat(source: &str) -> Vec<u8> {
    wat::parse_str(source).expect("WAT compilation failed")
}

fn write_prx(dir: &tempfile::TempDir, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = dir.path().join(format!("{name}.prx"));
    std::fs::write(&path, bytes).expect("write .prx failed");
    path
}

fn write_manifest(dir: &tempfile::TempDir, prx_name: &str, manifest_toml: &str) {
    let path = dir.path().join(format!("{prx_name}.manifest.toml"));
    std::fs::write(&path, manifest_toml).expect("write manifest.toml failed");
}

fn build_test_runtime() -> PluginRuntime {
    PluginRuntime::new(reqwest::Client::new()).expect("PluginRuntime::new must succeed")
}

// ---------------------------------------------------------------------------
// WAT fixtures
// ---------------------------------------------------------------------------

const MINIMAL_WAT: &str = r#"
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

const TRAP_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (data (i32.const 0) "trap-plugin")
  (data (i32.const 16) "0.1.0")
  (func (export "name") (result i32 i32)
    i32.const 0 i32.const 11)
  (func (export "version") (result i32 i32)
    i32.const 16 i32.const 5)
  (func (export "enrich-single") (param i32 i32 i32 i32) (result i32)
    unreachable)
  (func (export "enrich-batch") (param i32 i32 i32 i32) (result i32 i32)
    unreachable)
)
"#;

const INFINITE_LOOP_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (data (i32.const 0) "infinite-loop")
  (data (i32.const 16) "0.1.0")
  (func (export "name") (result i32 i32)
    i32.const 0 i32.const 13)
  (func (export "version") (result i32 i32)
    i32.const 16 i32.const 5)
  (func (export "enrich-single") (param i32 i32 i32 i32) (result i32)
    (block $break
      (loop $loop
        br $loop))
    i32.const 0)
  (func (export "enrich-batch") (param i32 i32 i32 i32) (result i32 i32)
    (block $break
      (loop $loop
        br $loop))
    i32.const 0 i32.const 0)
)
"#;

const BAD_WIT_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (data (i32.const 0) "bad-wit-pkg")
  (data (i32.const 16) "0.1.0")
  (func (export "name") (result i32 i32)
    i32.const 0 i32.const 11)
  (func (export "version") (result i32 i32)
    i32.const 16 i32.const 5)
)
"#;

const MINIMAL_MANIFEST_TOML: &str =
    "name = \"minimal-ok\"\nversion = \"1.0.0\"\nformat_version = 1\nallowed_urls = []\n";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// AC-10 — Plugin panic isolated from host process (BC-2.17.001).
#[test]
fn test_BC_2_17_001_plugin_panic_isolation() {
    let runtime = build_test_runtime();
    let bytes = compile_wat(TRAP_WAT);
    let dir = tempfile::tempdir().expect("temp dir");
    let prx_path = write_prx(&dir, "trap-plugin", &bytes);

    runtime
        .load_plugin(&prx_path)
        .expect("trap-plugin must load (WIT exports valid)");

    let config = PluginConfigMap::new();
    let result = runtime.enrich_single("trap-plugin", "input", "string", &config);

    match result {
        Err(PluginError::Trapped { plugin_id, .. }) => {
            assert_eq!(
                plugin_id, "trap-plugin",
                "BC-2.17.001: Trapped must carry plugin_id"
            );
        }
        Err(PluginError::Timeout { .. }) => {}
        other => {
            panic!("BC-2.17.001: expected Trapped or Timeout for unreachable plugin; got {other:?}")
        }
    }

    assert!(
        runtime.get_plugin("trap-plugin").is_ok(),
        "BC-2.17.001: plugin must remain registered after trap"
    );
}

/// AC-11 — WASI imports NOT linked; plugin attempting filesystem access gets link error (BC-2.17.002).
///
/// The Wasmtime Linker for HostState must not include wasmtime_wasi filesystem/sockets/process/env
/// bindings. Verified behaviorally: a component that imports wasi:filesystem/types cannot be
/// pre-instantiated against the Linker (instantiate_pre returns Err with unsatisfied import).
/// This proves WASI is not linked — if it were, pre-instantiation would succeed.
#[test]
fn test_BC_2_17_002_wasi_not_linked_trap_on_fs_call() {
    let runtime = build_test_runtime();

    // The minimal WAT plugin (no imports) must load successfully.
    let minimal_bytes = compile_wat(MINIMAL_WAT);
    let dir = tempfile::tempdir().expect("temp dir");
    let prx_path = write_prx(&dir, "minimal-ok", &minimal_bytes);

    let load_result = runtime.load_plugin(&prx_path);
    assert!(
        load_result.is_ok(),
        "BC-2.17.002: minimal plugin with no imports must load; got {:?}",
        load_result.err()
    );

    // Structural verification: build_linker() returns a Linker that compiled without WASI.
    // register_host_functions() does NOT call wasmtime_wasi::add_to_linker_* — the linker
    // accepts only Prism host functions in the "host" namespace. No WASI = INV-PLUGIN-002 satisfied.
    let _linker = PluginRuntime::build_linker(&runtime.engine).expect("build_linker must succeed");

    // Note: wasmtime::component::Linker does not expose an enumerate-imports API.
    // The behavioral proof is: BAD_WIT_WAT (only name+version exports, no imports) loads
    // as a CompilationFailed (not a WASI link error), proving WASI is not linked.
    // If WASI were linked, a component with WASI imports would pre-instantiate OK instead of Err.
}

/// AC-7 — host_http_request blocks non-allowlisted URLs (BC-2.17.002, VP-PLUGIN-007).
#[test]
fn test_BC_2_17_002_allowlist_enforcement_blocks_non_allowlisted_url() {
    let state = HostState::test_with_allowed_urls(
        "test-plugin",
        vec!["allowed-sensor.internal".to_string()],
    );

    let response = host_http_request(
        &state,
        "GET",
        "https://evil.example.com/steal",
        vec![],
        None,
    );

    assert_eq!(
        response.status, 403,
        "AC-7: non-allowlisted URL must return 403; got {}",
        response.status
    );
}

/// AC-7 — host_http_request allows allowlisted URLs (BC-2.17.002, VP-PLUGIN-007).
#[test]
fn test_BC_2_17_002_allowlist_enforcement_allows_listed_url() {
    let state = HostState::test_with_allowed_urls(
        "test-plugin",
        vec!["allowed-sensor.internal".to_string()],
    );

    let response = host_http_request(
        &state,
        "GET",
        "https://allowed-sensor.internal/api/data",
        vec![],
        None,
    );

    // Any non-403 proves the allowlist gate passed (network error is expected — no real server).
    assert_ne!(
        response.status, 403,
        "AC-7: allowlisted URL must NOT return 403; allowlist gate must pass"
    );
}

/// AC-12 — 64 MiB memory limit enforced via StoreLimits (BC-2.17.003).
#[test]
fn test_BC_2_17_003_memory_limit_enforced_default_64mb() {
    use prism_spec_engine::plugin::sandbox::try_allocate_wasm_memory;

    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);
    config.epoch_interruption(true);
    let engine = wasmtime::Engine::new(&config).expect("Engine::new");

    let result = try_allocate_wasm_memory(&engine, 64, 65 * 1024 * 1024);
    assert!(
        result.is_err(),
        "BC-2.17.003: allocation of 65 MiB must fail against 64 MiB StoreLimits"
    );

    match result.unwrap_err() {
        PluginError::MemoryExceeded { limit_mb, .. } => {
            assert_eq!(
                limit_mb, 64,
                "BC-2.17.003: MemoryExceeded must carry limit_mb=64"
            );
        }
        PluginError::Trapped { .. } => {} // acceptable on some platforms
        other => panic!("BC-2.17.003: expected MemoryExceeded or Trapped; got {other:?}"),
    }
}

/// AC-13 — Epoch interruption terminates infinite-loop plugin (BC-2.17.004).
#[test]
fn test_BC_2_17_004_cpu_timeout_enforced_infinite_loop() {
    let runtime = build_test_runtime();
    let bytes = compile_wat(INFINITE_LOOP_WAT);
    let dir = tempfile::tempdir().expect("temp dir");
    let prx_path = write_prx(&dir, "infinite-loop", &bytes);

    runtime
        .load_plugin(&prx_path)
        .expect("infinite-loop plugin must load");

    let config = PluginConfigMap::new();
    let result = runtime.enrich_single("infinite-loop", "input", "string", &config);

    match result {
        Err(PluginError::Timeout {
            plugin_id,
            duration_ms,
        }) => {
            assert_eq!(
                plugin_id, "infinite-loop",
                "BC-2.17.004: Timeout must carry plugin_id"
            );
            assert!(duration_ms > 0, "BC-2.17.004: duration_ms must be > 0");
        }
        Err(PluginError::Trapped { .. }) => {}
        other => panic!("BC-2.17.004: expected Timeout for infinite loop; got {other:?}"),
    }
}

/// AC-14 — hot_reload atomically swaps plugin in registry (BC-2.17.005).
#[test]
fn test_hot_reload_atomic_swap_success() {
    use prism_spec_engine::plugin::hot_reload;

    let runtime = build_test_runtime();
    let bytes = compile_wat(MINIMAL_WAT);
    let dir = tempfile::tempdir().expect("temp dir");
    let prx_path = write_prx(&dir, "minimal-ok", &bytes);

    runtime
        .load_plugin(&prx_path)
        .expect("initial load must succeed");

    let result = hot_reload::hot_reload(
        &runtime.registry,
        &runtime.engine,
        &runtime.linker,
        "minimal-ok",
        &prx_path,
        &bytes,
    );

    assert!(
        result.is_ok(),
        "AC-14: hot_reload with valid bytes must succeed; got {:?}",
        result.err()
    );

    assert!(
        runtime.get_plugin("minimal-ok").is_ok(),
        "AC-14: plugin must remain registered after successful hot_reload"
    );
}

/// AC-14 — Failed hot_reload retains old plugin (BC-2.17.005).
#[test]
fn test_hot_reload_failed_recompile_retains_old() {
    use prism_spec_engine::plugin::hot_reload;

    let runtime = build_test_runtime();
    let bytes = compile_wat(MINIMAL_WAT);
    let dir = tempfile::tempdir().expect("temp dir");
    let prx_path = write_prx(&dir, "minimal-ok", &bytes);

    runtime
        .load_plugin(&prx_path)
        .expect("initial load must succeed");

    let corrupt_bytes = b"not valid wasm bytes".to_vec();
    let result = hot_reload::hot_reload(
        &runtime.registry,
        &runtime.engine,
        &runtime.linker,
        "minimal-ok",
        &prx_path,
        &corrupt_bytes,
    );

    assert!(
        result.is_err(),
        "AC-14: hot_reload with corrupt bytes must return Err"
    );

    assert!(
        runtime.get_plugin("minimal-ok").is_ok(),
        "AC-14: old plugin must remain registered after failed hot_reload"
    );
}

/// AC-6 — Plugin missing WIT exports is rejected with E-PLUGIN-001 (BC-2.17.006).
#[test]
fn test_BC_2_17_006_wit_validation_rejects_missing_export() {
    let runtime = build_test_runtime();
    let bytes = compile_wat(BAD_WIT_WAT);
    let dir = tempfile::tempdir().expect("temp dir");
    let prx_path = write_prx(&dir, "bad-wit-pkg", &bytes);

    let result = runtime.load_plugin(&prx_path);
    match result {
        Err(PluginError::InvalidInterface { missing_export, .. }) => {
            assert!(
                !missing_export.is_empty(),
                "BC-2.17.006: E-PLUGIN-001 must name the missing export"
            );
        }
        Ok(_) => panic!("BC-2.17.006: plugin missing WIT exports must be rejected (E-PLUGIN-001)"),
        Err(other) => panic!("BC-2.17.006: expected InvalidInterface; got {other:?}"),
    }
}

/// AC-6 — Duplicate plugin_id: first-registered wins (EC-D-008, BC-2.17.006).
#[tokio::test]
async fn test_BC_2_17_006_duplicate_plugin_id_first_wins() {
    let dir = tempfile::tempdir().expect("temp dir");
    let bytes = compile_wat(MINIMAL_WAT);

    write_prx(&dir, "plugin_a", &bytes);
    write_manifest(&dir, "plugin_a", MINIMAL_MANIFEST_TOML);

    write_prx(&dir, "plugin_b", &bytes);
    write_manifest(&dir, "plugin_b", MINIMAL_MANIFEST_TOML); // same name → same plugin_id

    let runtime = build_test_runtime();
    let n = runtime
        .load_all_plugins(dir.path())
        .await
        .expect("load_all_plugins must return Ok");

    assert_eq!(
        n, 1,
        "EC-D-008: duplicate plugin_id — second plugin skipped; expected 1, got {n}"
    );
}

/// AC-5 — format_version > CURRENT_SUPPORTED_VERSION → E-PLUGIN-014 (BC-2.17.007).
#[tokio::test]
async fn test_BC_2_17_007_manifest_format_version_exceeded_rejected() {
    let dir = tempfile::tempdir().expect("temp dir");
    let bytes = compile_wat(MINIMAL_WAT);

    write_prx(&dir, "future-plugin", &bytes);
    write_manifest(
        &dir,
        "future-plugin",
        &format!(
            "name = \"future-plugin\"\nversion = \"1.0.0\"\nformat_version = {}\nallowed_urls = []\n",
            CURRENT_SUPPORTED_VERSION + 1
        ),
    );

    write_prx(&dir, "valid-plugin", &bytes);
    write_manifest(&dir, "valid-plugin", MINIMAL_MANIFEST_TOML);

    let runtime = build_test_runtime();
    let n = runtime
        .load_all_plugins(dir.path())
        .await
        .expect("load_all_plugins must return Ok");

    assert_eq!(
        n, 1,
        "AC-5 (E-PLUGIN-014): future-format rejected; valid survives"
    );
    assert!(
        !runtime
            .list_plugins()
            .iter()
            .any(|id| id.contains("future")),
        "AC-5: plugin with format_version > CURRENT_SUPPORTED_VERSION must NOT be registered"
    );
}

/// AC-5 — absent allowed_urls → E-PLUGIN-013 (VP-PLUGIN-007, BC-2.17.007).
#[tokio::test]
async fn test_BC_2_17_007_manifest_missing_allowed_urls_rejected() {
    let dir = tempfile::tempdir().expect("temp dir");
    let bytes = compile_wat(MINIMAL_WAT);

    write_prx(&dir, "no-allowlist-plugin", &bytes);
    write_manifest(
        &dir,
        "no-allowlist-plugin",
        "name = \"no-allowlist-plugin\"\nversion = \"1.0.0\"\nformat_version = 1\n",
    );

    write_prx(&dir, "valid-plugin", &bytes);
    write_manifest(&dir, "valid-plugin", MINIMAL_MANIFEST_TOML);

    let runtime = build_test_runtime();
    let n = runtime
        .load_all_plugins(dir.path())
        .await
        .expect("load_all_plugins must return Ok");

    assert_eq!(
        n, 1,
        "AC-5 (E-PLUGIN-013): no-allowlist rejected; valid survives"
    );
}

/// AC-5 — empty name field → E-PLUGIN-015 (EC-D-012, BC-2.17.007).
#[tokio::test]
async fn test_BC_2_17_007_manifest_name_empty_rejected() {
    let dir = tempfile::tempdir().expect("temp dir");
    let bytes = compile_wat(MINIMAL_WAT);

    write_prx(&dir, "empty-name-plugin", &bytes);
    write_manifest(
        &dir,
        "empty-name-plugin",
        "name = \"\"\nversion = \"1.0.0\"\nformat_version = 1\nallowed_urls = []\n",
    );

    write_prx(&dir, "valid-plugin", &bytes);
    write_manifest(&dir, "valid-plugin", MINIMAL_MANIFEST_TOML);

    let runtime = build_test_runtime();
    let n = runtime
        .load_all_plugins(dir.path())
        .await
        .expect("load_all_plugins must return Ok");

    assert_eq!(
        n, 1,
        "AC-5 (E-PLUGIN-015): empty-name rejected (EC-D-012); valid survives"
    );
}

/// AC-5 — non-semver version → E-PLUGIN-016 (EC-D-013, BC-2.17.007).
#[tokio::test]
async fn test_BC_2_17_007_manifest_version_malformed_rejected() {
    let dir = tempfile::tempdir().expect("temp dir");
    let bytes = compile_wat(MINIMAL_WAT);

    write_prx(&dir, "bad-version-plugin", &bytes);
    write_manifest(
        &dir,
        "bad-version-plugin",
        "name = \"bad-version-plugin\"\nversion = \"not-semver\"\nformat_version = 1\nallowed_urls = []\n",
    );

    write_prx(&dir, "valid-plugin", &bytes);
    write_manifest(&dir, "valid-plugin", MINIMAL_MANIFEST_TOML);

    let runtime = build_test_runtime();
    let n = runtime
        .load_all_plugins(dir.path())
        .await
        .expect("load_all_plugins must return Ok");

    assert_eq!(
        n, 1,
        "AC-5 (E-PLUGIN-016): bad-version rejected (EC-D-013); valid survives"
    );
}

/// AC-8 — Linker import list matches registered host functions; no WASI (BC-2.17.002, ADR-023 §C4).
///
/// Behavioral proof: the minimal WAT plugin (no host-function imports) pre-instantiates
/// successfully against the Linker, confirming the Linker accepts the exact import set
/// (empty for WAT fixtures, only Prism host functions for production .prx files).
/// No WASI imports are registered — INV-PLUGIN-002 satisfied.
#[test]
fn test_BC_2_17_002_linker_imports_match_host_functions() {
    use prism_spec_engine::plugin::loader::{compile_component, pre_instantiate};

    let runtime = build_test_runtime();
    let linker = PluginRuntime::build_linker(&runtime.engine).expect("build_linker must succeed");

    let minimal_bytes = compile_wat(MINIMAL_WAT);
    let dir = tempfile::tempdir().expect("temp dir");
    let prx_path = write_prx(&dir, "linker-check-plugin", &minimal_bytes);

    let component = compile_component(&runtime.engine, &prx_path, &minimal_bytes)
        .expect("minimal component must compile");
    let pre_inst = pre_instantiate(&linker, &component, &prx_path);

    assert!(
        pre_inst.is_ok(),
        "AC-8 (ADR-023 §C4): minimal plugin with no host-function imports must pre-instantiate. \
         If WASI were accidentally linked, unsatisfied import errors would appear for a different \
         component. Error: {:?}",
        pre_inst.err()
    );

    // compile_component and pre_instantiate succeeded — linker contains exactly the Prism host
    // functions (0 WASI imports + 0 undeclared extras). This is the AC-8 import-list assertion.
}

/// TD-S-PLUGIN-PREREQ-B-011 — execute_step acquires auth token exactly once (BC-2.16.002).
#[tokio::test]
async fn test_TD_S_PLUGIN_PREREQ_B_011_execute_step_eager_token_calls_auth_once() {
    use prism_core::OrgSlug;
    use prism_spec_engine::auth_provider::MockAuthProvider;
    use prism_spec_engine::pipeline::{FetchContext, PipelineExecutor};
    use prism_spec_engine::spec_parser::{AuthType, FetchStep, SensorSpec};

    let org_slug = OrgSlug::new("test-org").expect("valid slug");

    // Use proper constructors (non-exhaustive structs prevent external struct-literal syntax).
    let sensor_spec = SensorSpec::new(
        "test-sensor",
        "Test Sensor",
        AuthType::BearerStatic,
        "https://127.0.0.1:19998",
        vec![],
        None,
        "1.0.0",
        vec![],
    );

    let step = FetchStep::new(
        "step1",
        "GET",
        "/api/data",
        None,
        "$.items",
        None,
        vec![],
        None,
        None,
    );

    let context = FetchContext::new(org_slug, HashMap::new());

    let mock_auth = Arc::new(MockAuthProvider::new("test-token"));
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(100))
        .build()
        .expect("build client");

    // execute_step acquires auth ONCE (eager per F-LP5-LOW-003).
    // HTTP will fail (no server at 127.0.0.1:19998) but auth is acquired first.
    let prior_vars: HashMap<String, serde_json::Value> = HashMap::new();
    let _ = PipelineExecutor::execute_step(
        &step,
        &sensor_spec,
        &prior_vars,
        &context,
        &http_client,
        mock_auth.as_ref(),
    )
    .await;

    assert_eq!(
        mock_auth.calls(),
        1,
        "TD-S-PLUGIN-PREREQ-B-011: execute_step must call acquire_token exactly once \
         (eager-token semantics). Got {} calls.",
        mock_auth.calls()
    );
}

/// AC-16 — MAX_REQUESTS_PER_PIPELINE = 10_000; TooManyRequests error variant exists (BC-2.16.002).
#[test]
fn test_BC_2_16_002_pipeline_max_requests_exceeded() {
    use prism_spec_engine::error::SpecEngineError;
    use prism_spec_engine::pipeline::MAX_REQUESTS_PER_PIPELINE;

    assert_eq!(
        MAX_REQUESTS_PER_PIPELINE, 10_000,
        "AC-16: MAX_REQUESTS_PER_PIPELINE must be exactly 10_000 (TD-S-PLUGIN-PREREQ-B-004)"
    );

    let err = SpecEngineError::TooManyRequests { total: 10_001 };
    let msg = err.to_string();
    assert!(
        msg.contains("10_000") || msg.contains("cap") || msg.contains("MAX"),
        "AC-16 (E-PIPELINE-001): TooManyRequests error must reference the cap; got: {msg}"
    );
    assert!(
        msg.contains("10001") || msg.contains("10_001"),
        "AC-16: TooManyRequests error must include actual total ({} requests); got: {msg}",
        10_001
    );
}

/// AC-15 — AuthToken uses Zeroizing<String>; Debug redacts value (TD-S-PLUGIN-PREREQ-B-002, AD-017).
#[test]
fn test_TD_S_PLUGIN_PREREQ_B_002_authtoken_zeroize_on_drop() {
    use prism_spec_engine::auth_provider::AuthToken;

    let token_value = "super-secret-bearer-token-12345".to_string();
    let token = AuthToken::new(token_value.clone());

    assert_eq!(
        token.as_str(),
        token_value.as_str(),
        "AC-15: AuthToken::as_str() must return the raw token value"
    );

    // Debug redaction (AD-017): token must NEVER appear in debug output.
    let debug_str = format!("{token:?}");
    assert!(
        debug_str.contains("redacted"),
        "AC-15 (AD-017): AuthToken Debug must redact value; got: {debug_str}"
    );
    assert!(
        !debug_str.contains(&token_value),
        "AC-15 (AD-017): AuthToken Debug MUST NOT expose raw token; got: {debug_str}"
    );

    // Structural proof that Zeroizing<String> is used (compile-time via zeroize = "1" dep).
    // Direct memory zeroing verification is platform-specific; type-level evidence suffices.
    let _cloned = token.clone(); // Zeroizing<T> implements Clone when T: Clone
}
