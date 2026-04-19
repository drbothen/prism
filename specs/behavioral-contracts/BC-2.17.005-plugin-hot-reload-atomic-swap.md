---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-17"
capability: "CAP-030"
lifecycle_status: active
introduced: cycle-1
---

# BC-2.17.005: Plugin Hot Reload — Atomic Module Swap, In-Flight Calls Complete Against Old Version

## Description

When a `.prx` plugin file is modified on disk, the `notify` file watcher fires. The
new plugin binary is compiled, validated against the WIT interface, and swapped into
the plugin registry via `arc_swap`. In-flight calls that acquired a reference to the
old `InstancePre<HostState>` via Arc complete normally against the old module version
without interruption. New calls after the swap use the new module. If recompilation
fails, the old plugin remains active. This is INV-PLUGIN-005.

## Preconditions

- `PluginRuntime` is running with a plugin registered under `plugin_id`
- The `notify` file watcher is monitoring `{config_dir}/plugins/*.prx`
- One or more in-flight plugin calls hold an `Arc<LoadedPlugin>` with the old module
- The `.prx` file for the plugin is replaced on disk with a new version

## Postconditions

- **Successful hot reload:**
  - `notify` fires a `Modify` event for the `.prx` file
  - The new `.prx` is loaded via `load_plugin(path)` in `tokio::task::spawn_blocking`
    (compilation is CPU-intensive and must not block the tokio runtime)
  - WIT interface validation is performed on the new binary (BC-2.17.006)
  - On validation success: the plugin registry entry is updated via arc-swap atomically
  - In-flight calls holding the old `Arc<LoadedPlugin>` complete normally using the old
    `InstancePre` (Arc reference count keeps the old module alive until all holders drop)
  - New calls after the swap use the new module
  - An `INFO`-level log: `"Plugin '{plugin_id}' hot-reloaded from '{path}'"`
- **Failed recompilation (invalid new `.prx`):**
  - Compilation or WIT validation fails
  - The plugin registry entry is NOT updated — the old version remains active
  - An `ERROR`-level log: `"Plugin '{plugin_id}' hot-reload failed: {error}. Previous version retained."`
  - In-flight and new calls continue using the old module
- **Plugin file deleted:**
  - The plugin is removed from the registry
  - In-flight calls holding an `Arc<LoadedPlugin>` complete normally
  - New calls return `E-PLUGIN-002: "Plugin '{plugin_id}' is not loaded."`

## Invariants

- INV-PLUGIN-005: Hot reload swaps the compiled WASM module atomically; in-flight calls complete against the old module
- Failed recompilation MUST NOT unload a currently-working plugin (CI-002 hot reload invariant)
- `Component::from_binary` compilation MUST be run in `tokio::task::spawn_blocking` (not on the tokio runtime thread)
- The arc-swap is the ONLY mechanism for registry updates — no global lock on the hot path
- The old `Arc<LoadedPlugin>` is not dropped until all callers release their Arc references

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-PLUGIN-001` | New `.prx` fails WIT validation | Old plugin retained; `ERROR` log; `E-PLUGIN-001` details in log |
| `E-PLUGIN-008` | New `.prx` fails to compile (bad WASM binary) | Old plugin retained; `ERROR` log |
| `E-PLUGIN-002` | Plugin deleted from disk; new call arrives after removal | `Err(PluginError::NotLoaded { plugin_id })` returned to caller |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-17-019 | Plugin file replaced while 50 concurrent in-flight calls are running | All 50 in-flight calls complete using old module; subsequent calls use new module; no errors |
| EC-17-020 | Plugin file replaced 3 times in rapid succession (< 500ms debounce) | `notify` debouncer suppresses duplicate events; only the final version triggers reload |
| EC-17-021 | New `.prx` is valid WASM but wrong WIT interface version | WIT validation fails → `E-PLUGIN-001`; old plugin retained |
| EC-17-022 | Plugin KV store state is preserved across hot reload | `HostState.kv_store` is shared across instances; new module instances see same KV state |
| EC-17-023 | Plugin added to `plugins/` directory for the first time | `notify` fires `Create` event; plugin is loaded and registered; follows same validate-then-register flow |

## Related BCs

- BC-2.17.006 — WIT Validation (hot reload triggers full WIT validation on the new binary)
- BC-2.16.007 — Sensor Spec Hot Reload (same arc-swap + notify pattern applied to plugins)
- BC-2.16.006 — Arc-Swap Config Access (same lock-free pattern)

## Architecture Anchors

- AD-007: arc-swap for hot config reload — same pattern applied to plugin registry
- AD-018: Automatic filesystem watching for config reload
- AD-019: WASM plugins — hot reload
- `specs/architecture/sensor-adapters.md` — plugin registry, arc-swap, notify watcher
- S-1.15 Task 7: `plugin/hot_reload.rs`

## Story Anchor

S-1.15 — prism-spec-engine: WASM Plugin Runtime (AC-6 covers hot reload with in-flight safety)

## VP Anchors

Integration test: `tests/plugin_tests.rs` — "Verify hot reload: drop `.prx` file, reload with modified fixture → verify updated behavior."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-030 |
| Story Invariant | INV-PLUGIN-005 |
| ADR | AD-007, AD-018, AD-019 |
| Story | S-1.15 |
| Priority | P0 |
