---
document_type: behavioral-contract
level: L3
version: "1.0"
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
---

# BC-2.15.005: Crash Recovery Dirty Bits — Set Before Operation, Clear After, Detect on Restart

## Preconditions
- The RocksDB `dirty_bits` column family is initialized (BC-2.15.001)
- A query (ad-hoc or scheduled) is about to execute

## Postconditions
- **Before query execution:** a dirty bit is set in RocksDB with `sync: true`: key = `{query_hash}`, value = serialized `DirtyBitEntry { query_hash, query_source: AdHoc | Scheduled { schedule_name, client_id }, started_at, consecutive_crashes }` (matching data-layer.md DirtyBitEntry struct)
- **After successful query execution:** the dirty bit is cleared (key deleted)
- **On startup:** all remaining dirty bits are scanned from the `dirty_bits` column family (after schema version check per data-layer.md startup protocol)
- For each dirty bit found on startup:
  - `consecutive_crashes` is incremented
  - If `consecutive_crashes >= 3`: query_hash added to watchdog denylist (86400s expiry)
  - If source is `Scheduled`: log WARN ("schedule interrupted, will retry on next tick")
  - If source is `AdHoc`: log WARN only (ad-hoc queries are not retried)
  - After processing, dirty bit is cleared
- **Scope:** Dirty bits cover query execution only (ad-hoc + scheduled). Detection evaluation and case updates do not use dirty bits — detection state (correlation windows, sequence trackers) IS persisted to RocksDB, but the acceptable failure mode on crash is window reset (E-DETECT-004/006: "correlation windows reset to empty; warning logged; detection resumes from clean state"), making dirty bits redundant for detection. Case updates use RocksDB WriteBatch atomicity (no dirty bit needed).

## Invariants
- Dirty bit is always set before the operation begins and cleared after it completes
- Dirty bit write is a single RocksDB `put` (not part of the operation's WriteBatch) to ensure it's visible even if the operation's batch fails
- Startup recovery is idempotent: running recovery twice produces the same result

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STORE-009` | Dirty bit write fails | Query is aborted (fail-closed). Without a dirty bit, a crashing query cannot be denylisted on restart — the entire crash recovery safety mechanism is bypassed. The query is rejected with E-STORE-009 and the analyst is advised to investigate storage health. |
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
| Priority | P0 |
