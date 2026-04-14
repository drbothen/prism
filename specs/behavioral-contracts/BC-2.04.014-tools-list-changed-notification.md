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

# BC-2.04.014: notifications/tools/list_changed on Client Context Switch

## Preconditions
- An analyst switches client context mid-session (e.g., from Client A to Client B)
- Client A and Client B have different enabled capabilities

## Postconditions
- Feature flags are re-evaluated for the new client context
- The MCP tool registry is rebuilt based on Client B's capabilities
- A `notifications/tools/list_changed` notification is sent to the MCP client (Claude Code)
- The MCP client re-fetches `tools/list` and sees Client B's available tools
- The transition is logged in the audit trail with both old and new client contexts

## Invariants
- The tool list always reflects the active client's capabilities
- `notifications/tools/list_changed` is only sent when the tool set actually changes

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | MCP client does not support `notifications/tools/list_changed` | Notification sent but ignored by client; tool list may be stale; no Prism-side error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-029 | Client A and Client B have identical capabilities | Tool list does not change; `notifications/tools/list_changed` is NOT sent (no-op optimization) |
| EC-04-030 | Rapid context switches (A -> B -> A in quick succession) | Each switch triggers re-evaluation; final state reflects the last active client |
| EC-04-031 | Context switch to `client_id: null` (cross-client mode) | Tool list reflects the union of capabilities? No -- uses the most restrictive set (deny-by-default); only tools available for ALL clients appear |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |
