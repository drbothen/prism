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

# BC-2.12.005: Differential Result Computation — Hash Previous Results, Return Added/Removed

## Preconditions
- A scheduled query execution has completed for a (schedule, client_id) pair
- The current result set is an array of OCSF-normalized records (Arrow RecordBatch)
- Previous results (if any) are available from the RocksDB `diff_results` domain, keyed by (schedule_name, client_id, epoch - 1)

## Postconditions
- If this is the first execution (epoch == 0) or `snapshot: true`: all current results are emitted as "added" with no "removed" entries
- If previous results exist and `snapshot: false`:
  - Each record is hashed (SHA-256 of all column values concatenated in schema order) to produce a fingerprint set
  - `added` = records in current fingerprint set but not in previous fingerprint set
  - `removed` = records in previous fingerprint set but not in current fingerprint set (only if `removed: true` on the schedule)
  - If both sets are identical (no added, no removed): no result is emitted and no notification is sent (silent skip)
- The `DiffResults` structure contains: `added` (array of OCSF records), `removed` (array of OCSF records), `schedule_name`, `client_id`, `epoch`, `timestamp`, `query_execution_time_ms`
- Current result fingerprints are persisted to RocksDB as the new "previous" for the next epoch
- Differential results are persisted to the `diff_results` domain for retrieval via `get_diff_results` (BC-2.12.007)

## Invariants
- Record hashing is deterministic: the same record always produces the same fingerprint regardless of column iteration order (columns sorted by schema-defined order)
- No silent data loss: if differential computation fails (e.g., schema change between epochs), the full current result set is emitted as a snapshot with a warning annotation

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-DIFF-001` | Previous result schema differs from current (schema evolution) | Fallback to snapshot mode for this epoch; emit all current results as "added" with warning; next epoch resumes differential |
| `E-DIFF-002` | RocksDB read failure for previous fingerprints | Fallback to snapshot mode with warning |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-013 | Current results are empty but previous had records | All previous records emitted as "removed" (if `removed: true`) |
| EC-12-014 | Identical results across 10 consecutive epochs | No diff results emitted for any of those epochs; epoch counter still increments |
| EC-12-015 | Record contains `null` values in some columns | Null values are included in hash computation as a sentinel byte (0x00); two records differing only in null vs. non-null are distinct |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 |
| L2 Invariants | DI-008 |
| Priority | P0 |
