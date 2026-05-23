# PLUGIN-MIGRATION-001-E — LOCAL Adversary Pass-3

**Date:** 2026-05-22
**Feature HEAD:** `1d06a4bf`
**Cascade state at start:** streak 0/3, pass-3 of N

## Part A — Pass-2 closure durability verification

| Pass-2 Finding | Closure Status | Production Caller Verified |
|---|---|---|
| F-LP2-CRIT-001 (Arc<KvStore>) | DURABLE | `mod.rs` callers: dispatch_plugin_acquire_token (line 671-676), enrich_single (819-823), enrich_batch (902); loader.rs LoadedPlugin.kv_store field (107, 124) |
| F-LP2-CRIT-002 (validate_auth_plugin_fields) | DURABLE | boot.rs:228-245 run_boot_sequence step 7.5b; BootError::UnknownAuthPlugin exit 2 at boot.rs:126 |
| F-LP2-HIGH-001 (PluginAuthProvider production) | DURABLE-CODE / WEAK-TEST | boot.rs:254-263 step 7.5b constructs Arc<PluginAuthProvider>; integration test exists at plugin_boot_tests.rs:1306 but only verifies empty map for empty plugin dir. NO integration test drives step 7.5b iteration with non-empty sensor_specs. See F-LP3-MED-001 |
| F-LP2-HIGH-002 (wit-bindgen) | PARTIAL | Host imports rewired via wit_bindgen::generate!; guest exports still hand-rolled core-module ABI. See F-LP3-HIGH-001 |
| F-LP2-HIGH-003 (Justfile) | DURABLE | Recipe exits non-zero on validation failure; positive (component grep added |
| F-LP2-HIGH-004 (panic gating) | DURABLE | Native stubs gated under #[cfg(all(not(target_arch = "wasm32"), test))] |
| F-LP2-HIGH-005 (UTF-8 safety) | DURABLE | All from_utf8_unchecked replaced with checked from_utf8 |
| F-LP2-MED-001 (#[ignore]'d .prx test) | DURABLE | SID-1-compliant citation S-PLUGIN-CI-001 |
| F-LP2-MED-002 (structured error) | DURABLE | AuthPluginDispatchFailed variant + real spec.sensor_id |
| F-LP2-MED-003 (401-retry assertion) | DURABLE | .expect(1) on mock + Bearer wat-fixture-token matcher |
| F-LP2-LOW-001 (Cargo.toml comment) | DURABLE | Comment reconciled |

**Regression count: 0. Paper-fix count: 1 (F-LP2-HIGH-001 wired code but tests only structural surface).**

## Part B — NEW findings

### F-LP3-HIGH-001 — wit-bindgen rework left WIT EXPORTS as hand-rolled core-module ABI; Component Model dispatch surface mismatch [HIGH, HIGH confidence]

**Evidence:**
- `lib.rs:117-123` invokes `wit_bindgen::generate!({ world: "crowdstrike-oauth2", path: "wit/sensor-auth.wit", })` — but only HOST IMPORTS consumed (`prism::crowdstrike_oauth2::host::http_request`, `kv_get`, `kv_set`, `current_time_secs`, `get_config`).
- WIT spec at `wit/sensor-auth.wit:107-110` declares `world crowdstrike-oauth2 { import host; export sensor-auth; }`. With `export sensor-auth`, wit-bindgen generates a `Guest`/`SensorAuth` trait the plugin must `impl` and register via `export!`.
- Grep `impl exports::|Guest|impl Sensor|export!` returns ZERO matches.
- Exports use hand-rolled `#[unsafe(no_mangle)] pub unsafe extern "C" fn acquire_token_export(...)` at lib.rs:393-424, get_token_export at 437-459, auth_type_name_export at 372-378 — manual `((ptr << 32) | len)` u64 ABI.
- `discovery.rs:26` requires Component exports named `auth-type-name`, `acquire-token`, `get-token` (kebab-case). Hand-rolled exports compile to snake_case with `_export` suffix.

**Why it fails:** Built .prx loaded by `PluginRuntime::load_plugin` will fail `validate_wit_interface` (E-PLUGIN-001) because expected kebab-case sensor-auth exports are absent. Only test catching this is `test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime` at line 1265 — `#[ignore]`'d. Pass-2 HIGH-002 "wit-bindgen wired" claim was partial.

**Routing:** implementer — add `impl exports::prism::crowdstrike_oauth2::sensor_auth::Guest for Component` block delegating to existing Rust functions, add `export!(Component)`. Delete or gate the manual `*_export` wrappers under feature flag. Add non-#[ignore] unit test that compiles plugin to wasm32-wasip1 + lifts to component + verifies `SENSOR_AUTH_REQUIRED_EXPORTS` present, OR un-#[ignore] MED-001 test conditional on wasm32-wasip1 toolchain availability.

---

### F-LP3-MED-001 — Step 7.5b iteration logic has no production-path integration test; closure depends on run_boot_sequence which panics at step 7 todo!() [MED, HIGH confidence]

**Evidence:**
- boot.rs:207-272 step 7.5b is only place where validate_auth_plugin_fields called on loaded plugin set AND PluginAuthProvider::new invoked with real sensor_specs.
- boot.rs:276 immediately calls step7_init_storage().await? which is todo!() at line 1276-1280. Any test calling run_boot_sequence panics at step 7 before step 7.5b's side effects observable.
- Test at plugin_boot_tests.rs:1306-1351 only:
  - Calls PluginAuthProvider::new directly via test helpers
  - Calls plugin_load_step (NOT run_boot_sequence) with empty plugin dir
  - Asserts plugin_auth_providers.is_empty() — vacuously true
- Test at line 466 (`test_F_PASS2_CRIT_001`) merely verifies run_boot_sequence is pub via type_name_of_val; does NOT invoke it.
- No test exercises: load real plugin + sensor spec with auth_plugin = "crowdstrike-oauth2" + verify plugin_auth_providers.get("crowdstrike") returns Some(Arc<PluginAuthProvider>).

**Why it fails:** Closure narrative claims step 7.5b's HashMap-population logic is "wired into production." CODE is wired. LOGIC is untested. Future refactor introducing iteration bug, wrong key, or missed sensor_spec.auth_plugin.is_some() check would not be caught. TD-VSDD-059 paper-fix risk — structural visibility test passes (HashMap field exists), behavioral verification absent.

**Routing:** implementer — extract step 7.5b's iteration logic into a pure function `fn validate_and_construct_auth_providers(snapshot: &ConfigSnapshot, runtime: &PluginRuntime) -> Result<HashMap<String, Arc<PluginAuthProvider>>, BootError>`. Add integration tests:
1. Minimal PluginLoadResult with one loaded plugin "crowdstrike-oauth2" via WAT fixture
2. ConfigSnapshot with one sensor spec auth_plugin = Some("crowdstrike-oauth2") — verify map has exactly one entry for configured sensor_id
3. Inverse: auth_plugin = Some("typo") — verify BootError::UnknownAuthPlugin returned
4. Empty: empty sensor_specs — verify empty map

---

### F-LP3-LOW-001 — boot.rs:265 tracing emission lacks event_type field; inconsistent with sibling boot emissions [LOW, MEDIUM confidence]

**Evidence:**
- boot.rs:265-269 `tracing::info!(sensor_id = %sensor_id, plugin_id = %plugin_id, "boot: PluginAuthProvider constructed for sensor (F-LP2-HIGH-001)")` — no event_type field.
- Sibling boot emissions carry event_type: boot.rs:998 "boot.audit.initialized", 1098 "plugin_load_disabled_via_envvar", 1223 "plugin_registration_rolled_back".
- SAP-1 standing probe applies only to emissions with event_type=. Strictly, this is exempt — but inconsistent with surrounding pattern.

**Routing:** implementer — (a) add `event_type = "plugin_auth_provider_constructed"` + register row in BC-2.16.002 §Postconditions catalog (route via PO), OR (b) demote to tracing::debug!() as per-sensor diagnostic noise. Option (a) preferred for consistency with sibling boot emissions.

## CLEAN strict: NO — CLEAN (PR-merge): NO

**Reason:** 1 HIGH (F-LP3-HIGH-001), 1 MED (F-LP3-MED-001), 1 LOW (F-LP3-LOW-001) — 3 findings total.

## Streak advancement: 0/3 → 0/3 (no change)

## Novelty Assessment

MEDIUM-HIGH novelty. F-LP3-HIGH-001 NEW class — pass-2 HIGH-002 closure verified host-import rewire but did NOT verify export-side rewire. Fresh context surfaced asymmetry. F-LP3-MED-001 = recurring paper-fix pattern — closure code real, closure test structural-not-behavioral, like FB-IMPL-1's CRIT-003/HIGH-010 papered over and exposed at pass-2. **Pattern persists across THREE fix bursts.**

## Decay trajectory

Pass-1: 20 findings (4 CRIT, 7 HIGH, 6 MED, 1 LOW, PROCESS-GAP, OBS)
Pass-2: 12 findings (2 CRIT, 5 HIGH, 3 MED, 1 LOW, OBS) — 40% reduction
Pass-3: 3 findings (0 CRIT, 1 HIGH, 1 MED, 1 LOW) — 75% reduction
Trajectory is encouraging — severity AND count both declining. One more fix burst should reach 1/3 streak.

## Total counts

| Severity | Count |
|---|---|
| HIGH | 1 |
| MED | 1 |
| LOW | 1 |
| **TOTAL** | **3** |
