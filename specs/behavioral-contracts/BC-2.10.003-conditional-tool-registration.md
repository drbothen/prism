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
- If a cargo feature is present but the runtime TOML config denies the capability for the current client, the tool is excluded from `tools/list` for that client context
- The hidden tools pattern: disabled write tools are completely omitted from `tools/list`, not listed as unavailable
- When the client context switches (different `client_id`), tool registration is re-evaluated and `notifications/tools/list_changed` is sent if the available tool set changes (BC-2.10.006)
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
| DEC-006 | Config changed after startup to enable a capability | Running session still uses startup-time capabilities; restart required |
| EC-10-005 | All write features disabled at compile time | Zero write tools in binary; `list_capabilities` shows all write capabilities as "compile-time disabled" |
| EC-10-006 | Client A has containment enabled, Client B does not | When querying Client A, containment tool appears in `tools/list`; when switching to Client B, it disappears and `notifications/tools/list_changed` fires |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| L2 Edge Cases | DEC-006 |
| Priority | P0 |
