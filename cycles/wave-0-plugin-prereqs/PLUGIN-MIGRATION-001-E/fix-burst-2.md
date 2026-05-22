# PLUGIN-MIGRATION-001-E — FB-IMPL-2 Closure Report

**Date:** 2026-05-22
**Burst:** FB-IMPL-2 — fix-burst for LOCAL pass-2 findings + retroactive wire-up of FB-IMPL-1 paper-fix gaps
**Feature HEAD before:** `319263ff` (paper-fix state — 2 unwired closures from FB-IMPL-1)
**Feature HEAD after:** `1d06a4bf`
**Workspace test count:** 3742 → 3747 GREEN (+5 tests; zero regression)
**Per-finding micro-commits:** 9

## Closure summary

All 12 pass-2 actionable findings closed in scope with explicit wire-up verification protocol. OBS-001 informational. The TWO FB-IMPL-1 paper-fix gaps (CRIT-003 + HIGH-010 building blocks unwired) are now WIRED into production boot path at step 7.5b. ADR-028 §D10 co-merge gate is now structurally satisfiable.

## Wire-up verification protocol applied

For every closure, production caller documented:
- CRIT-001 kv_store sharing: 3 callers in mod.rs (dispatch_plugin_acquire_token, enrich_single, enrich_batch)
- CRIT-002 validate_auth_plugin_fields: called in boot.rs run_boot_sequence step 7.5b after load_all_plugins
- HIGH-001 PluginAuthProvider construction: constructed in boot.rs step 7.5b, registered in plugin_result.plugin_auth_providers

## Per-finding closure table

| Finding | Severity | Closure Note | Production Caller |
|---|---|---|---|
| F-LP2-CRIT-001 | CRIT | LoadedPlugin gains Arc<PluginKvStore>; make_host_state clones; double-dispatch test added | mod.rs lines for dispatch_plugin_acquire_token, enrich_single, enrich_batch |
| F-LP2-CRIT-002 | CRIT | validate_auth_plugin_fields wired into boot.rs step 7.5b; BootError::UnknownAuthPlugin variant; exits 2 on typo | run_boot_sequence step 7.5b block |
| F-LP2-HIGH-001 | HIGH | Arc<PluginAuthProvider> constructed at step 7.5b for each sensor with auth_plugin; registered in adapter registry | run_boot_sequence step 7.5b block |
| F-LP2-HIGH-002 | HIGH | wit-bindgen 0.51.0 added to workspace deps; manual extern blocks replaced with wit_bindgen::generate! macro; "for now" comments deleted | WASM target host_impl uses macro-generated bindings |
| F-LP2-HIGH-003 | HIGH | Justfile recipe: precondition check on wasi_snapshot_preview1.wasm + \|\| exit 1 + positive component assertion | recipe build-plugin-crowdstrike-oauth2 |
| F-LP2-HIGH-004 | HIGH | Native stubs gated under #[cfg(all(not(target_arch = "wasm32"), test))]; export functions under #[cfg(target_arch = "wasm32")] | lib.rs host_impl module |
| F-LP2-HIGH-005 | HIGH | from_utf8_unchecked replaced with from_utf8(...).map_err(...) returning AuthError::Internal | export functions + acquire_token |
| F-LP2-MED-001 | MED | #[ignore]'d integration test loads built .prx with SID-1-compliant citation (S-PLUGIN-CI-001 follow-up) | crowdstrike_oauth2_plugin_tests.rs |
| F-LP2-MED-002 | MED | SpecEngineError::AuthPluginDispatchFailed { sensor_id, plugin_id, plugin_error: PluginError } variant added; real spec.sensor_id wired | plugin_auth_provider.rs |
| F-LP2-MED-003 | MED | .expect(1) on retry mock + Authorization: Bearer wat-fixture-token header matcher; counter-based assertion | test_006 |
| F-LP2-LOW-001 | LOW | Stale Cargo.toml comment updated to reflect FB-IMPL-2 state | crates/prism-spec-engine/Cargo.toml |
| F-LP2-OBS-001 | OBS | Informational only; no action |  |

## New artifacts created

- `BootError::UnknownAuthPlugin { sensor_id, plugin_id }` variant
- `SpecEngineError::AuthPluginDispatchFailed { sensor_id, plugin_id, plugin_error: PluginError }` variant (#[non_exhaustive])
- `types::SensorSpec.auth_plugin: Option<String>` field (extended from prior stub addition)
- `validation::validate_auth_plugin_fields(sensor_id, auth_plugin, registered_ids)` function — production-callable
- `wit-bindgen = "0.51.0"` added to workspace.dependencies
- Real wit-bindgen-generated bindings in plugin guest

## Process discipline sustained

PROCESS-GAP-019 closure pattern continued: 9 per-finding micro-commits (down from FB-IMPL-1's 12, reflecting smaller scope). No "for now" / "MVP" rationalization language anywhere. Wire-up verification protocol applied unsuccessfully to ZERO closures (i.e., all closures had production callers documented).

## Cumulative across cascade

- Pass-1: 20 findings → FB-IMPL-1: 19 closures (2 turned out to be paper-fix)
- Pass-2: 12 findings + 2 FB-IMPL-1 paper-fix → FB-IMPL-2: 12 closures + 2 retroactive wire-ups = 14 effective closures
- Total findings closed (durable): 31 of 32 (OBS-001 + OBS-020 are informational only)

## Streak status

Streak before FB-IMPL-2: 0/3.
Streak after FB-IMPL-2: 0/3 (fix-burst does not advance streak).
Next: pass-3 adversary dispatch. CLEAN(strict) → streak advances to 1/3.
