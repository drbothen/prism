---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 3-patch
origin: greenfield
subsystem: "WASM Plugin Runtime"
capability: "CAP-032"
lifecycle_status: active
---

# BC-2.17.003: Plugin Sandbox — Memory Limit Enforced Per Plugin Instance (default 64MB)

## Description

Each WASM plugin instance is constrained to a maximum memory allocation. The default
limit is 64MB per plugin instance, configurable per plugin via
`[plugin_config].memory_limit_mb`. When a plugin attempts to allocate memory beyond its
configured limit, the allocation fails with a WASM trap, which is caught at the host
boundary and returned as `Err(PluginError::MemoryExceeded)`. The host process is
unaffected. This is INV-PLUGIN-003.

## Preconditions

- `PluginRuntime` is initialized with a `wasmtime::Engine` configured with epoch
  interruption and `StoreLimits`
- A plugin call is in progress (plugin instance executing within a `wasmtime::Store`)
- The plugin's WASM code attempts to allocate memory beyond the configured limit
  (default 64MB, configurable via `[plugin_config].memory_limit_mb`)

## Postconditions

- The wasmtime `StoreLimits` memory guard fires, causing a WASM trap
- The trap is caught at the `instance.call_*` boundary in Rust
- The calling method returns `Err(PluginError::MemoryExceeded { plugin_id: String, limit_mb: u64 })`
- A `WARN`-level log entry is emitted: `"Plugin '{plugin_id}' exceeded memory limit of {limit_mb}MB"`
- The plugin instance (and its `wasmtime::Store`) is dropped immediately
- The host process memory is not impacted beyond the plugin's allocated (then released) WASM linear memory
- The plugin registry entry is retained (the plugin is not unregistered)

## Invariants

- INV-PLUGIN-003: Each plugin instance has a memory limit (default 64MB) enforced by wasmtime
- Memory limits are enforced PER INSTANCE, not globally across all plugins
- The limit applies to WASM linear memory; host-side allocations (KV store, HTTP response buffers) are separate
- `wasmtime::Store` is created fresh per plugin call — memory from a terminated instance is released

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-PLUGIN-006` | Plugin exceeds memory limit | `Err(PluginError::MemoryExceeded { plugin_id, limit_mb })` returned to caller |
| — | `memory_limit_mb` config value is 0 or exceeds system limits | Rejected at `PluginRuntime` construction with configuration error |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-17-009 | Plugin allocated exactly at the limit (64MB) | Allocation succeeds; WASM execution continues |
| EC-17-010 | Plugin attempts 1 byte over the limit | Allocation fails with trap → `Err(MemoryExceeded)` |
| EC-17-011 | Per-plugin `memory_limit_mb = 128` overrides the 64MB default | Plugin gets 128MB limit for this plugin's instances only |
| EC-17-012 | Test fixture: WAT module that calls `memory.grow` beyond limit | Returns `Err(MemoryExceeded)` within the configured limit; verified in `plugin_tests.rs` |
| EC-17-013 | 10 concurrent plugin calls each reaching 63MB | All 10 succeed independently; no interaction; host process sees ~630MB peak from WASM linear memory |

## Related BCs

- BC-2.17.001 — Plugin Panic Isolation (memory OOM trap is handled by the same trap catch)
- BC-2.17.004 — CPU Time Limit Enforcement (orthogonal resource dimension)
- BC-2.17.002 — No Direct Filesystem/Network Access (orthogonal sandbox dimension)

## Architecture Anchors

- AD-019: WASM plugins — sandbox constraints
- `specs/architecture/sensor-adapters.md` — memory limits, StoreLimits, wasmtime configuration
- S-1.15 Task 5: `plugin/sandbox.rs` — `wasmtime::StoreLimits` with `memory_size: 64 * 1024 * 1024`

## Story Anchor

S-1.15 — prism-spec-engine: WASM Plugin Runtime (AC-3 covers CPU; memory limit is tested in `plugin_tests.rs`)

## VP Anchors

Integration test: `tests/plugin_tests.rs` — "Verify memory limit: WAT module that allocates > 64MB → verify `Err(MemoryExceeded)`."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-032 |
| Story Invariant | INV-PLUGIN-003 |
| ADR | AD-019 |
| Story | S-1.15 |
| Priority | P0 |
