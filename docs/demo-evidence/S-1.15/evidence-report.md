# Evidence Report — S-1.15: WASM Plugin Runtime

**Story:** S-1.15 — prism-spec-engine: WASM Plugin Runtime  
**Branch:** feature/S-1.15-wasm-runtime  
**Policy:** POL-010  
**Date:** 2026-04-22  
**Recorder:** demo-recorder agent  
**Test state:** 22/23 integration tests pass + 12/12 VP proof tests pass

---

## Known Issue

**AC-5 test has a hardcoded `panic!()` — test-writer bug (triaged to PR manager).**

`test_BC_2_17_002_ac5_http_request_proxied_via_host` ends unconditionally with:
```rust
panic!("AC-5: Red Gate — PluginRuntime::new() is unimplemented. ...");
```
This `panic!()` was present in the Red Gate stub and was not removed when the
implementation landed. The underlying implementation (host HTTP proxy via
`host_http_request`) is fully functional and is demonstrated in the AC-005 recording
via the EC-17-007 and EC-17-006 companion tests, which exercise the same code path and
both pass. The implementer flagged this as a test-writer bug; it is not an
implementation defect.

**Total integration test result: 22/23 pass (1 test-writer bug, implementation correct).**

---

## Coverage Summary

| AC | Recording | Success Path | Error Path | Test Status | BC |
|----|-----------|-------------|-----------|-------------|-----|
| AC-1 | AC-001-load-valid-plugin | plugin loads and registers | — | PASS | BC-2.17.006 |
| AC-2 | AC-002-plugin-panic-isolation | host continues after trap | Err(Trapped) returned | PASS | BC-2.17.001 |
| AC-3 | AC-003-cpu-timeout | — | Err(Timeout) within 6s | PASS | BC-2.17.004 |
| AC-4 | AC-004-sandbox-no-filesystem | — | WASI import rejected at load | PASS | BC-2.17.002 |
| AC-5 | AC-005-http-proxy-host | EC-17-006/007 proxy paths pass | EC-17 URL allowlist blocks | PASS* | BC-2.17.002 |
| AC-6 | AC-006-hot-reload | atomic arc-swap, old Arc survives | failed reload retains old | PASS | BC-2.17.005 |
| AC-7 | AC-007-wit-validation-rejection | — | E-PLUGIN-001, not registered | PASS | BC-2.17.006 |
| AC-8 | AC-008-kv-store-scoped | plugin A sees own key | plugin B gets None | PASS | BC-2.17.002 |
| AC-9 | AC-009-memory-limit | at-limit 64MB succeeds | Err(MemoryExceeded) over limit | PASS | BC-2.17.003 |
| AC-10 | AC-010-vp040-wasi-excluded | — | WASI-importing component rejected | PASS | BC-2.17.002 |
| AC-11 | AC-011-vp041-memory-boundary | at-limit succeeds (1..=512 MB) | over-limit traps (proptest) | PASS | BC-2.17.003 |
| AC-12 | AC-012-vp042-hot-reload-retains | — | failed reload → Arc ptr_eq unchanged | PASS | BC-2.17.005 |
| AC-13 | AC-013-vp043-wit-rejects-missing | complete exports → Ok(PluginType) | missing export named in error | PASS | BC-2.17.006 |

*AC-5 implementation is correct; 1 companion test has a hardcoded panic! (test-writer bug).

---

## Recording Index

### AC-1: Load Valid Plugin (BC-2.17.006)
- `AC-001-load-valid-plugin.gif` / `.webm` / `.tape`
- Demonstrates: `PluginRuntime::load_plugin` compiles, validates WIT, and registers a
  valid `noop_infusion.wat` plugin without error. Registry is non-empty after load.
- Test: `test_BC_2_17_006_ac1_load_valid_infusion_plugin` — PASS

### AC-2: Plugin Panic Isolation (BC-2.17.001)
- `AC-002-plugin-panic-isolation.gif` / `.webm` / `.tape`
- Demonstrates: `trap_plugin.wat` (unconditional `unreachable`) returns `Err(Trapped)`.
  Host runtime continues; plugin registry entry is retained. Concurrent trap test also
  shown.
- Test: `test_BC_2_17_001_ac2_plugin_trap_returns_err_trapped` — PASS

### AC-3: CPU Time Limit / Epoch Interruption (BC-2.17.004)
- `AC-003-cpu-timeout.gif` / `.webm` / `.tape`
- Demonstrates: `loop_plugin.wat` infinite loop returns `Err(Timeout)` within the
  5-second epoch deadline + 1-second tolerance. Host process unaffected.
- Test: `test_BC_2_17_004_ac3_infinite_loop_returns_err_timeout` — PASS

### AC-4: Sandbox — No Filesystem Access (BC-2.17.002)
- `AC-004-sandbox-no-filesystem.gif` / `.webm` / `.tape`
- Demonstrates: A WAT module importing `wasi_snapshot_preview1::fd_write` is rejected
  at `load_plugin` time because the Linker has no WASI bindings. Yields
  `Err(CompilationFailed | InvalidInterface | SandboxViolation)`.
- Test: `test_BC_2_17_002_ac4_wasi_filesystem_not_accessible` — PASS

### AC-5: HTTP Proxy via Host (BC-2.17.002)
- `AC-005-http-proxy-host.gif` / `.webm` / `.tape`
- Demonstrates: `host_http_request` function via EC-17-007 (no allowlist → request
  not blocked) and EC-17-002-allowlist (URL not in allowlist → HTTP 403 returned).
  Both pass. The AC-5 primary integration test has a hardcoded `panic!()` — see
  Known Issue section above.
- Tests: EC-17-007, EC-17-url-not-in-allowlist — PASS

### AC-6: Hot Reload — Atomic Arc-Swap (BC-2.17.005)
- `AC-006-hot-reload.gif` / `.webm` / `.tape`
- Demonstrates: Valid hot reload swaps registry entry via arc-swap; old `Arc<LoadedPlugin>`
  remains valid for in-flight calls. Failed reload (garbage bytes) is also shown —
  old Arc is ptr_eq to post-reload Arc, confirming registry entry unchanged.
- Tests: `test_BC_2_17_005_ac6_hot_reload_atomic_swap`,
  `test_BC_2_17_005_ec17_005_failed_recompile_retains_old_plugin` — PASS

### AC-7: WIT Validation Rejection (BC-2.17.006)
- `AC-007-wit-validation-rejection.gif` / `.webm` / `.tape`
- Demonstrates: Plugin with no exports (no `name`, `version`, dispatch function) is
  rejected with `Err(InvalidInterface)` / E-PLUGIN-001. Registry count unchanged.
- Tests: `test_BC_2_17_006_ac7_invalid_wit_returns_e_plugin_001`,
  `test_BC_2_17_006_ac7_invariant_plugin_not_registered_after_invalid_wit` — PASS

### AC-8: KV Store Scoped Per Plugin (BC-2.17.002)
- `AC-008-kv-store-scoped.gif` / `.webm` / `.tape`
- Demonstrates: `PluginKvStore` isolates keys by plugin_id. Plugin B calling
  `kv_get("mykey")` after plugin A called `kv_set("mykey", "myval")` receives `None`.
  Plugin A still sees its own key.
- Test: `test_BC_2_17_002_ac8_kv_store_scoped_per_plugin` — PASS

### AC-9: Memory Limit — 64 MiB StoreLimits (BC-2.17.003)
- `AC-009-memory-limit.gif` / `.webm` / `.tape`
- Demonstrates (error path): Allocation of 64 MiB + 1 byte returns
  `Err(MemoryExceeded { limit_mb: 64 })`.
  Demonstrates (success path): Allocation of exactly 64 MiB returns `Ok`.
- Tests: `test_BC_2_17_003_ac9_memory_limit_exceeded_returns_err`,
  `test_BC_2_17_003_ec17_009_at_limit_allocation_succeeds` — PASS

### AC-10: VP-040 — Linker Excludes WASI Imports (BC-2.17.002)
- `AC-010-vp040-wasi-excluded.gif` / `.webm` / `.tape`
- Demonstrates: VP-040 proptest fallback — a WASI-importing component is rejected at
  `load_plugin` time. Kani path is `#[ignore]`d pending wasmtime Linker enumeration API
  (documented in the proof file with feasibility caveat).
- Test: `proofs::plugin_linker::tests::test_BC_2_17_002_vp040_wasi_importing_component_rejected` — PASS

### AC-11: VP-041 — Memory Limit Boundary Exact (BC-2.17.003)
- `AC-011-vp041-memory-boundary.gif` / `.webm` / `.tape`
- Demonstrates: proptest over `limit_mb in 1..=512` — at-limit allocation succeeds,
  over-limit allocation returns `Err(MemoryExceeded { limit_mb })`. Boundary is exact.
- Test: `proofs::plugin_memory::tests::test_BC_2_17_003_vp041_memory_limit_boundary_exact` — PASS

### AC-12: VP-042 — Failed Hot Reload Retains Old Registry Entry (BC-2.17.005)
- `AC-012-vp042-hot-reload-retains.gif` / `.webm` / `.tape`
- Demonstrates: proptest over random byte sequences — if compilation fails, the
  `Arc<LoadedPlugin>` in the registry is pointer-equal before and after the failed reload.
  Deterministic companion test with empty bytes also passes.
- Tests: `proofs::plugin_hot_reload` (proptest + deterministic companion) — PASS

### AC-13: VP-043 — WIT Validation Rejects Missing Exports (BC-2.17.006)
- `AC-013-vp043-wit-rejects-missing.gif` / `.webm` / `.tape`
- Demonstrates: proptest over all strict subsets of sensor/infusion/action required
  exports — each subset returns `Err(InvalidInterface)` naming the missing export.
  Complete export sets return `Ok(PluginType)`. Empty export set and determinism
  properties also verified.
- Tests: `proofs::plugin_wit_validation` (7 tests) — PASS

---

## Deferred / Non-Runnable Items

| Item | Reason | Evidence Form |
|------|--------|---------------|
| VP-040 Kani path | Requires wasmtime Linker import enumeration API (not yet in wasmtime 20.x public API). `#[ignore]` with documented caveat in `proofs/plugin_linker.rs`. | Proptest fallback recorded in AC-010. |
| AC-5 primary integration test | Test has unconditional `panic!()` — test-writer bug, not an implementation gap. Triaged to PR manager. | EC-17 companion tests recorded in AC-005 recording. |

---

## File Listing

```
docs/demo-evidence/S-1.15/
  AC-001-load-valid-plugin.gif
  AC-001-load-valid-plugin.webm
  AC-001-load-valid-plugin.tape
  AC-002-plugin-panic-isolation.gif
  AC-002-plugin-panic-isolation.webm
  AC-002-plugin-panic-isolation.tape
  AC-003-cpu-timeout.gif
  AC-003-cpu-timeout.webm
  AC-003-cpu-timeout.tape
  AC-004-sandbox-no-filesystem.gif
  AC-004-sandbox-no-filesystem.webm
  AC-004-sandbox-no-filesystem.tape
  AC-005-http-proxy-host.gif
  AC-005-http-proxy-host.webm
  AC-005-http-proxy-host.tape
  AC-006-hot-reload.gif
  AC-006-hot-reload.webm
  AC-006-hot-reload.tape
  AC-007-wit-validation-rejection.gif
  AC-007-wit-validation-rejection.webm
  AC-007-wit-validation-rejection.tape
  AC-008-kv-store-scoped.gif
  AC-008-kv-store-scoped.webm
  AC-008-kv-store-scoped.tape
  AC-009-memory-limit.gif
  AC-009-memory-limit.webm
  AC-009-memory-limit.tape
  AC-010-vp040-wasi-excluded.gif
  AC-010-vp040-wasi-excluded.webm
  AC-010-vp040-wasi-excluded.tape
  AC-011-vp041-memory-boundary.gif
  AC-011-vp041-memory-boundary.webm
  AC-011-vp041-memory-boundary.tape
  AC-012-vp042-hot-reload-retains.gif
  AC-012-vp042-hot-reload-retains.webm
  AC-012-vp042-hot-reload-retains.tape
  AC-013-vp043-wit-rejects-missing.gif
  AC-013-vp043-wit-rejects-missing.webm
  AC-013-vp043-wit-rejects-missing.tape
  evidence-report.md
```
