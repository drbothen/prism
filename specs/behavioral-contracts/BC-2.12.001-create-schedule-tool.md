---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Scheduled Queries & Differential Results"
capability: "CAP-017"
---

# BC-2.12.001: `create_schedule` MCP Tool — Create a Scheduled Query

## Preconditions
- The `create_schedule` MCP tool is invoked with required parameters: `name` (unique identifier, `[a-z0-9_-]{1,64}`), `query` (AxiQL query string), `interval` (seconds, minimum 60, maximum 86400)
- Optional parameters: `clients` (array of client IDs or null for all), `sensors` (array), `sources` (array), `splay_percent` (0-25, default 10), `snapshot` (boolean, default false -- if true emit full results, not differential), `removed` (boolean, default true -- include removed rows in differential), `enabled` (boolean, default true)
- The `schedule.write` capability is allowed for the invoking context
- The AxiQL query string passes parsing and security limit validation (BC-2.11.006)

## Postconditions
- A new `ScheduledQuery` record is persisted to the RocksDB `schedules` domain (BC-2.12.010)
- The `splayed_interval` is computed as `interval + (interval * splay_percent / 100) * hash(client_id, name)` per client, spreading API load
- The splay offset per (name, client_id) pair is persisted for deterministic scheduling across restarts
- `next_run` is computed as `now + splayed_interval` for each targeted client
- `epoch` counter is initialized to 0
- The schedule is registered with the execution loop (BC-2.12.004) and begins ticking on the next loop iteration
- An audit entry is emitted for the tool invocation (DI-004)
- Response includes: `schedule_id`, `name`, `interval`, `splay_percent`, `splayed_intervals` (map of client_id to actual interval), `next_run` (map of client_id to next execution time), `enabled`
- The `create_schedule` tool is gated by `schedule.write` capability and follows the hidden-tools pattern (BC-2.04.005)

## Invariants
- DI-004: Audit completeness -- exactly one AuditEntry emitted
- DI-008: Client data separation -- schedule scoping respects client boundaries
- DI-019: Query security limits enforced on the AxiQL query string at creation time

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SCHED-001` | Schedule `name` already exists | Structured error with existing schedule details; use `delete_schedule` + `create_schedule` to replace |
| `E-SCHED-002` | `interval` < 60 or > 86400 | Structured error with valid range |
| `E-SCHED-003` | `splay_percent` > 25 | Structured error; splay capped at 25% to prevent excessive drift |
| `E-QUERY-001` | AxiQL query string cannot be parsed | Structured error with position and suggestion |
| `E-CAP-001` | `schedule.write` capability denied | Structured error (BC-2.04.015) |
| `E-SCHED-004` | `name` contains invalid characters | Structured error with allowed character set |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-001 | `clients: null` with 50+ configured clients | Schedule created for all clients; splay spreads executions across the interval to avoid thundering herd |
| EC-12-002 | Query references a sensor not available for some targeted clients | Schedule created; unavailable sensors produce `sensor_errors` at execution time, not at creation time |
| EC-12-003 | `interval: 60` with `splay_percent: 25` | Splayed intervals range from 60-75 seconds depending on client_id hash |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 |
| L2 Invariants | DI-004, DI-008, DI-019 |
| Priority | P0 |
