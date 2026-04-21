---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.17.005
input-hash: "86bd0c600317884c06c03f8aefb26392"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.17.005
module: prism-spec-engine
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-2-patch
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-042: Plugin Hot Reload — Failed Compile Retains Old InstancePre

## Property Statement

Given a `PluginRegistry` with a valid plugin registered under `plugin_id`, invoking
`hot_reload(plugin_id, invalid_bytes)` where compilation of `invalid_bytes` fails
leaves the registry entry unchanged: the old `Arc<LoadedPlugin>` is still returned
for `plugin_id` after the failed reload attempt. The registry never transitions to
a partially-loaded or empty state for that plugin.

## Source Contract

- **Anchor Story:** `S-1.15`
- **Source BC:** BC-2.17.005 — Plugin Hot Reload Atomic Swap
- **Module:** prism-spec-engine
- **Category:** Correctness / Availability

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — random valid plugin bytes + arbitrary invalid bytes | All compilation-failure scenarios: bad WAT, wrong type, missing exports |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_spec_engine::plugin::PluginRegistry::hot_reload
//
// Pattern mirrors VP-032 (ConfigManager::reload) using arc-swap:
//
// proptest!(|(valid_plugin in arb_valid_plugin_bytes(), invalid_bytes in arb_bytes())| {
//     let registry = PluginRegistry::new();
//     let plugin_id = PluginId::new("test-plugin");
//
//     // Load valid plugin initially
//     registry.register(plugin_id.clone(), &valid_plugin).unwrap();
//     let old_ptr = registry.get(&plugin_id).unwrap();
//
//     // Attempt reload with bytes that will fail compilation
//     // (use bytes that are not valid WASM or are missing required WIT exports)
//     let reload_result = registry.hot_reload(plugin_id.clone(), &invalid_bytes);
//
//     // If compilation failed, old plugin must still be present unchanged
//     if reload_result.is_err() {
//         let current_ptr = registry.get(&plugin_id).unwrap();
//         prop_assert!(Arc::ptr_eq(&old_ptr, &current_ptr),
//             "failed reload must retain old plugin Arc");
//     }
// });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Proptest generates valid plugin bytes + invalid bytes |
| Tool support? | Full | proptest + arc-swap; identical pattern to VP-032 |
| Execution time budget | <120 seconds | Compilation of invalid bytes is fast (early rejection) |
| Assumptions required | Test uses a MockCompiler oracle so invalid bytes are deterministically rejected without wasmtime execution | Avoids non-determinism from async compile |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.17.005. Structurally identical to VP-032 arc-swap pattern targeting PluginRegistry. In-flight disruption half remains integration test. |
