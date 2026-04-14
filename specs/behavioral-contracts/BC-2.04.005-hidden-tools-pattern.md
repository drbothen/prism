---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Feature Flag System"
capability: "CAP-005"
---

# BC-2.04.005: Hidden Tools Pattern -- Stateless Tool List Based on Configured Capabilities

## Preconditions
- An MCP client requests `tools/list`
- The server has resolved capabilities for all configured clients

## Postconditions
- Read-only tools always appear in the `tools/list` response regardless of client or feature flags. The tool list shows the union of all read tools across all configured clients.
- Write tools are shown based on per-call `client_id` resolution at invocation time, not pre-filtered by any session-level client context. At `tools/list` time, write tools are included if ANY configured client has the capability enabled.
- When a write tool is invoked, the `client_id` parameter determines whether the caller has the required capability. If the capability is denied for that client, a structured error is returned (not "unknown tool").
- There is no session-level "active client" concept. The server is stateless with respect to client context.
- Disabled write tools (disabled for ALL clients) are completely absent from the response (not visible to the AI agent)

## Invariants
- DI-003: Disabled tools are hidden, not visible-but-disabled
- Tool visibility is stateless: the `tools/list` response is the same regardless of prior tool calls

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Flag` | Agent invokes a write tool with a `client_id` that lacks the required capability | Structured error: `code: "E-FLAG-001"`, with the denied capability path and suggestion |
| N/A | Agent invokes a tool hidden from `tools/list` (disabled for all clients) | MCP protocol returns "unknown tool" error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-010 | Write tool enabled for Client A but not Client B | Tool appears in `tools/list`. Invocation with `client_id: "a"` succeeds capability check. Invocation with `client_id: "b"` returns `E-FLAG-001`. |
| EC-04-011 | No clients have any write capabilities enabled | Only read tools appear in `tools/list`; all write tools are hidden |
| EC-04-033 | Write tool invoked with `client_id: null` (cross-client) | Write operations with `client_id: null` are not supported; returns `E-FLAG-006` structured error |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Addresses | ADV-1-001, ADV-2-003, ADV-2-004 |
| Priority | P0 |
