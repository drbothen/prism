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
input-hash: "ac6b633"
traces_to: ["CAP-032"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.17.004: Plugin Sandbox — CPU Time Limit Enforced via Epoch Interruption (default 5s)

## Description

Each WASM plugin call is subject to a CPU time limit (default 5 seconds per call,
configurable per plugin via `[plugin_config].timeout_seconds`). The limit is enforced
via `wasmtime`'s epoch interruption mechanism: a background tokio task increments the
engine epoch every 1ms, and each plugin call sets a `Store::epoch_deadline` proportional
to the configured timeout. When the deadline fires, the plugin is terminated and the
call returns `Err(PluginError::Timeout)`. The host process is unaffected. This is
INV-PLUGIN-004.

## Preconditions

- `PluginRuntime` is initialized with a `wasmtime::Engine` configured with
  `Config::epoch_interruption(true)`
- A long-running background tokio task is incrementing the engine epoch every 1ms
  (started once at `PluginRuntime::new()`, not per call)
- A plugin call is in progress
- The plugin's WASM code executes a long-running or infinite loop that does not yield

## Postconditions

- The `Store::epoch_deadline` fires when the configured timeout elapses
- wasmtime traps the plugin execution at the next epoch check point
- The trap is caught at the `instance.call_*` boundary in Rust
- The calling method returns `Err(PluginError::Timeout { plugin_id: String, duration_ms: u64 })`
  where `duration_ms` approximates the elapsed wall time
- A `WARN`-level log entry is emitted:
  `"Plugin '{plugin_id}' timed out after {duration_ms}ms (limit: {timeout_ms}ms)"`
- The plugin instance and its `wasmtime::Store` are dropped
- The background epoch ticker task continues unaffected
- The plugin registry entry is retained

## Invariants

- INV-PLUGIN-004: CPU time limit (default 5s per plugin call) enforced by wasmtime epoch interruption
- The epoch background task is started ONCE at `PluginRuntime` construction, NOT per call
- `Store::epoch_deadline` is set ONCE per plugin call, before the call is made
- The timeout applies to WASM execution time only; host function execution time
  (`host::http_request`) has a separate 10-second per-request timeout (BC-2.17.002)
- WASM plugin actions MUST be spawned on a separate `tokio::task` — plugin CPU time
  limit interruption must not block the trigger evaluation loop (S-4.08 constraint)

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-PLUGIN-007` | Plugin call exceeds configured CPU time limit | `Err(PluginError::Timeout { plugin_id, duration_ms })` returned to caller |
| — | Epoch ticker task panics or terminates unexpectedly | `PluginRuntime` is in degraded state; all plugin calls will eventually not be interrupted; log `ERROR "Plugin epoch ticker terminated"` and attempt restart |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-17-014 | Plugin completes in exactly the timeout duration | Whether it succeeds or fails depends on epoch check granularity (1ms); either outcome is acceptable within 1ms tolerance |
| EC-17-015 | Per-plugin `timeout_seconds = 30` overrides the 5s default | Plugin gets a 30-second CPU time limit |
| EC-17-016 | Test fixture: WAT module with infinite loop (`loop {}`) | Returns `Err(Timeout)` within configured deadline + 1s tolerance; verified by AC-3 |
| EC-17-017 | Plugin call is interrupted mid-`host::http_request` | `http_request` completes (host-side) or its 10s timeout fires; either way the WASM epoch interrupt fires when WASM resumes; `Err(Timeout)` returned |
| EC-17-018 | 20 concurrent plugin calls all timing out simultaneously | All 20 return `Err(Timeout)` independently; epoch ticker continues normally |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-17-004-happy | Plugin call completes within 5s | Success result returned | Baseline |
| TV-17-004-timeout | WAT module with infinite loop; 5s deadline | `Err(PluginError::Timeout)` within deadline + 1s tolerance | EC-17-016 |
| TV-17-004-override | `timeout_seconds = 30`; plugin runs 25s | Succeeds under extended limit | EC-17-015 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| (none) | CPU time limit via epoch interruption is time-dependent; correct implementation is verified by integration test with infinite-loop WAT fixture in tests/plugin_tests.rs AC-3; no provable pure-function invariant | — |

## Related BCs

- BC-2.17.001 — Plugin Panic Isolation (timeout trap handled by the same catch mechanism)
- BC-2.17.003 — Memory Limit Enforcement (orthogonal resource dimension)
- BC-2.17.002 — No Direct Filesystem/Network Access (10s http_request timeout is separate)

## Architecture Anchors

- AD-019: WASM plugins — CPU time limit via epoch interruption
- `specs/architecture/sensor-adapters.md` — epoch interruption, background ticker, per-call deadline
- S-1.15 Task 5: `plugin/sandbox.rs` — `epoch_interruption`, background tokio task, `Store::epoch_deadline`
- S-4.08 Architecture Compliance: "WASM plugin actions MUST NOT be made on the trigger evaluation loop's task"

## Story Anchor

S-1.15 — prism-spec-engine: WASM Plugin Runtime (AC-3 covers infinite loop timeout)

## VP Anchors

Integration test: `tests/plugin_tests.rs` — "Simulate plugin timeout (WAT module with infinite loop) → verify `Err(Timeout)` within configured deadline, host process continues."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-032 |
| Story Invariant | INV-PLUGIN-004 |
| ADR | AD-019 |
| Story | S-1.15 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (MARK-NONE); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
