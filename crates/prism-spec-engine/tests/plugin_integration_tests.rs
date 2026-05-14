//! S-PLUGIN-PREREQ-D Red Gate tests — prism-spec-engine plugin integration.
//!
//! All 18 tests stub with `todo!()` and MUST FAIL until the implementer wires
//! the full plugin runtime in `crates/prism-spec-engine/src/plugin/` (BC-5.38.001 Red Gate).
//!
//! Traces to: S-PLUGIN-PREREQ-D (v1.32)
//! BCs: BC-2.16.002, BC-2.17.001, BC-2.17.002, BC-2.17.003, BC-2.17.004,
//!      BC-2.17.006, BC-2.17.007
//! TDs: TD-S-PLUGIN-PREREQ-B-002, TD-S-PLUGIN-PREREQ-B-011
//!
//! # Test → AC / BC / TD mapping
//!
//! | Test | AC / TD | BC / anchor |
//! |------|---------|-------------|
//! | test_BC_2_17_001_plugin_panic_isolation | AC-10 | BC-2.17.001 |
//! | test_BC_2_17_002_wasi_not_linked_trap_on_fs_call | AC-11 | BC-2.17.002 |
//! | test_BC_2_17_002_allowlist_enforcement_blocks_non_allowlisted_url | AC-7 | BC-2.17.002; VP-PLUGIN-007 |
//! | test_BC_2_17_002_allowlist_enforcement_allows_listed_url | AC-7 | BC-2.17.002; VP-PLUGIN-007 |
//! | test_BC_2_17_003_memory_limit_enforced_default_64mb | AC-12 | BC-2.17.003 |
//! | test_BC_2_17_004_cpu_timeout_enforced_infinite_loop | AC-13 | BC-2.17.004 |
//! | test_hot_reload_atomic_swap_success | AC-14 | BC-2.17.005 (programmatic hot_reload API) |
//! | test_hot_reload_failed_recompile_retains_old | AC-14 | BC-2.17.005 |
//! | test_BC_2_17_006_wit_validation_rejects_missing_export | AC-6 | BC-2.17.006 |
//! | test_BC_2_17_006_duplicate_plugin_id_first_wins | AC-6 | BC-2.17.006; EC-D-008 |
//! | test_BC_2_17_007_manifest_format_version_exceeded_rejected | AC-5 | BC-2.17.007; E-PLUGIN-014 |
//! | test_BC_2_17_007_manifest_missing_allowed_urls_rejected | AC-5 | BC-2.17.007; E-PLUGIN-013 |
//! | test_BC_2_17_007_manifest_name_empty_rejected | AC-5 | BC-2.17.007; E-PLUGIN-015; EC-D-012 |
//! | test_BC_2_17_007_manifest_version_malformed_rejected | AC-5 | BC-2.17.007; E-PLUGIN-016; EC-D-013 |
//! | test_BC_2_17_002_linker_imports_match_host_functions | AC-8 | BC-2.17.002; ADR-023 §C4 |
//! | test_TD_S_PLUGIN_PREREQ_B_011_execute_step_eager_token_calls_auth_once | Task 8 / TD-S-PLUGIN-PREREQ-B-011 | BC-2.16.002 |
//! | test_BC_2_16_002_pipeline_max_requests_exceeded | AC-16 | BC-2.16.002; TD-S-PLUGIN-PREREQ-B-004 |
//! | test_TD_S_PLUGIN_PREREQ_B_002_authtoken_zeroize_on_drop | AC-15 | TD-S-PLUGIN-PREREQ-B-002; AD-017 |

#![allow(dead_code, unused_imports)]

/// AC-10 (S-PLUGIN-PREREQ-D) — A panicking guest WASM module is isolated from the host process.
///
/// BC-2.17.001: when a WASM plugin guest panics (trap), `PluginRuntime` catches the Wasmtime
/// trap and returns `PluginError::Trapped`; the host process does NOT crash. A fresh `Store`
/// is created for the next invocation (no cross-call WASM state leakage).
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires panic isolation in
/// `PluginRuntime` (fresh-Store-per-call + Trap → PluginError::Trapped).
#[test]
fn test_BC_2_17_001_plugin_panic_isolation() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-10)")
}

/// AC-11 (S-PLUGIN-PREREQ-D) — WASI filesystem imports are NOT linked; any WASM call that
/// attempts a filesystem operation results in a Wasmtime link trap (`Error::Trap`).
///
/// BC-2.17.002: the Wasmtime `Linker` must not include `wasmtime_wasi` filesystem bindings.
/// A plugin that attempts `wasi:filesystem/...` calls must receive a link error at instantiation
/// time (missing import), causing a trap that surfaces as `PluginError::Trapped`.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer configures the Linker
/// without WASI filesystem in `PluginRuntime`.
#[test]
fn test_BC_2_17_002_wasi_not_linked_trap_on_fs_call() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-11)")
}

/// AC-7 (S-PLUGIN-PREREQ-D) — `host_http_request` blocks requests to URLs whose host is NOT
/// in the plugin's `allowed_urls` manifest list; returns HTTP 403 to the plugin.
///
/// BC-2.17.002 + VP-PLUGIN-007: allowlist enforcement uses host-only `==` comparison (no
/// substring matching). A plugin with `allowed_urls: ["allowed-sensor.internal"]` that attempts
/// to fetch `https://evil.example.com/steal` must receive a 403 response.
/// BC-2.16.002 catalog row `plugin_http_request_blocked` (WARN) must be emitted.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer closes the None-short-circuit
/// in `host_http_request` and wires allowlist enforcement.
#[test]
fn test_BC_2_17_002_allowlist_enforcement_blocks_non_allowlisted_url() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-7, VP-PLUGIN-007)")
}

/// AC-7 (S-PLUGIN-PREREQ-D) — `host_http_request` permits requests to URLs whose host IS
/// in the plugin's `allowed_urls` manifest list; the request proceeds normally.
///
/// BC-2.17.002 + VP-PLUGIN-007: a plugin with `allowed_urls: ["allowed-sensor.internal"]`
/// that fetches `https://allowed-sensor.internal/api/data` must NOT be blocked by the
/// allowlist check; the request is forwarded to the real (or mocked) HTTP client.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires allowlist allow-path.
#[test]
fn test_BC_2_17_002_allowlist_enforcement_allows_listed_url() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-7, VP-PLUGIN-007)")
}

/// AC-12 (S-PLUGIN-PREREQ-D) — Default per-plugin memory limit of 64 MiB is enforced.
///
/// BC-2.17.003: a WASM plugin that attempts to allocate beyond the 64 MiB StoreLimits cap
/// must receive a Wasmtime OOM trap, surfaced as `PluginError::Trapped`. The default limit
/// applies unless overridden by a per-plugin `memory_limit_mb` manifest field.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer configures StoreLimits
/// in the Wasmtime Store per BC-2.17.003.
#[test]
fn test_BC_2_17_003_memory_limit_enforced_default_64mb() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-12)")
}

/// AC-13 (S-PLUGIN-PREREQ-D) — CPU timeout via Wasmtime epoch interruption terminates
/// an infinite-loop plugin within the configured deadline.
///
/// BC-2.17.004: `PluginRuntime::new` starts a single epoch ticker thread (started once,
/// preventing resource leaks across N plugins). A plugin that enters an infinite loop must
/// be interrupted by the epoch mechanism and return `PluginError::Timeout` (or equivalent
/// trap) within the configured deadline. The ticker must not be re-started on subsequent calls.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires epoch-based CPU
/// timeout in `PluginRuntime::new` per BC-2.17.004.
#[test]
fn test_BC_2_17_004_cpu_timeout_enforced_infinite_loop() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-13)")
}

/// AC-14 (S-PLUGIN-PREREQ-D) — Programmatic `hot_reload()` API atomically swaps the
/// registry to the newly compiled plugin; callers after the swap see the new version.
///
/// BC-2.17.005 (programmatic hot_reload API only — boot notify watcher is S-1.12-FOLLOWUP scope):
/// after `hot_reload(path)` completes successfully, the arc-swap registry contains the new
/// plugin binary. Callers that acquired a registry snapshot before the reload see the old
/// version; callers after see the new version (atomicity guarantee).
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires `hot_reload()` API.
#[test]
fn test_hot_reload_atomic_swap_success() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-14)")
}

/// AC-14 (S-PLUGIN-PREREQ-D) — When `hot_reload()` fails (e.g., corrupt `.prx` bytes),
/// the existing registry entry is retained; the old plugin remains callable.
///
/// BC-2.17.005: a failed recompile must NOT replace the existing registry entry. The arc-swap
/// must not be updated on error. The caller receives `Err(PluginError::LoadFailed)` (or similar)
/// and the old plugin is still registered and callable.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires failed-reload retention.
#[test]
fn test_hot_reload_failed_recompile_retains_old() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-14)")
}

/// AC-6 (S-PLUGIN-PREREQ-D) — A plugin whose WASM component is missing one or more required
/// WIT exports is rejected with `E-PLUGIN-001`; it is NOT added to the registry.
///
/// BC-2.17.006: `validate_wit_interface` checks for all required WIT exports. A plugin binary
/// lacking a required export must cause `load_plugin` to return `Err(E-PLUGIN-001)` and the
/// plugin must NOT appear in the registry after `load_all_plugins` completes.
/// BC-2.16.002 catalog row `plugin_load_failed_wit_invalid` must be emitted.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires `validate_wit_interface`.
#[test]
fn test_BC_2_17_006_wit_validation_rejects_missing_export() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-6)")
}

/// AC-6 (S-PLUGIN-PREREQ-D) — When two `.prx` files declare the same `plugin_id`, the first
/// registered plugin is retained and the second is silently skipped with a WARN log.
///
/// BC-2.17.006 invariant + EC-D-008: `first-registered wins`. The second plugin must not
/// replace the first in the arc-swap registry. A `WARN "Duplicate plugin_id '...': first-registered
/// plugin retained"` log must be emitted for the discarded second plugin.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires duplicate-ID check.
#[test]
fn test_BC_2_17_006_duplicate_plugin_id_first_wins() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-6)")
}

/// AC-5 (S-PLUGIN-PREREQ-D) — Plugin manifest `format_version` exceeding `CURRENT_SUPPORTED_VERSION`
/// causes the plugin to be rejected with `E-PLUGIN-014`; n-1 survivor rule applies.
///
/// BC-2.17.007: manifest validation must check `format_version <= CURRENT_SUPPORTED_VERSION` (= 1).
/// A manifest with `format_version = 2` must be rejected. BC-2.16.002 catalog row
/// `plugin_load_failed_format_version_exceeded` (ERROR, fields: `plugin_path`, `format_version`,
/// `max_supported`) must be emitted. Remaining plugins continue loading.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires format_version check
/// in manifest validation.
#[test]
fn test_BC_2_17_007_manifest_format_version_exceeded_rejected() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-5, E-PLUGIN-014)")
}

/// AC-5 (S-PLUGIN-PREREQ-D) — Plugin manifest that omits the `allowed_urls` field (or sets
/// it to `null`) is rejected with `E-PLUGIN-013`; n-1 survivor rule applies.
///
/// BC-2.17.007 + VP-PLUGIN-007: `allowed_urls` is a REQUIRED field. Absent or null value must
/// be rejected. An explicitly empty list `[]` is accepted (default-deny semantics). BC-2.16.002
/// catalog row `plugin_load_failed_manifest_no_allowed_urls` (ERROR, fields: `plugin_path`,
/// `error: E-PLUGIN-013`) must be emitted.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires allowed_urls presence check.
#[test]
fn test_BC_2_17_007_manifest_missing_allowed_urls_rejected() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-5, E-PLUGIN-013)")
}

/// AC-5 (S-PLUGIN-PREREQ-D) — Plugin manifest whose `name` field is absent or an empty string
/// is rejected with `E-PLUGIN-015`; n-1 survivor rule applies.
///
/// BC-2.17.007 + EC-D-012: `name` must be a non-empty string. Missing or `""` value must be
/// rejected. BC-2.16.002 catalog row `plugin_load_failed_manifest_name_missing` (ERROR, fields:
/// `plugin_path`, `error: E-PLUGIN-015`) must be emitted.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires name validation.
#[test]
fn test_BC_2_17_007_manifest_name_empty_rejected() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-5, E-PLUGIN-015)")
}

/// AC-5 (S-PLUGIN-PREREQ-D) — Plugin manifest whose `version` field is not valid semver is
/// rejected with `E-PLUGIN-016`; n-1 survivor rule applies.
///
/// BC-2.17.007 + EC-D-013: `version` must parse as valid semver (e.g., `"1.0.0"`). A value
/// like `"not-semver"` or `"1.x"` must be rejected. BC-2.16.002 catalog row
/// `plugin_load_failed_manifest_version_malformed` (ERROR, fields: `plugin_path`, `version_value`,
/// `error: E-PLUGIN-016`) must be emitted.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires semver validation.
#[test]
fn test_BC_2_17_007_manifest_version_malformed_rejected() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-5, E-PLUGIN-016)")
}

/// AC-8 (S-PLUGIN-PREREQ-D) — The Wasmtime `Linker` import list at instantiation time
/// exactly matches the set of registered host functions (no extras, no gaps).
///
/// BC-2.17.002 + ADR-023 §C4: a compile-time / instantiation-time assertion must verify
/// that the host functions registered in the `Linker` are exactly the set declared in the
/// WIT interface — no undeclared imports are linked, no declared imports are missing.
/// This prevents silent API surface drift between the WIT contract and the host implementation.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires the `#[cfg(test)]`
/// linker-import assertion per ADR-023 §C4.
#[test]
fn test_BC_2_17_002_linker_imports_match_host_functions() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-8)")
}

/// Task 8 / TD-S-PLUGIN-PREREQ-B-011 (S-PLUGIN-PREREQ-D) — `execute_step` acquires an
/// auth token exactly once per invocation (eager-token semantics), regardless of how many
/// HTTP requests the step makes.
///
/// BC-2.16.002 pipeline executor contract: a `MockAuthProvider` that counts `authenticate()`
/// calls must report `calls() == 1` after a single `execute_step` invocation, even if the
/// step makes multiple sub-requests. This closes TD-S-PLUGIN-PREREQ-B-011 by confirming the
/// eager-token wiring in the PREREQ-D integration context.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires execute_step eager-token
/// semantics and this test uses a real MockAuthProvider.
#[test]
fn test_TD_S_PLUGIN_PREREQ_B_011_execute_step_eager_token_calls_auth_once() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D Task 8 / TD-S-PLUGIN-PREREQ-B-011)")
}

/// AC-16 (S-PLUGIN-PREREQ-D) — The `PipelineExecutor` cumulative HTTP request counter is
/// capped at `MAX_REQUESTS_PER_PIPELINE = 10_000`; reaching the cap aborts the pipeline.
///
/// BC-2.16.002 + TD-S-PLUGIN-PREREQ-B-004: once the cumulative request count across all steps
/// reaches 10 000, the pipeline must abort and emit BC-2.16.002 catalog row
/// `pipeline_max_requests_exceeded` (ERROR, fields: `plugin_id`, `total_requests`,
/// `max: MAX_REQUESTS_PER_PIPELINE`). The 10 001st request must never be sent.
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer wires the
/// MAX_REQUESTS_PER_PIPELINE counter in pipeline.rs executor loop.
#[test]
fn test_BC_2_16_002_pipeline_max_requests_exceeded() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-16)")
}

/// AC-15 (S-PLUGIN-PREREQ-D) — `AuthToken` zeroes its credential bytes on drop.
///
/// TD-S-PLUGIN-PREREQ-B-002 + AD-017: `AuthToken` must use `zeroize::Zeroizing<String>`
/// (or an explicit `Drop` impl calling `zeroize()`) so that credential bytes are overwritten
/// in memory when the token is dropped. This prevents credential retention in freed memory.
/// The test verifies the `Drop` contract, typically by inspecting that the backing memory is
/// zeroed after the value is dropped (or by confirming the `Zeroize` impl exists at compile time).
///
/// Red Gate per BC-5.38.001 — fails with todo!() until implementer adds zeroize dep and
/// `Zeroizing<String>` wrapper to `AuthToken` in auth_provider.rs.
#[test]
fn test_TD_S_PLUGIN_PREREQ_B_002_authtoken_zeroize_on_drop() {
    todo!("not yet implemented (S-PLUGIN-PREREQ-D AC-15)")
}
