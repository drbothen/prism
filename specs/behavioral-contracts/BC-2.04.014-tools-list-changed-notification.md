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

# BC-2.04.014: notifications/tools/list_changed on Config Reload or Server Startup

## Preconditions
- The MCP client has connected and received an initial `tools/list`
- A configuration reload is triggered (e.g., SIGHUP, config file change detection) or the server has just completed startup

## Postconditions
- On config reload: feature flags are re-evaluated from the updated TOML configuration
- If the resolved set of available tools changes (write tools added or removed due to capability changes), a `notifications/tools/list_changed` notification is sent to the MCP client
- The MCP client re-fetches `tools/list` and sees the updated set of tools
- If the tool set does not change after config reload, no notification is sent (no-op optimization)
- On server startup: the initial `tools/list` reflects the startup-time configuration; no notification is needed (the client fetches `tools/list` as part of initialization)
- There is no session-level "client context switch" concept. The server is stateless. Notifications are triggered only by configuration changes, not by the `client_id` used in individual tool calls.

## Invariants
- `notifications/tools/list_changed` is only sent when the tool set actually changes
- The notification is triggered by config changes, not by per-call client_id routing

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | MCP client does not support `notifications/tools/list_changed` | Notification sent but ignored by client; tool list may be stale; no Prism-side error |
| N/A | Notification delivery failure (MCP transport issue) | Best-effort; client may have stale tool list until next `tools/list` request |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-029 | Config reload produces the same capability set | Tool list does not change; `notifications/tools/list_changed` is NOT sent |
| EC-04-030 | Rapid config reloads in quick succession | Each reload triggers re-evaluation; final notification reflects the last resolved state; intermediate states may be skipped if reloads coalesce |
| EC-04-035 | Config reload adds a new client with write capabilities | Write tools newly enabled (across all clients) appear in the next `tools/list`; notification sent |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Addresses | ADV-1-001, ADV-2-003 |
| Priority | P0 |
