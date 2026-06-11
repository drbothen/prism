---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-15"
capability: "CAP-024"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "2580252"
traces_to:
  - "CAP-024"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.15.007: Watchdog Query Termination — Kill Query Exceeding Limits, Return Structured Error

## Description

While a query executes, the watchdog monitors it for three resource violations: timeout,
memory (process-RSS kill threshold), and materialization record count. On any violation
the query's DataFusion SessionContext is cancelled via CancellationToken, all in-flight
sensor API calls are aborted, materialized data is dropped, and a structured error is
returned to the caller. Partial results are never returned.

Division of labor for memory enforcement (taxonomy v1.68, architect adjudication D2):
the per-query 200MB memory budget is enforced by the DataFusion GreedyMemoryPool
(BC-2.11.006 → `E-WATCHDOG-001`); the watchdog's memory enforcement is the process-RSS
kill (→ `E-WATCHDOG-002`), consistent with EC-15-027.

A grace period prevents false-positive memory termination: a single spike above the
kill threshold does not immediately kill the query; the watchdog must see the threshold
exceeded on two consecutive checks before acting (DI-027). Queries terminated for
timeout or process-RSS kill are recorded in watchdog state for denylist evaluation by
BC-2.15.008; record-count termination (`E-QUERY-005`) is explicitly NOT
denylist-eligible — it is a query-at-fault validation error that, for denylist-counter
purposes, counts as a non-terminated execution and resets any prior
watchdog-termination/crash streak (EC-15-043, DI-027).

## Preconditions
- A query (ad-hoc via `query` tool or scheduled via execution loop) is executing
- The resource watchdog is active and monitoring (BC-2.15.006)

## Postconditions
- The watchdog monitors each running query for three limit violations:
  1. **Timeout:** query execution time exceeds `query_timeout`
  2. **Memory (process RSS):** process RSS exceeds the kill threshold (95% of the 512MB process budget). The per-query 200MB memory budget is NOT enforced by the watchdog — it is enforced by the DataFusion GreedyMemoryPool (BC-2.11.006 → `E-WATCHDOG-001`); the watchdog's memory enforcement is the process-RSS kill (→ `E-WATCHDOG-002`), consistent with EC-15-027
  3. **Record count:** materialization streaming counter exceeds `max_materialized_records`
- When a limit is violated:
  - The query's DataFusion SessionContext is cancelled (via CancellationToken)
  - All in-flight sensor API calls for this query are aborted
  - Materialized data is dropped
  - A structured error is returned to the caller:
    - **Timeout:** `E-QUERY-004` with `elapsed_ms` (display: `"E-QUERY-004: query timed out after {elapsed_ms}ms"`) and narrowing suggestions
    - **Memory (process-RSS kill):** `E-WATCHDOG-002` (display: `"E-WATCHDOG-002: watchdog killed query — process RSS exceeded kill threshold ({budget_bytes} bytes budget); query token cancelled"`) — retryable; the killed query is not necessarily at fault (process pressure from concurrent queries)
    - **Record count:** `E-QUERY-005` with `count`, `max` (display: `"E-QUERY-005: materialization limit exceeded: fetched {count} records (max {max})"`) and narrowing suggestions
- **Memory grace period (DI-027):** A single spike above the process-RSS kill threshold does not immediately terminate the query. The watchdog must observe the threshold exceeded on two consecutive checks before terminating, to avoid killing queries that briefly spike during Arrow RecordBatch materialization.
- **Denylist eligibility (DI-027 scope):** only timeout (`E-QUERY-004`) and process-RSS kill (`E-WATCHDOG-002`) terminations are recorded in watchdog state for denylist evaluation (BC-2.15.008). Record-count termination (`E-QUERY-005`) is explicitly NOT denylist-eligible — it does not feed the denylist counter and, for counter purposes, is treated as a non-terminated execution that resets any prior watchdog-termination/crash streak (EC-15-043)
- An audit entry is emitted with the violation type and query details

## Invariants
- Query termination is non-negotiable: no query can exceed the configured limits
- Termination is clean: no leaked resources (SessionContext, RecordBatches, HTTP connections)
- Partial results are never returned on termination (all-or-nothing)

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-004` | Query timeout | Structured error with `elapsed_ms` timeout details |
| `E-QUERY-005` | Materialization record limit exceeded (streaming counter; pre-v1.5 this row incorrectly cited `E-QUERY-006`, the pre-fetch scope estimate) | Structured error with `count`/`max` details |
| `E-WATCHDOG-002` | Watchdog kill — process RSS exceeded kill threshold | Structured retryable error with process-budget details; the killed query is not necessarily at fault. (Per-query memory-budget violations surface `E-WATCHDOG-001` via the DataFusion GreedyMemoryPool per BC-2.11.006, not via the watchdog kill path.) |

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
| Timeout | query exceeds 30s | `E-QUERY-004` with `elapsed_ms`; audit entry; recorded for denylist |
| Memory grace period | single process-RSS spike above kill threshold | Not terminated on first check; terminated on second consecutive check with `E-WATCHDOG-002`; recorded for denylist evaluation (BC-2.15.008) |
| Record limit | materialized records exceed 10K | `E-QUERY-005`; partial results dropped; NOT recorded for denylist evaluation — resets any prior watchdog-termination/crash streak (EC-15-043) |
| Concurrent queries | query A exceeds memory; query B running | A terminated; B continues unaffected |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-058 | Watchdog memory grace period: single check does not terminate; two consecutive checks do — `should_terminate_for_memory(state)` returns true iff `state.consecutive_over_limit >= 2`; a single check with memory above limit does not terminate (returns false); two consecutive checks above limit do terminate (returns true); the threshold is exactly 2 checks. Method: Proptest. Priority: P0. |
| VP-036 | SessionContext dropped before error propagation and on panic in execute_scheduled callers. Method: integration_test. Priority: P0. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 |
| L2 Invariants | DI-004, DI-019, DI-027 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.6 | QRY cascade P3-02 MED denylist-eligibility carve-out (review-2026-06-10 PO micro-burst) | 2026-06-10 | product-owner | Scoped the denylist-evaluation clause to the ratified BC-2.15.008 v1.4 trigger set (architect adjudication D2, `proposals/cache-envelope-adjudication-2026-06-10.md`; this BC's v1.5 residual unqualified framing caused a parallel story-writer burst to encode the retired trigger set — P3-02). Description ¶3 final sentence and the postcondition "recorded in watchdog state for denylist evaluation" qualified: only timeout (`E-QUERY-004`) and process-RSS kill (`E-WATCHDOG-002`) terminations are denylist-eligible; record-count termination (`E-QUERY-005`) is explicitly NOT denylist-eligible — for denylist-counter purposes it is treated as a non-terminated execution that resets any prior watchdog-termination/crash streak (EC-15-043, DI-027). Record-limit canonical vector annotated NOT-recorded/streak-reset; memory grace-period vector annotated recorded-for-denylist for symmetry with the timeout vector. Termination semantics of this BC unchanged: record-count violations still cancel the SessionContext, abort in-flight calls, drop materialized data, and return `E-QUERY-005` — the carve-out is denylist-eligibility only. Sibling sweep: timeout vector's "recorded for denylist" verified accurate (eligible class); no other unqualified denylist phrasing remains in this BC. |
| 1.5 | QRY cascade pass-1 P1-04 D2 companion sweep (review-2026-06-10 PO consolidated amendment burst) | 2026-06-10 | product-owner | Aligned to error-taxonomy v1.68 (architect adjudication D2, `proposals/cache-envelope-adjudication-2026-06-10.md`). Watchdog kill code corrected `E-WATCHDOG-001` → `E-WATCHDOG-002` with verbatim process-RSS display; division of labor restated: per-query 200MB memory budget is enforced by the DataFusion GreedyMemoryPool (BC-2.11.006 → E-WATCHDOG-001), the watchdog's memory enforcement is the process-RSS kill (→ E-WATCHDOG-002), consistent with EC-15-027. Record-count code corrected `E-QUERY-006` (pre-fetch scope estimate) → `E-QUERY-005` (streaming-counter materialization limit, taxonomy v1.67+); timeout fields aligned to shipped `QueryTimeout { elapsed_ms }` display. Description, Postconditions (monitored-limits list, structured-error bullets, DI-027 grace-period threshold = process RSS), Error Conditions rows, and Canonical Test Vectors updated coherently. |
| 1.4 | pass-85 F85-004 | 2026-04-21 | architect | Added VP-036 back-reference to VP Anchors to close bidirectional anchor asymmetry. |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
