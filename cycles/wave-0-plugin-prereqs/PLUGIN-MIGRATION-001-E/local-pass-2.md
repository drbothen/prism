# PLUGIN-MIGRATION-001-E — LOCAL Adversary Pass-2

**Date:** 2026-05-22
**Feature HEAD:** `319263ff`
**Cascade state at start:** streak 0/3, pass-2 of N
**Closures verified (Part A sampling):** 4/4 CRITs, 4/7 HIGHs, 2/6 MEDs, 1/1 LOW, 1/1 PROCESS-GAP

## Part A — Pass-1 closure durability verification

| Pass-1 Finding | Closure Status | Notes |
|---|---|---|
| F-LP1-CRIT-001 | DURABLE | Plugin guest is real Rust (no `todo!()`); JSON parse + TTL math real; but cache-hit path is dead (see F-LP2-CRIT-001) |
| F-LP1-CRIT-002 | DURABLE | WAT fixture at offset 18, len 25 ("oauth2_client_credentials"); test invokes export via wasmtime Linker |
| F-LP1-CRIT-003 | **PAPER-FIX** | `validate_auth_plugin_registered` defined; ZERO production callers (test-only) — see F-LP2-CRIT-002 |
| F-LP1-CRIT-004 | DURABLE | `current-time-secs: func() -> u64;` added; host registers it |
| F-LP1-HIGH-005 | DURABLE | `#[non_exhaustive]` on `HttpResponse` |
| F-LP1-HIGH-006 | DURABLE | Real tracing capture asserts `plugin_load_unsigned` |
| F-LP1-HIGH-007 | DURABLE | Real tracing capture asserts no token leak |
| F-LP1-HIGH-010 | **PAPER-FIX** | `PluginAuthProvider` exists; no production construction — see F-LP2-HIGH-001 |
| F-LP1-HIGH-011 | DURABLE | `build_test_runtime()` uses 30s timeout |
| F-LP1-MED-012/013 | DURABLE | Workspace lints + workspace deps |
| F-LP1-MED-017 | DURABLE | Justfile recipe exists — but see F-LP2-HIGH-003 + MED-001 |
| F-LP1-LOW-018 | DURABLE | `#![allow(dead_code)]` removed |

## Part B — NEW findings

### F-LP2-CRIT-001 — Token cache destroyed every dispatch; `get_token` cache-hit path is dead in production [CRIT, HIGH confidence]

**Evidence:**
- `crates/prism-spec-engine/src/plugin/mod.rs:768-777` `make_host_state` constructs `kv_store: Arc::new(PluginKvStore::new())` — fresh empty store EVERY call.
- `LoadedPlugin` struct (loader.rs:85-99) has NO `kv_store` field — no per-plugin persistent KV.
- Plugin guest `get_token` reads kv twice; both reads will ALWAYS return None across separate dispatches.
- AC-004 test passes because it stays inside ONE `HostState`.

**Why it fails:** Story AC-004 requires "Token cached within TTL; subsequent calls reuse cache (no second request)." Production will issue a fresh POST /oauth2/token on EVERY API call — P0 perf + rate-limit regression vs legacy `CrowdStrikeAdapter`.

**Routing:** implementer/architect — `LoadedPlugin` must carry `Arc<PluginKvStore>`; `make_host_state` clones the Arc across dispatches. Add test that calls `dispatch_plugin_acquire_token` twice and asserts only ONE POST.

---

### F-LP2-CRIT-002 — `validate_auth_plugin_registered` defined but never called from production boot path; CRIT-003 closure is paper-fix [CRIT, HIGH confidence]

**Evidence:**
- `crates/prism-spec-engine/src/validation.rs:641` defines the function; re-exported at lib.rs:86.
- Grep returns ONLY test callers in `crowdstrike_oauth2_plugin_tests.rs` (lines 802, 838, 880).
- `crates/prism-bin/src/boot.rs` step 7.5 loads plugins at line 1033 but has ZERO calls to validate_auth_plugin_registered.
- Validator's own docstring (validation.rs:629) says "In boot.rs step 7.5, after plugins are loaded, iterate over all loaded SensorSpecs and call this validator" — that iteration is NOT implemented.

**Why it fails:** Pass-1 CRIT-003 said typo'd `auth_plugin = "crowdstirke-oauth2"` would silently break in production. That risk REMAINS unaddressed. Standing Rule 3 §3 violation. TD-VSDD-059 paper-fix.

**Routing:** implementer — add call site in boot.rs step 7.5: iterate loaded SensorSpecs, build `HashSet<String>` of registered plugin_ids, call validator, propagate `UnknownAuthPlugin` as BootError with exit code 2. Add integration test in prism-bin/tests/ driving boot with typo'd TOML.

---

### F-LP2-HIGH-001 — No production construction site for `PluginAuthProvider`; HIGH-010 closure is paper-fix [HIGH, HIGH confidence]

**Evidence:**
- Grep `PluginAuthProvider::new` returns matches only in module file + test file.
- `prism-bin/src/boot.rs` and `prism-sensors/src/registry.rs` (AdapterRegistry::init_registry_for_org) — ZERO references.

**Why it fails:** After 001-A merges and deletes crowdstrike.rs, CrowdStrike will have NO AuthProvider in production. ADR-028 §D10 co-merge gate STILL structurally unsatisfiable. Standing Rule 3 §4 violation.

**Routing:** architect → implementer. Architect: decide construction location (boot.rs step 7.5 OR AdapterRegistry::init_registry_for_org with PluginRuntime injection). Implementer: wire `Arc<PluginAuthProvider>` per sensor spec where `spec.auth_plugin.is_some()`. Add boot integration test loading crowdstrike TOML + plugin and asserting PluginAuthProvider is the registered auth provider.

---

### F-LP2-HIGH-002 — Plugin guest hand-rolls bindings via `#[link]` extern blocks; "wit-bindgen wired" claim contradicted by source [HIGH, HIGH confidence]

**Evidence:**
- `lib.rs:112-145` declares manual `#[link(wasm_import_module = "host")] extern "C"` blocks.
- `lib.rs:106` admits: "A wit-bindgen pass would generate these automatically; for now we use the manual pattern."
- `lib.rs:117` admits: "Production Component ABI would be generated by wit-bindgen."
- Plugin Cargo.toml has NO wit-bindgen dependency. No build.rs. No wit_bindgen::generate! macro.
- Simplified ABI packs `(status << 32 | len)` into u64 + empty body bytes — NOT Component Model compatible.

**Why it fails:** Direct MVP-deferral language ("for now") forbidden by CLAUDE.md Canonical Principle Rule 1. Component build will produce a binary that does NOT interoperate with host's Component Model dispatch (`Val::S64` mismatch). AC-006's production-grade claim regresses.

**Routing:** implementer — add wit-bindgen to plugin Cargo.toml, replace manual extern blocks with `wit_bindgen::generate!` pointed at WIT file. Verify Component Model ABI alignment via non-WAT integration test loading real wit-bindgen-compiled .prx.

---

### F-LP2-HIGH-003 — `Justfile build-plugin-crowdstrike-oauth2` recipe silently falls through on Component validation failure [HIGH, MEDIUM confidence]

**Evidence:**
- `Justfile:216-225` uses `wasm-tools validate --features=component-model ... && echo "PASS" || echo "INFO: core module produced..."`
- Both branches exit 0; recipe never returns non-zero even on Component validation FAIL.
- References `tests/fixtures/wasi_snapshot_preview1.wasm` for --adapt (verify existence).

**Why it fails:** CI pipeline running this recipe will silently fall through to "INFO" even if Component build genuinely fails — false-green per CI-as-Code Review Axis. Anti-pattern.

**Routing:** dx-engineer/implementer — verify wasi_snapshot_preview1.wasm exists (fail-fast at recipe start); replace `|| echo "INFO"` with `|| exit 1`; add positive-coverage assertion `wasm-tools print ... | grep -q '(component'`.

---

### F-LP2-HIGH-004 — Plugin native stub paths use `panic!()` in non-test, non-WASM build [HIGH, MEDIUM confidence]

**Evidence:**
- `lib.rs:209-235` — native host stubs use `panic!(...)`. NOT `#[cfg(test)]`; gated by `#[cfg(not(target_arch = "wasm32"))]`.
- In `cargo check --target x86_64-apple-darwin`, panics are reachable production code.

**Routing:** implementer — either (a) gate entire `host_impl` native module under `#[cfg(test)]` to prove non-production-reachable, OR (b) return `Result<HttpResponse, AuthError>` from native stubs and propagate via `?`. Option (a) is cheapest.

---

### F-LP2-HIGH-005 — `str::from_utf8_unchecked` on WASM-host-supplied bytes is UB if host ever passes non-UTF8 [HIGH, MEDIUM confidence]

**Evidence:**
- `lib.rs:417, 421, 455, 459` use `std::str::from_utf8_unchecked(slice)`.
- Justification cites WASM linear memory pointer validity — but pointer validity != UTF-8 validity.

**Routing:** implementer — replace `from_utf8_unchecked` with `from_utf8(...).map_err(|_| AuthError::Internal("...".into()))?` and propagate. Defense-in-depth aligned with production-grade default.

---

### F-LP2-MED-001 — Justfile recipe writes .prx artifact that no test or CI consumes [MED, MEDIUM confidence]

**Evidence:**
- `Justfile:215, 219` writes `crowdstrike-oauth2.prx`.
- AC-001/002/006 tests use WAT-compiled bytes in tempdir, never reference built artifact.
- No CI invocation of build recipe.

**Routing:** implementer — either (a) add `#[ignore]`d integration test loading the .prx post-build (with citation to follow-up story to un-ignore), OR (b) add CI job running build + Component validation + load via PluginRuntime.

---

### F-LP2-MED-002 — `PluginAuthProvider::acquire_token` stringifies structured PluginError into AuthAcquisitionFailed.detail [MED, MEDIUM confidence]

**Evidence:**
- `plugin_auth_provider.rs:113-118` converts `plugin_err.to_string()` into `SpecEngineError::AuthAcquisitionFailed { detail: ... }`.
- `client_id` hardcoded to `"plugin-auth"` sentinel (line 116) — obscures real org_id.

**Routing:** implementer — extend AuthAcquisitionFailed with `cause: Option<PluginError>` OR add `AuthPluginDispatchFailed { plugin_id, plugin_error: PluginError }`. Wire `client_id` from real `_client_id.as_str()`.

---

### F-LP2-MED-003 — AC-006 401-retry test assertion is weak [MED, MEDIUM confidence]

**Evidence:**
- `crowdstrike_oauth2_plugin_tests.rs:720-724` asserts only `request_count >= 2`.
- Does NOT assert second request used refreshed token; does NOT spy on `PluginAuthProvider::acquire_token` call count.

**Routing:** implementer — wrap PluginAuthProvider in `Arc<AtomicUsize>` counter; assert exactly 1 refresh, second request carries `Bearer wat-fixture-token`.

---

### F-LP2-LOW-001 — Stale comment in prism-spec-engine/Cargo.toml lines 213-217 references `test-helpers` feature [LOW, MEDIUM confidence]

**Routing:** implementer — reconcile or remove the misleading comment.

---

### F-LP2-OBS-001 — Adversary pass-1 byte count for "oauth2_client_credentials" was 24; correct count is 25 [OBS, informational]

The plugin source's author-comment notes pass-1 cited wrong byte count. The underlying defect (test not invoking export) was real; the byte count was over-broad. Informational.

---

## CLEAN strict: NO — CLEAN (PR-merge): NO

**Reason:** 2 CRIT, 5 HIGH, 3 MED, 1 LOW, 1 OBS = 12 findings.

## Streak advancement: 0/3 → 0/3 (no change)

## Novelty Assessment

HIGH novelty. NEW classes not addressed in pass-1:
- F-LP2-CRIT-001 (KV-store-per-call) — deep architectural defect pass-1 missed entirely
- F-LP2-CRIT-002 + HIGH-001 — TWO paper-fix detections; building blocks exist but unwired in production
- F-LP2-HIGH-002 — direct contradiction between closure claim and source ("for now")
- F-LP2-HIGH-003 — SAP-1-class false-green in build tooling

**Conclusion:** Recurring paper-fix pattern is structural. Fresh context surfaced 2 new CRITs and 5 new HIGHs FB-IMPL-1 missed. FB-IMPL-2 dispatch needed — focus on wire-up gaps + wit-bindgen rework.

## Total counts

| Severity | Count |
|---|---|
| CRIT | 2 |
| HIGH | 5 |
| MED | 3 |
| LOW | 1 |
| OBS | 1 |
| **TOTAL** | **12** |
