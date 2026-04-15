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
capability: "CAP-017"
---

# BC-2.12.010: Schedule State Persistence — RocksDB Domain for Scheduling Metadata

## Preconditions
- The RocksDB database is initialized with the `schedules` and `diff_results` column families (BC-2.15.001)

## Postconditions
- The `schedules` column family stores:
  - Schedule definitions: key = `sched:{name}`, value = serialized ScheduledQuery struct
  - Splay offsets: key = `splay:{name}:{client_id}`, value = u64 splay offset in seconds
  - Epoch counters: key = `epoch:{name}:{client_id}`, value = u64 epoch
  - Global counter: key = `counter:global`, value = u64
  - Timing state: key = `timing:{name}:{client_id}`, value = serialized `{last_run, next_run}` timestamps
  - Result fingerprints: key = `fingerprints:{name}:{client_id}`, value = serialized HashSet of SHA-256 fingerprints from the most recent execution
- The `diff_results` column family stores:
  - Differential results: key = `diff:{name}:{client_id}:{epoch}`, value = serialized DiffResults (added/removed records)
  - Results are retained for `diff_retention_period` (configurable, default 7 days); expired results are purged during periodic cleanup
- All writes use RocksDB `WriteBatch` for atomicity within a single execution cycle
- On startup, all schedule state is loaded and the execution loop resumes from persisted `next_run` times

## Invariants
- Schedule state survives server restarts
- WriteBatch ensures epoch, timing, and fingerprints are updated atomically per execution
- Differential result retention is bounded; unbounded growth is prevented by periodic purge

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STORE-001` | RocksDB column family does not exist | Fatal startup error; database must be re-initialized |
| `E-STORE-002` | Deserialization failure for persisted state | Log error; affected schedule is disabled with warning; other schedules continue |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-028 | Database opened after schema migration adds new fields | Missing fields use default values; existing data is forward-compatible |
| EC-12-029 | Fingerprint set for a schedule exceeds 10K entries | Fingerprints are bounded by the 10K materialization cap (DI-019); no additional bounding needed |
| EC-12-030 | `diff_retention_period` set to 0 | No differential results are retained; `get_diff_results` always returns empty |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 |
| L2 Invariants | DI-008 |
| Priority | P1 |
