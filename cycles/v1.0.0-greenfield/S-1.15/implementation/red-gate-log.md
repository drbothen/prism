# Red Gate Log — S-1.15: WASM Plugin Runtime

**Cycle:** v1.0.0-greenfield
**Story:** S-1.15 — prism-spec-engine: WASM Plugin Runtime
**Status:** RED GATE VERIFIED
**Date:** 2026-04-22
**Agent:** test-writer

---

## Summary

All stubs written. All tests fail with `not yet implemented` panics.
Zero tests pass. Implementation may begin.

---

## Test Results

### Integration Tests (`tests/plugin_tests.rs`)

```
test result: FAILED. 0 passed; 23 failed; 0 ignored
```

| Test | BC | AC | Status |
|------|----|----|--------|
| test_BC_2_17_006_ac1_load_valid_infusion_plugin | BC-2.17.006 | AC-1 | FAIL |
| test_BC_2_17_001_ac2_plugin_trap_returns_err_trapped | BC-2.17.001 | AC-2 | FAIL |
| test_BC_2_17_004_ac3_infinite_loop_returns_err_timeout | BC-2.17.004 | AC-3 | FAIL |
| test_BC_2_17_002_ac4_wasi_filesystem_not_accessible | BC-2.17.002 | AC-4 | FAIL |
| test_BC_2_17_002_ac5_http_request_proxied_via_host | BC-2.17.002 | AC-5 | FAIL |
| test_BC_2_17_005_ac6_hot_reload_atomic_swap | BC-2.17.005 | AC-6 | FAIL |
| test_BC_2_17_006_ac7_invalid_wit_returns_e_plugin_001 | BC-2.17.006 | AC-7 | FAIL |
| test_BC_2_17_006_ac7_invariant_plugin_not_registered_after_invalid_wit | BC-2.17.006 | AC-7 | FAIL |
| test_BC_2_17_002_ac8_kv_store_scoped_per_plugin | BC-2.17.002 | AC-8 | FAIL |
| test_BC_2_17_003_ac9_memory_limit_exceeded_returns_err | BC-2.17.003 | AC-9 | FAIL |
| test_BC_2_17_001_ec17_001_trap_on_first_call_plugin_stays_registered | BC-2.17.001 | EC-17-001 | FAIL |
| test_BC_2_17_001_ec17_003_batch_trap_returns_no_partial_results | BC-2.17.001 | EC-17-003 | FAIL |
| test_BC_2_17_001_ec17_004_concurrent_traps_independent | BC-2.17.001 | EC-17-004 | FAIL |
| test_BC_2_17_003_ec17_009_at_limit_allocation_succeeds | BC-2.17.003 | EC-17-009 | FAIL |
| test_BC_2_17_003_ec17_011_per_plugin_memory_override | BC-2.17.003 | EC-17-011 | FAIL |
| test_BC_2_17_005_ec17_005_failed_recompile_retains_old_plugin | BC-2.17.005 | EC-17-005 | FAIL |
| test_BC_2_17_005_ec17_delete_plugin_new_calls_return_not_loaded | BC-2.17.005 | TV-17-005-delete | FAIL |
| test_BC_2_17_006_ec17_026_bulk_discovery_partial_failure | BC-2.17.006 | EC-17-026 | FAIL |
| test_BC_2_17_006_ec17_027_empty_plugin_id_rejected | BC-2.17.006 | EC-17-027 | FAIL |
| test_BC_2_17_002_ec17_007_http_request_no_allowlist_allowed | BC-2.17.002 | EC-17-007 | FAIL |
| test_BC_2_17_002_ec17_006_http_request_allowlisted_url_succeeds | BC-2.17.002 | EC-17-006 | FAIL |
| test_BC_2_17_002_ec17_url_not_in_allowlist_returns_403 | BC-2.17.002 | TV-17-002-allowlist | FAIL |
| test_BC_2_17_004_ec17_015_per_plugin_timeout_override | BC-2.17.004 | EC-17-015 | FAIL |

### VP Proof Tests (lib tests in `src/proofs/`)

```
test result: FAILED. 0 passed; 22 failed; 1 ignored
```

| Test | VP | Status |
|------|----|--------|
| proofs::plugin_linker::tests::test_BC_2_17_002_vp040_wasi_importing_component_rejected | VP-040 | FAIL |
| proofs::plugin_linker::tests::test_BC_2_17_002_vp040_linker_has_no_wasi_namespace | VP-040 | IGNORED (Kani feasibility pending) |
| proofs::plugin_memory::tests::test_BC_2_17_003_vp041_memory_limit_boundary_exact | VP-041 | FAIL (proptest) |
| proofs::plugin_hot_reload::tests::test_BC_2_17_005_vp042_failed_reload_retains_old_arc | VP-042 | FAIL (proptest) |
| proofs::plugin_hot_reload::tests::test_BC_2_17_005_vp042_empty_bytes_reload_retains_old_plugin | VP-042 | FAIL |
| proofs::plugin_wit_validation::tests (7 tests) | VP-043 | FAIL |

**Total: 45 tests failing, 0 passing, 1 ignored.**

---

## Notes for Implementer

1. **WAT fixtures are core WAT modules**, not Component Model binaries. Use `wasm-tools component new` (or `build.rs` with the `wat` crate) to wrap them as Component Model components before `Component::from_binary` can accept them.
2. **VP-040 Kani path is `#[ignore]`d** — the proptest fallback is the active test.
3. **AC-5 test explicitly panics** — structural test only; full HTTP proxy integration requires a mock HTTP server.
4. All `PluginRuntime` struct fields are `pub` (not `pub(crate)`) to allow external integration tests to access `engine`, `linker`, `registry` for VP-042 hot-reload testing.
5. Cargo.toml adds `anyhow = "1"` as an explicit dep (wasmtime already brings it transitively, but it is explicitly needed in `sandbox.rs`).
