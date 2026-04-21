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
input-hash: "3eb97f3"
traces_to:
  - "CAP-024"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.15.008: Query Denylisting — After N Consecutive Failures, Denylist with Manual Override

## Description

After a configurable number of consecutive watchdog-triggered terminations (default 3),
the offending query's SHA-256 hash is added to a denylist stored in the `watchdog`
RocksDB column family. Denylisted queries are immediately rejected without execution,
preventing crash loops and runaway resource consumption. Denylist entries auto-expire
after 24 hours and can be bypassed on demand via `force_execute: true`.

A single successful execution of a previously failing query resets its consecutive
failure counter to zero, allowing organic recovery without manual intervention.

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

## Error Conditions
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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — 3 failures | query terminated 3 consecutive times | Denylisted with 24h expiry; next attempt returns E-QUERY-008 |
| Intervening success | timeout, success, timeout | Counter resets on success; third timeout does not denylist |
| force_execute override | denylisted query with `force_execute: true` | Query executes (still subject to watchdog); success clears denylist |
| Auto-expiry | denylist entry at t=0; attempt at t=86401s | Entry expired; query executes normally |
| threshold=1 | `watchdog.denylist_threshold: 1`; first failure | Denylisted immediately |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Consecutive-only counter is structurally identical to VP-057 (crash-recovery denylist counter); proof shape would be a duplicate; denylist-survives-restart is a restart integration test; covered by integration tests in watchdog test suite. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 |
| L2 Invariants | DI-019 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
