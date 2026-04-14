---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Feature Flag System"
capability: "CAP-005"
---

# BC-2.04.005: Hidden Tools Pattern -- Disabled Write Tools Omitted from tools/list

## Preconditions
- An MCP client requests `tools/list`
- The current client context has a resolved set of enabled capabilities

## Postconditions
- Only tools whose capabilities are enabled (both compile-time and runtime) appear in the `tools/list` response
- Disabled write tools are completely absent from the response (not visible to the AI agent)
- Read-only tools always appear regardless of feature flags
- The tool list is re-evaluated and `notifications/tools/list_changed` is sent when the client context switches

## Invariants
- DI-003: Disabled tools are hidden, not visible-but-disabled
- The `tools/list` response is consistent with the capabilities of the active client

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Agent attempts to call a tool that was hidden | MCP protocol returns "unknown tool" error; this is a protocol-level error, not a Prism error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-010 | Client context switches from Client A (write-enabled) to Client B (read-only) | Write tools disappear from `tools/list`; `notifications/tools/list_changed` emitted; agent's next `tools/list` call reflects the new set |
| EC-04-011 | No client context active (session just started) | Only tools that require no client context (e.g., `list_capabilities`, `list_clients`) appear |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |
