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

# BC-2.08.004: Last Successful Query Timestamp Per Sensor Per Client

## Preconditions
- A valid `client_id` and `sensor_id` are provided
- The sensor adapter tracks the timestamp of its most recent successful API call

## Postconditions
- The health response includes `last_successful_query_at: Option<DateTime<Utc>>`
- If no successful query has been made during this session, the field is `null`
- The timestamp reflects the most recent HTTP 2xx response from the sensor API for this (client_id, sensor_id) pair
- The timestamp is stored in memory only (not persisted across restarts)

## Invariants
- DI-008: Client data separation -- timestamp is scoped to the specific (client_id, sensor_id) pair

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | No successful queries in this session | `last_successful_query_at: null` with `status: "no_successful_queries"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-008 | Prism just started, no queries executed yet | `last_successful_query_at: null`; health check still returns valid status for other fields |
| EC-08-009 | Most recent query succeeded but a subsequent health check runs minutes later | Timestamp reflects the query time, not the health check time |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-008 |
| Priority | P1 |
