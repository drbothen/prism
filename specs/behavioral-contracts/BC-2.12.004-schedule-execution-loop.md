---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Scheduler"
capability: "CAP-017"
---

# BC-2.12.004: Schedule Execution Loop — Tick-Based with Splay and In-Flight Skip

## Preconditions
- The Prism server has started and at least one enabled schedule exists
- The RocksDB `schedules` domain is initialized and readable (BC-2.12.010)
- The execution loop background task has been spawned

## Postconditions
- The execution loop runs on a 1-second tick interval
- On each tick, the loop iterates all enabled schedules and checks: `now >= next_run[client_id]` for each (schedule, client_id) pair
- For each due (schedule, client_id) pair:
  - If a previous execution for this (schedule, client_id) is still in-flight: skip this tick (no concurrent executions for the same schedule+client)
  - Otherwise: spawn an async task that executes the schedule's PrismQL query scoped to the single client via the query engine (BC-2.11.001)
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
| `E-SCHED-006` | Query execution fails for a client | Error recorded in schedule execution history; `last_run` updated with error status; schedule continues for other clients and future ticks |
| `E-SCHED-007` | Query exceeds watchdog limits (BC-2.15.007) | Execution terminated; error recorded; schedule remains active (not auto-disabled) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-009 | Server restarts mid-interval | On startup, `next_run` is loaded from persisted state; if `next_run` is in the past, execution fires on the next tick |
| EC-12-010 | Schedule with `interval: 60` and query taking 90 seconds | In-flight skip prevents overlap; next execution starts after completion; drift compensation adjusts timing |
| EC-12-011 | 100+ schedules all due on the same tick | Executions are spawned as async tasks with bounded concurrency (max 16 concurrent schedule executions across all schedules) |
| EC-12-012 | Client removed from config while schedule targets it | Execution for removed client silently skipped; schedule continues for remaining clients |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 |
| L2 Invariants | DI-004, DI-008, DI-022, DI-032 |
| Priority | P0 |
