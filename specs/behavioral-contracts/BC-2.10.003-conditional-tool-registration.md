---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "MCP Server & Transport"
capability: "CAP-005"
---

# BC-2.10.003: Conditional Tool Registration (Feature-Flag Gated)

## Preconditions
- Two-tier feature flag system is active: compile-time cargo features + runtime per-client TOML config
- Write tools (containment, alert acknowledgment, device actions) are defined behind `#[cfg(feature = "...")]` gates

## Postconditions
- If a cargo feature is absent at compile time, the corresponding write tools do not exist in the binary at all
- If a cargo feature is present but the runtime TOML config denies the capability for ALL configured clients, the tool is excluded from `tools/list`
- The hidden tools pattern: write tools disabled for all clients are completely omitted from `tools/list`, not listed as unavailable. Write tools enabled for at least one client are shown; per-call `client_id` determines authorization at invocation time.
- The server is stateless with respect to client context. There is no "client context switch". `notifications/tools/list_changed` is sent at server startup only (when the initial tool set is computed from config). There is no hot-reload and no session-level context switch that would trigger a subsequent `notifications/tools/list_changed`.
- `confirm_action` is hidden from `tools/list` if and only if no write tools are registered (all write capabilities disabled for all clients). If at least one write tool is registered, `confirm_action` is visible.
- The `list_capabilities` meta-tool (BC-2.10.011) reveals the full capability matrix regardless of what is currently registered

## Invariants
- DI-003: Feature flag deny-by-default -- both tiers must permit the operation

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Structured error | Agent calls a write tool that is runtime-disabled for the current client | Tool not found in `tools/list`; if agent somehow invokes it directly, returns `PrismError::Permission` with `suggestion: "Write capability '{path}' is not enabled for client '{id}'. Enable in TOML config at clients.{id}.capabilities.{path}"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-006 | Config changed after startup to enable a capability | Running session still uses startup-time capabilities unless config reload is triggered |
| EC-10-005 | All write features disabled at compile time | Zero write tools in binary; `list_capabilities` shows all write capabilities as "compile-time disabled" |
| EC-10-006 | Client A has containment enabled, Client B does not | Containment tool appears in `tools/list` (enabled for at least one client). Invocation with `client_id: "a"` succeeds. Invocation with `client_id: "b"` returns `E-FLAG-001`. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| L2 Edge Cases | DEC-006 |
| Priority | P0 |
