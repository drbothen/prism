---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-12"
capability: "CAP-017"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "365fb25"
traces_to: ["CAP-017"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.12.004: Schedule Execution Loop — Tick-Based with Splay and In-Flight Skip

## Description

The schedule execution loop runs as a background async task ticking every 1 second. Each tick evaluates all enabled (schedule, client_id) pairs against their `next_run` timestamp; due pairs are dispatched as async tasks bounded by a 16-permit global semaphore. If the semaphore is exhausted, the pair emits E-SCHED-004 and waits until the next tick (DI-032). Concurrent executions for the same (schedule, client_id) pair are blocked by an in-flight guard. On completion, differential results are computed, detection engine is invoked, epoch is updated, and next_run is scheduled. Time drift compensation prevents unbounded lag when queries run long.

## Preconditions
- The Prism server has started and at least one enabled schedule exists
- The RocksDB `schedules` domain is initialized and readable (BC-2.12.010)
- The execution loop background task has been spawned

## Postconditions
- The execution loop runs on a 1-second tick interval
- On each tick, the loop iterates all enabled schedules and checks: `now >= next_run[client_id]` for each (schedule, client_id) pair
- For each due (schedule, client_id) pair:
  - If the global concurrent-execution semaphore permit cannot be acquired via `try_acquire()` (16 permits already held): emit E-SCHED-004, log at WARN, skip this tick without incrementing epoch/counter; schedule retries at next tick (DI-032)
  - If a previous execution for this (schedule, client_id) is still in-flight: skip this tick (no concurrent executions for the same schedule+client)
  - Otherwise: acquire a semaphore permit and spawn an async task that executes the schedule's PrismQL query scoped to the single client via the query engine (BC-2.11.001)
  - On completion: compute differential results (BC-2.12.005), increment epoch (BC-2.12.006), update `last_run`, compute `next_run = now + splayed_interval`, persist state (BC-2.12.010)
  - After differential computation completes for a (schedule_name, client_id) pair, the detection engine is invoked with DiffResults.added for single-event rules, and DiffResults.added is fed into correlation/sequence persistent state. This handoff is synchronous within the scheduler tick.
- Time drift compensation: if query execution takes longer than the interval, the next execution is scheduled relative to the intended time (not the completion time), up to a maximum drift of 60 seconds, after which drift is dropped and rescheduled from `now`
- The execution loop gracefully stops on SIGTERM/SIGINT (BC-2.10.010), allowing in-flight executions to complete within the shutdown grace period

## Invariants
- No concurrent executions for the same (schedule, client_id) pair
- Splay offsets are deterministic: the same (name, client_id) pair always produces the same splay offset across restarts
- Each execution produces exactly one audit entry

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SCHED-004` | Global semaphore permit cannot be acquired via `try_acquire()` (16 concurrent executions already in-flight) | Execution skipped for this tick; logged at WARN level; epoch/counter NOT incremented; schedule retried at next tick (DI-032) |
| `E-SCHED-006` | Query execution fails for a client | Error recorded in schedule execution history; `last_run` updated with error status; schedule continues for other clients and future ticks |
| `E-SCHED-007` | Query exceeds watchdog limits (BC-2.15.007) | Execution terminated; error recorded; schedule remains active (not auto-disabled) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-009 | Server restarts mid-interval | On startup, `next_run` is loaded from persisted state; if `next_run` is in the past, execution fires on the next tick |
| EC-12-010 | Schedule with `interval: 60` and query taking 90 seconds | In-flight skip prevents overlap; next execution starts after completion; drift compensation adjusts timing |
| EC-12-011 | 100+ schedules all due on the same tick | Executions are spawned as async tasks with bounded concurrency (max 16 concurrent schedule executions across all schedules); schedules beyond the cap emit E-SCHED-004 and are skipped until the next tick (DI-032) |
| EC-12-012 | Client removed from config while schedule targets it | Execution for removed client silently skipped; schedule continues for remaining clients |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| Single schedule due on next tick | Query executes; diff computed; epoch incremented; next_run updated | happy-path |
| 16 concurrent executions in-flight when 17th schedule is due | 17th emits E-SCHED-004; retried next tick | error |
| Schedule due while same (schedule, client_id) is in-flight | In-flight skip; no duplicate execution | edge-case |
| Server restart with past next_run value | Execution fires on next tick after restart | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-026 | Splay computation: deterministic per (query, client) | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 |
| L2 Invariants | DI-004, DI-008, DI-022, DI-032 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
