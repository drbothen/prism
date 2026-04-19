---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-10"
capability: "CAP-034"
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

# BC-2.10.009: MCP Prompts for Common Workflows

## Preconditions
- MCP prompts are registered in `prompts/list`
- Prompts provide pre-built conversation starters for common analyst workflows

## Postconditions
- The following prompts are registered (at minimum):
  - `triage_alerts`: "Triage open alerts for a client" -- guides the agent through checking all sensors for open high/critical alerts
  - `investigate_host`: "Investigate a specific host across all sensors" -- guides cross-sensor correlation by hostname or IP
  - `client_overview`: "Security posture overview for a client" -- guides pulling alert counts, health status, and recent activity
  - `cross_client_status`: "Cross-client security status" -- guides checking all clients for critical alerts
- Each prompt includes:
  - `name`: snake_case identifier
  - `description`: one-line summary of the workflow
  - `arguments`: parameterized inputs (e.g., `client_id`, `hostname`, `time_range`)
- Prompt messages include security reminders about untrusted sensor data
- Prompts are static (defined at build time, not generated dynamically)

## Invariants
- DI-006: Prompts include reminders to treat sensor data as untrusted

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Prompt not found | Invalid prompt name | MCP error: "Prompt '{name}' not found" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-016 | Prompt references a sensor not configured for the specified client | The prompt generates tool calls; the tool handles the "sensor not configured" case normally |
| EC-10-017 | Prompt argument `client_id` is null | Prompt operates in cross-client mode where applicable |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| L2 Invariants | DI-006 |
| Priority | P1 |
