---
document_type: behavioral-contract
level: L3
version: "1.8"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-12"
capability: "CAP-017"
lifecycle_status: active
introduced: cycle-1
modified: 2026-05-03
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "412c872"
traces_to: ["CAP-017"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.12.004: Schedule Execution Loop — Tick-Based with Splay and In-Flight Skip

> **Supersedes note:** Earlier draft used a 1-second tick interval and a 16-permit shared
> semaphore (pre-D-209). Current spec reflects D-209 LOCKED 8-permit independent
> `schedule_executor_semaphore` (module-private to schedule executor; NOT shared with
> action delivery) and ADR-013 §2.1 60-second default tick.

## Description

The schedule execution loop runs as a background async task ticking every **60 seconds by
default** (configurable via `PRISM_SCHEDULER_TICK_SECS`, valid range 10–3600 s; per
ADR-013 §2.1). Each tick evaluates all enabled schedules against their `next_run_at` timestamp; due
schedules are dispatched as a single fire that internally iterates over the schedule's
`client_scope` per ADR-013 §2.5/§2.6. Dispatch is bounded by an **8-permit
`schedule_executor_semaphore`** (module-private to the schedule executor; per ADR-013 §2.3
and D-209). If the semaphore is exhausted, the schedule emits E-SCHED-004 and waits until
the next tick (DI-032). Concurrent executions for the same `(OrgId, ScheduleId)` are
blocked by an in-flight guard. On completion, differential results are computed, detection
engine is invoked, epoch is updated, and next_run_at is scheduled. Time drift compensation
prevents unbounded lag when queries run long.

## Preconditions
- The Prism server has started and at least one enabled schedule exists
- The RocksDB `schedules` domain is initialized and readable (BC-2.12.010)
- The execution loop background task has been spawned

## Postconditions
- The execution loop runs on a **60-second default tick interval** (configurable via
  `PRISM_SCHEDULER_TICK_SECS` env var, range 10–3600 s; per ADR-013 §2.1)
- On each tick, the loop iterates all enabled schedules and checks: `now >= next_run_at` per `(OrgId, ScheduleId)` per ADR-013 §2.5
- For each due schedule:
  - If the **8-permit `schedule_executor_semaphore`** permit cannot be acquired via
    `try_acquire()` (8 permits already held; per ADR-013 §2.3 + D-209): emit E-SCHED-004,
    log at WARN, skip this tick without incrementing epoch/counter; schedule retries at
    next tick (DI-032)
  - If a previous execution for this `(OrgId, ScheduleId)` is still in-flight: skip this tick (no concurrent executions for the same schedule)
  - Otherwise: acquire a semaphore permit and spawn an async task that fires once, internally iterating over the schedule's `client_scope` (per ADR-013 §2.6), executing the PrismQL query per client via the query engine (BC-2.11.001)
  - On completion: compute differential results (BC-2.12.005), increment epoch (BC-2.12.006), update `last_run`, compute `next_run_at = now + splayed_interval`, persist state (BC-2.12.010)
  - After differential computation completes for each `(OrgId, ScheduleId)` fire, the detection engine is invoked with DiffResults.added for single-event rules, and DiffResults.added is fed into correlation/sequence persistent state. This handoff is synchronous within the scheduler tick.
- Time drift compensation: if query execution takes longer than the interval, the next execution is scheduled relative to the intended time (not the completion time), up to a maximum drift of 60 seconds, after which drift is dropped and rescheduled from `now`
- The execution loop gracefully stops on SIGTERM/SIGINT (BC-2.10.010), allowing in-flight executions to complete within the shutdown grace period

## Invariants
- No concurrent executions for the same `(OrgId, ScheduleId)` (in-flight tracked per ADR-013 §2.5)
- Splay offsets are deterministic: the same `(OrgId, ScheduleId)` always produces the same splay offset across restarts
- Each execution produces exactly one audit entry
- The `schedule_executor_semaphore` is module-private to the schedule executor and is NOT
  shared with the action delivery subsystem (per D-209)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SCHED-004` | `schedule_executor_semaphore` permit cannot be acquired via `try_acquire()` (8 concurrent executions already in-flight; per ADR-013 §2.3 + D-209) | Execution skipped for this tick; logged at WARN level; epoch/counter NOT incremented; schedule retried at next tick (DI-032) |
| `E-SCHED-006` | Query execution fails for a client | Error recorded in schedule execution history; `last_run` updated with error status; schedule continues for other clients and future ticks |
| `E-SCHED-007` | Query exceeds watchdog limits (BC-2.15.007) | Execution terminated; error recorded; schedule remains active (not auto-disabled) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-009 | Server restarts mid-interval | On startup, `next_run` is loaded from persisted state; if `next_run` is in the past, execution fires on the next tick |
| EC-12-010 | Schedule with `interval: 60` (at default tick) and query taking 90 seconds | In-flight skip prevents overlap; next execution starts after completion; drift compensation adjusts timing; in-flight skip semantics generalize across the configurable [10, 3600]s tick range |
| EC-12-011 | 100+ schedules all due on the same tick | Executions are spawned as async tasks with bounded concurrency (max 8 concurrent schedule executions across all schedules per ADR-013 §2.3); schedules beyond the cap emit E-SCHED-004 and are skipped until the next tick (DI-032) |
| EC-12-012 | Client removed from config while schedule targets it | Execution for removed client silently skipped; schedule continues for remaining clients |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| Single schedule due on next tick | Query executes; diff computed; epoch incremented; next_run updated | happy-path |
| 8 concurrent executions in-flight when 9th schedule is due | 9th emits E-SCHED-004; retried next tick | error |
| Schedule due while same `(OrgId, ScheduleId)` is in-flight | In-flight skip; no duplicate execution | edge-case |
| Server restart with past next_run value | Execution fires on next tick after restart | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-026 | Splay computation: deterministic per (query, client) | kani |
| VP-137 | Per-subsystem semaphore liveness — schedule executor concurrency cap holds at 8 permits (ADR-013 §2.5 / D-209 SS-12 = 8/8); anchored via S-4.01 task 8 (semaphore acquire/release) | Proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 |
| L2 Invariants | DI-004, DI-008, DI-022, DI-032 |
| Priority | P0 |

## Phase 4.A Pass 6 Remediation Notes

**Adversary finding:** HIGH-001 (Pass 6) — BC body specified 1-second tick and 16-permit
shared semaphore, contradicting locked ADR-013 §2.1, ADR-013 §2.3, and D-209.

**Changes made (2026-05-02):**
- Tick interval corrected: 1 second → **60-second default**, configurable via
  `PRISM_SCHEDULER_TICK_SECS` (range 10–3600 s) per ADR-013 §2.1
- Semaphore corrected: 16-permit global → **8-permit `schedule_executor_semaphore`**,
  module-private to schedule executor, NOT shared with action delivery per ADR-013 §2.3
  and D-209
- Updated: Description, all Postconditions, Invariants, Error Cases table, Edge Cases
  EC-12-011, Canonical Test Vectors (16→8 in test vector row)
- Added supersedes note at top of body

## Phase 4.A Pass 12 Remediation Notes

v1.6 (P12 fix): Fire-loop iteration model aligned to ADR-013 §2.5/§2.6 — single fire per schedule with internal client iteration; in-flight tracked by (OrgId, ScheduleId), not (ScheduleId, ClientId) (F-P12-M-001).

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.8 | wave4-pass14-surgical | 2026-05-03 | product-owner | F-P14-H-002: corrected future-dated `modified` frontmatter field + v1.7 changelog row date (2026-05-04 → 2026-05-03; Pass 13 burst was 2026-05-03 per sibling Wave 4 BC dates). |
| 1.7 | wave4-pass13-surgical | 2026-05-03 | product-owner | F-P13-M-003: added VP-137 (per-subsystem semaphore liveness) to Verification Properties table — was missing despite S-4.01 v1.10 anchoring VP-137 to BC-2.12.004 + VP-INDEX listing S-4.01 as VP-137 anchor story. Reverse traceability restored (POL-4). |
| 1.6 | wave4-pass12-surgical | 2026-05-03 | product-owner | P12 fix (F-P12-M-001): fire-loop iteration model aligned to ADR-013 §2.5/§2.6 — single fire per schedule with internal client iteration; in-flight tracked by (OrgId, ScheduleId), not (ScheduleId, ClientId). |
| 1.5 | wave4-pass7-surgical | 2026-05-02 | state-manager | P7-MEDIUM-002: set modified date to 2026-05-02. P7-LOW-002: EC-12-010 — added "(at default tick)" parenthetical and tick-range generalization note. |
| 1.4 | wave4-pass6-bc-sweep | 2026-05-02 | product-owner | Phase 4.A Pass 6 remediation (HIGH-001): corrected tick interval to 60s default (ADR-013 §2.1) and semaphore to 8-permit module-private schedule_executor_semaphore (ADR-013 §2.3 + D-209). |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
