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
capability: "CAP-025"
---

# BC-2.15.007: Watchdog Query Termination — Kill Query Exceeding Limits, Return Structured Error

## Preconditions
- A query (ad-hoc via `query` tool or scheduled via execution loop) is executing
- The resource watchdog is active and monitoring (BC-2.15.006)

## Postconditions
- The watchdog monitors each running query for three limit violations:
  1. **Timeout:** query execution time exceeds `query_timeout`
  2. **Memory:** query's materialized data exceeds `per_query_memory_budget` (estimated from RecordBatch sizes)
  3. **Record count:** materialization streaming counter exceeds `max_materialized_records`
- When a limit is violated:
  - The query's DataFusion SessionContext is cancelled (via CancellationToken)
  - All in-flight sensor API calls for this query are aborted
  - Materialized data is dropped
  - A structured error is returned to the caller:
    - **Timeout:** `E-QUERY-004` with `timeout_seconds`, `elapsed_seconds`, and narrowing suggestions
    - **Memory:** `E-QUERY-007` with `memory_limit_mb`, `estimated_usage_mb`, and suggestions to reduce scope
    - **Record count:** `E-QUERY-006` with `record_limit`, `estimated_count`, and narrowing suggestions (existing from BC-2.11.001)
- The terminated query is recorded in watchdog state for denylist evaluation (BC-2.15.008)
- An audit entry is emitted with the violation type and query details

## Invariants
- Query termination is non-negotiable: no query can exceed the configured limits
- Termination is clean: no leaked resources (SessionContext, RecordBatches, HTTP connections)
- Partial results are never returned on termination (all-or-nothing)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-004` | Query timeout | Structured error with timeout details |
| `E-QUERY-006` | Materialization record limit exceeded | Structured error with count details |
| `E-QUERY-007` | Query memory budget exceeded | Structured error with memory details |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-024 | Query at 29.9s on a 30s timeout; last sensor API call returns at 30.1s | Query terminated at 30s; partial API response discarded |
| EC-15-025 | Scheduled query terminated by watchdog | Schedule continues; failure recorded; next execution proceeds normally |
| EC-15-026 | Multiple queries executing concurrently; one exceeds memory | Only the offending query is terminated; others continue |
| EC-15-027 | Process RSS approaches process-level memory limit | Warning logged; individual queries are not pre-emptively killed (only per-query budget is enforced per-query; process-level limit triggers if aggregate exceeds) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-025 |
| L2 Invariants | DI-004, DI-019 |
| Priority | P0 |
