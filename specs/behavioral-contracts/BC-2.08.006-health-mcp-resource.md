---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-08"
capability: "CAP-008"
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

# BC-2.08.006: Health Status MCP Resource

## Preconditions
- The MCP resource `prism://health/{client_id}` is registered in `resources/list`
- The resource URI template accepts a `client_id` path parameter

## Postconditions
- Reading the resource returns the most recent health status for all sensors of the specified client
- The resource content is JSON with the same schema as the `check_sensor_health` tool response
- The resource reflects cached health data from the most recent health check or query attempt (not a fresh check)
- If no health data exists yet (no queries or checks performed), the resource returns an empty health status with `status: "unknown"` for each configured sensor

## Invariants
- DI-008: Client data separation -- resource returns data only for the specified client

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Resource not found | `client_id` not in config | MCP resource error with "Client not found" message |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-011 | Resource read immediately after startup, before any queries | All sensors report `status: "unknown"`, `reachable: null`, `auth_valid: null` |
| EC-08-012 | Health data is stale (last check was 10 minutes ago) | Resource includes `last_checked_at` timestamp so the consumer can assess freshness |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-008 |
| Priority | P1 |
