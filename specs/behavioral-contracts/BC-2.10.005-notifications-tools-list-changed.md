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
capability: "CAP-005, CAP-009"
---

# BC-2.10.005: notifications/tools/list_changed on Client Context Switch

## Preconditions
- The MCP client has connected and received an initial `tools/list`
- The analyst switches to operating on a different client (different `client_id` in subsequent tool calls)
- The new client has a different capability set than the previous client

## Postconditions
- Prism detects that the available tool set has changed based on the new client's capabilities
- Prism sends a `notifications/tools/list_changed` notification to the MCP client
- The MCP client re-fetches `tools/list` and sees the updated set of tools (write tools may appear or disappear)
- If the new client has the same capability set as the previous client, no notification is sent (idempotent)

## Invariants
- DI-003: Feature flag deny-by-default -- tool set reflects the current client's resolved capabilities

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | Notification delivery failure (MCP transport issue) | Best-effort; if notification fails, the client may have stale tool list until next `tools/list` request |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-008 | Cross-client query (client_id: null) followed by single-client query | Tool set for cross-client mode includes only read tools; switching to a write-enabled client triggers notification |
| EC-10-009 | Rapid client switching (multiple different clients in quick succession) | Each switch evaluates capabilities; notification sent only when the tool set actually changes |
| DEC-006 | Config changed externally while session is active | No notification until restart; running session uses startup-time capabilities |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005, CAP-009 |
| L2 Invariants | DI-003 |
| L2 Edge Cases | DEC-006 |
| Priority | P0 |
