---
document_type: behavioral-contract
level: L3
version: "1.2"
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
input-hash: "3ff257e"
traces_to: ["CAP-032"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.17.001: Plugin Panic Isolation — Crashed Plugin Does Not Terminate Host Process

## Description

When a WASM plugin executes an instruction that causes a trap (e.g., `unreachable`,
out-of-bounds memory access, or any other fatal WASM error), the `wasmtime` trap is
caught at the host boundary and converted to a structured `Err(PluginError::Trapped)`
return value. The Prism host process continues executing normally. No unwind, no crash,
no process termination. This is INV-PLUGIN-001.

## Preconditions

- `PluginRuntime` is initialized and the plugin (identified by `plugin_id`) is loaded
  in the registry (`LoadedPlugin` with a compiled `InstancePre<HostState>`)
- A call to a plugin method is made (`enrich_single`, `enrich_batch`, `fire_alert`, etc.)
- The plugin's WASM code executes a trap instruction (`unreachable`, memory fault, etc.)

## Postconditions

- The trapping WASM instance is dropped immediately
- The `wasmtime::Trap` is caught at the Rust call boundary (wrapping `instance.call_*`)
- The calling method returns `Err(PluginError::Trapped { plugin_id: String, message: String })`
  where `message` is the WASM trap description from wasmtime
- A `WARN`-level log entry is emitted: `"Plugin '{plugin_id}' trapped: {message}"`
- The Prism host process continues executing: other plugin calls, sensor queries, and
  MCP tool invocations are unaffected
- The plugin registry entry for the trapped plugin is retained — the plugin is NOT
  unregistered; subsequent calls may succeed (transient traps)

## Invariants

- INV-PLUGIN-001: A plugin panic or trap MUST NOT terminate the host process
- The `wasmtime::Store` is created fresh per plugin call — state from a trapped instance
  cannot contaminate subsequent calls
- The host tokio runtime thread is never unwound by a plugin trap

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-PLUGIN-004` | Plugin trap caught at Rust boundary | `Err(PluginError::Trapped { plugin_id, message })` returned to caller; host continues |
| — | Trap message contains null bytes or non-UTF-8 | Message is sanitized with `to_string_lossy()` before inclusion in the error |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-17-001 | Plugin traps on first call after loading | `Err(Trapped)` returned; plugin stays registered; next call creates fresh Store and may succeed |
| EC-17-002 | Plugin traps on every call (persistent bug) | Each call returns `Err(Trapped)`; the caller (infusion pipeline, action engine) is responsible for circuit-breaking |
| EC-17-003 | Plugin traps inside a batch call (`enrich_batch` with 500 inputs) | The entire batch returns `Err(Trapped)`; no partial results; caller retries or degrades gracefully |
| EC-17-004 | Two plugins trap concurrently in different tokio tasks | Both tasks receive independent `Err(Trapped)`; neither affects the other; host process unaffected |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-17-001-happy | Call plugin method; WASM executes `unreachable` | `Err(PluginError::Trapped { plugin_id, message })` | Happy path: trap caught |
| TV-17-001-edge | Two concurrent plugin traps | Both tasks independently return `Err(Trapped)`; host unaffected | EC-17-004 |
| TV-17-001-batch | `enrich_batch` with 500 inputs; plugin traps | Entire batch returns `Err(Trapped)`; no partial results | EC-17-003 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| (none) | INV-PLUGIN-001 enforced by wasmtime host-boundary construction; integration test in tests/plugin_tests.rs; no pure-function invariant to prove formally | — |

## Related BCs

- BC-2.17.004 — CPU Time Limit Enforcement (timeout is a separate termination path from trap)
- BC-2.17.006 — WIT Validation Before Registration (invalid plugins rejected before they can trap)
- BC-2.16.004 — Rust Escape Hatch (alternative to plugins for cases requiring native reliability)

## Architecture Anchors

- AD-019: WASM plugins for custom sensor adapters and infusions
- `specs/architecture/sensor-adapters.md` — PluginRuntime, trap isolation, per-call Store creation
- S-1.15 Task 5: `plugin/sandbox.rs` — wrap all `instance.call_*` in trap catch

## Story Anchor

S-1.15 — prism-spec-engine: WASM Plugin Runtime (AC-2 covers this behavior)

## VP Anchors

Test fixture: `tests/fixtures/trap_plugin.wat` — a minimal WAT module that executes `unreachable`.
Integration test: `tests/plugin_tests.rs` — "Simulate plugin trap → verify Err(Trapped) returned, host process continues."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-032 |
| Story Invariant | INV-PLUGIN-001 |
| ADR | AD-019 |
| Story | S-1.15 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (MARK-NONE); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
