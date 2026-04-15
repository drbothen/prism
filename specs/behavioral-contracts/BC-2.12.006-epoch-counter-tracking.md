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
capability: "CAP-018"
---

# BC-2.12.006: Epoch/Counter Tracking — Exactly-Once Semantics, Persist to Storage After Each Run

## Preconditions
- A scheduled query execution has completed and differential results have been computed (BC-2.12.005)
- The RocksDB `schedules` domain is writable (BC-2.12.010)

## Postconditions
- The `epoch` counter for the (schedule_name, client_id) pair is atomically incremented by 1 after successful differential computation and result persistence
- The epoch is persisted to RocksDB before the execution is considered complete
- The `counter` field (monotonically increasing across all executions, not per-client) is also incremented and persisted, providing a global ordering of all schedule executions
- Every `DiffResults` entry is tagged with both `epoch` (per-schedule-per-client) and `counter` (global) for correlation
- On server restart, epoch and counter values are restored from RocksDB; no epoch is skipped or repeated

## Invariants
- Exactly-once semantics: each successful execution increments the epoch exactly once; a failed execution does not increment (dirty bit pattern from BC-2.15.005 detects incomplete executions)
- Epoch values are monotonically increasing per (schedule_name, client_id); gaps may exist if executions fail
- Counter values are monotonically increasing globally; no two executions share a counter value

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SCHED-008` | RocksDB write failure during epoch persistence | Execution marked as incomplete via dirty bit; on restart, the incomplete execution is detected and re-run (BC-2.15.005) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-016 | Server crashes after differential computation but before epoch increment | Dirty bit is set (BC-2.15.005); on restart, epoch is not incremented; re-execution produces the same differential results (idempotent) |
| EC-12-017 | Schedule deleted while epoch persistence is in progress | Epoch write completes (atomic); schedule deletion proceeds; orphaned epoch data is harmless |
| EC-12-018 | Epoch counter reaches u64::MAX | Practically impossible (18.4 quintillion executions); no rollover handling required |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 |
| L2 Invariants | DI-004 |
| Priority | P1 |
