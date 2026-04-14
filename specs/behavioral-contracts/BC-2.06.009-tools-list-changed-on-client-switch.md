---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Client Configuration"
capability: "CAP-009"
---

# BC-2.06.009: Client Context Switch Triggers notifications/tools/list_changed

## Preconditions
- An analyst switches client context (changes the active `client_id` for subsequent operations)
- The new client has a different resolved capability set than the previous client

## Postconditions
- Prism sends an MCP `notifications/tools/list_changed` notification to the AI agent
- The notification causes the agent to re-fetch the `tools/list`, which now reflects the new client's capabilities
- Write tools available for the previous client but not the new client are omitted from the updated `tools/list`
- Write tools available for the new client but not the previous client are added to the updated `tools/list`
- Read tools are unaffected by the context switch (they are always available)

## Invariants
- DI-003: Feature flag deny-by-default -- the tools/list reflects the deny-by-default resolution for the active client

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Notification delivery failure | MCP transport cannot deliver the notification (e.g., client disconnected) | Warning logged; the stale tool list may cause the agent to attempt now-unavailable tools, which return capability-denied errors |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-016 | Client switch to a client with identical capabilities | `notifications/tools/list_changed` is still sent (Prism does not diff capability sets to suppress redundant notifications) |
| EC-06-017 | Cross-client query (`client_id: null`) does not constitute a "context switch" | No notification is sent; the cross-client query uses the union of read tools; write operations in cross-client mode are not supported |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-003 |
| Priority | P0 |
