---
document_type: behavioral-contract
level: L3
version: "1.4"
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

# BC-2.15.008: Query Denylisting — After N Consecutive Failures, Denylist with Manual Override

## Description

After a configurable number of consecutive denylist-eligible failures (default 3),
the offending query's SHA-256 hash is added to a denylist stored in the `watchdog`
RocksDB column family. A denylist-eligible failure is a watchdog termination —
query timeout (`E-QUERY-004`) or process-RSS kill (`E-WATCHDOG-002`), per
BC-2.15.007 — or a process crash detected via a CrashRecovery marker on restart
(BC-2.15.005). Per-query GreedyMemoryPool budget trips (`E-WATCHDOG-001`,
BC-2.11.006) and record-cap violations (`E-QUERY-005`) are query-at-fault
validation errors and do NOT feed the denylist counter (DI-027 division of labor,
error-taxonomy v1.68 architect adjudication D2). Denylisted queries are immediately
rejected without execution, preventing crash loops and runaway resource consumption.
Denylist entries auto-expire after 24 hours and can be bypassed on demand via
`force_execute: true`.

An intervening non-terminated execution of a previously failing query — one that
does not end in a watchdog termination or crash — resets its consecutive failure
counter to zero, allowing organic recovery without manual intervention.

## Preconditions
- A query has been terminated by the watchdog (BC-2.15.007: query timeout `E-QUERY-004`, or process-RSS kill `E-WATCHDOG-002`) or caused a process crash detected via a CrashRecovery marker on restart (BC-2.15.005)
- The RocksDB `watchdog` column family is initialized (BC-2.15.001)

## Postconditions
- Each denylist-eligible failure is recorded in the `watchdog` column family with the query hash (SHA-256 of the normalized query string) and a consecutive failure counter
- When a query hash accumulates **3 consecutive** watchdog terminations (query timeout `E-QUERY-004` or process-RSS kill `E-WATCHDOG-002`, per BC-2.15.007) or process crashes (detected via CrashRecovery markers on restart, BC-2.15.005) — threshold configurable via `watchdog.denylist_threshold`, default 3:
  - The query hash is added to the denylist with: `denylisted_at`, `reason` (`timeout` = query timeout `E-QUERY-004` / `rss_kill` = process-RSS kill `E-WATCHDOG-002` / `crash` = CrashRecovery-detected process crash), `failure_count`, `expiry` (configurable, default 86400 seconds / 24 hours)
  - Future attempts to execute a query matching this hash are immediately rejected with `E-QUERY-008`
- **Counter exclusions (DI-027):** per-query GreedyMemoryPool budget trips (`E-WATCHDOG-001`, BC-2.11.006) and record-cap violations (`E-QUERY-005`, DI-019 10K materialization limit) do NOT feed the denylist counter — they are query-at-fault validation errors, not watchdog terminations
- Denylist entries automatically expire after the configured duration
- **Manual override:** the `query` tool accepts a `force_execute: true` parameter that bypasses the denylist for a single execution (still subject to watchdog limits)
- An intervening non-terminated execution of the same query hash — one that does not end in a watchdog termination or crash, i.e., a successful run or a run failing outside the trigger set (e.g., `E-WATCHDOG-001` pool trip, `E-QUERY-005` record cap) — resets the consecutive failure counter to 0
- Denylist state is persisted in RocksDB and survives restarts

## Invariants
- Only consecutive denylist-eligible failures (watchdog termination — timeout or process-RSS kill — or crash) trigger denylisting; an intervening non-terminated execution resets the counter (DI-027)
- Per-query GreedyMemoryPool budget trips (`E-WATCHDOG-001`) and record-cap violations (`E-QUERY-005`) never feed the denylist counter (DI-027; error-taxonomy v1.68 D2 division of labor)
- Denylisted queries can always be overridden via `force_execute: true` (no permanent ban)
- Denylist entries expire automatically; no manual cleanup required

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-008` | Query matches a denylisted hash | Structured error (Message Format verbatim per error-taxonomy v1.69): "Query has been denylisted after {N} consecutive failures ({reason}). Denylist expires at {expiry}." Recovery guidance: modify the query to change its hash, clear the denylist via `watchdog_status`, or use `force_execute: true` to override. |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-028 | Query times out twice, succeeds on third attempt | Counter reset to 0; no denylisting |
| EC-15-029 | Query times out 3 times, denylisted, then executed with `force_execute: true` and succeeds | Denylist entry removed (success clears denylist for that hash) |
| EC-15-030 | Scheduled query is denylisted | Schedule execution skips the denylisted query with a warning; other scheduled queries continue; schedule is not disabled |
| EC-15-031 | Two different queries produce the same SHA-256 hash | Extremely improbable (collision); both affected by the same denylist entry |
| EC-15-032 | `watchdog.denylist_threshold: 1` | Denylisted on first denylist-eligible failure; aggressive but valid |
| EC-15-043 | Query trips the per-query GreedyMemoryPool budget (`E-WATCHDOG-001`) or the 10K record cap (`E-QUERY-005`) on 3 consecutive executions | Never denylisted — these failures are outside the trigger set (DI-027); the counter is not incremented, and as non-terminated executions they reset any prior watchdog-termination/crash streak |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — 3 failures | query watchdog-terminated (timeout `E-QUERY-004` or process-RSS kill `E-WATCHDOG-002`) 3 consecutive times | Denylisted with 24h expiry; next attempt returns E-QUERY-008 |
| Intervening success | timeout, success, timeout | Counter resets on success; third timeout does not denylist |
| Excluded reason — pool trip | timeout, `E-WATCHDOG-001` pool trip, timeout | Pool trip does not feed the counter and resets the streak (EC-15-043); no denylisting |
| force_execute override | denylisted query with `force_execute: true` | Query executes (still subject to watchdog); success clears denylist |
| Auto-expiry | denylist entry at t=0; attempt at t=86401s | Entry expired; query executes normally |
| threshold=1 | `watchdog.denylist_threshold: 1`; first denylist-eligible failure | Denylisted immediately |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Consecutive-only counter is structurally identical to VP-057 (crash-recovery denylist counter); proof shape would be a duplicate; denylist-survives-restart is a restart integration test; covered by integration tests in watchdog test suite. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 |
| Capability Anchor Justification | CAP-024 ("Resource Watchdog") per capabilities.md §CAP-024 — this BC specifies CAP-024's query-denylisting clause ("Query denylisting: if a query is terminated by the watchdog (timeout, or process-RSS kill — `E-WATCHDOG-002`) or causes a crash … on 3 consecutive runs … it is denylisted for 86400 seconds") |
| L2 Invariants | DI-027 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | QRY cascade P1-04 D2 companion sweep (review-2026-06-10 PO micro-burst; flagged by the QRY fix-burst STOP report, 2026-06-10) | 2026-06-10 | product-owner | Trigger set aligned to the ratified D2 watchdog division of labor (architect adjudication D2, `proposals/cache-envelope-adjudication-2026-06-10.md`; error-taxonomy v1.69; DI-027 invariants v1.7; DEC-033 edge-cases v1.2; BC-2.15.007 v1.5; capabilities v1.16; nfr-catalog v1.6). Pre-v1.4 trigger "3 consecutive timeouts or memory violations" with `reason` ∈ {timeout, memory, record_limit} — which under the implemented architecture spanned `E-QUERY-004` + `E-WATCHDOG-001` (pool) + `E-QUERY-005` — replaced by: 3 consecutive watchdog terminations (`timeout` = query timeout `E-QUERY-004`; `rss_kill` = process-RSS kill `E-WATCHDOG-002`) or crashes (`crash` = CrashRecovery marker, BC-2.15.005); per-query GreedyMemoryPool budget trips (`E-WATCHDOG-001`, BC-2.11.006) and record-cap violations (`E-QUERY-005`) explicitly excluded from the counter; intervening non-terminated execution (success or excluded failure) resets the counter to 0. EC-15-043 + "Excluded reason — pool trip" vector added (append-only). E-QUERY-008 Error Conditions row regularized to the taxonomy v1.69 verbatim Message Format (force_execute guidance moved to recovery guidance). Traceability: L2 Invariants corrected DI-019 → DI-027 (DI-027 names BC-2.15.008 as enforcer; this BC does not enforce DI-019's limits — the record cap is now excluded from the trigger set); Capability Anchor Justification row added (CAP-024 "Resource Watchdog"). Persistence, 86400s expiry, SHA-256 fingerprinting, `watchdog.denylist_threshold` default 3, and force_execute bypass preserved unchanged. NOTE: production wiring of this BC (`denylist::record_failure` has no production caller; adjacent finding #2 of the D2 adjudication) is tracked by the watchdog wiring story per 2026-06-10 review (story-writer verifying/creating the anchor in a parallel burst). |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
