---
document_type: behavioral-contract
level: L3
version: "1.1"
status: removed
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "MCP Server & Transport"
capability: [CAP-005, CAP-009]
---

# BC-2.10.005: notifications/tools/list_changed on Config Reload

## Preconditions
- The MCP client has connected and received an initial `tools/list`
- A configuration reload is triggered (e.g., SIGHUP, config file watch, or explicit reload command)

## Postconditions
- Prism re-evaluates capabilities from the updated TOML configuration
- If the resolved set of available tools changes (write tools added or removed), Prism sends a `notifications/tools/list_changed` notification to the MCP client
- The MCP client re-fetches `tools/list` and sees the updated set of tools
- If the tool set does not change after reload, no notification is sent (idempotent)
- There is no "client context switch" trigger. The server is stateless with respect to client context. Per-call `client_id` determines capability resolution at invocation time, not at `tools/list` time.

## Invariants
- DI-003: Feature flag deny-by-default -- tool set reflects resolved capabilities across all clients
- Notifications triggered only by config reload, not by per-call client_id routing

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | Notification delivery failure (MCP transport issue) | Best-effort; if notification fails, the client may have stale tool list until next `tools/list` request |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-008 | Config reload removes a client that had unique write capabilities | Write tools that are no longer enabled for any client disappear from `tools/list`; notification sent |
| EC-10-009 | Rapid config reloads in quick succession | Each reload evaluates capabilities; notification sent only when the tool set actually changes |
| DEC-006 | Config file changes on disk without explicit reload signal | Prism does not auto-detect file changes unless a file watcher is configured; the running session uses the last-loaded config |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005, CAP-009 |
| L2 Invariants | DI-003 |
| L2 Edge Cases | DEC-006 |
| Addresses | ADV-1-001, ADV-2-003 |
| Priority | P0 |
