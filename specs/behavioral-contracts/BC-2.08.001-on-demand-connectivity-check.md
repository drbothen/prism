---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Health"
capability: "CAP-008"
---

# BC-2.08.001: On-Demand Connectivity Check Per Sensor Per Client

## Preconditions
- A valid `client_id` is provided in the health check tool call
- The target sensor is configured and `enabled: true` for the specified client
- The sensor adapter for the target sensor type is initialized

## Postconditions
- The `verify_connectivity()` method on the sensor adapter is invoked against the sensor's API endpoint
- The response includes `reachable: true` or `reachable: false` with a reason string
- The check completes within the sensor-specific timeout (default 30s)
- An AuditEntry is emitted for the health check invocation

## Invariants
- DI-008: Client data separation -- health check targets only the specified client's sensor instance
- DI-004: Audit completeness -- exactly one AuditEntry is emitted

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `client_id` fails validation | Structured error with rejected value and allowed pattern |
| `PrismError::Config` | Client or sensor not found in config | Structured error: "Sensor '{sensor}' not configured for client '{id}'" |
| `PrismError::Sensor` | HTTP connection refused or timed out | Returns health status `reachable: false` with reason, not a tool-level error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-001 | Sensor API returns HTTP 503 during health check | Health status reports `reachable: false`, `reason: "service_unavailable"` |
| EC-08-002 | Sensor is configured but `enabled: false` | Health check returns `status: "disabled"` without making any API call |
| EC-08-003 | Health check times out after 30s | Returns `reachable: false`, `reason: "timeout"`, `timeout_seconds: 30` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P1 |
