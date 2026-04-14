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

# BC-2.08.005: Health Check MCP Tool

## Preconditions
- The `check_sensor_health` MCP tool is registered in `tools/list`
- The tool accepts `client_id: String` (required) and `sensor_id: Option<SensorId>` (optional -- null means all sensors for client)

## Postconditions
- When `sensor_id` is provided: returns health status for that single sensor
- When `sensor_id` is null: returns health status for all configured sensors for the client
- Each sensor health entry contains: `sensor_id`, `reachable`, `auth_valid`, `rate_limit`, `last_successful_query_at`
- Response uses `structuredContent` for machine-parseable health data
- Response includes `content[].text` prose summary (e.g., "2 of 3 sensors healthy for client 'acme'")
- Response metadata includes `trust_level: "internal"` (health data is Prism-internal, not sensor-sourced)
- Tool annotations: `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: true`

## Invariants
- DI-004: Audit completeness -- exactly one AuditEntry emitted per tool invocation
- DI-008: Client data separation -- only the specified client's sensors are checked

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Invalid `client_id` format | Structured error with validation details |
| `PrismError::Config` | `client_id` not found in config | Structured error with suggestion to check config |
| `PrismError::InvalidInput` | Invalid `sensor_id` value | Structured error listing valid sensor IDs: CrowdStrike, Cyberint, Claroty, Armis |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-004 | Client has zero sensors configured | Returns empty health array with message "Client '{id}' has no sensors configured" |
| EC-08-010 | One sensor healthy, another unreachable | Returns partial health results; does not fail the entire tool call |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P1 |
