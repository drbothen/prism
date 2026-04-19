---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.009: Config Reload Triggers notifications/tools/list_changed

## Preconditions
- The MCP server is running and a client is connected
- A configuration reload occurs (e.g., SIGHUP, config file change detection) that changes the resolved capability set

## Postconditions
- Prism sends an MCP `notifications/tools/list_changed` notification to the AI agent
- The notification causes the agent to re-fetch the `tools/list`, which now reflects the updated capabilities
- Write tools newly enabled (for any client) appear in the updated `tools/list`
- Write tools no longer enabled (for any client) are removed from the updated `tools/list`
- Read tools are unaffected (they are always available)
- There is no session-level "client context switch" concept. The server is stateless. Tool visibility is the union of capabilities across all configured clients; per-call `client_id` determines authorization at invocation time.

## Invariants
- DI-003: Feature flag deny-by-default -- the tools/list reflects deny-by-default resolution across all clients

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Notification delivery failure | MCP transport cannot deliver the notification (e.g., client disconnected) | Warning logged; the stale tool list may cause the agent to attempt now-unavailable tools, which return capability-denied errors |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-016 | Config reload produces the same capability set | `notifications/tools/list_changed` is NOT sent (no-op optimization) |
| EC-06-017 | Config reload adds a new client | Tool list re-evaluated; notification sent if the new client enables previously-hidden write tools |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-003 |
| Addresses | ADV-1-001, ADV-2-003 |
| Priority | P0 |
