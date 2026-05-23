---
document_type: fix-burst-report
cascade: PLUGIN-MIGRATION-001-E LOCAL
burst_number: 4
date: 2026-05-22
feature_head_before: d7ec60a7
feature_head_after: 639d89e1
develop_head_baseline: f19575ff
findings_addressed: [F-LP5-HIGH-001, F-LP5-HIGH-002]
findings_closed: 2
findings_deferred: 1
micro_commits: 2
workspace_test_count: 3760
just_check_status: PASS
inputs:
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/local-pass-5.md
  - .factory/stories/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E.md
input-hash: "[live-fb-4]"
---

# PLUGIN-MIGRATION-001-E — FB-IMPL-4

**Date:** 2026-05-22
**Feature HEAD before:** `d7ec60a7`
**Feature HEAD after:** `639d89e1`
**Develop baseline:** `f19575ff`
**Findings addressed:** F-LP5-HIGH-001 + F-LP5-HIGH-002
**Micro-commits:** 2 (fd16c0bc + 639d89e1)
**Workspace tests:** 3760 GREEN
**just check:** PASS

## F-LP5-HIGH-001 — wit-bindgen Guest impl + CI wasm32 compile gate

### Root cause discovered during the fix

The pass-3 F-LP3-HIGH-001 closure (added `Guest` impl + `export!(Component)` in `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs`) was a **structural compilation failure masquerading as completion**. `sensor-auth.wit` placed `record http-response`, `enum log-level`, and `variant auth-error` at the top level (outside any `interface` block). wit-bindgen 0.51+ requires type definitions inside `interface` blocks. `cargo check --target wasm32-wasip1 -p crowdstrike-oauth2-plugin` failed with: `expected world, interface or use, found keyword record`. This had never been observed because no CI step compiled the wasm32 target and no host-target build invokes the wit-bindgen macro (gated `#[cfg(target_arch = "wasm32")]`).

The pass-5 structural-coverage axis exposed this — the "Guest impl present in source" check would have continued to pass forever; the "does it actually compile" check failed on first attempt.

### Closure

1. **`wit/sensor-auth.wit`** — moved `record http-response`, `enum log-level`, and `variant auth-error` inside the `host` interface block. Updated `interface sensor-auth` to `use host.{auth-error}` (dropped `http-response` from the `use` list since `sensor-auth` function signatures don't reference it).
2. **`.github/workflows/ci.yml`** — added a new job `wasm32-compile-check` (depends on `clippy`, runs on `ubuntu-latest`, installs the `wasm32-wasip1` target via `dtolnay/rust-toolchain`, runs `cargo check --target wasm32-wasip1 -p crowdstrike-oauth2-plugin`). Added a reachability assertion in the existing `verify-workflow-structure` job so the new job cannot be silently dropped.

### Local verification

`cargo check --target wasm32-wasip1 -p crowdstrike-oauth2-plugin` exits 0.

### Anti-paper-fix evidence

The fix is a real CI YAML step that runs on every push. If the wit-bindgen macro stops compiling for any reason (signature drift, attribute change, Guest trait change), the `wasm32-compile-check` job fails CI. The "wit-bindgen Guest impl is present" claim is now load-bearing — the macro is exercised every build.

### Deferred (in-scope-justified)

**Option (a)** — checking in `tests/fixtures/wasi_snapshot_preview1.wasm` + the built `.prx` + adding `just build-plugin-crowdstrike-oauth2` to CI + un-`#[ignore]`-ing `test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime` — deferred to story **S-PLUGIN-CI-001**. Blocking dependency: wasm-tools + Wasmtime component adapter toolchain availability decision is not in scope for PLUGIN-MIGRATION-001-E. Option (b) — the finding's stated minimum closure — is complete and structurally hardens against the F-LP3-HIGH-001 / F-LP5-HIGH-001 recurrence pattern.

## F-LP5-HIGH-002 — HostInterface trait + MockHost + 9 behavioral unit tests

### Closure

1. **`trait HostInterface`** defined in `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` (gated `#[cfg(any(target_arch = "wasm32", test))]`) with 5 methods: `http_request`, `kv_get`, `kv_set`, `current_time_secs`, `get_config`. Signatures match the existing `host_impl::*` surface verbatim — the trait is a wrapper for testability, not a redesign.
2. **`WasmHost`** unit struct (wasm32-only) implements `HostInterface` by delegating to `host_impl::*`. The Guest impl now constructs `WasmHost` and passes it to `acquire_token` / `get_token`.
3. **`acquire_token` + `get_token`** refactored to accept `host: &impl HostInterface`.
4. **Native `host_impl` stub module removed** (was panic!()-bodies; now dead code because tests use MockHost).
5. **`MockHost`** in `#[cfg(test)] mod tests`: FIFO HTTP response queue, `RefCell`-backed KV store, kv_set error injection.

### 9 behavioral tests (all variant-matching, not bare `is_err()` / `is_ok()`)

| Test | Edge case | Assertion |
|---|---|---|
| `test_acquire_token_EC_001_401_returns_invalid_credentials` | EC-001 | `matches!(err, AuthError::InvalidCredentials)` + KV write count == 0 |
| `test_acquire_token_EC_002_non_2xx_returns_response_parse` | EC-002 | `matches!(err, AuthError::ResponseParse)` + status code in detail |
| `test_acquire_token_EC_003_missing_access_token_returns_response_parse` | EC-003 | `matches!(err, AuthError::ResponseParse)` + "missing access_token" in detail |
| `test_acquire_token_EC_004_missing_expires_in_defaults_to_1799` | EC-004 | KV `expires_at_secs = now + 1769` (1799 - 30 saturating_sub) |
| `test_acquire_token_EC_005_kv_set_error_propagates` | EC-005 | `matches!(err, AuthError::Internal)` + exact error message |
| `test_get_token_cache_hit_returns_cached_value` | cache hit | HTTP call count == 0 |
| `test_get_token_cache_miss_calls_acquire_token` | cache miss | HTTP call count == 1 |
| `test_get_token_cache_hit_but_empty_token_treats_as_miss` | empty-cached-token branch | HTTP call count == 1 |
| `test_acquire_token_form_body_contains_required_params` | form construction | request body contains `grant_type=client_credentials` |

### Test results

`cargo nextest run -p crowdstrike-oauth2-plugin` — 11 tests (2 pre-existing + 9 new) all GREEN. Zero warnings. Zero clippy errors.

### Anti-paper-fix evidence

Each test asserts on a SPECIFIC behavioral signature (error variant, KV-store contents, HTTP call count, request body content) — not just `is_err()` / `is_ok()`. A regression flipping `if response.status == 401` to `if response.status != 401`, or changing `unwrap_or(1799)` to `unwrap_or(0)`, or removing the `!cached_token.is_empty()` branch would now fail at least one specific test with a clear failure message.

EC-006..EC-009 remain covered by wasm32 Guest impl / WAT-fixture / integration tests (not closed in FB-IMPL-4 — out of scope for this burst; tracked in the story §Edge Cases table).

## Workspace verification

`just check`: PASS. 3760 tests run, 3760 passed, 25 skipped (the 25 are the pre-existing `#[ignore]`'d set including MED-001's DTU-gated and S-PLUGIN-CI-001-gated tests). Net new tests added in this burst: 9.

## Files modified

- `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` (HostInterface trait + WasmHost + MockHost + 9 tests + native stub removal)
- `crates/prism-spec-engine/plugins/crowdstrike-oauth2/wit/sensor-auth.wit` (type defs moved inside `host` interface)
- `.github/workflows/ci.yml` (+40 lines: new `wasm32-compile-check` job + reachability assertion)

## Streak attempt impact

FB-IMPL-4 closes the two HIGH findings from pass-5. Next adversary pass (pass-6) attempts streak 0/3 → 1/3 with the structural-coverage axis now part of the cascade discipline. Pass-6 must apply the same axis to verify FB-IMPL-4 closures are themselves load-bearing (not paper-fix recurrence — though the CI step + variant-matching test pattern structurally prevent the previous failure mode).
