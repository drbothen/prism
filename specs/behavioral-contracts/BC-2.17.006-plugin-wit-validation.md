---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-17"
capability: "CAP-032"
lifecycle_status: active
introduced: cycle-1
modified: 2026-04-20
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "1e29f9d"
traces_to: ["CAP-032"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.17.006: WIT Interface Validation Before Plugin Registration

## Description

Before any WASM plugin binary is added to the plugin registry, its WIT interface
compatibility is validated. The binary must export the expected function names for
one of the three recognized Prism plugin types: `prism:sensor-plugin`,
`prism:infusion-plugin`, or `prism:action-plugin`. Required exports include `name`,
`version`, and the primary dispatch function for the plugin type. Plugins that do not
implement a recognized Prism WIT interface are rejected with error `E-PLUGIN-001`
and are not registered. Incompatible plugins are logged and skipped without blocking
other plugins from loading. This is INV-PLUGIN-006.

## Preconditions

- A `.prx` file is being loaded by `PluginRuntime::load_plugin(path)`
  (either at startup discovery or during hot reload)
- The WASM Component binary is compiled successfully from the `.prx` file bytes

## Postconditions

- **Valid plugin (recognized WIT interface):**
  - The exported functions `name()`, `version()`, and the primary dispatch function
    are present on the component
  - A `LoadedPlugin` struct is created with the compiled `Component` and `InstancePre`
  - The plugin is added to the `PluginRuntime.registry` under its declared `plugin_id`
  - An `INFO`-level log: `"Plugin '{plugin_id}' loaded from '{path}' (type: {plugin_type})"`
- **Invalid plugin (unrecognized WIT interface):**
  - The component is missing one or more required exports (`name`, `version`, or
    the primary dispatch function)
  - The plugin is NOT added to the registry
  - An `ERROR`-level log: `"Plugin '{path}' rejected: does not implement a recognized Prism WIT interface"`
  - Error returned: `E-PLUGIN-001` with message:
    `"Plugin '{path}' does not implement a recognized Prism WIT interface. Expected one of: prism:sensor-plugin, prism:infusion-plugin, prism:action-plugin."`
  - Other plugins in the directory continue loading (this plugin's failure is isolated)

## Invariants

- INV-PLUGIN-006: Plugin WIT interface compatibility is validated before registration — incompatible plugins are rejected with a clear error
- WIT validation occurs on EVERY load: startup discovery AND hot reload
- A plugin that passes WIT validation may still trap at runtime (WIT validates interface shape, not behavior)
- `plugin_id` is extracted from the plugin's `name()` export after WIT validation passes
- Duplicate `plugin_id` values: if two `.prx` files export the same `plugin_id`, the second
  load logs a `WARN` and the first-registered plugin is retained (first-wins)

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-PLUGIN-001` | Plugin binary missing required WIT exports | Plugin not registered; error logged; other plugins unaffected |
| `E-PLUGIN-008` | Plugin binary is not valid WASM Component Model binary | `Component::from_binary` fails; `E-PLUGIN-008` logged; plugin not registered |
| — | Plugin binary is > 50MB | Rejected at load time with `E-PLUGIN-009: "Plugin binary exceeds maximum size of 50MB"` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-17-024 | Plugin exports `name` and `version` but not the dispatch function | Rejected with `E-PLUGIN-001`; specific missing export named in error message |
| EC-17-025 | Plugin implements correct WIT interface but wrong version (`prism:infusion-plugin@0.2.0` vs expected `@0.1.0`) | WIT version mismatch detected; plugin rejected with `E-PLUGIN-001` noting version incompatibility |
| EC-17-026 | Directory scan finds 10 `.prx` files, 2 invalid | 8 valid plugins registered; 2 invalid logged with `E-PLUGIN-001`; startup continues normally |
| EC-17-027 | Plugin passes validation but `name()` returns an empty string | Rejected: `plugin_id` cannot be empty; logged as `E-PLUGIN-010`; plugin not registered |
| EC-17-028 | Valid sensor plugin `.prx` but used in an infusion context | No runtime error from WIT validation; the infusion engine attempting to call `enrich_single` on a sensor plugin will trap (dispatch function name mismatch) → `Err(Trapped)` |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-17-006-happy | Valid infusion plugin `.prx` with `name`, `version`, `enrich_single` exports | Plugin registered; INFO log | AC-1 |
| TV-17-006-missing | Plugin missing dispatch function export | `E-PLUGIN-001`; plugin not registered | EC-17-024 |
| TV-17-006-version | Plugin with `prism:infusion-plugin@0.2.0` (wrong version) | `E-PLUGIN-001` with version incompatibility note | EC-17-025 |
| TV-17-006-bulk | 10 plugins scanned; 2 invalid | 8 registered; 2 logged as `E-PLUGIN-001` | EC-17-026 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-043 | For any WASM Component with a strict subset of the required Prism WIT exports, `validate_wit_interface()` returns `Err(PluginError::InvalidInterface)` with a message naming the missing export; for a component with all required exports, returns `Ok(plugin_type)` | Proptest |

## Related BCs

- BC-2.17.001 — Plugin Panic Isolation (runtime trap at dispatch, separate from WIT validation)
- BC-2.17.005 — Hot Reload Atomic Swap (WIT validation runs on every hot reload)

## Architecture Anchors

- AD-019: WASM plugins — WIT interface contract
- `specs/architecture/sensor-adapters.md` — WIT files, plugin discovery, validation
- S-1.15 Task 6: `plugin/discovery.rs` — WIT validation, `E-PLUGIN-001` error
- S-1.15 Task 2: `plugin/loader.rs` — `load_plugin` function, validation step

## Story Anchor

S-1.15 — prism-spec-engine: WASM Plugin Runtime (AC-1 and AC-7 cover this behavior)

## VP Anchors

Integration test: `tests/plugin_tests.rs` — "Load a minimal valid infusion `.prx` fixture → verify registered in runtime" (AC-1) and "Given a `.prx` file that does not export `name`, `version`, or expected dispatch functions → verify `Err` with `E-PLUGIN-001`" (AC-7).

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-032 |
| Story Invariant | INV-PLUGIN-006 |
| ADR | AD-019 |
| Story | S-1.15 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-043); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
