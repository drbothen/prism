---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Platform Infrastructure"
capability: "CAP-024"
---

# BC-2.15.008: Query Denylisting — After N Consecutive Failures, Denylist with Manual Override

## Preconditions
- A query has been terminated by the watchdog (BC-2.15.007) or caused a process-level error
- The RocksDB `watchdog` column family is initialized (BC-2.15.001)

## Postconditions
- Each query termination is recorded in the `watchdog` column family with the query hash (SHA-256 of the normalized query string) and a failure counter
- When a query hash accumulates **3 consecutive** timeouts or memory violations (configurable via `watchdog.denylist_threshold`, default 3):
  - The query hash is added to the denylist with: `denylisted_at`, `reason` (timeout/memory/record_limit), `failure_count`, `expiry` (configurable, default 86400 seconds / 24 hours)
  - Future attempts to execute a query matching this hash are immediately rejected with `E-QUERY-008`
- Denylist entries automatically expire after the configured duration
- **Manual override:** the `query` tool accepts a `force_execute: true` parameter that bypasses the denylist for a single execution (still subject to watchdog limits)
- A non-terminated execution of the same query hash resets the consecutive failure counter to 0
- Denylist state is persisted in RocksDB and survives restarts

## Invariants
- Only consecutive failures trigger denylisting; an intervening success resets the counter
- Denylisted queries can always be overridden via `force_execute: true` (no permanent ban)
- Denylist entries expire automatically; no manual cleanup required

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-008` | Query matches a denylisted hash | Structured error: "Query has been denylisted after {N} consecutive failures ({reason}). Denylist expires at {expiry}. Use force_execute: true to override." |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-028 | Query times out twice, succeeds on third attempt | Counter reset to 0; no denylisting |
| EC-15-029 | Query times out 3 times, denylisted, then executed with `force_execute: true` and succeeds | Denylist entry removed (success clears denylist for that hash) |
| EC-15-030 | Scheduled query is denylisted | Schedule execution skips the denylisted query with a warning; other scheduled queries continue; schedule is not disabled |
| EC-15-031 | Two different queries produce the same SHA-256 hash | Extremely improbable (collision); both affected by the same denylist entry |
| EC-15-032 | `watchdog.denylist_threshold: 1` | Denylisted on first failure; aggressive but valid |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 |
| L2 Invariants | DI-019 |
| Priority | P0 |
