---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Platform Infrastructure"
capability: "CAP-024"
---

# BC-2.15.005: Crash Recovery Dirty Bits — Set Before Operation, Clear After, Detect on Restart

## Preconditions
- The RocksDB `dirty_bits` column family is initialized (BC-2.15.001)
- An operation that requires crash recovery detection is about to execute (scheduled query execution, detection engine state flush, case update)

## Postconditions
- **Before operation:** a dirty bit is set in RocksDB: key = `dirty:{operation_type}:{operation_id}`, value = serialized `{ started_at, description, context }`
- **After successful operation:** the dirty bit is cleared (key deleted)
- **On startup:** all remaining dirty bits are scanned from the `dirty_bits` column family
- For each dirty bit found on startup:
  - The operation is identified by its type and ID
  - A warning is logged: `"Incomplete operation detected: {operation_type}:{operation_id}, started at {started_at}. Initiating recovery."`
  - Recovery action depends on operation type:
    - **Scheduled query execution:** epoch is not incremented; re-execution fires on next tick (idempotent via BC-2.12.006)
    - **Detection state flush:** affected correlation windows/sequence trackers are reset to last known good state
    - **Case update:** case state is rolled back to pre-update (WriteBatch atomicity should prevent this; dirty bit is a belt-and-suspenders check)
  - After recovery, the dirty bit is cleared

## Invariants
- Dirty bit is always set before the operation begins and cleared after it completes
- Dirty bit write is a single RocksDB `put` (not part of the operation's WriteBatch) to ensure it's visible even if the operation's batch fails
- Startup recovery is idempotent: running recovery twice produces the same result

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STORE-009` | Dirty bit write fails | Operation proceeds with warning: "Crash recovery disabled for this operation" |
| `E-STORE-010` | Recovery action fails on startup | Warning logged; dirty bit is NOT cleared; recovery retried on next startup |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-017 | Clean shutdown (SIGTERM) | All in-flight operations complete; all dirty bits are cleared; startup finds zero dirty bits |
| EC-15-018 | Crash during dirty bit write itself | Dirty bit may or may not be set (RocksDB atomic write); at worst, one operation goes undetected |
| EC-15-019 | 100 dirty bits on startup (many concurrent operations crashed) | All processed sequentially; recovery may take several seconds |
| EC-15-020 | Dirty bit from a previous Prism version with unknown operation type | Warning logged; dirty bit cleared; no recovery attempted |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 |
| L2 Invariants | -- |
| Priority | P1 |
