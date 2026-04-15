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
capability: "CAP-019"
---

# BC-2.15.004: Audit Buffer Overflow — Purge Oldest Entries When Exceeding 100K, Log Warning

## Preconditions
- The RocksDB `audit_buffer` column family contains entries that have not yet been forwarded
- The entry count approaches or exceeds the configured maximum (default: 100,000 entries)

## Postconditions
- The buffer forwarder background task monitors buffer size during each forward cycle
- When the buffer exceeds 100K entries:
  1. A warning is emitted to stderr: `"Audit buffer overflow: {count} entries exceed 100K limit. Purging oldest {purge_count} entries."`
  2. The oldest entries (by timestamp key) are purged until the buffer is at 90K entries (10% headroom)
  3. A special audit entry is written recording the purge event: `{ event: "audit_buffer_purge", purged_count: N, oldest_purged_timestamp: T, newest_purged_timestamp: T }`
- The purge operation uses `remove_range` (BC-2.15.002) for efficient batch deletion
- Purged entries are lost -- they were not successfully forwarded to any sink
- The overflow threshold is configurable via `audit.buffer_max` in TOML (default 100,000; minimum 1,000)

## Invariants
- Buffer size is bounded: unbounded growth cannot exhaust disk space
- Purge preserves the newest entries (most recent audit data is higher value)
- The purge event itself is audit-logged (meta-audit)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-AUDIT-004` | Purge operation fails (RocksDB error) | Critical warning; buffer continues growing; next purge cycle retries |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-014 | Buffer reaches 100K with all entries from the last 5 minutes (extremely high tool invocation rate) | Purge proceeds; warning indicates rapid accumulation suggesting Vector endpoint is down |
| EC-15-015 | `audit.buffer_max` set to 1000 (minimum) | Aggressive purging under normal load; warning recommends increasing the limit |
| EC-15-016 | Buffer at exactly 100K entries | Purge triggered on the next insert that would exceed the limit |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-019 |
| L2 Invariants | DI-004 |
| Priority | P1 |
