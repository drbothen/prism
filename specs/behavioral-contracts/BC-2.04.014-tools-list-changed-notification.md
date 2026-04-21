---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-04"
capability: "CAP-005"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "8bd996e"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.014: notifications/tools/list_changed on Config Reload or Server Startup

## Description

When a configuration reload changes the resolved set of available tools (write tools added
or removed due to capability changes), Prism sends an MCP `notifications/tools/list_changed`
notification to the connected client. The notification is only sent when the tool set actually
changes — a reload that produces the same capability set sends no notification.

There is no session-level "client context switch" concept. The server is stateless. The
`notifications/tools/list_changed` notification is triggered exclusively by configuration
changes (SIGHUP, file change detection), not by the `client_id` used in individual tool
calls.

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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.014.

| Scenario | Config Change | Notification Sent? |
|----------|-------------|-------------------|
| Write tool enabled on reload | Add `sensor.crowdstrike.containment: Allow` | Yes |
| Write tool disabled on reload | Remove write capability | Yes |
| Reload with no capability change | Same TOML, no effective delta | No |
| New client with write capabilities | Add `[clients.new_client]` with write flag | Yes |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify `notifications/tools/list_changed` emission. Placeholder for future VP covering the notification is only sent on actual tool set change.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Addresses | ADV-1-001, ADV-2-003 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended ## Changelog row. |
| 1.1 | Phase 1 | 2026-04-14 | product-owner | Previous version |
