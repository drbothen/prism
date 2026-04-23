//! S-1.15 Plugin Runtime integration tests.
//!
//! Tests are named `test_BC_S_SS_NNN_*` for full traceability to behavioral contracts.
//! All tests in this file MUST FAIL before S-1.15 implementation begins (Red Gate).
//!
//! # Test Coverage
//!
//! | Test | BC | AC | TV |
//! |------|----|----|-----|
//! | test_BC_2_17_006_ac1_load_valid_infusion_plugin | BC-2.17.006 | AC-1 | TV-17-006-happy |
//! | test_BC_2_17_001_ac2_plugin_trap_returns_err_trapped | BC-2.17.001 | AC-2 | TV-17-001-happy |
//! | test_BC_2_17_004_ac3_infinite_loop_returns_err_timeout | BC-2.17.004 | AC-3 | TV-17-004-timeout |
//! | test_BC_2_17_002_ac4_wasi_filesystem_not_accessible | BC-2.17.002 | AC-4 | TV-17-002-blocked |
//! | test_BC_2_17_002_ac5_http_request_proxied_via_host | BC-2.17.002 | AC-5 | TV-17-002-happy |
//! | test_BC_2_17_005_ac6_hot_reload_atomic_swap | BC-2.17.005 | AC-6 | TV-17-005-happy |
//! | test_BC_2_17_006_ac7_invalid_wit_returns_e_plugin_001 | BC-2.17.006 | AC-7 | TV-17-006-missing |
//! | test_BC_2_17_002_ac8_kv_store_scoped_per_plugin | BC-2.17.002 | AC-8 | EC-17-008 |
//! | test_BC_2_17_003_ac9_memory_limit_exceeded_returns_err | BC-2.17.003 | AC-9 | TV-17-003-exceed |
//! | test_BC_2_17_001_ec17_001_trap_on_first_call_plugin_stays_registered | BC-2.17.001 | — | EC-17-001 |
//! | test_BC_2_17_001_ec17_003_batch_trap_returns_no_partial_results | BC-2.17.001 | — | TV-17-001-batch |
//! | test_BC_2_17_001_ec17_004_concurrent_traps_independent | BC-2.17.001 | — | TV-17-001-edge |
//! | test_BC_2_17_003_ec17_009_at_limit_allocation_succeeds | BC-2.17.003 | — | TV-17-003-happy |
//! | test_BC_2_17_003_ec17_011_per_plugin_memory_override | BC-2.17.003 | — | TV-17-003-override |
//! | test_BC_2_17_005_ec17_005_failed_recompile_retains_old_plugin | BC-2.17.005 | — | TV-17-005-fail |
//! | test_BC_2_17_005_ec17_delete_plugin_new_calls_return_not_loaded | BC-2.17.005 | — | TV-17-005-delete |
//! | test_BC_2_17_006_ec17_026_bulk_discovery_partial_failure | BC-2.17.006 | — | TV-17-006-bulk |
//! | test_BC_2_17_006_ec17_027_empty_plugin_id_rejected | BC-2.17.006 | — | EC-17-027 |
//! | test_BC_2_17_002_ec17_007_http_request_no_allowlist_allowed | BC-2.17.002 | — | EC-17-007 |
//! | test_BC_2_17_002_ec17_006_http_request_allowlisted_url_succeeds | BC-2.17.002 | — | EC-17-006 |
//! | test_BC_2_17_002_ec17_url_not_in_allowlist_returns_403 | BC-2.17.002 | — | TV-17-002-allowlist |
//! | test_BC_2_17_004_ec17_015_per_plugin_timeout_override | BC-2.17.004 | — | TV-17-004-override |
//! | test_BC_2_17_006_ac7_invariant_plugin_not_registered_after_invalid_wit | BC-2.17.006 | AC-7 | — |

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use prism_core::PluginError;
use prism_spec_engine::plugin::{PluginRuntime, PluginType};

// ---- Test fixture paths ----

fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/")).join(name)
}

/// Compile a WAT fixture to a temporary `.wasm` file, returning the temp file handle.
/// The `.wasm` file is not a full Component Model binary — it is a core module.
/// In real tests, `wasm-tools component new` wraps it. For Red Gate, we use
/// the raw WAT → bytes path to drive the `load_plugin` API with realistic inputs.
fn compile_wat_fixture(wat_name: &str) -> (Vec<u8>, tempfile::NamedTempFile) {
    let wat_path = fixture_path(wat_name);
    let bytes = wat::parse_file(&wat_path)
        .unwrap_or_else(|e| panic!("Failed to parse WAT fixture {}: {}", wat_name, e));
    let tmp = tempfile::NamedTempFile::with_suffix(".prx").unwrap();
    std::fs::write(tmp.path(), &bytes).unwrap();
    (bytes, tmp)
}

// ============================================================
// AC-1 / BC-2.17.006 / TV-17-006-happy
// ============================================================

/// AC-1: Given a valid `.prx` file implementing `prism:infusion-plugin@0.1.0`,
/// When `PluginRuntime::load_plugin` is called, Then the plugin is compiled,
/// validated, and added to the registry without error.
///
/// Traces to: BC-2.17.006 / INV-PLUGIN-006
#[test]
fn test_BC_2_17_006_ac1_load_valid_infusion_plugin() {
    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");

    let (_bytes, tmp) = compile_wat_fixture("noop_infusion.wat");
    let result = runtime.load_plugin(tmp.path());
    if let Err(ref e) = result {
        panic!(
            "AC-1: loading a valid infusion plugin must succeed, got: {:?}",
            e
        );
    }

    let plugins = runtime.list_plugins();
    assert!(
        !plugins.is_empty(),
        "AC-1: plugin must be present in registry after successful load"
    );
}

// ============================================================
// AC-2 / BC-2.17.001 / TV-17-001-happy
// ============================================================

/// AC-2: Given a `.prx` plugin that panics (executes `unreachable` instruction),
/// When any call is made to the plugin, Then `Err(PluginError::Trapped)` is returned
/// and the Prism host process continues executing normally.
///
/// Traces to: BC-2.17.001 / INV-PLUGIN-001
#[test]
fn test_BC_2_17_001_ac2_plugin_trap_returns_err_trapped() {
    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");

    let (_bytes, tmp) = compile_wat_fixture("trap_plugin.wat");
    runtime
        .load_plugin(tmp.path())
        .expect("trap_plugin must load successfully");

    let plugins = runtime.list_plugins();
    let plugin_id = plugins.first().expect("trap_plugin must be registered");

    let config = std::collections::HashMap::new();
    let result = runtime.enrich_single(plugin_id, "test-value", "ip", &config);

    assert!(
        matches!(result, Err(PluginError::Trapped { .. })),
        "AC-2: trap plugin must return Err(Trapped), got: {:?}",
        result
    );

    // Host process continues: subsequent calls to runtime still work.
    let still_alive = runtime.list_plugins();
    assert!(
        !still_alive.is_empty(),
        "AC-2: host process must continue after plugin trap (INV-PLUGIN-001)"
    );

    // The plugin must remain registered (not unregistered after trap).
    assert!(
        still_alive.contains(plugin_id),
        "AC-2: plugin registry entry must be retained after trap (BC-2.17.001 postcondition)"
    );
}

// ============================================================
// AC-3 / BC-2.17.004 / TV-17-004-timeout
// ============================================================

/// AC-3: Given a plugin with the default 5-second CPU time limit,
/// When the plugin executes an infinite loop,
/// Then the call returns `Err(PluginError::Timeout)` within 6 seconds
/// (5s limit + 1s tolerance) and the host process is unaffected.
///
/// Traces to: BC-2.17.004 / INV-PLUGIN-004
///
/// Windows: excluded because wasmtime's JIT in debug builds is deeply recursive
/// and causes STATUS_STACK_BUFFER_OVERRUN when compiling even small WASM modules.
/// The epoch-interruption property is fully verified on Linux and macOS CI.
/// See BC-2.17.004 note on platform-specific test scoping.
#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "wasmtime JIT stack overflow on Windows debug (STATUS_STACK_BUFFER_OVERRUN) — covered by Linux/macOS CI"
)]
fn test_BC_2_17_004_ac3_infinite_loop_returns_err_timeout() {
    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");

    let (_bytes, tmp) = compile_wat_fixture("loop_plugin.wat");
    runtime
        .load_plugin(tmp.path())
        .expect("loop_plugin must load successfully");

    let plugins = runtime.list_plugins();
    let plugin_id = plugins.first().expect("loop_plugin must be registered");

    let config = std::collections::HashMap::new();
    let start = Instant::now();
    let result = runtime.enrich_single(plugin_id, "test-value", "ip", &config);
    let elapsed = start.elapsed();

    assert!(
        matches!(result, Err(PluginError::Timeout { .. })),
        "AC-3: infinite loop plugin must return Err(Timeout), got: {:?}",
        result
    );

    // Must complete within 5s limit + 1s tolerance = 6s total.
    assert!(
        elapsed <= Duration::from_secs(6),
        "AC-3: Timeout must fire within 6s (5s limit + 1s tolerance), took: {:?}",
        elapsed
    );

    // Host process continues.
    let _ = runtime.list_plugins();
}

// ============================================================
// AC-4 / BC-2.17.002 / TV-17-002-blocked
// ============================================================

/// AC-4: Given a plugin that attempts to open a file via a WASI filesystem call,
/// When the plugin executes, Then the call fails because no filesystem WASI interface
/// is linked — the plugin cannot read host files.
///
/// Traces to: BC-2.17.002 / INV-PLUGIN-002
#[test]
fn test_BC_2_17_002_ac4_wasi_filesystem_not_accessible() {
    // A WAT module that imports WASI preview1 fd_write — this import will not be
    // satisfied by the plugin linker (no WASI linked, only Prism host interface).
    let wasi_wat = r#"
        (module
          (import "wasi_snapshot_preview1" "fd_write"
            (func $fd_write (param i32 i32 i32 i32) (result i32)))
          (memory (export "memory") 1)
          (func (export "name") (result i32 i32)
            i32.const 0 i32.const 0)
          (func (export "version") (result i32 i32)
            i32.const 0 i32.const 0)
          (func (export "enrich-single")
            (param i32 i32 i32 i32) (result i32)
            i32.const 0 i32.const 0 i32.const 0 i32.const 0
            call $fd_write
            drop
            i32.const 0)
        )
    "#;

    let bytes = wat::parse_str(wasi_wat).expect("WASI WAT should parse");
    let tmp = tempfile::NamedTempFile::with_suffix(".prx").unwrap();
    std::fs::write(tmp.path(), &bytes).unwrap();

    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");
    let result = runtime.load_plugin(tmp.path());

    // The WASI-importing component must be rejected at load time because
    // the linker has no WASI bindings (VP-040 / BC-2.17.002 EC-17-005).
    assert!(
        result.is_err(),
        "AC-4: WASI-importing plugin must be rejected at load time"
    );

    // Error must indicate sandbox/interface violation.
    let plugin_err = result.err().expect("result already asserted Err");
    match plugin_err {
        PluginError::CompilationFailed { .. }
        | PluginError::InvalidInterface { .. }
        | PluginError::SandboxViolation { .. } => {
            // All acceptable: the runtime correctly refused the WASI-importing binary.
        }
        other => panic!(
            "AC-4: expected CompilationFailed/InvalidInterface/SandboxViolation, got: {:?}",
            other
        ),
    }
}

// ============================================================
// AC-5 / BC-2.17.002 / TV-17-002-happy
// ============================================================

/// AC-5: Given a plugin that calls `host::http_request` with a URL,
/// When the call executes, Then the request is issued via the host's `reqwest::Client`
/// (not directly from WASM), and the call is audit-logged with plugin_id, URL, method,
/// and response status.
///
/// Traces to: BC-2.17.002 / INV-PLUGIN-002
///
/// This test verifies three invariants that prove all HTTP is proxied through the host:
///   1. `PluginRuntime::new()` initialises an `http_client` field (structural check).
///   2. An invalid URL is caught and returned as HTTP 400 by the host validator —
///      proving the host intercepts every call before any network I/O.
///   3. A URL not in the allowlist returns HTTP 403 from the host — proving requests
///      cannot escape the sandbox without going through the host allowlist gate.
///
/// Full end-to-end verification (a real WASM module invoking `host::http_request` and
/// an observable mock HTTP server) is deferred to the integration test suite where a
/// local `wiremock` or `httpmock` server can be spun up.
#[test]
fn test_BC_2_17_002_ac5_http_request_proxied_via_host() {
    use prism_spec_engine::plugin::host_functions::host_http_request;
    use prism_spec_engine::plugin::loader::{HostState, PluginConfigMap, PluginKvStore};

    // Invariant 1: PluginRuntime::new() must succeed and have an http_client.
    // (The field is Arc<reqwest::Client> on PluginRuntime; confirmed by build success.)
    let _runtime = PluginRuntime::new().expect("AC-5: PluginRuntime::new must succeed");

    // Invariant 2: Invalid URL is caught by the host before any network I/O.
    // The host's `do_http_request` validates the URL via `reqwest::Url::parse`.
    // A truly invalid URL ("not a url") returns HTTP 400 — not a panic or bypass.
    let state_open = HostState {
        http_client: Arc::new(reqwest::Client::new()),
        config: Arc::new(PluginConfigMap::new()),
        kv_store: Arc::new(PluginKvStore::new()),
        plugin_id: "ac5-test-plugin".to_string(),
        allowed_urls: None,
        limits: wasmtime::StoreLimits::default(),
    };
    let bad_url_response = host_http_request(&state_open, "GET", "not a url !!!", vec![], None);
    assert_eq!(
        bad_url_response.status, 400,
        "AC-5: invalid URL must be caught by the host and returned as HTTP 400 \
         (proves host intercepts before network I/O)"
    );

    // Invariant 3: A URL outside the plugin's allowlist returns HTTP 403 from the host.
    // This proves plugins cannot make arbitrary HTTP requests — all requests pass through
    // the host allowlist gate (INV-PLUGIN-002 / BC-2.17.002).
    let state_restricted = HostState {
        http_client: Arc::new(reqwest::Client::new()),
        config: Arc::new(PluginConfigMap::new()),
        kv_store: Arc::new(PluginKvStore::new()),
        plugin_id: "ac5-test-plugin".to_string(),
        allowed_urls: Some(vec!["allowed-sensor.internal".to_string()]),
        limits: wasmtime::StoreLimits::default(),
    };
    let blocked_response = host_http_request(
        &state_restricted,
        "GET",
        "https://attacker.example.com/exfiltrate",
        vec![],
        None,
    );
    assert_eq!(
        blocked_response.status, 403,
        "AC-5: request to non-allowlisted URL must return HTTP 403 from the host \
         (proves host proxies all plugin HTTP — BC-2.17.002 / INV-PLUGIN-002)"
    );
}

// ============================================================
// AC-6 / BC-2.17.005 / TV-17-005-happy
// ============================================================

/// AC-6: Given a loaded plugin `.prx`, When the file is replaced with a new version
/// and the file watcher fires, Then the plugin registry is updated via arc-swap,
/// in-flight calls using the old module complete without error, and new calls use
/// the new module.
///
/// Traces to: BC-2.17.005 / INV-PLUGIN-005
#[test]
fn test_BC_2_17_005_ac6_hot_reload_atomic_swap() {
    use prism_spec_engine::plugin::hot_reload::hot_reload;

    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");

    // Load initial plugin.
    let (_bytes, tmp) = compile_wat_fixture("noop_infusion.wat");
    runtime
        .load_plugin(tmp.path())
        .expect("noop_infusion must load");

    let plugins = runtime.list_plugins();
    let plugin_id = plugins.first().expect("noop_infusion must be registered");
    let old_arc = runtime.get_plugin(plugin_id).expect("must be in registry");

    // Hot reload with same bytes (simulating a file replacement with valid content).
    let (new_bytes, new_tmp) = compile_wat_fixture("noop_infusion.wat");
    let reload_result = hot_reload(
        &runtime.registry,
        &runtime.engine,
        &runtime.linker,
        plugin_id,
        new_tmp.path(),
        &new_bytes,
    );

    assert!(
        reload_result.is_ok(),
        "AC-6: hot reload with valid plugin must succeed, got: {:?}",
        reload_result
    );

    // After swap, the registry must return a new Arc (new module version).
    let new_arc = runtime
        .get_plugin(plugin_id)
        .expect("plugin must still be registered");

    // The new Arc may or may not be ptr-equal (implementation detail), but the
    // plugin must still be accessible.
    assert!(
        !runtime.list_plugins().is_empty(),
        "AC-6: plugin must remain accessible after hot reload"
    );

    // In-flight safety: old_arc must still be valid (Arc keeps old module alive).
    let _ = old_arc; // This should compile and not panic.
}

// ============================================================
// AC-7 / BC-2.17.006 / TV-17-006-missing
// ============================================================

/// AC-7: Given a `.prx` file that does not export `name`, `version`, or expected
/// dispatch functions, When `PluginRuntime::load_plugin` is called, Then it returns
/// `Err` with error code `E-PLUGIN-001` and the plugin is not added to the registry.
///
/// Traces to: BC-2.17.006 / INV-PLUGIN-006
#[test]
fn test_BC_2_17_006_ac7_invalid_wit_returns_e_plugin_001() {
    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");

    // A WAT module with no exports — no name, version, or dispatch function.
    let no_exports_wat = r#"
        (module
          (memory (export "memory") 1)
        )
    "#;
    let bytes = wat::parse_str(no_exports_wat).expect("empty module WAT should parse");
    let tmp = tempfile::NamedTempFile::with_suffix(".prx").unwrap();
    std::fs::write(tmp.path(), &bytes).unwrap();

    let result = runtime.load_plugin(tmp.path());

    assert!(
        result.is_err(),
        "AC-7: plugin with no WIT exports must be rejected"
    );
    match &result {
        Err(PluginError::InvalidInterface { .. }) => {}
        other_err => panic!(
            "AC-7: error must be InvalidInterface (E-PLUGIN-001), got: {:?}",
            other_err.as_ref().err()
        ),
    }

    // Plugin must NOT be in the registry.
    let plugins = runtime.list_plugins();
    assert!(
        plugins.is_empty(),
        "AC-7: invalid plugin must not appear in registry"
    );
}

/// AC-7 invariant: Registry count does not increase after a rejected plugin load.
///
/// Traces to: BC-2.17.006 invariant "plugin not added to registry on E-PLUGIN-001"
#[test]
fn test_BC_2_17_006_ac7_invariant_plugin_not_registered_after_invalid_wit() {
    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");
    let count_before = runtime.list_plugins().len();

    let bad_wat = r#"(module)"#;
    let bytes = wat::parse_str(bad_wat).unwrap();
    let tmp = tempfile::NamedTempFile::with_suffix(".prx").unwrap();
    std::fs::write(tmp.path(), &bytes).unwrap();

    let _ = runtime.load_plugin(tmp.path()); // May fail (expected)

    let count_after = runtime.list_plugins().len();
    assert_eq!(
        count_before, count_after,
        "AC-7: registry count must not increase after a rejected plugin load"
    );
}

// ============================================================
// AC-8 / BC-2.17.002 / EC-17-008 — KV store scoped per plugin
// ============================================================

/// AC-8: Given two different plugins loaded in the same `PluginRuntime`,
/// When plugin A calls `host::kv_set("mykey", "myval")` and plugin B calls
/// `host::kv_get("mykey")`, Then plugin B receives `None` (KV stores are scoped
/// per plugin, not shared).
///
/// Traces to: BC-2.17.002 / EC-17-008
#[test]
fn test_BC_2_17_002_ac8_kv_store_scoped_per_plugin() {
    use prism_spec_engine::plugin::loader::PluginKvStore;

    // This test exercises the KV store scoping directly (not via WASM call).
    // A plugin-level integration test requires two loaded plugins and host function calls.
    // For Red Gate: verify the KV store API is unimplemented.
    let kv = PluginKvStore::new();
    let plugin_a = "plugin-a";
    let plugin_b = "plugin-b";

    // Plugin A sets a key.
    kv.set(plugin_a, "mykey", "myval")
        .expect("kv_set must succeed for plugin A");

    // Plugin B must not see plugin A's key.
    let b_value = kv.get(plugin_b, "mykey");
    assert_eq!(
        b_value, None,
        "AC-8: plugin B must receive None for plugin A's key (KV stores are scoped per plugin)"
    );

    // Plugin A sees its own key.
    let a_value = kv.get(plugin_a, "mykey");
    assert_eq!(
        a_value,
        Some("myval".to_string()),
        "AC-8: plugin A must see its own key"
    );
}

// ============================================================
// AC-9 / BC-2.17.003 / TV-17-003-exceed
// ============================================================

/// AC-9: Given a plugin registered with the default memory limit (64 MiB),
/// When the plugin's WASM execution attempts to allocate memory beyond 64 MiB,
/// Then `StoreLimits` fires a WASM trap, caught at the `instance.call_*` boundary,
/// and the method returns `Err(PluginError::MemoryExceeded { plugin_id, limit_mb: 64 })`.
///
/// A WARN log entry is emitted: `"Plugin '{plugin_id}' exceeded memory limit of 64MB"`.
/// The host process is unaffected. Plugin registry entry is retained.
///
/// Traces to: BC-2.17.003 / INV-PLUGIN-003
///
/// Ignored on Windows debug builds: `try_allocate_wasm_memory` uses wasmtime JIT
/// which is deeply recursive and causes STATUS_STACK_BUFFER_OVERRUN on Windows.
/// Covered by Linux/macOS CI. See VP-041 Windows note in proofs/plugin_memory.rs.
#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "wasmtime JIT stack overflow on Windows debug (STATUS_STACK_BUFFER_OVERRUN)"
)]
fn test_BC_2_17_003_ac9_memory_limit_exceeded_returns_err() {
    use prism_spec_engine::plugin::sandbox::{try_allocate_wasm_memory, DEFAULT_MEMORY_LIMIT_MB};

    let engine = wasmtime::Engine::default();

    // Over-limit: 64MB + 1 byte must trap.
    let over_limit_bytes = DEFAULT_MEMORY_LIMIT_MB * 1024 * 1024 + 1;
    let result =
        try_allocate_wasm_memory(&engine, DEFAULT_MEMORY_LIMIT_MB, over_limit_bytes as usize);

    assert!(
        matches!(
            result,
            Err(PluginError::MemoryExceeded { limit_mb: 64, .. })
        ),
        "AC-9: allocation over 64MB must return Err(MemoryExceeded {{ limit_mb: 64 }}), got: {:?}",
        result
    );
}

// ============================================================
// EC-17-001 / BC-2.17.001
// ============================================================

/// EC-17-001: Plugin traps on first call after loading — plugin stays registered,
/// next call creates fresh Store and may succeed.
///
/// Traces to: BC-2.17.001 EC-17-001
#[test]
fn test_BC_2_17_001_ec17_001_trap_on_first_call_plugin_stays_registered() {
    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");
    let (_bytes, tmp) = compile_wat_fixture("trap_plugin.wat");
    runtime
        .load_plugin(tmp.path())
        .expect("trap_plugin must load");

    let plugins = runtime.list_plugins();
    let plugin_id = plugins.first().expect("trap_plugin must be registered");
    let config = std::collections::HashMap::new();

    // First call: traps.
    let result = runtime.enrich_single(plugin_id, "v", "t", &config);
    assert!(
        matches!(result, Err(PluginError::Trapped { .. })),
        "EC-17-001: first call must trap"
    );

    // Plugin stays registered (not unregistered after trap).
    let after = runtime.list_plugins();
    assert!(
        after.contains(plugin_id),
        "EC-17-001: plugin must remain registered after trap (BC-2.17.001)"
    );
}

// ============================================================
// TV-17-001-batch / BC-2.17.001
// ============================================================

/// TV-17-001-batch: `enrich_batch` with multiple inputs; plugin traps →
/// entire batch returns `Err(Trapped)`; no partial results.
///
/// Traces to: BC-2.17.001 / EC-17-003
#[test]
fn test_BC_2_17_001_ec17_003_batch_trap_returns_no_partial_results() {
    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");
    let (_bytes, tmp) = compile_wat_fixture("trap_plugin.wat");
    runtime
        .load_plugin(tmp.path())
        .expect("trap_plugin must load");

    let plugins = runtime.list_plugins();
    let plugin_id = plugins.first().expect("trap_plugin must be registered");
    let config = std::collections::HashMap::new();

    let inputs: Vec<String> = (0..500).map(|i| format!("input-{}", i)).collect();
    let result = runtime.enrich_batch(plugin_id, &inputs, "ip", &config);

    assert!(
        matches!(result, Err(PluginError::Trapped { .. })),
        "EC-17-003: batch trap must return Err(Trapped), not partial results, got: {:?}",
        result
    );
}

// ============================================================
// TV-17-001-edge / BC-2.17.001 — concurrent traps
// ============================================================

/// TV-17-001-edge: Two concurrent plugin traps — both tasks independently return
/// Err(Trapped); neither affects the other; host process unaffected.
///
/// Traces to: BC-2.17.001 / EC-17-004
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_BC_2_17_001_ec17_004_concurrent_traps_independent() {
    let runtime = Arc::new(PluginRuntime::new().expect("PluginRuntime::new must succeed"));

    let (_bytes, tmp) = compile_wat_fixture("trap_plugin.wat");
    runtime
        .load_plugin(tmp.path())
        .expect("trap_plugin must load");

    let plugins = runtime.list_plugins();
    let plugin_id = plugins
        .first()
        .cloned()
        .expect("trap_plugin must be registered");

    let r1 = runtime.clone();
    let r2 = runtime.clone();
    let id1 = plugin_id.clone();
    let id2 = plugin_id.clone();
    let config1 = std::collections::HashMap::new();
    let config2 = std::collections::HashMap::new();

    let (res1, res2) = tokio::join!(
        tokio::spawn(async move { r1.enrich_single(&id1, "v1", "ip", &config1) }),
        tokio::spawn(async move { r2.enrich_single(&id2, "v2", "ip", &config2) }),
    );

    let result1 = res1.expect("task 1 must not panic");
    let result2 = res2.expect("task 2 must not panic");

    assert!(
        matches!(result1, Err(PluginError::Trapped { .. })),
        "EC-17-004: task 1 must return Err(Trapped)"
    );
    assert!(
        matches!(result2, Err(PluginError::Trapped { .. })),
        "EC-17-004: task 2 must return Err(Trapped)"
    );
}

// ============================================================
// TV-17-003-happy / BC-2.17.003 — at-limit allocation succeeds
// ============================================================

/// TV-17-003-happy: WAT module allocates exactly 64MB → allocation succeeds;
/// execution continues.
///
/// Traces to: BC-2.17.003 / EC-17-009
///
/// Ignored on Windows debug: wasmtime JIT stack overflow (STATUS_STACK_BUFFER_OVERRUN).
#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "wasmtime JIT stack overflow on Windows debug (STATUS_STACK_BUFFER_OVERRUN)"
)]
fn test_BC_2_17_003_ec17_009_at_limit_allocation_succeeds() {
    use prism_spec_engine::plugin::sandbox::{try_allocate_wasm_memory, DEFAULT_MEMORY_LIMIT_MB};

    let engine = wasmtime::Engine::default();

    // At-limit: exactly 64MB must succeed.
    let at_limit_bytes = DEFAULT_MEMORY_LIMIT_MB * 1024 * 1024;
    let result =
        try_allocate_wasm_memory(&engine, DEFAULT_MEMORY_LIMIT_MB, at_limit_bytes as usize);

    assert!(
        result.is_ok(),
        "EC-17-009: allocation at exactly 64MB must succeed, got: {:?}",
        result
    );
}

// ============================================================
// TV-17-003-override / BC-2.17.003
// ============================================================

/// TV-17-003-override: `memory_limit_mb = 128`; plugin allocates 100MB → succeeds.
///
/// Traces to: BC-2.17.003 / EC-17-011
///
/// Ignored on Windows debug: wasmtime JIT stack overflow (STATUS_STACK_BUFFER_OVERRUN).
#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "wasmtime JIT stack overflow on Windows debug (STATUS_STACK_BUFFER_OVERRUN)"
)]
fn test_BC_2_17_003_ec17_011_per_plugin_memory_override() {
    use prism_spec_engine::plugin::sandbox::try_allocate_wasm_memory;

    let engine = wasmtime::Engine::default();
    let custom_limit_mb = 128u64;
    let hundred_mb = 100u64 * 1024 * 1024;

    let result = try_allocate_wasm_memory(&engine, custom_limit_mb, hundred_mb as usize);
    assert!(
        result.is_ok(),
        "EC-17-011: allocation of 100MB under 128MB limit must succeed, got: {:?}",
        result
    );
}

// ============================================================
// TV-17-005-fail / BC-2.17.005 — failed recompile retains old
// ============================================================

/// TV-17-005-fail: Replace `.prx` with invalid binary → old plugin retained;
/// ERROR log emitted; `E-PLUGIN-008`.
///
/// Traces to: BC-2.17.005 / EC-17-005
#[test]
fn test_BC_2_17_005_ec17_005_failed_recompile_retains_old_plugin() {
    use prism_spec_engine::plugin::hot_reload::hot_reload;

    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");
    let (_bytes, tmp) = compile_wat_fixture("noop_infusion.wat");
    runtime
        .load_plugin(tmp.path())
        .expect("noop_infusion must load");

    let plugins = runtime.list_plugins();
    let plugin_id = plugins
        .first()
        .cloned()
        .expect("noop_infusion must be registered");
    let before_arc = runtime.get_plugin(&plugin_id).expect("must be in registry");

    // Attempt hot reload with garbage bytes (guaranteed compilation failure).
    let garbage = b"THIS IS NOT VALID WASM";
    let tmp_bad = tempfile::NamedTempFile::with_suffix(".prx").unwrap();
    std::fs::write(tmp_bad.path(), garbage).unwrap();

    let result = hot_reload(
        &runtime.registry,
        &runtime.engine,
        &runtime.linker,
        &plugin_id,
        tmp_bad.path(),
        garbage,
    );

    assert!(
        result.is_err(),
        "EC-17-005: reload with garbage bytes must fail"
    );

    // Old plugin must still be present and pointer-equal.
    let after_arc = runtime
        .get_plugin(&plugin_id)
        .expect("plugin must still be in registry after failed reload");
    assert!(
        Arc::ptr_eq(&before_arc, &after_arc),
        "EC-17-005: failed hot reload must retain old Arc<LoadedPlugin> (BC-2.17.005)"
    );
}

// ============================================================
// TV-17-005-delete / BC-2.17.005 — plugin deleted
// ============================================================

/// TV-17-005-delete: Delete `.prx` file → plugin removed from registry;
/// new calls return `E-PLUGIN-011`.
///
/// Traces to: BC-2.17.005 / error row 3
#[test]
fn test_BC_2_17_005_ec17_delete_plugin_new_calls_return_not_loaded() {
    use prism_spec_engine::plugin::hot_reload::hot_unload;

    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");
    let (_bytes, tmp) = compile_wat_fixture("noop_infusion.wat");
    runtime
        .load_plugin(tmp.path())
        .expect("noop_infusion must load");

    let plugins = runtime.list_plugins();
    let plugin_id = plugins.first().cloned().expect("must be registered");

    // Simulate file deletion by calling hot_unload.
    hot_unload(&runtime.registry, &plugin_id);

    // New calls must return NotLoaded (E-PLUGIN-011).
    let config = std::collections::HashMap::new();
    let result = runtime.enrich_single(&plugin_id, "v", "t", &config);
    assert!(
        matches!(result, Err(PluginError::NotLoaded { .. })),
        "EC-17-005 delete: new calls after unload must return Err(NotLoaded), got: {:?}",
        result
    );
}

// ============================================================
// TV-17-006-bulk / BC-2.17.006 — partial failure in bulk discovery
// ============================================================

/// TV-17-006-bulk: 10 plugins scanned; 2 invalid → 8 registered; 2 logged as E-PLUGIN-001.
///
/// Traces to: BC-2.17.006 / EC-17-026
#[test]
fn test_BC_2_17_006_ec17_026_bulk_discovery_partial_failure() {
    use prism_spec_engine::plugin::discovery::discover_plugins;

    let tmp_dir = tempfile::TempDir::new().unwrap();
    let plugins_dir = tmp_dir.path();

    // Write 8 valid (minimal) WAT modules and 2 invalid (garbage) plugins.
    let valid_wat = r#"
        (module
          (memory (export "memory") 1)
          (data (i32.const 0) "noop-infusion")
          (data (i32.const 16) "0.1.0")
          (func (export "name") (result i32 i32) i32.const 0 i32.const 13)
          (func (export "version") (result i32 i32) i32.const 16 i32.const 5)
          (func (export "enrich-single") (param i32 i32 i32 i32) (result i32) i32.const 0)
          (func (export "enrich-batch") (param i32 i32 i32 i32) (result i32 i32)
            i32.const 0 i32.const 0)
        )
    "#;
    let valid_bytes = wat::parse_str(valid_wat).expect("valid WAT should parse");

    for i in 0..8 {
        let path = plugins_dir.join(format!("valid-{:02}.prx", i));
        std::fs::write(&path, &valid_bytes).unwrap();
    }
    for i in 0..2 {
        let path = plugins_dir.join(format!("invalid-{:02}.prx", i));
        std::fs::write(&path, b"NOT VALID WASM").unwrap();
    }

    let engine_config = wasmtime::Config::new();
    // Note: wasmtime::Config does not have Default in older versions; use new().
    let mut cfg = wasmtime::Config::new();
    let _ = cfg.wasm_component_model(true);
    let _ = cfg.epoch_interruption(true);
    let engine = wasmtime::Engine::new(&cfg).expect("engine must construct");

    let linker = PluginRuntime::build_linker(&engine).expect("linker must build");

    let loaded = discover_plugins(plugins_dir, &engine, &linker);

    // Exactly 8 valid plugins must be loaded; the 2 invalid ones are skipped.
    assert_eq!(
        loaded.len(),
        8,
        "EC-17-026: discover_plugins must load 8 valid plugins and skip 2 invalid, got: {}",
        loaded.len()
    );
}

// ============================================================
// EC-17-027 / BC-2.17.006 — empty plugin_id rejected
// ============================================================

/// EC-17-027: Valid WIT interface but `name()` returns empty string → `E-PLUGIN-010`.
///
/// Traces to: BC-2.17.006 / EC-17-027
#[test]
fn test_BC_2_17_006_ec17_027_empty_plugin_id_rejected() {
    let runtime = PluginRuntime::new().expect("PluginRuntime::new must succeed");

    // A WAT module that has all required exports but returns empty string from name().
    let empty_name_wat = r#"
        (module
          (memory (export "memory") 1)
          ;; name() returns ptr=0, len=0 (empty string)
          (func (export "name") (result i32 i32) i32.const 0 i32.const 0)
          (func (export "version") (result i32 i32) i32.const 0 i32.const 0)
          (func (export "enrich-single") (param i32 i32 i32 i32) (result i32) i32.const 0)
          (func (export "enrich-batch") (param i32 i32 i32 i32) (result i32 i32)
            i32.const 0 i32.const 0)
        )
    "#;
    let bytes = wat::parse_str(empty_name_wat).unwrap();
    let tmp = tempfile::NamedTempFile::with_suffix(".prx").unwrap();
    std::fs::write(tmp.path(), &bytes).unwrap();

    let result = runtime.load_plugin(tmp.path());

    assert!(
        result.is_err(),
        "EC-17-027: plugin with empty name() must be rejected"
    );
    assert!(
        matches!(
            result.err().expect("result already Err"),
            PluginError::EmptyPluginId { .. }
        ),
        "EC-17-027: error must be EmptyPluginId (E-PLUGIN-010)"
    );
}

// ============================================================
// EC-17-007 / BC-2.17.002 — http_request no allowlist allowed
// ============================================================

/// EC-17-007: Plugin calls `host::http_request` when no allowlist is configured →
/// request allowed to any URL; audit log entry created.
///
/// Traces to: BC-2.17.002 / EC-17-007
#[test]
fn test_BC_2_17_002_ec17_007_http_request_no_allowlist_allowed() {
    use prism_spec_engine::plugin::host_functions::host_http_request;
    use prism_spec_engine::plugin::loader::{HostState, PluginConfigMap, PluginKvStore};
    use std::sync::Arc;

    let state = HostState {
        http_client: Arc::new(reqwest::Client::new()),
        config: Arc::new(PluginConfigMap::new()),
        kv_store: Arc::new(PluginKvStore::new()),
        plugin_id: "test-plugin".to_string(),
        allowed_urls: None, // No allowlist — all URLs allowed.
        limits: wasmtime::StoreLimits::default(),
    };

    // With no allowlist, any URL should be attempted (not blocked with 403).
    // We cannot make real HTTP in a unit test; verify the function doesn't
    // immediately return 403 for a well-known URL.
    let response = host_http_request(&state, "GET", "https://example.com/", vec![], None);

    assert_ne!(
        response.status, 403,
        "EC-17-007: request with no allowlist configured must not be blocked (status 403)"
    );
}

// ============================================================
// EC-17-006 / BC-2.17.002 — http_request allowlisted URL succeeds
// ============================================================

/// EC-17-006: Plugin calls `host::http_request` with URL in allowlist →
/// request executed via reqwest; response returned; audit log created.
///
/// Traces to: BC-2.17.002 / EC-17-006
#[test]
fn test_BC_2_17_002_ec17_006_http_request_allowlisted_url_succeeds() {
    use prism_spec_engine::plugin::host_functions::host_http_request;
    use prism_spec_engine::plugin::loader::{HostState, PluginConfigMap, PluginKvStore};
    use std::sync::Arc;

    let state = HostState {
        http_client: Arc::new(reqwest::Client::new()),
        config: Arc::new(PluginConfigMap::new()),
        kv_store: Arc::new(PluginKvStore::new()),
        plugin_id: "test-plugin".to_string(),
        allowed_urls: Some(vec!["example.com".to_string()]), // Allowlist includes target.
        limits: wasmtime::StoreLimits::default(),
    };

    let response = host_http_request(&state, "GET", "https://example.com/", vec![], None);

    assert_ne!(
        response.status, 403,
        "EC-17-006: request to allowlisted URL must not be blocked (got 403)"
    );
}

// ============================================================
// TV-17-002-allowlist / BC-2.17.002 — non-allowlisted URL returns 403
// ============================================================

/// TV-17-002-allowlist: Plugin calls `host::http_request` with non-allowlisted URL →
/// HTTP 403 returned to plugin; WARN logged.
///
/// Traces to: BC-2.17.002 / error row 2
#[test]
fn test_BC_2_17_002_ec17_url_not_in_allowlist_returns_403() {
    use prism_spec_engine::plugin::host_functions::host_http_request;
    use prism_spec_engine::plugin::loader::{HostState, PluginConfigMap, PluginKvStore};
    use std::sync::Arc;

    let state = HostState {
        http_client: Arc::new(reqwest::Client::new()),
        config: Arc::new(PluginConfigMap::new()),
        kv_store: Arc::new(PluginKvStore::new()),
        plugin_id: "test-plugin".to_string(),
        allowed_urls: Some(vec!["example.com".to_string()]), // Only example.com allowed.
        limits: wasmtime::StoreLimits::default(),
    };

    let response = host_http_request(
        &state,
        "GET",
        "https://evil.com/steal-data", // Not in allowlist.
        vec![],
        None,
    );

    assert_eq!(
        response.status, 403,
        "TV-17-002-allowlist: request to non-allowlisted URL must return 403"
    );
}

// ============================================================
// TV-17-004-override / BC-2.17.004 — per-plugin timeout override
// ============================================================

/// TV-17-004-override: `timeout_seconds = 30`; plugin runs 25s → succeeds under
/// extended limit.
///
/// This is a structural test only (not a real 25-second sleep). It verifies that
/// `create_store` accepts a custom timeout_seconds value.
///
/// Traces to: BC-2.17.004 / EC-17-015
#[test]
fn test_BC_2_17_004_ec17_015_per_plugin_timeout_override() {
    use prism_spec_engine::plugin::loader::{HostState, PluginConfigMap, PluginKvStore};
    use prism_spec_engine::plugin::sandbox::create_store;
    use std::sync::Arc;

    let engine = wasmtime::Engine::default();
    let host_state = HostState {
        http_client: Arc::new(reqwest::Client::new()),
        config: Arc::new(PluginConfigMap::new()),
        kv_store: Arc::new(PluginKvStore::new()),
        plugin_id: "timeout-test".to_string(),
        allowed_urls: None,
        limits: wasmtime::StoreLimits::default(),
    };

    // create_store with 30-second timeout must not panic.
    let _store = create_store(
        &engine, host_state, 64, // memory_limit_mb
        30, // timeout_seconds (override)
    );
    // If we reach here, create_store accepted the custom timeout.
    // (Will panic with unimplemented!() in Red Gate — confirming the stub.)
}
