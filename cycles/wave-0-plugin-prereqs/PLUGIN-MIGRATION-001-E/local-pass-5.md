---
document_type: adversary-pass-report
cascade: PLUGIN-MIGRATION-001-E LOCAL
pass_number: 5
date: 2026-05-22
feature_head: d7ec60a7
develop_head_baseline: f19575ff
streak_before: 1/3
streak_after: 0/3
clean_strict: false
clean_pr_merge: false
findings_total: 2
findings_by_severity:
  CRIT: 0
  HIGH: 2
  MED: 0
  LOW: 0
  OBS: 0
  PROCESS-GAP: 0
decay_trajectory: "20 → 12 → 3 → 0 → 2"
inputs:
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/local-pass-{1,2,3,4}.md
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/fix-burst-{1,2,3}.md
  - .factory/stories/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E.md
  - crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs
  - tests/integration/crowdstrike_oauth2_plugin_tests.rs
  - Justfile
  - .github/workflows/
input-hash: "[live-pass-5]"
---

# PLUGIN-MIGRATION-001-E — LOCAL Adversary Pass-5

**Date:** 2026-05-22
**Feature HEAD:** `d7ec60a7`
**Develop HEAD baseline:** `f19575ff`
**Cascade state at start:** streak 1/3, pass-5
**Decay trajectory:** 20 → 12 → 3 → 0 → 2

## Part A — Pass-3 + Pass-4 closure durability

| Finding | Closure status | Verdict |
|---|---|---|
| F-LP3-HIGH-001 (wit-bindgen Guest impl + export!(Component)) | CODE present in lib.rs `mod host_impl`; manual `*_export` wrappers DELETED | **CODE-DURABLE / TEST-UNDURABLE** — see F-LP5-HIGH-001 |
| F-LP3-MED-001 (validate_and_construct_auth_providers extraction + 4 behavioral tests) | 4 tests present at plugin_boot_tests.rs (happy_path / typo / empty / mixed) | DURABLE |
| F-LP3-LOW-001 (event_type field + BC-2.16.002 row) | BC-2.16.002 row 113 still present; emission at boot.rs still has event_type="plugin_auth_provider_constructed" | DURABLE |
| Pass-4 baseline (0 findings) | Re-derived: spot-checked KV-store cross-call sharing — all dispatch sites pass `plugin.kv_store.clone()` not `Arc::new(PluginKvStore::new())` | DURABLE |

**Regression count: 0. Paper-fix count: 1 (F-LP3-HIGH-001 — code-durable but never compiled in CI; never exercised by any non-#[ignore]'d test).**

## Part B — NEW findings

### F-LP5-HIGH-001 — wit-bindgen `Guest` impl + `export!(Component)` is unbuilt and untested; F-LP3-HIGH-001 closure is paper-fix (3rd pattern recurrence) [HIGH, HIGH confidence]

**Surface:** `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` `mod host_impl` (gated `#[cfg(target_arch = "wasm32")]`); Justfile recipe `build-plugin-crowdstrike-oauth2`; `tests/external/` and `tests/fixtures/wasi_snapshot_preview1.wasm`.

**Evidence:**
- `lib.rs` `mod host_impl` containing `wit_bindgen::generate!`, `impl Guest for Component`, and `export!(Component)` is gated `#[cfg(target_arch = "wasm32")]`. Host-target builds (`cargo check`, `just check`, all CI runs) do NOT compile this module — so the wit-bindgen macro never runs and the export symbols are never emitted in any build that CI executes.
- The Justfile recipe `build-plugin-crowdstrike-oauth2` requires `tests/fixtures/wasi_snapshot_preview1.wasm` as the first fail-fast precondition. Glob('tests/fixtures/wasi_snapshot_preview1.wasm') against the worktree returns NO files. The recipe is non-runnable.
- Glob('crates/prism-spec-engine/plugins/crowdstrike-oauth2/*.prx') returns NO files — the `.prx` artifact is not built and not checked into the repo.
- The only test that loads the real built `.prx` is `test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime` — `#[ignore]`'d pending S-PLUGIN-CI-001, AND requires the absent .prx to even start.
- CI workflow audit: `Grep('build-plugin-crowdstrike-oauth2|wasm32-wasip1', .github/workflows)` returns NO matches.
- The "WAT-fixture test path exercises 3 kebab-case exports" claim in pass-4 closure: `CROWDSTRIKE_OAUTH2_WAT` is a hand-written core-module WAT with manually-named kebab-case exports. It does NOT exercise the wit-bindgen Guest impl. They are two distinct code paths.

**Why it fails:** Pass-3 F-LP3-HIGH-001 was opened because hand-rolled snake_case exports failed `validate_wit_interface`. The closure ADDED a Guest impl + `export!(Component)`, but that code is never compiled by CI, never linked, never invoked. Any breakage in the wit-bindgen attribute set, the Guest trait signature, the `to_wit_auth_error` mapping, the `get-config("token_endpoint")` lookup, or `wit_bindgen::generate!` macro output would not be caught by any test in the workspace. The F-LP3-HIGH-001 risk (built `.prx` failing `validate_wit_interface`) is structurally unchanged: there is no built `.prx` in the repo, and the code that would produce one is unverified. TD-VSDD-059 paper-fix pattern recurrence (3rd: F-LP1-CRIT-001 → F-LP2-CRIT-001 → F-LP2-HIGH-001 → F-LP3-HIGH-001 → now). Standing Rule 3 §1 violation: claim ("Component model dispatch via wit-bindgen") without enforcement gate (no compile, no link, no exercise).

**Routing:** dx-engineer/implementer — three remediation options, all in-scope under Canonical Principle Rule 4:
(a) Check the .wasm adapter fixture into the repo + run `build-plugin-crowdstrike-oauth2` in CI: copy `wasi_snapshot_preview1.wasm` from upstream wasmtime release into `tests/fixtures/`, add a CI job step `just build-plugin-crowdstrike-oauth2` after the existing `cargo build` step, and check the produced `.prx` into the repo. Un-`#[ignore]` `test_PLUGIN_MIGRATION_001_E_med_001` once .prx is committed. This satisfies F-LP3-HIGH-001 with structural test coverage.
(b) Add a `cargo check --target wasm32-wasip1 -p crowdstrike-oauth2-plugin` step in CI: minimally proves the `host_impl` module + wit-bindgen macro compiles, even if not lifted to a Component. Catches signature drift and macro-expansion errors. Lower assurance than (a) but cheap.
(c) Move wit-bindgen-Guest-impl validation to a host-target proof crate: introduce a `prism-wit-validator` test crate using `wit-parser` to load `wit/sensor-auth.wit` and verify the Guest trait signature matches what production expects. Slower payoff but addresses the root cause (host build cannot exercise wasm-gated code).

Default routing: (a) + (b) combined — (b) gets CI coverage in this fix-burst, (a) requires S-PLUGIN-CI-001 toolchain availability.

**Paper-fix risk:** HIGH. A doc-comment-only closure ("added cargo check for wasm32") that doesn't actually run in CI would recurse this pattern. Closure must include an actual CI YAML step + a passing CI run.

---

### F-LP5-HIGH-002 — `acquire_token` and `get_token` Rust functions have ZERO load-bearing test coverage; "Plugin guest is real Rust" closure narrative (pass-2) was structural-not-behavioral [HIGH, HIGH confidence]

**Surface:** `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` functions `acquire_token` and `get_token`; test module `#[cfg(test)] mod tests`.

**Evidence:**
- `acquire_token(credential_handle, token_endpoint)` is gated `#[cfg(any(target_arch = "wasm32", test))]`. The body contains: form-body construction, HTTP dispatch via `host_impl::http_request`, status code branching (401 → InvalidCredentials; non-2xx → ResponseParse), JSON parse via `serde_json::from_str`, access_token extraction, expires_in default-1799 fallback (EC-004), `host_impl::current_time_secs()` call, `expires_at = now + expires_in.saturating_sub(30)`, two `host_impl::kv_set` calls — a substantial amount of load-bearing logic.
- `get_token(credential_handle, token_endpoint)` body contains: TTL check (`expires_at_str.parse::<u64>` + comparison), cache hit return path, cache miss fallthrough to `acquire_token`.
- The native `#[cfg(test)] mod tests` contains EXACTLY two tests (`test_auth_type_name_returns_canonical_value` + `test_auth_type_name_byte_length_is_25`) — both only exercise the trivial constant-return `auth_type_name()`. No test exercises `acquire_token` or `get_token`.
- The `#[cfg(any(target_arch = "wasm32", test))]` gate means `acquire_token`/`get_token` ARE compiled in test mode — but on native test builds, every `host_impl::*` call panics (native stubs are `panic!()` bodies). Any unit test invoking `acquire_token` natively would panic at the first `host_impl::http_request` call. So unit testing in this crate is structurally impossible without a host-function mock layer — and none exists.
- The integration test path (`crowdstrike_oauth2_plugin_tests.rs`) uses WAT fixtures whose `acquire-token` export returns hardcoded `i32 i32` constants. The WAT does NOT call into the Rust `acquire_token` function. They are two unrelated code paths.
- The integration test `test_PLUGIN_MIGRATION_001_E_003` exercises `host_http_request` directly — it tests the HOST side, not the plugin's `acquire_token` logic.
- Therefore: EC-001 (401 branch), EC-002 (non-2xx parse error), EC-003 (missing access_token), EC-004 (expires_in default 1799 fallback), EC-005 (KV size limit propagation), the TTL `saturating_sub(30)` math, and the cache-hit-then-empty-token edge case in `get_token` — all are claimed by the story and the closures, none are exercised by any load-bearing test.

**Why it fails:** Story §Edge Cases enumerates EC-001 through EC-009 with explicit "Expected Behavior" rows. The plugin code branches on these conditions, but no test asserts the branches. A regression flipping `if response.status == 401` to `if response.status != 401`, or changing the `unwrap_or(1799)` default to `unwrap_or(0)`, or removing the `!cached_token.is_empty()` check, would compile clean and ship green. TD-VSDD-059 paper-fix on the entire core-flow logic. The pass-2 F-LP1-CRIT-001 closure asserted "Plugin guest is real Rust (no `todo!()`); JSON parse + TTL math real" — this is true at the code level but VACUOUS at the verification level. The pass-2 narrative led the cascade to declare pass-4 CLEAN despite zero behavioral test coverage on the EC table.

**Routing:** implementer — extract the host-side I/O into a thin trait (`trait HostInterface { fn http_request(...); fn kv_get(...); fn kv_set(...); fn current_time_secs(); fn get_config(...); }`), make `acquire_token` + `get_token` accept `&impl HostInterface` instead of calling `host_impl::*` directly, and add a `MockHost` in `#[cfg(test)] mod tests` that records calls + returns canned responses. Then add unit tests covering EC-001 / EC-002 / EC-003 / EC-004 / EC-005 / cache-hit / cache-miss / cache-empty-token / TTL-saturation. ~10 unit tests total. The wasm32 path keeps calling `host_impl::*` via a concrete `WasmHost` impl. SID-1 compliant: unit tests at production module's `#[cfg(test)] mod tests` driving the behavior via mock at the dependency boundary, no external dependency required.

**Paper-fix risk:** HIGH. A doc-comment-only closure ("added tests for acquire_token") with shallow assertions (e.g., only `result.is_err()` without variant matching) would recurse the pattern. Closure must include: per-EC behavioral test with variant matching (`assert!(matches!(err, AuthError::InvalidCredentials))`), assertions on KV-set call ordering, assertions on the form-body content, and assertions on the cache-hit fast path. Minimum 8 tests covering the EC table + cache logic.

---

## CLEAN (strict): no
## CLEAN (PR-merge): no

**Reason:** 2 HIGH findings (paper-fix recurrence patterns under structural-coverage lens). CLEAN (PR-merge) requires zero CRIT+HIGH+MED → NO. CLEAN (strict) requires zero ANY severity → NO.

## Streak advancement: 1/3 → **0/3** (RESET per BC-5.39.001)

## Novelty Assessment

HIGH novelty. Pass-4 spot-checked closures at the CODE level (Guest impl present, 4 behavioral tests present, BC catalog row present) and declared CLEAN — but did NOT verify the closures were exercised by CI / test runners. Fresh-context discipline surfaced the structural compilation gap (wit-bindgen Guest never built) AND the missing host-function abstraction (acquire_token / get_token Rust logic never exercised by any test). This is the SAME paper-fix pattern that recurred in passes 1→2→3 — implementer code is real, but test coverage is structural-not-behavioral. The cascade has NOT structurally hardened — the underlying coverage discipline gap persists.

## Decay trajectory

| Pass | Findings | Severity high-water | Closure depth |
|---|---|---|---|
| 1 | 20 | 4 CRIT, 7 HIGH | code-level review |
| 2 | 12 | 2 CRIT, 5 HIGH | wire-up + paper-fix detection |
| 3 | 3 | 0 CRIT, 1 HIGH | wit-bindgen exports + test extraction |
| 4 | 0 | — | code-level durability sample |
| 5 | 2 | 0 CRIT, 2 HIGH | **structural-coverage verification (new axis)** |

Pass-5 introduces a NEW review axis that prior passes did not apply: structural-coverage verification — "is the closure code actually compiled and exercised by CI?". This axis catches the F-LP5-HIGH-001 + F-LP5-HIGH-002 class which is invisible to file-presence and test-count checks alone.

## Recommended next action

Dispatch FB-IMPL-4 to implementer:
1. (HIGH-001) Add `cargo check --target wasm32-wasip1 -p crowdstrike-oauth2-plugin` to CI workflow (minimum proof the Guest impl compiles). Optionally check in `wasi_snapshot_preview1.wasm` + `crowdstrike-oauth2.prx` and un-`#[ignore]` MED-001 test. Both paths require commit messages citing this finding.
2. (HIGH-002) Extract `HostInterface` trait, add 8+ unit tests in `lib.rs` `#[cfg(test)] mod tests` covering EC-001 through EC-005 + cache-hit/miss/empty-token paths.
3. Re-run pass-6 with same structural-coverage axis applied; require BOTH HIGH-001 and HIGH-002 closures show non-doc-comment evidence (real CI step + real assertion counts).

After FB-IMPL-4 lands, pass-6 attempts streak 0/3 → 1/3.

## Total counts

| Severity | Count |
|---|---|
| HIGH | 2 |
| **TOTAL** | **2** |
