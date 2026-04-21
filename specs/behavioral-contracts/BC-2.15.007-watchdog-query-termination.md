---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-15"
capability: "CAP-024"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "97d8ce0"
traces_to:
  - "CAP-024"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.15.007: Watchdog Query Termination — Kill Query Exceeding Limits, Return Structured Error

## Description

While a query executes, the watchdog monitors it for three resource violations: timeout,
per-query memory budget, and materialization record count. On any violation the query's
DataFusion SessionContext is cancelled via CancellationToken, all in-flight sensor API
calls are aborted, materialized data is dropped, and a structured error is returned to
the caller. Partial results are never returned.

A grace period prevents false-positive memory termination: a single spike above the
per-query limit does not immediately kill the query; the watchdog must see the limit
exceeded on two consecutive checks before acting (DI-027). The terminated query is
recorded in watchdog state for denylist evaluation by BC-2.15.008.

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
    - **Memory:** `E-WATCHDOG-001` with `memory_limit_mb`, `estimated_usage_mb`, and suggestions to reduce scope
    - **Record count:** `E-QUERY-006` with `record_limit`, `estimated_count`, and narrowing suggestions (existing from BC-2.11.001)
- **Memory grace period (DI-027):** A single spike above the per-query memory limit does not immediately terminate the query. The watchdog must observe the limit exceeded on two consecutive checks before terminating, to avoid killing queries that briefly spike during Arrow RecordBatch materialization.
- The terminated query is recorded in watchdog state for denylist evaluation (BC-2.15.008)
- An audit entry is emitted with the violation type and query details

## Invariants
- Query termination is non-negotiable: no query can exceed the configured limits
- Termination is clean: no leaked resources (SessionContext, RecordBatches, HTTP connections)
- Partial results are never returned on termination (all-or-nothing)

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-004` | Query timeout | Structured error with timeout details |
| `E-QUERY-006` | Materialization record limit exceeded | Structured error with count details |
| `E-WATCHDOG-001` | Query memory budget exceeded | Structured error with memory details |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-024 | Query at 29.9s on a 30s timeout; last sensor API call returns at 30.1s | Query terminated at 30s; partial API response discarded |
| EC-15-025 | Scheduled query terminated by watchdog | Schedule continues; failure recorded; next execution proceeds normally |
| EC-15-026 | Multiple queries executing concurrently; one exceeds memory | Only the offending query is terminated; others continue |
| EC-15-027 | Process RSS approaches process-level memory limit (512MB) | Process-level RSS guard triggers process exit at 512MB (separate from per-query termination). Per-query memory is estimated from RecordBatch sizes, not from process RSS. The process-level RSS guard is a last-resort safety net that protects against aggregate memory growth across concurrent queries, leaked allocations, or non-query memory growth. |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — query completes | query finishes within all limits | No termination; results returned |
| Timeout | query exceeds 30s | `E-QUERY-004` with elapsed_seconds; audit entry; recorded for denylist |
| Memory grace period | single spike above limit | Not terminated on first check; terminated on second consecutive check |
| Record limit | materialized records exceed 10K | `E-QUERY-006`; partial results dropped |
| Concurrent queries | query A exceeds memory; query B running | A terminated; B continues unaffected |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-058 | Watchdog memory grace period: single check does not terminate; two consecutive checks do — `should_terminate_for_memory(state)` returns true iff `state.consecutive_over_limit >= 2`; a single check with memory above limit does not terminate (returns false); two consecutive checks above limit do terminate (returns true); the threshold is exactly 2 checks. Method: Proptest. Priority: P0. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 |
| L2 Invariants | DI-004, DI-019, DI-027 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
